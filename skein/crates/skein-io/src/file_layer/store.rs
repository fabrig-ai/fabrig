//! Atomic replacement beside an opened parent directory (skein/docs/design/runtime/io/files.md, 2).
//!
//! Each state owns its descriptors and temporary name, with one kernel operation
//! in flight. A terminal follows every close and temporary removal attempt;
//! failures distinguish a committed replacement and any cleanup residue.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;

use skein_lib::sha256::Sha256;
use skein_lib::{Env, Queue, Token, usize_of};

use super::{Contents, FileIo, Limits, Pending, Work};
use crate::digest::{Digest, digest};
use crate::file::{Event, Expect, Residue};
use crate::kernel::{Complete, Done, Error, Fd, Kind, Op, OpenHow, Submit, is_name};

const ATTEMPTS: u8 = 8;
type SplitPath = (Box<[u8]>, Box<[u8]>);

/// One store lifecycle (skein/docs/foundation/programming-model.md, 5.4).
#[derive(Debug)]
enum State {
    ParentOpening { input: Input },
    OldOpening { parent: Fd, input: Input },
    OldStating { parent: Fd, old: Fd, input: Input },
    OldClosing { parent: Fd, old: Fd, input: Input },
    TempCreating { parent: Fd, input: Input, name: Box<[u8]>, attempts: u8 },
    TempWriting(Opened),
    TempSyncing(Opened),
    TempClosing(Opened),
    CheckOpening(Named),
    CheckStating(Opened),
    CheckReading(Reading),
    CheckClosing { check: Opened, verdict: Verdict },
    Renaming(Named),
    DirectorySyncing { parent: Fd },
    ParentClosing { parent: Fd },
    Settling { failure: Failure, step: Settle },
    Finished,
}

#[derive(Debug)]
struct Input {
    bytes: Contents,
    mode: u32,
}

#[derive(Debug)]
struct Named {
    parent: Fd,
    name: Box<[u8]>,
}

#[derive(Debug)]
struct Opened {
    named: Named,
    fd: Fd,
}

#[derive(Debug)]
struct Reading {
    check: Opened,
    seen: Sha256,
    seen_bytes: u64,
}

#[derive(Debug)]
enum Verdict {
    Matches,
    Refused(Failure),
}

#[derive(Debug)]
enum Settle {
    ClosingCheck(Opened),
    ClosingOld { parent: Fd, old: Fd },
    ClosingTemp(Opened),
    Removing(Named),
    ClosingParent { parent: Fd, left: Left },
}

#[derive(Debug)]
enum Left {
    Nothing,
    Committed,
    Residue(Residue),
}

#[derive(Clone, Copy, Debug)]
enum Failure {
    Kernel(Error),
    Conflict { now: Option<Digest> },
}

#[derive(Debug)]
pub(super) struct Store {
    owner: Token,
    state: State,
    target: Box<[u8]>,
    expected: Expect,
    produced: Digest,
    no_follow: bool,
    private_owner: Option<u32>,
}

impl Store {
    pub(super) fn deadline(&self, deadline: Option<skein_lib::Time>) -> Option<skein_lib::Time> {
        match &self.state {
            State::Settling { .. } | State::CheckClosing { verdict: Verdict::Refused(_), .. } | State::Finished => None,
            State::ParentOpening { .. }
            | State::OldOpening { .. }
            | State::OldStating { .. }
            | State::OldClosing { .. }
            | State::TempCreating { .. }
            | State::TempWriting(_)
            | State::TempSyncing(_)
            | State::TempClosing(_)
            | State::CheckOpening(_)
            | State::CheckStating(_)
            | State::CheckReading(_)
            | State::CheckClosing { verdict: Verdict::Matches, .. }
            | State::Renaming(_)
            | State::DirectorySyncing { .. }
            | State::ParentClosing { .. } => deadline,
        }
    }

    pub(super) const fn owner(&self) -> Token {
        self.owner
    }

