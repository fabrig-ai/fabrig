//! Store through file io, the simulated kernel and fake filesystem.

use std::collections::BTreeMap;

use skein_fake_machine::{How, Item, Machine, serve};
use skein_io::file;
use skein_io::file_layer::{self, FileIo};
use skein_io::kernel::{Complete, Fd, Submit};
use skein_kv::{Event, Limits, Request, Store};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_sim::{Config, Handle, Pid, Sim};

pub struct World {
    machine: Machine,
    pub sim: Sim,
    pid: Pid,
    pub root: Fd,
    pub io: FileIo,
    store: Store,
    pub env: Env<Limits>,
    pub above: Queue<Event>,
    pub below: Queue<file::Request>,
    file_events: Queue<file::Event>,
    subs: Queue<Submit>,
    completions: Queue<Complete>,
}

impl World {
    #[must_use]
    pub fn new(limits: Limits) -> World {
        Self::with_items(limits, &[])
    }

    #[must_use]
    pub fn with_items(limits: Limits, items: &[Item]) -> World {
        let mut machine = Machine::new();
        let handle = machine.lay(items);
        let mut sim = Sim::new(1, Config::calm());
        let pid = sim.spawn_process();
        let root = sim.root(pid, Handle::new(handle.raw()));
        World {
            machine,
            sim,
            pid,
            root,
            io: FileIo::new(8, limits.chunk, 1024, limits.deadline),
            store: Store::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(128),
            below: Queue::with_capacity(128),
            file_events: Queue::with_capacity(128),
            subs: Queue::with_capacity(128),
            completions: Queue::with_capacity(128),
        }
    }

    pub fn request(&mut self, request: Request) {
        self.start(request);
        self.drive(true);
    }

    pub fn request_no_fire(&mut self, request: Request) {
        self.start(request);
        self.drive(false);
    }

    pub fn start(&mut self, request: Request) {
        skein_kv::down(&mut self.store, &self.env, request, &mut self.above, &mut self.below);
    }

    pub fn fire_once(&mut self) {
        skein_kv::fire(&mut self.store, &self.env, &mut self.above, &mut self.below);
        self.drive(false);
    }

    pub fn drive(&mut self, auto_fire: bool) {
        for _ in 0..10_000 {
            if !self.step(auto_fire) {
                return;
            }
        }
        panic!("store did not quiesce");
    }

    pub fn step(&mut self, auto_fire: bool) -> bool {
        let mut progress = false;
        if self.io.is_due(self.env.now) {
            file_layer::expire(&mut self.io, self.env.now, &mut self.subs);
            progress = true;
        }
        if self.io.takes()
            && let Some(request) = self.below.pop()
        {
            file_layer::down(&mut self.io, self.env.now, request, &mut self.file_events, &mut self.subs);
            progress = true;
        }
        if !self.subs.is_empty() {
            self.sim.submit(self.pid, &mut self.subs);
            serve(&mut self.machine, &mut self.sim);
            progress = true;
        }
        self.sim.reap(self.pid, &mut self.completions);
        while let Some(complete) = self.completions.pop() {
            file_layer::up(&mut self.io, complete, &mut self.file_events, &mut self.subs);
            progress = true;
        }
        while let Some(event) = self.file_events.pop() {
            skein_kv::up(&mut self.store, &self.env, event, &mut self.above, &mut self.below);
            progress = true;
        }
        if auto_fire && self.io.takes() && self.below.is_empty() && skein_kv::is_ready(&self.store, &self.env.limits) {
            skein_kv::fire(&mut self.store, &self.env, &mut self.above, &mut self.below);
            progress = true;
        }
        progress
    }

    pub fn crash(&mut self, seed: u64) {
        self.machine.crash(seed);
        let handle = self.machine.reopen_root(0);
        self.sim = Sim::new(seed, Config::calm());
        self.pid = self.sim.spawn_process();
        self.root = self.sim.root(self.pid, Handle::new(handle.raw()));
        self.io = FileIo::new(8, self.env.limits.chunk, 1024, self.env.limits.deadline);
        self.store = Store::new(&self.env.limits);
        self.above = Queue::with_capacity(128);
        self.below = Queue::with_capacity(128);
        self.file_events = Queue::with_capacity(128);
        self.subs = Queue::with_capacity(128);
        self.completions = Queue::with_capacity(128);
    }

    pub fn reopen(&mut self) {
        self.store = Store::new(&self.env.limits);
        self.io = FileIo::new(8, self.env.limits.chunk, 1024, self.env.limits.deadline);
        self.request(Request::Open { owner: Token::new(1), root: self.root });
    }

    /// Every file in the store's root, by name, with its bytes.
    pub fn files(&mut self) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
        // A fresh handle lists from the beginning, independent of recovery's cursor.
        let root = self.machine.reopen_root(0);
        let mut files = BTreeMap::new();
        loop {
            // The file driver's directory page sizes, with room for any name.
            let entries = self.machine.list(root, 16, 1024).expect("the store's root lists");
            if entries.is_empty() {
                break;
            }
            for (_, name) in entries {
                let file = self.machine.open(root, &name, How::ReadNoFollow).expect("a file in the store's root");
                let len = u32::try_from(self.machine.stat(file).size).expect("the fake disk's file limit fits u32");
                let bytes = self.machine.read(file, 0, len).expect("an open store file reads");
                self.machine.close(file);
                files.insert(name, bytes.into_boxed_slice());
            }
        }
        self.machine.close(root);
        files
    }

    pub fn pop(&mut self) -> Event {
        self.above.pop().expect("one owner event")
    }
}
