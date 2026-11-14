//! Startup's durable whole-file store (runtime/shell.md, 6.7).
use crate::{Clock, Host, Kernel};
use skein_io::file_layer::{FileIo, Limits, step};
use skein_io::{file, kernel};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

/// The most operations a startup store has in flight: the ring a caller opens for it alone.
pub const STORE_OPERATIONS: u32 = 8;
const OWNER: Token = Token::new(1);

/// The directory a startup store writes into (runtime/shell.md, 6.7).
#[derive(Clone, Copy, Debug)]
pub enum StoreDirectory<'a> {
    /// The store's root itself.
    Root,
    /// The named directory under the root, opened through io's private-file checks as `user`.
    Private { name: &'a [u8], user: u32 },
}
/// One durable whole-file write at startup.
pub struct StartupStore<'a> {
    /// An open directory from `open_root`; the store closes it on every path.
    pub root: kernel::Fd,
    pub directory: StoreDirectory<'a>,
    pub name: &'a [u8],
    pub bytes: Box<[u8]>,
    pub expected: file::Expect,
    pub timeout: Duration,
    pub seed: u64,
}
/// How a startup store ended, with everything it opened closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stored {
    /// The file is durable.
    Kept,
    /// The expectation failed; nothing was written.
    Conflict,
}
/// Why a startup store failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unstored {
    /// io's private-file checks refused the directory.
    Unsafe(file::Unsafe),
    /// The store or a close after it failed; the first such error.
    Kernel(kernel::Error),
    /// The bytes exceed a `u32`.
    TooLarge,
}

/// Stores one file durably and closes the supplied root (runtime/shell.md, 6.7).
pub fn store_file(kernel: &mut Kernel, clock: &Clock, store: StartupStore<'_>) -> Result<Stored, Unstored> {
    let maximum = u32::try_from(store.bytes.len());
    let failure = maximum.err().map(|_| Unstored::TooLarge);
    let whole = maximum.unwrap_or(1).max(1);
    let limits = Limits { files: 2, read: 4096, entries: 1, whole, timeout: store.timeout };
    let mut io = FileIo::with_limits(&limits);
    if let StoreDirectory::Private { user, .. } = store.directory {
        io = io.with_effective_user(user);
    }
    io.seed_randomness(store.seed);
    let root = io.adopt_root(store.root).expect("one root");
    let opening = match store.directory {
        StoreDirectory::Root => None,
        StoreDirectory::Private { name, .. } => {
            Some(file::Request::OpenPrivate { owner: OWNER, root, path: name.into() })
        }
    };
    let request = Some(file::Request::Store {
        owner: OWNER,
        root,
        path: store.name.into(),
        bytes: store.bytes,
        expected: store.expected,
        no_follow: true,
    });
    let mut host = Storing {
        io,
        limits,
        roots: vec![root],
        opening,
        request,
        terminal: failure.is_some(),
        closed: false,
        outcome: Stored::Conflict,
        failure,
        events: Queue::with_capacity(4),
        completions: Queue::with_capacity(4),
        submissions: Queue::with_capacity(4),
    };
    let _exit = crate::drive(kernel, clock, &mut host);
    host.failure.map_or(Ok(host.outcome), Err)
}

struct Storing {
    io: FileIo,
    limits: Limits,
    roots: Vec<Token>,
    opening: Option<file::Request>,
    request: Option<file::Request>,
    terminal: bool,
    closed: bool,
    outcome: Stored,
    failure: Option<Unstored>,
    events: Queue<file::Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}
impl Storing {
    fn fail(&mut self, why: Unstored) {
        self.failure.get_or_insert(why);
        self.terminal = true;
        self.closed = self.io.open_files() == 0;
    }
}
impl Host for Storing {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        while let Some(complete) = self.completions.pop() {
            step::up(&mut self.io, &env, complete, &mut self.events, &mut self.submissions);
        }
        if self.io.is_due(now) {
            step::expire(&mut self.io, &env, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            match event {
                file::Event::Stored { .. } => {
                    self.outcome = Stored::Kept;
                    self.terminal = true;
                }
                file::Event::Conflict { .. } => self.terminal = true,
                file::Event::Opened { file, .. } => {
                    self.roots.push(file);
                    if let Some(file::Request::Store { root, .. }) = &mut self.request {
                        *root = file;
                    }
                }
                file::Event::Closed { .. } => self.closed = self.roots.is_empty(),
                file::Event::Failed { error, .. } => self.fail(Unstored::Kernel(error)),
                file::Event::Refused { found, .. } => self.fail(Unstored::Unsafe(found)),
                file::Event::Cancelled { .. } => self.fail(Unstored::Kernel(kernel::Error::Cancelled)),
                file::Event::Made { .. }
                | file::Event::Locked { .. }
                | file::Event::Busy { .. }
                | file::Event::Loaded { .. }
                | file::Event::Scanned { .. }
                | file::Event::TooLarge { .. }
                | file::Event::Stated { .. }
                | file::Event::Written { .. }
                | file::Event::Read { .. }
                | file::Event::Cut { .. }
                | file::Event::Synced { .. }
                | file::Event::Linked { .. }
                | file::Event::Renamed { .. }
                | file::Event::Removed { .. }
                | file::Event::Listed { .. } => unreachable!("startup store vocabulary"),
            }
        }
        if self.io.takes() && !self.closed {
            let request = if self.terminal {
                self.request = None;
                self.roots.pop().map(|file| file::Request::Close { owner: OWNER, file })
            } else {
                self.opening.take().or_else(|| self.request.take())
            };
            if let Some(request) = request {
                step::down(&mut self.io, &env, request, &mut self.events, &mut self.submissions);
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.events.is_empty()
            || !self.completions.is_empty()
            || self.io.is_due(now)
            || (self.io.takes() && !self.closed && (self.request.is_some() || self.terminal))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn is_empty(&self) -> bool {
        self.closed && self.submissions.is_empty() && self.completions.is_empty() && self.events.is_empty()
    }
    fn operations(&self) -> u32 {
        STORE_OPERATIONS
    }
}
