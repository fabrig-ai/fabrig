//! Serial Store requests and independent no-follow read-back of the focused cases.
//! smith/docs/design/testing.md, 2.5.

use crate::store_world::{Disk, Next, Owner, Policy};
use skein_fake_machine::{How, Is, Item, Machine as Files, Node, Opened};
use skein_io::{file, file_layer};
use skein_lib::{Duration, List, Queue, Time};
use skein_sim::{Config, Handle};
use skein_world::{Memory, Outcome, Referee, World};
use smith_local_domain::Event;
use smith_local_protocol::{Action, Store, StoreLimits, StoreRequest};
use smith_protocol_translate::{Endpoint, Endpoints};
use std::{cell::RefCell, collections::VecDeque};

pub(super) const LIMITS: StoreLimits = smith_local_protocol_fixtures::STORE;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Entry {
    pub(super) name: Box<[u8]>,
    pub(super) node: Node,
    pub(super) kind: Is,
    pub(super) bytes: Vec<u8>,
    pub(super) target: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Snapshot {
    pub(super) root: Vec<Entry>,
    pub(super) child: Option<Vec<Entry>>,
}

fn entries(files: &mut Files, directory: Opened) -> Vec<Entry> {
    let mut entries = Vec::new();
    // The machine lists names in order. Page to the end, independently of
    // both Store scan caps, and read every regular file at its observed size.
    loop {
        let page = files.list(directory, 16, 4096).unwrap();
        if page.is_empty() {
            return entries;
        }
        for (_, name) in page {
            let entry = files.open(directory, &name, How::PathNoFollow).unwrap();
            let facts = files.stat(entry);
            let bytes = if facts.is == Is::File {
                let file = files.open(directory, &name, How::ReadNoFollow).unwrap();
                let bytes = files.read(file, 0, u32::try_from(facts.size).unwrap()).unwrap();
                files.close(file);
                bytes
            } else {
                Vec::new()
            };
            entries.push(Entry {
                name,
                node: files.node(entry),
                kind: facts.is,
                bytes,
                target: files.link_target(entry),
            });
            files.close(entry);
        }
    }
}

/// One laid fake disk, reused across fresh owners.
pub(super) struct Fixture {
    disk: Disk,
    limits: StoreLimits,
    child: Option<&'static [u8]>,
}

impl Fixture {
    pub(super) fn new(items: &[Item]) -> Self {
        Self::bounded(items, LIMITS)
    }

    pub(super) fn bounded(items: &[Item], limits: StoreLimits) -> Self {
        Self { disk: Disk::laid(items), limits, child: None }
    }

    pub(super) fn watch_child(mut self, name: &'static [u8]) -> Self {
        self.child = Some(name);
        self
    }

    pub(super) fn create(&self, name: &[u8], bytes: &[u8]) {
        let mut files = self.disk.files();
        let root = files.reopen_root(0);
        let file = files.open(root, name, How::CreateNoFollow { mode: 0o600 }).unwrap();
        files.write(file, 0, bytes).unwrap();
        files.close(file);
        files.close(root);
    }

    pub(super) fn remove(&self, name: &[u8]) {
        let mut files = self.disk.files();
        let root = files.reopen_root(0);
        files.remove(root, name, false).unwrap();
        files.close(root);
    }

    pub(super) fn snapshot(&self) -> Snapshot {
        let mut files = self.disk.files();
        let root = files.reopen_root(0);
        let child = self.child.map(|name| {
            let directory = files.open(root, name, How::DirectoryNoFollow).unwrap();
            let snapshot = entries(&mut files, directory);
            files.close(directory);
            snapshot
        });
        let snapshot = Snapshot { root: entries(&mut files, root), child };
        files.close(root);
        snapshot
    }

    pub(super) fn run<const N: usize>(&self, requests: impl FnOnce() -> [StoreRequest; N]) -> Run<'_> {
        self.run_with(requests, |_| {})
    }

    /// Always closes the owner. Run again on this fixture to get a fresh Store
    /// on the same disk. `between` can plant facts after a terminal and before
    /// the same owner receives its next request (or closes).
    pub(super) fn run_with<const N: usize>(
        &self,
        requests: impl FnOnce() -> [StoreRequest; N],
        mut between: impl FnMut(usize),
    ) -> Run<'_> {
        let before = self.snapshot();
        let mut after = Vec::with_capacity(N);
        let root = Handle::new(self.disk.files().reopen_root(0).raw());
        let mut world = World::new(713, Config::calm(), Judge { expected: N, done: false }, Memory::Checked)
            .with_machine(self.disk.borrowed());
        world.spawn_root(root, |fd| {
            let limits = self.limits;
            let file_limits = file_layer::Limits {
                files: 4,
                read: limits.file_bytes,
                entries: limits.turns + limits.deliveries + 4,
                whole: limits.file_bytes,
                timeout: Duration::from_secs(30),
            };
            let mut endpoints = List::with_capacity(1);
            endpoints.push(Endpoint { name: b"main".as_slice().into(), number: 2, dialect: 1 }).unwrap();
            Owner::new(fd, limits, file_limits, Endpoints::new(endpoints), None, Script::new(requests()))
        });
        let outcome = world.run_with_inputs(|sim, pids, owners| {
            // Each turn starts at most one new request. Observe the completed
            // one here before the host can start the next request or close.
            let completed = owners[0].policy.terminals.len();
            if completed > after.len() {
                assert_eq!(completed, after.len() + 1, "one terminal per request");
                assert_eq!(sim.in_flight(pids[0]), 0, "a terminal settles its IO");
                after.push(self.snapshot());
                between(completed);
            }
        });
        assert_eq!(after.len(), N);
        assert!(outcome.procs[0].policy.requests.iter().all(|&kind| kind == Kind::Close));
        assert_eq!(self.disk.files().open_handles(), 0);
        Run { before, after, closed: self.snapshot(), outcome }
    }
}

