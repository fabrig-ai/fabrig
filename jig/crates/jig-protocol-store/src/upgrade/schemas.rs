//! Caller-prepared binary texts, matched whole (protocol/store/opening-owners.md, 1).
use skein_codec::schema::{self, Schema};

pub(crate) fn matching(schemas: &[Schema], text: &[u8]) -> Option<usize> {
    for (index, schema) in schemas.iter().enumerate() {
        if schema.text() == text {
            return Some(index);
        }
    }
    None
}

pub(crate) fn find<'a>(prepared: &'a [Schema], text: &[u8]) -> Option<&'a Schema> {
    prepared.get(matching(prepared, text)?)
}

pub(crate) fn get<'a>(prepared: &'a [Schema], text: &[u8], parsed: &'a mut Option<Schema>) -> Option<&'a Schema> {
    if let Some(schema) = find(prepared, text) {
        return Some(schema);
    }
    *parsed = Some(schema::parse(text).ok()?);
    parsed.as_ref()
}
