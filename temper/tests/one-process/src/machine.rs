//! Factories close unused startup signal descriptors before running their host.
use skein_io::kernel::{Complete, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
const CLOSE: Token = Token::new(u64::MAX);
pub(crate) struct SignalHost<H> {
    host: H,
    signal: Option<Fd>,
    closing: bool,
}
impl<H> SignalHost<H> {
    pub fn new(host: H, signal: Option<Fd>) -> Self {
        Self { host, signal, closing: false }
    }
}
impl<H> Deref for SignalHost<H> {
    type Target = H;
    fn deref(&self) -> &H {
        &self.host
    }
}
impl<H> DerefMut for SignalHost<H> {
    fn deref_mut(&mut self) -> &mut H {
        &mut self.host
    }
}
impl<H: Host> Host for SignalHost<H> {
    fn drain(&mut self) {
        self.host.drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        for _ in 0..self.host.completions().len() {
            let complete = self.host.completions().pop().expect("completion count");
            if complete.op == CLOSE {
                complete.result.expect("startup signal close");
                self.signal = None;
            } else {
                self.host.completions().push(complete);
            }
        }
        if !self.closing
            && self.host.submissions().room() > 0
            && let Some(fd) = self.signal
        {
            self.closing = true;
            self.host.submissions().push(Submit { op: CLOSE, kind: Op::Close { fd } });
        }
        self.host.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now) || (self.signal.is_some() && !self.closing)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.signal.is_none() && self.host.is_empty()
    }
    fn exit(&self) -> Option<skein_io::kernel::Exit> {
        self.host.exit()
    }
    fn operations(&self) -> u32 {
        self.host.operations() + 1
    }
}

