//! Caller-prepared binary texts, matched whole (protocol/store/opening-owners.md, 1).
use skein_codec::schema::{self, Schema};
use skein_lib::List;

/// Reuse successful parses within one owner-set check, matched by exact text.
/// At most one cached schema per current family; other frame texts use scratch.
pub(super) fn current<'a>(prepared: &'a [Schema], cache: &'a mut List<Schema>, text: &[u8]) -> Option<&'a Schema> {
    if let Some(schema) = find(prepared, text) {
        return Some(schema);
    }
    for index in 0..cache.len() {
        if cache.get(index)?.text() == text {
            return cache.get(index);
        }
    }
    let index = cache.len();
    cache.push(schema::parse(text).ok()?).ok()?;
    cache.get(index)
}

pub(super) fn get<'a>(prepared: &'a [Schema], cache: &'a [Schema], text: &[u8]) -> Option<&'a Schema> {
    match find(prepared, text) {
        Some(schema) => Some(schema),
        None => find(cache, text),
    }
}

pub(crate) fn frame<'a>(
    prepared: &'a [Schema],
    cache: &'a [Schema],
    text: &[u8],
    parsed: &'a mut Option<Schema>,
) -> Option<&'a Schema> {
    if let Some(schema) = get(prepared, cache, text) {
        return Some(schema);
    }
    *parsed = Some(schema::parse(text).ok()?);
    parsed.as_ref()
}

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
