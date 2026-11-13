//! Decision objects and retained rows (`client/domain/objects.md`, sections 4–8;
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! `client/domain/pages.md`, 5.2). Store readers supply retained rows. Routing
//! trails and before/after history require those store rows; no trail is cached.

use super::{
    AuthorityLine, Chip, ContractLine, Ending, HoldLine, Parameter, Part, PriorPhase, Proposer, Task, TaskLink, Text,
};
use crate::{Limits, documents::Missing};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

/// A specification without application values; loss is explicit on every text and list.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SpecLine {
    pub words: Text,
    pub parameters: List<Parameter>,
    pub inputs: List<u64>,
    pub cut: bool,
    pub asks: bool,
}
/// One future batch member; dependencies retain allocated or local batch numbers.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProposedTask {
    pub deadline: tasks::Deadline,
    pub number: u64,
    pub executor: tasks::Executor,
    pub spec: SpecLine,
    pub contract: ContractLine,
    pub authority: AuthorityLine,
    pub budget: u64,
    pub funder: tasks::Funder,
    pub dependencies: List<u64>,
    pub resources: List<Part>,
    pub wake: tasks::WakePolicy,
    pub priority: Option<u32>,
    pub cut: bool,
}
/// The relevant task fields before and after one proposed amendment.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AmendmentState {
    pub addressed: Option<tasks::PersonAddress>,
    pub taken_by: Option<u64>,
    /// Known current instructions in proposals; exact retained override in history.
    pub instructions: Option<Text>,
    pub spec: SpecLine,
    pub authority: AuthorityLine,
    pub wake: tasks::WakePolicy,
    pub dependencies: List<u64>,
    pub cut: bool,
}
/// What accepting the immutable proposal would do.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalAction {
    Settings {
        against: u32,
        items: List<crate::reads::RevisionItemHead>,
        review: Option<Box<super::SettingsReview>>,
        made: Option<u32>,
    },
    Batch(List<ProposedTask>),
    Effect {
        part: Part,
        authority: AuthorityLine,
        attempt: u64,
        completion: u32,
        position: u32,
        purpose: u64,
    },
    ProcedureEffect {
        part: Part,
        authority: AuthorityLine,
        step: u64,
        purpose: u64,
    },
    Amend {
        task: TaskLink,
        addressed: Option<tasks::AddressChange>,
        before: Box<AmendmentState>,
        after: Box<AmendmentState>,
        reason: Text,
        release: Option<u64>,
    },
    Widen {
        task: TaskLink,
        before: AuthorityLine,
        after: AuthorityLine,
    },
    Release {
        task: TaskLink,
        hold: Option<HoldLine>,
    },
}
/// Current recipient; earlier recipients are retained proposal history rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProposalWaiting {
    pub holder: tasks::ProposalHolder,
    pub since: Wall,
}
/// Funding facts before an acceptance; absent funds have never been allocated.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FundingOffer {
    pub source: tasks::Funder,
    pub needed: u64,
    pub left: Option<u64>,
    pub after: Option<u64>,
}
/// A pending proposal. Acts and a store-loaded routing trail are supplied later.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Proposal {
    pub projects: List<u32>,
    pub acts: List<crate::acts::Offered>,
    pub proposer: Proposer,
    pub number: u64,
    pub project: u32,
    pub reason: Text,
    pub action: ProposalAction,
    pub as_holder: bool,
    pub waiting: ProposalWaiting,
    pub funding: List<FundingOffer>,
}
/// The full task overview supplies the hold's tries, resources, dependencies,
/// budget, previous phase and kept result without duplicating a second snapshot.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Escalation {
    pub acts: List<crate::acts::Offered>,
    pub task: Box<Task>,
}
/// A question has bounded words with their loss stated; its activity keeps them whole.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Question {
    pub acts: List<crate::acts::Offered>,
    pub task: Chip,
    pub message: u64,
    pub from: tasks::Party,
    pub at: Wall,
    pub words: Text,
}
/// A shown dependency result; content not in live memory is named for a store read.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ShownResult {
    pub task: TaskLink,
    pub result: Option<ResultDoc>,
}
/// The person task's current spec, address, form and shown results.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PersonTask {
    pub acts: List<crate::acts::Offered>,
    pub task: Chip,
    pub revision: u64,
    pub requester: tasks::Party,
    pub spec: SpecLine,
    pub address: tasks::PersonAddress,
    pub taken_by: Option<u64>,
    pub since: Option<Wall>,
    pub stall: Option<skein_lib::Duration>,
    pub deadline: Option<Wall>,
    pub contract: ContractLine,
    pub results: List<ShownResult>,
    pub cut: bool,
}
/// One result by its contract; a store-loaded row supplies ended outcomes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResultDoc {
    pub acts: List<crate::acts::Offered>,
    pub task: Chip,
    pub contract: ContractLine,
    pub ending: Ending,
    pub at: Option<Wall>,
    pub requester: tasks::Party,
    pub executor: tasks::Executor,
    pub chosen_by: Option<u64>,
    /// Exact result-created tasks, including those already ended or taken away.
    /// The owner always supplies `Some`, including a known empty batch.
    pub followups: Option<List<TaskLink>>,
    pub delegates: List<TaskLink>,
    pub proposal: Option<super::ProposalRef>,
    pub cut: bool,
}
/// Timestamped semantic history with immutable amendment settings.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HistoryLine {
    pub task: u64,
    pub revision: u64,
    pub by: tasks::Party,
    pub change: tasks::Change,
    pub reason: Text,
    pub proposal: Option<ProposalHistory>,
    pub amendment: Option<(Box<AmendmentState>, Box<AmendmentState>)>,
    pub at: Option<Wall>,
}
/// Proposal facts retained with a history row, including its recipient or decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProposalHistory {
    pub reason: Text,
    pub action: RetainedAction,
    pub number: u64,
    pub proposer: u64,
    pub as_holder: bool,
    pub state: ProposalHistoryState,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalHistoryState {
    Pending(ProposalWaiting),
    Accepted { by: tasks::Party },
    Rejected { by: tasks::Party, reason: Text },
    Withdrawn,
}
/// The immutable action kept by a proposal history row; an amendment names
/// only its requested fields because no before snapshot is retained there.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RetainedAction {
    Settings { against: u32, items: List<crate::reads::RevisionItemHead>, made: Option<u32>, projects: List<u32> },
    Batch(List<ProposedTask>),
    Effect { part: Part, authority: AuthorityLine, attempt: u64, completion: u32, position: u32, purpose: u64 },
    ProcedureEffect { part: Part, authority: AuthorityLine, step: u64, purpose: u64 },
    Amend { task: u64, change: Box<AmendmentChange> },
    Widen { task: u64, authority: AuthorityLine },
    Release { task: u64 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AmendmentChange {
    pub addressed: Option<tasks::AddressChange>,
    pub instructions: Option<Text>,
    pub release: Option<u64>,
    pub spec: Option<SpecLine>,
    pub authority: Option<AuthorityLine>,
    pub wake: Option<tasks::WakePolicy>,
    pub dependencies: Option<List<u64>>,
    pub reason: Text,
    pub cut: bool,
}
/// Timestamped lifecycle row named by a task activity position.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MilestoneLine {
    pub task: u64,
    pub position: u64,
    pub phase: MilestonePhase,
    pub at: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MilestonePhase {
    Waiting,
    Active(tasks::Active),
    Closing { stage: tasks::Stage, ending: Ending },
    Held { before: PriorPhase, why: tasks::Hold, source: Option<Part>, ending: Option<Ending> },
    Ended(Ending),
}
pub fn document_bound(limits: &super::DocumentLimits) -> Option<u64> {
    // A task page prices every nested authority, part, chip, form and result
    // carrier. A proposal batch or person task can hold one per bounded line.
    super::task_tree::overview_bound(limits)?
        .checked_mul(u64::from(limits.lines).checked_add(4)?)?
        .checked_add(u64::from(limits.lines).checked_add(2)?.checked_mul(super::offers::heap_bound(limits)?)?)?
        .checked_add(u64::try_from(size_of::<Proposal>()).ok()?)?
        .checked_add(u64::try_from(size_of::<PersonTask>()).ok()?)
}

/// The ended delegate's identity and outcome, made from its retained task row.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EndedDelegate {
    pub chip: Chip,
    pub ended: Ending,
    pub at: Option<Wall>,
}

/// Copy an ended delegate after the store reader has checked project visibility.
pub fn ended_delegate_from_row(limits: &Limits, row: &tasks::TaskRecord) -> Result<EndedDelegate, Missing> {
    match &row.phase {
        tasks::Phase::Ended(value) => Ok(EndedDelegate {
            chip: super::small::chip_from_row(limits, row)?,
            ended: super::task_tree::ending(limits, value)?,
            at: row.ended_at,
        }),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            Err(Missing::Unknown)
        }
    }
}
