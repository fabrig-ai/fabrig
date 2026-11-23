//! Caller-prepared binary texts, matched whole (protocol/store/opening.md, 3).
use skein_codec::schema::{self, Schema};

pub(crate) fn matching(schemas: &[Schema], text: &[u8]) -> Option<usize> {
    for (index, schema) in schemas.iter().enumerate() {
        if schema.text() == text {
            return Some(index);
        }
    }
    None
}

pub(super) fn get<'a>(prepared: &'a [Schema], text: &[u8], parsed: &'a mut Option<Schema>) -> Option<&'a Schema> {
    if let Some(index) = matching(prepared, text) {
        return prepared.get(index);
    }
    *parsed = Some(schema::parse(text).ok()?);
    parsed.as_ref()
}
