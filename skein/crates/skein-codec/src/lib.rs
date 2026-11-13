//! Shared bounded reads and pieces for codecs (data/codec/README.md, sections 2 to 4).
//! Generated decoders call the read helpers after choosing a limit.
//! Schemas are read into bounded tables (data/codec/README.md, section 7).
//! Family patches are read and planned by [`patch`] (data/codec/patches.md).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::boxed::Box;
use skein_lib::Reader;

pub mod bridge;
pub mod patch;
pub mod piece;
pub mod schema;
pub mod value;

#[cfg(test)]
mod tests;

/// Why decoding a record failed; a generated problem adds its field path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reason {
    /// The bytes ended before the field did.
    Short,
    /// A length or count passed its field's limit.
    Bound,
    /// An enumeration's tag has no variant.
    Tag,
    /// A boolean was neither zero nor one.
    Bool,
    /// Text was not UTF-8.
    Utf8,
    /// The record has a version this decoder does not read.
    Version,
    /// Bytes remained after the record.
    Trailing,
    /// A piece's item lengths did not end exactly at its end.
    Malformed,
}

/// Reads a length and checks both its limit and the available bytes.
pub fn read_len(reader: &mut Reader<'_>, limit: u32) -> Result<u32, Reason> {
    let length = reader.u32().ok_or(Reason::Short)?;
    if length > limit {
        return Err(Reason::Bound);
    }
    if length > reader.remaining() {
        return Err(Reason::Short);
    }
    Ok(length)
}

/// Reads a list count and checks its limit. Item reads check remaining bytes.
pub fn read_count(reader: &mut Reader<'_>, limit: u32) -> Result<u32, Reason> {
    let count = reader.u32().ok_or(Reason::Short)?;
    if count > limit {
        return Err(Reason::Bound);
    }
    Ok(count)
}

/// Reads bounded, UTF-8 text into a box allocated at its final length.
pub fn read_text(reader: &mut Reader<'_>, limit: u32) -> Result<Box<[u8]>, Reason> {
    let length = read_len(reader, limit)?;
    let bytes = reader.bytes(length).ok_or(Reason::Short)?;
    if !text_is_valid(bytes) {
        return Err(Reason::Utf8);
    }
    Ok(Box::from(bytes))
}

/// Reads a versioned record's leading version without decoding the record.
pub fn version_of(bytes: &[u8]) -> Result<u16, Reason> {
    let mut reader = Reader::new(bytes);
    reader.u16().ok_or(Reason::Short)
}

/// Reports whether a complete text field is UTF-8 (RFC 3629, section 4).
#[must_use]
pub fn text_is_valid(bytes: &[u8]) -> bool {
    skein_lib::bytes::is_utf8(bytes)
}
