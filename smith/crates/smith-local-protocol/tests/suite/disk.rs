//! The real-directory Store/FileIo adapter for the fresh-journal recovery test.
//! smith/docs/design/testing.md, 2.5.

use skein_io::file_layer::step::down_until;
use skein_io::{
    file,
    file_layer::{self, FileIo, step},
    kernel::{Complete, Submit},
};
use skein_lib::{Duration, Env, List, Queue, Token};
use skein_scratch::Scratch;
use skein_shell::{Backend, Clock, Config, Kernel, Wait};
use skein_world::{Deadline, Expired};
use smith_local_domain as local;
use smith_local_protocol::{Action, Store, StoreRequest};
use std::path::Path;

/// The fixture's single main endpoint: number 2, dialect 1, without an address.
#[must_use]
fn endpoints() -> smith_protocol_translate::Endpoints {
    let mut entries = List::with_capacity(1);
    entries
        .push(smith_protocol_translate::Endpoint { name: b"main".as_slice().into(), number: 2, dialect: 1 })
        .unwrap();
    smith_protocol_translate::Endpoints::new(entries)
}

/// A concrete real directory and the explicitly closed Store/`FileIo`/kernel stack.
#[derive(Debug)]
pub struct Disk {
    scratch: Scratch,
    store: Store,
    io: FileIo,
    file_limits: file_layer::Limits,
    kernel: Kernel,
    clock: Clock,
    actions: Queue<Action>,
    events: Queue<file::Event>,
    submissions: Queue<Submit>,
    completions: Queue<Complete>,
    closed: bool,
}

impl Disk {
    /// Create a fresh scratch directory with tiny Store limits on the real readiness backend.
    #[must_use]
    #[expect(
        clippy::new_without_default,
        reason = "a real-directory fixture is constructed and explicitly closed, never defaulted"
    )]
    pub fn new() -> Self {
        let limits = smith_local_protocol_fixtures::STORE;
        let scratch = Scratch::new("smith-store");
        let fd = skein_shell::open_root(scratch.path()).unwrap();
        let scan = limits
            .turns
            .checked_add(limits.deliveries)
            .and_then(|count| count.checked_add(4))
            .expect("tiny scan capacity");
        let timeout = Duration::from_secs(30);
        let file_limits =
            file_layer::Limits { files: 4, read: limits.file_bytes, entries: scan, whole: limits.file_bytes, timeout };
        let mut io = FileIo::with_limits(&file_limits);
        io.seed_randomness(713);
        let root = io.adopt_root(fd).unwrap();
        let endpoints = endpoints();
        let store = Store::new(Token::new(1), root, fd, limits, endpoints).unwrap();
        Self {
            scratch,
            store,
            io,
            file_limits,
            kernel: Kernel::open_on(Config { operations: 8 }, Backend::Readiness).expect("real readiness backend"),
            clock: Clock::new(),
            actions: Queue::with_capacity(3),
            events: Queue::with_capacity(1),
            submissions: Queue::with_capacity(8),
            completions: Queue::with_capacity(8),
            closed: false,
        }
    }

    /// The owned scratch path; fixtures can lay files only between serial requests.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.scratch.path()
    }

    /// Drive one request to exactly one domain terminal through real `FileIo` and kernel.
    pub fn drive(&mut self, request: StoreRequest) -> local::Event {
        assert!(!self.closed);
        self.store.request(request, self.clock.now().now, &mut self.actions);
        self.pump().expect("one domain terminal")
    }

    /// Explicitly drain Store close, both owned roots and all kernel operations.
    pub fn close(&mut self) {
        self.store.close(self.clock.now().now, &mut self.actions);
        assert!(self.pump().is_none());
        assert!(self.closed);
        assert_eq!(self.io.open_files(), 0);
        assert_eq!(self.kernel.in_flight(), 0);
    }

    fn pump(&mut self) -> Option<local::Event> {
        let deadline = Deadline::arm("a tiny Store operation", Duration::from_secs(5));
        loop {
            if deadline.passed() {
                match deadline.expired() {
                    // No child or peer is owned; files and kernel operations end with this process.
                    Expired::Starved(starved) => starved.pass(),
                    Expired::Failed(figures) => panic!("a tiny Store operation settles promptly: {figures}"),
                }
            }
            let clock = self.clock.now();
            let now = clock.now;
            let env = Env { now, wall: clock.wall, limits: self.file_limits };
            self.kernel.reap(&mut self.completions);
            while let Some(complete) = self.completions.pop() {
                step::up(&mut self.io, &env, complete, &mut self.events, &mut self.submissions);
            }
            while let Some(event) = self.events.pop() {
                self.store.from_io(event, now, &mut self.actions);
            }
            while let Some(action) = self.actions.pop() {
                match action {
                    Action::Io { request, deadline } => {
                        down_until(&mut self.io, &env, deadline, request, &mut self.events, &mut self.submissions);
                    }
                    Action::Domain(event) => {
                        assert!(self.actions.is_empty());
                        assert!(self.submissions.is_empty());
                        assert_eq!(self.kernel.in_flight(), 0);
                        return Some(event);
                    }
                    Action::Fault(_) => {} // A diagnostic, not the request's terminal.
                    Action::Closed => {
                        self.closed = true;
                        return None;
                    }
                    Action::CleanupWarning { error } => panic!("unexpected cleanup warning {error:?}"),
                }
            }
            if self.io.is_due(now) {
                step::expire(&mut self.io, &env, &mut self.submissions);
            }
            let wait = if self.events.is_empty() { Wait::Until(deadline.until()) } else { Wait::No };
            self.kernel.submit(&mut self.submissions, wait);
        }
    }
}
