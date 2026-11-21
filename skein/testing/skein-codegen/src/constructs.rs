//! The pin's schema constructs (data/codec/README.md, section 6).
//! Table metadata retains holds, explicit defaults and former names.

use skein_codec::schema::{self, DeclIx, FieldIx, Held, Named, Shape, Type, TypeIx, VariantIx};

use crate::{Error, parse};

/// A schema type without its bound or declaration name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    U8,
    U16,
    U32,
    U64,
    Bool,
    Duration,
    Fixed,
    Bytes,
    Secret,
    Text,
    List,
    Option,
    Record,
    Enum,
}

/// One construct the generator's pin can exercise, named by its `Debug` form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Construct {
    Field(Kind),
    ListItem(Kind),
    OptionValue(Kind),
    EmptyRecord,
    VersionedRecord,
    UnitVariant,
    RecordVariant,
    BoxedVariant,
    BytesHolds,
    SecretHolds,
    BytesPieces,
    SecretPieces,
    Default,
    FormerName,
}

/// The constructs held by the generator's pin, each once.
pub const CONSTRUCTS: &[Construct] = &[
    Construct::Field(Kind::U8),
    Construct::Field(Kind::U16),
    Construct::Field(Kind::U32),
    Construct::Field(Kind::U64),
    Construct::Field(Kind::Bool),
    Construct::Field(Kind::Duration),
    Construct::Field(Kind::Fixed),
    Construct::Field(Kind::Bytes),
    Construct::Field(Kind::Secret),
    Construct::Field(Kind::Text),
    Construct::Field(Kind::List),
    Construct::Field(Kind::Option),
    Construct::Field(Kind::Record),
    Construct::Field(Kind::Enum),
    Construct::ListItem(Kind::U8),
    Construct::ListItem(Kind::U16),
    Construct::ListItem(Kind::U32),
    Construct::ListItem(Kind::U64),
    Construct::ListItem(Kind::Bool),
    Construct::ListItem(Kind::Duration),
    Construct::ListItem(Kind::Fixed),
    Construct::ListItem(Kind::Bytes),
    Construct::ListItem(Kind::Secret),
    Construct::ListItem(Kind::Text),
    Construct::ListItem(Kind::List),
    Construct::ListItem(Kind::Option),
    Construct::ListItem(Kind::Record),
    Construct::ListItem(Kind::Enum),
    Construct::OptionValue(Kind::U8),
    Construct::OptionValue(Kind::U16),
    Construct::OptionValue(Kind::U32),
    Construct::OptionValue(Kind::U64),
    Construct::OptionValue(Kind::Bool),
    Construct::OptionValue(Kind::Duration),
    Construct::OptionValue(Kind::Fixed),
    Construct::OptionValue(Kind::Bytes),
    Construct::OptionValue(Kind::Secret),
    Construct::OptionValue(Kind::Text),
    Construct::OptionValue(Kind::List),
    Construct::OptionValue(Kind::Option),
    Construct::OptionValue(Kind::Record),
    Construct::OptionValue(Kind::Enum),
    Construct::EmptyRecord,
    Construct::VersionedRecord,
    Construct::UnitVariant,
    Construct::RecordVariant,
    Construct::BoxedVariant,
    Construct::BytesHolds,
    Construct::SecretHolds,
    Construct::BytesPieces,
    Construct::SecretPieces,
    Construct::Default,
    Construct::FormerName,
];

fn push(found: &mut Vec<Construct>, construct: Construct) {
    if !found.contains(&construct) {
        found.push(construct);
    }
}

fn kind(tables: &schema::Schema, ty: Type) -> Kind {
    match ty {
        Type::U8 => Kind::U8,
        Type::U16 => Kind::U16,
        Type::U32 => Kind::U32,
        Type::U64 => Kind::U64,
        Type::Bool => Kind::Bool,
        Type::Duration => Kind::Duration,
        Type::Fixed(_) => Kind::Fixed,
        Type::Bytes { bound: _, held: _ } => Kind::Bytes,
        Type::Secret { bound: _, held: _ } => Kind::Secret,
        Type::Text(_) => Kind::Text,
        Type::List { bound: _, item: _ } => Kind::List,
        Type::Option(_) => Kind::Option,
        Type::Named(at) => {
            let schema::Declaration { name: _, line: _, shape } = tables.declaration(at);
            match shape {
                Shape::Record { versioned: _, first: _, count: _ } => Kind::Record,
                Shape::Enum { first: _, count: _ } => Kind::Enum,
            }
        }
    }
}

