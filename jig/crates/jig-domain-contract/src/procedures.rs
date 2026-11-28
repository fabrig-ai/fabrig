//! Procedure steps, decisions and delegates (jig/docs/design/domain/connectors.md, 2).

use crate::ops::{
    Amend, Amendment, Authority, Cancel, Contract, Deadline, Executor, Flags, Grant, Hold, HoldSource, LlmChoice,
    Message, MessageKind, New, Proposal, ProposalAction, Spec, TaskResult, WakePolicy,
};
use alloc::boxed::Box;

mod waits;
pub use waits::{ReportRefusal, Reported, ReportedBound};

use crate::effects::{Answer, Finding};
use crate::ops::{Problem, RunContext, party::Moved};
use skein_lib::List;

/// A dependency named by a delegate before the core assigns batch task IDs.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Dependency {
    /// One member of the same batch, by zero-based position.
    Batch(u32),
    /// A live task already referenced by the creator.
    Existing(u64),
}

/// One proposed direct child before the core supplies its number and funder.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Delegate {
    pub executor: Executor,
    pub charter_scope: crate::ops::CharterScope,
    pub spec: Spec,
    pub contract: Contract,
    pub authority: Authority,
    /// When the delegate must end: its authority's wall, or a span after
    /// the decision that makes it (domain/authority-budgets.md, 1). Named with no
    /// wall ends with its creator's.
    pub deadline: Deadline,
    /// An agent child's LLM choice; none for the core to resolve its charter's default within
    /// the child's authority, as at Create (domain/tasks/README.md, 3).
    pub llm: Option<LlmChoice>,
    /// Open connector grants narrowed with the allocated task number.
    pub symbolic_grants: Box<[Grant]>,
    pub dependencies: Box<[Dependency]>,
    pub wake: Option<WakePolicy>,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: Flags,
    pub budget: Budget,
}

/// A connector-owned procedure's chosen task action before core admission.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ProcedureStep {
    /// Replace this procedure task's specification words, within the specification byte bound.
    pub words: Option<Box<[u8]>>,
    pub batch: Box<[Delegate]>,
    pub amendments: Box<[StepAmendment]>,
    pub cancels: Box<[StepCancel]>,
    pub messages: Box<[StepMessage]>,
    pub proposals: Box<[ProposalAction]>,
    /// Dependencies of the procedure's task its work started from; an amendment may never
    /// remove them (jig/docs/design/domain/connectors-procedures.md, 1.1).
    pub started_from: Box<[u64]>,
    pub end: StepEnd,
}

impl ProcedureStep {
    #[must_use]
    pub fn new(end: StepEnd) -> Self {
        Self {
            words: None,
            batch: Box::new([]),
            amendments: Box::new([]),
            cancels: Box::new([]),
            messages: Box::new([]),
            proposals: Box::new([]),
            started_from: Box::new([]),
            end,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StepAmendment {
    pub task: u64,
    pub revision: u64,
    pub amendment: Amendment,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StepCancel {
    pub task: u64,
    pub reason: Box<[u8]>,
    pub grace: Option<skein_lib::Duration>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StepMessage {
    /// A question whose answer this step takes, to spend its kept notice room.
    pub notice: Option<u64>,
    pub target: u64,
    pub kind: MessageKind,
    pub words: Box<[u8]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum StepEnd {
    Result(TaskResult),
    Hold(Hold),
    HoldFrom { why: Hold, source: Box<HoldSource> },
    Wait,
}

/// One procedure decision. Each step commits its task changes with the owner's state.
/// (domain/tasks/lifecycle.md, section 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProcedureDecision {
    /// Replace this procedure task's specification words, within the specification byte bound.
    pub words: Option<Box<[u8]>>,
    pub batch: Box<[New]>,
    pub amendments: Box<[Amend]>,
    pub cancels: Box<[Cancel]>,
    pub messages: Box<[ProcedureMessage]>,
    pub proposals: Box<[Proposal]>,
    /// Dependencies the procedure's work started from (domain/connectors-procedures.md, 1.1).
    pub started_from: Box<[u64]>,
    pub end: ProcedureEnd,
}

impl ProcedureDecision {
    #[must_use]
    pub fn new(end: ProcedureEnd) -> Self {
        Self {
            words: None,
            batch: Box::new([]),
            amendments: Box::new([]),
            cancels: Box::new([]),
            messages: Box::new([]),
            proposals: Box::new([]),
            started_from: Box::new([]),
            end,
        }
    }
}

/// A procedure word, optionally spending the room kept by a question it asked.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProcedureMessage {
    pub message: Message,
    pub notice: Option<u64>,
}

impl ProcedureMessage {
    #[must_use]
    pub fn ordinary(message: Message) -> Self {
        Self { message, notice: None }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProcedureEnd {
    Result(TaskResult),
    Hold(Hold),
    HoldFrom { why: Hold, source: Box<HoldSource> },
    Wait,
}

/// How much of the party's available budget to give a member.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Budget {
    Named,
    AsAllowed { cap: Option<u64> },
}

/// Why an entire connector-owned procedure step was refused.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum StepRefusal {
    Moved { task: u64, now: Moved },
    Refused(Problem),
    Denied { answer: Answer, findings: Box<[Finding]> },
}

/// What the engine asks of a procedure connector.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Notification: start a procedure task with this context; no answer.
    Start { context: Box<RunContext>, step: u64, chat: Option<u64> },
    /// Notification: the step committed with these spec revisions; no answer.
    StepMade { task: u64, step: u64, first: Option<u64>, revisions: Box<[(u64, u64)]> },
    /// Notification: no part of the step committed; no answer.
    StepRefused { task: u64, step: u64, why: StepRefusal },
    /// Notification: the waits report was refused independently of the step; no answer.
    WaitsRefused { task: u64, why: ReportRefusal },
}

/// A procedure connector's choices and wait reports.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// A procedure step's decision.
    Step { task: u64, step: u64, code: u32, decision: ProcedureStep },
    /// Unasked: replace the task's wait report; empty clears it.
    Waits { task: u64, waits: List<Reported> },
    /// Unasked: a procedure is ready to step again.
    Wake { task: u64 },
}
