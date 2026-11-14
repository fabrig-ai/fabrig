//! Bounded file operations over kernel records (runtime/io/files.md).
//!
//! `FileIo` keeps open file and root descriptors behind tokens, one request
//! in flight, its deadline, and bounded bytes or entries while a whole-file
//! operation progresses. It does not know a workspace's writable policy or
//! interpret a file's content or version. `step::down` and `step::down_until`
//! accept one request while `takes` is true; `step::up` handles one completion;
//! `step::expire` settles a deadline.
//! Each admitted request produces one terminal event after owned descriptors
//! are closed. Loads refuse excess bytes; scans retain a name-order prefix
//! within their entry and byte caps and count all omitted entries. Private
//! roots keep their verified effective user beside the descriptor; loads
//! check metadata before reading and stores make temporaries mode 0600 (5.3).

#![expect(clippy::disallowed_types, reason = "read and directory buffers are bounded by FileIo's limits")]
#![expect(clippy::disallowed_macros, reason = "fixed and limit-sized kernel buffers need initialized storage")]
#![expect(clippy::large_enum_variant, reason = "FileIo holds one pending store without a separate allocation")]

use crate::file::{Entry, Event, Request};
use crate::kernel::{Complete, Done, Error, Fd, Op, OpenHow, Submit};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use skein_lib::{Duration, Env, Map, Queue, Time, Token, usize_of};

mod directory;
mod opens;
mod private;
pub mod step;
mod store;

/// The file-operation lane, below a service's namespace tags (runtime/io/files.md; runtime/lib.md, 4).
pub const LANE: u64 = 1 << 59;

/// Whether an operation belongs to `FileIo` rather than `Io` (runtime/io/files.md).
#[must_use]
pub const fn owns(op: Token) -> bool {
    op.raw() & LANE != 0
}

// runtime/io/files.md: a file's contents print as their length, never their bytes.
struct Contents(Vec<u8>);

impl core::fmt::Debug for Contents {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Contents").field("len", &self.0.len()).finish()
    }
}

#[derive(Debug)]
enum Pending {
    Lock(Token),
    Create(Token),
    OpenRead(Token),
    OpenDirectory(Token),
    Stat {
        owner: Token,
        fd: Fd,
    },
    Stating(Token),
    Cleanup {
        owner: Token,
        error: Error,
    },
    Write(Token),
    Read {
        owner: Token,
        fd: Fd,
        offset: u64,
        max: u32,
        bytes: Contents,
    },
    Cut(Token),
    Sync(Token),
    Close(Token),
    Rename(Token),
    HardLink(Token),
    Remove(Token),
    ListOpening(Token),
    List {
        owner: Token,
        fd: Fd,
        entries: Vec<Entry>,
    },
    Page {
        owner: Token,
        fd: Fd,
        max: u32,
        entries: Vec<Entry>,
    },
    ListClosing {
        owner: Token,
        entries: Vec<Entry>,
    },
    LoadOpening {
        owner: Token,
        max: u32,
        loading: Option<Loading>,
    },
    LoadReopening {
        owner: Token,
        max: u32,
        loading: Loading,
    },
    LoadStating {
        owner: Token,
        fd: Fd,
        max: u32,
        loading: Option<Loading>,
    },
    LoadReading {
        owner: Token,
        fd: Fd,
        max: u32,
        bytes: Contents,
    },
    LoadClosing {
        owner: Token,
        bytes: Contents,
    },
    LoadTooLargeClosing {
        owner: Token,
        size: u64,
    },
    ScanOpening {
        owner: Token,
        max: u32,
        max_bytes: u64,
    },
    ScanListing {
        owner: Token,
        fd: Fd,
        max: u32,
        max_bytes: u64,
        entries: Vec<Entry>,
        bytes: u64,
        cutoff: Option<Box<[u8]>>,
        total: u64,
    },
    ScanClosing {
        owner: Token,
        entries: Vec<Entry>,
        more: u64,
    },
    Store(store::Store),
    Private(private::Opening),
}

#[derive(Clone, Copy, Debug)]
enum Stop {
    Running,
    Deadline,
    Cancel,
}

// programming-model.md, 5.4: request and cancellation have separate lifecycles.
#[derive(Debug)]
enum Work {
    Idle,
    Active { pending: Pending, op: Token, deadline: Option<Time>, stop: Stop },
}

#[derive(Debug)]
enum Cancelling {
    Quiet,
    Asked { op: Token, target: Token },
}

impl Pending {
    fn owner(&self) -> Token {
        match self {
            Pending::Create(owner)
            | Pending::OpenRead(owner)
            | Pending::OpenDirectory(owner)
            | Pending::Stat { owner, .. }
            | Pending::Stating(owner)
            | Pending::Cleanup { owner, .. }
            | Pending::Write(owner)
            | Pending::Read { owner, .. }
            | Pending::Cut(owner)
            | Pending::Lock(owner)
            | Pending::Sync(owner)
            | Pending::Close(owner)
            | Pending::Rename(owner)
            | Pending::HardLink(owner)
            | Pending::Remove(owner)
            | Pending::ListOpening(owner)
            | Pending::List { owner, .. }
            | Pending::Page { owner, .. }
            | Pending::ListClosing { owner, .. }
            | Pending::LoadOpening { owner, .. }
            | Pending::LoadReopening { owner, .. }
            | Pending::LoadStating { owner, .. }
            | Pending::LoadReading { owner, .. }
            | Pending::LoadClosing { owner, .. }
            | Pending::LoadTooLargeClosing { owner, .. }
            | Pending::ScanOpening { owner, .. }
            | Pending::ScanListing { owner, .. }
            | Pending::ScanClosing { owner, .. } => *owner,
            Pending::Store(store) => store.owner(),
            Pending::Private(opening) => opening.owner(),
        }
    }

