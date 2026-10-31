//! Stop results, finishing intent and signal escalation (smith/docs/design/protocol/programs.md, 5.7–5.9).
//! Waiting states own their result action and grace; escalation retains that
//! action until its result arrives. No startup or completion alarm lives here.
//! `begin` chooses the result action, `result` settles it, and `terminate`
//! advances group signals. `spawned` gives a late child its first signal grace.

use super::process::Process;
use core::mem;
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Queue, Time};

/// The domain's requested continuation after the current stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Intent {
    /// The next completion continues the conversation.
    Ordinary,
    /// The owner asked to interrupt once startup or replacement settles.
    YieldRequested,
    /// The finishing note uses the retained program process.
    Finishing,
}

/// What the interrupted program result releases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    /// Close input for cancellation or window replacement.
    Close,
    /// Close input after recording a held wait.
    Park,
    /// Send the retained finishing note.
    Finish,
    /// Return the interrupted completion to the domain.
    Yield,
}

#[derive(Clone, Copy)]
enum Pending {
    Result(Action),
    Done,
}

/// A stop result and its signal escalation for one process invocation.
pub(super) struct Stopping {
    state: State,
}

enum State {
    Open(Intent),
    Waiting { intent: Intent, action: Action, until: Option<Time> },
    Grace { intent: Intent, until: Option<Time> },
    Terminating { intent: Intent, pending: Pending, until: Option<Time> },
    Killing { intent: Intent, pending: Pending, until: Option<Time> },
}

impl Stopping {
    pub fn new() -> Self {
        Self { state: State::Open(Intent::Ordinary) }
    }

    pub fn intent(&self) -> Intent {
        match self.state {
            State::Open(intent)
            | State::Waiting { intent, .. }
            | State::Grace { intent, .. }
            | State::Terminating { intent, .. }
            | State::Killing { intent, .. } => intent,
        }
    }

    pub fn action(&self) -> Option<Action> {
        match self.state {
            State::Waiting { action, .. }
            | State::Terminating { pending: Pending::Result(action), .. }
            | State::Killing { pending: Pending::Result(action), .. } => Some(action),
            State::Open(_)
            | State::Grace { .. }
            | State::Terminating { pending: Pending::Done, .. }
            | State::Killing { pending: Pending::Done, .. } => None,
        }
    }

    pub fn waiting(&self) -> bool {
        self.action().is_some()
    }

    pub fn intend(&mut self, intent: Intent) {
        let state = mem::replace(&mut self.state, State::Open(Intent::Ordinary));
        self.state = match state {
            State::Open(_) => State::Open(intent),
            State::Waiting { action, until, .. } => State::Waiting { intent, action, until },
            State::Grace { until, .. } => State::Grace { intent, until },
            State::Terminating { pending, until, .. } => State::Terminating { intent, pending, until },
            State::Killing { pending, until, .. } => State::Killing { intent, pending, until },
        };
        follow(self);
    }

    pub fn begin(&mut self, action: Action, until: Option<Time>) {
        let state = mem::replace(&mut self.state, State::Open(Intent::Ordinary));
        self.state = match state {
            State::Open(intent) | State::Grace { intent, .. } => State::Waiting { intent, action, until },
            State::Waiting { intent, until, .. } => State::Waiting { intent, action, until },
            State::Terminating { intent, pending, until: held } => State::Terminating {
                intent,
                pending: Pending::Result(action),
                until: retained_deadline(pending, held, until),
            },
            State::Killing { intent, pending, until: held } => State::Killing {
                intent,
                pending: Pending::Result(action),
                until: retained_deadline(pending, held, until),
            },
        };
        follow(self);
    }

    pub fn result(&mut self) {
        let state = mem::replace(&mut self.state, State::Open(Intent::Ordinary));
        self.state = match state {
            State::Waiting { intent, until, .. } => settled(intent, until),
            State::Terminating { intent, until, .. } => State::Terminating { intent, pending: Pending::Done, until },
            State::Killing { intent, until, .. } => State::Killing { intent, pending: Pending::Done, until },
            state @ (State::Open(_) | State::Grace { .. }) => state,
        };
        follow(self);
    }

    pub fn release_alarm(&mut self) {
        let state = mem::replace(&mut self.state, State::Open(Intent::Ordinary));
        self.state = match state {
            State::Open(intent) | State::Grace { intent, .. } => State::Open(intent),
            State::Waiting { intent, action, .. } => State::Waiting { intent, action, until: None },
            State::Terminating { intent, pending, .. } => State::Terminating { intent, pending, until: None },
            State::Killing { intent, pending, .. } => State::Killing { intent, pending, until: None },
        };
        follow(self);
    }

