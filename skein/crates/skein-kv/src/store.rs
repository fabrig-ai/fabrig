//! The store's transitions (skein/docs/design/data/kv.md, 2–5).
//! At most one file request is outstanding.

use alloc::boxed::Box;
#[expect(
    clippy::disallowed_types,
    reason = "segments follow directory listings; recovery holds at most Limits::commit plus one read"
)]
use alloc::vec::Vec;
use core::mem;
use skein_io::file;
use skein_io::kernel::{Error, Fd};
use skein_lib::{Env, Queue, Token, usize_of};

use crate::boundary::{Event, Exhausted, Failure, Large, Op, Page, Range, Refusal, Request};
use crate::frame;
use crate::map::{KvMap, Queued};
use crate::snapshot;
use crate::{Census, CensusRule, Limits};

mod compaction;

use compaction::Compaction;

const IO_OWNER: Token = Token::new(0x4b56);

#[derive(Debug)]
struct Pending {
    owner: Token,
    number: u64,
    ops: Box<[Op]>,
    frame: Box<[u8]>,
}

#[expect(clippy::disallowed_types, reason = "replay retains at most Limits::commit plus one read")]
#[derive(Debug)]
enum Phase {
    Listing,
    OpeningLog { name: Box<[u8]>, next: usize },
    ReadingLog { file: Token, len: u64, at: u64, bytes: Vec<u8>, next: usize },
    ClosingRecovered { next: usize },
    RemovingTail { name: Box<[u8]> },
    Creating { name: Box<[u8]> },
    Announcing { file: Token },
    Ready { log: Log },
    Writing { log: Log, batch: Queue<Pending> },
    Syncing { log: Log, batch: Queue<Pending> },
    Closing,
    RecoveryClosing,
    RolloverClosing,
    SnapIo { log: Log },
    SnapshotFailureClosing { log: Log },
    OpenFailureClosing { failure: Failure },
    OpeningSnapshot,
    ReadingSnapshot { file: Token, len: u64, at: u64, reader: snapshot::Reader },
    ClosingSnapshot { next: usize },
    RemovingTempOnOpen { snapshot: bool },
}

impl Phase {
    /// The open log, when no file request is outstanding and none must be.
    fn ready(&self) -> Option<Log> {
        match self {
            Self::Ready { log } => Some(*log),
            Self::Listing
            | Self::OpeningLog { .. }
            | Self::ReadingLog { .. }
            | Self::ClosingRecovered { .. }
            | Self::RemovingTail { .. }
            | Self::Creating { .. }
            | Self::Announcing { .. }
            | Self::Writing { .. }
            | Self::Syncing { .. }
            | Self::Closing
            | Self::RecoveryClosing
            | Self::RolloverClosing
            | Self::SnapIo { .. }
            | Self::SnapshotFailureClosing { .. }
            | Self::OpenFailureClosing { .. }
            | Self::OpeningSnapshot
            | Self::ReadingSnapshot { .. }
            | Self::ClosingSnapshot { .. }
            | Self::RemovingTempOnOpen { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Log {
    file: Token,
    offset: u64,
}

/// One store under one directory. It sends file requests below and events
/// above; the caller's queues may be uncapped.
#[derive(Debug)]
pub struct Store {
    state: State,
    repair: Repair,
    last: u64,
    map: KvMap,
    rule: Option<Box<CensusRule>>,
    queue: Queue<Pending>,
    queued_bytes: u64,
    admitted: Queued,
}

/// The store's life (skein/docs/foundation/programming-model.md, 5.4).
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::large_enum_variant,
        reason = "the open state owns its directory and phase inline without another allocation"
    )
)]
#[derive(Debug)]
enum State {
    Closed,
    Memory,
    Open { dir: Dir, phase: Phase },
}

/// What an open directory holds in every phase.
#[derive(Debug)]
struct Dir {
    root: Fd,
    serving: Serving,
    close_owner: Option<Token>,
    #[expect(
        clippy::disallowed_types,
        reason = "segment names hold the bounded directory listing plus logs since the snapshot's start"
    )]
    segments: Vec<Box<[u8]>>,
    compaction: Compaction,
    log_bytes: u64,
}

/// Who hears when the store serves (skein/docs/design/data/kv.md, 2).
#[derive(Debug)]
enum Serving {
    Yes,
    Opening { owner: Token },
    Recovering,
}

/// Whether recovery needs a snapshot before serving (skein/docs/design/data/kv.md, 3 and 4).
#[derive(Debug)]
enum Repair {
    Clean,
    Required,
}

impl Dir {
    #[expect(
        clippy::disallowed_types,
        reason = "segment names hold the bounded directory listing plus logs since the snapshot's start"
    )]
    fn new(root: Fd, owner: Token) -> Dir {
        Dir {
            root,
            serving: Serving::Opening { owner },
            close_owner: None,
            segments: Vec::new(),
            compaction: Compaction::Idle,
            log_bytes: 0,
        }
    }

    fn at(self, phase: Phase) -> State {
        State::Open { dir: self, phase }
    }
}

/// What the one outstanding request came back with.
enum Done {
    Listed { entries: Box<[file::Entry]> },
    Opened { file: Token, len: u64 },
    Read { bytes: Box<[u8]> },
    Finished,
    Failed { error: Error },
}

