//! Pure landing-by-turn policy (domain/landing.md, sections 1–10).
//!
//! This crate keeps nothing. The connector top owns each landing, receipts,
//! keys in flight and effects. A step returns one decision and the state to
//! commit with it. Keys and run verdicts are pure functions too.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod boundary;
mod domain;
mod key;
mod single;
#[cfg(test)]
mod fuzzy;
#[cfg(test)]
mod tests;
mod verdict;

pub use boundary::{
    CHECK_NAME_BYTES, Combine, Commit, Decision, Facts, Flight, Gate, GateFacts, GateProof, Hold, Key, Landing, Leave,
    Limits, Member, MemberFacts, Outcome, Phase, Posted, Posting, Preparation, Project, Proof, Repository, Round, RoundEnd,
    Status, Stepped, WORKFLOW_BYTES,
};
pub use domain::{join, leave, step};
pub use key::{Entry, EntryKind, KeyBound, KeyLimits, Keyed, Tree, definition, follow_key, key};
pub use verdict::{Conclusion, Fault, Job, Run, RunState, Verdict, verdict};