    /// A store can still leave the old target in place before its rename.
    pub(super) fn can_abandon(&self) -> bool {
        match self.state {
            State::ParentOpening { .. }
            | State::OldOpening { .. }
            | State::OldStating { .. }
            | State::OldClosing { .. }
            | State::TempCreating { .. }
            | State::TempWriting(_)
            | State::TempSyncing(_)
            | State::TempClosing(_)
            | State::CheckOpening(_)
            | State::CheckStating(_)
            | State::CheckReading(_)
            | State::CheckClosing { .. } => true,
            State::Renaming(_)
            | State::DirectorySyncing { .. }
            | State::ParentClosing { .. }
            | State::Settling { .. }
            | State::Finished => false,
        }
    }

    pub(super) fn can_cancel(&self) -> bool {
        match self.state {
            State::ParentOpening { .. }
            | State::OldOpening { .. }
            | State::TempCreating { .. }
            | State::TempWriting(_)
            | State::TempSyncing(_)
            | State::CheckOpening(_)
            | State::CheckReading(_)
            | State::DirectorySyncing { .. } => true,
            State::OldStating { .. }
            | State::OldClosing { .. }
            | State::TempClosing(_)
            | State::CheckStating(_)
            | State::CheckClosing { .. }
            | State::Renaming(_)
            | State::ParentClosing { .. }
            | State::Settling { .. }
            | State::Finished => false,
        }
    }
}

