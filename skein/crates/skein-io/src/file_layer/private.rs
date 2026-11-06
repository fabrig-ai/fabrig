//! Opening private roots and making directories (skein/docs/design/io.md, 5.3).
//! Each state owns its descriptors until cleanup (skein/docs/foundation/programming-model.md, 5.4).
//! `start`, `up`, and `stopped` run only through `FileIo`'s entry points.

use super::{FileIo, Pending, Work};
use crate::file::{Event, Unsafe};
use crate::kernel::{Complete, Done, Error, Fd, Kind, Op, OpenHow, Stat, Submit};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};

#[derive(Clone, Copy, Debug)]
pub(super) enum Action {
    Make { mode: u32 },
    Private { user: u32 },
}

#[derive(Debug)]
enum Then {
    Made,
    Open { user: u32 },
}

#[derive(Debug)]
enum MayMake {
    Try,
    Tried,
}

#[derive(Debug)]
enum Held {
    Parent(Fd),
    Gone,
}

#[derive(Debug)]
enum Check {
    Root { user: u32 },
    File { user: u32 },
}

#[derive(Debug)]
enum Answer {
    Terminal(Event),
    Admit { file: Fd, user: u32 },
}

#[derive(Debug)]
enum State {
    ParentOpening { name: Box<[u8]>, action: Action },
    Making { parent: Fd, name: Box<[u8]>, then: Then },
    SyncingParent { parent: Fd, name: Box<[u8]>, then: Then },
    Opening { parent: Fd, name: Box<[u8]>, user: u32, may_make: MayMake },
    Stating { parent: Fd, file: Fd, user: u32 },
    DeniedOpening { parent: Held, name: Box<[u8]>, denied: Event, check: Check },
    DeniedStating { parent: Held, file: Fd, denied: Event, check: Check },
    ClosingFile { parent: Held, terminal: Event },
    ClosingParent { parent: Fd, answer: Answer },
}

#[derive(Debug)]
pub(super) struct Opening {
    owner: Token,
    state: State,
}

impl Opening {
    pub(super) const fn owner(&self) -> Token {
        self.owner
    }
    pub(super) fn can_abandon(&self) -> bool {
        match &self.state {
            State::ParentOpening { .. } | State::Opening { .. } | State::Stating { .. } => true,
            State::Making { then, .. } | State::SyncingParent { then, .. } => match then {
                Then::Made => false,
                Then::Open { .. } => true,
            },
            State::ClosingParent { answer, .. } => match answer {
                Answer::Terminal(..) => false,
                Answer::Admit { .. } => true,
            },
            State::DeniedOpening { .. } | State::DeniedStating { .. } | State::ClosingFile { .. } => false,
        }
    }

    pub(super) fn can_cancel(&self) -> bool {
        match self.state {
            State::ParentOpening { .. } | State::Opening { .. } | State::DeniedOpening { .. } => true,
            State::Making { .. }
            | State::SyncingParent { .. }
            | State::Stating { .. }
            | State::DeniedStating { .. }
            | State::ClosingFile { .. }
            | State::ClosingParent { .. } => false,
        }
    }
}

// A transition either submits the target state's operation or answers after cleanup.
enum Next {
    Issue { state: State, operation: Op },
    Answer(Answer),
}

fn follow(io: &mut FileIo, owner: Token, next: Next, events: &mut Queue<Event>, subs: &mut Queue<Submit>) -> Work {
    match next {
        Next::Issue { state, operation } => issue(io, owner, state, operation, subs),
        Next::Answer(answer) => {
            let event = match answer {
                Answer::Terminal(event) => event,
                Answer::Admit { file, user } => {
                    let file = io.insert_private(file, user).expect("admission reserved an open file slot");
                    Event::Opened { owner, file, len: 0 }
                }
            };
            events.push(event);
            Work::Idle
        }
    }
}

pub(super) fn unsafe_root(stat: Stat, expected: u32) -> Option<Unsafe> {
    if stat.owner != expected {
        return Some(Unsafe::Owner { found: stat.owner, expected });
    }
    if stat.kind == Kind::Symlink {
        return Some(Unsafe::Link);
    }
    if stat.kind != Kind::Directory {
        return Some(Unsafe::Kind(stat.kind));
    }
    if stat.mode & 0o077 != 0 {
        return Some(Unsafe::Mode(stat.mode & 0o077));
    }
    None
}

