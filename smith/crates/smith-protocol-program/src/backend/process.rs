//! The program child and its pipes (smith/docs/design/protocol/programs.md, 5.6–5.8).
//! A pending spawn owns no handles; a live child owns its remaining pipes.
//! Exit retains its code and drain grace until closure settles every handle.
//! This machine knows no completion, window, MCP listener or stop policy.

use core::mem;
use skein_io::{self as io, kernel};
use skein_lib::{Queue, Time, Token};

use super::Pipes;

pub(super) struct Process {
    state: State,
}

struct Resources {
    child: Option<Token>,
    pipes: Pipes,
}

enum State {
    Dormant,
    Spawning,
    Live { resources: Resources },
    Draining { resources: Resources, exit: u32, grace: Grace },
    Settled { exit: Option<u32> },
}

enum Grace {
    Waiting { until: Time },
    Expired,
}

impl Process {
    pub fn new() -> Self {
        Self { state: State::Dormant }
    }

    pub fn reset(&mut self) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Dormant | State::Settled { .. } => State::Dormant,
            State::Spawning | State::Live { .. } | State::Draining { .. } => {
                unreachable!("reset follows resource settlement")
            }
        };
        follow(self);
    }

    pub fn spawn(&mut self) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Dormant | State::Settled { .. } => State::Spawning,
            State::Spawning | State::Live { .. } | State::Draining { .. } => {
                unreachable!("the previous invocation settles before another spawn")
            }
        };
        follow(self);
    }

    pub fn spawned(&mut self, child: Token, pipes: Pipes) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Spawning => spawned(child, pipes),
            State::Dormant | State::Live { .. } | State::Draining { .. } | State::Settled { .. } => {
                unreachable!("Spawned ends one outstanding spawn")
            }
        };
        follow(self);
    }

    pub fn failed(&mut self) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Spawning => State::Settled { exit: None },
            state @ (State::Dormant | State::Live { .. } | State::Draining { .. } | State::Settled { .. }) => state,
        };
        follow(self);
    }

    pub fn exited(&mut self, exit: kernel::Exit, until: Time, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Live { resources } | State::Draining { resources, .. } => exited(resources, exit, until, below),
            State::Dormant | State::Spawning | State::Settled { .. } => {
                unreachable!("Exited belongs to a spawned child")
            }
        };
        follow(self);
    }

    pub fn closed(&mut self, owner: Token, child_owner: Token) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Live { resources } => State::Live { resources: closed(resources, owner, child_owner) },
            State::Draining { resources, exit, grace } => {
                State::Draining { resources: closed(resources, owner, child_owner), exit, grace }
            }
            state @ (State::Dormant | State::Spawning | State::Settled { .. }) => state,
        };
        follow(self);
    }

    /// Abort residual streams once, after an exited child's drain grace.
    pub fn drain(&mut self, now: Time, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Dormant);
        self.state = match state {
            State::Draining { resources, exit, grace } => drained(resources, exit, grace, now, below),
            state @ (State::Dormant | State::Spawning | State::Live { .. } | State::Settled { .. }) => state,
        };
        follow(self);
    }

    pub fn child(&self) -> Option<Token> {
        match &self.state {
            State::Live { resources } | State::Draining { resources, .. } => resources.child,
            State::Dormant | State::Spawning | State::Settled { .. } => None,
        }
    }

    pub fn pipes(&self) -> Pipes {
        match &self.state {
            State::Live { resources } | State::Draining { resources, .. } => resources.pipes,
            State::Dormant | State::Spawning | State::Settled { .. } => Pipes::CLOSED,
        }
    }

    pub fn spawning(&self) -> bool {
        match self.state {
            State::Spawning => true,
            State::Dormant | State::Live { .. } | State::Draining { .. } | State::Settled { .. } => false,
        }
    }

    pub fn exit(&self) -> Option<u32> {
        match self.state {
            State::Draining { exit, .. } => Some(exit),
            State::Settled { exit } => exit,
            State::Dormant | State::Spawning | State::Live { .. } => None,
        }
    }

    pub fn next_deadline(&self) -> Option<Time> {
        match &self.state {
            State::Draining { grace: Grace::Waiting { until }, .. } => Some(*until),
            State::Draining { grace: Grace::Expired, .. }
            | State::Dormant
            | State::Spawning
            | State::Live { .. }
            | State::Settled { .. } => None,
        }
    }

    pub fn settled(&self) -> bool {
        match self.state {
            State::Dormant | State::Settled { .. } => true,
            State::Spawning | State::Live { .. } | State::Draining { .. } => false,
        }
    }

    #[cfg(test)]
    pub fn attached(child: Option<Token>, pipes: Pipes, exit: Option<u32>) -> Self {
        let resources = Resources { child, pipes };
        let state = match exit {
            Some(exit) => State::Draining { resources, exit, grace: Grace::Expired },
            None => State::Live { resources },
        };
        let mut process = Self { state };
        follow(&mut process);
        process
    }
}

fn spawned(child: Token, pipes: Pipes) -> State {
    State::Live { resources: Resources { child: Some(child), pipes } }
}

fn exited(resources: Resources, exit: kernel::Exit, until: Time, below: &mut Queue<io::Request>) -> State {
    let exit = match exit {
        kernel::Exit::Code(code) => u32::from(code),
        kernel::Exit::Signal(signal) => 128_u32.checked_add(signal).expect("signal code"),
    };
    if let Some(child) = resources.child {
        below.push(io::Request::Close { entity: child });
    }
    if let Some(input) = resources.pipes.input {
        below.push(io::Request::Close { entity: input });
    }
    State::Draining { resources, exit, grace: Grace::Waiting { until } }
}

fn closed(mut resources: Resources, owner: Token, child_owner: Token) -> Resources {
    if owner == child_owner {
        resources.child = None;
    }
    resources.pipes.closed(owner);
    resources
}

fn drained(resources: Resources, exit: u32, grace: Grace, now: Time, below: &mut Queue<io::Request>) -> State {
    let grace = match grace {
        Grace::Waiting { until } if now >= until => {
            for pipe in resources.pipes.all().into_iter().flatten() {
                below.push(io::Request::Abort { entity: pipe });
            }
            if let Some(child) = resources.child {
                below.push(io::Request::Close { entity: child });
            }
            Grace::Expired
        }
        grace @ (Grace::Waiting { .. } | Grace::Expired) => grace,
    };
    State::Draining { resources, exit, grace }
}

/// The last resource terminal settles the invocation and releases its grace.
fn follow(process: &mut Process) {
    let state = mem::replace(&mut process.state, State::Dormant);
    process.state = match state {
        State::Live { resources } if resources.child.is_none() && resources.pipes == Pipes::CLOSED => {
            State::Settled { exit: None }
        }
        State::Draining { resources, exit, .. } if resources.child.is_none() && resources.pipes == Pipes::CLOSED => {
            State::Settled { exit: Some(exit) }
        }
        state @ (State::Dormant
        | State::Spawning
        | State::Live { .. }
        | State::Draining { .. }
        | State::Settled { .. }) => state,
    };
}

#[cfg(test)]
mod tests;