fn done(event: file::Event) -> Done {
    match event {
        file::Event::Listed { entries, .. } => Done::Listed { entries },
        file::Event::Opened { file, len, .. } => Done::Opened { file, len },
        file::Event::Read { bytes, .. } => Done::Read { bytes },
        file::Event::Written { .. }
        | file::Event::Synced { .. }
        | file::Event::Closed { .. }
        | file::Event::Removed { .. }
        | file::Event::Linked { .. }
        | file::Event::Renamed { .. } => Done::Finished,
        file::Event::Failed { error, .. } => Done::Failed { error },
        file::Event::Cancelled { .. } => unreachable!("the store never cancels a file request"),
        file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Cut { .. } => unreachable!("the store sends only streaming file and directory requests"),
    }
}

/// The inputs and outputs borrowed for one transition, never kept in state.
struct Step<'a> {
    env: &'a Env<Limits>,
    above: &'a mut Queue<Event>,
    below: &'a mut Queue<file::Request>,
}

impl Store {
    #[must_use]
    pub fn new(limits: &Limits) -> Store {
        assert!(limits.is_usable(), "a store needs usable limits");
        Store {
            state: State::Closed,
            repair: Repair::Clean,
            last: 0,
            map: KvMap::new(limits),
            rule: None,
            queue: Queue::with_capacity(limits.queued),
            queued_bytes: 0,
            admitted: Queued::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn projected_keys(&self) -> u32 {
        self.admitted.keys()
    }

    #[cfg(test)]
    pub(crate) fn exhaust_numbers(&mut self) {
        self.last = u64::MAX;
    }

    /// An in-memory store for protocol tests; commits are applied at once.
    #[must_use]
    pub fn memory(limits: &Limits) -> Store {
        let mut store = Store::new(limits);
        store.state = State::Memory;
        store
    }

    /// Enable the census before opening, or before the first in-memory commit
    /// (skein/docs/design/data/kv.md, 2.2). Prefix lengths are clamped to `1..=CENSUS_PREFIX`.
    pub fn count(&mut self, rule: CensusRule) {
        assert!(self.last == 0 && self.map.len() == 0, "counting starts before the map is loaded");
        self.rule = Some(Box::new(rule));
    }

    #[must_use]
    pub fn census(&self) -> &Census {
        &self.map.census
    }

    #[must_use]
    pub const fn last(&self) -> u64 {
        self.last
    }

    #[must_use]
    pub fn used(&self) -> u64 {
        self.map.used()
    }

    #[must_use]
    pub fn len(&self) -> u32 {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
    }

    fn next_number(&self, phase: &Phase) -> Option<u64> {
        let in_flight = match phase {
            Phase::Writing { batch, .. } | Phase::Syncing { batch, .. } => u64::from(batch.len()),
            Phase::Listing
            | Phase::OpeningLog { .. }
            | Phase::ReadingLog { .. }
            | Phase::ClosingRecovered { .. }
            | Phase::RemovingTail { .. }
            | Phase::Creating { .. }
            | Phase::Announcing { .. }
            | Phase::Ready { .. }
            | Phase::Closing
            | Phase::RecoveryClosing
            | Phase::RolloverClosing
            | Phase::SnapIo { .. }
            | Phase::SnapshotFailureClosing { .. }
            | Phase::OpenFailureClosing { .. }
            | Phase::OpeningSnapshot
            | Phase::ReadingSnapshot { .. }
            | Phase::ClosingSnapshot { .. }
            | Phase::RemovingTempOnOpen { .. } => 0,
        };
        self.last.checked_add(in_flight)?.checked_add(u64::from(self.queue.len()))?.checked_add(1)
    }

    fn start_batch(&mut self, dir: Dir, log: Log, limits: &Limits, below: &mut Queue<file::Request>) -> State {
        if self.queue.is_empty() {
            return dir.at(Phase::Ready { log });
        }
        let Log { file, offset } = log;
        if offset >= limits.segment {
            below.push(file::Request::Close { owner: IO_OWNER, file });
            return dir.at(Phase::RolloverClosing);
        }
        let mut batch = Queue::with_capacity(limits.queued);
        let total = usize::try_from(self.queued_bytes).expect("batch bytes fit usize");
        let mut bytes = skein_lib::Writer::new(total);
        for _ in 0..self.queue.len() {
            let pending = self.queue.pop().expect("the batch takes the queue's initial length");
            bytes.put(&pending.frame).expect("batch allocation holds every queued frame");
            batch.push(pending);
        }
        self.queued_bytes = 0;
        below.push(file::Request::WriteAt { owner: IO_OWNER, file, offset, bytes: bytes.finish() });
        dir.at(Phase::Writing { log, batch })
    }

    fn recover(&mut self, mut dir: Dir, step: &mut Step<'_>) -> State {
        self.last = 0;
        self.map = KvMap::new(&step.env.limits);
        dir.segments.clear();
        dir.compaction = Compaction::Idle;
        dir.log_bytes = 0;
        step.below.push(file::Request::List { owner: IO_OWNER, root: dir.root });
        dir.at(Phase::Listing)
    }

    fn finish_recovery(&mut self, mut dir: Dir, step: &mut Step<'_>) -> State {
        let next = self.last.checked_add(1).expect("commit number did not overflow");
        match self.repair {
            Repair::Clean => {}
            Repair::Required => dir.compaction = Compaction::Due { start: next },
        }
        let name = segment_name(next);
        for (index, entry) in dir.segments.iter().enumerate() {
            if entry.as_ref() == name.as_ref() {
                step.below.push(file::Request::Remove {
                    owner: IO_OWNER,
                    root: dir.root,
                    name: dir.segments.remove(index),
                    directory: false,
                });
                return dir.at(Phase::RemovingTail { name });
            }
        }
        create_segment(dir, name, step.below)
    }

    fn next_log(&mut self, mut dir: Dir, next: usize, step: &mut Step<'_>) -> State {
        let Some(name) = dir.segments.get(next).cloned() else {
            if dir.log_bytes >= step.env.limits.snapshot_bound(self.map.live()) {
                // Recovery creates the snapshot's starting segment without another rollover.
                dir.compaction =
                    Compaction::Due { start: self.last.checked_add(1).expect("commit number did not overflow") };
            }
            return self.finish_recovery(dir, step);
        };
        let next = next.checked_add(1).expect("segment index fits");
        step.below.push(file::Request::OpenRead { owner: IO_OWNER, root: dir.root, name: name.clone() });
        dir.at(Phase::OpeningLog { name, next })
    }

    fn start_recovery(&mut self, dir: Dir, snapshot: bool, step: &mut Step<'_>) -> State {
        if snapshot {
            step.below.push(file::Request::OpenRead {
                owner: IO_OWNER,
                root: dir.root,
                name: Box::from(&b"snapshot"[..]),
            });
            dir.at(Phase::OpeningSnapshot)
        } else {
            self.next_log(dir, 0, step)
        }
    }

    fn apply_snapshot(&mut self, reader: &mut snapshot::Reader, read: &[u8], limits: &Limits) -> Result<(), Failure> {
        let mut input = read;
        // Each part consumes bytes of this read; an incomplete part ends the pass.
        for _ in 0..read.len() {
            if input.is_empty() {
                break;
            }
            let part = reader.next(&mut input, limits.chunk).ok_or(Failure::Corrupt)?;
            match part {
                snapshot::Part::NeedMore => break,
                snapshot::Part::Start { start } => {
                    self.last = start.checked_sub(1).expect("the checked snapshot starts past zero");
                    self.map.census.restart(start);
                }
                snapshot::Part::Rows { rows } => {
                    for row in rows {
                        let op = Op::Put { key: row.key, value: row.value };
                        let Ok(used) = self.map.check(core::slice::from_ref(&op), limits) else {
                            return Err(Failure::Full);
                        };
                        self.map.apply(core::slice::from_ref(&op), used, self.rule.as_deref(), false);
                    }
                }
                snapshot::Part::End => {}
            }
        }
        Ok(())
    }

    #[expect(clippy::disallowed_types, reason = "replay retains at most Limits::commit plus one read")]
    fn replay(&mut self, dir: &mut Dir, bytes: &mut Vec<u8>, limits: &Limits) -> Result<bool, Failure> {
        let mut at: usize = 0;
        // Each complete frame consumes bytes; an incomplete frame ends the pass.
        for _ in 0..bytes.len() {
            if at == bytes.len() {
                break;
            }
            let Some(head_end) = at.checked_add(usize_of(frame::HEADER)) else {
                return Ok(false);
            };
            let Some(head) = bytes.get(at..head_end) else {
                break;
            };
            let Some(length) = frame::frame_len(head, limits.commit) else {
                return Ok(false);
            };
            let Some(end) = at.checked_add(length) else { return Ok(false) };
            let Some(encoded) = bytes.get(at..end) else {
                break;
            };
            let Some(decoded) = frame::decode(encoded, limits.commit) else {
                return Ok(false);
            };
            if decoded.ops.len() > usize_of(limits.ops) {
                return Ok(false);
            }
            if decoded.number != self.last.checked_add(1).ok_or(Failure::Corrupt)? {
                return Ok(false);
            }
            let Ok(used) = self.map.check(&decoded.ops, limits) else {
                return Err(Failure::Full);
            };
            self.map.apply(&decoded.ops, used, self.rule.as_deref(), true);
            self.last = decoded.number;
            dir.log_bytes = dir.log_bytes.checked_add(skein_lib::len_u64(length)).expect("log bytes fit u64");
            at = end;
        }
        // Keep only the unapplied suffix between reads, moving it once per
        // read rather than once per frame; the applied allocation is released.
        if at != 0 {
            *bytes = bytes.split_off(at);
        }
        frame::reserve_frame(bytes, limits.commit);
        Ok(true)
    }
}

fn segment_name(number: u64) -> Box<[u8]> {
    let mut name = *b"log-00000000000000000000";
    let mut number = number;
    for pos in (4..name.len()).rev() {
        let digit = u8::try_from(number % 10).expect("decimal digit fits byte");
        *name.get_mut(pos).expect("position is inside segment name") =
            b'0'.checked_add(digit).expect("digit fits ASCII");
        number /= 10;
    }
    Box::from(name)
}

fn log_number(name: &[u8]) -> Option<u64> {
    if name.len() != 24 || name.get(..4)? != b"log-" {
        return None;
    }
    let mut number = 0_u64;
    for &byte in name.get(4..)? {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}

/// What a recovery read asks: the target, or what the file holds past `at` (data/kv.md, 3).
fn read_size(target: u32, len: u64, at: u64) -> u32 {
    let left = len.checked_sub(at).expect("recovery reads only within the file");
    u32::try_from(u64::from(target).min(left)).expect("a recovery read is at most the target")
}

/// Where this read ends, if it is a read this file can give.
fn read_end(source: &ReadAt, read: &[u8]) -> Option<u64> {
    let end = source.at.checked_add(skein_lib::len_u64(read.len()))?;
    if read.is_empty() || end > source.len { None } else { Some(end) }
}

/// Ask the next read of `file`, at `at`.
fn read_next(file: Token, len: u64, at: u64, step: &mut Step<'_>) {
    step.below.push(file::Request::ReadAt {
        owner: IO_OWNER,
        file,
        offset: at,
        max: read_size(step.env.limits.chunk_target(), len, at),
    });
}

/// The owner's request.
pub fn down(
    store: &mut Store,
    env: &Env<Limits>,
    request: Request,
    above: &mut Queue<Event>,
    below: &mut Queue<file::Request>,
) {
    let state = mem::replace(&mut store.state, State::Closed);
    let mut step = Step { env, above, below };
    store.state = match request {
        Request::Open { owner, root } => match state {
            State::Closed | State::Memory => store.recover(Dir::new(root, owner), &mut step),
            state @ State::Open { .. } => {
                step.above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
                state
            }
        },
        Request::Commit { owner, ops } => match state {
            State::Closed => {
                step.above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
                State::Closed
            }
            State::Memory => {
                commit_memory(store, &env.limits, owner, ops, step.above);
                State::Memory
            }
            State::Open { dir, phase } => commit_disk(store, dir, phase, owner, ops, &mut step),
        },
        Request::Get { owner, key } => {
            let available = match &state {
                State::Closed => false,
                State::Memory => true,
                State::Open { dir, phase } => available(dir, phase),
            };
            get(store, owner, key, available, &mut step);
            state
        }
        Request::Load { owner, range, max } => {
            let available = match &state {
                State::Closed => false,
                State::Memory => true,
                State::Open { dir, phase } => available(dir, phase),
            };
            load(store, owner, range, max, available, &mut step);
            state
        }
        Request::Close { owner } => match state {
            State::Closed => {
                step.above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
                State::Closed
            }
            State::Memory => {
                step.above.push(Event::Closed { owner });
                State::Closed
            }
            State::Open { dir, phase } => close_requested(store, dir, phase, owner, &mut step),
        },
    };
}

fn available(dir: &Dir, phase: &Phase) -> bool {
    match dir.serving {
        Serving::Opening { .. } | Serving::Recovering => false,
        Serving::Yes => match phase {
            Phase::Ready { .. }
            | Phase::Writing { .. }
            | Phase::Syncing { .. }
            | Phase::RolloverClosing
            | Phase::Creating { .. }
            | Phase::Announcing { .. }
            | Phase::SnapIo { .. } => true,
            Phase::Listing
            | Phase::OpeningLog { .. }
            | Phase::ReadingLog { .. }
            | Phase::ClosingRecovered { .. }
            | Phase::RemovingTail { .. }
            | Phase::Closing
            | Phase::RecoveryClosing
            | Phase::OpeningSnapshot
            | Phase::ReadingSnapshot { .. }
            | Phase::ClosingSnapshot { .. }
            | Phase::RemovingTempOnOpen { .. }
            | Phase::SnapshotFailureClosing { .. }
            | Phase::OpenFailureClosing { .. } => false,
        },
    }
}

fn begin_batch(store: &mut Store, dir: Dir, phase: Phase, step: &mut Step<'_>) -> State {
    match phase.ready() {
        Some(log) => store.start_batch(dir, log, &step.env.limits, step.below),
        None => dir.at(phase),
    }
}

fn commit_disk(store: &mut Store, dir: Dir, phase: Phase, owner: Token, ops: Box<[Op]>, step: &mut Step<'_>) -> State {
    if !available(&dir, &phase) || dir.close_owner.is_some() {
        step.above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
        return dir.at(phase);
    }
    let Step { env, above, below: _ } = step;
    if store.queue.room() == 0 {
        above.push(Event::Refused { owner, refusal: Refusal::Busy });
        return dir.at(phase);
    }
    let Some(number) = store.next_number(&phase) else {
        above.push(Event::Refused { owner, refusal: Refusal::Full(Exhausted::Numbers) });
        return dir.at(phase);
    };
    if let Some(large) = too_large(&ops, &env.limits) {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(large) });
        return dir.at(phase);
    }
    let Some(bytes) = frame::encode(number, &ops, env.limits.commit) else {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(Large::Frame) });
        return dir.at(phase);
    };
    let projected = match store.admitted.check(&store.map, &ops, &env.limits) {
        Ok(projected) => projected,
        Err(refusal) => {
            above.push(Event::Refused { owner, refusal });
            return dir.at(phase);
        }
    };
    let total = store.queued_bytes.checked_add(skein_lib::len_u64(bytes.len()));
    if match total {
        Some(total) => total > env.limits.queued_bytes,
        None => true,
    } {
        above.push(Event::Refused { owner, refusal: Refusal::Busy });
        return dir.at(phase);
    }
    store.admitted.reserve(&ops, projected, number);
    store.queued_bytes = total.expect("checked queue bytes");
    store.queue.push(Pending { owner, number, ops, frame: bytes });
    begin_batch(store, dir, phase, step)
}

