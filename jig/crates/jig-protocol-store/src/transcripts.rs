//! Transcript frames, queued commits and durable places (protocol/store/transcripts.md).
pub(crate) mod cut;
pub(crate) mod drop;
pub mod frame;
pub(crate) mod patches;
pub(crate) mod read;
pub(crate) mod write;
use crate::{CommitCheck, CommitFailure, Event, Limits, Phase, Store, codec, keys, open};
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Token};
#[derive(Debug)]
pub(crate) struct Frame {
    pub task: u64,
    pub offset: u64,
    pub bytes: Box<[u8]>,
}
#[derive(Debug)]
pub(crate) struct Pending {
    pub number: u64,
    pub frames: Queue<Frame>,
    pub ops: Box<[skein_kv::Op]>,
    pub drops: Queue<u64>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Place {
    pub number: u64,
    pub row: codec::component::Transcript,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct File {
    pub token: Token,
    pub touched: u64,
    pub writable: bool,
    pub readable: bool,
}
#[derive(Debug)]
pub(crate) enum Stage {
    Idle,
    Opening,
    Evicting { task: u64 },
    Making,
    RootSync,
    DirectoryOpen,
    Creating,
    Writing,
    Syncing,
    DirectorySync,
    DirectoryClose,
    Dropping,
}
#[derive(Debug)]
pub(crate) struct State {
    pub queue: Queue<Pending>,
    pub current: Option<Pending>,
    pub frame: Option<Frame>,
    pub places: Map<u64, Place>,
    pub attempts: Map<(u64, u64), u64>,
    pub files: Map<u64, File>,
    pub stage: Stage,
    pub directory: Option<crate::Root>,
    pub touch: u64,
    pub reads: Map<Token, read::Read>,
    pub reading: Option<Token>,
    pub machines: patches::Machines,
    pub cut: cut::State,
    pub drop: drop::State,
}
impl State {
    pub fn new(limits: &Limits) -> State {
        let count = reservation_count(limits).expect("bounded transcript reservations");
        State {
            queue: Queue::with_capacity(limits.queue),
            current: None,
            frame: None,
            places: Map::with_capacity(count),
            attempts: Map::with_capacity(count),
            files: Map::with_capacity(limits.files),
            stage: Stage::Idle,
            directory: None,
            touch: 0,
            reads: Map::with_capacity(limits.reads),
            reading: None,
            machines: patches::Machines::new(),
            cut: cut::State::new(),
            drop: drop::State::new(),
        }
    }
}
pub(crate) fn reservation_count(limits: &Limits) -> Option<u32> {
    limits.queue.checked_add(1)?.checked_add(limits.store.queued)?.checked_mul(limits.commits.writes)
}
pub(crate) fn place(
    store: &mut Store,
    number: u64,
    turn: &jig_domain_core_vocabulary::TurnRecord,
    length: u64,
    ops: &mut List<skein_kv::Op>,
) -> Result<u64, CommitCheck> {
    let task = turn.task;
    let key = keys::encode_own(&keys::Own::Transcript { task }, store.limits.store.key).ok_or(CommitCheck::Encoding)?;
    let mut row = match store.transcripts.places.get(&task) {
        Some(place) => place.row,
        None => match open::get(store, Token::new(0), key.clone()).or(Err(CommitCheck::Reads))? {
            Some(bytes) => codec::component::Transcript::decode(&bytes).or(Err(CommitCheck::Reads))?,
            None => codec::component::Transcript { length: 0, window: 0 },
        },
    };
    let offset = row.length;
    row.length = row.length.checked_add(length).ok_or(CommitCheck::Capacity)?;
    if turn.opens_window {
        row.window = offset;
    }
    let _: Option<Place> =
        store.transcripts.places.insert(task, Place { number, row }).or(Err(CommitCheck::Capacity))?;
    ops.push(skein_kv::Op::Put { key, value: row.encode().or(Err(CommitCheck::Encoding))? })
        .or(Err(CommitCheck::Capacity))?;
    let attempt = (task, turn.attempt);
    if !store.transcripts.attempts.contains_key(&attempt) {
        let key = keys::encode_own(&keys::Own::Attempt { task, attempt: turn.attempt }, store.limits.store.key)
            .ok_or(CommitCheck::Encoding)?;
        if open::get(store, Token::new(0), key.clone()).or(Err(CommitCheck::Reads))?.is_none() {
            let value = codec::component::Attempt { first: offset }.encode().or(Err(CommitCheck::Encoding))?;
            ops.push(skein_kv::Op::Put { key, value }).or(Err(CommitCheck::Capacity))?;
            let _: Option<u64> = store.transcripts.attempts.insert(attempt, number).or(Err(CommitCheck::Capacity))?;
        }
    }
    Ok(offset)
}
pub(crate) fn committed(store: &mut Store, number: u64) {
    let mut tasks = List::with_capacity(store.transcripts.places.len());
    for (task, place) in &store.transcripts.places {
        if place.number <= number {
            tasks.push(*task).expect("bounded reservations");
        }
    }
    for task in tasks.into_boxed() {
        let _: Option<Place> = store.transcripts.places.remove(&task);
    }
    let mut attempts = List::with_capacity(store.transcripts.attempts.len());
    for (attempt, through) in &store.transcripts.attempts {
        if *through <= number {
            attempts.push(*attempt).expect("bounded attempts");
        }
    }
    for attempt in attempts.into_boxed() {
        let _: Option<u64> = store.transcripts.attempts.remove(&attempt);
    }
}
pub(crate) fn failed(store: &mut Store, number: Option<u64>, why: CommitFailure, above: &mut Queue<Event>) {
    store.phase = Phase::Stopped;
    if let Some(number) = number {
        above.push(Event::Failed { number, why });
    }
    if let Some(current) = store.transcripts.current.take() {
        above.push(Event::Failed {
            number: current.number,
            why: if number.is_some() { CommitFailure::Stopped } else { why },
        });
    }
    for _ in 0..store.limits.queue {
        if let Some(pending) = store.transcripts.queue.pop() {
            above.push(Event::Failed { number: pending.number, why: CommitFailure::Stopped });
        }
    }
}
#[must_use]
pub(crate) fn name(number: u64) -> Box<[u8]> {
    let mut bytes = skein_lib::Writer::new(16);
    for shift in (0_u32..16).rev() {
        let digit = u8::try_from((number >> shift.saturating_mul(4)) & 15).expect("one nibble");
        let byte = if digit < 10 {
            b'0'.checked_add(digit)
        } else {
            b'a'.checked_add(digit.checked_sub(10).expect("hex letter"))
        }
        .expect("hex digit");
        bytes.put(&[byte]).expect("sixteen hex digits");
    }
    bytes.finish()
}
fn path(task: u64) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(33);
    writer.put(&name(task >> 12)).expect("high digits");
    writer.put(b"/").expect("separator");
    writer.put(&name(task)).expect("task digits");
    writer.finish()
}
