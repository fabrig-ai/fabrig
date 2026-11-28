//! One whole borrowed TOML 1.0 document in, an owned document or a line's refusal out
//! (skein/docs/design/data/toml.md, 2–6).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod atom;
mod cursor;
mod document;
mod parser;
mod string;

pub use document::{Array, DateTimeKind, Document, Entry, Table, TableKind, Text, Value, ValueKind, document_bound};
pub use parser::parse;

#[cfg(test)]
mod tests;

/// Admission bounds (skein/docs/design/data/toml.md, 6). Each is at least one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// Input length, including any BOM.
    pub input_bytes: u32,
    /// Open arrays and inline tables, plus the segments of a key being read.
    pub depth: u32,
    /// Table entries and array elements, including implicit tables, excluding the root.
    pub entries: u32,
}

/// The bound a document passed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Limit {
    InputBytes,
    Depth,
    Entries,
}

/// The first fault (skein/docs/design/data/toml.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Refusal {
    pub line: u64,
    pub why: Why,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    InvalidUtf8,
    Grammar,
    KeyRedefined,
    TableRedefined,
    Range,
    Limit(Limit),
}