#[expect(clippy::manual_map, reason = "the subset uses explicit Option arms for a copy of at most Limits::value bytes")]
fn get(store: &Store, owner: Token, key: Box<[u8]>, available: bool, step: &mut Step<'_>) {
    let Step { env, above, below: _ } = step;
    if key.len() > usize_of(env.limits.key) {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(Large::Key) });
        return;
    }
    if !available {
        above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
        return;
    }
    let value = match store.map.get(&key) {
        Some(value) => Some(Box::from(value)),
        None => None,
    };
    above.push(Event::Got { owner, value });
}

fn load(store: &Store, owner: Token, range: Range, max: Page, available: bool, step: &mut Step<'_>) {
    let Step { env, above, below: _ } = step;
    if !available {
        above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
        return;
    }
    if max.rows == 0 || max.bytes == 0 || max.rows > env.limits.page.rows || max.bytes > env.limits.page.bytes {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(Large::Page) });
        return;
    }
    if range.start.len() > usize_of(env.limits.key)
        || match &range.end {
            Some(end) => end.len() > usize_of(env.limits.key),
            None => false,
        }
    {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(Large::Key) });
        return;
    }
    if let Some(end) = &range.end
        && end.as_ref() < range.start.as_ref()
    {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(Large::Range) });
        return;
    }
    match store.map.page(&range, max) {
        Ok(page) => above.push(Event::Loaded { owner, rows: page.rows, next: page.next }),
        Err(refusal) => above.push(Event::Refused { owner, refusal }),
    }
}

