//! Calm file drivers for the unseeded-store catch-up fixture.
use jig_protocol_store::{self as store, Arrived, Event, Io, Request, Root, Store};
use skein_fake_machine::{Machine, serve};
use skein_io::{
    file,
    file_layer::{FileIo, Limits, step},
    kernel::{Complete, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_sim::{Config, Handle, Pid, Sim};

pub(in crate::tests) struct Disk {
    machine: Machine,
    sim: Sim,
    pid: Pid,
    root: Root,
    io: [FileIo; 3],
    files: Limits,
    env: Env<store::Limits>,
    seeded: store::Seeded,
}
impl Disk {
    pub fn new() -> Self {
        Self::with_items(&[])
    }
    pub fn with_items(items: &[skein_fake_machine::Item]) -> Self {
        let mut limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).store;
        limits.store.budget = 4_194_304;
        limits.store.ops = 256;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let files = Limits {
            files: 8,
            read: limits.store.chunk,
            whole: limits.store.chunk,
            entries: 32,
            timeout: limits.store.deadline,
        };
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(items);
        let mut sim = Sim::new(1, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        let mut io = core::array::from_fn(|_| {
            let mut io = FileIo::with_limits(&files).with_effective_user(0);
            assert_eq!(io.adopt_root(fd), Some(Token::new(1)));
            io
        });
        let [a, b, c] = &mut io;
        let seeded = store::seed([a, b, c], 1);
        Self { machine, sim, pid, root: Root { token: Token::new(1), fd }, io, files, env, seeded }
    }
    pub fn active(&mut self) -> Vec<u8> {
        let root = self.machine.reopen_root(0);
        let opened = self.machine.open(root, b"active", skein_fake_machine::How::ReadNoFollow).unwrap();
        let bytes = self.machine.read(opened, 0, 64).unwrap();
        self.machine.close(opened);
        self.machine.close(root);
        bytes
    }
    pub fn contents(&mut self) -> Vec<(Vec<u8>, u32, Vec<u8>)> {
        let root = self.machine.reopen_root(0);
        let mut found = Vec::new();
        self.walk(root, b"", &mut found);
        self.machine.close(root);
        found.sort();
        found
    }
    fn walk(&mut self, root: skein_fake_machine::Opened, path: &[u8], found: &mut Vec<(Vec<u8>, u32, Vec<u8>)>) {
        use skein_fake_machine::{How, Is};
        let dir = self.machine.open(root, if path.is_empty() { b"." } else { path }, How::DirectoryNoFollow).unwrap();
        loop {
            let entries = self.machine.list(dir, 32, 8192).unwrap();
            if entries.is_empty() {
                break;
            }
            for (kind, name) in entries {
                let path = if path.is_empty() { name.to_vec() } else { [path, b"/", &name].concat() };
                let file = self.machine.open(root, &path, How::ReadNoFollow).unwrap();
                let mode = self.machine.stat(file).mode;
                let bytes = if kind == Is::File { self.machine.read(file, 0, u32::MAX).unwrap() } else { vec![] };
                self.machine.close(file);
                if kind != Is::File || !bytes.is_empty() {
                    found.push((path.clone(), mode, bytes));
                }
                if kind == Is::Directory {
                    self.walk(root, &path, found);
                }
            }
        }
        self.machine.close(dir);
    }
    pub fn restart(&mut self) {
        self.machine.crash(1);
        let handle = self.machine.reopen_root(0);
        self.sim = Sim::new(1, Config::calm());
        self.pid = self.sim.spawn_process();
        let fd = self.sim.root(self.pid, Handle::new(handle.raw()));
        self.root = Root { token: Token::new(1), fd };
        self.io = core::array::from_fn(|_| {
            let mut io = FileIo::with_limits(&self.files).with_effective_user(0);
            assert_eq!(io.adopt_root(fd), Some(Token::new(1)));
            io
        });
        let [a, b, c] = &mut self.io;
        self.seeded = store::seed([a, b, c], 1);
    }
    pub fn limits(&self) -> store::Limits {
        self.env.limits
    }
    pub fn seeded(&self) -> store::Seeded {
        self.seeded
    }
    pub fn open(&mut self, store: &mut Store) -> Vec<Event> {
        self.request(store, Request::Open { owner: Token::new(1), root: self.root, installation: [0; 16] })
    }
    pub fn request(&mut self, store: &mut Store, request: Request) -> Vec<Event> {
        let mut above = Queue::uncapped();
        let mut below =
            store::Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() };
        store::down(store, &self.env, request, &mut above, &mut below);
        for _ in 0_u32..10_000 {
            store::fire(store, &self.env, &mut above, &mut below);
            let next = below
                .store
                .pop()
                .map(|r| (0, r))
                .or_else(|| below.secrets.pop().map(|r| (1, r)))
                .or_else(|| below.transcripts.pop().map(|r| (2, r)));
            if let Some((lane, request)) = next {
                let arrived = self.file(lane, request);
                let io = match lane {
                    0 => Io::Store(arrived),
                    1 => Io::Secrets(arrived),
                    2 => Io::Transcripts(arrived),
                    _ => unreachable!(),
                };
                store::up(store, &self.env, io, &mut above, &mut below);
            } else if !store::sizes::is_patching(store) {
                let mut events = Vec::new();
                while let Some(event) = above.pop() {
                    events.push(event);
                }
                return events;
            } else if !above.is_empty() {
                // A written job waits for its owner's answer.
                let mut events = Vec::new();
                while let Some(event) = above.pop() {
                    events.push(event);
                }
                return events;
            }
        }
        panic!("calm store settles");
    }
    fn file(&mut self, lane: usize, request: file::Request) -> Arrived {
        let io = &mut self.io[lane];
        assert!(io.takes());
        let env = self.env.with(self.files);
        let mut events = Queue::uncapped();
        let mut subs: Queue<Submit> = Queue::uncapped();
        let mut completions: Queue<Complete> = Queue::uncapped();
        step::down(io, &env, request, &mut events, &mut subs);
        for _ in 0_u32..100 {
            self.sim.submit(self.pid, &mut subs);
            serve(&mut self.machine, &mut self.sim);
            self.sim.reap(self.pid, &mut completions);
            while let Some(complete) = completions.pop() {
                step::up(io, &env, complete, &mut events, &mut subs);
            }
            if io.takes() {
                break;
            }
        }
        assert!(io.takes(), "calm file operation settles");
        let event = events.pop().expect("one file terminal");
        assert!(events.is_empty());
        let fd = if let file::Event::Opened { file, .. } = &event { io.descriptor(*file) } else { None };
        Arrived { event, fd }
    }
    pub fn map(&mut self, generation: u64) -> skein_kv::Store {
        let root = self.file(
            0,
            file::Request::OpenDirectory {
                owner: Token::new(1),
                root: self.root.fd,
                name: format!("generations/{generation}/store").into_bytes().into(),
                no_follow: true,
            },
        );
        let mut map = skein_kv::Store::new(&self.env.limits.store);
        let events = self.kv(&mut map, skein_kv::Request::Open { owner: Token::new(1), root: root.fd.unwrap() });
        let [skein_kv::Event::Opened { .. }] = events.as_slice() else { panic!("fixture map opens: {events:?}") };
        map
    }
    pub fn kv(&mut self, map: &mut skein_kv::Store, request: skein_kv::Request) -> Vec<skein_kv::Event> {
        let env = self.env.with(self.env.limits.store);
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        skein_kv::down(map, &env, request, &mut above, &mut below);
        for _ in 0_u32..10_000 {
            let Some(request) = below.pop() else {
                let mut events = Vec::new();
                while let Some(event) = above.pop() {
                    events.push(event);
                }
                return events;
            };
            let arrived = self.file(0, request);
            skein_kv::up(map, &env, arrived.event, &mut above, &mut below);
        }
        panic!("fixture map settles");
    }
}
