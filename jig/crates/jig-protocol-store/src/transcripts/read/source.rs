//! File places win; otherwise read durable turns-log records (protocol/store/logs.md, 5).
use super::{
    Below, Decoded, Event, Queue, Read, Step, Store, Token, TurnRecord, codec, frame, keys, next, party, patching,
    settle,
};
use crate::{logs, open};
use alloc::boxed::Box;

pub(crate) const OWNER: Token = Token::new(0x5456);
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
        | Step::Frame { .. }
        | Step::Site { .. } => Err(()),
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
    let ready = record(store, &mut read, records, above, below);
    settle(store, owner, read, ready, above);
}

fn record(
    store: &mut Store,
    read: &mut Read,
    records: Option<Box<[skein_kv::LogRecord]>>,
    above: &mut Queue<Event>,
    below: &mut Below,
) -> Result<bool, ()> {
    let Step::Log { max, waiting: true } = read.step else { return Err(()) };
    // Each decode may stop at a written patch site. Only its first record is
    // consumed; the next demand starts at that record's opposite edge.
    let record = records.ok_or(())?.into_iter().next().ok_or(())?;
    if record.end > read.end
        || record.at >= record.end
        || (if read.backwards { record.end } else { record.at }) != read.position
    {
        return Err(());
    }
    let Some(body) = record.record else {
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
    match frame::Decode::from_body(&body, &mut store.transcripts.machines, &store.codec_limits).or(Err(()))? {
        frame::Decode::Done { turn, size } => complete(store, read, record.at, length, turn, size, below),
        frame::Decode::Patching(patch) => patching(store, read, record.at, length, patch, above, below),
    }
}

fn complete(
    store: &mut Store,
    read: &mut Read,
    offset: u64,
    length: u32,
    mut turn: TurnRecord,
    size: u32,
    below: &mut Below,
) -> Result<bool, ()> {
    let charged = store.transcripts.machines.body.is_some() || store.transcripts.machines.turn.is_some();
    let transcript = core::mem::replace(&mut turn.transcript, Box::new([]));
    let payload = u32::try_from(transcript.len()).or(Err(()))?;
    if charged && let Some(party) = &mut read.party {
        party.frame_bytes = Some(size);
    }
    let cost = if charged { size } else { payload };
    if party::single(read) && party::header(store, read, &turn, cost)? {
        return Ok(true);
    }
    if let Some(page) = &mut read.activity {
        let selected = match read.scan {
            super::Scan::Turn { attempt, turn: number } => turn.attempt == attempt && turn.turn == number,
            super::Scan::Page => true,
            super::Scan::Window { .. } => return Err(()),
        };
        if selected {
            if page.identity() != Some((turn.task, turn.attempt, turn.turn)) {
                return Err(());
            }
            let row = page.rows.get_mut(page.index).expect("pending row");
            row.related = Some(super::domain::Record::Core(super::domain::CoreRecord::Turn(turn.clone())));
            let bytes = row.owned_bytes().ok_or(())?.checked_add(u64::from(cost)).ok_or(())?;
            row.related = None;
            if bytes > page.bound {
                return super::sized(store, read, offset, length, turn, cost, below);
            }
            if bytes > page.bytes {
                return Ok(true);
            }
        }
    }
    turn.transcript = transcript;
    super::accept(store, read, Decoded { offset, length, turn, size, charge_frame: charged, cut: None }, below)
}