fn close_requested(store: &Store, mut dir: Dir, phase: Phase, owner: Token, step: &mut Step<'_>) -> State {
    if dir.close_owner.is_some() {
        step.above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
        return dir.at(phase);
    }
    dir.close_owner = Some(owner);
    match phase.ready() {
        Some(log) => {
            if dir.compaction.is_idle() && store.queue.is_empty() {
                step.below.push(file::Request::Close { owner: IO_OWNER, file: log.file });
                dir.at(Phase::Closing)
            } else {
                dir.at(Phase::Ready { log })
            }
        }
        None => dir.at(phase),
    }
}

fn too_large(ops: &[Op], limits: &Limits) -> Option<Large> {
    if ops.len() > usize_of(limits.ops) {
        return Some(Large::Ops);
    }
    for op in ops {
        if op.key().len() > usize_of(limits.key) {
            return Some(Large::Key);
        }
        match op {
            Op::Put { value, .. } if value.len() > usize_of(limits.value) => {
                return Some(Large::Value);
            }
            Op::Put { .. } | Op::Erase { .. } => {}
        }
    }
    match frame::encoded_len(ops) {
        Some(size) if size <= usize_of(limits.commit) => None,
        Some(..) | None => Some(Large::Frame),
    }
}

fn commit_memory(store: &mut Store, limits: &Limits, owner: Token, ops: Box<[Op]>, above: &mut Queue<Event>) {
    let Some(number) = store.last.checked_add(1) else {
        above.push(Event::Refused { owner, refusal: Refusal::Full(Exhausted::Numbers) });
        return;
    };
    if let Some(large) = too_large(&ops, limits) {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge(large) });
        return;
    }
    match store.map.check(&ops, limits) {
        Ok(used) => {
            store.map.apply(&ops, used, store.rule.as_deref(), true);
            store.last = number;
            above.push(Event::Committed { owner, number });
        }
        Err(refusal) => above.push(Event::Refused { owner, refusal }),
    }
}