pub(super) fn unsafe_file(stat: Stat, expected: u32) -> Option<Unsafe> {
    if stat.owner != expected {
        return Some(Unsafe::Owner { found: stat.owner, expected });
    }
    if stat.kind == Kind::Symlink {
        return Some(Unsafe::Link);
    }
    if stat.kind != Kind::File {
        return Some(Unsafe::Kind(stat.kind));
    }
    if stat.mode & 0o077 != 0 {
        return Some(Unsafe::Mode(stat.mode & 0o077));
    }
    if stat.links != 1 {
        return Some(Unsafe::Links(stat.links));
    }
    None
}

pub(super) fn start(
    io: &mut FileIo,
    owner: Token,
    root: Token,
    path: Box<[u8]>,
    action: Action,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let Some(fd) = io.file(root) else {
        return super::terminal_failure(events, owner, Error::NotFound);
    };
    if path.len() >= 4096 || path.contains(&0) {
        return super::terminal_failure(events, owner, Error::InvalidArgument);
    }
    let Some((parent, name)) = super::store::split_path(&path) else {
        return super::terminal_failure(events, owner, Error::InvalidArgument);
    };
    match action {
        Action::Private { .. } if io.files.len() == io.files.capacity() => {
            return super::terminal_failure(events, owner, Error::TooManyOpenFiles);
        }
        Action::Private { .. } | Action::Make { .. } => {}
    }
    let no_follow = match action {
        Action::Private { .. } => true,
        Action::Make { mode } => {
            if mode & !crate::kernel::PERMISSIONS != 0 {
                return super::terminal_failure(events, owner, Error::InvalidArgument);
            }
            io.private_owner(root).is_some()
        }
    };
    let how = if no_follow { OpenHow::DirectoryNoFollow } else { OpenHow::Directory };
    issue(io, owner, State::ParentOpening { name, action }, Op::Open { root: fd, path: parent, how }, subs)
}

fn issue(io: &mut FileIo, owner: Token, state: State, operation: Op, subs: &mut Queue<Submit>) -> Work {
    super::issue(io, Pending::Private(Opening { owner, state }), operation, subs)
}

fn open(parent: Fd, name: Box<[u8]>, user: u32, may_make: MayMake) -> Next {
    let operation = Op::Open { root: parent, path: name.clone(), how: OpenHow::ReadNoFollow };
    Next::Issue { state: State::Opening { parent, name, user, may_make }, operation }
}

fn close_parent(parent: Held, answer: Answer) -> Next {
    match parent {
        Held::Parent(parent) => {
            Next::Issue { state: State::ClosingParent { parent, answer }, operation: Op::Close { fd: parent } }
        }
        Held::Gone => Next::Answer(answer),
    }
}

fn close_file(parent: Held, file: Fd, terminal: Event) -> Next {
    Next::Issue { state: State::ClosingFile { parent, terminal }, operation: Op::Close { fd: file } }
}

fn parent_opened(owner: Token, name: Box<[u8]>, action: Action, result: Result<Done, Error>) -> Next {
    match result {
        Ok(done) => {
            let parent = super::store::done_fd(done);
            match action {
                Action::Private { user } => open(parent, name, user, MayMake::Try),
                Action::Make { mode } => {
                    let operation = Op::MakeDirectory { dir: parent, name: name.clone(), mode };
                    Next::Issue { state: State::Making { parent, name, then: Then::Made }, operation }
                }
            }
        }
        Err(error) => {
            let terminal = match action {
                Action::Private { .. } if error == Error::TooManyLinks => Event::Refused { owner, found: Unsafe::Link },
                Action::Private { .. } | Action::Make { .. } => Event::failed(owner, error),
            };
            Next::Answer(Answer::Terminal(terminal))
        }
    }
}

fn made(owner: Token, parent: Fd, name: Box<[u8]>, then: Then, result: Result<Done, Error>) -> Next {
    match result {
        Ok(done) => {
            super::store::done_nothing(done);
            Next::Issue { state: State::SyncingParent { parent, name, then }, operation: Op::Sync { fd: parent } }
        }
        Err(error) => match then {
            Then::Open { user } if error == Error::Exists => open(parent, name, user, MayMake::Tried),
            Then::Open { .. } | Then::Made => {
                close_parent(Held::Parent(parent), Answer::Terminal(Event::failed(owner, error)))
            }
        },
    }
}