    pub fn reset(&mut self) {
        let intent = match self.intent() {
            Intent::YieldRequested => Intent::YieldRequested,
            Intent::Ordinary | Intent::Finishing => Intent::Ordinary,
        };
        self.state = match mem::replace(&mut self.state, State::Open(Intent::Ordinary)) {
            State::Open(_)
            | State::Waiting { .. }
            | State::Grace { .. }
            | State::Terminating { .. }
            | State::Killing { .. } => State::Open(intent),
        };
        follow(self);
    }

    pub fn escalated(&self) -> bool {
        match self.state {
            State::Terminating { .. } | State::Killing { .. } => true,
            State::Open(_) | State::Waiting { .. } | State::Grace { .. } => false,
        }
    }

    pub fn next_deadline(&self) -> Option<Time> {
        match self.state {
            State::Open(_) => None,
            State::Waiting { until, .. }
            | State::Grace { until, .. }
            | State::Terminating { until, .. }
            | State::Killing { until, .. } => until,
        }
    }

    pub fn terminate(&mut self, now: Time, grace: Duration, process: &mut Process, below: &mut Queue<io::Request>) {
        if process.exit().is_some() {
            self.release_alarm();
            process.drain(now, below);
            return;
        }
        let until =
            if process.child().is_some() || process.spawning() { Some(now.saturating_add(grace)) } else { None };
        let state = mem::replace(&mut self.state, State::Open(Intent::Ordinary));
        self.state = match state {
            State::Open(intent) | State::Grace { intent, .. } => {
                terminating(intent, Pending::Done, until, process, below)
            }
            State::Waiting { intent, action, .. } => {
                terminating(intent, Pending::Result(action), until, process, below)
            }
            State::Terminating { intent, pending, .. } | State::Killing { intent, pending, .. } => {
                killing(intent, pending, until, process, below)
            }
        };
        follow(self);
    }

    pub fn spawned(&mut self, now: Time, grace: Duration, process: &mut Process, below: &mut Queue<io::Request>) {
        let due = match self.next_deadline() {
            Some(until) => now >= until,
            None => false,
        };
        if self.escalated() || due {
            let state = mem::replace(&mut self.state, State::Open(Intent::Ordinary));
            self.state = match state {
                State::Terminating { intent, pending, .. } | State::Killing { intent, pending, .. } => {
                    restarted(intent, pending)
                }
                State::Waiting { intent, action, .. } => State::Waiting { intent, action, until: None },
                State::Open(intent) | State::Grace { intent, .. } => State::Open(intent),
            };
            follow(self);
            self.terminate(now, grace, process, below);
        }
    }
}

fn retained_deadline(pending: Pending, held: Option<Time>, new: Option<Time>) -> Option<Time> {
    match pending {
        Pending::Result(_) => held,
        Pending::Done => new,
    }
}

fn settled(intent: Intent, until: Option<Time>) -> State {
    State::Grace { intent, until }
}

fn restarted(intent: Intent, pending: Pending) -> State {
    match pending {
        Pending::Done => State::Open(intent),
        Pending::Result(action) => State::Waiting { intent, action, until: None },
    }
}

fn terminating(
    intent: Intent,
    pending: Pending,
    until: Option<Time>,
    process: &Process,
    below: &mut Queue<io::Request>,
) -> State {
    if let Some(child) = process.child() {
        below.push(io::Request::Signal { child, to: kernel::Target::Group, signal: kernel::Signal::Terminate });
    }
    State::Terminating { intent, pending, until }
}

fn killing(
    intent: Intent,
    pending: Pending,
    until: Option<Time>,
    process: &Process,
    below: &mut Queue<io::Request>,
) -> State {
    if let Some(child) = process.child() {
        below.push(io::Request::Signal { child, to: kernel::Target::Group, signal: kernel::Signal::Kill });
        for pipe in process.pipes().all().into_iter().flatten() {
            below.push(io::Request::Abort { entity: pipe });
        }
        below.push(io::Request::Close { entity: child });
    }
    State::Killing { intent, pending, until }
}

fn follow(stopping: &mut Stopping) {
    let state = mem::replace(&mut stopping.state, State::Open(Intent::Ordinary));
    stopping.state = match state {
        State::Grace { intent, until: None } => State::Open(intent),
        state @ (State::Waiting { .. }
        | State::Open(_)
        | State::Grace { until: Some(_), .. }
        | State::Terminating { .. }
        | State::Killing { .. }) => state,
    };
}

#[cfg(test)]
mod tests;
