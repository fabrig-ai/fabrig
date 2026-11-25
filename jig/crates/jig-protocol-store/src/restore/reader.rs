//! Read-only restore pages (jig/docs/design/protocol/store/loads.md, 1 and 2).
use super::{Arithmetic, Cause, DecodeFailure, Failure, JoinFailure, RangeFailure, RowSite, ready, site};
use crate::{Cursor, Restore, Row, Store, codec, intervals, keys, pages};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{Key, Record};
use jig_domain_tasks as tasks;
use skein_lib::{List, Token, length::len_u64};

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
    ready(store)?;
    let ranges = ranges(store, range)?;
    let walked = pages::reader::walk(store, &ranges, after, rows)?;
    let capacity = rows.min(store.limits.store.page.bytes);
    let mut found = List::with_capacity(capacity);
    let mut used = 0_u64;
    let mut cursor = None;
    for item in walked.rows {
        let key = &item.row.key;
        let (row, joined, size) = decode(store, key, item.row.value)?;
        let Some(total) = used.checked_add(size) else {
            return Err(Failure::at(Cause::Arithmetic(Arithmetic::PageBytes), site(store, key)));
        };
        if total > bytes && found.is_empty() {
            return Err(Failure::at(Cause::RowPastPage { needs: size, has: bytes }, site(store, key)));
        }
        if total > bytes || found.room() < if joined.is_some() { 2 } else { 1 } {
            let next = Some(pages::resume(cursor, after));
            super::assert_progress(after, found.is_empty(), next.as_ref());
            return Ok(Loaded { items: found.into_boxed(), next });
        }
        used = total;
        cursor = Some(item.cursor);
        found.push(row).expect("bounded restore items");
        if let Some(joined) = joined {
            found.push(joined).expect("pair room checked");
        }
    }
    super::assert_progress(after, found.is_empty(), walked.next.as_ref());
    Ok(Loaded { items: found.into_boxed(), next: walked.next })
}

fn ranges(store: &Store, range: Restore) -> Result<Box<[skein_kv::Range]>, Failure> {
    match range {
        Restore::Application(ranges) => {
            if u32::try_from(ranges.len()).is_err() {
                return Err(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)));
            }
            let Some(intervals) = pages::application(&ranges, store.limits.store.key) else {
                return Err(Failure::rowless(Cause::Range(RangeFailure::Encoding)));
            };
            Ok(intervals)
        }
        Restore::Jig(ranges) => match intervals::collect(&ranges) {
            Ok(intervals) => Ok(intervals),
            Err(intervals::CountFailure::Census | intervals::CountFailure::Total) => {
                Err(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))
            }
        },
    }
}

pub(crate) fn decode_row(store: &mut Store, raw: &[u8], value: Box<[u8]>) -> Result<(Row, u64), Cause> {
    if let Some(key) = keys::of_application(raw) {
        let key_bytes = len_u64(key.len());
        let value_bytes = len_u64(value.len());
        let Some(size) = key_bytes.checked_add(value_bytes) else {
            return Err(Cause::Arithmetic(Arithmetic::RowBytes));
        };
        return Ok((Row::Application { key: Box::from(key), value }, size));
    }
    let Some(key) = keys::decode(raw) else { return Err(Cause::Decode(DecodeFailure::Key)) };
    if let Key::Tasks(tasks::Key::Live(number)) = key {
        return live(store, number, &value);
    }
    let record = match decode_value(store, &key, &value) {
        Ok(record) => record,
        Err(failure) => return Err(Cause::Decode(failure)),
    };
    let size = len_u64(value.len());
    Ok((Row::Jig(record), size))
}

fn live(store: &mut Store, number: u64, hot: &[u8]) -> Result<(Row, u64), Cause> {
    let cold_key = Key::Tasks(tasks::Key::LiveCold(number));
    let Some(cold_key) = keys::encode(&cold_key, store.limits.store.key) else {
        return Err(Cause::Decode(DecodeFailure::Key));
    };
    let Ok(Some(cold)) = crate::open::get(store, Token::new(0), cold_key) else {
        return Err(Cause::Decode(DecodeFailure::Value));
    };
    let Ok(record) = codec::tasks::decode_live(hot, &cold, &store.codec_limits.tasks, &store.names) else {
        let key = Key::Tasks(tasks::Key::Live(number));
        return Err(Cause::Decode(value_failure(store, &key, hot, Some(&cold))));
    };
    if record.key() != tasks::Key::Live(number) {
        return Err(Cause::Decode(DecodeFailure::KeyValueDisagree));
    }
    let Some(size) = len_u64(hot.len()).checked_add(len_u64(cold.len())) else {
        return Err(Cause::Arithmetic(Arithmetic::JoinedBytes));
    };
    Ok((Row::Jig(Record::Tasks(record)), size))
}

fn value_failure(store: &Store, key: &Key, value: &[u8], cold: Option<&[u8]>) -> DecodeFailure {
    let Some(stock) = &store.stock_core else { return DecodeFailure::Value };
    match codec::lowering(key, value, cold, &store.limits.core, stock, &store.names) {
        Some(lowered) => DecodeFailure::Lowered(lowered),
        None => DecodeFailure::Value,
    }
}

fn decode_value(store: &Store, key: &Key, value: &[u8]) -> Result<Record, DecodeFailure> {
    let Some(record) = codec::decode(key, value, &store.codec_limits, &store.names) else {
        return Err(value_failure(store, key, value, None));
    };
    if record.key() != *key {
        return Err(DecodeFailure::KeyValueDisagree);
    }
    Ok(record)
}

fn decode(store: &mut Store, key: &[u8], value: Box<[u8]>) -> Result<(Item, Option<Item>, u64), Failure> {
    let (row, size) = match decode_row(store, key, value) {
        Ok(decoded) => decoded,
        Err(cause) => return Err(Failure::at(cause, site(store, key))),
    };
    let task = match &row {
        Row::Jig(Record::Tasks(tasks::Stored::Result(row))) => row.task,
        Row::Jig(Record::People(jig_domain_people::Stored::EndedRead { task, .. })) => *task,
        Row::Jig(_) | Row::Application { .. } => return Ok((Item::Row(row), None, size)),
    };
    let target = Key::Tasks(tasks::Key::Ended(task));
    let owner = ended(store, &target, &site(store, key))?;
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
            let Some(joined) = size.checked_add(owner_size) else {
                return Err(Failure::at(Cause::Arithmetic(Arithmetic::JoinedBytes), site(store, key)));
            };
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
    let value = match crate::view::get(store, encoded) {
        Ok(value) => value,
        Err(failure) => return Err(Failure::at(Cause::Join { target, failure }, source.clone())),
    };
    let Some(mut value) = value else { return Ok(None) };
    crate::registry::read(store, &raw, &mut value);
    let record = match decode_value(store, key, &value) {
        Ok(record) => record,
        Err(failure) => {
            return Err(Failure::at(Cause::Join { target, failure: JoinFailure::Decode(failure) }, source.clone()));
        }
    };
    let size = len_u64(value.len());
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
            | skein_kv::Event::LogRead { .. }
            | skein_kv::Event::LogUnread { .. }
            | skein_kv::Event::Loaded { .. }
            | skein_kv::Event::Held { .. }
            | skein_kv::Event::Released { .. }
            | skein_kv::Event::Closed { .. }
            | skein_kv::Event::Recovered { .. },
        ) => Err(JoinFailure::UnexpectedTerminal),
    }
}
