//! Pure landing-by-turn policy (domain/landing.md, sections 1–4, and domain/landing-prepare.md, section 1, and domain/landing-proof.md, sections 1–2, and domain/landing-turn.md, sections 1–2, and domain/landing.md, section 10).
//!
//! This crate keeps nothing. The connector top owns each landing, receipts,
//! keys in flight and effects. A step returns one decision and the state to
//! commit with it. Keys and run verdicts are pure functions too.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod boundary;
mod domain;
#[cfg(test)]
mod fuzzy;
mod key;
mod liveness;
mod single;
#[cfg(test)]
mod tests;
mod verdict;

pub use boundary::{
    Awaited, CHECK_NAME_BYTES, Combine, Commit, Decision, Facts, Flight, Gate, GateFacts, GateProof, GateReport, Hold,
    Key, Landing, LastPrepare, Leave, Limits, Member, MemberFacts, Outcome, Phase, Posted, Posting, Preparation,
    Probed, Project, Proof, Repository, Round, RoundEnd, RoundPull, Search, SearchTests, Searched, Status, Stepped,
    Stuck, TestId, WORKFLOW_BYTES,
};
pub use domain::{claim, cleanup_hold, held, leave, probe, step, top};
pub use key::{Entry, EntryKind, KeyBound, KeyLimits, Keyed, Tree, definition, follow_key, key};
pub use verdict::{Conclusion, Fault, Job, Judgement, Run, RunState, Shown, Verdict, job_fault, judge, verdict};

pub use temper_engine_domain_forge_tiers::Format;
