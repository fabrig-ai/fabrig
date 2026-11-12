//! Pure change procedure and landing queue policy (domain/forge/change/README.md and domain/forge/queues.md and domain/forge/README.md, section 6).
//!
//! This crate keeps no state: the connector top owns and commits every
//! `Change`. It never knows a task or a forge API call. `step` reads a coherent
//! snapshot and returns the next state and one decision; the top checks a
//! requested effect with authority and saves its state with the request.
//! The procedure is level-triggered: a repeated snapshot yields the same
//! decision, while a committed pending state asks for no duplicate action.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
#![expect(clippy::disallowed_macros, reason = "the relay contracts require Default for Route and Questions")]

extern crate alloc;

mod boundary;
mod domain;
mod queue;
mod relay;
#[cfg(test)]
mod tests;

pub use boundary::{
    Ask, AuthorityHold, AuthorityLack, AuthorityRecovery, Change, Ci, Decision, Delegate, DelegateSpend, Early, Effect,
    EffectResult, Facts, Fault, Freshness, Gate, GateFailure, GateKind, GateReport, Heard, Hold, Judgement, Limits, Open, Pending,
    PendingKind, Pull, QUESTION_LINE, QUESTIONS_OPEN, Questions, Ready, Repair, Route, Stands, StartedFrom, State,
    Status, Stepped, Verification, Way,
};
pub use domain::{prefix_verdict, authority_words, held_on_ci, released, review_failed, step};
pub use queue::first;
pub use relay::{Asked, RelayAct, RelayHeard, Relayed, Settled, relay};