// Agent roots bind to Start's workspace before the agent uses its supplied descriptor
// (temper/docs/design/domain/testing.md, 2.5).
#[derive(Default)]
struct Roots {
    next: u64,
    bindings: BTreeMap<u64, Binding>,
}
struct Binding {
    supplied: skein_sim::Handle,
    path: Option<Box<[u8]>>,
    opened: Option<skein_sim::Handle>,
}
thread_local! { static ROOTS: RefCell<Roots> = RefCell::new(Roots::default()); }
pub(crate) fn agent_roots(_spawn: &skein_io::kernel::Spawn) -> Vec<skein_world::StartupRoot> {
    if crate::lower::github::active() {
        return vec![skein_world::StartupRoot { name: b"worker".as_slice().into(), path: b"agent".as_slice().into() }];
    }
    let name = ROOTS.with(|roots| {
        let mut roots = roots.borrow_mut();
        let number = roots.next;
        roots.next += 1;
        format!("agent/{number}").into_bytes().into_boxed_slice()
    });
    vec![skein_world::StartupRoot { name: name.clone(), path: name }]
}
/// The supplied root is resolved from the same Start the hosted agent reads.
pub(crate) struct Rooted<H> {
    host: H,
    binding: Option<u64>,
    input: Option<Fd>,
    frame: Vec<u8>,
}
impl<H> Rooted<H> {
    pub fn new(host: H, inherited: &skein_world::Inherited) -> Self {
        let binding = inherited
            .roots
            .iter()
            .find_map(|(name, _)| name.strip_prefix(b"agent/"))
            .map(|number| std::str::from_utf8(number).expect("root number").parse().expect("root number"));
        Self {
            host,
            binding,
            input: binding.map(|_| inherited.pipes.iter().find(|(child, _)| *child == 0).expect("agent input").1),
            frame: Vec::new(),
        }
    }
    fn start(&mut self, complete: &Complete) {
        let Some(input) = self.input else {
            return;
        };
        if let (Op::PipeRead { fd, buf }, Ok(skein_io::kernel::Done::Count(count))) = (&complete.kind, &complete.result)
            && *fd == input
        {
            self.frame.extend_from_slice(&buf[..*count as usize]);
            while self.frame.len() >= 8 {
                let length = u32::from_be_bytes(self.frame[4..8].try_into().expect("Start length")) as usize;
                assert!(length <= crate::fixtures::agent_limits().channel.start_bytes as usize, "bounded Start");
                if self.frame.len() < length + 8 {
                    return;
                }
                if self.frame[..2] != 0x0100_u16.to_be_bytes() {
                    self.frame.drain(..8 + length);
                    continue;
                }
                let start = smith_channel::Start::decode(
                    &crate::fixtures::agent_limits().channel.bodies,
                    &mut skein_lib::Reader::new(&self.frame[8..8 + length]),
                )
                .expect("agent Start");
                let path = if let Some(workspace) = start.workspace() {
                    assert_eq!(workspace.directories().len(), 1, "one supplied workspace root");
                    workspace.directories().as_slice()[0]
                        .path()
                        .strip_prefix(b"/worker/")
                        .expect("worker workspace")
                        .into()
                } else {
                    b".".as_slice().into()
                };
                let number = self.binding.expect("pending root binding");
                self.input = None;
                ROOTS.with(|roots| {
                    roots.borrow_mut().bindings.get_mut(&number).expect("supplied root").path = Some(path);
                });
                self.frame.clear();
                break;
            }
        }
    }
}
impl<H: Host> Host for Rooted<H> {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host.state_digest()
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        for _ in 0..self.host.completions().len() {
            let complete = self.host.completions().pop().expect("completion count");
            self.start(&complete);
            self.host.completions().push(complete);
        }
        self.host.iterate(now, wall);
    }
    fn drain(&mut self) {
        self.host.drain();
        if let Some(number) = self.binding {
            for _ in 0..self.host.submissions().len() {
                let mut submit = self.host.submissions().pop().expect("submission count");
                if let Op::Spawn { spawn } = &mut submit.kind
                    && spawn.program.as_ref() == b"/bin/sh"
                {
                    ROOTS.with(|roots| {
                        let roots = roots.borrow();
                        let path = roots.bindings[&number].path.as_ref().expect("bound shell workspace");
                        spawn.dir = [path.as_ref(), b"/", spawn.dir.as_ref()].concat().into();
                    });
                }
                self.host.submissions().push(submit);
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host.is_empty()
    }
    fn operations(&self) -> u32 {
        self.host.operations()
    }
    fn exit(&self) -> Option<skein_io::kernel::Exit> {
        self.host.exit()
    }
}
pub(crate) struct Machine(pub skein_fake_machine::Machine);
impl skein_world::Machine for Machine {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, skein_io::kernel::Error> {
        if let Some(number) = path.strip_prefix(b"agent/") {
            let number = std::str::from_utf8(number).expect("root number").parse().expect("root number");
            let supplied = skein_sim::Handle::new(self.0.reopen_root(1).raw());
            ROOTS.with(|roots| {
                assert!(
                    roots
                        .borrow_mut()
                        .bindings
                        .insert(number, Binding { supplied, path: None, opened: None })
                        .is_none()
                );
            });
            return Ok(supplied);
        }
        if path == b"agent" {
            let worker = self.0.reopen_root(1);
            // The parent has no workspace; repository runs adopt its prepared directory.
            let path = if crate::lower::github::active() {
                let workspaces =
                    self.0.open(worker, b"workspaces", skein_fake_machine::How::Directory).expect("workspace scaffold");
                let entries = self.0.list(workspaces, 16, 4096).expect("scenario workspace names");
                // The GitHub scenarios prepare the chat first, then one change.
                assert!(entries.len() <= 2, "one chat and at most one change workspace");
                self.0.close(workspaces);
                entries.last().map_or_else(
                    || crate::fixtures::PREPARED_REPOSITORY.to_vec(),
                    |(_, key)| [b"workspaces/", key.as_ref(), b"/app"].concat(),
                )
            } else {
                crate::fixtures::PREPARED_REPOSITORY.to_vec()
            };
            let root = match self.0.open(worker, &path, skein_fake_machine::How::Directory) {
                Ok(root) => {
                    self.0.close(worker);
                    root
                }
                Err(skein_fake_machine::Refusal::NotFound) => worker,
                Err(why) => panic!("prepared agent directory: {why:?}"),
            };
            return Ok(skein_sim::Handle::new(root.raw()));
        }
        let index = match path {
            b"state" => 0,
            b"worker" => 1,
            b"client" => 2,
            b"/setup" => 3,
            _ => return Err(skein_io::kernel::Error::NotFound),
        };
        Ok(skein_sim::Handle::new(self.0.reopen_root(index).raw()))
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.release(root);
        self.0.close(skein_fake_machine::Opened::new(root.raw()));
    }
    fn cut(&mut self, cut: skein_world::Cut, held: &[skein_sim::Handle], seed: u64) {
        match cut {
            skein_world::Cut::Kill => {
                for handle in held {
                    self.release(*handle);
                    self.0.close(skein_fake_machine::Opened::new(handle.raw()));
                }
            }
            skein_world::Cut::PowerLoss => {
                ROOTS.with(|roots| roots.borrow_mut().bindings.clear());
                self.0.crash(seed);
            }
        }
    }
    fn step(&mut self, mut call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        use skein_sim::Ask;
        match &mut call.ask {
            Ask::Close { file } => self.release(*file),
            Ask::Open { root, .. } | Ask::Spawn { root, .. } => *root = self.resolve(*root),
            Ask::Rename { from_dir, to_dir, .. } => {
                *from_dir = self.resolve(*from_dir);
                *to_dir = self.resolve(*to_dir);
            }
            Ask::Remove { dir, .. } | Ask::MakeDirectory { dir, .. } | Ask::List { dir, .. } => {
                *dir = self.resolve(*dir);
            }
            Ask::Lock { file }
            | Ask::Read { file, .. }
            | Ask::Write { file, .. }
            | Ask::Append { file, .. }
            | Ask::Truncate { file, .. }
            | Ask::Sync { file }
            | Ask::Stat { file } => *file = self.resolve(*file),
        }
        skein_fake_machine::step(&mut self.0, call, answers);
    }
}
impl Machine {
    fn resolve(&mut self, supplied: skein_sim::Handle) -> skein_sim::Handle {
        ROOTS.with(|roots| {
            let mut roots = roots.borrow_mut();
            let Some(binding) = roots.bindings.values_mut().find(|binding| binding.supplied == supplied) else {
                return supplied;
            };
            if let Some(opened) = binding.opened {
                return opened;
            }
            let path = binding.path.as_ref().expect("workspace bound before file access");
            let opened = self
                .0
                .open(skein_fake_machine::Opened::new(supplied.raw()), path, skein_fake_machine::How::Directory)
                .expect("Start workspace root");
            let opened = skein_sim::Handle::new(opened.raw());
            binding.opened = Some(opened);
            opened
        })
    }
    fn release(&mut self, supplied: skein_sim::Handle) {
        ROOTS.with(|roots| {
            let mut roots = roots.borrow_mut();
            let number =
                roots.bindings.iter().find_map(|(number, binding)| (binding.supplied == supplied).then_some(*number));
            if let Some(number) = number
                && let Some(opened) = roots.bindings.remove(&number).expect("root binding").opened
            {
                self.0.close(skein_fake_machine::Opened::new(opened.raw()));
            }
        });
    }
}
pub(crate) fn fresh() -> skein_fake_machine::Machine {
    ROOTS.with(|roots| *roots.borrow_mut() = Roots::default());
    let mut machine = skein_fake_machine::Machine::for_user(1000);
    for items in [
        vec![],
        vec![
            skein_fake_machine::Item::directory(b"workspaces"),
            skein_fake_machine::Item::directory(b"trash"),
            skein_fake_machine::Item::directory(b"home"),
        ],
        vec![],
    ] {
        let root = machine.lay(&items);
        machine.close(root);
    }
    machine
}
