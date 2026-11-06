//! Call arguments for the work line (temper/docs/design/client/README.md, 3.5).
use alloc::boxed::Box;
use skein_json::{Document, Kind};
use skein_lib::Writer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Touched {
    Doc { topic: Box<[u8]>, from: Option<u64> },
    Search { words: Box<[u8]> },
    Inventory,
    Setting { kind: Box<[u8]>, number: Option<u64> },
    SettingHistory,
    Other { argument: Option<Box<[u8]>> },
}
#[must_use]
pub fn touched(tool: &[u8], input: &[u8]) -> Touched {
    let Some(doc) = parse(input) else { return Touched::Other { argument: None } };
    if tool == b"read_docs" {
        if let Some(words) = string(&doc, 0, b"search") {
            return Touched::Search { words: bounded(words) };
        }
        if let Some(topic) = string(&doc, 0, b"topic") {
            return Touched::Doc { topic: bounded(topic), from: number(&doc, 0, b"from") };
        }
    }
    if tool == b"settings" {
        if let Some(setting) = doc.member(0, b"setting")
            && let Some(kind) = string(&doc, setting, b"kind")
        {
            return Touched::Setting { kind: bounded(kind), number: number(&doc, setting, b"number") };
        }
        if let Some(history) = doc.member(0, b"history")
            && let Some(token) = doc.token(history)
            && token.kind != Kind::False
            && token.kind != Kind::Null
        {
            return Touched::SettingHistory;
        }
        return Touched::Inventory;
    }
    let mut cursor = 1;
    for _step in 0..doc.len() {
        let Some(key) = doc.token(cursor) else { break };
        if key.kind != Kind::Key {
            break;
        }
        let Some(value) = cursor.checked_add(1) else { break };
        let Some(token) = doc.token(value) else { break };
        if token.kind == Kind::String {
            return Touched::Other {
                argument: match doc.text(token) {
                    Some(text) => Some(bounded(text)),
                    None => None,
                },
            };
        }
        if token.kind == Kind::Number {
            let key = doc.text(key).unwrap_or(b"");
            let value = doc.text(token).unwrap_or(b"");
            let mut argument = Writer::new(
                key.len()
                    .checked_add(value.len())
                    .expect("input bounds joined text")
                    .checked_add(1)
                    .expect("separator fits"),
            );
            argument.put(key).expect("key fits");
            argument.put(b" ").expect("separator fits");
            argument.put(value).expect("number fits");
            return Touched::Other { argument: Some(bounded(&argument.finish())) };
        }
        let Some(end) = doc.value_end(value) else { break };
        cursor = end;
    }
    Touched::Other { argument: None }
}
fn parse(input: &[u8]) -> Option<Document> {
    let length = u32::try_from(input.len()).ok()?;
    if length == u32::MAX {
        return None;
    }
    skein_json::parse(
        input,
        &skein_json::tokenizer::Limits { depth: 16, string: length, number: length, chunk: 256, length },
        &skein_json::document::Limits { tokens: length, text: length },
    )
    .ok()
}
fn string<'a>(doc: &'a Document, object: u32, key: &[u8]) -> Option<&'a [u8]> {
    let token = doc.token(doc.member(object, key)?)?;
    if token.kind == Kind::String { doc.text(token) } else { None }
}
fn number(doc: &Document, object: u32, key: &[u8]) -> Option<u64> {
    let token = doc.token(doc.member(object, key)?)?;
    if token.kind != Kind::Number {
        return None;
    }
    let mut number = 0_u64;
    for byte in doc.text(token)? {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
fn bounded(bytes: &[u8]) -> Box<[u8]> {
    let bytes = first_line(bytes);
    let mut end = bytes.len().min(256);
    for _continuation in 0_u8..3 {
        match bytes.get(end) {
            Some(byte) if byte & 0xc0 == 0x80 => end = end.saturating_sub(1),
            Some(_) | None => break,
        }
    }
    skein_lib::bytes::copy_of(bytes.get(..end).expect("character boundary"))
}
#[must_use]
pub fn first_line(outcome: &[u8]) -> &[u8] {
    for (at, byte) in outcome.iter().enumerate() {
        if *byte == b'\n' || *byte == b'\r' {
            return outcome.get(..at).expect("first line prefix");
        }
    }
    outcome
}
