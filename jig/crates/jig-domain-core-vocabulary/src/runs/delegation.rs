//! Delegates and procedure steps (jig/docs/design/domain/engine/README.md, 4.4).

use alloc::boxed::Box;
use jig_domain_tasks as tasks;

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
    pub executor: tasks::Executor,
    pub spec: tasks::Spec,
    pub contract: tasks::Contract,
    pub authority: tasks::Authority,
    /// When the delegate must end: its authority's wall, or a span after
    /// the decision that makes it (domain/authority.md, 7). Named with no
    /// wall ends with its creator's.
    pub deadline: tasks::Deadline,
    /// An agent child's LLM choice; none for the core to resolve its charter's default within
    /// the child's authority, as at Create (domain/tasks/README.md, 3).
    pub llm: Option<tasks::LlmChoice>,
    /// Open connector grants narrowed with the allocated task number.
    pub symbolic_grants: Box<[tasks::Grant]>,
    pub dependencies: Box<[Dependency]>,
    pub wake: Option<tasks::WakePolicy>,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: tasks::Flags,
    pub budget: jig_domain_people::Budget,
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
    pub proposals: Box<[tasks::ProposalAction]>,
    /// Dependencies of the procedure's task its work started from; an amendment may never
    /// remove them (jig/docs/design/domain/connectors.md, 6.1).
    pub started_from: Box<[u64]>,
    pub end: StepEnd,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StepAmendment {
    pub task: u64,
    pub revision: u64,
    pub amendment: tasks::Amendment,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StepCancel {
    pub task: u64,
    pub reason: Box<[u8]>,
    pub grace: Option<skein_lib::Duration>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct StepMessage {
    pub target: u64,
    pub kind: tasks::MessageKind,
    pub words: Box<[u8]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum StepEnd {
    Result(tasks::TaskResult),
    Hold(tasks::Hold),
    HoldFrom { why: tasks::Hold, source: Box<tasks::HoldSource> },
    Wait,
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
