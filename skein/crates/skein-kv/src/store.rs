//! The store's transitions (skein/docs/design/kv.md, 2–5).
//! At most one file request is outstanding.

#![expect(
    clippy::disallowed_types,
    reason = "commit queues, segments, and snapshot chunks are bounded by the store limits"
)]
#![expect(clippy::disallowed_methods, reason = "pure bounded admission and recovery scans have no hidden effects")]
#![expect(clippy::struct_excessive_bools, reason = "recovery flags distinguish independent durable milestones")]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;
use skein_io::file;
use skein_io::kernel::{Error, Fd};
use skein_lib::{Env, Queue, Token};

use crate::Limits;
use crate::boundary::{Event, Failure, Op, Page, Range, Refusal, Request};
use crate::frame;
use crate::map::{KvMap, Queued};
use crate::snapshot;

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

#[derive(Debug)]
enum Phase {
    Listing,
    OpeningLog { name: Box<[u8]> },
    ReadingLog { file: Token, len: u64, at: u64, bytes: Vec<u8>, replayed: Result<bool, Failure> },
    ClosingRecovered,
    RemovingTail,
    Creating { name: Box<[u8]> },
    Announcing { file: Token },
    Ready { file: Token, offset: u64 },
    Writing { file: Token, offset: u64 },
    Syncing { file: Token, offset: u64 },
    Closing,
    RecoveryClosing,
    RolloverClosing,
    SnapIo { log: Log },
    SnapshotFailureClosing { log: Log },
    OpenFailureClosing { failure: Failure },
    OpeningSnapshot,
    ReadingSnapshot { file: Token, len: u64, at: u64, reader: snapshot::Reader, failure: Option<Failure> },
    ClosingSnapshot,
    RemovingTempOnOpen,
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
    repair_required: bool,
    last: u64,
    map: KvMap,
    queue: Queue<Pending>,
    queued_bytes: u64,
    admitted: Queued,
}

/// The store's life (skein/docs/foundation/programming-model.md, 5.4).
#[expect(
    clippy::large_enum_variant,
    reason = "the open state owns its directory and phase inline without another allocation"
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
    open_owner: Option<Token>,
    close_owner: Option<Token>,
    active: Vec<Pending>,
    segments: Vec<Box<[u8]>>,
    segment_index: usize,
    tail_name: Option<Box<[u8]>>,
    recovered: bool,
    compaction: Compaction,
    has_snapshot: bool,
    has_temp: bool,
    log_bytes: u64,
    repairing: bool,
    recovering_after_failure: bool,
}

impl Dir {
    fn new(root: Fd, owner: Token) -> Dir {
        Dir {
            root,
            open_owner: Some(owner),
            close_owner: None,
            active: Vec::new(),
            segments: Vec::new(),
            segment_index: 0,
            tail_name: None,
            recovered: false,
            compaction: Compaction::Idle,
            has_snapshot: false,
            has_temp: false,
            log_bytes: 0,
            repairing: false,
            recovering_after_failure: false,
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
            repair_required: false,
            last: 0,
            map: KvMap::new(limits),
            queue: Queue::with_capacity(limits.queued),
            queued_bytes: 0,
            admitted: Queued::new(),
        }
    }

