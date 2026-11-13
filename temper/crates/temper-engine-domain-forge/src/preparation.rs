//! The worker's per-member prepare outcomes (domain/landing.md, 5.2).
use alloc::boxed::Box;
use skein_json::{Kind, document::Document};
use skein_lib::List;
use temper_engine_domain_forge_landing::Outcome;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Prepared {
    /// Position in the landing, counted from zero.
    pub member: u16,
    pub outcome: Outcome,
    pub files: Box<[u8]>,
}

#[must_use]
pub fn outcomes(bytes: &[u8], members: u32) -> Option<Box<[Prepared]>> {
    let length = u32::try_from(bytes.len()).ok()?;
    let limits = skein_json::tokenizer::Limits { depth: 8, string: length, number: 20, chunk: 256, length };
    let document = skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length }).ok()?;
    let mut result: List<Prepared> = List::with_capacity(members);
    let mut recognized = false;
    for name in [b"combined".as_slice(), b"conflicts"] {
        let Some(array) = document.member(0, name) else { continue };
        recognized = true;
        if document.token(array)?.kind != Kind::ArrayStart { return None; }
        let end = document.value_end(array)?.checked_sub(1)?;
        let start = array.checked_add(1)?;
        let mut cursor = start;
        for at in start..end {
            if at != cursor { continue; }
            let token = document.token(document.member(at, b"member")?)?;
            if token.kind != Kind::Number { return None; }
            let member = skein_lib::bytes::decimal(document.text(token)?)?;
            let member = u16::try_from(member.checked_sub(1)?).ok()?;
            if u32::from(member) >= members { return None; }
            let mut files = List::with_capacity(length);
            let outcome = if name == b"combined" {
                match text(&document, at, b"by")?.as_ref() {
                    b"merge" | b"squash" => Outcome::Combined,
                    _ => return None,
                }
            } else {
                let array = document.member(at, b"files")?;
                if document.token(array)?.kind != Kind::ArrayStart { return None; }
                for at in array.checked_add(1)?..document.value_end(array)?.checked_sub(1)? {
                    let value = document.token(at)?;
                    if value.kind != Kind::String { return None; }
                    if !files.is_empty() { append(&mut files, b", ")?; }
                    append(&mut files, document.text(value)?)?;
                }
                match text(&document, at, b"with")?.as_ref() {
                    b"branch" => Outcome::Conflict,
                    b"member" => Outcome::EarlierConflict,
                    _ => return None,
                }
            };
            for prior in &result { if prior.member == member { return None; } }
            result.push(Prepared { member, outcome, files: files.into_boxed() }).ok()?;
            cursor = document.value_end(at)?;
        }
    }
    if !recognized || result.is_empty() { return None; }
    Some(result.into_boxed())
}
fn text(document: &Document, object: u32, name: &[u8]) -> Option<Box<[u8]>> {
    let token = document.token(document.member(object, name)?)?;
    if token.kind != Kind::String { return None; }
    Some(Box::from(document.text(token)?))
}

fn append(list: &mut List<u8>, bytes: &[u8]) -> Option<()> {
    for byte in bytes { list.push(*byte).ok()?; }
    Some(())
}
