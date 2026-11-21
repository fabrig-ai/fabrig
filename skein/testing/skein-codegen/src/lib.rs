//! A schema parser and generator for bounded codecs (data/codec/README.md, sections 3-6).
//! The schema and declaration tree are author-owned; this crate has no peer
//! input or running machine state. [`parse`] validates a schema before code
//! generation.

extern crate alloc;

mod analysis;
mod check;
mod constructs;
pub mod diff;
mod emit_limits;
mod emit_tests;
mod emit_types;
mod largest;
mod lean;
mod maps;
mod parse;
pub mod patch;
mod values;

pub use constructs::{CONSTRUCTS, Construct, Kind, constructs, missing, unpinned};
pub use emit_limits::{pascal, snake};
pub use largest::{largest, largest_variants, roots};
pub use parse::{Error, parse};
pub use values::{Values, values};
pub use maps::translate;

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
    /// Whether this field frames records as pieces, which stay by hand in maps.
    pub pieces: bool,
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

/// Writes the records' module for one schema version.
#[must_use]
pub fn generate(schema: &Schema) -> Output {
    lean::generate(schema)
}

#[cfg(test)]
mod lean_tests;
#[cfg(test)]
mod tests;
