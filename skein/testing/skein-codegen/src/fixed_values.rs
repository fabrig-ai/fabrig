//! Fixed-rule values (data/codec/golden.md, section 1).

use crate::{Declaration, Enumeration, FixedValue, Record, Type, Variant, analysis::Context, emit_limits};

fn pattern(bound: u32, text: bool) -> Vec<u8> {
    let source: &[u8] = if text { b"abc" } else { &[0, 0x7f, 0xff] };
    source.iter().copied().take(usize::try_from(bound).expect("u32 fits usize")).collect()
}

fn write_record(record: &Record, full: bool, schema: &Context<'_>, out: &mut Vec<u8>) {
    if record.versioned {
        out.extend_from_slice(&schema.version.to_be_bytes());
    }
    for field in &record.fields {
        write_type(&field.ty, full, schema, out);
    }
}

fn write_variant(enumeration: &Enumeration, variant: &Variant, full: bool, schema: &Context<'_>, out: &mut Vec<u8>) {
    let index =
        enumeration.variants.iter().position(|candidate| candidate.name == variant.name).expect("variant in enum");
    out.push(u8::try_from(index).expect("at most 256 variants"));
    if let Some(record) = &variant.record {
        match schema.declaration(record) {
            Declaration::Record(record) => write_record(record, full, schema, out),
            Declaration::Enum(_) => unreachable!("a variant only holds a record"),
        }
    }
}

fn write_type(ty: &Type, full: bool, schema: &Context<'_>, out: &mut Vec<u8>) {
    match ty {
        Type::U8 => out.push(if full { u8::MAX } else { 0 }),
        Type::U16 => out.extend_from_slice(&(if full { u16::MAX } else { 0 }).to_be_bytes()),
        Type::U32 => out.extend_from_slice(&(if full { u32::MAX } else { 0 }).to_be_bytes()),
        Type::U64 | Type::Duration => out.extend_from_slice(&(if full { u64::MAX } else { 0 }).to_be_bytes()),
        Type::Bool => out.push(u8::from(full)),
        Type::Fixed(bound) => {
            for _ in 0_u32..*bound {
                out.push(if full { 0xa5 } else { 0 });
            }
        }
        Type::Bytes(bound) | Type::Secret(bound) | Type::Text(bound) => {
            let bytes = if full { pattern(*bound, matches!(ty, Type::Text(_))) } else { Vec::new() };
            out.extend_from_slice(&u32::try_from(bytes.len()).expect("pattern fits u32").to_be_bytes());
            out.extend_from_slice(&bytes);
        }
        Type::List(bound, item) => {
            let count = u32::from(full && *bound > 0);
            out.extend_from_slice(&count.to_be_bytes());
            if count > 0 {
                write_type(item, true, schema, out);
            }
        }
        Type::Option(item) => {
            out.push(u8::from(full));
            if full {
                write_type(item, true, schema, out);
            }
        }
        Type::Named(name) => match schema.declaration(name) {
            Declaration::Record(record) => write_record(record, full, schema, out),
            Declaration::Enum(enumeration) => {
                let variant = enumeration.variants.first().expect("enum nonempty");
                write_variant(enumeration, variant, full, schema, out);
            }
        },
    }
}

/// Fixed-rule values paired with their schema declaration, without parsing filenames.
pub(crate) fn fixed_values(schema: &crate::Schema) -> Vec<(skein_codec::schema::DeclIx, FixedValue)> {
    named_fixed_values(&Context::new(schema))
}

fn named_fixed_values(schema: &Context<'_>) -> Vec<(skein_codec::schema::DeclIx, FixedValue)> {
    let mut fixed_values = Vec::new();
    for (index, declaration) in schema.declarations.iter().enumerate() {
        let root = skein_codec::schema::DeclIx::new(u32::try_from(index).expect("schema declaration index"));
        match declaration {
            Declaration::Record(record) => {
                for (label, full) in [("smallest", false), ("full", true)] {
                    let mut bytes = Vec::new();
                    write_record(record, full, schema, &mut bytes);
                    let name = format!("record_{}_{}", emit_limits::snake(&record.name), label);
                    fixed_values.push((root, FixedValue { name: format!("{name}.bin"), bytes }));
                }
            }
            Declaration::Enum(enumeration) => {
                for variant in &enumeration.variants {
                    for (label, full) in [("smallest", false), ("full", true)] {
                        let mut bytes = Vec::new();
                        write_variant(enumeration, variant, full, schema, &mut bytes);
                        let name = format!("enum_{}_{}_{}", emit_limits::snake(&enumeration.name), variant.name, label);
                        fixed_values.push((root, FixedValue { name: format!("{name}.bin"), bytes }));
                    }
                }
            }
        }
    }
    fixed_values
}
