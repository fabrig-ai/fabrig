//! A route's list of models (skein/docs/design/llm.md, 4.9). Pure: holds no connection.

use alloc::boxed::Box;
use skein_json::{Document, Kind, document, tokenizer};
use skein_lib::{List, bytes};

use crate::Provider;

/// One model, with only the facts the provider supplies.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Listed {
    pub name: Box<[u8]>,
    pub shown: Option<Box<[u8]>>,
    pub window: Option<u32>,
    pub output: Option<u32>,
    pub efforts: Box<[Box<[u8]>]>,
}

/// Admission bounds for a list's body and the rows kept.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub body: u32,
    pub rows: u32,
}

impl Limits {
    pub const DEFAULT: Limits = Limits { body: 1 << 20, rows: 256 };
}

/// A body past its bound, invalid JSON, or a document without the route's rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unreadable;

/// The first page of the route's models.
#[must_use]
pub fn target(provider: Provider) -> &'static [u8] {
    match provider {
        Provider::OpenAi => b"/v1/models",
        Provider::Anthropic => b"/v1/models?limit=1000",
    }
}

/// Reads one complete list, preserving provider order and omitting fields past their bounds.
pub fn parse(provider: Provider, body: &[u8], limits: &Limits) -> Result<Box<[Listed]>, Unreadable> {
    let Ok(length) = u32::try_from(body.len()) else {
        return Err(Unreadable);
    };
    if length > limits.body || length == u32::MAX {
        return Err(Unreadable);
    }
    // Each token, decoded byte and nesting level consumes at least one input byte.
    let document = match skein_json::parse(
        body,
        &tokenizer::Limits { depth: length, string: length, number: length, chunk: 256, length },
        &document::Limits { tokens: length, text: length },
    ) {
        Ok(document) => document,
        Err(_error) => return Err(Unreadable),
    };
    let plan = provider == Provider::OpenAi && document.member(0, b"models").is_some();
    let member = if plan { b"models".as_slice() } else { b"data".as_slice() };
    let array = document.member(0, member).ok_or(Unreadable)?;
    if document.token(array).ok_or(Unreadable)?.kind != Kind::ArrayStart {
        return Err(Unreadable);
    }
    let mut rows = List::with_capacity(limits.rows);
    let mut cursor = array.checked_add(1).ok_or(Unreadable)?;
    for _step in 0..document.len() {
        if rows.room() == 0 || document.token(cursor).ok_or(Unreadable)?.kind == Kind::ArrayEnd {
            break;
        }
        if let Some(row) = read_row(&document, cursor, provider, plan) {
            rows.push(row).expect("the row limit was checked before reading");
        }
        cursor = document.value_end(cursor).ok_or(Unreadable)?;
    }
    Ok(rows.into_boxed())
}

#[expect(clippy::manual_map, reason = "the step subset spells Option conversions with a match")]
fn read_row(document: &Document, object: u32, provider: Provider, plan: bool) -> Option<Listed> {
    if plan && text(document, document.member(object, b"visibility"), 4)? != b"list" {
        return None;
    }
    let name_key = if plan { b"slug".as_slice() } else { b"id".as_slice() };
    let name = bytes::copy_of(text(document, document.member(object, name_key), 128)?);
    let shown = match provider {
        Provider::OpenAi if !plan => None,
        Provider::OpenAi | Provider::Anthropic => match text(document, document.member(object, b"display_name"), 64) {
            Some(value) => Some(bytes::copy_of(value)),
            None => None,
        },
    };
    let (window, output, efforts) = match provider {
        Provider::OpenAi => (None, None, List::with_capacity(0).into_boxed()),
        Provider::Anthropic => (
            number(document, object, b"max_input_tokens"),
            number(document, object, b"max_tokens"),
            efforts(document, object),
        ),
    };
    Some(Listed { name, shown, window, output, efforts })
}

fn text(document: &Document, value: Option<u32>, bound: usize) -> Option<&[u8]> {
    let token = document.token(value?)?;
    if token.kind != Kind::String {
        return None;
    }
    let value = document.text(token)?;
    if value.len() > bound {
        return None;
    }
    Some(value)
}

fn number(document: &Document, object: u32, key: &[u8]) -> Option<u32> {
    let token = document.token(document.member(object, key)?)?;
    if token.kind != Kind::Number {
        return None;
    }
    let mut value = 0_u32;
    for &byte in document.text(token)? {
        let digit = byte.checked_sub(b'0')?;
        if digit > 9 {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u32::from(digit))?;
    }
    if value == 0 { None } else { Some(value) }
}

fn efforts(document: &Document, object: u32) -> Box<[Box<[u8]>]> {
    let mut kept = List::with_capacity(8);
    if let Some(capabilities) = document.member(object, b"capabilities")
        && let Some(effort) = document.member(capabilities, b"effort")
    {
        for name in [b"low".as_slice(), b"medium", b"high", b"xhigh", b"max"] {
            if let Some(level) = document.member(effort, name)
                && let Some(supported) = document.member(level, b"supported")
                && let Some(token) = document.token(supported)
                && token.kind == Kind::True
            {
                kept.push(bytes::copy_of(name)).expect("five known efforts fit eight slots");
            }
        }
    }
    kept.into_boxed()
}