    /// An in-memory store for protocol tests; commits are applied at once.
    #[must_use]
    pub fn memory(limits: &Limits) -> Store {
        let mut store = Store::new(limits);
        store.state = State::Memory;
        store
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

    fn next_number(&self, dir: &Dir) -> Option<u64> {
        let active = u64::try_from(dir.active.len()).ok()?;
        self.last.checked_add(active)?.checked_add(u64::from(self.queue.len()))?.checked_add(1)
    }

    fn start_batch(
        &mut self,
        mut dir: Dir,
        file: Token,
        offset: u64,
        limits: &Limits,
        below: &mut Queue<file::Request>,
    ) -> State {
        if self.queue.is_empty() {
            return dir.at(Phase::Ready { file, offset });
        }
        if offset >= limits.segment {
            below.push(file::Request::Close { owner: IO_OWNER, file });
            return dir.at(Phase::RolloverClosing);
        }
        while let Some(pending) = self.queue.pop() {
            self.queued_bytes = self
                .queued_bytes
                .checked_sub(skein_lib::len_u64(pending.frame.len()))
                .expect("queued bytes account for every frame");
            dir.active.push(pending);
        }
        let mut bytes = Vec::new();
        for pending in &dir.active {
            bytes.extend_from_slice(&pending.frame);
        }
        below.push(file::Request::WriteAt { owner: IO_OWNER, file, offset, bytes: bytes.into_boxed_slice() });
        dir.at(Phase::Writing { file, offset })
    }

    fn recover(&mut self, mut dir: Dir, below: &mut Queue<file::Request>, limits: &Limits) -> State {
        self.last = 0;
        self.map = KvMap::new(limits);
        dir.segments.clear();
        dir.segment_index = 0;
        dir.tail_name = None;
        dir.recovered = false;
        dir.compaction = Compaction::Idle;
        dir.has_snapshot = false;
        dir.has_temp = false;
        dir.log_bytes = 0;
        dir.repairing = false;
        below.push(file::Request::List { owner: IO_OWNER, root: dir.root });
        dir.at(Phase::Listing)
    }

    fn finish_recovery(&mut self, mut dir: Dir, below: &mut Queue<file::Request>) -> State {
        let next = self.last.checked_add(1).expect("commit number did not overflow");
        if self.repair_required {
            dir.repairing = true;
        }
        if dir.repairing {
            dir.compaction = Compaction::Due { start: next };
        }
        let name = segment_name(next);
        if dir.segments.iter().any(|entry| entry.as_ref() == name.as_ref()) {
            dir.tail_name = Some(name);
            below.push(file::Request::Remove {
                owner: IO_OWNER,
                root: dir.root,
                name: dir.tail_name.as_ref().expect("set").clone(),
                directory: false,
            });
            dir.at(Phase::RemovingTail)
        } else {
            create_segment(dir, name, below)
        }
    }

    fn next_log(&mut self, mut dir: Dir, below: &mut Queue<file::Request>, limits: &Limits) -> State {
        let Some(name) = dir.segments.get(dir.segment_index).cloned() else {
            if dir.log_bytes >= limits.snapshot_bound(self.map.live()) {
                // Recovery creates the snapshot's starting segment without another rollover.
                dir.compaction =
                    Compaction::Due { start: self.last.checked_add(1).expect("commit number did not overflow") };
            }
            return self.finish_recovery(dir, below);
        };
        dir.segment_index = dir.segment_index.checked_add(1).expect("segment index fits");
        below.push(file::Request::OpenRead { owner: IO_OWNER, root: dir.root, name: name.clone() });
        dir.at(Phase::OpeningLog { name })
    }

    fn start_recovery(&mut self, dir: Dir, below: &mut Queue<file::Request>, limits: &Limits) -> State {
        if dir.has_snapshot {
            below.push(file::Request::OpenRead { owner: IO_OWNER, root: dir.root, name: Box::from(&b"snapshot"[..]) });
            dir.at(Phase::OpeningSnapshot)
        } else {
            self.next_log(dir, below, limits)
        }
    }

    fn apply_snapshot(
        &mut self,
        reader: &mut snapshot::Reader,
        read: &[u8],
        failure: &mut Option<Failure>,
        limits: &Limits,
    ) {
        if *failure == Some(Failure::Corrupt) {
            return;
        }
        let mut input = read;
        // Each part consumes bytes of this read; an incomplete part ends the pass.
        for _ in 0..read.len() {
            if input.is_empty() {
                break;
            }
            let Some(part) = reader.next(&mut input, limits.chunk) else {
                *failure = Some(Failure::Corrupt);
                return;
            };
            match part {
                snapshot::Part::NeedMore => break,
                snapshot::Part::Start { start } => {
                    self.last = start.checked_sub(1).expect("the checked snapshot starts past zero");
                }
                snapshot::Part::Rows { rows } => {
                    for row in rows {
                        if failure.is_some() {
                            break;
                        }
                        let op = Op::Put { key: row.key, value: row.value };
                        match self.map.check(core::slice::from_ref(&op), limits) {
                            Ok(used) => self.map.apply(core::slice::from_ref(&op), used),
                            Err(_) => *failure = Some(Failure::Full),
                        }
                    }
                }
                snapshot::Part::End => {}
            }
        }
    }

    fn replay(&mut self, dir: &mut Dir, bytes: &mut Vec<u8>, limits: &Limits) -> Result<bool, Failure> {
        let mut at: usize = 0;
        while at < bytes.len() {
            let Some(head_end) = at.checked_add(usize::try_from(frame::HEADER).expect("frame header fits usize"))
            else {
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
            if decoded.ops.len() > usize::try_from(limits.ops).expect("u32 fits usize") {
                return Ok(false);
            }
            if decoded.number != self.last.checked_add(1).ok_or(Failure::Corrupt)? {
                return Ok(false);
            }
            let used = self.map.check(&decoded.ops, limits).map_err(|_| Failure::Full)?;
            self.map.apply(&decoded.ops, used);
            self.last = decoded.number;
            dir.log_bytes = dir.log_bytes.checked_add(skein_lib::len_u64(length)).expect("log bytes fit u64");
            at = end;
        }
        // Keep only the unapplied suffix between reads, moving it once per
        // read rather than once per frame; the applied allocation is released.
        if at != 0 {
            *bytes = bytes.get(at..).expect("replay ended within its bytes").to_vec();
        }
        Ok(true)
    }
}

fn segment_name(number: u64) -> Box<[u8]> {
    let mut name = *b"log-00000000000000000000";
    let mut number = number;
    let mut pos = name.len();
    while pos > 4 {
        pos = pos.checked_sub(1).expect("position is positive");
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

/// What a recovery read asks: a chunk, or what the file holds past `at` (kv.md, 3).
fn read_size(chunk: u32, len: u64, at: u64) -> u32 {
    let left = len.checked_sub(at).expect("recovery reads only within the file");
    u32::try_from(u64::from(chunk).min(left)).expect("a recovery read is at most one chunk")
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
            State::Closed | State::Memory => store.recover(Dir::new(root, owner), step.below, &env.limits),
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
    dir.recovered
        && match phase {
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
            | Phase::ClosingRecovered
            | Phase::RemovingTail
            | Phase::Closing
            | Phase::RecoveryClosing
            | Phase::OpeningSnapshot
            | Phase::ReadingSnapshot { .. }
            | Phase::ClosingSnapshot
            | Phase::RemovingTempOnOpen
            | Phase::SnapshotFailureClosing { .. }
            | Phase::OpenFailureClosing { .. } => false,
        }
}

fn begin_batch(store: &mut Store, dir: Dir, phase: Phase, step: &mut Step<'_>) -> State {
    match phase {
        Phase::Ready { file, offset } => store.start_batch(dir, file, offset, &step.env.limits, step.below),
        phase @ (Phase::Listing
        | Phase::OpeningLog { .. }
        | Phase::ReadingLog { .. }
        | Phase::ClosingRecovered
        | Phase::RemovingTail
        | Phase::Creating { .. }
        | Phase::Announcing { .. }
        | Phase::Writing { .. }
        | Phase::Syncing { .. }
        | Phase::Closing
        | Phase::RecoveryClosing
        | Phase::RolloverClosing
        | Phase::SnapIo { .. }
        | Phase::SnapshotFailureClosing { .. }
        | Phase::OpenFailureClosing { .. }
        | Phase::OpeningSnapshot
        | Phase::ReadingSnapshot { .. }
        | Phase::ClosingSnapshot
        | Phase::RemovingTempOnOpen) => dir.at(phase),
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
    let Some(number) = store.next_number(&dir) else {
        above.push(Event::Refused { owner, refusal: Refusal::Full });
        return dir.at(phase);
    };
    if ops.len() > usize::try_from(env.limits.ops).expect("u32 fits usize") {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return dir.at(phase);
    }
    if ops.iter().any(|op| {
        op.key().len() > usize::try_from(env.limits.key).expect("key limit fits usize")
            || match op {
                Op::Put { value, .. } => {
                    value.len() > usize::try_from(env.limits.value).expect("value limit fits usize")
                }
                Op::Erase { .. } => false,
            }
    }) {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return dir.at(phase);
    }
    if frame::encoded_len(&ops)
        .is_none_or(|size| size > usize::try_from(env.limits.commit).expect("commit limit fits usize"))
    {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return dir.at(phase);
    }
    let Some(bytes) = frame::encode(number, &ops, env.limits.commit) else {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return dir.at(phase);
    };
    let total = store.queued_bytes.checked_add(skein_lib::len_u64(bytes.len()));
    if total.is_none_or(|total| total > env.limits.queued_bytes) {
        let refusal = match store.admitted.check(&store.map, &ops, &env.limits) {
            Ok(..) => Refusal::Busy,
            Err(refusal) => refusal,
        };
        above.push(Event::Refused { owner, refusal });
        return dir.at(phase);
    }
    if let Err(refusal) = store.admitted.admit(&store.map, &ops, &env.limits) {
        above.push(Event::Refused { owner, refusal });
        return dir.at(phase);
    }
    store.queued_bytes = total.expect("checked queue bytes");
    store.queue.push(Pending { owner, number, ops, frame: bytes });
    begin_batch(store, dir, phase, step)
}

fn get(store: &Store, owner: Token, key: Box<[u8]>, available: bool, step: &mut Step<'_>) {
    let Step { env, above, below: _ } = step;
    if key.len() > usize::try_from(env.limits.key).expect("key limit fits") {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return;
    }
    if !available {
        above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
        return;
    }
    above.push(Event::Got { owner, value: store.map.get(&key).map(Box::from) });
}

fn load(store: &Store, owner: Token, range: Range, max: Page, available: bool, step: &mut Step<'_>) {
    let Step { env, above, below: _ } = step;
    if !available {
        above.push(Event::Refused { owner, refusal: Refusal::Unavailable });
        return;
    }
    if max.rows == 0 || max.bytes == 0 || max.rows > env.limits.page.rows || max.bytes > env.limits.page.bytes {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return;
    }
    if range.start.len() > usize::try_from(env.limits.key).expect("key limit fits")
        || range.end.as_ref().is_some_and(|end| end.len() > usize::try_from(env.limits.key).expect("key limit fits"))
        || range.end.as_ref().is_some_and(|end| end.as_ref() < range.start.as_ref())
    {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
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
    match phase {
        Phase::Ready { file, offset } => {
            if dir.compaction.is_idle() && store.queue.is_empty() {
                step.below.push(file::Request::Close { owner: IO_OWNER, file });
                dir.at(Phase::Closing)
            } else {
                dir.at(Phase::Ready { file, offset })
            }
        }
        phase @ (Phase::Listing
        | Phase::OpeningLog { .. }
        | Phase::ReadingLog { .. }
        | Phase::ClosingRecovered
        | Phase::RemovingTail
        | Phase::Creating { .. }
        | Phase::Announcing { .. }
        | Phase::Writing { .. }
        | Phase::Syncing { .. }
        | Phase::Closing
        | Phase::RecoveryClosing
        | Phase::RolloverClosing
        | Phase::SnapIo { .. }
        | Phase::SnapshotFailureClosing { .. }
        | Phase::OpenFailureClosing { .. }
        | Phase::OpeningSnapshot
        | Phase::ReadingSnapshot { .. }
        | Phase::ClosingSnapshot
        | Phase::RemovingTempOnOpen) => dir.at(phase),
    }
}

fn commit_memory(store: &mut Store, limits: &Limits, owner: Token, ops: Box<[Op]>, above: &mut Queue<Event>) {
    let Some(number) = store.last.checked_add(1) else {
        above.push(Event::Refused { owner, refusal: Refusal::Full });
        return;
    };
    if ops.iter().any(|op| {
        op.key().len() > usize::try_from(limits.key).expect("key limit fits usize")
            || match op {
                Op::Put { value, .. } => value.len() > usize::try_from(limits.value).expect("value limit fits usize"),
                Op::Erase { .. } => false,
            }
    }) || frame::encoded_len(&ops)
        .is_none_or(|size| size > usize::try_from(limits.commit).expect("commit limit fits usize"))
    {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return;
    }
    if ops.len() > usize::try_from(limits.ops).expect("u32 fits usize") {
        above.push(Event::Refused { owner, refusal: Refusal::TooLarge });
        return;
    }
    match store.map.check(&ops, limits) {
        Ok(used) => {
            store.map.apply(&ops, used);
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
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Opened { .. } | Done::Read { .. } | Done::Finished => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::RemovingTempOnOpen => match event {
                Done::Finished => temp_removed(store, dir, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::OpeningSnapshot => match event {
                Done::Opened { file, len } => snapshot_opened(dir, file, len, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::ReadingSnapshot { file, len, at, reader, failure } => match event {
                Done::Read { bytes } => {
                    snapshot_read(store, dir, ReadAt { file, len, at }, reader, failure, bytes, &mut step)
                }
                Done::Failed { .. } => open_failed_file(dir, file, Failure::Io, step.below),
                Done::Listed { .. } | Done::Opened { .. } | Done::Finished => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::ClosingSnapshot | Phase::ClosingRecovered => match event {
                Done::Finished => recovered_file_closed(store, dir, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::OpeningLog { name } => match event {
                Done::Opened { file, len } => log_opened(store, dir, name, file, len, &mut step),
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::ReadingLog { file, len, at, bytes, replayed } => match event {
                Done::Read { bytes: read } => {
                    log_read(store, dir, ReadAt { file, len, at }, bytes, replayed, read, &mut step)
                }
                Done::Failed { .. } => open_failed_file(dir, file, Failure::Io, step.below),
                Done::Listed { .. } | Done::Opened { .. } | Done::Finished => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::RemovingTail => match event {
                Done::Finished => tail_removed(dir, step.below),
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::Creating { name } => match event {
                Done::Opened { file, .. } => segment_created(dir, name, file, step.below),
                Done::Failed { .. } => open_failed(dir, Failure::Io, step.above),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::Announcing { file } => match event {
                Done::Finished => segment_announced(store, dir, file, &mut step),
                Done::Failed { .. } => open_failed_file(dir, file, Failure::Io, step.below),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::Writing { file, offset } => match event {
                Done::Finished => batch_written(dir, file, offset, step.below),
                Done::Failed { .. } => commits_failed(store, dir, Some(file), &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::Syncing { file, offset } => match event {
                Done::Finished => batch_synced(store, dir, file, offset, &mut step),
                Done::Failed { .. } => commits_failed(store, dir, Some(file), &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::Closing => match event {
                Done::Finished | Done::Failed { .. } => closed(dir, step.above),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::RecoveryClosing => match event {
                Done::Finished | Done::Failed { .. } => recovery_closed(store, dir, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::RolloverClosing => match event {
                Done::Finished => rollover_closed(store, dir, &mut step),
                Done::Failed { .. } => commits_failed(store, dir, None, &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::SnapshotFailureClosing { log } => match event {
                Done::Finished | Done::Failed { .. } => commits_failed(store, dir, Some(log.file), &mut step),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
            },
            Phase::OpenFailureClosing { failure } => match event {
                Done::Finished | Done::Failed { .. } => open_failed(dir, failure, step.above),
                Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } => {
                    unreachable!("the file answer must match this phase's request")
                }
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

fn listed(store: &mut Store, mut dir: Dir, entries: Box<[file::Entry]>, step: &mut Step<'_>) -> State {
    dir.segments.clear();
    for entry in entries {
        if entry.name.as_ref() == b"snapshot" {
            dir.has_snapshot = true;
        }
        if entry.name.as_ref() == b"snapshot.tmp" {
            dir.has_temp = true;
        }
        if log_number(&entry.name).is_some() {
            dir.segments.push(entry.name);
        }
    }
    dir.segments.sort_unstable();
    dir.segment_index = 0;
    if dir.has_temp {
        step.below.push(file::Request::Remove {
            owner: IO_OWNER,
            root: dir.root,
            name: Box::from(&b"snapshot.tmp"[..]),
            directory: false,
        });
        dir.at(Phase::RemovingTempOnOpen)
    } else {
        store.start_recovery(dir, step.below, &step.env.limits)
    }
}

fn temp_removed(store: &mut Store, dir: Dir, step: &mut Step<'_>) -> State {
    store.start_recovery(dir, step.below, &step.env.limits)
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
    step.below.push(file::Request::ReadAt {
        owner: IO_OWNER,
        file,
        offset: 0,
        max: read_size(step.env.limits.chunk, len, 0),
    });
    dir.at(Phase::ReadingSnapshot { file, len, at: 0, reader: snapshot::Reader::new(), failure: None })
}

fn snapshot_read(
    store: &mut Store,
    mut dir: Dir,
    source: ReadAt,
    mut reader: snapshot::Reader,
    mut failure: Option<Failure>,
    read: Box<[u8]>,
    step: &mut Step<'_>,
) -> State {
    let ReadAt { file, len, at } = source;
    let next = at.checked_add(skein_lib::len_u64(read.len()));
    if read.is_empty() || next.is_none_or(|next| next > len) {
        return open_failed_file(dir, file, Failure::Corrupt, step.below);
    }
    store.apply_snapshot(&mut reader, &read, &mut failure, &step.env.limits);
    let next = next.expect("checked read offset");
    if next < len {
        step.below.push(file::Request::ReadAt {
            owner: IO_OWNER,
            file,
            offset: next,
            max: read_size(step.env.limits.chunk, len, next),
        });
        dir.at(Phase::ReadingSnapshot { file, len, at: next, reader, failure })
    } else {
        let start = reader.finish();
        if start.is_none() {
            failure = Some(Failure::Corrupt);
        }
        if let Some(failure) = failure {
            return open_failed_file(dir, file, failure, step.below);
        }
        let start = start.expect("the snapshot ends with a checked trailer");
        dir.segment_index = dir
            .segments
            .iter()
            .position(|name| log_number(name).is_some_and(|number| number >= start))
            .unwrap_or(dir.segments.len());
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::ClosingSnapshot)
    }
}

fn recovered_file_closed(store: &mut Store, dir: Dir, step: &mut Step<'_>) -> State {
    store.next_log(dir, step.below, &step.env.limits)
}

fn log_opened(store: &mut Store, mut dir: Dir, name: Box<[u8]>, file: Token, len: u64, step: &mut Step<'_>) -> State {
    let cap = step.env.limits.segment.saturating_add(step.env.limits.queued_bytes);
    if len > cap {
        return open_failed_file(dir, file, Failure::Full, step.below);
    }
    let first = log_number(&name).expect("an opened log has a numbered name");
    if len != 0 {
        let Some(next) = store.last.checked_add(1) else {
            return open_failed_file(dir, file, Failure::Corrupt, step.below);
        };
        if first != next {
            dir.segment_index = dir.segments.len();
            dir.repairing = true;
            store.repair_required = true;
            step.below.push(file::Request::Close { owner: IO_OWNER, file });
            return dir.at(Phase::ClosingRecovered);
        }
    }
    if len == 0 {
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::ClosingRecovered)
    } else {
        step.below.push(file::Request::ReadAt {
            owner: IO_OWNER,
            file,
            offset: 0,
            max: read_size(step.env.limits.chunk, len, 0),
        });
        dir.at(Phase::ReadingLog { file, len, at: 0, bytes: Vec::new(), replayed: Ok(true) })
    }
}

fn log_read(
    store: &mut Store,
    mut dir: Dir,
    source: ReadAt,
    mut bytes: Vec<u8>,
    mut replayed: Result<bool, Failure>,
    read: Box<[u8]>,
    step: &mut Step<'_>,
) -> State {
    let ReadAt { file, len, at } = source;
    let next = at.checked_add(skein_lib::len_u64(read.len()));
    if read.is_empty() || next.is_none_or(|next| next > len) {
        return open_failed_file(dir, file, Failure::Corrupt, step.below);
    }
    if replayed == Ok(true) {
        bytes.reserve_exact(read.len());
        bytes.extend_from_slice(&read);
        replayed = store.replay(&mut dir, &mut bytes, &step.env.limits);
        if replayed != Ok(true) {
            bytes = Vec::new();
        }
    }
    let next = next.expect("checked read offset");
    if next < len {
        step.below.push(file::Request::ReadAt {
            owner: IO_OWNER,
            file,
            offset: next,
            max: read_size(step.env.limits.chunk, len, next),
        });
        dir.at(Phase::ReadingLog { file, len, at: next, bytes, replayed })
    } else {
        match replayed {
            Ok(whole) => {
                if !whole || !bytes.is_empty() {
                    dir.segment_index = dir.segments.len();
                    dir.repairing = true;
                    store.repair_required = true;
                }
                step.below.push(file::Request::Close { owner: IO_OWNER, file });
                dir.at(Phase::ClosingRecovered)
            }
            Err(failure) => open_failed_file(dir, file, failure, step.below),
        }
    }
}

fn tail_removed(mut dir: Dir, below: &mut Queue<file::Request>) -> State {
    let name = dir.tail_name.take().expect("tail to remove");
    create_segment(dir, name, below)
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
    let was_recovered = dir.recovered;
    dir.recovered = !dir.repairing;
    dir.compaction = dir.compaction.announced(&mut dir.log_bytes);
    if !was_recovered && !dir.repairing {
        announce_recovered(&mut dir, store.last, step.above);
    }
    follow_ready(store, dir, file, 0, step)
}

fn batch_written(dir: Dir, file: Token, offset: u64, below: &mut Queue<file::Request>) -> State {
    below.push(file::Request::Sync { owner: IO_OWNER, file });
    dir.at(Phase::Syncing { file, offset })
}

fn batch_synced(store: &mut Store, mut dir: Dir, file: Token, offset: u64, step: &mut Step<'_>) -> State {
    let mut bytes: u64 = 0;
    for pending in dir.active.drain(..) {
        let used = store.map.check(&pending.ops, &step.env.limits).expect("a queued commit passed admission");
        store.map.apply(&pending.ops, used);
        store.last = pending.number;
        bytes = bytes.checked_add(skein_lib::len_u64(pending.frame.len())).expect("batch size fits u64");
        step.above.push(Event::Committed { owner: pending.owner, number: pending.number });
    }
    if dir.active.is_empty() && store.queue.is_empty() {
        store.admitted.clear();
    }
    let offset = offset.checked_add(bytes).expect("segment offset fits u64");
    dir.log_bytes = dir.log_bytes.checked_add(bytes).expect("log bytes fit u64");
    follow_ready(store, dir, file, offset, step)
}

fn follow_ready(store: &mut Store, dir: Dir, file: Token, offset: u64, step: &mut Step<'_>) -> State {
    if !store.queue.is_empty() {
        store.start_batch(dir, file, offset, &step.env.limits, step.below)
    } else if dir.close_owner.is_some() && dir.compaction.is_idle() {
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::Closing)
    } else {
        dir.at(Phase::Ready { file, offset })
    }
}

fn closed(mut dir: Dir, above: &mut Queue<Event>) -> State {
    if let Some(owner) = dir.close_owner.take() {
        above.push(Event::Closed { owner });
    }
    State::Closed
}

fn recovery_closed(store: &mut Store, dir: Dir, step: &mut Step<'_>) -> State {
    store.recover(dir, step.below, &step.env.limits)
}

fn rollover_closed(store: &mut Store, dir: Dir, step: &mut Step<'_>) -> State {
    let next = store.last.checked_add(1).expect("commit number did not overflow");
    create_segment(dir, segment_name(next), step.below)
}

fn commits_failed(store: &mut Store, mut dir: Dir, file: Option<Token>, step: &mut Step<'_>) -> State {
    store.repair_required = true;
    dir.recovering_after_failure = true;
    for pending in dir.active.drain(..) {
        step.above.push(Event::Failed { owner: pending.owner, number: pending.number });
    }
    while let Some(pending) = store.queue.pop() {
        step.above.push(Event::Failed { owner: pending.owner, number: pending.number });
    }
    store.queued_bytes = 0;
    store.admitted.clear();
    if let Some(file) = file {
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        dir.at(Phase::RecoveryClosing)
    } else {
        store.recover(dir, step.below, &step.env.limits)
    }
}

fn announce_recovered(dir: &mut Dir, last: u64, above: &mut Queue<Event>) {
    if let Some(owner) = dir.open_owner.take() {
        above.push(Event::Opened { owner, last });
    } else if dir.recovering_after_failure {
        above.push(Event::Recovered { last });
    }
    dir.recovering_after_failure = false;
}

fn open_failed(mut dir: Dir, failure: Failure, above: &mut Queue<Event>) -> State {
    if let Some(owner) = dir.open_owner.take() {
        above.push(Event::OpenFailed { owner, failure });
    } else if dir.recovering_after_failure {
        above.push(Event::RecoveryFailed { failure });
    }
    if let Some(owner) = dir.close_owner.take() {
        above.push(Event::Closed { owner });
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
        State::Open { dir, phase } => match phase {
            Phase::Ready { .. } => {
                !dir.compaction.is_idle()
                    || dir.log_bytes >= limits.snapshot_bound(store.map.live())
                    || dir.close_owner.is_some()
            }
            Phase::Listing
            | Phase::OpeningLog { .. }
            | Phase::ReadingLog { .. }
            | Phase::ClosingRecovered
            | Phase::RemovingTail
            | Phase::Creating { .. }
            | Phase::Announcing { .. }
            | Phase::Writing { .. }
            | Phase::Syncing { .. }
            | Phase::Closing
            | Phase::RecoveryClosing
            | Phase::RolloverClosing
            | Phase::SnapIo { .. }
            | Phase::SnapshotFailureClosing { .. }
            | Phase::OpenFailureClosing { .. }
            | Phase::OpeningSnapshot
            | Phase::ReadingSnapshot { .. }
            | Phase::ClosingSnapshot
            | Phase::RemovingTempOnOpen => false,
        },
    }
}

/// Writes one snapshot chunk or advances its replacement sequence.
pub fn fire(store: &mut Store, env: &Env<Limits>, _above: &mut Queue<Event>, below: &mut Queue<file::Request>) {
    let state = mem::replace(&mut store.state, State::Closed);
    store.state = match state {
        state @ (State::Closed | State::Memory) => state,
        State::Open { dir, phase } => match phase {
            Phase::Ready { file, offset } => compaction::fire_ready(dir, Log { file, offset }, store, env, below),
            phase @ (Phase::Listing
            | Phase::OpeningLog { .. }
            | Phase::ReadingLog { .. }
            | Phase::ClosingRecovered
            | Phase::RemovingTail
            | Phase::Creating { .. }
            | Phase::Announcing { .. }
            | Phase::Writing { .. }
            | Phase::Syncing { .. }
            | Phase::Closing
            | Phase::RecoveryClosing
            | Phase::RolloverClosing
            | Phase::SnapIo { .. }
            | Phase::SnapshotFailureClosing { .. }
            | Phase::OpenFailureClosing { .. }
            | Phase::OpeningSnapshot
            | Phase::ReadingSnapshot { .. }
            | Phase::ClosingSnapshot
            | Phase::RemovingTempOnOpen) => dir.at(phase),
        },
    };
}
