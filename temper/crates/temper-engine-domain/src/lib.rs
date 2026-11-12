//! Temper's engine root composes jig's core and the forge connector.
//! Routing, translation, assembly and the core-selected restart adapters follow
//! jig's domain/root.md; skein-lib's journal is the only output path.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
mod admission;
mod archive;
mod assemble;
mod boundary;
mod budget;
mod deadline;
mod decision;
mod domain;
mod journal_stop;
pub use journal_stop::JournalStop;
mod limits;
mod link;
pub mod loads;
mod push;
mod restart;
pub use temper_engine_domain_forge::{Push, PushAnswer};
pub mod room;
mod route;
mod settling;
mod stopping;
mod store;
#[cfg(test)]
mod tests;
mod translate;
pub use decision::{
    Counters, Decision, Delivery, Journal, JournalOutput, Limits as JournalLimits, accept_routed, commit, committed,
    fresh, resume, uncommitted, valid as journal_valid,
};
pub use store::{
    CallAnswer, CallKey, CallRecord, Changed, CoreRange, Deployment, DocumentOutcome, DocumentRefusal, Documented,
    EscalationDecisionRecord, Family, Key, NoteFamily, PartsAnswer, ProposalDecisionRecord, Range, Record, RememberDid,
    RememberRefusal, Remembered, RunProof, TerminalRecord, TurnProof, TurnRecord, Write, record_bytes,
};

pub use admission::{completes, try_step};
pub use boundary::Request as Output;
pub use boundary::*;
pub use domain::{Domain, Refused, StartStop, fire, refused, release, step};
pub use jig_domain_core::{Charter, Delegate, Dependency, Model, ProcedureStep, RunCharter, RunPolicy, StepEnd};
pub use limits::{
    BriefBudgets, BriefLimits, FACTS, Limits, TRANSCRIPT_TAIL_BYTES, VIEWS, WATCHES_PER_STREAM, core_limits,
    default_watermark, policy_row_bound, valid, views_for,
};
pub use link::{inbox_sender, inbox_words};
pub use route::{
    Approval, ForgePerson, Freshness, Gate, HostDelivery, HostRequest, LandingRule, OfferedModel, PersonForge,
    ProjectPerson, RepositoryIdentity, SeeAnswer, SeeLanding, SeeName, SeeNames, SeeNotRead, SeePerson, SeeProposal,
    SeeRead, SeeRepository, SeeSettings, SeeWords, adopt_repository_ask,
};
pub use temper_engine_domain_forge::commit_text;
pub use translate::{
    AfterLandingCheck, Config, GateCheck, LANDING_ATTEMPTS, LandingRules, ProjectLanding, RepositoryChecks, TierCheck,
    Way, landing_stock,
};

/// Startup's secret-free decision (protocol/credentials.md, 2.2).
/// Runs are volatile; every surviving carried row is no longer in use.
#[must_use]
pub fn carried_at_start(number: u64) -> Request {
    assert!(number >= 1_u64 << 63_u32, "carried number space");
    Request::EraseCarried { number }
}

pub use temper_engine_domain_ways as ways;

/// The standing follow procedure and its owner-word refusals.
pub use route::forge_route::FOLLOW_CODE;
pub use temper_engine_domain_forge::FollowRefusal;

#[cfg(test)]
mod fuzzy {
    mod approval_people;
}
