//! Shared schema-default and patch-value encoding (data/codec/patches.md, 2).

use super::{Held, Schema, Shape, Type, TypeIx, VariantIx, add};
use skein_lib::{List, bytes, usize_of};

fn emit(pool: &mut List<u8>, byte: u8) {
    pool.push(byte).expect("encoded values bounded before allocation");
}
fn uint(pool: &mut List<u8>, value: u64, width: u32) {
    let bytes = value.to_be_bytes();
    let start = 8_u32.checked_sub(width).expect("integer width at most eight");
    for at in start..8 {
        emit(pool, *bytes.get(usize_of(at)).expect("integer byte index"));
    }
}
fn encoded_hex(pool: &mut List<u8>, value: &[u8], bound: u32, fixed: bool, pieces: bool) -> Result<(), ()> {
    if !value.starts_with(b"0x") {
        return Err(());
    }
    let digits = value.get(2..).expect("hex prefix checked");
    let length = u32::try_from(digits.len()).expect("text ceiling");
    if length.checked_rem(2) != Some(0) {
        return Err(());
    }
    let length = length.checked_div(2).expect("nonzero divisor");
    if length > bound || (fixed && length != bound) || (pieces && length != 0) {
        return Err(());
    }
    if !fixed {
        uint(pool, u64::from(length), 4);
    }
    for &[high, low] in digits.as_chunks::<2>().0 {
        // A value's hex digits are lower case (data/codec/patches.md, 2), though lib
        // reads either case.
        if high.is_ascii_uppercase() || low.is_ascii_uppercase() {
            return Err(());
        }
        let a = bytes::hex_value(high).ok_or(())?;
        let b = bytes::hex_value(low).ok_or(())?;
        emit(pool, a.checked_mul(16).expect("hex digit").checked_add(b).expect("hex byte"));
    }
    Ok(())
}

/// Encodes one value word, returning the inner type when `some` needs another.
#[expect(clippy::too_many_lines, reason = "exhaustively lists every type's value forms")]
pub(crate) fn encode(pool: &mut List<u8>, schema: &Schema, ty: TypeIx, word: &[u8]) -> Result<Option<TypeIx>, ()> {
    if word == b"some" {
        match schema.ty(ty) {
            Type::Option(inner) => {
                emit(pool, 1);
                return Ok(Some(inner));
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { .. }
            | Type::Text(_)
            | Type::Bytes { .. }
            | Type::List { .. }
            | Type::Named(_) => return Err(()),
        }
    }
    if word == b"none" {
        match schema.ty(ty) {
            Type::Option(_) => emit(pool, 0),
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { .. }
            | Type::Text(_)
            | Type::Bytes { .. }
            | Type::List { .. }
            | Type::Named(_) => return Err(()),
        }
    } else {
        match schema.ty(ty) {
            integer @ (Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Duration) => {
                let (width, maximum) = match integer {
                    Type::U8 => (1, u64::from(u8::MAX)),
                    Type::U16 => (2, u64::from(u16::MAX)),
                    Type::U32 => (4, u64::from(u32::MAX)),
                    Type::U64 | Type::Duration => (8, u64::MAX),
                    Type::Bool
                    | Type::Fixed(_)
                    | Type::Secret { .. }
                    | Type::Text(_)
                    | Type::Bytes { .. }
                    | Type::List { .. }
                    | Type::Option(_)
                    | Type::Named(_) => unreachable!("integer type"),
                };
                let value = bytes::decimal(word).ok_or(())?;
                if value > maximum {
                    return Err(());
                }
                uint(pool, value, width);
            }
            Type::Bool => {
                if word == b"true" {
                    emit(pool, 1);
                } else if word == b"false" {
                    emit(pool, 0);
                } else {
                    return Err(());
                }
            }
            Type::Fixed(bound) => {
                encoded_hex(pool, word, bound, true, false)?;
            }
            Type::Bytes { bound, held } => {
                let pieces = match held {
                    Held::Opaque => false,
                    Held::Pieces(_) => true,
                    Held::Record { .. } => return Err(()),
                };
                encoded_hex(pool, word, bound, false, pieces)?;
            }
            Type::Text(bound) => {
                if !word.starts_with(b"\"") || !word.ends_with(b"\"") {
                    return Err(());
                }
                let content = word.get(1..word.len().checked_sub(1).ok_or(())?).ok_or(())?;
                let length = u32::try_from(content.len()).expect("text ceiling");
                if length > bound || !crate::text_is_valid(content) {
                    return Err(());
                }
                uint(pool, u64::from(length), 4);
                for &byte in content {
                    emit(pool, byte);
                }
            }
            Type::List { .. } => {
                if word != b"[" {
                    return Err(());
                }
                uint(pool, 0, 4);
            }
            Type::Named(decl) => match schema.declaration(decl).shape {
                Shape::Record { .. } => return Err(()),
                Shape::Enum { first, count } => {
                    let mut selected = None;
                    for index in 0..count {
                        let variant = schema.variant(VariantIx::new(add(first.index(), index)));
                        if word == schema.spelled(variant.name) && variant.record.is_none() {
                            selected = Some(index);
                            break;
                        }
                    }
                    emit(pool, u8::try_from(selected.ok_or(())?).expect("enum tag at most 255"));
                }
            },
            Type::Secret { .. } | Type::Option(_) => return Err(()),
        }
    }
    Ok(None)
}
