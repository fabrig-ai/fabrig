//! Git, marker and plain-directory IO retained by the host (protocol/hosts.md, section 5.5).
//! Snapshots are complete or unknown; policy receives typed terminals only.
use super::{HostIo, git_limits, plain_limits};
use alloc::boxed::Box;
use core::mem;
use skein_io::{file, file_layer::step, kernel};
use skein_lib::{List, Map, Queue, Time, Token};
use smith_local_domain as local;
use smith_local_protocol as local_protocol;

/// One capture of the plain directories: from `next`, until `deadline`.
#[derive(Debug)]
pub(super) struct Capture {
    directories: List<u32>,
    next: u32,
    deadline: Time,
}

/// The one file inspection at a time; every file terminal that is neither the store's nor a
/// closing root's belongs to it (protocol/hosts.md, section 5.5).
#[derive(Debug)]
pub(super) enum Inspection {
    /// No inspection in flight.
    Idle,
    /// Scanning `directory` for a capture.
    Capturing { capture: Capture, directory: u32, scan: local_protocol::Plain },
    /// Comparing `directory` with its snapshot, for `owner`.
    Comparing { owner: Token, directory: u32, scan: local_protocol::Plain },
    /// Reading files for conflict markers, for `owner`.
    Markers { owner: Token, markers: local_protocol::Markers },
}

impl Inspection {
    pub(super) const fn idle(&self) -> bool {
        match self {
            Self::Idle => true,
            Self::Capturing { .. } | Self::Comparing { .. } | Self::Markers { .. } => false,
        }
    }
}

#[derive(Debug)]
pub(super) struct DeliveryIo {
    pub(super) git: Option<local_protocol::GitChild>,
    pub(super) inspection: Inspection,
    pub(super) snapshots: Map<u32, Option<local_protocol::Snapshot>>,
    pub(super) capture_done: Option<bool>,
    pub(super) delivery_roots: Box<[kernel::Fd]>,
    pub(super) git_environment: Box<[Box<[u8]>]>,
}

impl DeliveryIo {
    pub(super) fn new() -> Self {
        Self {
            git: None,
            inspection: Inspection::Idle,
            snapshots: Map::with_capacity(64),
            capture_done: None,
            delivery_roots: Box::new([]),
            git_environment: Box::new([]),
        }
    }
}

impl HostIo {
    pub(crate) fn adopt_delivery_roots(&mut self, roots: Box<[kernel::Fd]>, environment: Box<[Box<[u8]>]>) {
        let mut file_roots = List::with_capacity(u32::try_from(roots.len()).expect("admitted root count"));
        for root in &roots {
            file_roots
                .push(self.files.adopt_root(*root).expect("bounded workspace roots"))
                .expect("one token per root");
        }
        self.file_roots = file_roots.into_boxed();
        self.delivery.delivery_roots = roots;
        self.delivery.git_environment = environment;
    }

    pub(crate) fn capture_plain(&mut self, directories: List<u32>, deadline: Time) {
        assert!(self.delivery.inspection.idle(), "file inspection is serial");
        self.delivery.snapshots = Map::with_capacity(64);
        self.delivery.capture_done = None;
        self.next_capture(Capture { directories, next: 0, deadline });
    }

    pub(crate) fn take_capture_done(&mut self) -> Option<bool> {
        self.delivery.capture_done.take()
    }

    pub(super) fn next_capture(&mut self, capture: Capture) {
        match capture.directories.get(capture.next) {
            Some(directory) => {
                let directory = *directory;
                let root = match self.file_roots.get(usize::try_from(directory).expect("bounded directory")) {
                    Some(root) => *root,
                    None => {
                        self.delivery.capture_done = Some(false);
                        return;
                    }
                };
                let scan = local_protocol::Plain::new(Token::new(0), root, plain_limits(), capture.deadline)
                    .expect("checked plain limits");
                let action = scan.start();
                self.delivery.inspection = Inspection::Capturing { capture, directory, scan };
                match action {
                    local_protocol::PlainAction::File { request, deadline } => {
                        self.file_requests.push((request, deadline));
                    }
                    local_protocol::PlainAction::Done | local_protocol::PlainAction::Failed => {
                        unreachable!("root is first scan")
                    }
                }
            }
            None => self.delivery.capture_done = Some(true),
        }
    }

    pub(crate) fn plain_status(
        &mut self,
        owner: Token,
        directory: u32,
        deadline: Time,
        events: &mut Queue<local::Event>,
    ) {
        assert!(self.delivery.inspection.idle(), "file inspection is serial");
        let root = match self.file_roots.get(usize::try_from(directory).expect("bounded directory")) {
            Some(root) => *root,
            None => {
                events.push(local::Event::StoreFailed { turn: None, reason: local::StoreFailure::Read });
                return;
            }
        };
        match self.delivery.snapshots.get(&directory) {
            Some(Some(_)) => {}
            Some(None) => {
                events.push(local::Event::PlainStatus { owner, changed: true });
                return;
            }
            None => {
                events.push(local::Event::StoreFailed { turn: None, reason: local::StoreFailure::Read });
                return;
            }
        }
        let scan = local_protocol::Plain::new(owner, root, plain_limits(), deadline).expect("checked plain limits");
        let action = scan.start();
        self.delivery.inspection = Inspection::Comparing { owner, directory, scan };
        self.plain_action(action, events);
    }

