//! Caller-prepared binary texts, matched whole (protocol/store/opening-owners.md, 1).
use skein_codec::schema::{self, Schema};

#[expect(clippy::manual_find, reason = "step code uses no closures (programming-model.md, 10.3)")]
pub(crate) fn matching<'a>(prepared: &'a [Schema], text: &[u8]) -> Option<&'a Schema> {
    for schema in prepared {
        if schema.text() == text {
            return Some(schema);
        }
    }
    None
}

pub(crate) fn get<'a>(prepared: &'a [Schema], text: &[u8], parsed: &'a mut Option<Schema>) -> Option<&'a Schema> {
    if let Some(schema) = matching(prepared, text) {
        return Some(schema);
    }
    *parsed = Some(schema::parse(text).ok()?);
    parsed.as_ref()
}
