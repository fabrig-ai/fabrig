//! One interrupt/result/input-close sequence (protocol/programs.md 5.7).
//! Keeps whether a result is owed; knows no run budget. Cancel, close and the
//! wind-down enter here; the backend's existing grace escalates to termination.

use super::{Backend, Below, Phase, tail, turns};
use crate::mcp;
use skein_io::{self as io, kernel};
use skein_lib::{Env, List, Queue, Time};
use smith_domain::llm;
use smith_protocol_llm as protocol;
use smith_protocol_llm::{FromDomain, ToDomain};

pub(super) const INTERRUPTED: &[u8] = b"not run: the program turn was interrupted";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Stop {
    None,
    Closing,
    WindDown,
    /// End an in-flight completion so the domain can issue its finishing note.
    Yielding,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum WindDown {
    Ordinary,
    /// A one-shot domain request waits for startup or a window switch to settle.
    YieldRequested,
    Finishing,
}

impl Backend {
    /// Whether the backend can act on a domain yield request without
    /// repeating an interrupt already waiting on its result (programs.md 5.9).
    #[must_use]
    fn can_yield_for_wind_down(&self) -> bool {
        self.phase == Phase::Running
            && self.owner.is_some()
            && self.stop == Stop::None
            && self.winding != WindDown::Finishing
            && !self.turns.result
    }

    /// Relay the domain's request to end main's in-flight program turn before
    /// its finishing note. The result remains this Complete's one terminal.
    pub(super) fn yield_for_wind_down(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        if !self.can_yield_for_wind_down() {
            return;
        }
        self.winding = WindDown::Ordinary;
        if self.active_turn() {
            self.stop(env.now, true, true);
            self.stop = Stop::Yielding;
        } else {
            self.deliver(env, up, below);
        }
        self.flush(below);
    }

    pub(super) fn active_turn(&self) -> bool {
        if self.turns.result {
            return false;
        }
        if self.request.is_some() || self.owner.is_some() || self.turns.message.is_some() || !self.calls.is_empty() {
            return true;
        }
        for call in &self.ledger {
            if !call.answered {
                return true;
            }
        }
        false
    }

    pub(super) fn wind_down(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        mut request: FromDomain,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) -> Option<FromDomain> {
        let (prompt, capture) = match &mut request {
            FromDomain::Complete { prompt, observation, .. } => (prompt, observation.is_some()),
            FromDomain::Cancel { .. } | FromDomain::Yield { .. } => unreachable!("completion route"),
        };
        match &prompt.choice {
            llm::ToolChoice::Only(_) | llm::ToolChoice::None if self.winding == WindDown::Ordinary => {}
            llm::ToolChoice::Only(_) | llm::ToolChoice::Auto | llm::ToolChoice::None => return Some(request),
        }
        self.winding = WindDown::Finishing;
        let Some(rendered) =
            tail::render_observed(prompt, &self.ledger, &self.settings.adapter, self.settings.input_bytes, capture)
        else {
            self.fail(env, llm::Failure::Invalid, b"smith: wind-down input exceeds its bound", up, below);
            return None;
        };
        self.outgoing = Some(rendered.line);
        let active = self.active_turn();
        self.request = Some(request);
        if let Some(text) = rendered.text {
            self.prompt_text_measure(env.now, text);
        }
        self.stop(env.now, active, true);
        if self.stop == Stop::None {
            self.completion_deadline(env, true);
        }
        self.flush(below);
        None
    }

    pub(super) fn stop(&mut self, now: Time, active: bool, keeping: bool) {
        self.flush_deltas(now);
        let waiting = self.stop != Stop::None;
        self.stop = if keeping { Stop::WindDown } else { Stop::Closing };
        if active && let Some(child) = self.child {
            if !waiting {
                self.io.push(io::Request::Signal {
                    child,
                    to: kernel::Target::Group,
                    signal: kernel::Signal::Interrupt,
                });
            }
        } else if !waiting {
            self.finish_stop();
        }
        if !waiting {
            self.deadline = if self.child.is_some() || self.spawning {
                Some(now.saturating_add(self.settings.grace))
            } else {
                None
            };
        }
    }

    pub(super) fn stopped_result(&mut self, env: &Env<protocol::ComponentLimits>) {
        let yielding = self.stop == Stop::Yielding;
        if yielding {
            // A complete assistant message retains its text and usage. A partial
            // message has no completion to charge (programs.md 5.5, 5.9).
            self.turns.interrupted();
        }
        self.finish_stop();
        if self.phase == Phase::Running && !self.switching {
            self.completion_deadline(env, true);
        }
    }

    fn refuse_unanswered(&mut self) {
        let calls = core::mem::replace(&mut self.calls, List::with_capacity(u32::from(self.settings.mcp.calls)));
        for pending in calls.into_boxed() {
            match pending {
                mcp::Up::Call { call, .. } => {
                    self.server.as_mut().expect("held MCP call").answer(
                        call,
                        mcp::Answer::Error(INTERRUPTED.into()),
                        &mut self.io,
                    );
                }
                mcp::Up::Prompt { call, .. } => {
                    self.server.as_mut().expect("held permission prompt").verdict(
                        call,
                        mcp::Verdict::Deny { message: INTERRUPTED.into() },
                        &mut self.io,
                    );
                }
                mcp::Up::Listening { .. } | mcp::Up::Connected | mcp::Up::Ended(_) => unreachable!("held calls"),
            }
        }
        for index in 0..self.ledger.len() {
            let call = self.ledger.get_mut(index).expect("ledger cell");
            if !call.answered
                && let Some(rpc) = call.rpc.take()
            {
                self.server.as_mut().expect("correlated MCP call").answer(
                    rpc,
                    mcp::Answer::Error(INTERRUPTED.into()),
                    &mut self.io,
                );
            }
        }
    }

    fn finish_stop(&mut self) {
        self.refuse_unanswered();
        for index in 0..self.ledger.len() {
            let call = self.ledger.get_mut(index).expect("ledger cell");
            call.rpc = None;
            call.answer = None;
            call.answered = true;
        }
        match self.stop {
            Stop::Closing => {
                if let Some(input) = self.pipes[0] {
                    self.io.push(io::Request::Close { entity: input });
                }
            }
            Stop::WindDown => {
                let closed = self.turns.closed;
                self.turns = turns::Turns::new(&self.settings.receiving, self.settings.adapter.client.answer);
                self.turns.closed = closed;
                self.send_input();
            }
            Stop::Yielding | Stop::None => {}
        }
        self.stop = Stop::None;
    }
}
