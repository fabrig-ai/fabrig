//! Rust field types for data/codec/README.md, section 4.

use crate::{Record, Type, emit_limits};

pub(crate) fn rust_type(ty: &Type) -> String {
    match ty {
        Type::U8 => "u8".into(),
        Type::U16 => "u16".into(),
        Type::U32 => "u32".into(),
        Type::U64 => "u64".into(),
        Type::Bool => "bool".into(),
        Type::Duration => "skein_lib::Duration".into(),
        Type::Fixed(bound) => format!("[u8; {}]", emit_limits::literal(*bound)),
        Type::Secret(_) => "skein_lib::Secret".into(),
        Type::Bytes(_) | Type::Text(_) => "Box<[u8]>".into(),
        Type::List(_, item) => format!("List<{}>", rust_type(item)),
        Type::Option(item) => format!("Option<{}>", rust_type(item)),
        Type::Named(name) => name.clone(),
    }
}

pub(crate) fn excessive_bools(record: &Record) -> bool {
    record.fields.iter().filter(|field| matches!(field.ty, Type::Bool)).count() > 3
}
