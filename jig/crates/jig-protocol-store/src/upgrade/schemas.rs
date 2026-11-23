//! Caller-prepared binary texts, matched whole (protocol/store/opening-owners.md, 1).
use skein_codec::schema::{self, Schema};

pub(super) fn get<'a>(prepared: &'a [Schema], text: &[u8], parsed: &'a mut Option<Schema>) -> Option<&'a Schema> {
    for schema in prepared {
        if schema.text() == text {
            return Some(schema);
        }
    }
    *parsed = Some(schema::parse(text).ok()?);
    parsed.as_ref()
}
