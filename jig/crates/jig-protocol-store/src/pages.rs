//! Ordered memory pages and exclusive cursors (protocol/store.md, 4.3 and 7.1).
pub(crate) mod reader;
use crate::{Cursor, KeptMap, Row, Store, codec, keys};
use alloc::boxed::Box;
use skein_kv::{KeyWriter, Range};
use skein_lib::List;

#[derive(Debug)]
pub(crate) struct WalkRow {
    pub row: skein_kv::Row,
    pub cursor: Cursor,
}

pub(crate) struct Walked {
    pub rows: Box<[WalkRow]>,
    pub next: Option<Cursor>,
}

pub(crate) struct Decoded {
    pub rows: Box<[Row]>,
    pub next: Option<Cursor>,
}

pub(crate) fn past(key: &[u8]) -> Option<Box<[u8]>> {
    let mut writer = KeyWriter::new(u32::MAX);
    for &byte in key {
        writer.tag(byte);
    }
    writer.tag(0);
    writer.finish()
}

pub(crate) struct Page {
    pub(crate) rows: Box<[skein_kv::Row]>,
    pub(crate) next: Option<Box<[u8]>>,
}

pub(crate) fn load(store: &mut Store, range: Range) -> Result<Page, crate::restore::Failure> {
    reader::load_map(store, KeptMap::Store, range)
}

pub(crate) fn application(intervals: &[Range], limit: u32) -> Option<Box<[Range]>> {
    let mut tagged = List::with_capacity(u32::try_from(intervals.len()).ok()?);
    for interval in intervals {
        let start = keys::application(&interval.start, limit)?;
        let end: Option<Box<[u8]>> = match &interval.end {
            Some(end) => Some(keys::application(end, limit)?),
            None => Some(Box::new([keys::APPLICATION.checked_add(1)?])),
        };
        tagged.push(Range { start, end }).expect("one tagged range per input");
    }
    Some(tagged.into_boxed())
}

pub(crate) fn decode(store: &Store, row: skein_kv::Row) -> Option<(Row, u64)> {
    if let Some(key) = keys::of_application(&row.key) {
        let size = u64::try_from(key.len()).ok()?.checked_add(u64::try_from(row.value.len()).ok()?)?;
        return Some((Row::Application { key: Box::from(key), value: row.value }, size));
    }
    let key = keys::decode(&row.key)?;
    let record = codec::decode(&key, &row.value, &store.codec, &store.names)?;
    if record.key() != key {
        return None;
    }
    // The generated family's measure is its wire length. Once decoded,
    // these stored bytes give that measure without encoding the value again.
    Some((Row::Jig(record), u64::try_from(row.value.len()).ok()?))
}

pub(crate) fn map_limits(store: &Store, map: KeptMap) -> skein_kv::Limits {
    match map {
        KeptMap::Store => store.limits.store,
        KeptMap::Secrets => store.limits.secrets,
    }
}

/// Kept keys share the walk's map and application prefix (store.md 10.3).
pub(crate) fn kept_interval(store: &Store, owner: crate::Owner, map: KeptMap, keys: &crate::Keys) -> Option<Range> {
    if owner != crate::Owner::Application {
        return Some(Range { start: Box::from(keys.from), end: Some(Box::from(keys.to)) });
    }
    let limit = map_limits(store, map).key;
    let start = application_key(map, keys.from, limit)?;
    let end = Some(application_key(map, keys.to, limit)?);
    Some(Range { start, end })
}
fn application_key(map: KeptMap, key: &[u8], limit: u32) -> Option<Box<[u8]>> {
    match map {
        KeptMap::Store => keys::application(key, limit),
        KeptMap::Secrets => crate::secrets::application(key, limit),
    }
}
