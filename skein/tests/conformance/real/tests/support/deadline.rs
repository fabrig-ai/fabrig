//! Real-clock harness waits (skein/docs/foundation/testing-strategy.md, 2.10).

use skein_io::{self as io, Event, Request, stage};
use skein_lib::{Duration, Env, Queue, Token};
use skein_shell::{Clock, Kernel, Wait};
use skein_world::{Deadline, Expired};

/// Real-clock waits and their scheduler watches
/// (skein/docs/foundation/testing-strategy.md, 2.10).
/// One deadline watches the test and waited-on processes, including their
/// descendants; the largest single runnable wait decides starvation.
pub struct Patience {
    deadline: Deadline,
}

impl Patience {
    pub fn new(what: &'static str, patience: Duration) -> Self {
        let mut deadline = Deadline::arm(what, patience);
        deadline.watch_process(std::process::id());
        Self { deadline }
    }

    pub fn watch_process(&mut self, pid: u32) {
        self.deadline.watch_process(pid);
    }

    pub fn watch_spawn(&mut self, complete: &io::kernel::Complete) {
        if let Ok(io::kernel::Done::Spawned { pidfd, .. }) = complete.result {
            self.watch_process(skein_shell::child_id(pidfd).expect("the spawned child's PID"));
        }
    }

    pub fn passed(&self) -> bool {
        self.deadline.passed()
    }

    pub fn until(&self) -> skein_lib::Time {
        self.deadline.until()
    }

    pub fn remaining(&self) -> Duration {
        self.deadline.remaining()
    }

    pub fn expired(&self) -> Expired {
        self.deadline.expired()
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
    let mut requests = Queue::uncapped();
    for entity in entities {
        requests.push(Request::Abort { entity: *entity });
    }
    stage::down(state, &env, &mut requests, submissions);
    loop {
        let now = clock.now();
        let env = Env { now: now.now, wall: now.wall, limits };
        kernel.reap(completions);
        stage::up(state, &env, completions, events, submissions, now.now);
        while let Some(event) = events.pop() {
            match event {
                Event::Spawned { child, pipes, .. } => {
                    requests.push(Request::Abort { entity: child });
                    for pipe in pipes {
                        requests.push(Request::Abort { entity: pipe });
                    }
                }
                Event::Accepted { socket, .. } | Event::Connecting { socket, .. } => {
                    requests.push(Request::Abort { entity: socket });
                }
                Event::Listening { listener, .. } => {
                    requests.push(Request::Abort { entity: listener });
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
        stage::down(state, &env, &mut requests, submissions);
        state.reclaim();
        if state.is_empty() && kernel.in_flight() == 0 && submissions.is_empty() {
            return;
        }
        let wait = if state.is_ready() || !events.is_empty() || !completions.is_empty() || !requests.is_empty() {
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
