//! Read-only restore pages (jig/docs/design/protocol/store.md, 7.1 and 7.2).
use super::{Arithmetic, Cause, DecodeFailure, Failure, JoinFailure, RangeFailure, RowSite, ready, site};
use crate::{Cursor, Restore, Row, Store, codec, intervals, keys, pages};
use alloc::boxed::Box;
use jig_domain_core::{Key, Record, RestoreRange};
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Token};

#[derive(Debug)]
pub struct Loaded {
    pub items: Box<[Item]>,
    pub next: Option<Cursor>,
}

#[derive(Debug)]
pub enum Item {
    Row(Row),
    MissingLink { record: Key, target: Key },
}

pub fn page(
    store: &mut Store,
    range: Restore,
    after: Option<&Cursor>,
    rows: u32,
    bytes: u64,
) -> Result<Loaded, Failure> {
    store.consumed = true;
    ready(store)?;
    let ranges = ranges(store, range)?;
    let walked = pages::reader::walk(store, &ranges, after, rows)?;
    let capacity = rows.min(store.limits.store.page.bytes);
    let mut found = List::with_capacity(capacity);
    let mut used = 0_u64;
    let mut cursor = after.cloned().unwrap_or(Cursor { interval: 0, after: Box::new([]) });
    for item in walked.rows {
        let source = site(store, &item.row.key);
        let (row, joined, size) = decode(store, item.row, &source)?;
        let total = used.checked_add(size).ok_or(Failure::at(Cause::Arithmetic(Arithmetic::PageBytes), source))?;
        if total > bytes || found.room() < if joined.is_some() { 2 } else { 1 } {
            return Ok(progress(after, Loaded { items: found.into_boxed(), next: Some(cursor) }));
        }
        used = total;
        cursor = item.cursor;
        found.push(row).expect("bounded restore items");
        if let Some(joined) = joined {
            found.push(joined).expect("pair room checked");
        }
    }
    Ok(progress(after, Loaded { items: found.into_boxed(), next: walked.next }))
}

fn progress(after: Option<&Cursor>, page: Loaded) -> Loaded {
    if page.items.is_empty()
        && let Some(next) = &page.next
    {
        let advanced = match after {
            Some(after) => {
                next.interval > after.interval || (next.interval == after.interval && next.after > after.after)
            }
            None => next.interval > 0 || !next.after.is_empty(),
        };
        assert!(advanced, "empty nonterminal restore page must advance its cursor");
    }
    page
}

fn ranges(store: &Store, range: Restore) -> Result<Box<[skein_kv::Range]>, Failure> {
    match range {
        Restore::Application(ranges) => {
            u32::try_from(ranges.len()).ok().ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
            pages::application(&ranges, store.limits.store.key)
                .ok_or(Failure::rowless(Cause::Range(RangeFailure::Encoding)))
        }
        Restore::Jig(ranges) => {
            let mut count = 0_u32;
            for range in &ranges {
                // Check census cardinalities before intervals' infallible bounded constructor.
                let size = match range {
                    RestoreRange::Census(census) => {
                        let results = u32::try_from(census.results.len()).ok();
                        let questions = u32::try_from(census.questions.len()).ok();
                        let results = results.ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
                        let questions = questions.ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
                        match results.checked_add(questions) {
                            Some(count) => count.checked_add(1),
                            None => None,
                        }
                    }
                    RestoreRange::Accounts | RestoreRange::People | RestoreRange::Tasks => Some(2),
                    RestoreRange::Deployment | RestoreRange::Settings | RestoreRange::Core => Some(1),
                };
                let size = size.ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
                count = count.checked_add(size).ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
            }
            intervals::intervals(&ranges).ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))
        }
    }
}

pub(crate) fn decode_row(store: &Store, row: skein_kv::Row, source: &RowSite) -> Result<(Row, u64), Failure> {
    if let Some(key) = keys::of_application(&row.key) {
        let key_bytes = u64::try_from(key.len()).ok();
        let value_bytes = u64::try_from(row.value.len()).ok();
        let key_bytes = key_bytes.ok_or(Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), source.clone()))?;
        let value_bytes = value_bytes.ok_or(Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), source.clone()))?;
        let size = key_bytes
            .checked_add(value_bytes)
            .ok_or(Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), source.clone()))?;
        return Ok((Row::Application { key: Box::from(key), value: row.value }, size));
    }
    let key = keys::decode(&row.key).ok_or(Failure::at(Cause::Decode(DecodeFailure::Key), source.clone()))?;
    let record = decode_value(store, &key, &row.value);
    let record = match record {
        Ok(record) => record,
        Err(failure) => return Err(Failure::at(Cause::Decode(failure), source.clone())),
    };
    let size = u64::try_from(row.value.len())
        .ok()
        .ok_or(Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), source.clone()))?;
    Ok((Row::Jig(record), size))
}

