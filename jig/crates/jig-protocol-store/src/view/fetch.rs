//! Forward fetches retain only requested intervals; ended gets use their named frame.
use super::{Failure, Fetch, Task, covered, fail, raw_get};
use crate::{Below, Event, Store, codec, keys, logs, pages};
use alloc::boxed::Box;
use skein_kv::{Direction, LogRecord, Range};
use skein_lib::{List, Queue, Token, length::len_u64};
const OWNER: Token = Token::new(0x4849);

fn contains(range: &Range, key: &[u8]) -> bool {
    key >= range.start.as_ref()
        && match &range.end {
            Some(end) => key < end.as_ref(),
            None => true,
        }
}
fn wanted(ranges: &[Range], key: &[u8]) -> bool {
    for range in ranges {
        if contains(range, key) {
            return true;
        }
    }
    false
}
fn ended(range: &Range) -> bool {
    range.start.starts_with(&[1, 1, 6])
        && keys::logged(&range.start).is_some()
        && range.end == pages::past(&range.start)
}
pub(super) fn prepare(store: &mut Store) -> Result<(), Failure> {
    let active = store.view.active.as_ref().expect("held read");
    let mut selected = None;
    for (number, task) in &active.tasks {
        for range in &task.asked {
            if !covered(&task.covered, range) {
                selected = Some(*number);
                break;
            }
        }
        if selected.is_some() {
            break;
        }
    }
    let task = selected.expect("missing interval");
    let log = logs::history(task).ok_or(Failure::Corrupt)?;
    let extent = raw_get(store, logs::extent_key(log)).or(Err(Failure::Corrupt))?.ok_or(Failure::Corrupt)?;
    let extent = logs::extent(&extent).ok_or(Failure::Corrupt)?;
    let place = keys::encode_own(&keys::Own::History { task }, store.limits.store.key).ok_or(Failure::Corrupt)?;
    let place = raw_get(store, place).or(Err(Failure::Corrupt))?.ok_or(Failure::Corrupt)?;
    let place = codec::component::HistoryLog::decode(&place).or(Err(Failure::Corrupt))?;
    let active = store.view.active.as_mut().expect("held read");
    let known = active.tasks.get(&task).expect("missing task");
    let mut direct = true;
    for range in &known.asked {
        if !covered(&known.covered, range) {
            direct &= ended(range);
        }
    }
    let mut ranges = List::with_capacity(known.asked.len());
    for range in &known.asked {
        if !direct || !covered(&known.covered, range) {
            ranges.push(range.clone()).expect("counted intervals");
        }
    }
    let ranges = ranges.into_boxed();
    let mut remove = List::uncapped();
    for (key, _) in &active.rows {
        if wanted(&ranges, key) {
            remove.push(key.clone()).expect("fetched rows bound keys");
        }
    }
    for key in &remove {
        let value = active.rows.remove(key.as_ref()).expect("selected row");
        active.bytes = active
            .bytes
            .checked_sub(len_u64(key.len()).checked_add(len_u64(value.len())).ok_or(Failure::Corrupt)?)
            .ok_or(Failure::Corrupt)?;
    }
    if direct && place.record.is_none() {
        return cover(active.tasks.get_mut(&task).expect("missing task"), ranges, true);
    }
    let at = if direct { place.record.ok_or(Failure::Corrupt)? } else { 0 };
    if at >= extent {
        return Err(Failure::Corrupt);
    }
    store.view.fetch = Some(Fetch { task, at, extent, ended: place.record, direct, ranges, waiting: false });
    Ok(())
}