pub(super) struct Run<'a> {
    pub(super) before: Snapshot,
    pub(super) after: Vec<Snapshot>,
    pub(super) closed: Snapshot,
    outcome: Outcome<Owner<Script>, Disk<&'a RefCell<Files>>>,
}

impl Run<'_> {
    pub(super) fn terminal(&self, index: usize) -> &Terminal {
        &self.outcome.procs[0].policy.terminals[index]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Load,
    Scan,
    Store,
    Open,
    Make,
    Close,
    Remove,
    Sync,
}

pub(super) struct Terminal {
    pub(super) event: Event,
    pub(super) requests: Vec<Kind>,
    pub(super) starts_with_marker_load: bool,
}

struct Script {
    pending: VecDeque<StoreRequest>,
    terminals: Vec<Terminal>,
    requests: Vec<Kind>,
    starts_with_marker_load: bool,
    waiting: bool,
    closing: bool,
}

impl Script {
    fn new<const N: usize>(requests: [StoreRequest; N]) -> Self {
        Self {
            pending: VecDeque::from(requests),
            terminals: Vec::with_capacity(N),
            requests: Vec::new(),
            starts_with_marker_load: false,
            waiting: false,
            closing: false,
        }
    }
}

impl Policy for Script {
    fn ready(&self) -> bool {
        !self.waiting && !self.closing
    }

    fn next(&mut self) -> Next {
        if !self.ready() {
            return Next::Wait;
        }
        match self.pending.pop_front() {
            Some(request) => {
                self.waiting = true;
                Next::Request(request)
            }
            None => {
                self.closing = true;
                Next::Close
            }
        }
    }

    fn io(&mut self, request: &file::Request) {
        if self.requests.is_empty() {
            self.starts_with_marker_load =
                matches!(request, file::Request::Load { path, .. } if path.as_ref() == b"format");
        }
        let kind = match request {
            file::Request::Load { .. } => Kind::Load,
            file::Request::Scan { .. } => Kind::Scan,
            file::Request::Store { .. } => Kind::Store,
            file::Request::OpenDirectory { .. } => Kind::Open,
            file::Request::MakeDirectory { .. } => Kind::Make,
            file::Request::Close { .. } => Kind::Close,
            file::Request::Remove { .. } | file::Request::RemoveIn { .. } => Kind::Remove,
            file::Request::SyncDirectory { .. } => Kind::Sync,
            other => panic!("unexpected Store request {other:?}"),
        };
        self.requests.push(kind);
    }

    fn action(&mut self, action: Action, _: Time, _: &mut Store, actions: &mut Queue<Action>) {
        match action {
            Action::Domain(event) => {
                assert!(self.waiting, "unexpected terminal {event:?}");
                assert!(actions.is_empty(), "a terminal is the last action of its request");
                self.terminals.push(Terminal {
                    event,
                    requests: std::mem::take(&mut self.requests),
                    starts_with_marker_load: self.starts_with_marker_load,
                });
                self.waiting = false;
            }
            Action::Fault(_) => {} // A diagnostic, not the request's terminal.
            other => panic!("unexpected Store action {other:?}"),
        }
    }
}

struct Judge {
    expected: usize,
    done: bool,
}

impl Referee<Owner<Script>> for Judge {
    fn watch(&mut self, _: Time, owners: &[Owner<Script>]) -> Result<(), String> {
        let script = &owners[0].policy;
        self.done = script.closing && script.terminals.len() == self.expected;
        Ok(())
    }

    fn passed(&self) -> bool {
        self.done
    }
}
