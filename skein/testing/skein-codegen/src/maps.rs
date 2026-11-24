//! Translation from maps files (skein/docs/design/data/codec/maps.md, 2-7).

use alloc::collections::BTreeMap;

use crate::{Declaration, Error, Schema};

mod check;
mod emit;
mod parse;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Shape {
    #[default]
    Same,
    Slice,
    Boxed,
}

#[derive(Clone, Debug)]
struct Item {
    line: usize,
    name: String,
    rename: Option<String>,
    via: Option<String>,
    tuple: bool,
    shape: Shape,
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
    context: Option<(usize, String)>,
    refuses: BTreeMap<String, usize>,
    with_context: BTreeMap<String, usize>,
    aliases: BTreeMap<String, String>,
    declarations: Vec<Mapping>,
}

impl Maps {
    fn mapping(&self, name: &str) -> Option<&Mapping> {
        self.declarations.iter().find(|mapping| mapping.name == name)
    }

    fn path(&self, path: &str) -> String {
        let mut output = String::new();
        let mut word = String::new();
        for character in path.chars() {
            if character.is_alphanumeric() || character == '_' {
                word.push(character);
            } else {
                self.append_word(&mut output, &word);
                word.clear();
                output.push(character);
            }
        }
        self.append_word(&mut output, &word);
        output
    }

    fn append_word(&self, output: &mut String, word: &str) {
        let qualified = output.trim_end().ends_with("::") || output.ends_with('\'');
        let expanded = if qualified { word } else { self.aliases.get(word).map_or(word, String::as_str) };
        output.push_str(expanded);
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
    emit::generate(schema, &maps)
}