    pub(super) fn plain_action(&mut self, action: local_protocol::PlainAction, events: &mut Queue<local::Event>) {
        let finished = match action {
            local_protocol::PlainAction::File { request, deadline } => {
                self.file_requests.push((request, deadline));
                return;
            }
            local_protocol::PlainAction::Done => true,
            local_protocol::PlainAction::Failed => false,
        };
        let state = mem::replace(&mut self.delivery.inspection, Inspection::Idle);
        match state {
            Inspection::Capturing { mut capture, directory, scan } => {
                let snapshot = if finished { Some(scan.finish()) } else { None };
                self.delivery.snapshots.insert(directory, snapshot).expect("bounded snapshots");
                capture.next = capture.next.checked_add(1).expect("bounded directory position");
                self.next_capture(capture);
            }
            Inspection::Comparing { owner, directory, scan } => {
                let changed = if finished {
                    let snapshot = scan.finish();
                    let before = self
                        .delivery
                        .snapshots
                        .get(&directory)
                        .expect("capture precedes compare")
                        .as_ref()
                        .expect("unknown capture answers without a scan");
                    *before != snapshot
                } else {
                    true
                };
                events.push(local::Event::PlainStatus { owner, changed });
            }
            Inspection::Idle | Inspection::Markers { .. } => unreachable!("active plain scan"),
        }
    }

    pub(crate) fn start_markers(
        &mut self,
        owner: Token,
        directory: u32,
        paths: Box<[Box<[u8]>]>,
        deadline: Time,
        events: &mut Queue<local::Event>,
    ) {
        assert!(self.delivery.inspection.idle(), "one local file inspection at a time");
        let root = self.file_roots.get(usize::try_from(directory).expect("admitted directory"));
        let markers = match root {
            Some(root) => local_protocol::Markers::new(owner, *root, paths, deadline, git_limits(&self.limits)),
            None => None,
        };
        match markers {
            Some(markers) => {
                let action = markers.start();
                self.delivery.inspection = Inspection::Markers { owner, markers };
                self.marker_action(owner, action, events);
            }
            None => events.push(local::Event::Git {
                owner,
                result: local::GitResult::Failed {
                    reason: smith_domain::run::DeliveryReason::TooLarge,
                    diagnostic: Box::new(smith_domain::run::Diagnostic::empty()),
                },
            }),
        }
    }

    pub(super) fn marker_action(
        &mut self,
        owner: Token,
        action: local_protocol::MarkerAction,
        events: &mut Queue<local::Event>,
    ) {
        match action {
            local_protocol::MarkerAction::File { request, deadline } => self.file_requests.push((request, deadline)),
            local_protocol::MarkerAction::Done(result) => {
                self.delivery.inspection = Inspection::Idle;
                events.push(local::Event::Git { owner, result });
            }
        }
    }

    pub(crate) fn start_git(
        &mut self,
        owner: Token,
        directory: u32,
        op: smith_local_domain::GitOp,
        deadline: Time,
        events: &mut Queue<smith_local_domain::Event>,
    ) {
        assert!(self.delivery.git.is_none(), "the local domain serializes git operations");
        let root = match usize::try_from(directory) {
            Ok(index) => self.delivery.delivery_roots.get(index).copied(),
            Err(_) => None,
        };
        self.delivery.git = match root {
            Some(root) => local_protocol::GitChild::new(
                owner,
                Token::new(1 << 62),
                root,
                self.delivery.git_environment.clone(),
                op,
                deadline,
                git_limits(&self.limits),
                &mut self.io_requests,
            ),
            None => None,
        };
        if self.delivery.git.is_none() {
            events.push(smith_local_domain::Event::Git {
                owner,
                result: smith_local_domain::GitResult::Failed {
                    reason: smith_domain::run::DeliveryReason::Broken,
                    diagnostic: Box::new(smith_domain::run::Diagnostic::empty()),
                },
            });
        }
    }

    pub(super) fn markers_up(&mut self, now: Time, local_events: &mut Queue<local::Event>) {
        if self.files.is_due(now) {
            let env = self.file_env(now, skein_lib::Wall::EPOCH);
            step::expire(&mut self.files, &env, &mut self.file_submissions);
        }
        self.store_pass(now, local_events);
        for _ in 0..self.limits.queue {
            if !self.store_actions_dispatched() {
                break;
            }
            let Some(event) = self.file_events.pop() else { break };
            if event.owner() == Token::new(u64::MAX - 1) {
                if let file::Event::Opened { file, .. } = &event {
                    let descriptor = self.files.descriptor(*file).expect("opened directory is owned by the file layer");
                    self.store_io
                        .store
                        .as_mut()
                        .expect("store owns its directory open")
                        .opened_directory_descriptor(descriptor);
                }
                self.store_io.store.as_mut().expect("store owner remains until Closed").from_io(
                    event,
                    now,
                    &mut self.store_io.store_actions,
                );
                continue;
            }
            if self.close.settle_root(event.owner()) {
                continue;
            }
            match &mut self.delivery.inspection {
                Inspection::Capturing { scan, .. } | Inspection::Comparing { scan, .. } => {
                    let action = scan.from_file(event);
                    self.plain_action(action, local_events);
                }
                Inspection::Markers { owner, markers } => {
                    let owner = *owner;
                    let action = markers.from_file(event);
                    self.marker_action(owner, action, local_events);
                }
                Inspection::Idle => unreachable!("every other file terminal belongs to the one inspection"),
            }
        }
    }
}
