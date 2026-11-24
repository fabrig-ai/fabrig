//! File places win; otherwise read durable turns-log records (protocol/store/logs.md, 5).
use super::{Below, Event, Queue, Read, Step, Store, Token, frame, keys, next, settle};
use crate::{codec, logs, open};
use alloc::boxed::Box;

pub(crate) const OWNER: Token = Token::new(0x5456);

pub(crate) fn waiting(store: &Store) -> bool {
    if let Some(owner) = store.transcripts.reading
        && let Some(read) = store.transcripts.reads.get(&owner)
        && let Step::Log { waiting: true, .. } = read.step
    {
        return true;
    }
    false
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Source {
    File,
    Log(u64),
}

pub(super) fn place(
    store: &mut Store,
    owner: Token,
    task: u64,
) -> Result<Option<(Source, codec::component::Transcript)>, ()> {
    let key = keys::encode_own(&keys::Own::Transcript { task }, store.limits.store.key).ok_or(())?;
    if let Some(row) = open::get(store, owner, key).or(Err(()))? {
        return Ok(Some((Source::File, codec::component::Transcript::decode(&row).or(Err(()))?)));
    }
    let Some(log) = logs::turns(task) else { return Ok(None) };
    let key = keys::encode_own(&keys::Own::TurnLog { task }, store.limits.store.key).ok_or(())?;
    let row = open::get(store, owner, key).or(Err(()))?;
    let extent = open::get(store, owner, logs::extent_key(log)).or(Err(()))?;
    if row.is_none() && extent.is_none() {
        return Ok(None);
    }
    let length = match extent {
        Some(bytes) => logs::extent(&bytes).ok_or(())?,
        None => 0,
    };
    let window = match row {
        Some(bytes) => codec::component::TurnLog::decode(&bytes).or(Err(()))?.window.unwrap_or(length),
        None => length,
    };
    Ok(Some((Source::Log(log), codec::component::Transcript { length, window })))
}

pub(super) fn attempt(store: &mut Store, owner: Token, source: Source, task: u64, attempt: u64) -> Option<u64> {
    let own = match source {
        Source::File => keys::Own::Attempt { task, attempt },
        Source::Log(_) => keys::Own::LogAttempt { task, attempt },
    };
    let key = keys::encode_own(&own, store.limits.store.key)?;
    let row = open::get(store, owner, key).ok()??;
    match source {
        Source::File => Some(codec::component::Attempt::decode(&row).ok()?.first),
        Source::Log(_) => Some(codec::component::LogAttempt::decode(&row).ok()?.first),
    }
}

pub(super) fn start(store: &mut Store, owner: Token, mut read: Read, above: &mut Queue<Event>, below: &mut Below) {
    let ready = match read.step {
        Step::Log { waiting: false, .. } => Ok(false),
        Step::Waiting => next(store, &mut read, below),
        Step::Log { waiting: true, .. }
        | Step::Opening
        | Step::Evicting { .. }
        | Step::Length { .. }
        | Step::Header { .. }
        | Step::Sized { .. }
        | Step::Frame { .. } => Err(()),
    };
    settle(store, owner, read, ready, above);
    if ready != Ok(false) {
        return;
    }
    let read = store.transcripts.reads.get_mut(&owner).expect("scheduled log reader");
    let Step::Log { max, waiting } = &mut read.step else { unreachable!("log next schedules a record") };
    *waiting = true;
    store.transcripts.reading = Some(owner);
    let Source::Log(log) = read.source else { unreachable!("log reader") };
    let request = skein_kv::Request::ReadLog {
        owner: OWNER,
        log,
        at: read.position,
        direction: if read.backwards { skein_kv::Direction::Backward } else { skein_kv::Direction::Forward },
        max: *max,
        rows: 1,
    };
    let mut events = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        request,
        &mut events,
        &mut below.store,
    );
    crate::directory::events_up(store, false, &mut events, above, below);
}

pub(crate) fn arrived(
    store: &mut Store,
    records: Option<Box<[skein_kv::LogRecord]>>,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let Some(owner) = store.transcripts.reading else { return };
    let mut read = store.transcripts.reads.remove(&owner).expect("active log reader");
    let ready = record(store, &mut read, records, below);
    settle(store, owner, read, ready, above);
}

fn record(
    store: &mut Store,
    read: &mut Read,
    records: Option<Box<[skein_kv::LogRecord]>>,
    below: &mut Below,
) -> Result<bool, ()> {
    let Step::Log { max, waiting: true } = read.step else { return Err(()) };
    // Demand one body at a time, so a later record cannot fail this decode.
    let mut records = records.ok_or(())?.into_iter();
    let record = records.next().ok_or(())?;
    if records.next().is_some() {
        return Err(());
    }
    if record.end > read.end
        || record.at >= record.end
        || (if read.backwards { record.end } else { record.at }) != read.position
    {
        return Err(());
    }
    let Some(mut body) = record.record else {
        if max == store.limits.store.page.bytes {
            return Err(());
        }
        read.step = Step::Log { max: store.limits.store.page.bytes, waiting: false };
        return Ok(false);
    };
    if skein_lib::len_u64(body.len()) != u64::from(record.length) {
        return Err(());
    }
    let length = u32::try_from(record.end.checked_sub(record.at).ok_or(())?).or(Err(()))?;
    let (turn, size) = frame::read_body(store, &mut body).or(Err(()))?;
    super::complete(store, read, record.at, length, turn, size, below)
}