pub(super) fn advance(store: &mut Store, above: &mut Queue<Event>, below: &mut Below) -> bool {
    let fetch = store.view.fetch.as_mut().expect("fetch in progress");
    if fetch.waiting {
        return false;
    }
    fetch.waiting = true;
    let request = skein_kv::Request::ReadLog {
        owner: OWNER,
        log: logs::history(fetch.task).expect("checked log id"),
        at: fetch.at,
        direction: Direction::Forward,
        max: store.limits.store.page.bytes,
    };
    let mut events = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        request,
        &mut events,
        &mut below.store,
    );
    for _ in 0..events.len() {
        let Some(event) = events.pop() else { break };
        match event {
            skein_kv::Event::LogRead { records, next, .. } => read(store, records, next, above),
            skein_kv::Event::LogUnread { .. } | skein_kv::Event::Refused { .. } => fail(store, Failure::Corrupt, above),
            skein_kv::Event::Opened { .. }
            | skein_kv::Event::Committed { .. }
            | skein_kv::Event::Failed { .. }
            | skein_kv::Event::Got { .. }
            | skein_kv::Event::Loaded { .. }
            | skein_kv::Event::Closed { .. }
            | skein_kv::Event::OpenFailed { .. }
            | skein_kv::Event::Recovered { .. }
            | skein_kv::Event::RecoveryFailed { .. } => unreachable!("a log read has only log terminals"),
        }
    }
    match &store.view.fetch {
        Some(fetch) => !fetch.waiting,
        None => true,
    }
}

pub(crate) fn read(store: &mut Store, records: Box<[LogRecord]>, next: Option<u64>, above: &mut Queue<Event>) {
    if let Err(failure) = accept(store, records, next) {
        fail(store, failure, above);
    }
}
pub(crate) fn unread(store: &mut Store, above: &mut Queue<Event>) {
    fail(store, Failure::Corrupt, above);
}
fn cover(task: &mut Task, ranges: Box<[Range]>, keep: bool) -> Result<(), Failure> {
    let count = u32::try_from(ranges.len()).or(Err(Failure::Corrupt))?;
    let count = count.checked_add(if keep { task.covered.len() } else { 0 }).ok_or(Failure::Corrupt)?;
    let old = core::mem::replace(&mut task.covered, List::with_capacity(count));
    if keep {
        for range in old.into_boxed() {
            task.covered.push(range).expect("counted previous intervals");
        }
    }
    for range in ranges {
        task.covered.push(range).expect("counted fetched intervals");
    }
    Ok(())
}
fn accept(store: &mut Store, records: Box<[LogRecord]>, next: Option<u64>) -> Result<(), Failure> {
    let fetch = store.view.fetch.as_mut().ok_or(Failure::Corrupt)?;
    fetch.waiting = false;
    if records.is_empty() {
        return Err(Failure::Corrupt);
    }
    let active = store.view.active.as_mut().ok_or(Failure::Corrupt)?;
    for record in records {
        if record.at != fetch.at || record.end <= record.at || record.end > fetch.extent {
            return Err(Failure::Corrupt);
        }
        let body = record.record.ok_or(Failure::Corrupt)?;
        let rows = logs::record::decode(&body).ok_or(Failure::Corrupt)?;
        for (key, value) in rows {
            if keys::logged(&key) != Some(fetch.task) {
                return Err(Failure::Corrupt);
            }
            if !wanted(&fetch.ranges, &key) {
                continue;
            }
            if key.starts_with(&[1, 1, 6]) && fetch.ended != Some(record.at) {
                continue;
            }
            if let Some(old) = active.rows.remove(key.as_ref()) {
                active.bytes = active
                    .bytes
                    .checked_sub(len_u64(key.len()).checked_add(len_u64(old.len())).ok_or(Failure::Corrupt)?)
                    .ok_or(Failure::Corrupt)?;
            }
            if let Some(value) = value {
                let needs = active
                    .bytes
                    .checked_add(len_u64(key.len()))
                    .ok_or(Failure::Corrupt)?
                    .checked_add(len_u64(value.len()))
                    .ok_or(Failure::Corrupt)?;
                if needs > store.limits.fetch {
                    return Err(Failure::Bounds { needs, key });
                }
                active.bytes = needs;
                active.rows.insert(key, value).expect("fetched bytes bound rows");
            }
        }
        fetch.at = record.end;
        if fetch.direct {
            break;
        }
    }
    if fetch.direct || next.is_none() {
        if !fetch.direct && fetch.at != fetch.extent {
            return Err(Failure::Corrupt);
        }
        let fetch = store.view.fetch.take().expect("completed fetch");
        let task = active.tasks.get_mut(&fetch.task).expect("registered task");
        cover(task, fetch.ranges, fetch.direct)?;
    } else if next != Some(fetch.at) {
        return Err(Failure::Corrupt);
    }
    Ok(())
}