pub(super) fn split_path(path: &[u8]) -> Option<SplitPath> {
    let mut slash = None;
    for index in 0..path.len() {
        if path.get(index) == Some(&b'/') {
            slash = Some(index);
        }
    }
    match slash {
        Some(0) => None,
        Some(index) => {
            let parent = path.get(..index)?;
            let target = path.get(index.checked_add(1)?..)?;
            if is_name(target) { Some((Box::from(parent), Box::from(target))) } else { None }
        }
        None => {
            if is_name(path) {
                Some((Box::from(&b"."[..]), Box::from(path)))
            } else {
                None
            }
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the store request carries its complete bounded input")]
pub(super) fn start(
    io: &mut FileIo,
    env: &Env<Limits>,
    owner: Token,
    root: Token,
    path: Box<[u8]>,
    bytes: Box<[u8]>,
    expected: Expect,
    no_follow: bool,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let Some(root_fd) = io.file(root) else {
        events.push(Event::failed(owner, Error::NotFound));
        return Work::Idle;
    };
    if io.random.is_none() || bytes.len() > usize_of(env.limits.whole) || path.len() >= 4096 || path.contains(&0) {
        events.push(Event::failed(owner, Error::InvalidArgument));
        return Work::Idle;
    }
    let Some((parent_path, target)) = split_path(&path) else {
        events.push(Event::failed(owner, Error::InvalidArgument));
        return Work::Idle;
    };
    let private_owner = io.private_owner(root);
    let private = private_owner.is_some();
    let no_follow = no_follow || private;
    let produced = digest(&bytes);
    #[expect(
        clippy::disallowed_methods,
        reason = "runtime/io/files.md: retain the store's boxed allocation at its exact length"
    )]
    let bytes = Contents(bytes.into_vec());
    let store = Store {
        owner,
        state: State::ParentOpening { input: Input { bytes, mode: if private { 0o600 } else { 0o666 } } },
        target,
        expected,
        produced,
        no_follow,
        private_owner,
    };
    let how = if no_follow { OpenHow::DirectoryNoFollow } else { OpenHow::Directory };
    issue(io, store, Op::Open { root: root_fd, path: parent_path, how }, subs)
}

fn issue(io: &mut FileIo, store: Store, op: Op, subs: &mut Queue<Submit>) -> Work {
    super::issue(io, Pending::Store(store), op, subs)
}

fn temp_name(io: &mut FileIo, target: &[u8]) -> Box<[u8]> {
    let random = io.random.expect("a store has a seed").wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    io.random = Some(random);
    let prefix = target.get(..target.len().min(230)).expect("bounded prefix");
    let mut name = Vec::with_capacity(prefix.len().checked_add(23).expect("temporary name fits"));
    name.extend_from_slice(prefix);
    name.extend_from_slice(b".skein-");
    for index in 0..16_u32 {
        let shift = 60_u32.checked_sub(index.checked_mul(4).expect("hex position fits")).expect("hex position fits");
        let digit = u8::try_from((random >> shift) & 0xf).expect("hex digit");
        let ascii = if digit < 10 {
            b'0'.checked_add(digit).expect("decimal digit")
        } else {
            b'a'.checked_add(digit.checked_sub(10).expect("hex letter")).expect("hex letter")
        };
        name.push(ascii);
    }
    name.into_boxed_slice()
}

pub(super) fn done_fd(done: Done) -> Fd {
    match done {
        Done::Fd(fd) => fd,
        Done::Nothing
        | Done::Count(_)
        | Done::Accepted { .. }
        | Done::Bound(_)
        | Done::Stat(_)
        | Done::Spawned { .. }
        | Done::Exit(_)
        | Done::Usage(_)
        | Done::Signalled(_)
        | Done::Window(_) => unreachable!("an open returns a descriptor"),
    }
}

pub(super) fn done_stat(done: Done) -> crate::kernel::Stat {
    match done {
        Done::Stat(stat) => stat,
        Done::Nothing
        | Done::Count(_)
        | Done::Fd(_)
        | Done::Accepted { .. }
        | Done::Bound(_)
        | Done::Spawned { .. }
        | Done::Exit(_)
        | Done::Usage(_)
        | Done::Signalled(_)
        | Done::Window(_) => unreachable!("a stat returns metadata"),
    }
}

pub(super) fn done_count(done: Done) -> u32 {
    match done {
        Done::Count(count) => count,
        Done::Nothing
        | Done::Fd(_)
        | Done::Accepted { .. }
        | Done::Bound(_)
        | Done::Stat(_)
        | Done::Spawned { .. }
        | Done::Exit(_)
        | Done::Usage(_)
        | Done::Signalled(_)
        | Done::Window(_) => unreachable!("a transfer returns a count"),
    }
}

pub(super) fn done_nothing(done: Done) {
    match done {
        Done::Nothing => {}
        Done::Count(_)
        | Done::Fd(_)
        | Done::Accepted { .. }
        | Done::Bound(_)
        | Done::Stat(_)
        | Done::Spawned { .. }
        | Done::Exit(_)
        | Done::Usage(_)
        | Done::Signalled(_)
        | Done::Window(_) => unreachable!("this operation returns nothing"),
    }
}

pub(super) fn write_parts(kind: Op) -> (Fd, Box<[u8]>, u32, u64) {
    match kind {
        Op::Write { fd, bytes, from, at } => (fd, bytes, from, at),
        Op::Socket { .. }
        | Op::Bind { .. }
        | Op::Listen { .. }
        | Op::Accept { .. }
        | Op::Connect { .. }
        | Op::Recv { .. }
        | Op::Send { .. }
        | Op::Shutdown { .. }
        | Op::Close { .. }
        | Op::Open { .. }
        | Op::Read { .. }
        | Op::Append { .. }
        | Op::Truncate { .. }
        | Op::Sync { .. }
        | Op::Lock { .. }
        | Op::Stat { .. }
        | Op::HardLink { .. }
        | Op::Rename { .. }
        | Op::Remove { .. }
        | Op::MakeDirectory { .. }
        | Op::List { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => unreachable!("a store write completes a write"),
    }
}

pub(super) fn read_buffer(kind: Op) -> Box<[u8]> {
    match kind {
        Op::Read { buf, .. } => buf,
        Op::Socket { .. }
        | Op::Bind { .. }
        | Op::Listen { .. }
        | Op::Accept { .. }
        | Op::Connect { .. }
        | Op::Recv { .. }
        | Op::Send { .. }
        | Op::Shutdown { .. }
        | Op::Close { .. }
        | Op::Open { .. }
        | Op::Write { .. }
        | Op::Append { .. }
        | Op::Truncate { .. }
        | Op::Sync { .. }
        | Op::Lock { .. }
        | Op::Stat { .. }
        | Op::HardLink { .. }
        | Op::Rename { .. }
        | Op::Remove { .. }
        | Op::MakeDirectory { .. }
        | Op::List { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => unreachable!("a version read completes a read"),
    }
}

fn advance(io: &mut FileIo, mut store: Store, state: State, op: Op, subs: &mut Queue<Submit>) -> Work {
    store.state = state;
    issue(io, store, op, subs)
}

fn create_temp(
    io: &mut FileIo,
    store: Store,
    parent: Fd,
    input: Input,
    attempts: u8,
    subs: &mut Queue<Submit>,
) -> Work {
    let name = temp_name(io, &store.target);
    let attempts = attempts.checked_add(1).expect("bounded attempts");
    let how = if store.no_follow {
        OpenHow::CreateNoFollow { mode: Some(input.mode) }
    } else {
        OpenHow::Create { mode: Some(input.mode) }
    };
    let op = Op::Open { root: parent, path: name.clone(), how };
    advance(io, store, State::TempCreating { parent, input, name, attempts }, op, subs)
}

fn open_check(io: &mut FileIo, store: Store, named: Named, subs: &mut Queue<Submit>) -> Work {
    let how = match store.expected {
        Expect::Any => unreachable!("unconditional stores do not recheck content"),
        Expect::Absent => OpenHow::ReadNoFollow,
        Expect::Digest(_) => {
            if store.no_follow {
                OpenHow::ReadNoFollow
            } else {
                OpenHow::Read
            }
        }
    };
    let op = Op::Open { root: named.parent, path: store.target.clone(), how };
    advance(io, store, State::CheckOpening(named), op, subs)
}

fn rename(io: &mut FileIo, store: Store, named: Named, subs: &mut Queue<Submit>) -> Work {
    let no_replace = match store.expected {
        Expect::Absent => true,
        Expect::Any | Expect::Digest(_) => false,
    };
    let op = Op::Rename {
        from_dir: named.parent,
        from: named.name.clone(),
        to_dir: named.parent,
        to: store.target.clone(),
        no_replace,
    };
    advance(io, store, State::Renaming(named), op, subs)
}

fn read_check(io: &mut FileIo, env: &Env<Limits>, store: Store, reading: Reading, subs: &mut Queue<Submit>) -> Work {
    let buffer = alloc::vec![0_u8; usize_of(env.limits.read)].into_boxed_slice();
    let op = Op::read(reading.check.fd, buffer, reading.seen_bytes).expect("positive bounded read");
    advance(io, store, State::CheckReading(reading), op, subs)
}

fn terminal(store: Store, failure: Failure, left: Left, events: &mut Queue<Event>) -> Work {
    let (committed, residue) = match left {
        Left::Nothing => (false, None),
        Left::Committed => (true, None),
        Left::Residue(residue) => (false, Some(residue)),
    };
    match failure {
        Failure::Kernel(Error::Cancelled) if !committed && residue.is_none() => {
            events.push(Event::Cancelled { owner: store.owner });
        }
        Failure::Kernel(error) => events.push(Event::Failed { owner: store.owner, error, committed, residue }),
        Failure::Conflict { now } => events.push(Event::Conflict { owner: store.owner, now }),
    }
    Work::Idle
}

fn settle(io: &mut FileIo, store: Store, failure: Failure, step: Settle, subs: &mut Queue<Submit>) -> Work {
    let op = match &step {
        Settle::ClosingCheck(opened) | Settle::ClosingTemp(opened) => Op::Close { fd: opened.fd },
        Settle::ClosingOld { old, .. } => Op::Close { fd: *old },
        Settle::Removing(named) => Op::Remove { dir: named.parent, name: named.name.clone(), directory: false },
        Settle::ClosingParent { parent, .. } => Op::Close { fd: *parent },
    };
    advance(io, store, State::Settling { failure, step }, op, subs)
}

/// A completed close releases its descriptor even when the close failed.
fn abandon(
    io: &mut FileIo,
    store: Store,
    state: State,
    failure: Failure,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let step = match state {
        State::ParentOpening { .. } => return terminal(store, failure, Left::Nothing, events),
        State::OldOpening { parent, .. } | State::OldClosing { parent, .. } | State::TempCreating { parent, .. } => {
            Settle::ClosingParent { parent, left: Left::Nothing }
        }
        State::OldStating { parent, old, .. } => Settle::ClosingOld { parent, old },
        State::TempWriting(temp) | State::TempSyncing(temp) => Settle::ClosingTemp(temp),
        State::TempClosing(temp) => Settle::Removing(temp.named),
        State::CheckOpening(named) | State::Renaming(named) => Settle::Removing(named),
        State::CheckStating(check) | State::CheckReading(Reading { check, .. }) => Settle::ClosingCheck(check),
        State::CheckClosing { check, .. } => Settle::Removing(check.named),
        State::DirectorySyncing { parent } => Settle::ClosingParent { parent, left: Left::Committed },
        State::ParentClosing { .. } => return terminal(store, failure, Left::Committed, events),
        State::Settling { .. } | State::Finished => unreachable!("only an active operation is abandoned"),
    };
    settle(io, store, failure, step, subs)
}

fn finish_cleanup(
    io: &mut FileIo,
    store: Store,
    failure: Failure,
    step: Settle,
    result: Result<Done, Error>,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let later = match result {
        Ok(_) => failure,
        Err(error) => Failure::Kernel(error),
    };
    match step {
        Settle::ClosingCheck(check) => settle(io, store, later, Settle::Removing(check.named), subs),
        Settle::ClosingOld { parent, .. } => {
            settle(io, store, later, Settle::ClosingParent { parent, left: Left::Nothing }, subs)
        }
        Settle::ClosingTemp(temp) => settle(io, store, later, Settle::Removing(temp.named), subs),
        Settle::Removing(named) => {
            let (failure, left) = match result {
                Ok(_) | Err(Error::NotFound) => (failure, Left::Nothing),
                Err(error) => (later, Left::Residue(Residue { name: named.name, error })),
            };
            settle(io, store, failure, Settle::ClosingParent { parent: named.parent, left }, subs)
        }
        Settle::ClosingParent { left, .. } => terminal(store, later, left, events),
    }
}

pub(super) fn up(
    io: &mut FileIo,
    env: &Env<Limits>,
    mut store: Store,
    complete: Complete,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let state = mem::replace(&mut store.state, State::Finished);
    match state {
        State::Settling { failure, step } => finish_cleanup(io, store, failure, step, complete.result, events, subs),
        state @ (State::ParentOpening { .. }
        | State::OldOpening { .. }
        | State::OldStating { .. }
        | State::OldClosing { .. }
        | State::TempCreating { .. }
        | State::TempWriting(_)
        | State::TempSyncing(_)
        | State::TempClosing(_)
        | State::CheckOpening(_)
        | State::CheckStating(_)
        | State::CheckReading(_)
        | State::CheckClosing { .. }
        | State::Renaming(_)
        | State::DirectorySyncing { .. }
        | State::ParentClosing { .. }) => match complete.result {
            Ok(done) => success(io, env, store, state, complete.kind, done, events, subs),
            Err(error) => failed_operation(io, store, state, error, events, subs),
        },
        State::Finished => unreachable!("a finished store has no operation in flight"),
    }
}

fn failed_operation(
    io: &mut FileIo,
    store: Store,
    state: State,
    error: Error,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let failure = Failure::Kernel(error);
    match state {
        State::OldOpening { parent, input } => {
            let skip = match store.expected {
                Expect::Any | Expect::Absent => {
                    error == Error::NotFound
                        || error == Error::Permission
                        || error == Error::TooManyLinks
                        || error == Error::Escape
                        || error == Error::NotAFile
                }
                Expect::Digest(_) => error == Error::NotFound || error == Error::Permission,
            };
            let failure = match store.expected {
                Expect::Absent if skip && error != Error::NotFound => Failure::Conflict { now: None },
                Expect::Any | Expect::Absent | Expect::Digest(_) => {
                    if skip {
                        return create_temp(io, store, parent, input, 0, subs);
                    }
                    failure
                }
            };
            settle(io, store, failure, Settle::ClosingParent { parent, left: Left::Nothing }, subs)
        }
        State::TempCreating { parent, input, attempts, .. } => {
            if error == Error::Exists && attempts < ATTEMPTS {
                create_temp(io, store, parent, input, attempts, subs)
            } else {
                settle(io, store, failure, Settle::ClosingParent { parent, left: Left::Nothing }, subs)
            }
        }
        State::CheckOpening(named) => {
            let failure = match store.expected {
                Expect::Absent if error == Error::NotFound => return rename(io, store, named, subs),
                Expect::Digest(_) if error == Error::NotFound => Failure::Conflict { now: None },
                Expect::Absent
                    if error == Error::Permission
                        || error == Error::TooManyLinks
                        || error == Error::Escape
                        || error == Error::NotAFile
                        || error == Error::IsADirectory =>
                {
                    Failure::Conflict { now: None }
                }
                Expect::Any => unreachable!("unconditional stores skip the recheck"),
                Expect::Absent | Expect::Digest(_) => failure,
            };
            settle(io, store, failure, Settle::Removing(named), subs)
        }
        State::Renaming(named) => {
            let failure = match store.expected {
                Expect::Absent if error == Error::Exists => Failure::Conflict { now: None },
                Expect::Any | Expect::Absent | Expect::Digest(_) => failure,
            };
            settle(io, store, failure, Settle::Removing(named), subs)
        }
        state @ (State::ParentOpening { .. }
        | State::OldStating { .. }
        | State::OldClosing { .. }
        | State::TempWriting(_)
        | State::TempSyncing(_)
        | State::TempClosing(_)
        | State::CheckStating(_)
        | State::CheckReading(_)
        | State::CheckClosing { .. }
        | State::DirectorySyncing { .. }
        | State::ParentClosing { .. }) => abandon(io, store, state, failure, events, subs),
        State::Settling { .. } | State::Finished => unreachable!("cleanup completions are handled separately"),
    }
}

pub(super) fn stopped(
    io: &mut FileIo,
    mut store: Store,
    complete: Complete,
    reason: Error,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let state = mem::replace(&mut store.state, State::Finished);
    let state = match complete.result {
        Ok(Done::Fd(fd)) => match state {
            State::ParentOpening { input } => State::OldOpening { parent: fd, input },
            State::OldOpening { parent, input } => State::OldStating { parent, old: fd, input },
            State::TempCreating { parent, name, .. } => {
                State::TempSyncing(Opened { named: Named { parent, name }, fd })
            }
            State::CheckOpening(named) => State::CheckStating(Opened { named, fd }),
            State::OldStating { .. }
            | State::OldClosing { .. }
            | State::TempWriting(_)
            | State::TempSyncing(_)
            | State::TempClosing(_)
            | State::CheckStating(_)
            | State::CheckReading(_)
            | State::CheckClosing { .. }
            | State::Renaming(_)
            | State::DirectorySyncing { .. }
            | State::ParentClosing { .. }
            | State::Settling { .. }
            | State::Finished => unreachable!("only an open returns a descriptor"),
        },
        Ok(
            Done::Nothing
            | Done::Count(_)
            | Done::Stat(_)
            | Done::Accepted { .. }
            | Done::Bound(_)
            | Done::Spawned { .. }
            | Done::Exit(_)
            | Done::Usage(_)
            | Done::Signalled(_)
            | Done::Window(_),
        ) => match state {
            State::Renaming(named) => State::DirectorySyncing { parent: named.parent },
            state @ (State::ParentOpening { .. }
            | State::OldOpening { .. }
            | State::OldStating { .. }
            | State::OldClosing { .. }
            | State::TempCreating { .. }
            | State::TempWriting(_)
            | State::TempSyncing(_)
            | State::TempClosing(_)
            | State::CheckOpening(_)
            | State::CheckStating(_)
            | State::CheckReading(_)
            | State::CheckClosing { .. }
            | State::DirectorySyncing { .. }
            | State::ParentClosing { .. }) => state,
            State::Settling { .. } | State::Finished => unreachable!("only an active operation can be stopped"),
        },
        Err(_) => state,
    };
    abandon(io, store, state, Failure::Kernel(reason), events, subs)
}

#[expect(clippy::too_many_arguments, reason = "dispatch owns the source state and its one kernel completion")]
fn success(
    io: &mut FileIo,
    env: &Env<Limits>,
    store: Store,
    state: State,
    kind: Op,
    done: Done,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    match state {
        State::ParentOpening { input } => parent_opened(io, store, input, done, subs),
        State::OldOpening { parent, input } => old_opened(io, store, parent, input, done, subs),
        State::OldStating { parent, old, input } => old_stated(io, store, parent, old, input, done, subs),
        State::OldClosing { parent, old, input } => old_closed(io, store, parent, old, input, done, subs),
        State::TempCreating { parent, input, name, .. } => temp_created(io, store, parent, input, name, done, subs),
        State::TempWriting(temp) => temp_written(io, store, temp, kind, done, subs),
        State::TempSyncing(temp) => temp_synced(io, store, temp, done, subs),
        State::TempClosing(temp) => temp_closed(io, store, temp, done, subs),
        State::CheckOpening(named) => check_opened(io, store, named, done, subs),
        State::CheckStating(check) => check_stated(io, env, store, check, done, subs),
        State::CheckReading(reading) => check_read(io, env, store, reading, kind, done, subs),
        State::CheckClosing { check, verdict } => check_closed(io, store, check, verdict, done, subs),
        State::Renaming(named) => renamed(io, store, named, done, subs),
        State::DirectorySyncing { parent } => directory_synced(io, store, parent, done, subs),
        State::ParentClosing { parent } => parent_closed(store, parent, done, events),
        State::Settling { .. } | State::Finished => unreachable!("only an active operation completes successfully"),
    }
}

fn parent_opened(io: &mut FileIo, store: Store, input: Input, done: Done, subs: &mut Queue<Submit>) -> Work {
    let parent = done_fd(done);
    let how = if store.no_follow { OpenHow::ReadNoFollow } else { OpenHow::Read };
    let op = Op::Open { root: parent, path: store.target.clone(), how };
    advance(io, store, State::OldOpening { parent, input }, op, subs)
}

fn old_opened(io: &mut FileIo, store: Store, parent: Fd, input: Input, done: Done, subs: &mut Queue<Submit>) -> Work {
    let old = done_fd(done);
    match store.expected {
        Expect::Absent => settle(io, store, Failure::Conflict { now: None }, Settle::ClosingOld { parent, old }, subs),
        Expect::Any | Expect::Digest(_) => {
            advance(io, store, State::OldStating { parent, old, input }, Op::Stat { fd: old }, subs)
        }
    }
}

fn old_stated(
    io: &mut FileIo,
    store: Store,
    parent: Fd,
    old: Fd,
    mut input: Input,
    done: Done,
    subs: &mut Queue<Submit>,
) -> Work {
    let stat = done_stat(done);
    if stat.kind != Kind::File {
        let failure = match store.expected {
            Expect::Absent => Failure::Conflict { now: None },
            Expect::Any | Expect::Digest(_) => Failure::Kernel(Error::IsADirectory),
        };
        return settle(io, store, failure, Settle::ClosingOld { parent, old }, subs);
    }
    if store.private_owner.is_none() {
        input.mode = stat.mode;
    }
    advance(io, store, State::OldClosing { parent, old, input }, Op::Close { fd: old }, subs)
}

fn old_closed(
    io: &mut FileIo,
    store: Store,
    parent: Fd,
    _old: Fd,
    input: Input,
    done: Done,
    subs: &mut Queue<Submit>,
) -> Work {
    done_nothing(done);
    create_temp(io, store, parent, input, 0, subs)
}

fn temp_created(
    io: &mut FileIo,
    store: Store,
    parent: Fd,
    input: Input,
    name: Box<[u8]>,
    done: Done,
    subs: &mut Queue<Submit>,
) -> Work {
    let fd = done_fd(done);
    let temp = Opened { named: Named { parent, name }, fd };
    let bytes = input.bytes.0.into_boxed_slice();
    if bytes.is_empty() {
        return advance(io, store, State::TempSyncing(temp), Op::Sync { fd }, subs);
    }
    let op = Op::write(fd, bytes, 0, 0).expect("bounded nonempty store bytes");
    advance(io, store, State::TempWriting(temp), op, subs)
}

fn temp_written(io: &mut FileIo, store: Store, temp: Opened, kind: Op, done: Done, subs: &mut Queue<Submit>) -> Work {
    let count = done_count(done);
    let (fd, bytes, from, at) = write_parts(kind);
    if count == 0 {
        return settle(io, store, Failure::Kernel(Error::Other(5)), Settle::ClosingTemp(temp), subs);
    }
    let next = from.checked_add(count).expect("write count fits");
    if usize_of(next) == bytes.len() {
        return advance(io, store, State::TempSyncing(temp), Op::Sync { fd }, subs);
    }
    let offset = at.checked_add(u64::from(count)).expect("bounded offset");
    let op = Op::write(fd, bytes, next, offset).expect("short write continues inside bytes");
    advance(io, store, State::TempWriting(temp), op, subs)
}

fn temp_synced(io: &mut FileIo, store: Store, temp: Opened, done: Done, subs: &mut Queue<Submit>) -> Work {
    done_nothing(done);
    let op = Op::Close { fd: temp.fd };
    advance(io, store, State::TempClosing(temp), op, subs)
}

fn temp_closed(io: &mut FileIo, store: Store, temp: Opened, done: Done, subs: &mut Queue<Submit>) -> Work {
    done_nothing(done);
    match store.expected {
        Expect::Any => rename(io, store, temp.named, subs),
        Expect::Absent | Expect::Digest(_) => open_check(io, store, temp.named, subs),
    }
}

fn check_opened(io: &mut FileIo, store: Store, named: Named, done: Done, subs: &mut Queue<Submit>) -> Work {
    let fd = done_fd(done);
    let check = Opened { named, fd };
    match store.expected {
        Expect::Absent => settle(io, store, Failure::Conflict { now: None }, Settle::ClosingCheck(check), subs),
        Expect::Digest(_) => advance(io, store, State::CheckStating(check), Op::Stat { fd }, subs),
        Expect::Any => unreachable!("unconditional stores skip the recheck"),
    }
}

fn check_stated(
    io: &mut FileIo,
    env: &Env<Limits>,
    store: Store,
    check: Opened,
    done: Done,
    subs: &mut Queue<Submit>,
) -> Work {
    let stat = done_stat(done);
    if stat.kind == Kind::File {
        return read_check(io, env, store, Reading { check, seen: Sha256::new(), seen_bytes: 0 }, subs);
    }
    let op = Op::Close { fd: check.fd };
    let verdict = Verdict::Refused(Failure::Kernel(Error::IsADirectory));
    advance(io, store, State::CheckClosing { check, verdict }, op, subs)
}

fn check_read(
    io: &mut FileIo,
    env: &Env<Limits>,
    store: Store,
    mut reading: Reading,
    kind: Op,
    done: Done,
    subs: &mut Queue<Submit>,
) -> Work {
    let count = done_count(done);
    let buf = read_buffer(kind);
    reading.seen.update(buf.get(..usize_of(count)).expect("read count within buffer"));
    reading.seen_bytes = reading.seen_bytes.checked_add(u64::from(count)).expect("file length fits u64");
    if count != 0 {
        return read_check(io, env, store, reading, subs);
    }
    let now = Digest::from_bytes(reading.seen.finish());
    let verdict = match store.expected {
        Expect::Any => unreachable!("unconditional stores do not read a version"),
        Expect::Absent => unreachable!("expected absence is checked without a read"),
        Expect::Digest(expected) => {
            if now == expected {
                Verdict::Matches
            } else {
                Verdict::Refused(Failure::Conflict { now: Some(now) })
            }
        }
    };
    let op = Op::Close { fd: reading.check.fd };
    advance(io, store, State::CheckClosing { check: reading.check, verdict }, op, subs)
}

fn check_closed(
    io: &mut FileIo,
    store: Store,
    check: Opened,
    verdict: Verdict,
    done: Done,
    subs: &mut Queue<Submit>,
) -> Work {
    done_nothing(done);
    match verdict {
        Verdict::Matches => rename(io, store, check.named, subs),
        Verdict::Refused(failure) => settle(io, store, failure, Settle::Removing(check.named), subs),
    }
}

fn renamed(io: &mut FileIo, store: Store, named: Named, done: Done, subs: &mut Queue<Submit>) -> Work {
    done_nothing(done);
    let parent = named.parent;
    advance(io, store, State::DirectorySyncing { parent }, Op::Sync { fd: parent }, subs)
}

fn directory_synced(io: &mut FileIo, store: Store, parent: Fd, done: Done, subs: &mut Queue<Submit>) -> Work {
    done_nothing(done);
    advance(io, store, State::ParentClosing { parent }, Op::Close { fd: parent }, subs)
}

fn parent_closed(store: Store, _parent: Fd, done: Done, events: &mut Queue<Event>) -> Work {
    done_nothing(done);
    events.push(Event::Stored { owner: store.owner, digest: store.produced });
    Work::Idle
}
