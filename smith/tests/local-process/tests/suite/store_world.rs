//! The Store/FileIo host and disk shared by the focused cases and the power-loss sweeps.
//! smith/docs/design/testing-tiers.md, 5.

use skein_fake_machine::{Item, Machine as Files, Opened};
use skein_io::{
    file,
    file_layer::{self, FileIo, step},
    kernel::{self, Complete, Fd, Op, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::Host;
use skein_sim::{Answer, Call, Handle};
use skein_world::{Cut, Machine};
use smith_local_protocol::{Action, Store, StoreLimits, StoreRequest};
use smith_protocol_translate::Endpoints;
use std::{
    borrow::Borrow,
    cell::{RefCell, RefMut},
};

const SIGNAL: Token = Token::new(u64::MAX);

/// What policy asks the idle Store to do on this turn.
pub(super) enum Next {
    Wait,
    Request(StoreRequest),
    Close,
}

/// The test policy sees the Store boundary; only the owner drives FileIo.
pub(super) trait Policy {
    fn ready(&self) -> bool {
        false
    }

    fn next(&mut self) -> Next {
        Next::Wait
    }

    fn io(&mut self, request: &file::Request);

    fn action(&mut self, action: Action, now: Time, store: &mut Store, actions: &mut Queue<Action>);

    fn submitted(&mut self, _count: u32) {}
}

/// Drives the two real machines and routes Store actions to a test policy.
pub(super) struct Owner<P> {
    pub(super) policy: P,
    store: Store,
    io: FileIo,
    file_limits: file_layer::Limits,
    signal: Option<Fd>,
    settled: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    events: Queue<file::Event>,
    actions: Queue<Action>,
}

impl<P: Policy> Owner<P> {
    pub(super) fn new(
        fd: Fd,
        limits: StoreLimits,
        file_limits: file_layer::Limits,
        endpoints: Endpoints,
        signal: Option<Fd>,
        policy: P,
    ) -> Self {
        let mut io = FileIo::with_limits(&file_limits);
        io.seed_randomness(19);
        let root = io.adopt_root(fd).unwrap();
        Self {
            policy,
            store: Store::new(Token::new(1), root, fd, limits, endpoints).unwrap(),
            io,
            file_limits,
            signal,
            settled: false,
            completions: Queue::with_capacity(4),
            submissions: Queue::with_capacity(4),
            events: Queue::with_capacity(1),
            actions: Queue::with_capacity(3),
        }
    }

    pub(super) fn request(&mut self, request: StoreRequest, now: Time) {
        self.store.request(request, now, &mut self.actions);
    }

    fn actions(&mut self, env: &Env<file_layer::Limits>) {
        for _ in 0..3 {
            if !self.io.takes() || self.submissions.room() < 2 {
                break;
            }
            let Some(action) = self.actions.pop() else { break };
            match action {
                Action::Io { request, deadline } => {
                    self.policy.io(&request);
                    step::down_until(&mut self.io, env, deadline, request, &mut self.events, &mut self.submissions);
                }
                Action::Closed => match self.signal {
                    Some(fd) => self.submissions.push(Submit { op: SIGNAL, kind: Op::Close { fd } }),
                    None => self.settled = true,
                },
                other => self.policy.action(other, env.now, &mut self.store, &mut self.actions),
            }
        }
    }
}

impl<P: Policy> Host for Owner<P> {
    fn iterate(&mut self, now: Time, wall: Wall) {
        match self.policy.next() {
            Next::Wait => {}
            Next::Request(request) => self.request(request, now),
            Next::Close => self.store.close(now, &mut self.actions),
        }
        let env = Env { now, wall, limits: self.file_limits };
        while let Some(complete) = self.completions.pop() {
            if complete.op == SIGNAL {
                assert!(complete.result.is_ok());
                self.settled = true;
            } else {
                step::up(&mut self.io, &env, complete, &mut self.events, &mut self.submissions);
            }
        }
        if self.io.is_due(now) {
            step::expire(&mut self.io, &env, &mut self.submissions);
        }
        for _ in 0..4 {
            self.actions(&env);
            let Some(event) = self.events.pop() else { break };
            if let file::Event::Opened { file, .. } = &event {
                self.store.opened_directory_descriptor(self.io.descriptor(*file).unwrap());
            }
            self.store.from_io(event, now, &mut self.actions);
        }
        self.policy.submitted(self.submissions.len());
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, _: Time) -> bool {
        self.policy.ready()
            || !self.completions.is_empty()
            || !self.events.is_empty()
            || (!self.actions.is_empty() && self.io.takes())
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }

    fn is_empty(&self) -> bool {
        self.settled
            && self.io.open_files() == 0
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.actions.is_empty()
            && self.events.is_empty()
    }

    fn operations(&self) -> u32 {
        4
    }
}

/// The fuzzy world owns its disk; the focused world lends it so its observer
/// can read it between serial requests, outside the host's heap accounting.
#[derive(Debug)]
pub(super) struct Disk<T = RefCell<Files>>(T);

impl Disk {
    pub(super) fn laid(items: &[Item]) -> Self {
        let mut files = Files::new();
        let root = files.lay(items);
        files.close(root);
        Self(RefCell::new(files))
    }

    pub(super) fn borrowed(&self) -> Disk<&RefCell<Files>> {
        Disk(&self.0)
    }
}

impl<T: Borrow<RefCell<Files>>> Disk<T> {
    pub(super) fn files(&self) -> RefMut<'_, Files> {
        self.0.borrow().borrow_mut()
    }
}

impl<T: Borrow<RefCell<Files>>> Machine for Disk<T> {
    fn open_root(&mut self, _: &[u8]) -> Result<Handle, kernel::Error> {
        Ok(Handle::new(self.files().reopen_root(0).raw()))
    }

    fn close_root(&mut self, root: Handle) {
        self.files().close(Opened::new(root.raw()));
    }

    fn step(&mut self, call: Call, answers: &mut Queue<Answer>) {
        skein_fake_machine::step(&mut self.files(), call, answers);
    }

    fn cut(&mut self, cut: Cut, held: &[Handle], seed: u64) {
        let mut files = self.files();
        for handle in held {
            files.close(Opened::new(handle.raw()));
        }
        assert_eq!(cut, Cut::PowerLoss);
        files.crash(seed);
    }
}
