//! One domain completion and its retained prompt (protocol/programs.md, 5.5–5.9).
//! Admission claims the callback before startup validation. An answered request
//! keeps its context for correlated program calls, but owes no second terminal.
//! This machine knows neither process resources nor the program's stream grammar.

use alloc::boxed::Box;
use core::mem;
use skein_lib::{Queue, Time, Token};
use smith_domain::{GrantName, llm, run, session};
use smith_protocol_llm::{self as protocol, FromDomain, ToDomain};

pub(super) struct Complete {
    pub owner: Token,
    pub grant: Option<GrantName>,
    pub prompt: llm::Prompt,
    pub deadlines: session::Deadlines,
    pub outcome: run::outcome::OutcomeSpec,
    pub deliver: Option<run::outcome::ChangeSpec>,
    pub observation: Option<protocol::Observation>,
    pub deltas: protocol::Deltas,
}

impl Complete {
    pub fn from_request(request: FromDomain) -> Self {
        match request {
            FromDomain::Complete {
                owner,
                grant,
                prompt,
                deadlines,
                outcome,
                deliver,
                observation,
                deltas,
                bounds: _,
            } => Self { owner, grant, prompt, deadlines, outcome, deliver, observation, deltas },
            FromDomain::Cancel { .. } | FromDomain::Yield { .. } | FromDomain::Close { .. } => {
                unreachable!("the completion route admits Complete")
            }
        }
    }
}

pub(super) enum Terminal {
    Completed { completion: llm::Completion },
    Failed { failure: llm::Failure, evidence: llm::Evidence, detail: Box<[u8]>, report: llm::Report },
    Cancelled,
}

pub(super) struct Current {
    state: State,
}

enum State {
    Empty,
    Admitted { owner: Token },
    Active { owner: Token, request: Complete, progress: Progress },
    Held { request: Complete },
    Returning { owner: Token, request: Complete, terminal: Terminal },
    Refusing { owner: Token, terminal: Terminal },
}

enum Progress {
    Preparing,
    Receiving { phase: llm::CompletionPhase, until: Time },
}

impl Current {
    pub fn new() -> Self {
        Self { state: State::Empty }
    }

    pub fn claim(&mut self, owner: Token, up: &mut Queue<ToDomain>) {
        let state = mem::replace(&mut self.state, State::Empty);
        self.state = match state {
            State::Empty | State::Admitted { .. } => State::Admitted { owner },
            State::Active { request, progress, .. } => claimed(owner, request, progress),
            State::Held { request } => claimed(owner, request, Progress::Preparing),
            State::Returning { .. } | State::Refusing { .. } => unreachable!("follow emits before another entrance"),
        };
        follow(self, up);
    }

    pub fn retain(&mut self, request: Complete, up: &mut Queue<ToDomain>) {
        let state = mem::replace(&mut self.state, State::Empty);
        self.state = match state {
            State::Admitted { owner } => claimed(owner, request, Progress::Preparing),
            State::Active { owner, progress, .. } => claimed(owner, request, progress),
            State::Empty | State::Held { .. } => State::Held { request },
            State::Returning { .. } | State::Refusing { .. } => unreachable!("follow emits before another entrance"),
        };
        follow(self, up);
    }

    pub fn settle(&mut self, terminal: Terminal, up: &mut Queue<ToDomain>) {
        let state = mem::replace(&mut self.state, State::Empty);
        self.state = match state {
            State::Active { owner, request, .. } => returned(owner, request, terminal),
            State::Admitted { owner } => State::Refusing { owner, terminal },
            State::Empty | State::Held { .. } | State::Returning { .. } | State::Refusing { .. } => {
                unreachable!("a terminal consumes one outstanding completion")
            }
        };
        follow(self, up);
    }

    pub fn clear(&mut self, up: &mut Queue<ToDomain>) {
        let state = mem::replace(&mut self.state, State::Empty);
        self.state = match state {
            State::Empty | State::Held { .. } => State::Empty,
            State::Admitted { .. } | State::Active { .. } | State::Returning { .. } | State::Refusing { .. } => {
                unreachable!("the completion answers before its context is released")
            }
        };
        follow(self, up);
    }

