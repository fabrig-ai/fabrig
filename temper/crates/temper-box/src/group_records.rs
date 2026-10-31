//! Kind files and their started probes (temper/docs/design/draft/playground-reset.md, 3 and 4).
use crate::{
    lines::joined,
    machine::{Input, Why},
    pipe,
    records::path,
};
use alloc::boxed::Box;
use skein_io::{
    file::{self, Event, Request},
    file_layer::{self, FileIo},
    kernel::{Complete, Error, Fd, Op, Submit},
};
use skein_lib::{Decimal, Duration, List, Queue, Time, Token, bytes};
use temper_forge_setup::machine::{BoxKind, group_file, group_path, parse_group_file};
const TAG: u64 = 1_u64 << 62;
const OWNER: Token = Token::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Group {
    pub forge: u16,
    pub kind: BoxKind,
    pub started: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Open,
    List { dir: Token },
    ListClose { dir: Token },
    Load,
    Probe { forge: u16, kind: BoxKind },
    ProbeClose { forge: u16, kind: BoxKind, file: Token },
    Idle,
    RootClose,
    Closed,
    MakeBox { forge: u16, kind: BoxKind },
    MakeGroups { forge: u16, kind: BoxKind },
    Store { forge: u16, kind: BoxKind },
    UnlinkOpen { forge: u16, started: bool },
    Unlink { forge: u16, started: bool, dir: Token },
    UnlinkClose { forge: u16, started: bool, dir: Token, error: Option<Error> },
}
#[derive(Debug)]
pub(crate) struct GroupFiles {
    io: FileIo,
    root: Option<Token>,
    raw: Queue<Submit>,
    events: Queue<Event>,
    names: List<u16>,
    groups: List<Group>,
    at: u32,
    phase: Phase,
    waiting: bool,
    notice: Option<Box<[u8]>>,
    answer: Option<Input>,
}
impl GroupFiles {
    pub fn new(groups: u32, enabled: bool) -> Self {
        Self {
            io: FileIo::new(2, 8, groups.max(1), Duration::from_secs(30)),
            root: None,
            raw: Queue::with_capacity(2),
            events: Queue::uncapped(),
            names: List::with_capacity(groups),
            groups: List::with_capacity(groups),
            at: 0,
            phase: if enabled { Phase::Open } else { Phase::Idle },
            waiting: false,
            notice: None,
            answer: None,
        }
    }
    pub fn groups(&self) -> &[Group] {
        self.groups.as_slice()
    }
    pub fn ready(&self) -> bool {
        (self.phase == Phase::Idle || self.phase == Phase::Closed) && self.answer.is_none() && self.notice.is_none()
    }
    pub fn pending(&self) -> bool {
        !self.raw.is_empty() || !self.events.is_empty() || !self.waiting && !self.ready()
    }
    pub fn has_notice(&self) -> bool {
        self.notice.is_some()
    }
    pub fn take_notice(&mut self) -> Option<Box<[u8]>> {
        self.notice.take()
    }
    pub fn take_answer(&mut self) -> Option<Input> {
        self.answer.take()
    }
    pub fn close_root(&mut self) {
        assert!(self.ready(), "files drain before root close");
        self.phase = Phase::RootClose;
    }
    pub fn closed(&self) -> bool {
        self.phase == Phase::Closed
    }
    pub fn write(&mut self, forge: u16, kind: BoxKind) {
        assert!(self.ready(), "one group file operation");
        self.phase = Phase::MakeBox { forge, kind };
    }
    pub fn unlink(&mut self, forge: u16) {
        assert!(self.ready(), "one group file operation");
        self.phase = Phase::UnlinkOpen { forge, started: false };
    }
    pub fn owns(op: Token) -> bool {
        op.raw() & (3_u64 << 62) == TAG
    }
    pub fn complete(&mut self, mut complete: Complete) {
        complete.op = Token::new(complete.op.raw() & !TAG);
        match &mut complete.kind {
            Op::Cancel { target } => *target = Token::new(target.raw() & !TAG),
            Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Open { .. }
            | Op::Close { .. }
            | Op::Read { .. }
            | Op::Write { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Lock { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::List { .. }
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. }
            | Op::Spawn { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::ReadSignal { .. }
            | Op::Usage
            | Op::Window { .. } => {}
        }
        file_layer::up(&mut self.io, complete, &mut self.events, &mut self.raw);
    }
    pub fn deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    pub fn tick(&mut self, root: Fd, now: Time, subs: &mut Queue<Submit>) {
        if self.root.is_none() {
            self.root = Some(self.io.adopt_root(root).expect("root slot"));
        }
        if self.io.is_due(now) && self.raw.room() > 0 {
            file_layer::expire(&mut self.io, now, &mut self.raw);
        }
        if self.notice.is_none() && self.answer.is_none() {
            if let Some(event) = self.events.pop() {
                self.waiting = false;
                self.event(event);
            }
            if !self.waiting && self.notice.is_none() && self.answer.is_none() && self.io.takes() && self.raw.room() > 0
            {
                let owner = OWNER;
                let token = self.root.expect("adopted root");
                let request = match self.phase {
                    Phase::Open => {
                        Request::OpenDirectory { owner, root, name: bytes::copy_of(b"box/groups"), no_follow: true }
                    }
                    Phase::List { dir } => {
                        Request::List { owner, root: self.io.descriptor(dir).expect("groups directory") }
                    }
                    Phase::ListClose { dir } | Phase::UnlinkClose { dir, .. } => Request::Close { owner, file: dir },
                    Phase::Load => {
                        let Some(forge) = self.names.get(self.at) else {
                            self.phase = Phase::Idle;
                            return;
                        };
                        Request::Load { owner, root: token, path: group_path(*forge), max: 8, no_follow: true }
                    }
                    Phase::Probe { forge, .. } => Request::OpenReadNoFollow { owner, root, name: path(forge) },
                    Phase::ProbeClose { file, .. } => Request::Close { owner, file },
                    Phase::Idle | Phase::Closed => return,
                    Phase::RootClose => Request::Close { owner, file: token },
                    Phase::MakeBox { .. } => {
                        Request::MakeDirectory { owner, root: token, path: bytes::copy_of(b"box"), mode: 0o700 }
                    }
                    Phase::MakeGroups { .. } => {
                        Request::MakeDirectory { owner, root: token, path: bytes::copy_of(b"box/groups"), mode: 0o700 }
                    }
                    Phase::Store { forge, kind } => {
                        self.io.seed_randomness(now.as_nanos().wrapping_add(u64::from(forge)));
                        Request::Store {
                            owner,
                            root: token,
                            path: group_path(forge),
                            bytes: group_file(kind),
                            expected: file::Expect::Any,
                            no_follow: true,
                        }
                    }
                    Phase::UnlinkOpen { started, .. } => Request::OpenDirectory {
                        owner,
                        root,
                        name: bytes::copy_of(if started { b"box/started" } else { b"box/groups" }),
                        no_follow: true,
                    },
                    Phase::Unlink { forge, dir, .. } => Request::Remove {
                        owner,
                        root: self.io.descriptor(dir).expect("unlink directory"),
                        name: bytes::copy_of(Decimal::of(u64::from(forge)).as_bytes()),
                        directory: false,
                    },
                };
                self.waiting = true;
                file_layer::down(&mut self.io, now, request, &mut self.events, &mut self.raw);
            }
        }
        self.publish(subs);
    }
    fn publish(&mut self, subs: &mut Queue<Submit>) {
        for _ in 0..self.raw.len() {
            if subs.room() == 0 {
                break;
            }
            let mut submit = self.raw.pop().expect("counted file operation");
            assert!(submit.op.raw() < TAG, "file operation namespace");
            submit.op = Token::new(submit.op.raw() | TAG);
            match &mut submit.kind {
                Op::Cancel { target } => *target = Token::new(target.raw() | TAG),
                Op::Socket { .. }
                | Op::Bind { .. }
                | Op::Listen { .. }
                | Op::Accept { .. }
                | Op::Connect { .. }
                | Op::Recv { .. }
                | Op::Send { .. }
                | Op::Shutdown { .. }
                | Op::Open { .. }
                | Op::Close { .. }
                | Op::Read { .. }
                | Op::Write { .. }
                | Op::Append { .. }
                | Op::Truncate { .. }
                | Op::Sync { .. }
                | Op::Lock { .. }
                | Op::Stat { .. }
                | Op::Rename { .. }
                | Op::Remove { .. }
                | Op::MakeDirectory { .. }
                | Op::List { .. }
                | Op::PipeRead { .. }
                | Op::PipeWrite { .. }
                | Op::Spawn { .. }
                | Op::Wait { .. }
                | Op::Signal { .. }
                | Op::ReadSignal { .. }
                | Op::Usage
                | Op::Window { .. } => {}
            }
            subs.push(submit);
        }
    }
    fn event(&mut self, event: Event) {
        match event {
            Event::Opened { file, .. } => match self.phase {
                Phase::Open => self.phase = Phase::List { dir: file },
                Phase::Probe { forge, kind } => self.phase = Phase::ProbeClose { forge, kind, file },
                Phase::UnlinkOpen { forge, started } => self.phase = Phase::Unlink { forge, started, dir: file },
                Phase::List { .. }
                | Phase::ListClose { .. }
                | Phase::Load
                | Phase::ProbeClose { .. }
                | Phase::Idle
                | Phase::RootClose
                | Phase::Closed
                | Phase::MakeBox { .. }
                | Phase::MakeGroups { .. }
                | Phase::Store { .. }
                | Phase::Unlink { .. }
                | Phase::UnlinkClose { .. } => unreachable!("an open owns its terminal"),
            },
            Event::Listed { entries, .. } => self.listed(entries),
            Event::Loaded { bytes, .. } => {
                let forge = *self.names.get(self.at).expect("loading group");
                match parse_group_file(&bytes) {
                    Some(kind) => self.phase = Phase::Probe { forge, kind },
                    None => {
                        self.notice = Some(joined(&[b"box: invalid group file ", &group_path(forge), b"\n"]));
                        self.next();
                    }
                }
            }
            Event::Closed { .. } => self.closed_event(),
            Event::Made { .. } => self.made(),
            Event::Stored { .. } => {
                let forge = match self.phase {
                    Phase::Store { forge, .. } => forge,
                    Phase::Open
                    | Phase::List { .. }
                    | Phase::ListClose { .. }
                    | Phase::Load
                    | Phase::Probe { .. }
                    | Phase::ProbeClose { .. }
                    | Phase::Idle
                    | Phase::RootClose
                    | Phase::Closed
                    | Phase::MakeBox { .. }
                    | Phase::MakeGroups { .. }
                    | Phase::UnlinkOpen { .. }
                    | Phase::Unlink { .. }
                    | Phase::UnlinkClose { .. } => unreachable!("storing group"),
                };
                self.phase = Phase::Idle;
                self.answer = Some(Input::Written { forge });
            }
            Event::Removed { .. } => {
                let (forge, started, dir) = match self.phase {
                    Phase::Unlink { forge, started, dir } => (forge, started, dir),
                    Phase::Open
                    | Phase::List { .. }
                    | Phase::ListClose { .. }
                    | Phase::Load
                    | Phase::Probe { .. }
                    | Phase::ProbeClose { .. }
                    | Phase::Idle
                    | Phase::RootClose
                    | Phase::Closed
                    | Phase::MakeBox { .. }
                    | Phase::MakeGroups { .. }
                    | Phase::Store { .. }
                    | Phase::UnlinkOpen { .. }
                    | Phase::UnlinkClose { .. } => unreachable!("unlinking group"),
                };
                self.phase = Phase::UnlinkClose { forge, started, dir, error: None };
            }
            Event::Failed { error, .. } => self.error(error),
            Event::TooLarge { .. } | Event::Refused { .. } | Event::Conflict { .. } | Event::Cancelled { .. } => {
                self.error(Error::InvalidArgument);
            }
            Event::Locked { .. }
            | Event::Busy { .. }
            | Event::Scanned { .. }
            | Event::Stated { .. }
            | Event::Written { .. }
            | Event::Read { .. }
            | Event::Cut { .. }
            | Event::Synced { .. }
            | Event::Renamed { .. } => unreachable!("only group file requests"),
        }
    }
    fn listed(&mut self, entries: Box<[file::Entry]>) {
        let dir = match self.phase {
            Phase::List { dir } => dir,
            Phase::Open
            | Phase::ListClose { .. }
            | Phase::Load
            | Phase::Probe { .. }
            | Phase::ProbeClose { .. }
            | Phase::Idle
            | Phase::RootClose
            | Phase::Closed
            | Phase::MakeBox { .. }
            | Phase::MakeGroups { .. }
            | Phase::Store { .. }
            | Phase::UnlinkOpen { .. }
            | Phase::Unlink { .. }
            | Phase::UnlinkClose { .. } => unreachable!("listing groups"),
        };
        if entries.len() > usize::try_from(self.names.capacity()).expect("group bound fits") {
            self.notice = Some(bytes::copy_of(b"box: box/groups exceeds box.groups\n"));
            self.phase = Phase::ListClose { dir };
            return;
        }
        for entry in entries {
            if let Some(forge) = pipe::number(&entry.name)
                && self.names.room() > 0
            {
                self.names.push(forge).expect("group file bound");
            }
        }
        self.phase = Phase::ListClose { dir };
    }
    fn closed_event(&mut self) {
        match self.phase {
            Phase::RootClose => self.phase = Phase::Closed,
            Phase::ListClose { .. } => self.phase = Phase::Load,
            Phase::ProbeClose { forge, kind, .. } => {
                self.remember(forge, kind, true);
                self.next();
            }
            Phase::UnlinkClose { forge, started, error, .. } => match error {
                Some(error) => self.failed(forge, error, true),
                None => self.unlinked(forge, started),
            },
            Phase::Open
            | Phase::List { .. }
            | Phase::Load
            | Phase::Probe { .. }
            | Phase::Idle
            | Phase::Closed
            | Phase::MakeBox { .. }
            | Phase::MakeGroups { .. }
            | Phase::Store { .. }
            | Phase::UnlinkOpen { .. }
            | Phase::Unlink { .. } => unreachable!("a close owns its terminal"),
        }
    }
    fn made(&mut self) {
        match self.phase {
            Phase::MakeBox { forge, kind } => self.phase = Phase::MakeGroups { forge, kind },
            Phase::MakeGroups { forge, kind } => self.phase = Phase::Store { forge, kind },
            Phase::Open
            | Phase::List { .. }
            | Phase::ListClose { .. }
            | Phase::Load
            | Phase::Probe { .. }
            | Phase::ProbeClose { .. }
            | Phase::Idle
            | Phase::RootClose
            | Phase::Closed
            | Phase::Store { .. }
            | Phase::UnlinkOpen { .. }
            | Phase::Unlink { .. }
            | Phase::UnlinkClose { .. } => unreachable!("making group directories"),
        }
    }
    fn error(&mut self, error: Error) {
        match self.phase {
            Phase::Open if error == Error::NotFound => self.phase = Phase::Idle,
            Phase::Open | Phase::List { .. } | Phase::ListClose { .. } => {
                self.notice = Some(bytes::copy_of(b"box: cannot list box/groups (box.groups bounds the entries)\n"));
                // List owns an opened directory; release it even on a failed listing.
                match self.phase {
                    Phase::List { dir } => self.phase = Phase::ListClose { dir },
                    Phase::Open | Phase::ListClose { .. } => self.phase = Phase::Idle,
                    Phase::Load
                    | Phase::Probe { .. }
                    | Phase::ProbeClose { .. }
                    | Phase::Idle
                    | Phase::RootClose
                    | Phase::Closed
                    | Phase::MakeBox { .. }
                    | Phase::MakeGroups { .. }
                    | Phase::Store { .. }
                    | Phase::UnlinkOpen { .. }
                    | Phase::Unlink { .. }
                    | Phase::UnlinkClose { .. } => unreachable!("listing phase"),
                }
            }
            Phase::Load => {
                self.notice = Some(bytes::copy_of(b"box: cannot read group file\n"));
                self.next();
            }
            Phase::Probe { forge, kind } => {
                self.remember(forge, kind, false);
                if error != Error::NotFound {
                    self.notice = Some(bytes::copy_of(b"box: cannot probe group started record\n"));
                }
                self.next();
            }
            Phase::ProbeClose { .. } => {
                self.notice = Some(bytes::copy_of(b"box: cannot close group started record\n"));
                self.next();
            }
            Phase::MakeBox { .. } | Phase::MakeGroups { .. } if error == Error::Exists => self.made(),
            Phase::MakeBox { forge, .. } | Phase::MakeGroups { forge, .. } | Phase::Store { forge, .. } => {
                self.failed(forge, error, false);
            }
            Phase::UnlinkOpen { forge, started } if error == Error::NotFound => self.unlinked(forge, started),
            Phase::UnlinkOpen { forge, .. } | Phase::UnlinkClose { forge, .. } => self.failed(forge, error, true),
            Phase::Unlink { forge, started, dir } => {
                self.phase = Phase::UnlinkClose {
                    forge,
                    started,
                    dir,
                    error: if error == Error::NotFound { None } else { Some(error) },
                }
            }
            Phase::RootClose => self.phase = Phase::Closed,
            Phase::Idle | Phase::Closed => unreachable!("no idle file operation"),
        }
    }
    fn remember(&mut self, forge: u16, kind: BoxKind, mut started: bool) {
        if started {
            for index in 0..self.groups.len() {
                let previous = self.groups.get_mut(index).expect("group index");
                if previous.kind == kind && previous.started {
                    let loser = if previous.forge > forge {
                        started = false;
                        forge
                    } else {
                        previous.started = false;
                        previous.forge
                    };
                    self.notice = Some(joined(&[
                        b"box: forge ",
                        Decimal::of(u64::from(loser)).as_bytes(),
                        b": a second ",
                        kind.word(),
                        b" group is not started\n",
                    ]));
                }
            }
        }
        self.groups.push(Group { forge, kind, started }).expect("bounded group files");
    }
    fn next(&mut self) {
        self.at = self.at.checked_add(1).expect("bounded group files");
        self.phase = Phase::Load;
    }
    fn unlinked(&mut self, forge: u16, started: bool) {
        if started {
            self.phase = Phase::Idle;
            self.answer = Some(Input::Removed { forge });
        } else {
            self.phase = Phase::UnlinkOpen { forge, started: true };
        }
    }
    fn failed(&mut self, forge: u16, _error: Error, removing: bool) {
        self.notice = Some(joined(&[
            b"box: forge ",
            Decimal::of(u64::from(forge)).as_bytes(),
            if removing { b": files not removed\n" } else { b": group file not written\n" },
        ]));
        self.phase = Phase::Idle;
        self.answer = Some(if removing {
            Input::RemoveFailed { forge, why: Why::NotUnlinked }
        } else {
            Input::NotWritten { forge }
        });
    }
}
