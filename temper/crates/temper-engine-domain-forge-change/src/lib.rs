//! Pure change procedure and landing queue policy (domain/forge/change/README.md and domain/forge/queues.md and domain/forge/updates.md, section 1).
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
mod clock;
mod domain;
#[cfg(test)]
mod fuzzy;
mod passing;
mod queue;
mod relay;
#[cfg(test)]
mod tests;

pub use boundary::{
    AMENDMENTS_KEPT, ANSWERS_KEPT, Amended, Answered, Ask, AuthorityHold, AuthorityLack, AuthorityRecovery, Below,
    Blocked, Cause, Change, Ci, CloseAt, CloseOutdated, CloseRefused, Decision, Delegate, DelegateSpend, Early, Effect,
    EffectResult, FOLLOW_CHECK_GATE, Facts, Fault, Freshness, Gate, GateFailure, GateKind, GateReport, Handed, Heard,
    Hold, Judged, Judgement, Kind, Landed, Limits, Open, Pending, PendingKind, Pull, QUESTION_LINE, QUESTIONS_OPEN,
    Questions, Ready, Recheck, Repair, Replay, Reply, Route, Stands, StartedFrom, State, Status, Stepped, TemperDir,
    Took, Verification, WORDS_WAITING, Wait, Waiting, Way,
};
pub use clock::{Clock, Execution, Reading};
pub use domain::{
    ReleaseTrigger, authority_account, authority_recovery, authority_words, gates_pass, held_on_ci, prefix_verdict,
    release, review_failed, step,
};
pub use passing::{Passed, WordAct, pass};
pub use queue::first;
pub use relay::{Asked, RelayAct, RelayAnswer, RelayHeard, Relayed, relay};