    pub fn progress(&mut self, now: Time, head: bool, up: &mut Queue<ToDomain>) {
        let state = mem::replace(&mut self.state, State::Empty);
        self.state = match state {
            State::Active { owner, request, .. } => progressed(owner, request, now, head),
            state @ (State::Empty | State::Admitted { .. } | State::Held { .. }) => state,
            State::Returning { .. } | State::Refusing { .. } => unreachable!("follow emits before another entrance"),
        };
        follow(self, up);
    }

    pub fn next_deadline(&self) -> Option<Time> {
        match self.state {
            State::Active { progress: Progress::Receiving { until, .. }, .. } => Some(until),
            State::Active { progress: Progress::Preparing, .. }
            | State::Empty
            | State::Admitted { .. }
            | State::Held { .. }
            | State::Returning { .. }
            | State::Refusing { .. } => None,
        }
    }

    pub fn phase(&self, now: Time) -> llm::CompletionPhase {
        match &self.state {
            State::Active { request, .. } if now >= request.deadlines.whole => llm::CompletionPhase::Whole,
            State::Active { progress: Progress::Receiving { phase, .. }, .. } => *phase,
            State::Active { progress: Progress::Preparing, .. }
            | State::Empty
            | State::Admitted { .. }
            | State::Held { .. }
            | State::Returning { .. }
            | State::Refusing { .. } => llm::CompletionPhase::Head,
        }
    }

    pub fn owner(&self) -> Option<Token> {
        match self.state {
            State::Admitted { owner } | State::Active { owner, .. } => Some(owner),
            State::Empty | State::Held { .. } | State::Returning { .. } | State::Refusing { .. } => None,
        }
    }

    pub fn request(&self) -> Option<&Complete> {
        match &self.state {
            State::Active { request, .. } | State::Held { request } | State::Returning { request, .. } => Some(request),
            State::Empty | State::Admitted { .. } | State::Refusing { .. } => None,
        }
    }

    pub fn request_mut(&mut self) -> Option<&mut Complete> {
        match &mut self.state {
            State::Active { request, .. } | State::Held { request } | State::Returning { request, .. } => Some(request),
            State::Empty | State::Admitted { .. } | State::Refusing { .. } => None,
        }
    }
}

fn claimed(owner: Token, request: Complete, progress: Progress) -> State {
    State::Active { owner, request, progress }
}

fn progressed(owner: Token, request: Complete, now: Time, head: bool) -> State {
    let phase = if head { llm::CompletionPhase::Head } else { llm::CompletionPhase::Idle };
    let until = now
        .saturating_add(if head { request.deadlines.head } else { request.deadlines.idle })
        .min(request.deadlines.whole);
    State::Active { owner, request, progress: Progress::Receiving { phase, until } }
}

fn returned(owner: Token, request: Complete, terminal: Terminal) -> State {
    State::Returning { owner, request, terminal }
}

fn emit(owner: Token, terminal: Terminal, up: &mut Queue<ToDomain>) {
    up.push(match terminal {
        Terminal::Completed { completion } => ToDomain::Completed { owner, completion },
        Terminal::Failed { failure, evidence, detail, report } => {
            ToDomain::Failed { owner, failure, evidence, detail, report }
        }
        Terminal::Cancelled => ToDomain::Cancelled { owner },
    });
}

/// Emit the one terminal, retaining only context still used by the program.
fn follow(current: &mut Current, up: &mut Queue<ToDomain>) {
    let state = mem::replace(&mut current.state, State::Empty);
    current.state = match state {
        State::Returning { owner, request, terminal } => {
            emit(owner, terminal, up);
            State::Held { request }
        }
        State::Refusing { owner, terminal } => {
            emit(owner, terminal, up);
            State::Empty
        }
        state @ (State::Empty | State::Admitted { .. } | State::Active { .. } | State::Held { .. }) => state,
    };
}

#[cfg(test)]
mod tests;
