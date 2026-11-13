//! Limits by name and kept-version bounds (data/codec/README.md, sections 4 and 7.3).

use alloc::boxed::Box;
use skein_lib::{List, Map, Writer};

use super::{DeclIx, Field, FieldIx, Schema, Shape, Type, VariantIx, add};

/// A level within a field's type chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LimitLevel {
    /// An option's value.
    Some,
    /// A list's item.
    Item,
}

/// Names a field's outermost bound: snake-case record, underscore, field.
/// The names are identifiers from a parsed schema.
#[must_use]
pub fn limit_name(record: &[u8], field: &[u8]) -> Box<[u8]> {
    let length = record.len().checked_add(1).expect("schema names fit usize");
    let mut length = length.checked_add(field.len()).expect("schema names fit usize");
    for (at, byte) in record.iter().enumerate() {
        if at != 0 && byte.is_ascii_uppercase() {
            length = length.checked_add(1).expect("schema names fit usize");
        }
    }
    let mut out = Writer::new(length);
    for (at, byte) in record.iter().enumerate() {
        if at != 0 && byte.is_ascii_uppercase() {
            out.put(b"_").expect("counted name separator");
        }
        let byte = if *byte == b'.' || *byte == b'-' { b'_' } else { byte.to_ascii_lowercase() };
        out.put(&[byte]).expect("counted name byte");
    }
    out.put(b"_").expect("counted field separator");
    out.put(field).expect("counted field name");
    out.finish()
}

/// Names the next bound within an option or list, level by level.
#[must_use]
pub fn limit_child(parent: &[u8], level: LimitLevel) -> Box<[u8]> {
    let suffix: &[u8] = match level {
        LimitLevel::Some => b"_some",
        LimitLevel::Item => b"_item",
    };
    let length = parent.len().checked_add(suffix.len()).expect("schema type chain fits usize");
    let mut out = Writer::new(length);
    out.put(parent).expect("counted parent name");
    out.put(suffix).expect("counted level suffix");
    out.finish()
}

fn bound(name: &[u8], ceiling: u32, limits: &Map<Box<[u8]>, u32>) -> u64 {
    match limits.get(name) {
        Some(value) if *value <= ceiling => u64::from(*value),
        Some(_) | None => u64::from(ceiling),
    }
}

fn field_size(
    schema: &Schema,
    field: Field,
    mut name: Box<[u8]>,
    limits: &Map<Box<[u8]>, u32>,
    sizes: &List<u64>,
) -> Option<u64> {
    let mut at = field.ty;
    let mut size = 0_u64;
    let mut copies = 1_u64;
    // Each wrapper advances in the type table. Prefixes count once per copy.
    for _ in 0..schema.types.len() {
        let leaf = match schema.ty(at) {
            Type::Option(item) => {
                size = size.checked_add(copies)?;
                name = limit_child(&name, LimitLevel::Some);
                at = item;
                continue;
            }
            Type::List { bound: ceiling, item } => {
                size = size.checked_add(copies.checked_mul(4)?)?;
                copies = copies.checked_mul(bound(&name, ceiling, limits))?;
                name = limit_child(&name, LimitLevel::Item);
                at = item;
                continue;
            }
            Type::U8 | Type::Bool => 1,
            Type::U16 => 2,
            Type::U32 => 4,
            Type::U64 | Type::Duration => 8,
            Type::Fixed(length) => u64::from(length),
            Type::Bytes { bound: ceiling, .. } | Type::Secret { bound: ceiling, .. } | Type::Text(ceiling) => {
                4_u64.checked_add(bound(&name, ceiling, limits))?
            }
            Type::Named(at) => *sizes.get(at.index()).expect("earlier declaration has its bound"),
        };
        return size.checked_add(copies.checked_mul(leaf)?);
    }
    unreachable!("a checked type chain ends in the table")
}

pub(super) fn most_bytes_within(schema: &Schema, at: DeclIx, names: &[&str], values: &[u32]) -> Option<u64> {
    if names.len() != values.len() {
        return None;
    }
    let mut limits = Map::with_capacity(u32::try_from(names.len()).ok()?);
    for (name, value) in names.iter().zip(values) {
        limits.insert(skein_lib::bytes::copy_of(name.as_bytes()), *value).expect("at most one entry per name");
    }
    let mut sizes = List::with_capacity(schema.declarations());
    for index in 0..=at.index() {
        let declaration = schema.declaration(DeclIx::new(index));
        let size = match declaration.shape {
            Shape::Record { versioned, first, count } => {
                let mut size = if versioned { 2_u64 } else { 0_u64 };
                for index in first.index()..add(first.index(), count) {
                    let field = schema.field(FieldIx::new(index));
                    let name = limit_name(schema.spelled(declaration.name), schema.spelled(field.name));
                    size = size.checked_add(field_size(schema, field, name, &limits, &sizes)?)?;
                }
                size
            }
            Shape::Enum { first, count } => {
                let mut biggest = 0;
                for index in first.index()..add(first.index(), count) {
                    let variant = schema.variant(VariantIx::new(index));
                    let size = match variant.record {
                        Some(record) => *sizes.get(record.index()).expect("earlier variant record has its bound"),
                        None => 0,
                    };
                    biggest = biggest.max(size);
                }
                biggest.checked_add(1)?
            }
        };
        sizes.push(size).expect("one bound per declaration");
    }
    sizes.get(at.index()).copied()
}