fn synced(owner: Token, parent: Fd, name: Box<[u8]>, then: Then, result: Result<Done, Error>) -> Next {
    match result {
        Ok(done) => {
            super::store::done_nothing(done);
            match then {
                Then::Open { user } => open(parent, name, user, MayMake::Tried),
                Then::Made => close_parent(Held::Parent(parent), Answer::Terminal(Event::Made { owner })),
            }
        }
        Err(error) => close_parent(Held::Parent(parent), Answer::Terminal(committed(owner, error))),
    }
}

fn committed(owner: Token, error: Error) -> Event {
    Event::Failed { owner, error, committed: true, residue: None }
}

fn opened(
    owner: Token,
    parent: Fd,
    name: Box<[u8]>,
    user: u32,
    may_make: MayMake,
    result: Result<Done, Error>,
) -> Next {
    match result {
        Ok(done) => {
            let file = super::store::done_fd(done);
            Next::Issue { state: State::Stating { parent, file, user }, operation: Op::Stat { fd: file } }
        }
        Err(Error::Permission) => {
            let operation = Op::Open { root: parent, path: name.clone(), how: OpenHow::PathNoFollow };
            Next::Issue {
                state: State::DeniedOpening {
                    parent: Held::Parent(parent),
                    name,
                    denied: Event::failed(owner, Error::Permission),
                    check: Check::Root { user },
                },
                operation,
            }
        }
        Err(error) => {
            if error == Error::NotFound {
                match may_make {
                    MayMake::Try => {
                        let operation = Op::MakeDirectory { dir: parent, name: name.clone(), mode: 0o700 };
                        return Next::Issue {
                            state: State::Making { parent, name, then: Then::Open { user } },
                            operation,
                        };
                    }
                    MayMake::Tried => {}
                }
            }
            let terminal = match unsafe_open(error, true) {
                Some(found) => Event::Refused { owner, found },
                None => Event::failed(owner, error),
            };
            close_parent(Held::Parent(parent), Answer::Terminal(terminal))
        }
    }
}

fn stated(owner: Token, parent: Fd, file: Fd, user: u32, result: Result<Done, Error>) -> Next {
    let terminal = match result {
        Ok(done) => match unsafe_root(super::store::done_stat(done), user) {
            Some(found) => Event::Refused { owner, found },
            None => return close_parent(Held::Parent(parent), Answer::Admit { file, user }),
        },
        Err(error) => Event::failed(owner, error),
    };
    close_file(Held::Parent(parent), file, terminal)
}

fn denied_opened(parent: Held, denied: Event, check: Check, result: Result<Done, Error>) -> Next {
    match result {
        Ok(done) => {
            let file = super::store::done_fd(done);
            Next::Issue {
                state: State::DeniedStating { parent, file, denied, check },
                operation: Op::Stat { fd: file },
            }
        }
        Err(_error) => close_parent(parent, Answer::Terminal(denied)),
    }
}

fn denied_stated(
    owner: Token,
    parent: Held,
    file: Fd,
    denied: Event,
    check: Check,
    result: Result<Done, Error>,
) -> Next {
    let terminal = match result {
        Ok(done) => {
            let stat = super::store::done_stat(done);
            let found = match check {
                Check::Root { user } => unsafe_root(stat, user),
                Check::File { user } => unsafe_file(stat, user),
            };
            match found {
                Some(found) => Event::Refused { owner, found },
                None => denied,
            }
        }
        Err(_error) => denied,
    };
    close_file(parent, file, terminal)
}

fn file_closed(parent: Held, terminal: Event, result: Result<Done, Error>) -> Next {
    match result {
        Ok(done) => super::store::done_nothing(done),
        Err(_error) => {}
    }
    close_parent(parent, Answer::Terminal(terminal))
}

fn parent_closed(owner: Token, _parent: Fd, answer: Answer, result: Result<Done, Error>) -> Next {
    match result {
        Ok(done) => {
            super::store::done_nothing(done);
            Next::Answer(answer)
        }
        Err(error) => match answer {
            Answer::Terminal(terminal) => Next::Answer(Answer::Terminal(terminal)),
            Answer::Admit { file, .. } => close_file(Held::Gone, file, Event::failed(owner, error)),
        },
    }
}

