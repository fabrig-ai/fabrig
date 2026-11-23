//! The engine root's store vocabulary: keys, records, ranges and bounded pages.
//! temper/docs/design/domain/root.md, 7, and temper/docs/design/domain/root-restart.md, 1.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use temper_engine_domain_forge as forge;
pub mod loads;
mod store;
pub use store::{
    CallAnswer, CallKey, CallRecord, Changed, CoreRange, Deployment, DocumentOutcome, DocumentRefusal, Documented,
    EscalationDecisionRecord, Family, Key, NoteFamily, OpenedTerm, OpenedWire, PartsAnswer, ProposalDecisionRecord,
    Range, Record, RememberDid, RememberRefusal, Remembered, RunProof, TerminalRecord, TurnProof, TurnRecord,
    WorkerOpening, WorkerStatement, Write, call_answer_bytes, owned_bytes, program_answer_bytes, record_bytes,
};

/// What a `document` call does with its body (temper/docs/design/domain/agent/ask-temper.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentAction {
    Write,
    Append,
    Replace,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowToolAnswer {
    Write { follow: u32, refusal: Option<Box<[u8]>> },
    List(forge::FollowsView),
    Show { follow: u32, view: Option<Box<forge::FollowView>> },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RedToolAnswer {
    Write(forge::RedReply),
    List { goal: u64, bound: u32, reds: Box<[forge::Red]> },
    NoGoal { task: u64 },
}

/// A snapshot for text rendering; writes retain this result under their call name.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FindingToolAnswer {
    Write(forge::FindingReply),
    List(forge::Findings),
    Show { project: u32, number: u32, finding: Option<forge::FindingAnswer> },
}
