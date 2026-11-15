//! Uniform declaration entries and their JSON forms (hearth/docs/design/protocol/format/tasks.md, 2 to 7 and 14, and hearth/docs/design/protocol/format/README.md, 7.1 and 7.3 and 8.1
//! ).
use crate::{
    Declared, Error, Expected, Object, Position,
    declaration::{Number, Text, fault, object, text, texts, unknown, wrong},
    json::{Kind, Value},
};
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::{
    errors::Takes,
    revision::declaration::{Map, Name},
};
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entries {
    pub map: Map,
    pub at: Position,
    pub entries: Box<[Entry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub key: Text,
    pub at: Position,
    pub when: Option<Text>,
    pub for_each: Option<Text>,
    pub fields: Box<[Field]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Field {
    pub name: Name,
    pub at: Position,
    pub value: Literal,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Literal {
    Text(Text),
    Plain(Text),
    Typed(Text),
    Int(Number),
    Bool { at: Position, value: bool },
    Strings { at: Position, each: Box<[Text]> },
    Range { at: Position, fields: Box<[Field]> },
}
const FIELDS: &[(Map, &[Name])] = &[
    (Map::Packages, &[Name::Component]),
    (Map::Preseeds, &[Name::Type, Name::Value, Name::Owner]),
    (Map::Initramfs, &[]),
    (Map::Cmdline, &[Name::Param, Name::Order]),
    (Map::Files, &[Name::Template, Name::Copy, Name::Link, Name::Mode, Name::Owner, Name::Group, Name::Directory]),
    (
        Map::Users,
        &[
            Name::Group,
            Name::Gecos,
            Name::Home,
            Name::Shell,
            Name::Password,
            Name::From,
            Name::Uid,
            Name::Gid,
            Name::Groups,
            Name::Subordinate,
        ],
    ),
    (Map::Groups, &[Name::From, Name::Gid]),
    (Map::Presets, &[Name::Action]),
    (Map::Tmpfiles, &[Name::Type, Name::Mode, Name::Owner, Name::Group, Name::Age, Name::Argument]),
    (Map::Data, &[Name::Path, Name::Mode, Name::Owner, Name::Group, Name::Class]),
    (Map::Secrets, &[Name::Path, Name::Mode, Name::Owner, Name::Group]),
    (Map::Identity, &[Name::Kind, Name::Algorithm, Name::Path, Name::PublicPath, Name::Mode, Name::Owner, Name::Group]),
    (
        Map::Software,
        &[Name::Url, Name::Sha256, Name::Unpack, Name::Path, Name::Mode, Name::Owner, Name::Group, Name::Strip],
    ),
    (Map::Live, &[Name::Unit, Name::Action, Name::Paths]),
];
pub(crate) fn read(map: Map, at: Position, value: Value) -> Result<Entries, Error> {
    let entries = object(value)?;
    let mut each = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        let mut item = Entry {
            key: Text { at: entry.at, text: entry.key },
            at: entry.value.at,
            when: None,
            for_each: None,
            fields: Box::new([]),
        };
        let fields = object(entry.value)?;
        let mut each_field = List::with_capacity(u32::try_from(fields.len()).expect("document bounded by u32"));
        for field in fields {
            match &*field.key {
                b"if" => item.when = Some(text(field.value)?),
                b"for_each" => item.for_each = Some(text(field.value)?),
                b"artifact" if map == Map::Software => return Err(fault(field.at, Declared::Later)),
                _ => {
                    let (name, takes) = shape(map, &field.key).ok_or(unknown(field.at, Object::Entry(map)))?;
                    let value = literal(field.value, takes, map == Map::Files && name == Name::Template)?;
                    each_field.push(Field { name, at: field.at, value }).expect("one field per source entry");
                }
            }
        }
        item.fields = each_field.into_boxed();
        each.push(item).expect("one entry per source entry");
    }
    Ok(Entries { map, at, entries: each.into_boxed() })
}
fn shape(map: Map, key: &[u8]) -> Option<(Name, Takes)> {
    for &(which, fields) in FIELDS {
        if which != map {
            continue;
        }
        return field_shape(fields, key);
    }
    None
}
fn field_shape(fields: &[Name], key: &[u8]) -> Option<(Name, Takes)> {
    for &name in fields {
        if name.key() == key {
            return Some((name, name.takes()));
        }
    }
    None
}
fn literal(value: Value, takes: Takes, plain: bool) -> Result<Literal, Error> {
    if takes != Takes::Range {
        return scalar(value, takes, plain);
    }
    match value.kind {
        Kind::String(text) => Ok(Literal::Typed(Text { at: value.at, text })),
        Kind::Map(entries) => {
            let mut fields = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
            for entry in entries {
                let (name, takes) =
                    field_shape(&[Name::Base, Name::Count], &entry.key).ok_or(unknown(entry.at, Object::Range))?;
                let literal = scalar(entry.value, takes, false)?;
                fields.push(Field { name, at: entry.at, value: literal }).expect("one field per source entry");
            }
            Ok(Literal::Range { at: value.at, fields: fields.into_boxed() })
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) => Err(wrong(value.at, Expected::Field(takes))),
    }
}
fn scalar(value: Value, takes: Takes, plain: bool) -> Result<Literal, Error> {
    let at = value.at;
    match value.kind {
        Kind::String(text) => {
            let text = Text { at, text };
            Ok(if plain {
                Literal::Plain(text)
            } else if takes == Takes::Text {
                Literal::Text(text)
            } else {
                Literal::Typed(text)
            })
        }
        Kind::Int(value) if takes == Takes::Int => Ok(Literal::Int(Number { at, value })),
        Kind::Bool(value) if takes == Takes::Bool => Ok(Literal::Bool { at, value }),
        Kind::List(each) if takes == Takes::Strings => {
            let each = texts(Value { at, kind: Kind::List(each) }, false)?;
            Ok(Literal::Strings { at, each })
        }
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::Map(_) => {
            Err(wrong(at, Expected::Field(takes)))
        }
    }
}
