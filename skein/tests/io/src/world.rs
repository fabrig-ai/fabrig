//! The world: one thread, one loop (testing-strategy.md, 1), driving each
//! process's io and its scripted owner over the simulator, iteration by
//! iteration, as a service's loop drives its layers (programming-model.md,
//! section 2; testing/simulator.md, 3), and checking as it goes:
//!
//! - **contracts:** each entry point within its `MAX_OUT`; io's contract with
//!   the owner, in the [`Ledger`];
//! - **the scenario's expectations,** in the [`Referee`];
//! - **invariants once settled:** every slab empty and nothing in flight in
//!   io, nothing in flight in the simulator and every descriptor closed, every
//!   entity told `Closed`;
//! - **replay:** a run returns its trace, the simulator's records and io's
//!   events, for a test to compare with the same seed's.

use std::cell::RefCell;
use std::rc::Rc;

use skein_io::kernel::{Complete, Op, Submit};
use skein_io::{Event, Io, Limits, Request, stage};
use skein_lib::{Env, Queue, Time, Wall};
use skein_sim::{Config, Entry, Pid, Sim};
use skein_world::{Host, Memory};

use crate::ledger::Ledger;
use crate::owner::{Directory, Owner};
use crate::referee::Referee;

/// The most iterations a world runs before it is declared stuck.
const STEPS: usize = 200_000;

/// Room for what a process's loop holds between its stages.
const ROOM: u32 = 64;

/// One process: io, the owner above it, and the loop's queues between them.
#[derive(Debug)]
pub struct Proc {
    pub io: Io,
    pub env: Env<Limits>,
    pub owner: Owner,
    pub ledger: Ledger,
    pub completions: Queue<Complete>,
    pub subs: Queue<Submit>,
    events: Queue<Event>,
    requests: Queue<Request>,
    /// What io told, in order, for replay.
    pub log: Vec<String>,
    /// Iterations whose down stage stopped with requests waiting, as io
    /// could hold no more refusals (`Io::takes`).
    pub held_back: u32,
}

impl Proc {
    #[must_use]
    pub fn new(limits: Limits, owner: Owner) -> Proc {
        let mut proc = Proc {
            io: Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            owner,
            ledger: Ledger::default(),
            completions: Queue::with_capacity(ROOM),
            subs: Queue::with_capacity(ROOM),
            events: Queue::uncapped(),
            requests: Queue::with_capacity(ROOM),
            log: Vec::new(),
            held_back: 0,
        };
        proc.owner.start(&mut proc.requests);
        for request in &proc.requests {
            proc.ledger.asked(request);
        }
        proc
    }

    /// One iteration of the process's loop, between its reap and its submit:
    /// io's up stage, the owner's stage, io's down stage, the reclaim point.
    pub fn iterate(&mut self, now: Time, wall: Wall, directory: &mut Directory) {
        self.env.now = now;
        self.env.wall = wall;
        stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.subs, now);
        // The owner's stage.
        let queued = self.requests.len();
        while let Some(event) = self.events.pop() {
            self.ledger.event(&event);
            self.log.push(format!("{} {event:?}", now.as_nanos()));
            self.owner.on(now, event, directory, &mut self.requests);
        }
        self.owner.tick(now, directory, &mut self.requests);
        for request in self.requests.iter().skip(usize::try_from(queued).expect("small request queue")) {
            self.ledger.asked(request);
        }
        // io's down stage.
        let before = self.requests.len();
        stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.subs);
        self.ledger.taken(before - self.requests.len());
        if !self.io.takes() && !self.requests.is_empty() {
            self.held_back += 1;
        }
        // The accept batch (runtime/io/sockets.md, 2): no more accepts armed in an
        // iteration than it allows, over every listener.
        let mut accepts = 0;
        for submit in &self.subs {
            if let Op::Accept { .. } = submit.kind {
                accepts += 1;
            }
        }
        assert!(accepts <= self.env.limits.accepts, "{accepts} accepts armed in one iteration, past the batch");
        self.io.reclaim();
    }

    /// Whether the loop has work now, without the kernel or time.
    #[must_use]
    pub fn busy(&self, now: Time) -> bool {
        self.io.is_ready()
            || self.io.is_due(now)
            || !self.completions.is_empty()
            || !self.subs.is_empty()
            || !self.requests.is_empty()
    }
}

/// A process with the world's directory visible within each turn (runtime/io/README.md, 5).
pub(crate) struct Hosted {
    pub proc: Proc,
    pub directory: Rc<RefCell<Directory>>,
}

impl Host for Hosted {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.proc.iterate(now, wall, &mut self.directory.borrow_mut());
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.proc.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.proc.subs
    }
    fn work_pending(&self, now: Time) -> bool {
        self.proc.busy(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.proc.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.proc.owner.next_deadline(self.proc.env.now)
    }
    fn is_empty(&self) -> bool {
        self.proc.owner.done() && self.proc.io.is_empty()
    }
    fn operations(&self) -> u32 {
        skein_io::operations(&self.proc.env.limits).expect("small io limits")
    }
}

/// A world: the simulator, its processes, and the referee.
pub struct World {
    pub sim: Sim,
    pub procs: Vec<Proc>,
    /// Each process's in the simulator.
    pids: Vec<Pid>,
    pub referee: Referee,
    pub directory: Directory,
}

/// What a run left, for a test to compare or count.
#[derive(Debug)]
pub struct Outcome {
    pub trace: Vec<Entry>,
    pub logs: Vec<Vec<String>>,
    pub iterations: u32,
    /// Where each iteration began in the trace, to tell which iteration an
    /// entry belongs to.
    pub marks: Vec<usize>,
    /// Each process's iterations held back for want of room for a refusal.
    pub held_back: Vec<u32>,
}

impl World {
    #[must_use]
    pub fn new(seed: u64, config: Config, referee: Referee) -> World {
        World { sim: Sim::new(seed, config), procs: Vec::new(), pids: Vec::new(), referee, directory: Directory::new() }
    }

    /// Adds a process running io under `limits`, with `owner` above it.
    pub fn spawn(&mut self, limits: Limits, owner: Owner) -> usize {
        self.pids.push(self.sim.spawn_process());
        self.procs.push(Proc::new(limits, owner));
        self.procs.len() - 1
    }

    /// Runs the world until the referee passed and everything settled, then
    /// checks the invariants of a settled world.
    #[must_use]
    pub fn run(self) -> Outcome {
        let directory = Rc::new(RefCell::new(self.directory));
        let mut world = skein_world::World::new(
            self.sim.seed(),
            *self.sim.config(),
            crate::referee::Judge::new(self.referee),
            Memory::Unchecked,
        );
        for proc in self.procs {
            world.spawn(|| Hosted { proc, directory: directory.clone() });
        }
        let mut sim = Some(self.sim);
        let mut marks = Vec::new();
        let mut outcome = world.run_with_inputs(|current, pids, _| {
            if let Some(sim) = sim.take() {
                *current = sim;
                assert_eq!(pids, self.pids, "the world's process identities stay in admission order");
            }
            assert!(marks.len() + 1 < STEPS, "the world settles\n{}", current.render_trace());
            marks.push(current.trace().len());
        });
        for host in &outcome.procs {
            host.proc.ledger.settled();
        }
        Outcome {
            trace: std::mem::take(&mut outcome.trace),
            held_back: outcome.procs.iter().map(|host| host.proc.held_back).collect(),
            logs: outcome.procs.iter_mut().map(|host| std::mem::take(&mut host.proc.log)).collect(),
            iterations: outcome.iterations,
            marks,
        }
    }
}
