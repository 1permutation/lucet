#![allow(dead_code)]
#![allow(unused_variables)]

use crate::backend::BackendConfig;
use crate::error::IDLError;
use crate::generator::Generator;
use crate::module::Module;
use crate::pretty_writer::PrettyWriter;
use crate::target::Target;
use crate::types::AtomType;
use crate::types::{DataType, DataTypeRef, FuncDecl, Ident, Named};
use heck::{CamelCase, SnakeCase};
use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, Debug)]
struct CTypeInfo<'t> {
    /// The native type name
    type_name: String,
    /// Alignment rules for that type
    type_align: usize,
    /// The native type size
    type_size: usize,
    /// The leaf type node
    leaf_data_type_ref: &'t DataTypeRef,
}

/// Generator for the C backend
pub struct RustGenerator {
    pub target: Target,
    pub backend_config: BackendConfig,
    pub defined: HashMap<Ident, String>,
    pub w: PrettyWriter,
}

impl RustGenerator {
    pub fn new(target: Target, backend_config: BackendConfig, w: Box<dyn Write>) -> Self {
        Self {
            target,
            backend_config,
            defined: HashMap::new(),
            w: PrettyWriter::new(w),
        }
    }

    fn define_name(&mut self, data_type_entry: &Named<DataType>) -> String {
        let typename = data_type_entry.name.name.to_camel_case();
        self.defined.insert(data_type_entry.id, typename.clone());
        typename
    }

    fn get_defined_name(&self, data_type_ref: &DataTypeRef) -> &str {
        match data_type_ref {
            DataTypeRef::Defined(id) => self.defined.get(id).expect("definition exists"),
            DataTypeRef::Atom(a) => Self::atom_name(a),
        }
    }

    fn atom_name(atom_type: &AtomType) -> &'static str {
        use AtomType::*;
        match atom_type {
            Bool => "bool",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            I8 => "i32",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            F32 => "f32",
            F64 => "f64",
        }
    }
}

impl Generator for RustGenerator {
    fn gen_type_header(
        &mut self,
        _module: &Module,
        data_type_entry: &Named<DataType>,
    ) -> Result<(), IDLError> {
        self.w.eob()?.write_line(
            format!("/// {}: {:?}", data_type_entry.name.name, data_type_entry).as_bytes(),
        )?;
        Ok(())
    }

    fn gen_alias(
        &mut self,
        module: &Module,
        data_type_entry: &Named<DataType>,
    ) -> Result<(), IDLError> {
        let (pointee, _attrs) =
            if let DataType::Alias { to: pointee, attrs } = &data_type_entry.entity {
                (pointee, attrs)
            } else {
                unreachable!()
            };

        let typename = self.define_name(data_type_entry);
        let pointee_name = self.get_defined_name(pointee);

        self.w
            .write_line(format!("type {} = {};", typename, pointee_name).as_bytes())?
            .eob()?;
        Ok(())
    }

    fn gen_struct(
        &mut self,
        module: &Module,
        data_type_entry: &Named<DataType>,
    ) -> Result<(), IDLError> {
        let (named_members, _attrs) = if let DataType::Struct {
            members: named_members,
            attrs,
        } = &data_type_entry.entity
        {
            (named_members, attrs)
        } else {
            unreachable!()
        };

        let typename = data_type_entry.name.name.to_camel_case();
        self.defined.insert(data_type_entry.id, typename.clone());

        self.w
            .write_line("#[repr(C)]".as_bytes())?
            .write_line(format!("struct {} {{", typename).as_bytes())?;

        let mut w = self.w.new_block();
        for m in named_members {
            w.write_line(
                format!(
                    "{}: {},",
                    m.name.to_snake_case(),
                    self.get_defined_name(&m.type_)
                )
                .as_bytes(),
            )?;
        }

        self.w.write_line("}".as_bytes())?.eob()?;
        Ok(())
    }

    // Enums generate both a specific typedef, and a traditional C-style enum
    // The typedef is required to use a native type which is consistent across all architectures
    fn gen_enum(
        &mut self,
        module: &Module,
        data_type_entry: &Named<DataType>,
    ) -> Result<(), IDLError> {
        let (named_members, _attrs) = if let DataType::Enum {
            members: named_members,
            attrs,
        } = &data_type_entry.entity
        {
            (named_members, attrs)
        } else {
            unreachable!()
        };

        let typename = data_type_entry.name.name.to_camel_case();
        self.defined.insert(data_type_entry.id, typename.clone());

        self.w
            .write_line("#[repr(C)]".as_bytes())?
            .write_line("#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]".as_bytes())?
            .write_line(format!("enum {} {{", typename).as_bytes())?;

        let mut w = self.w.new_block();
        for m in named_members {
            w.write_line(format!("{},", m.name.to_camel_case()).as_bytes())?;
        }

        self.w.write_line("}".as_bytes())?.eob()?;
        Ok(())
    }

    fn gen_function(
        &mut self,
        module: &Module,
        func_decl_entry: &Named<FuncDecl>,
    ) -> Result<(), IDLError> {
        unimplemented!();
    }
}
