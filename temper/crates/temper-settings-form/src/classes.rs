//! Project classes in settings files (temper/docs/design/client/tools.md, 5.3).
use crate::file::{Decoder, Error, error, field, optional};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;
use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Refusal, Subject};

pub(super) fn read(d: &Decoder<'_>, at: u32) -> Result<w::ProjectClasses, Error> {
    let fields = d.object(at, &[b"items", b"sets"])?;
    let positions = d.array(
        field(&fields, b"items")?,
        d.kinds.project_classes_items.min(w::CEILINGS.project_classes_items),
        b"items",
    )?;
    let mut items = List::with_capacity(positions.len());
    for at in &positions {
        let item = match item(d, *at) {
            Ok(item) => item,
            Err(why) => return Err(why.at(b"items")),
        };
        items.push(item).expect("counted items");
    }
    let positions =
        d.array(field(&fields, b"sets")?, d.kinds.project_classes_sets.min(w::CEILINGS.project_classes_sets), b"sets")?;
    let mut sets = List::with_capacity(positions.len());
    for at in &positions {
        let set = match set(d, *at) {
            Ok(set) => set,
            Err(why) => return Err(why.at(b"sets")),
        };
        sets.push(set).expect("counted sets");
    }
    let classes = Classes { items: items.into_boxed(), sets: sets.into_boxed() };
    if let Err(why) = classes.check() {
        return Err(refusal(why));
    }
    temper_protocol_reaches::classes_record(&classes).ok_or(error(b"", b"wire value bound"))
}

fn item(d: &Decoder<'_>, at: u32) -> Result<Item, Error> {
    let fields = d.object(at, &[b"kind", b"set", b"words", b"class"])?;
    let subject = match (optional(&fields, b"kind"), optional(&fields, b"set")) {
        (Some(at), None) => {
            let word = match d.string(at) {
                Ok(word) => word,
                Err(why) => return Err(why.at(b"kind")),
            };
            Subject::Kind(Kind::named(word).ok_or(error(b"kind", b"unknown reach kind"))?)
        }
        (None, Some(at)) => Subject::Set(number(d, at, b"set")?),
        (Some(_), Some(_)) | (None, None) => return Err(error(b"kind/set", b"exactly one of kind or set required")),
    };
    let words = match optional(&fields, b"words") {
        Some(at) => words(d, at, d.kinds.class_item_words.min(w::CEILINGS.class_item_words), b"words")?,
        None => Box::new([]),
    };
    let at = field(&fields, b"class")?;
    let word = match d.string(at) {
        Ok(word) => word,
        Err(why) => return Err(why.at(b"class")),
    };
    let class = match word {
        b"decide" => Class::Decide,
        b"ask" => Class::Ask,
        b"ask-away" => Class::AskAway,
        _ => return Err(error(b"class", b"decide, ask or ask-away required")),
    };
    Ok(Item { subject, words, class })
}

fn number(d: &Decoder<'_>, at: u32, name: &[u8]) -> Result<u8, Error> {
    let value = match d.number(at) {
        Ok(value) => value,
        Err(why) => return Err(why.at(name)),
    };
    u8::try_from(value).or(Err(error(name, b"u8 bound")))
}

fn words(d: &Decoder<'_>, at: u32, most: u32, name: &[u8]) -> Result<Box<[u8]>, Error> {
    match d.words(at, most, name) {
        Ok(words) => Ok(words.into_bytes()),
        Err(why) => Err(why.named(name)),
    }
}

fn set(d: &Decoder<'_>, at: u32) -> Result<PathSet, Error> {
    let fields = d.object(at, &[b"number", b"name", b"patterns"])?;
    let number = number(d, field(&fields, b"number")?, b"number")?;
    let name = words(d, field(&fields, b"name")?, d.kinds.path_set_name.min(w::CEILINGS.path_set_name), b"name")?;
    let positions = d.array(
        field(&fields, b"patterns")?,
        d.kinds.path_set_patterns.min(w::CEILINGS.path_set_patterns),
        b"patterns",
    )?;
    let mut patterns = List::with_capacity(positions.len());
    for at in &positions {
        let pattern =
            words(d, *at, d.kinds.path_set_patterns_item.min(w::CEILINGS.path_set_patterns_item), b"patterns")?;
        patterns.push(pattern).expect("counted patterns");
    }
    Ok(PathSet { number, name, patterns: patterns.into_boxed() })
}

fn refusal(why: Refusal) -> Error {
    match why {
        Refusal::Items => error(b"items", b"at most 20 items"),
        Refusal::Sets => error(b"sets", b"at most 6 sets"),
        Refusal::Fixed { .. } => {
            error(b"items.kind", b"money, irreversible, rules and other always wait; no item allowed")
        }
        Refusal::Wait { .. } => error(b"items.class", b"wait is not a table class"),
        Refusal::Words { .. } => error(b"items.words", b"at most 120 bytes"),
        Refusal::NoSet { .. } => error(b"items.set", b"must name a set in this table"),
        Refusal::Twice { .. } => error(b"items.words", b"only one item without words per kind or set"),
        Refusal::Number { .. } => error(b"sets.number", b"distinct numbers from 1 to 6 required"),
        Refusal::Name { .. } => error(b"sets.name", b"from 1 to 40 bytes required"),
        Refusal::Patterns { .. } => error(b"sets.patterns", b"from 1 to 6 patterns required"),
        Refusal::Pattern { .. } => {
            error(b"sets.patterns", b"well formed repository-relative patterns of 1 to 80 bytes required")
        }
    }
}

pub(super) fn render(value: &w::ProjectClasses, out: &mut Encoder) -> bool {
    let classes = temper_protocol_reaches::classes_from(value);
    if classes.check().is_err() {
        return false;
    }
    out.object_start();
    out.key(b"items");
    out.array_start();
    for item in &classes.items {
        out.object_start();
        match item.subject {
            Subject::Kind(kind) => {
                out.key(b"kind");
                out.string(kind.name());
            }
            Subject::Set(number) => {
                out.key(b"set");
                out.unsigned(u64::from(number));
            }
        }
        out.key(b"words");
        out.string(&item.words);
        out.key(b"class");
        out.string(match item.class {
            Class::Decide => b"decide",
            Class::Ask => b"ask",
            Class::AskAway => b"ask-away",
            Class::Wait => unreachable!("checked table has no wait item"),
        });
        out.object_end();
    }
    out.array_end();
    out.key(b"sets");
    out.array_start();
    for set in &classes.sets {
        out.object_start();
        out.key(b"number");
        out.unsigned(u64::from(set.number));
        out.key(b"name");
        out.string(&set.name);
        out.key(b"patterns");
        out.array_start();
        for pattern in &set.patterns {
            out.string(pattern);
        }
        out.array_end();
        out.object_end();
    }
    out.array_end();
    out.object_end();
    true
}