fn decode_value(store: &Store, key: &Key, value: &[u8]) -> Result<Record, DecodeFailure> {
    let record = codec::decode(key, value, &store.codec, &store.names).ok_or(DecodeFailure::Value)?;
    if record.key() != *key {
        return Err(DecodeFailure::KeyValueDisagree);
    }
    Ok(record)
}

fn decode(store: &mut Store, row: skein_kv::Row, source: &RowSite) -> Result<(Item, Option<Item>, u64), Failure> {
    let (row, size) = decode_row(store, row, source)?;
    let task = match &row {
        Row::Jig(Record::Tasks(tasks::Stored::Result(row))) => row.task,
        Row::Jig(Record::People(jig_domain_people::Stored::EndedRead { task, .. })) => *task,
        Row::Jig(_) | Row::Application { .. } => return Ok((Item::Row(row), None, size)),
    };
    let target = Key::Tasks(tasks::Key::Ended(task));
    let owner = ended(store, &target, source)?;
    let Some((owner, owner_size)) = owner else {
        let record = match row {
            Row::Jig(record) => record.key(),
            Row::Application { .. } => unreachable!("census rows have domain keys"),
        };
        // A missing link occupies one item and the source row's decoded bytes.
        return Ok((Item::MissingLink { record, target }, None, size));
    };
    match row {
        Row::Jig(Record::People(jig_domain_people::Stored::EndedRead { .. })) => {
            let joined = size
                .checked_add(owner_size)
                .ok_or(Failure::at(Cause::Arithmetic(Arithmetic::JoinedBytes), source.clone()))?;
            Ok((Item::Row(row), Some(Item::Row(Row::Jig(owner))), joined))
        }
        Row::Jig(_) | Row::Application { .. } => Ok((Item::Row(Row::Jig(owner)), None, owner_size)),
    }
}

pub(crate) fn ended(store: &mut Store, key: &Key, source: &RowSite) -> Result<Option<(Record, u64)>, Failure> {
    // Encode at the type's fixed Ended-key size even when the configured allowance is smaller,
    // so a key-encoding failure can still identify its bounded target.
    let raw = keys::encode(key, u32::MAX).expect("fixed-width ended task key fits u32");
    let target = site(store, &raw);
    let Some(encoded) = keys::encode(key, store.limits.store.key) else {
        return Err(Failure::at(Cause::Join { target, failure: JoinFailure::KeyEncoding }, source.clone()));
    };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Get { owner: Token::new(0), key: encoded },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory census joins issue no files");
    let value = match joined(above.pop()) {
        Ok(value) => value,
        Err(failure) => return Err(Failure::at(Cause::Join { target, failure }, source.clone())),
    };
    let Some(value) = value else { return Ok(None) };
    let record = match decode_value(store, key, &value) {
        Ok(record) => record,
        Err(failure) => {
            return Err(Failure::at(Cause::Join { target, failure: JoinFailure::Decode(failure) }, source.clone()));
        }
    };
    let size = u64::try_from(value.len())
        .ok()
        .ok_or(Failure::at(Cause::Arithmetic(Arithmetic::JoinedBytes), source.clone()))?;
    Ok(Some((record, size)))
}

pub(crate) fn joined(event: Option<skein_kv::Event>) -> Result<Option<Box<[u8]>>, JoinFailure> {
    match event {
        Some(skein_kv::Event::Got { value, .. }) => Ok(value),
        None => Err(JoinFailure::NoTerminal),
        Some(skein_kv::Event::Refused { refusal, .. }) => Err(JoinFailure::Refused(refusal)),
        Some(skein_kv::Event::OpenFailed { failure, .. } | skein_kv::Event::RecoveryFailed { failure }) => {
            Err(JoinFailure::Failed(failure))
        }
        Some(
            skein_kv::Event::Opened { .. }
            | skein_kv::Event::Committed { .. }
            | skein_kv::Event::Failed { .. }
            | skein_kv::Event::Loaded { .. }
            | skein_kv::Event::Closed { .. }
            | skein_kv::Event::Recovered { .. },
        ) => Err(JoinFailure::UnexpectedTerminal),
    }
}
