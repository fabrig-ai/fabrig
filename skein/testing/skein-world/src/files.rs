//! A `FileIo` child for a world's socket owner (runtime/io/files.md, sections 2 and 3).
//! Queued requests keep their deadlines; descriptor cleanup remains admitted
//! until it can close the descriptor. Cancellation is routed even while a
//! different request runs. `FileIo`'s operation lane distinguishes its completions
//! from the socket driver's (runtime/io/files.md). The owner still routes file terminals
//! and closes its startup root after its component has settled.

use alloc::collections::VecDeque;
use skein_io::file::{Event, Request};
use skein_io::file_layer::{self, FileIo, step};
use skein_io::kernel::{Complete, Error, Fd, Submit};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

const ROOM: u32 = 64;

struct Queued {
    request: Request,
    deadline: Time,
}

/// The shared file-driver adapter, retaining file operations until their terminals.
#[expect(missing_debug_implementations, reason = "file requests may contain private bytes")]
pub struct Driver {
    io: FileIo,
    file_limits: file_layer::Limits,
    root: Token,
    queue: VecDeque<Queued>,
    events: Queue<Event>,
    submissions: Queue<Submit>,
    closing: bool,
}

impl Driver {
    /// Adopts the owner's startup root after configuring its effective user.
    #[must_use]
    pub fn new(root: Fd, files: u32, max: u32, stall: Duration, user: u32) -> Driver {
        let file_limits = file_layer::Limits { files, read: 32, entries: 4, whole: max, timeout: stall };
        let mut io = FileIo::with_limits(&file_limits).with_effective_user(user);
        io.seed_randomness(123);
        let root = io.adopt_root(root).expect("world startup root");
        Driver {
            io,
            file_limits,
            root,
            queue: VecDeque::with_capacity(usize::try_from(ROOM).expect("room")),
            events: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
            closing: false,
        }
    }

    /// The adopted startup root, which the component borrows and never closes.
    #[must_use]
    pub const fn root(&self) -> Token {
        self.root
    }

    /// Maps an adopted file token for requests whose roots still use descriptors.
    #[must_use]
    pub fn descriptor(&self, token: Token) -> Option<Fd> {
        self.io.descriptor(token)
    }

    /// Admits one bounded request, retaining its stated deadline while queued.
    pub fn request(&mut self, request: Request, deadline: Time) {
        assert!(self.queue.len() < usize::try_from(ROOM).expect("bounded queue"), "file request queue bound");
        self.queue.push_back(Queued { request, deadline });
    }

    /// Cancels a queued request once, or asks `FileIo` to cancel the active owner.
    pub fn cancel(&mut self, owner: Token) {
        if let Some(index) = self
            .queue
            .iter()
            .position(|queued| owner_of(&queued.request) == owner && !matches!(queued.request, Request::Close { .. }))
        {
            let removed = self.queue.remove(index).expect("found queued request");
            drop(removed);
            self.events.push(Event::Cancelled { owner });
        } else {
            file_layer::cancel(&mut self.io, owner, &mut self.submissions);
        }
    }

    /// Fires deadlines and starts the next request once `FileIo` has settled.
    pub fn progress(&mut self, now: Time) {
        let env = Env { now, wall: Wall::EPOCH, limits: self.file_limits };
        if self.io.is_due(now) {
            step::expire(&mut self.io, &env, &mut self.submissions);
        }
        for index in (0..self.queue.len()).rev() {
            let queued = self.queue.get(index).expect("queued request");
            if queued.deadline <= now && !matches!(queued.request, Request::Close { .. }) {
                let queued = self.queue.remove(index).expect("expired request");
                self.events.push(Event::Failed {
                    owner: owner_of(&queued.request),
                    error: Error::TimedOut,
                    committed: false,
                    residue: None,
                });
            }
        }
        if self.io.takes()
            && let Some(Queued { request, deadline }) = self.queue.pop_front()
        {
            if now >= deadline && !matches!(request, Request::Close { .. }) {
                self.events.push(Event::Failed {
                    owner: owner_of(&request),
                    error: Error::TimedOut,
                    committed: false,
                    residue: None,
                });
            } else {
                step::down_until(&mut self.io, &env, deadline, request, &mut self.events, &mut self.submissions);
            }
        }
    }

    /// Routes one kernel completion back to the file driver.
    pub fn up(&mut self, complete: Complete) {
        assert!(Self::owns(complete.op), "completion belongs to the file namespace");
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: self.file_limits };
        step::up(&mut self.io, &env, complete, &mut self.events, &mut self.submissions);
    }

    /// Whether this kernel owner token belongs to the file driver.
    #[must_use]
    pub const fn owns(token: Token) -> bool {
        file_layer::owns(token)
    }

    /// One kernel submission with its file-operation token and cancellation target.
    #[must_use]
    pub fn take_submit(&mut self) -> Option<Submit> {
        self.submissions.pop()
    }

    /// One file terminal for the component owner to route by its request token.
    #[must_use]
    pub fn take_event(&mut self) -> Option<Event> {
        self.events.pop()
    }

    /// Closes the startup root only after the component says Closed.
    pub fn close_root(&mut self, now: Time) {
        assert!(!self.closing, "startup root closes once after component settlement");
        self.closing = true;
        self.request(
            Request::Close { owner: Token::new(u64::MAX), file: self.root },
            now.saturating_add(Duration::from_secs(1)),
        );
    }

    /// The earliest queued or active request deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.queue
            .iter()
            .filter(|queued| !matches!(queued.request, Request::Close { .. }))
            .map(|queued| queued.deadline)
            .chain(self.io.next_deadline())
            .min()
    }

    /// Work the owner can do before sleeping.
    #[must_use]
    pub fn has_work(&self) -> bool {
        !self.events.is_empty() || !self.submissions.is_empty() || (self.io.takes() && !self.queue.is_empty())
    }

    /// Every descriptor and admitted operation has settled after startup-root closing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.closing
            && self.io.takes()
            && self.io.open_files() == 0
            && self.queue.is_empty()
            && self.events.is_empty()
            && self.submissions.is_empty()
    }
}

fn owner_of(request: &Request) -> Token {
    match request {
        Request::Lock { owner, .. }
        | Request::MakeDirectory { owner, .. }
        | Request::OpenPrivate { owner, .. }
        | Request::Create { owner, .. }
        | Request::CreateNoFollow { owner, .. }
        | Request::OpenWrite { owner, .. }
        | Request::OpenWriteNoFollow { owner, .. }
        | Request::Cut { owner, .. }
        | Request::OpenRead { owner, .. }
        | Request::OpenReadNoFollow { owner, .. }
        | Request::OpenDirectory { owner, .. }
        | Request::Load { owner, .. }
        | Request::Scan { owner, .. }
        | Request::Store { owner, .. }
        | Request::Stat { owner, .. }
        | Request::WriteAt { owner, .. }
        | Request::ReadAt { owner, .. }
        | Request::Sync { owner, .. }
        | Request::Close { owner, .. }
        | Request::SyncDirectory { owner, .. }
        | Request::HardLink { owner, .. }
        | Request::Rename { owner, .. }
        | Request::Remove { owner, .. }
        | Request::ListPage { owner, .. }
        | Request::List { owner, .. } => *owner,
    }
}
