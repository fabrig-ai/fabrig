//! Schema names and grammar spelling (data/codec/README.md, 7.1, and data/codec/diff.md, 2).

use skein_codec::patch::Target;
use skein_codec::schema::{DeclIx, FieldIx, Held, Schema, Shape, Span, Type, TypeIx, VariantIx};

pub(super) fn word(schema: &Schema, span: Span) -> String {
    String::from_utf8(schema.spelled(span).to_vec()).expect("schema names are ASCII")
}
pub(super) fn name(schema: &Schema, at: u32) -> String {
    word(schema, schema.declaration(DeclIx::new(at)).name)
}
pub(super) fn label(schema: &Schema, target: Target) -> String {
    word(
        schema,
        match target {
            Target::Declaration(at) => schema.declaration(at).name,
            Target::Field(at) => schema.field(at).name,
            Target::Variant(at) => schema.variant(at).name,
        },
    )
}
pub(super) fn parts(schema: &Schema, at: DeclIx) -> Vec<Target> {
    let (first, count, field) = match schema.declaration(at).shape {
        Shape::Record { first, count, .. } => (first.index(), count, true),
        Shape::Enum { first, count } => (first.index(), count, false),
    };
    (first..first.checked_add(count).expect("bounded parts table"))
        .map(|at| if field { Target::Field(FieldIx::new(at)) } else { Target::Variant(VariantIx::new(at)) })
        .collect()
}
pub(super) fn spell(schema: &Schema, mut at: TypeIx) -> String {
    let mut tokens = Vec::new();
    loop {
        let leaf = match schema.ty(at) {
            Type::List { bound, item } => {
                tokens.push(format!("list {bound}"));
                at = item;
                continue;
            }
            Type::Option(item) => {
                tokens.push("option".into());
                at = item;
                continue;
            }
            Type::U8 => "u8".into(),
            Type::U16 => "u16".into(),
            Type::U32 => "u32".into(),
            Type::U64 => "u64".into(),
            Type::Bool => "bool".into(),
            Type::Duration => "duration".into(),
            Type::Fixed(n) => format!("fixed {n}"),
            Type::Text(n) => format!("text {n}"),
            Type::Named(at) => name(schema, at.index()),
            ty @ (Type::Bytes { bound, held } | Type::Secret { bound, held }) => {
                let kind = if matches!(ty, Type::Secret { .. }) { "secret" } else { "bytes" };
                match held {
                    Held::Opaque => format!("{kind} {bound}"),
                    Held::Record { family, name } => {
                        format!("{kind} {bound} holds {} {}", word(schema, family), word(schema, name))
                    }
                    Held::Pieces(at) => format!("{kind} {bound} pieces {}", name(schema, at.index())),
                }
            }
        };
        tokens.push(leaf);
        return tokens.join(" ");
    }
}
