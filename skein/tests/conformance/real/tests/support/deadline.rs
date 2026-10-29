//! Real-clock harness waits (skein/docs/foundation/testing-strategy.md, 2.10).

use skein_io::{self as io, Event, Request};
use skein_lib::{Duration, Env, Queue, Token};
use skein_shell::{Clock, Kernel, Wait};
use skein_world::{Deadline, Expired};

/// Real-clock waits and their scheduler watches
/// (skein/docs/foundation/testing-strategy.md, 2.10).
/// Each `Deadline` has one process watch; keep the test and waited-on process
/// separate so registering a child cannot replace the test's helper threads.
pub struct Patience {
    process: Deadline,
    own: Deadline,
    watching: bool,
}

impl Patience {
    pub fn new(what: &'static str, patience: Duration) -> Self {
        // Arm the process watch first so it has passed whenever own has.
        let process = Deadline::arm(what, patience);
        let mut own = Deadline::arm(what, patience);
        own.watch_process(std::process::id());
        Self { process, own, watching: false }
    }

    pub fn watch_process(&mut self, pid: u32) {
        self.process.watch_process(pid);
        self.watching = true;
    }

    pub fn watch_spawn(&mut self, complete: &io::kernel::Complete) {
        if let Ok(io::kernel::Done::Spawned { pidfd, .. }) = complete.result {
            self.watch_process(skein_shell::child_id(pidfd).expect("the spawned child's PID"));
        }
    }

    pub fn passed(&self) -> bool {
        self.own.passed()
    }

    pub fn until(&self) -> skein_lib::Time {
        self.own.until()
    }

    pub fn remaining(&self) -> Duration {
        self.own.remaining()
    }

    pub fn expired(&self) -> Expired {
        match self.own.expired() {
            Expired::Starved(starved) => Expired::Starved(starved),
            failed @ Expired::Failed(_) => {
                if self.watching {
                    self.process.expired()
                } else {
                    failed
                }
            }
        }
    }
}

/// End io's owned entities and drain the kernel before a starved exit.
/// Spawn and accept events discovered while draining are aborted too.
pub fn end_io(
    kernel: &mut Kernel,
    state: &mut io::Io,
    limits: io::Limits,
    submissions: &mut Queue<io::kernel::Submit>,
    completions: &mut Queue<io::kernel::Complete>,
    events: &mut Queue<Event>,
    entities: &[Token],
) {
    let clock = Clock::new();
    let now = clock.now();
    let env = Env { now: now.now, wall: now.wall, limits };
    for entity in entities {
        io::down(state, &env, Request::Abort { entity: *entity }, submissions);
    }
    loop {
        let now = clock.now();
        let env = Env { now: now.now, wall: now.wall, limits };
        kernel.reap(completions);
        while let Some(complete) = completions.pop() {
            io::up(state, &env, complete, events, submissions);
        }
        while let Some(event) = events.pop() {
            match event {
                Event::Spawned { child, pipes, .. } => {
                    io::down(state, &env, Request::Abort { entity: child }, submissions);
                    for pipe in pipes {
                        io::down(state, &env, Request::Abort { entity: pipe }, submissions);
                    }
                }
                Event::Accepted { socket, .. } | Event::Connecting { socket, .. } => {
                    io::down(state, &env, Request::Abort { entity: socket }, submissions);
                }
                Event::Listening { listener, .. } => {
                    io::down(state, &env, Request::Abort { entity: listener }, submissions);
                }
                Event::Connected { .. }
                | Event::Stream { .. }
                | Event::Output { .. }
                | Event::Exited { .. }
                | Event::Usage { .. }
                | Event::Resized
                | Event::Window { .. }
                | Event::Shutdown { .. }
                | Event::Failed { .. }
                | Event::Closed { .. } => {}
            }
        }
        while state.is_ready() {
            io::resume(state, &env, events, submissions);
        }
        while state.is_due(now.now) {
            io::fire(state, &env, events, submissions);
        }
        state.reclaim();
        if state.is_empty() && kernel.in_flight() == 0 && submissions.is_empty() {
            return;
        }
        let wait = if state.is_ready() || !events.is_empty() {
            Wait::No
        } else {
            Wait::Until(now.now.saturating_add(Duration::from_millis(5)))
        };
        kernel.submit(submissions, wait);
    }
}

/// One driver's submission, completion and event storage.
pub fn records(room: u32) -> (Queue<io::kernel::Submit>, Queue<io::kernel::Complete>, Queue<Event>) {
    (Queue::with_capacity(room), Queue::with_capacity(room), Queue::with_capacity(room))
}
