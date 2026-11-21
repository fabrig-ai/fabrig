//! Translation from maps files (skein/docs/design/data/codec/maps.md, 2-7).

use alloc::collections::BTreeMap;

use crate::{Declaration, Error, Schema};

mod check;
mod emit;
mod parse;

#[derive(Clone, Debug)]
struct Item {
    line: usize,
    name: String,
    rename: Option<String>,
    via: Option<String>,
    tuple: bool,
    block: Option<Vec<Item>>,
}

#[derive(Debug)]
enum Mode {
    Mirror(String),
    Stands { field: String, via: Option<String> },
    Payload,
    Hand,
}

#[derive(Debug)]
struct Mapping {
    line: usize,
    name: String,
    enumeration: bool,
    mode: Mode,
    items: Vec<Item>,
}

#[derive(Default)]
struct Maps {
    records: String,
    hand: Option<String>,
    context: String,
    aliases: BTreeMap<String, String>,
    declarations: Vec<Mapping>,
}

impl Maps {
    fn mapping(&self, name: &str) -> Option<&Mapping> {
        self.declarations.iter().find(|mapping| mapping.name == name)
    }

    fn path(&self, path: &str) -> String {
        let (head, tail) = path.split_once("::").unwrap_or((path, ""));
        match self.aliases.get(head) {
            Some(alias) if tail.is_empty() => alias.clone(),
            Some(alias) => format!("{alias}::{tail}"),
            None => path.to_owned(),
        }
    }
}

fn name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Record(record) => &record.name,
        Declaration::Enum(enumeration) => &enumeration.name,
    }
}

fn error(schema: &Schema, line: usize, message: impl core::fmt::Display) -> Error {
    Error { line, message: format!("schema {}: {message}", schema.family) }
}

/// Writes both translation directions, or refuses before producing any output.
/// Refusals name the maps line and schema family (data/codec/maps.md, 6-7).
pub fn translate(schema: &Schema, source: &str) -> Result<String, Error> {
    let maps = parse::parse(schema, source)?;
    check::check(schema, &maps)?;
    Ok(emit::generate(schema, &maps))
}