pub(super) fn up(
    io: &mut FileIo,
    opening: Opening,
    complete: Complete,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let owner = opening.owner;
    let result = complete.result;
    let next = match opening.state {
        State::ParentOpening { name, action } => parent_opened(owner, name, action, result),
        State::Making { parent, name, then } => made(owner, parent, name, then, result),
        State::SyncingParent { parent, name, then } => synced(owner, parent, name, then, result),
        State::Opening { parent, name, user, may_make } => opened(owner, parent, name, user, may_make, result),
        State::Stating { parent, file, user } => stated(owner, parent, file, user, result),
        State::DeniedOpening { parent, name: _name, denied, check } => denied_opened(parent, denied, check, result),
        State::DeniedStating { parent, file, denied, check } => {
            denied_stated(owner, parent, file, denied, check, result)
        }
        State::ClosingFile { parent, terminal } => file_closed(parent, terminal, result),
        State::ClosingParent { parent, answer } => parent_closed(owner, parent, answer, result),
    };
    follow(io, owner, next, events, subs)
}

pub(super) fn stopped(
    io: &mut FileIo,
    opening: Opening,
    complete: Complete,
    reason: Error,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let owner = opening.owner;
    let terminal = if reason == Error::Cancelled { Event::Cancelled { owner } } else { Event::failed(owner, reason) };
    let next = match opening.state {
        State::ParentOpening { .. } => match complete.result {
            Ok(done) => close_parent(Held::Parent(super::store::done_fd(done)), Answer::Terminal(terminal)),
            Err(_error) => Next::Answer(Answer::Terminal(terminal)),
        },
        State::Opening { parent, .. } => match complete.result {
            Ok(done) => close_file(Held::Parent(parent), super::store::done_fd(done), terminal),
            Err(_error) => close_parent(Held::Parent(parent), Answer::Terminal(terminal)),
        },
        State::DeniedOpening { parent, denied, .. } => match complete.result {
            Ok(done) => close_file(parent, super::store::done_fd(done), denied),
            Err(_error) => close_parent(parent, Answer::Terminal(denied)),
        },
        State::SyncingParent { parent, .. } => {
            let terminal = if reason == Error::Cancelled { terminal } else { committed(owner, reason) };
            close_parent(Held::Parent(parent), Answer::Terminal(terminal))
        }
        State::Making { parent, .. } => close_parent(Held::Parent(parent), Answer::Terminal(terminal)),
        State::Stating { parent, file, .. } => close_file(Held::Parent(parent), file, terminal),
        State::DeniedStating { parent, file, denied, .. } => close_file(parent, file, denied),
        State::ClosingFile { parent, terminal } => close_parent(parent, Answer::Terminal(terminal)),
        State::ClosingParent { answer, .. } => match answer {
            Answer::Terminal(terminal) => Next::Answer(Answer::Terminal(terminal)),
            Answer::Admit { file, .. } => close_file(Held::Gone, file, terminal),
        },
    };
    follow(io, owner, next, events, subs)
}

/// Kernel path refusals that already establish unsafe private metadata.
pub(super) fn unsafe_open(error: Error, private: bool) -> Option<Unsafe> {
    if private && error == Error::TooManyLinks {
        return Some(Unsafe::Link);
    }
    if private && error == Error::NotAFile {
        return Some(Unsafe::Kind(Kind::Other));
    }
    None
}

pub(super) fn refuse_file(
    io: &mut FileIo,
    owner: Token,
    fd: Fd,
    found: Unsafe,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let next = close_file(Held::Gone, fd, Event::Refused { owner, found });
    follow(io, owner, next, events, subs)
}

/// A denied private load is probed only to name its refusal, never to admit bytes.
pub(super) fn denied_load(io: &mut FileIo, owner: Token, loading: super::Loading, subs: &mut Queue<Submit>) -> Work {
    let operation = Op::Open { root: loading.root, path: loading.path.clone(), how: OpenHow::PathNoFollow };
    let state = State::DeniedOpening {
        parent: Held::Gone,
        name: loading.path,
        denied: Event::failed(owner, Error::Permission),
        check: Check::File { user: loading.user },
    };
    issue(io, owner, state, operation, subs)
}
