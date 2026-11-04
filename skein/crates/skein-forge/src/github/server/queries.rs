//! Bounded GraphQL envelopes for the dialect's two byte-matched documents.
use super::{Limits, document};
use alloc::boxed::Box;
use skein_json::{
    Document,
    document::{self as json_document, Compact, Kind},
};
use skein_lib::bytes;

/// The two query documents understood by the fake.
#[derive(Debug)]
pub enum Graphql {
    /// The authenticated viewer.
    Viewer,
    /// A pull's opaque identity, merge commit and queue state.
    Pull { owner: Box<[u8]>, name: Box<[u8]>, number: u64 },
}

/// Decodes only VIEWER and PULL; other query documents are unsupported.
/// JSON envelope whitespace and member order do not change query identity.
#[must_use]
pub fn graphql_body(body: &[u8], limits: &Limits) -> Option<Graphql> {
    let document = parsed(body, limits)?;
    let mut walk = Walk { document: &document, at: 0 };
    walk.expect(Kind::ObjectStart)?;
    let mut query = None;
    let mut variables = None;
    for _field in 0..2_u32 {
        if walk.peek()?.kind == Kind::ObjectEnd {
            break;
        }
        let key = walk.text(Kind::Key)?;
        match key {
            b"query" => {
                if query.is_some() {
                    return None;
                }
                query = Some(walk.text(Kind::String)?);
            }
            b"variables" => {
                if variables.is_some() {
                    return None;
                }
                variables = Some(read_variables(&mut walk, limits)?);
            }
            _ => return None,
        }
    }
    walk.expect(Kind::ObjectEnd)?;
    if walk.peek().is_some() {
        return None;
    }
    match query? {
        document::VIEWER if variables.is_none() => Some(Graphql::Viewer),
        document::PULL => {
            let Variables { owner, name, number } = variables?;
            Some(Graphql::Pull { owner, name, number })
        }
        _ => None,
    }
}
#[derive(Debug)]
struct Variables {
    owner: Box<[u8]>,
    name: Box<[u8]>,
    number: u64,
}
fn read_variables(walk: &mut Walk<'_>, limits: &Limits) -> Option<Variables> {
    walk.expect(Kind::ObjectStart)?;
    let mut owner = None;
    let mut name = None;
    let mut number = None;
    for _field in 0..3_u32 {
        let key = walk.text(Kind::Key)?;
        match key {
            b"owner" if owner.is_none() => owner = Some(string(walk, limits)?),
            b"name" if name.is_none() => name = Some(string(walk, limits)?),
            b"number" if number.is_none() => {
                number = Some(bytes::decimal(walk.text(Kind::Number)?)?);
            }
            _ => return None,
        }
    }
    walk.expect(Kind::ObjectEnd)?;
    Some(Variables { owner: owner?, name: name?, number: number? })
}
fn string(walk: &mut Walk<'_>, limits: &Limits) -> Option<Box<[u8]>> {
    let text = walk.text(Kind::String)?;
    if text.is_empty() || u64::try_from(text.len()).ok()? > u64::from(limits.string_bytes) {
        return None;
    }
    Some(Box::from(text))
}
fn parsed(body: &[u8], limits: &Limits) -> Option<Document> {
    if u64::try_from(body.len()).ok()? > u64::from(limits.body_bytes) {
        return None;
    }
    skein_json::parse(body, &limits.json, &json_document::Limits { tokens: 16, text: limits.body_bytes }).ok()
}

#[derive(Debug)]
struct Walk<'a> {
    document: &'a Document,
    at: u32,
}
impl<'a> Walk<'a> {
    fn peek(&self) -> Option<&'a Compact> {
        self.document.token(self.at)
    }
    fn next(&mut self) -> Option<&'a Compact> {
        let record = self.peek()?;
        self.at = self.at.checked_add(1)?;
        Some(record)
    }
    fn expect(&mut self, kind: Kind) -> Option<()> {
        if self.next()?.kind != kind {
            return None;
        }
        Some(())
    }
    fn text(&mut self, kind: Kind) -> Option<&'a [u8]> {
        let record = self.next()?;
        if record.kind != kind {
            return None;
        }
        self.document.text(record)
    }
}
