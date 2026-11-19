//! A schema parser and generator for bounded codecs (data/codec/README.md, sections 3-6).
//! The schema and declaration tree are author-owned; this crate has no peer
//! input or running machine state. [`parse`] validates a schema before code
//! generation.

extern crate alloc;

use core::fmt::Write;

mod analysis;
mod check;
mod constructs;
pub mod diff;
mod emit_bounds;
mod emit_codec;
mod emit_functions;
mod emit_limits;
mod emit_tests;
mod emit_types;
mod largest;
mod lean;
mod parse;
pub mod patch;
mod values;

pub use constructs::{CONSTRUCTS, Construct, Kind, constructs, missing, unpinned};
pub use emit_limits::{pascal, snake};
pub use largest::{largest, largest_variants, roots};
pub use parse::{Error, parse};
pub use values::{Values, values};

/// One schema version of a family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub family: String,
    pub version: u16,
    pub declarations: Vec<Declaration>,
}

/// A record or enumeration in declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    Record(Record),
    Enum(Enumeration),
}

/// An ordered group of fields, optionally with a version prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub line: usize,
    pub name: String,
    pub versioned: bool,
    pub fields: Vec<Field>,
}

/// An ordered group of variants, each holding at most one earlier record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enumeration {
    pub line: usize,
    pub name: String,
    pub variants: Vec<Variant>,
}

/// A named field of a record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub line: usize,
    pub name: String,
    pub ty: Type,
}

/// A named variant with an optional record payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub line: usize,
    pub name: String,
    pub record: Option<String>,
    /// Whether the record is held in a box without changing its wire bytes.
    pub boxed: bool,
}

/// A field's bounded wire type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    Bool,
    Duration,
    Fixed(u32),
    Bytes(u32),
    Secret(u32),
    Text(u32),
    List(u32, Box<Type>),
    Option(Box<Type>),
    Named(String),
}

/// Generated Rust for one schema version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub records: String,
}

/// Frozen wire bytes for one fixed-rule value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Golden {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Fixed-rule encodings in declaration order (data/codec/README.md, section 6).
#[must_use]
pub fn goldens(schema: &Schema) -> Vec<Golden> {
    emit_tests::goldens(schema).into_iter().map(|(_, golden)| golden).collect()
}

fn uses_box(ty: &Type) -> bool {
    match ty {
        Type::Bytes(_) | Type::Secret(_) | Type::Text(_) => true,
        Type::List(_, item) | Type::Option(item) => uses_box(item),
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::Bool
        | Type::Duration
        | Type::Fixed(_)
        | Type::Named(_) => false,
    }
}

/// The owner's choice of generated record surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Sealed,
    Lean,
}

/// Reads the owner's `lean` mark; an absent mark leaves the family sealed.
pub fn form_of(schema_dir: &std::path::Path, family: &str) -> std::io::Result<Form> {
    let path = schema_dir.join("lean");
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Form::Sealed),
        Err(error) => return Err(error),
    };
    let mut form = Form::Sealed;
    for (index, line) in source.lines().enumerate() {
        if ["<<<<<<<", "|||||||", "=======", ">>>>>>>"].iter().any(|marker| line.starts_with(marker)) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "{}:{}: conflict marker in lean mark",
                    path.display(),
                    index.checked_add(1).expect("mark line fits usize")
                ),
            ));
        }
        if line.split('#').next().expect("a split has a first part").trim() == family {
            form = Form::Lean;
        }
    }
    Ok(form)
}

/// Checks the names added by a form, beyond the shared Rust checks in `parse`.
/// Sealed adds no check. Lean reserves its codec aliases and `Record` re-export.
pub fn check_as(schema: &Schema, form: Form) -> Result<(), Error> {
    match form {
        Form::Sealed => Ok(()),
        Form::Lean => {
            for declaration in &schema.declarations {
                let (name, line) = match declaration {
                    Declaration::Record(record) => (&record.name, record.line),
                    Declaration::Enum(enumeration) => (&enumeration.name, enumeration.line),
                };
                if name == "Record" || name == "Path" || name == "Problem" {
                    return Err(Error { line, message: format!("lean type name {name} is reserved") });
                }
            }
            Ok(())
        }
    }
}

/// Writes either form without changing the family's wire format.
/// A lean schema must pass `check_as` before generation.
#[must_use]
pub fn generate_as(schema: &Schema, form: Form) -> Output {
    match form {
        Form::Sealed => generate(schema),
        Form::Lean => {
            check_as(schema, form).expect("a lean schema passes check_as before generation");
            lean::generate(schema)
        }
    }
}

/// Writes the sealed module for one schema version.
#[must_use]
pub fn generate(schema: &Schema) -> Output {
    let context = analysis::Context::new(schema);
    let mut records =
        format!("// Generated by skein-codegen from {} {}. Do not edit.\n\n", schema.family, schema.version);
    let fields = schema
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Record(record) => Some(record.fields.as_slice()),
            Declaration::Enum(_) => None,
        })
        .flatten();
    let types = fields.map(|field| &field.ty).collect::<Vec<_>>();
    if types.iter().any(|ty| uses_box(ty)) || has_boxed_variants(schema) {
        records.push_str("use alloc::boxed::Box;\n");
    }
    records.push_str("use skein_lib::List;\n\n");
    writeln!(
        &mut records,
        "/// This family's schema version.\npub const VERSION: u16 = {};\n",
        emit_limits::literal(schema.version)
    )
    .expect("generated source");
    emit_limits::emit(&context, &mut records);
    emit_bounds::emit_family(&context, &mut records);
    for (index, declaration) in schema.declarations.iter().enumerate() {
        emit_types::emit(&context, declaration, &mut records);
        emit_codec::emit(&context, declaration, &mut records);
        emit_bounds::emit(
            declaration,
            schema.declarations.get(index.checked_add(1).expect("declaration index")),
            &mut records,
        );
        if let Declaration::Record(record) = declaration {
            emit_limits::emit_record(&context, record, &mut records);
        }
    }
    Output { records }
}

fn has_boxed_variants(schema: &Schema) -> bool {
    schema.declarations.iter().any(|declaration| match declaration {
        Declaration::Enum(enumeration) => enumeration.variants.iter().any(|variant| variant.boxed),
        Declaration::Record(_) => false,
    })
}

#[cfg(test)]
mod lean_tests;
#[cfg(test)]
mod tests;
