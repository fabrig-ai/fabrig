//! Full directory opens for worlds that exercise patch sets. Ordinary worlds
//! retain memory maps and direct positioned-file answers.
use jig_protocol_store::host::{Host, Lane, Submission};
use jig_protocol_store::{Event, Limits, Request, Root, Seeded, Store};
use skein_fake_machine::{How, Machine, serve};
use skein_io::kernel::Op;
use skein_lib::{Env, Queue, Token};
use skein_sim::{Config, Handle, Pid, Sim};

// The store world's file-driver queue capacity; each request settles before
// the next, with room for all completions of the eight driver flights.
const QUEUE: u32 = 128;

pub(super) struct Disk {
    machine: Machine,
    sim: Sim,
    pid: Pid,
    root: Root,
    host: Host,
}
impl Disk {
    pub fn new(limits: &Limits) -> Self {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(&[]);
        let (sim, pid, root, host) = drivers(limits, handle, 0);
        Self { machine, sim, pid, root, host }
    }
    pub fn root(&self) -> Root {
        self.root
    }
    pub fn seeded(&self) -> Seeded {
        self.host.seeded()
    }
    pub fn crash(&mut self, limits: &Limits, seed: u64) {
        self.machine.crash(seed);
        let handle = self.machine.reopen_root(0);
        (self.sim, self.pid, self.root, self.host) = drivers(limits, handle, seed);
    }
    pub fn unread_transcript(&mut self, task: u64) {
        let root = self.machine.reopen_root(0);
        let name = format!("transcripts/{:016x}/{task:016x}", task >> 12);
        let file = self.machine.open(root, name.as_bytes(), How::Write).expect("kept transcript");
        self.machine.truncate(file, 0).expect("writable transcript");
        self.machine.sync(file);
        self.machine.close(file);
        self.machine.close(root);
    }
    /// Stable file bytes beneath one directory, for generation-isolation stories.
    pub fn contents(&mut self, name: &[u8]) -> std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
        let root = self.machine.reopen_root(0);
        let mut result = std::collections::BTreeMap::new();
        collect(&mut self.machine, root, name, &mut result);
        self.machine.close(root);
        result
    }
    pub fn read(&mut self, name: &[u8]) -> Option<Vec<u8>> {
        let root = self.machine.reopen_root(0);
        let bytes = self.machine.open(root, name, How::ReadNoFollow).ok().map(|file| {
            let bytes = self.machine.read(file, 0, u32::MAX).expect("scenario file");
            self.machine.close(file);
            bytes
        });
        self.machine.close(root);
        bytes
    }
    /// Typed requests pass through the production host; fixture faults stay above it.
    pub fn request(&mut self, store: &mut Store, env: &Env<Limits>, request: Request, above: &mut Queue<Event>) {
        match request {
            Request::Open { owner, root, installation } => {
                self.host.adopt(root.fd, 0, owner, installation).expect("one fixture adoption");
            }
            request @ (Request::Switch { .. }
            | Request::Commit { .. }
            | Request::Restore { .. }
            | Request::Load { .. }
            | Request::Read { .. }
            | Request::Get { .. }
            | Request::Transcript { .. }
            | Request::Secrets { .. }
            | Request::Site { .. }) => self.host.down(store, env, request, above),
        }
        self.drive(store, env, above);
    }
    pub fn drive(&mut self, store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>) {
        for _ in 0..100_000 {
            self.host.iterate(store, env, above);
            let mut submissions = Queue::with_capacity(QUEUE);
            while let Some(Submission { lane, mut submit }) = self.host.pop_submission() {
                let tag = u8::try_from(lane.index() + 1).expect("three lanes");
                submit.op = Token::tagged(tag, submit.op.raw()).expect("local operation");
                if let Op::Cancel { target } = &mut submit.kind {
                    *target = Token::tagged(tag, target.raw()).expect("local cancellation");
                }
                submissions.push(submit);
            }
            self.sim.submit(self.pid, &mut submissions);
            serve(&mut self.machine, &mut self.sim);
            let mut completions = Queue::with_capacity(QUEUE);
            self.sim.reap(self.pid, &mut completions);
            while let Some(mut complete) = completions.pop() {
                let lane = *Lane::ALL.get(usize::from(complete.op.tag()) - 1).expect("driver tag");
                complete.op = Token::new(complete.op.untagged());
                if let Op::Cancel { target } = &mut complete.kind {
                    *target = Token::new(target.untagged());
                }
                self.host.complete(lane, complete);
            }
            if self.host.idle() && !self.host.work_pending(env.now) {
                return;
            }
        }
        panic!("calm directory host settles");
    }
}
fn drivers(limits: &Limits, handle: skein_fake_machine::Opened, seed: u64) -> (Sim, Pid, Root, Host) {
    let mut sim = Sim::new(seed, Config::calm());
    let pid = sim.spawn_process();
    let fd = sim.root(pid, Handle::new(handle.raw()));
    let host = Host::new(limits, seed);
    (sim, pid, Root { token: Token::new(1), fd }, host)
}

fn collect(
    machine: &mut Machine,
    root: skein_fake_machine::Opened,
    name: &[u8],
    into: &mut std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
) {
    let file = machine.open(root, name, How::ReadNoFollow).expect("generation path");
    if machine.stat(file).is == skein_fake_machine::Is::Directory {
        let entries = machine.list(file, 32, 8192).expect("bounded fixture directory");
        assert!(entries.len() < 32, "fixture fits one directory page");
        for (_, entry_name) in entries {
            let mut child = name.to_vec();
            child.push(b'/');
            child.extend_from_slice(&entry_name);
            collect(machine, root, &child, into);
        }
    } else {
        into.insert(name.to_vec(), machine.read(file, 0, u32::MAX).expect("generation bytes"));
    }
    machine.close(file);
}