fn held(found: &mut Vec<Construct>, value: Held, secret: bool) {
    match value {
        Held::Opaque => (),
        Held::Record { family: _, name: _ } => {
            push(found, if secret { Construct::SecretHolds } else { Construct::BytesHolds });
        }
        Held::Pieces(_) => push(found, if secret { Construct::SecretPieces } else { Construct::BytesPieces }),
    }
}

fn types(tables: &schema::Schema, mut at: TypeIx, found: &mut Vec<Construct>) {
    let mut place: fn(Kind) -> Construct = Construct::Field;
    loop {
        let ty = tables.ty(at);
        push(found, place(kind(tables, ty)));
        match ty {
            Type::List { bound: _, item } => {
                place = Construct::ListItem;
                at = item;
            }
            Type::Option(item) => {
                place = Construct::OptionValue;
                at = item;
            }
            Type::Bytes { bound: _, held: value } => {
                held(found, value, false);
                break;
            }
            Type::Secret { bound: _, held: value } => {
                held(found, value, true);
                break;
            }
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Text(_)
            | Type::Named(_) => break,
        }
    }
}

fn former(tables: &schema::Schema, named: Named, found: &mut Vec<Construct>) {
    if !tables.was(named).is_empty() {
        push(found, Construct::FormerName);
    }
}

fn declaration(tables: &schema::Schema, at: DeclIx, found: &mut Vec<Construct>) {
    former(tables, Named::Declaration(at), found);
    let schema::Declaration { name: _, line: _, shape } = tables.declaration(at);
    match shape {
        Shape::Record { versioned, first, count } => {
            if count == 0 {
                push(found, Construct::EmptyRecord);
            }
            if versioned {
                push(found, Construct::VersionedRecord);
            }
            for index in first.index()..first.index().checked_add(count).expect("field table bounded") {
                let at = FieldIx::new(index);
                former(tables, Named::Field(at), found);
                if tables.default_span(at).is_some() {
                    push(found, Construct::Default);
                }
                let schema::Field { name: _, line: _, ty } = tables.field(at);
                types(tables, ty, found);
            }
        }
        Shape::Enum { first, count } => {
            for index in first.index()..first.index().checked_add(count).expect("variant table bounded") {
                let at = VariantIx::new(index);
                former(tables, Named::Variant(at), found);
                let schema::Variant { name: _, line: _, record, boxed } = tables.variant(at);
                push(found, if record.is_some() { Construct::RecordVariant } else { Construct::UnitVariant });
                if boxed {
                    push(found, Construct::BoxedVariant);
                }
            }
        }
    }
}

/// Finds each construct once, in first-encounter order, in the schema.
/// Refuses the same texts, with the same error, as [`crate::parse`].
pub fn constructs(text: &str) -> Result<Vec<Construct>, Error> {
    let (_, tables) = parse::with_tables(text)?;
    let mut found = Vec::new();
    for at in 0..tables.declarations() {
        declaration(&tables, DeclIx::new(at), &mut found);
    }
    Ok(found)
}

fn without(found: Vec<Construct>, pinned: &[Construct]) -> Vec<Construct> {
    found.into_iter().filter(|construct| !pinned.contains(construct)).collect()
}

/// Finds the text's constructs not held by [`CONSTRUCTS`], in the same order.
pub fn unpinned(text: &str) -> Result<Vec<Construct>, Error> {
    Ok(without(constructs(text)?, CONSTRUCTS))
}

/// Finds the constructs of [`CONSTRUCTS`] the text lacks, in pin order, each once.
/// Refuses the same texts, with the same error, as [`crate::parse`].
pub fn missing(text: &str) -> Result<Vec<Construct>, Error> {
    Ok(without(CONSTRUCTS.to_vec(), &constructs(text)?))
}

#[cfg(test)]
#[path = "constructs_tests.rs"]
mod tests;