    fn can_cancel(&self) -> bool {
        match self {
            Pending::Create(_)
            | Pending::OpenRead(_)
            | Pending::OpenDirectory(_)
            | Pending::Write(_)
            | Pending::Read { .. }
            | Pending::Sync(_)
            | Pending::ListOpening(_)
            | Pending::LoadOpening { .. }
            | Pending::LoadReading { .. }
            | Pending::ScanOpening { .. } => true,
            Pending::Lock(_)
            | Pending::Cut(_)
            | Pending::Stat { .. }
            | Pending::Stating(_)
            | Pending::Cleanup { .. }
            | Pending::Close(_)
            | Pending::Rename(_)
            | Pending::HardLink(_)
            | Pending::Remove(_)
            | Pending::List { .. }
            | Pending::Page { .. }
            | Pending::ListClosing { .. }
            | Pending::LoadReopening { .. }
            | Pending::LoadStating { .. }
            | Pending::LoadClosing { .. }
            | Pending::LoadTooLargeClosing { .. }
            | Pending::ScanListing { .. }
            | Pending::ScanClosing { .. } => false,
            Pending::Store(store) => store.can_cancel(),
            Pending::Private(opening) => opening.can_cancel(),
        }
    }
}

#[derive(Debug)]
struct Loading {
    user: u32,
    root: Fd,
    path: Box<[u8]>,
    reopened: bool,
}

/// An owned descriptor and, for verified private roots, its effective user.
#[derive(Debug)]
struct Opened {
    fd: Fd,
    private_owner: Option<u32>,
}

/// File admission and transfer bounds (skein/docs/design/runtime/io/files.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub files: u32,
    pub read: u32,
    pub entries: u32,
    pub whole: u32,
    pub timeout: Duration,
}

/// Whether every file limit has room (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.files > 0 && limits.read > 0 && limits.entries > 0 && limits.whole > 0 && limits.timeout.as_nanos() > 0
}

/// A bounded file driver with one terminal per owner request (runtime/io/files.md).
#[derive(Debug)]
pub struct FileIo {
    files: Map<Token, Opened>,
    effective_user: Option<u32>,
    next_file: u64,
    next_op: u64,
    work: Work,
    cancelling: Cancelling,
    random: Option<u64>,
}

impl FileIo {
    #[must_use]
    pub fn with_limits(limits: &Limits) -> FileIo {
        assert!(valid(limits), "file limits need room");
        FileIo {
            files: Map::with_capacity(limits.files),
            effective_user: None,
            next_file: 1,
            next_op: 1,
            work: Work::Idle,
            cancelling: Cancelling::Quiet,
            random: None,
        }
    }

    /// Configures the shell's effective user read once at startup (runtime/io/files.md, section 3).
    #[must_use]
    pub fn with_effective_user(mut self, effective_user: u32) -> FileIo {
        assert!(self.takes() && self.files.is_empty(), "configure the user before adopting roots");
        self.effective_user = Some(effective_user);
        self
    }

    #[must_use]
    pub const fn takes(&self) -> bool {
        match self.work {
            Work::Idle => match self.cancelling {
                Cancelling::Quiet => true,
                Cancelling::Asked { .. } => false,
            },
            Work::Active { .. } => false,
        }
    }

    #[must_use]
    pub fn open_files(&self) -> u32 {
        self.files.len()
    }

    /// Adopts a directory descriptor opened by the shell as an io-owned root.
    /// The returned token is closed with `Request::Close`.
    pub fn adopt_root(&mut self, root: Fd) -> Option<Token> {
        self.insert(root)
    }

    /// Resolves an open file or root token within io.
    #[must_use]
    pub fn descriptor(&self, file: Token) -> Option<Fd> {
        self.file(file)
    }

    /// Supplies the random seed used to choose temporary names for stores.
    pub fn seed_randomness(&mut self, seed: u64) {
        assert!(self.takes(), "seed randomness between file requests");
        self.random = Some(seed);
    }

    #[must_use]
    pub const fn next_deadline(&self) -> Option<Time> {
        match self.work {
            Work::Idle => None,
            Work::Active { deadline, .. } => deadline,
        }
    }

    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        }
    }

    fn mint_op(&mut self) -> Token {
        assert!(self.next_op < LANE, "operation tokens stay below the file lane");
        let op = Token::new(LANE | self.next_op);
        self.next_op = self.next_op.checked_add(1).expect("operation tokens do not wrap");
        op
    }

    fn file(&self, token: Token) -> Option<Fd> {
        Some(self.files.get(&token)?.fd)
    }

    fn private_owner(&self, token: Token) -> Option<u32> {
        self.files.get(&token)?.private_owner
    }

    fn insert(&mut self, fd: Fd) -> Option<Token> {
        self.insert_opened(Opened { fd, private_owner: None })
    }

    fn insert_private(&mut self, fd: Fd, user: u32) -> Option<Token> {
        self.insert_opened(Opened { fd, private_owner: Some(user) })
    }

    fn insert_opened(&mut self, opened: Opened) -> Option<Token> {
        let token = Token::new(self.next_file);
        self.next_file = self.next_file.checked_add(1).expect("file tokens do not wrap");
        self.files.insert(token, opened).ok()?;
        Some(token)
    }
}