/// One terminal event from the file layer.
#[expect(clippy::too_many_lines, reason = "the total table lists each phase and its possible file answers")]
pub fn up(
    store: &mut Store,
    env: &Env<Limits>,
    event: file::Event,
    above: &mut Queue<Event>,
    below: &mut Queue<file::Request>,
) {
    assert!(event.owner() == IO_OWNER, "the store receives only its own file events");
    let event = done(event);
    let state = mem::replace(&mut store.state, State::Closed);
    let mut step = Step { env, above, below };
    store.state = match state {
        State::Closed | State::Memory => unreachable!("no file request is outstanding"),
        State::Open { dir, phase } => match phase {
            Phase::Listing => match event {
                Done::Listed { entries } => listed(store, dir, entries, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Opened { .. } | Done::Read { .. } | Done::Finished => wrong_answer(),
            },
            Phase::RemovingTempOnOpen { snapshot } => match event {
                Done::Finished => store.start_recovery(dir, snapshot, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::OpeningSnapshot => match event {
                Done::Opened { file, len } => snapshot_opened(dir, file, len, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => wrong_answer(),
            },
            Phase::ReadingSnapshot { file, len, at, reader } => match event {
                Done::Read { bytes } => snapshot_read(store, dir, ReadAt { file, len, at }, reader, bytes, &mut step),
                Done::Failed { .. } => open_failed_file(dir, file, Failure::Io, step.below),
                Done::Listed { .. } | Done::Opened { .. } | Done::Finished => wrong_answer(),
            },
            Phase::ClosingSnapshot { next } | Phase::ClosingRecovered { next } => match event {
                Done::Finished => store.next_log(dir, next, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::OpeningLog { name, next } => match event {
                Done::Opened { file, len } => log_opened(store, dir, name, next, file, len, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => wrong_answer(),
            },
            Phase::ReadingLog { file, len, at, bytes, next } => match event {
                Done::Read { bytes: read } => {
                    log_read(store, dir, ReadAt { file, len, at }, bytes, next, read, &mut step)
                }
                Done::Failed { .. } => open_failed_file(dir, file, Failure::Io, step.below),
                Done::Listed { .. } | Done::Opened { .. } | Done::Finished => wrong_answer(),
            },
            Phase::RemovingTail { name } => match event {
                Done::Finished => create_segment(dir, name, step.below),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::Creating { name } => match event {
                Done::Opened { file, .. } => segment_created(dir, name, file, step.below),
                Done::Failed { .. } => open_failed(dir, Failure::Io, &mut step),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => wrong_answer(),
            },
            Phase::Announcing { file } => match event {
                Done::Finished => segment_announced(store, dir, file, &mut step),
                Done::Failed { .. } => open_failed_file(dir, file, Failure::Io, step.below),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::Writing { log, batch } => match event {
                Done::Finished => batch_written(dir, log, batch, step.below),
                Done::Failed { .. } => commits_failed(store, dir, Some(log.file), batch, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::Syncing { log, batch } => match event {
                Done::Finished => batch_synced(store, dir, log, batch, &mut step),
                Done::Failed { .. } => commits_failed(store, dir, Some(log.file), batch, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::Closing => match event {
                Done::Finished | Done::Failed { .. } => closed(dir, step.above),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::RecoveryClosing => match event {
                Done::Finished | Done::Failed { .. } => recovery_closed(store, dir, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::RolloverClosing => match event {
                Done::Finished => rollover_closed(store, dir, &mut step),
                Done::Failed { .. } => commits_failed(store, dir, None, Queue::with_capacity(0), &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::SnapshotFailureClosing { log } => match event {
                Done::Finished | Done::Failed { .. } => {
                    commits_failed(store, dir, Some(log.file), Queue::with_capacity(0), &mut step)
                }
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::OpenFailureClosing { failure } => match event {
                Done::Finished | Done::Failed { .. } => open_failed(dir, failure, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => wrong_answer(),
            },
            Phase::Ready { .. } => unreachable!("no file request is outstanding"),
            Phase::SnapIo { log } => match event {
                Done::Finished => compaction::finished(store, dir, log, &mut step),
                Done::Opened { file, .. } => compaction::opened(store, dir, log, file, &mut step),
                Done::Failed { error } => compaction::failed(store, dir, log, error, &mut step),
                Done::Listed { .. } | Done::Read { .. } => {
                    unreachable!("snapshot replacement never lists or reads a file")
                }
            },
        },
    };
}

/// A file answer `up` has no cell for: at most one request is outstanding, so the phase knows it.
fn wrong_answer() -> ! {
    unreachable!("the file answer must match this phase's request")
}

fn listed(store: &mut Store, mut dir: Dir, entries: Box<[file::Entry]>, step: &mut Step<'_>) -> State {
    dir.segments.clear();
    let mut snapshot = false;
    let mut temp = false;
    for entry in entries {
        if entry.name.as_ref() == b"snapshot" {
            snapshot = true;
        }
        if entry.name.as_ref() == b"snapshot.tmp" {
            temp = true;
        }
        if log_number(&entry.name).is_some() {
            dir.segments.push(entry.name);
        }
    }
    dir.segments.sort_unstable();
    if temp {
        step.below.push(file::Request::Remove {
            owner: IO_OWNER,
            root: dir.root,
            name: Box::from(&b"snapshot.tmp"[..]),
            directory: false,
        });
        dir.at(Phase::RemovingTempOnOpen { snapshot })
    } else {
        store.start_recovery(dir, snapshot, step)
    }
}

struct ReadAt {
    file: Token,
    len: u64,
    at: u64,
}

fn snapshot_opened(dir: Dir, file: Token, len: u64, step: &mut Step<'_>) -> State {
    if len == 0 {
        return open_failed_file(dir, file, Failure::Corrupt, step.below);
    }
    let cap = step.env.limits.budget.saturating_mul(2).saturating_add(1024);
    if len > cap {
        return open_failed_file(dir, file, Failure::Full, step.below);
    }
    read_next(file, len, 0, step);
    dir.at(Phase::ReadingSnapshot { file, len, at: 0, reader: snapshot::Reader::new() })
}

fn snapshot_read(
    store: &mut Store,
    dir: Dir,
    source: ReadAt,
    mut reader: snapshot::Reader,
    read: Box<[u8]>,
    step: &mut Step<'_>,
) -> State {
    let ReadAt { file, len, .. } = source;
    let Some(end) = read_end(&source, &read) else {
        return open_failed_file(dir, file, Failure::Corrupt, step.below);
    };
    if let Err(failure) = store.apply_snapshot(&mut reader, &read, &step.env.limits) {
        return open_failed_file(dir, file, failure, step.below);
    }
    if end < len {
        read_next(file, len, end, step);
        dir.at(Phase::ReadingSnapshot { file, len, at: end, reader })
    } else {
        let Some(start) = reader.finish() else {
            return open_failed_file(dir, file, Failure::Corrupt, step.below);
        };
        let mut next = dir.segments.len();
        for (index, name) in dir.segments.iter().enumerate() {
            match log_number(name) {
                Some(number) if number >= start => {
                    next = index;
                    break;
                }
                Some(_) | None => {}
            }
        }
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::ClosingSnapshot { next })
    }
}

#[expect(clippy::disallowed_types, reason = "the recovery frame buffer grows only to Limits::commit plus one read")]
fn log_opened(
    store: &mut Store,
    dir: Dir,
    name: Box<[u8]>,
    next: usize,
    file: Token,
    len: u64,
    step: &mut Step<'_>,
) -> State {
    let cap = step.env.limits.segment.saturating_add(step.env.limits.queued_bytes);
    if len > cap {
        return open_failed_file(dir, file, Failure::Full, step.below);
    }
    let first = log_number(&name).expect("an opened log has a numbered name");
    if len != 0 {
        let Some(number) = store.last.checked_add(1) else {
            return open_failed_file(dir, file, Failure::Corrupt, step.below);
        };
        if first != number {
            return log_damaged(store, dir, file, step.below);
        }
    }
    if len == 0 {
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::ClosingRecovered { next })
    } else {
        read_next(file, len, 0, step);
        dir.at(Phase::ReadingLog { file, len, at: 0, bytes: Vec::new(), next })
    }
}

#[expect(clippy::disallowed_types, reason = "replay retains at most Limits::commit plus one read")]
fn log_read(
    store: &mut Store,
    mut dir: Dir,
    source: ReadAt,
    mut bytes: Vec<u8>,
    next: usize,
    read: Box<[u8]>,
    step: &mut Step<'_>,
) -> State {
    let ReadAt { file, len, .. } = source;
    let Some(end) = read_end(&source, &read) else {
        return open_failed_file(dir, file, Failure::Corrupt, step.below);
    };
    bytes.reserve_exact(read.len());
    bytes.extend_from_slice(&read);
    match store.replay(&mut dir, &mut bytes, &step.env.limits) {
        Err(failure) => return open_failed_file(dir, file, failure, step.below),
        Ok(false) => return log_damaged(store, dir, file, step.below),
        Ok(true) => {}
    }
    if end < len {
        read_next(file, len, end, step);
        dir.at(Phase::ReadingLog { file, len, at: end, bytes, next })
    } else if bytes.is_empty() {
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::ClosingRecovered { next })
    } else {
        log_damaged(store, dir, file, step.below)
    }
}

fn log_damaged(store: &mut Store, dir: Dir, file: Token, below: &mut Queue<file::Request>) -> State {
    store.repair = Repair::Required;
    let next = dir.segments.len();
    below.push(file::Request::Close { owner: IO_OWNER, file });
    dir.at(Phase::ClosingRecovered { next })
}

fn create_segment(dir: Dir, name: Box<[u8]>, below: &mut Queue<file::Request>) -> State {
    below.push(file::Request::Create { owner: IO_OWNER, root: dir.root, name: name.clone(), mode: 0o600 });
    dir.at(Phase::Creating { name })
}

fn segment_created(mut dir: Dir, name: Box<[u8]>, file: Token, below: &mut Queue<file::Request>) -> State {
    dir.segments.push(name);
    below.push(file::Request::SyncDirectory { owner: IO_OWNER, root: dir.root });
    dir.at(Phase::Announcing { file })
}

fn segment_announced(store: &mut Store, mut dir: Dir, file: Token, step: &mut Step<'_>) -> State {
    dir.compaction = dir.compaction.announced(&mut dir.log_bytes, &mut store.map.census);
    match store.repair {
        Repair::Clean => announce_recovered(&mut dir, store.last, step.above),
        Repair::Required => {}
    }
    follow_ready(store, dir, Log { file, offset: 0 }, step)
}

fn batch_written(dir: Dir, log: Log, batch: Queue<Pending>, below: &mut Queue<file::Request>) -> State {
    below.push(file::Request::Sync { owner: IO_OWNER, file: log.file });
    dir.at(Phase::Syncing { log, batch })
}

fn batch_synced(
    store: &mut Store,
    mut dir: Dir,
    mut log: Log,
    mut batch: Queue<Pending>,
    step: &mut Step<'_>,
) -> State {
    let mut bytes: u64 = 0;
    for _ in 0..batch.len() {
        let pending = batch.pop().expect("the batch has its initial length");
        let used = store.map.check(&pending.ops, &step.env.limits).expect("a queued commit passed admission");
        store.map.apply(&pending.ops, used, store.rule.as_deref(), true);
        store.admitted.applied(&pending.ops, pending.number);
        store.last = pending.number;
        bytes = bytes.checked_add(skein_lib::len_u64(pending.frame.len())).expect("batch size fits u64");
        step.above.push(Event::Committed { owner: pending.owner, number: pending.number });
    }
    log.offset = log.offset.checked_add(bytes).expect("segment offset fits u64");
    dir.log_bytes = dir.log_bytes.checked_add(bytes).expect("log bytes fit u64");
    follow_ready(store, dir, log, step)
}

fn follow_ready(store: &mut Store, dir: Dir, log: Log, step: &mut Step<'_>) -> State {
    if !store.queue.is_empty() {
        store.start_batch(dir, log, &step.env.limits, step.below)
    } else if dir.close_owner.is_some() && dir.compaction.is_idle() {
        step.below.push(file::Request::Close { owner: IO_OWNER, file: log.file });
        dir.at(Phase::Closing)
    } else {
        dir.at(Phase::Ready { log })
    }
}

fn closed(mut dir: Dir, above: &mut Queue<Event>) -> State {
    if let Some(owner) = dir.close_owner.take() {
        above.push(Event::Closed { owner });
    }
    State::Closed
}

fn recovery_closed(store: &mut Store, mut dir: Dir, step: &mut Step<'_>) -> State {
    match mem::replace(&mut dir.compaction, Compaction::Idle).file() {
        Some(file) => {
            step.below.push(file::Request::Close { owner: IO_OWNER, file });
            dir.at(Phase::RecoveryClosing)
        }
        None => store.recover(dir, step),
    }
}

fn rollover_closed(store: &mut Store, dir: Dir, step: &mut Step<'_>) -> State {
    let next = store.last.checked_add(1).expect("commit number did not overflow");
    create_segment(dir, segment_name(next), step.below)
}

fn commits_failed(
    store: &mut Store,
    mut dir: Dir,
    file: Option<Token>,
    mut batch: Queue<Pending>,
    step: &mut Step<'_>,
) -> State {
    store.repair = Repair::Required;
    dir.serving = match dir.serving {
        Serving::Yes | Serving::Recovering => Serving::Recovering,
        state @ Serving::Opening { .. } => state,
    };
    for _ in 0..batch.len() {
        let pending = batch.pop().expect("the batch has its initial length");
        step.above.push(Event::Failed { owner: pending.owner, number: pending.number });
    }
    for _ in 0..store.queue.len() {
        let pending = store.queue.pop().expect("the queue has its initial length");
        step.above.push(Event::Failed { owner: pending.owner, number: pending.number });
    }
    store.queued_bytes = 0;
    store.admitted.clear();
    if let Some(file) = file {
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::RecoveryClosing)
    } else {
        recovery_closed(store, dir, step)
    }
}

fn announce_recovered(dir: &mut Dir, last: u64, above: &mut Queue<Event>) {
    match mem::replace(&mut dir.serving, Serving::Yes) {
        Serving::Opening { owner } => above.push(Event::Opened { owner, last }),
        Serving::Recovering => above.push(Event::Recovered { last }),
        Serving::Yes => {}
    }
}

fn open_failed(mut dir: Dir, failure: Failure, step: &mut Step<'_>) -> State {
    if let Some(file) = mem::replace(&mut dir.compaction, Compaction::Idle).file() {
        return open_failed_file(dir, file, failure, step.below);
    }
    match dir.serving {
        Serving::Opening { owner } => step.above.push(Event::OpenFailed { owner, failure }),
        Serving::Recovering => step.above.push(Event::RecoveryFailed { failure }),
        Serving::Yes => {}
    }
    if let Some(owner) = dir.close_owner.take() {
        step.above.push(Event::Closed { owner });
    }
    State::Closed
}

fn open_failed_file(dir: Dir, file: Token, failure: Failure, below: &mut Queue<file::Request>) -> State {
    below.push(file::Request::Close { owner: IO_OWNER, file });
    dir.at(Phase::OpenFailureClosing { failure })
}

/// Whether `fire` has one chunk or one replacement step ready to do.
#[must_use]
pub fn is_ready(store: &Store, limits: &Limits) -> bool {
    match &store.state {
        State::Closed | State::Memory => false,
        State::Open { dir, phase } => match phase.ready() {
            Some(_) => {
                !dir.compaction.is_idle()
                    || dir.log_bytes >= limits.snapshot_bound(store.map.live())
                    || dir.close_owner.is_some()
            }
            None => false,
        },
    }
}

/// Writes one snapshot chunk or advances its replacement sequence.
pub fn fire(store: &mut Store, env: &Env<Limits>, _above: &mut Queue<Event>, below: &mut Queue<file::Request>) {
    let state = mem::replace(&mut store.state, State::Closed);
    store.state = match state {
        state @ (State::Closed | State::Memory) => state,
        State::Open { dir, phase } => match phase.ready() {
            Some(log) => compaction::fire_ready(dir, log, store, env, below),
            None => dir.at(phase),
        },
    };
}