fn issue(io: &mut FileIo, pending: Pending, kind: Op, subs: &mut Queue<Submit>) -> Work {
    assert!(kind.is_valid(), "file request makes a valid kernel record");
    let op = io.mint_op();
    subs.push(Submit { op, kind });
    Work::Active { pending, op, deadline: None, stop: Stop::Running }
}

fn list(io: &mut FileIo, pending: Pending, fd: Fd, subs: &mut Queue<Submit>) -> Work {
    issue(
        io,
        pending,
        Op::List {
            fd,
            entries: vec![crate::kernel::Entry::BLANK; 32].into_boxed_slice(),
            names: vec![0_u8; 8192].into_boxed_slice(),
        },
        subs,
    )
}

#[expect(clippy::too_many_lines, reason = "one exhaustive request match keeps file admission in one place")]
fn down_inner(
    io: &mut FileIo,
    env: &Env<Limits>,
    request: Request,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    match request {
        Request::Lock { owner, root } => directory::lock(io, owner, root, events, subs),
        Request::MakeDirectory { owner, root, path, mode } => {
            private::start(io, owner, root, path, private::Action::Make { mode }, events, subs)
        }
        Request::OpenPrivate { owner, root, path } => match io.effective_user {
            Some(user) => private::start(io, owner, root, path, private::Action::Private { user }, events, subs),
            None => terminal_failure(events, owner, Error::InvalidArgument),
        },
        Request::Create { owner, root, name, mode } => create(io, owner, root, name, mode, false, events, subs),
        Request::CreateNoFollow { owner, root, name, mode } => create(io, owner, root, name, mode, true, events, subs),
        Request::OpenRead { owner, root, name } => opens::open_read(io, owner, root, name, OpenHow::Read, events, subs),
        Request::OpenReadNoFollow { owner, root, name } => {
            opens::open_read(io, owner, root, name, OpenHow::ReadNoFollow, events, subs)
        }
        Request::OpenWrite { owner, root, name } => {
            opens::open_read(io, owner, root, name, OpenHow::Write, events, subs)
        }
        Request::OpenWriteNoFollow { owner, root, name } => {
            opens::open_read(io, owner, root, name, OpenHow::WriteNoFollow, events, subs)
        }
        Request::Cut { owner, file, length } => opens::cut(io, owner, file, length, events, subs),
        Request::OpenDirectory { owner, root, name, no_follow } => {
            if name.contains(&0) {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            if io.files.len() == io.files.capacity() {
                events.push(Event::failed(owner, Error::TooManyOpenFiles));
                return Work::Idle;
            }
            let how = if no_follow { OpenHow::DirectoryNoFollow } else { OpenHow::Directory };
            issue(io, Pending::OpenDirectory(owner), Op::Open { root, path: name, how }, subs)
        }
        Request::Load { owner, root, path, max, no_follow } => {
            let Some(fd) = io.file(root) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            if max > env.limits.whole || path.contains(&0) || path.len() >= 4096 {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            #[expect(clippy::manual_map, reason = "programming-model.md, 5.4: explicitly match private load metadata")]
            let loading = match io.private_owner(root) {
                Some(user) => Some(Loading { user, root: fd, path: path.clone(), reopened: false }),
                None => None,
            };
            let how = if no_follow || loading.is_some() { OpenHow::ReadNoFollow } else { OpenHow::Read };
            issue(io, Pending::LoadOpening { owner, max, loading }, Op::Open { root: fd, path, how }, subs)
        }
        Request::Scan { owner, root, path, max, max_bytes, no_follow } => {
            let Some(fd) = io.file(root) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            if max > env.limits.entries || path.contains(&0) || path.len() >= 4096 {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            let how = if no_follow { OpenHow::DirectoryNoFollow } else { OpenHow::Directory };
            issue(io, Pending::ScanOpening { owner, max, max_bytes }, Op::Open { root: fd, path, how }, subs)
        }
        Request::Store { owner, root, path, bytes, expected, no_follow } => {
            store::start(io, env, owner, root, path, bytes, expected, no_follow, events, subs)
        }
        Request::Stat { owner, file } => {
            let Some(fd) = io.file(file) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            issue(io, Pending::Stating(owner), Op::Stat { fd }, subs)
        }
        Request::WriteAt { owner, file, offset, bytes } => {
            let Some(fd) = io.file(file) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            if bytes.is_empty() {
                events.push(Event::Written { owner });
                return Work::Idle;
            }
            let Ok(op) = Op::write(fd, bytes, 0, offset) else {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            };
            issue(io, Pending::Write(owner), op, subs)
        }
        Request::ReadAt { owner, file, offset, max } => {
            let Some(fd) = io.file(file) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            if max == 0 || max > env.limits.read {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            let buffer = vec![0_u8; usize_of(max)].into_boxed_slice();
            let Ok(op) = Op::read(fd, buffer, offset) else {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            };
            issue(io, Pending::Read { owner, fd, offset, max, bytes: Contents(Vec::new()) }, op, subs)
        }
        Request::Sync { owner, file } => {
            let Some(fd) = io.file(file) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            issue(io, Pending::Sync(owner), Op::Sync { fd }, subs)
        }
        Request::Close { owner, file } => {
            let Some(opened) = io.files.remove(&file) else {
                events.push(Event::failed(owner, Error::NotFound));
                return Work::Idle;
            };
            issue(io, Pending::Close(owner), Op::Close { fd: opened.fd }, subs)
        }
        Request::SyncDirectory { owner, root } => issue(io, Pending::Sync(owner), Op::Sync { fd: root }, subs),
        Request::Rename { owner, from_root, from, to_root, to } => {
            if !crate::kernel::is_name(&from) || !crate::kernel::is_name(&to) {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            issue(
                io,
                Pending::Rename(owner),
                Op::Rename { from_dir: from_root, from, to_dir: to_root, to, no_replace: false },
                subs,
            )
        }
        Request::HardLink { owner, from_root, from, to_root, to } => {
            if !crate::kernel::is_name(&from)
                || !crate::kernel::is_name(&to)
                || from.len() > crate::kernel::LONGEST_NAME
                || to.len() > crate::kernel::LONGEST_NAME
            {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            issue(io, Pending::HardLink(owner), Op::HardLink { from_dir: from_root, from, to_dir: to_root, to }, subs)
        }
        Request::Remove { owner, root, name, directory } => {
            if !crate::kernel::is_name(&name) {
                events.push(Event::failed(owner, Error::InvalidArgument));
                return Work::Idle;
            }
            issue(io, Pending::Remove(owner), Op::Remove { dir: root, name, directory }, subs)
        }
        Request::ListPage { owner, directory, max } => directory::page(io, env, owner, directory, max, events, subs),
        Request::List { owner, root } => issue(
            io,
            Pending::ListOpening(owner),
            Op::Open { root, path: Box::from(&b"."[..]), how: OpenHow::Directory },
            subs,
        ),
    }
}

#[expect(clippy::too_many_arguments, reason = "the file admission inputs and two output queues are explicit")]
fn create(
    io: &mut FileIo,
    owner: Token,
    root: Fd,
    name: Box<[u8]>,
    mode: u32,
    no_follow: bool,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    if name.contains(&0) || mode & !crate::kernel::PERMISSIONS != 0 {
        events.push(Event::failed(owner, Error::InvalidArgument));
        return Work::Idle;
    }
    if io.files.len() == io.files.capacity() {
        events.push(Event::failed(owner, Error::TooManyOpenFiles));
        return Work::Idle;
    }
    let how =
        if no_follow { OpenHow::CreateNoFollow { mode: Some(mode) } } else { OpenHow::Create { mode: Some(mode) } };
    issue(io, Pending::Create(owner), Op::Open { root, path: name, how }, subs)
}

fn read_whole(
    io: &mut FileIo,
    env: &Env<Limits>,
    owner: Token,
    fd: Fd,
    max: u32,
    bytes: Contents,
    subs: &mut Queue<Submit>,
) -> Work {
    let used = u32::try_from(bytes.0.len()).expect("whole load stays within its u32 bound");
    let left = max.checked_sub(used).expect("whole load stays within its bound");
    let asked = env.limits.read.min(left.saturating_add(1));
    let buffer = vec![0_u8; usize_of(asked)].into_boxed_slice();
    let offset = u64::from(used);
    let op = Op::read(fd, buffer, offset).expect("a positive bounded whole-file read");
    issue(io, Pending::LoadReading { owner, fd, max, bytes }, op, subs)
}

/// Requests cancellation of the active owner's file operation. The terminal
/// reports `Cancelled` only when the request was stopped before completion.
pub fn cancel(io: &mut FileIo, owner: Token, subs: &mut Queue<Submit>) {
    let work = mem::replace(&mut io.work, Work::Idle);
    io.work = match work {
        Work::Active { pending, op, deadline, stop: Stop::Running } if pending.owner() == owner => {
            let can_abandon = match &pending {
                Pending::Store(store) => store.can_abandon(),
                Pending::Private(opening) => opening.can_abandon(),
                Pending::Cleanup { .. } => false,
                Pending::Lock(_)
                | Pending::Create(_)
                | Pending::OpenRead(_)
                | Pending::OpenDirectory(_)
                | Pending::Stat { .. }
                | Pending::Stating(_)
                | Pending::Write(_)
                | Pending::Read { .. }
                | Pending::Cut(_)
                | Pending::Sync(_)
                | Pending::Close(_)
                | Pending::Rename(_)
                | Pending::HardLink(_)
                | Pending::Remove(_)
                | Pending::ListOpening(_)
                | Pending::List { .. }
                | Pending::Page { .. }
                | Pending::ListClosing { .. }
                | Pending::LoadOpening { .. }
                | Pending::LoadReopening { .. }
                | Pending::LoadStating { .. }
                | Pending::LoadReading { .. }
                | Pending::LoadClosing { .. }
                | Pending::LoadTooLargeClosing { .. }
                | Pending::ScanOpening { .. }
                | Pending::ScanListing { .. }
                | Pending::ScanClosing { .. } => true,
            };
            let stop = if can_abandon { Stop::Cancel } else { Stop::Running };
            Work::Active { pending, op, deadline, stop }
        }
        other @ (Work::Idle | Work::Active { .. }) => {
            io.work = other;
            return;
        }
    };
    follow_cancel(io, subs);
}

fn follow_cancel(io: &mut FileIo, subs: &mut Queue<Submit>) {
    match io.cancelling {
        Cancelling::Asked { .. } => return,
        Cancelling::Quiet => {}
    }
    match &io.work {
        Work::Active { pending, op, stop: Stop::Deadline | Stop::Cancel, .. } if pending.can_cancel() => {
            io.cancelling = submit_cancel(io, *op, subs);
        }
        Work::Idle | Work::Active { .. } => {}
    }
}

fn submit_cancel(io: &mut FileIo, target: Token, subs: &mut Queue<Submit>) -> Cancelling {
    let op = io.mint_op();
    subs.push(Submit { op, kind: Op::Cancel { target } });
    Cancelling::Asked { op, target }
}

fn follow_work(io: &mut FileIo, deadline: Option<Time>) {
    match &mut io.work {
        Work::Idle => {}
        Work::Active { pending, deadline: next, .. } => {
            *next = match pending {
                Pending::Lock(_) => None,
                Pending::Store(store) => store.deadline(deadline),
                Pending::Create(_)
                | Pending::OpenRead(_)
                | Pending::OpenDirectory(_)
                | Pending::Stat { .. }
                | Pending::Stating(_)
                | Pending::Cleanup { .. }
                | Pending::Write(_)
                | Pending::Read { .. }
                | Pending::Cut(_)
                | Pending::Sync(_)
                | Pending::Close(_)
                | Pending::Rename(_)
                | Pending::HardLink(_)
                | Pending::Remove(_)
                | Pending::ListOpening(_)
                | Pending::List { .. }
                | Pending::Page { .. }
                | Pending::ListClosing { .. }
                | Pending::LoadOpening { .. }
                | Pending::LoadReopening { .. }
                | Pending::LoadStating { .. }
                | Pending::LoadReading { .. }
                | Pending::LoadClosing { .. }
                | Pending::LoadTooLargeClosing { .. }
                | Pending::ScanOpening { .. }
                | Pending::ScanListing { .. }
                | Pending::ScanClosing { .. }
                | Pending::Private(_) => deadline,
            };
        }
    }
}

fn cancelled(
    io: &mut FileIo,
    env: &Env<Limits>,
    pending: Pending,
    complete: Complete,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let can_abandon = match &pending {
        Pending::Store(store) => store.can_abandon(),
        Pending::Private(opening) => opening.can_abandon(),
        Pending::OpenRead(_)
        | Pending::Stat { .. }
        | Pending::ListOpening(_)
        | Pending::List { .. }
        | Pending::Page { .. }
        | Pending::LoadOpening { .. }
        | Pending::LoadReopening { .. }
        | Pending::LoadStating { .. }
        | Pending::LoadReading { .. }
        | Pending::ScanOpening { .. }
        | Pending::ScanListing { .. } => true,
        Pending::Write(_) => !write_finished(&complete),
        Pending::Read { max, bytes, .. } => !read_finished(&complete, *max, bytes.0.len()),
        Pending::Create(_)
        | Pending::OpenDirectory(_)
        | Pending::Stating(_)
        | Pending::Cleanup { .. }
        | Pending::Cut(_)
        | Pending::Lock(_)
        | Pending::Sync(_)
        | Pending::Close(_)
        | Pending::Rename(_)
        | Pending::HardLink(_)
        | Pending::Remove(_)
        | Pending::ListClosing { .. }
        | Pending::LoadClosing { .. }
        | Pending::LoadTooLargeClosing { .. }
        | Pending::ScanClosing { .. } => false,
    };
    let stop_won = match &complete.result {
        Err(Error::Cancelled) => match &pending {
            Pending::Store(_) | Pending::Private(_) => can_abandon,
            Pending::Create(_)
            | Pending::OpenRead(_)
            | Pending::OpenDirectory(_)
            | Pending::Stat { .. }
            | Pending::Stating(_)
            | Pending::Cleanup { .. }
            | Pending::Write(_)
            | Pending::Read { .. }
            | Pending::Cut(_)
            | Pending::Lock(_)
            | Pending::Sync(_)
            | Pending::Close(_)
            | Pending::Rename(_)
            | Pending::HardLink(_)
            | Pending::Remove(_)
            | Pending::ListOpening(_)
            | Pending::List { .. }
            | Pending::Page { .. }
            | Pending::ListClosing { .. }
            | Pending::LoadOpening { .. }
            | Pending::LoadReopening { .. }
            | Pending::LoadStating { .. }
            | Pending::LoadReading { .. }
            | Pending::LoadClosing { .. }
            | Pending::LoadTooLargeClosing { .. }
            | Pending::ScanOpening { .. }
            | Pending::ScanListing { .. }
            | Pending::ScanClosing { .. } => true,
        },
        Ok(_) | Err(_) => can_abandon,
    };
    if stop_won {
        stopped(io, env, pending, complete, Error::Cancelled, events, subs)
    } else {
        up_inner(io, env, pending, complete, events, subs)
    }
}

fn write_finished(complete: &Complete) -> bool {
    match &complete.kind {
        Op::Write { bytes, from, .. } => match complete.result {
            Ok(Done::Count(count)) => {
                u64::from(*from).saturating_add(u64::from(count)) >= skein_lib::len_u64(bytes.len())
            }
            Ok(_) | Err(_) => false,
        },
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
        | Op::Rename { .. }
        | Op::HardLink { .. }
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
        | Op::Cancel { .. } => false,
    }
}

fn read_finished(complete: &Complete, max: u32, prior: usize) -> bool {
    match complete.result {
        Ok(Done::Count(count)) => {
            count == 0
                || u64::try_from(prior).expect("bytes fit u64").saturating_add(u64::from(count)) >= u64::from(max)
        }
        Ok(_) | Err(_) => false,
    }
}

fn stopped(
    io: &mut FileIo,
    env: &Env<Limits>,
    pending: Pending,
    complete: Complete,
    reason: Error,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    let owner = pending.owner();
    let fd = match pending {
        Pending::Private(opening) => {
            return private::stopped(io, opening, complete, reason, events, subs);
        }
        Pending::Store(store) => {
            return store::stopped(io, store, complete, reason, events, subs);
        }
        Pending::Lock(owner) => {
            return directory::locked(owner, complete, events);
        }
        direct @ (Pending::Stating(_) | Pending::Rename(_) | Pending::HardLink(_) | Pending::Remove(_)) => {
            return up_inner(io, env, direct, complete, events, subs);
        }
        page @ Pending::Page { .. } => {
            return directory::answered(io, page, complete, Some(reason), events, subs);
        }

        Pending::Create(_)
        | Pending::OpenRead(_)
        | Pending::OpenDirectory(_)
        | Pending::ListOpening(_)
        | Pending::LoadOpening { .. }
        | Pending::ScanOpening { .. } => match complete.result {
            Ok(Done::Fd(fd)) => Some(fd),
            Ok(
                Done::Nothing
                | Done::Count(_)
                | Done::Accepted { .. }
                | Done::Bound(_)
                | Done::Stat(_)
                | Done::Spawned { .. }
                | Done::Exit(_)
                | Done::Usage(_)
                | Done::Signalled(_)
                | Done::Window(_),
            )
            | Err(_) => None,
        },
        Pending::LoadStating { fd, .. }
        | Pending::LoadReading { fd, .. }
        | Pending::ScanListing { fd, .. }
        | Pending::Stat { fd, .. }
        | Pending::List { fd, .. } => Some(fd),
        Pending::Cleanup { .. }
        | Pending::Write(_)
        | Pending::Read { .. }
        | Pending::Cut(_)
        | Pending::Sync(_)
        | Pending::Close(_)
        | Pending::ListClosing { .. }
        | Pending::LoadReopening { .. }
        | Pending::LoadClosing { .. }
        | Pending::LoadTooLargeClosing { .. }
        | Pending::ScanClosing { .. } => None,
    };
    if let Some(fd) = fd {
        issue(io, Pending::Cleanup { owner, error: reason }, Op::Close { fd }, subs)
    } else {
        terminal_failure(events, owner, reason)
    }
}

fn terminal_failure(events: &mut Queue<Event>, owner: Token, reason: Error) -> Work {
    if reason == Error::Cancelled {
        events.push(Event::Cancelled { owner });
    } else {
        events.push(Event::failed(owner, reason));
    }
    Work::Idle
}

/// Retains a name-order prefix under both scan bounds. `cutoff` is the
/// smallest omitted name seen so far; a later entry cannot cross it.
fn retain_scan_entry(
    entries: &mut Vec<Entry>,
    bytes: &mut u64,
    cutoff: &mut Option<Box<[u8]>>,
    max: u32,
    max_bytes: u64,
    name: &[u8],
    kind: crate::kernel::Kind,
) {
    match cutoff {
        Some(first_omitted) if name >= first_omitted.as_ref() => return,
        Some(_) | None => {}
    }
    let mut at = 0_usize;
    for entry in entries.iter() {
        if entry.name.as_ref() >= name {
            break;
        }
        at = at.checked_add(1).expect("bounded retained entries");
    }
    let cost = u64::try_from(size_of::<Entry>())
        .expect("entry cell fits u64")
        .checked_add(skein_lib::len_u64(name.len()))
        .expect("entry cost fits u64");
    entries.insert(at, Entry { name: Box::from(name), kind });
    *bytes = bytes.checked_add(cost).expect("bounded scan bytes fit u64");
    for _ in 0..entries.len() {
        if entries.len() <= usize_of(max) && *bytes <= max_bytes {
            break;
        }
        let removed = entries.pop().expect("overfull scan has a last entry");
        let removed_cost = u64::try_from(size_of::<Entry>())
            .expect("entry cell fits u64")
            .checked_add(skein_lib::len_u64(removed.name.len()))
            .expect("entry cost fits u64");
        *bytes = bytes.checked_sub(removed_cost).expect("removed entry was counted");
        *cutoff = Some(removed.name);
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive completion transition handles every file operation")]
fn up_inner(
    io: &mut FileIo,
    env: &Env<Limits>,
    pending: Pending,
    complete: Complete,
    events: &mut Queue<Event>,
    subs: &mut Queue<Submit>,
) -> Work {
    if let Err(error) = complete.result {
        let owner = match pending {
            Pending::Stat { owner, fd }
            | Pending::List { owner, fd, .. }
            | Pending::LoadStating { owner, fd, .. }
            | Pending::LoadReading { owner, fd, .. }
            | Pending::ScanListing { owner, fd, .. } => {
                return issue(io, Pending::Cleanup { owner, error }, Op::Close { fd }, subs);
            }
            Pending::LoadOpening { owner, loading, .. } => {
                if error == Error::Permission
                    && let Some(loading) = loading
                {
                    return private::denied_load(io, owner, loading, subs);
                }
                match private::unsafe_open(error, loading.is_some()) {
                    Some(found) => events.push(Event::Refused { owner, found }),
                    None => return terminal_failure(events, owner, error),
                }
                return Work::Idle;
            }
            Pending::Cleanup { owner, error } => {
                return terminal_failure(events, owner, error);
            }
            Pending::Create(owner)
            | Pending::OpenRead(owner)
            | Pending::OpenDirectory(owner)
            | Pending::Stating(owner)
            | Pending::Write(owner)
            | Pending::Read { owner, .. }
            | Pending::Cut(owner)
            | Pending::Sync(owner)
            | Pending::Close(owner)
            | Pending::Rename(owner)
            | Pending::HardLink(owner)
            | Pending::Remove(owner)
            | Pending::ListOpening(owner)
            | Pending::ListClosing { owner, .. }
            | Pending::LoadReopening { owner, .. }
            | Pending::LoadClosing { owner, .. }
            | Pending::LoadTooLargeClosing { owner, .. }
            | Pending::ScanOpening { owner, .. }
            | Pending::ScanClosing { owner, .. } => owner,
            Pending::Private(opening) => return private::up(io, opening, complete, events, subs),
            Pending::Store(store) => return store::up(io, env, store, complete, events, subs),
            Pending::Lock(owner) => return directory::locked(owner, complete, events),
            page @ Pending::Page { .. } => return directory::answered(io, page, complete, None, events, subs),
        };
        events.push(Event::failed(owner, error));
        return Work::Idle;
    }
    let done = complete.result.expect("checked success");
    match pending {
        Pending::Create(owner) => {
            let fd = store::done_fd(done);
            if let Some(file) = io.insert(fd) {
                events.push(Event::Opened { owner, file, len: 0 });
            } else {
                events.push(Event::failed(owner, Error::TooManyOpenFiles));
            }
        }
        Pending::OpenRead(owner) => {
            let fd = store::done_fd(done);
            return issue(io, Pending::Stat { owner, fd }, Op::Stat { fd }, subs);
        }
        Pending::OpenDirectory(owner) => {
            let fd = store::done_fd(done);
            if let Some(file) = io.insert(fd) {
                events.push(Event::Opened { owner, file, len: 0 });
            } else {
                return issue(io, Pending::Cleanup { owner, error: Error::TooManyOpenFiles }, Op::Close { fd }, subs);
            }
        }
        Pending::Stating(owner) => {
            let stat = store::done_stat(done);
            events.push(Event::Stated { owner, stat });
        }
        Pending::Stat { owner, fd } => {
            let stat = store::done_stat(done);
            if let Some(file) = io.insert(fd) {
                events.push(Event::Opened { owner, file, len: stat.size });
            } else {
                events.push(Event::failed(owner, Error::TooManyOpenFiles));
            }
        }
        Pending::Cleanup { owner, error } => {
            store::done_nothing(done);
            return terminal_failure(events, owner, error);
        }
        Pending::ListOpening(owner) => {
            let fd = store::done_fd(done);
            return list(io, Pending::List { owner, fd, entries: Vec::new() }, fd, subs);
        }
        Pending::LoadOpening { owner, max, loading } => {
            let fd = store::done_fd(done);
            return issue(io, Pending::LoadStating { owner, fd, max, loading }, Op::Stat { fd }, subs);
        }
        Pending::LoadReopening { owner, max, loading } => {
            store::done_nothing(done);
            let operation = Op::Open { root: loading.root, path: loading.path.clone(), how: OpenHow::ReadNoFollow };
            return issue(io, Pending::LoadOpening { owner, max, loading: Some(loading) }, operation, subs);
        }
        Pending::LoadStating { owner, fd, max, mut loading } => {
            let stat = store::done_stat(done);
            let unsafe_found = match loading.as_ref() {
                Some(loading) => private::unsafe_file(stat, loading.user),
                None => None,
            };
            if unsafe_found == Some(crate::file::Unsafe::Links(0))
                && match loading.as_ref() {
                    Some(loading) => !loading.reopened,
                    None => false,
                }
            {
                loading.as_mut().expect("a private load reopens").reopened = true;
                return issue(
                    io,
                    Pending::LoadReopening {
                        owner,
                        max,
                        loading: loading.take().expect("a private reopen retains its path"),
                    },
                    Op::Close { fd },
                    subs,
                );
            } else if let Some(found) = unsafe_found {
                return private::refuse_file(io, owner, fd, found, events, subs);
            } else if stat.kind != crate::kernel::Kind::File {
                return issue(io, Pending::Cleanup { owner, error: Error::IsADirectory }, Op::Close { fd }, subs);
            } else if stat.size > u64::from(max) {
                return issue(io, Pending::LoadTooLargeClosing { owner, size: stat.size }, Op::Close { fd }, subs);
            }
            let capacity = usize_of(max).checked_add(1).expect("one more byte fits");
            return read_whole(io, env, owner, fd, max, Contents(Vec::with_capacity(capacity)), subs);
        }
        Pending::LoadReading { owner, fd, max, mut bytes } => {
            let count = store::done_count(done);
            let buf = store::read_buffer(complete.kind);
            let n = usize_of(count);
            bytes.0.extend_from_slice(buf.get(..n).expect("read count is within buffer"));
            if bytes.0.len() > usize_of(max) {
                let size = skein_lib::len_u64(bytes.0.len());
                return issue(io, Pending::LoadTooLargeClosing { owner, size }, Op::Close { fd }, subs);
            } else if count == 0 {
                return issue(io, Pending::LoadClosing { owner, bytes }, Op::Close { fd }, subs);
            }
            return read_whole(io, env, owner, fd, max, bytes, subs);
        }
        Pending::LoadClosing { owner, bytes } => {
            store::done_nothing(done);
            events.push(Event::Loaded { owner, bytes: bytes.0.into_boxed_slice() });
        }
        Pending::LoadTooLargeClosing { owner, size } => {
            store::done_nothing(done);
            events.push(Event::TooLarge { owner, size });
        }
        Pending::ScanOpening { owner, max, max_bytes } => {
            let fd = store::done_fd(done);
            let cell = u64::try_from(size_of::<Entry>()).expect("entry cell fits u64");
            let by_bytes = max_bytes.checked_div(cell).expect("entry cell is nonzero");
            let capacity = u64::from(max).min(by_bytes).checked_add(1).expect("one transient entry fits");
            return list(
                io,
                Pending::ScanListing {
                    owner,
                    fd,
                    max,
                    max_bytes,
                    entries: Vec::with_capacity(usize::try_from(capacity).expect("scan capacity fits usize")),
                    bytes: 0,
                    cutoff: None,
                    total: 0,
                },
                fd,
                subs,
            );
        }
        Pending::ScanListing { owner, fd, max, max_bytes, mut entries, mut bytes, mut cutoff, mut total } => {
            let count = store::done_count(done);
            let (records, names) = list_parts(complete.kind);
            let count = usize_of(count);
            if count == 0 {
                let kept = skein_lib::len_u64(entries.len());
                let more = total.checked_sub(kept).expect("retained entries were counted");
                return issue(io, Pending::ScanClosing { owner, entries, more }, Op::Close { fd }, subs);
            }
            for record in records.get(..count).expect("list count is within buffer") {
                let name = record.name(&names).expect("a listed entry names bytes");
                if name != b"." && name != b".." {
                    total = total.checked_add(1).expect("directory entry count fits u64");
                    retain_scan_entry(&mut entries, &mut bytes, &mut cutoff, max, max_bytes, name, record.kind);
                }
            }
            return list(
                io,
                Pending::ScanListing { owner, fd, max, max_bytes, entries, bytes, cutoff, total },
                fd,
                subs,
            );
        }
        Pending::ScanClosing { owner, entries, more } => {
            store::done_nothing(done);
            events.push(Event::Scanned { owner, entries: entries.into_boxed_slice(), more });
        }
        Pending::Write(owner) => {
            let count = store::done_count(done);
            let (fd, bytes, from, at) = store::write_parts(complete.kind);
            let next = from.checked_add(count).expect("write count fits");
            if count == 0 {
                events.push(Event::failed(owner, Error::Other(5)));
            } else if usize_of(next) == bytes.len() {
                events.push(Event::Written { owner });
            } else {
                let at = at.checked_add(u64::from(count)).expect("write offset fits");
                let Ok(op) = Op::write(fd, bytes, next, at) else {
                    events.push(Event::failed(owner, Error::InvalidArgument));
                    return Work::Idle;
                };
                return issue(io, Pending::Write(owner), op, subs);
            }
        }
        Pending::Read { owner, fd, offset, max, mut bytes } => {
            let count = store::done_count(done);
            let buf = store::read_buffer(complete.kind);
            let n = usize_of(count);
            bytes.0.extend_from_slice(buf.get(..n).expect("read count is within buffer"));
            let done = count == 0 || u32::try_from(bytes.0.len()).expect("read no more than max") == max;
            if done {
                events.push(Event::Read { owner, bytes: bytes.0.into_boxed_slice() });
            } else {
                let at = offset.checked_add(skein_lib::len_u64(bytes.0.len())).expect("offset fits");
                let left = max.checked_sub(u32::try_from(bytes.0.len()).expect("length fits u32")).expect("within max");
                let buffer = vec![0_u8; usize_of(left)].into_boxed_slice();
                let Ok(op) = Op::read(fd, buffer, at) else {
                    events.push(Event::failed(owner, Error::InvalidArgument));
                    return Work::Idle;
                };
                return issue(io, Pending::Read { owner, fd, offset, max, bytes }, op, subs);
            }
        }
        Pending::Cut(owner) => {
            store::done_nothing(done);
            events.push(Event::Cut { owner });
        }
        Pending::Sync(owner) => {
            store::done_nothing(done);
            events.push(Event::Synced { owner });
        }
        Pending::Close(owner) => {
            store::done_nothing(done);
            events.push(Event::Closed { owner });
        }
        Pending::HardLink(owner) => {
            store::done_nothing(done);
            events.push(Event::Linked { owner });
        }
        Pending::Rename(owner) => {
            store::done_nothing(done);
            events.push(Event::Renamed { owner });
        }
        Pending::Remove(owner) => {
            store::done_nothing(done);
            events.push(Event::Removed { owner });
        }
        Pending::List { owner, fd, mut entries } => {
            let count = store::done_count(done);
            let (records, names) = list_parts(complete.kind);
            let count = usize_of(count);
            if count == 0 {
                return issue(io, Pending::ListClosing { owner, entries }, Op::Close { fd }, subs);
            }
            for record in records.get(..count).expect("list count is within buffer") {
                let name = record.name(&names).expect("a listed entry names bytes");
                entries.push(Entry { name: Box::from(name), kind: record.kind });
            }
            if entries.len() > usize_of(env.limits.entries) {
                return issue(io, Pending::Cleanup { owner, error: Error::NoBufferSpace }, Op::Close { fd }, subs);
            }
            return list(io, Pending::List { owner, fd, entries }, fd, subs);
        }
        Pending::ListClosing { owner, entries } => {
            store::done_nothing(done);
            events.push(Event::Listed { owner, entries: entries.into_boxed_slice() });
        }
        Pending::Private(opening) => {
            return private::up(io, opening, Complete { result: Ok(done), ..complete }, events, subs);
        }
        Pending::Store(store) => {
            return store::up(io, env, store, Complete { result: Ok(done), ..complete }, events, subs);
        }
        Pending::Lock(owner) => return directory::locked(owner, Complete { result: Ok(done), ..complete }, events),
        page @ Pending::Page { .. } => {
            return directory::answered(io, page, Complete { result: Ok(done), ..complete }, None, events, subs);
        }
    }
    Work::Idle
}

fn list_parts(kind: Op) -> (Box<[crate::kernel::Entry]>, Box<[u8]>) {
    match kind {
        Op::List { entries, names, .. } => (entries, names),
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
        | Op::Rename { .. }
        | Op::HardLink { .. }
        | Op::Remove { .. }
        | Op::MakeDirectory { .. }
        | Op::Read { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => unreachable!("a directory list completes a list"),
    }
}
