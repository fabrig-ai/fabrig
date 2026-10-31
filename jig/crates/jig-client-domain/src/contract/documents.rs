//! Whole committed documents, made by the engine and owned by this client.
//! See client/domain/contract.md, 5, and client/domain/objects.md, 3–8.
//! List capacities and Text byte bounds are validated by the protocol before entry;
//! the working set keeps only what its own configured limits admit.
use super::{
    Actor, DocumentText, Ending, Executor, NoteScope, Part, Party, Phase, ProposalKey, Refusal, Revision, TaskLine,
    Text, TurnName, TurnRead,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
/// Exact counts of work needing this party.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Needs {
    pub questions: u32,
    pub proposals: u32,
    pub escalations: u32,
    pub person_tasks: u32,
}

/// A named task wherever a page mentions it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chip {
    pub task: u64,
    pub title: DocumentText,
    pub executor: Executor,
    pub phase: Phase,
    /// Whole live phase facts; compact `phase` remains the document's filter value.
    pub details: Box<ChipPhase>,
}

/// Exact producer phase with its mandatory owner metadata.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChipPhase {
    pub phase: super::OwnerChipPhase,
    pub details: Box<super::OwnerChipDetails>,
}

/// The settings revisions an agent task's runs read their charter from
/// (client/domain/contract.md, 5.1; domain/engine.md, 17.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskSettings {
    /// The revision the task records.
    pub task: u32,
    /// Its live run's, only while that run was prepared under another.
    pub run: Option<u32>,
    /// The newest revision's number.
    pub newest: u32,
}

/// A complete task document with independently moving revisions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Task {
    /// None for a procedure's or a person's task.
    pub settings: Option<TaskSettings>,
    pub label: u32,
    pub flags: super::TaskFlags,
    pub last_activity: Wall,
    pub waits_on: super::WaitsOn,
    pub chip: Chip,
    /// The actual goal this task serves, including itself when it is a goal.
    pub goal: Option<u64>,
    pub priority: Option<u32>,
    pub lineage_chips: List<Chip>,
    pub inputs: List<super::OwnerTaskLink>,
    pub project: u32,
    pub requester: super::OwnerRequester,
    pub lineage: List<u64>,
    pub spec: super::SpecificationDocument,
    pub spec_revision: Revision,
    /// The independent person-task revision, when supplied by its owner.
    pub person_revision: Option<Revision>,
    /// Exact current taker identity; a display name is not required.
    pub taken_by: Option<u64>,
    /// Supplied instruction text; absence supplies no effective text.
    pub instructions: Option<DocumentText>,
    pub wake: super::OwnerWakePolicy,
    /// Standing interests supplied separately from activation policy.
    pub subscriptions: super::Subscriptions,
    pub result_contract: super::TaskContractDocument,
    pub dependencies: List<super::OwnerTaskLink>,
    pub dependency_facts: List<super::Dependency>,
    pub references: List<super::OwnerTaskLink>,
    /// At least one task list was omitted; this does not identify which list.
    pub cut: bool,
    pub resources: List<super::OwnerResourceLine>,
    pub budget: super::OwnerBudget,
    pub authority: super::OwnerAuthorityDocument,
    pub hold: Option<Box<super::OwnerChipHold>>,
    pub proposals: List<ProposalKey>,
    pub activity: u64,
    pub changed: Wall,
    pub acts: List<Offered>,
    pub result: Option<super::TaskResult>,
    pub delegates: List<LiveDelegate>,
    /// Separately supplied context; the task owner document supplies none.
    pub context: Option<Box<TaskContext>>,
    /// The task's LLM choice, when its executor runs a model.
    pub llm: Option<super::LlmChoice>,
    pub kept_account: Option<u32>,
    pub run_wait: Option<super::TaskRunWait>,
}

/// Whole separately supplied task context (client/domain/contract.md, 5.1).
/// Absence supplies no runtime, impact, spend, panel or writer-slot facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskContext {
    pub parts: List<Part>,
    pub closing: Option<Closing>,
    pub spend: List<SpendLine>,
    pub runtime: Runtime,
    pub impact: Impact,
    pub writer_slots: List<WriterSlot>,
}

/// What a task still waits for while closing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Closing {
    pub delegates: u32,
    /// None for an unjoined owner row; Some(empty) means known no connector waits.
    pub connectors: Option<List<u16>>,
    pub stage: Option<super::ClosingStage>,
    /// Actual pending outcome, not an ending reconstructed from settlement facts.
    pub ending: Option<Ending>,
}

/// Spend attributed to a run or delegate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpendLine {
    pub task: u64,
    pub attempt: Option<u64>,
    pub spent: u64,
}

/// Allocated offer slots for the closed act vocabulary (contract.md, 5.3).
/// Offers are independent of paged rows; their nested values keep their own bounds.
pub const MAX_OFFERED_ACTS: u32 = 25;

/// An engine offered act kind, without view words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActKind {
    /// Write to an agent.
    Message,
    /// Answer a question or person task.
    Answer,
    /// Take a role task.
    Take,
    /// Hand a task back.
    HandBack,
    /// Accept a proposal.
    Accept,
    /// Reject a proposal.
    Reject,
    /// Pass a decision up.
    PassUp,
    /// Release a held task.
    Release,
    /// Leave a task held.
    LeaveHeld,
    /// Amend a specification.
    Amend,
    /// Stop a live run.
    Stop,
    /// Cancel a subtree.
    Cancel,
    /// Become a requester.
    Move,
    /// Reorder tracked work in its project.
    Reorder,
    /// Mark updates read.
    Seen,
    /// Edit a note.
    EditNote,
    /// Delete a note.
    DeleteNote,
    /// Adopt a resource.
    Adopt,
    /// Change policy.
    ChangePolicy,
    /// Rename a project.
    Rename,
    /// Change roles.
    SetRoles,
    /// Make a service.
    MakeService,
    /// Set a pool.
    SetPool,
    /// Withdraw the party's own pending proposal.
    Withdraw,
}

/// What the engine says would happen on an act.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Admission {
    /// It would be made.
    Allowed,
    /// It would be proposed to a holder.
    Proposed { to: Recipient },
    /// Concrete drafts compare with engine-supplied facts before sending.
    Bounded { within: Bounds, beyond: Beyond },
    /// It is not admitted and says why.
    Refused(Refusal),
}

/// Numbers and lists supplied by the engine, never policy inferred by the client.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bounds {
    /// Creating choices within the party's offered authority.
    Create(super::CreateBounds),
    /// Additional budget and complete held grants for an amendment.
    Amend {
        budget_left: u64,
        grants: List<super::OwnerGrant>,
        /// Complete configured connector kind order; equality is implicit.
        implications: List<super::OwnerKindImplication>,
        /// Complete numbers and lists for directly made widening.
        authority: Box<super::OwnerAuthorityDocument>,
        /// Hard envelope of supported form choices.
        ceiling: Box<super::OwnerAuthorityDocument>,
    },
}

/// The engine's outcome when a concrete draft exceeds its supplied facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Beyond {
    /// It would be proposed to this holder.
    Proposed { to: Recipient },
    /// It would be refused with these facts.
    Refused(Refusal),
}

/// An act and its admission judged by the engine.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Offered {
    pub effect: Option<super::Effect>,
    pub act: ActKind,
    pub admission: Admission,
}

/// Where an object waits.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Recipient {
    /// At a task above.
    Task(u64),
    /// At a named party.
    Party(u64),
    /// At a policy role.
    Role { project: u32, role: u32 },
    /// Every policy role that may accept this kind of proposal, in supplied order.
    Roles { project: u32, roles: List<u32> },
}

/// A hold revision and the facts it decides.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hold {
    pub placed: Option<super::PlacedHold>,
    pub revision: Revision,
    pub reason: HoldReason,
    pub before: Phase,
    pub since: Wall,
    pub result: Option<ResultValue>,
    /// Spend snapshot attributed to each run and delegate when held.
    pub spend: List<SpendLine>,
}

/// Why a task is held, with the facts its card needs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HoldReason {
    /// Out of tries, with failures.
    Tries { failures: List<AttemptFailure> },
    /// Stalled on named work.
    Stalled { since: Wall, waiting: Text },
    /// A dependency ended unsuccessfully.
    Dependency { task: u64, how: Ending },
    /// A connector reported drift.
    Drift { part: Part },
    /// A connector effect failed for good.
    EffectFailed { part: Part },
    /// An unrecoverable effect may have happened.
    EffectUncertain { part: Part },
    /// Out of budget.
    Budget { spent: u64, limit: u64 },
    /// Past a deadline.
    Deadline { at: Wall },
    /// Stopped by a party.
    Stopped { by: Party, at: Wall },
    /// Waited too long for a hold.
    Resource { resource: Part, holder: Actor },
    /// A pool lost allocation.
    Pool { part: Part },
    /// A procedure needs a decision.
    Procedure { part: Part },
}

/// One failed attempt and its failure class.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttemptFailure {
    pub attempt: u64,
    pub class: FailureClass,
    pub reason: Text,
    pub at: Wall,
}

/// The failure class, as a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailureClass {
    /// A host was lost.
    HostLost,
    /// A transient failure.
    Temporary,
    /// A permanent failure.
    Permanent,
    /// The run stalled.
    Stall,
}

/// One task in a proposal batch.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProposedTask {
    pub number: u64,
    pub spec: super::SpecificationDocument,
    pub executor: super::OwnerExecutor,
    pub contract: super::TaskContractDocument,
    pub authority: super::OwnerAuthorityDocument,
    pub deadline: super::NewDeadline,
    pub budget: u64,
    pub funder: super::Funder,
    pub dependencies: List<u64>,
    pub resources: List<super::OwnerPart>,
    pub wake: super::OwnerWakePolicy,
    pub priority: Option<u32>,
    pub cut: bool,
}

/// Words addressed by one task message.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Question {
    pub task: Chip,
    pub from: Option<super::OwnerRequester>,
    pub message: u64,
    pub words: DocumentText,
    pub since: Wall,
    pub acts: List<Offered>,
    pub ending: Option<Ending>,
}

/// A numeric person identity with an optional supplied display name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TakenBy {
    pub number: u64,
    pub name: Option<Text>,
}

/// What a person task answer must say.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PersonContract {
    /// Choose from the named options.
    Choice { options: List<OptionLine> },
    /// Approve the named state.
    Approval { state: Text, part: Part },
    /// Answer in words.
    Words,
}

/// One option in a person task choice.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OptionLine {
    pub number: u32,
    pub words: Text,
    pub words_max: u32,
    pub followups: u32,
    pub fields: List<FieldRule>,
}

/// An opaque named field admitted by a verdict choice.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldRule {
    pub name: Box<[u8]>,
    pub max: u32,
}

/// An opaque named value supplied with a verdict.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResultField {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}

/// The kind of result a task owes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResultContract {
    /// A report in words.
    Report,
    /// A verdict and its fields.
    Verdict { choices: List<OptionLine> },
    /// A connector result.
    Connector { connector: u16, kind: u16 },
    /// One choice.
    Choice { options: List<OptionLine> },
    /// A named state approval.
    Approval,
    /// Words as a result.
    Words,
}

/// The result payload, with connector values as parts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResultValue {
    /// A report.
    Report(DocumentText),
    /// Exact owner verdict payload. `ResultDetails` separately supplies complete
    /// followup/proposal references, including stored tasks without a chip.
    OwnerVerdict { verdict: u32, words: DocumentText, fields: List<ResultField> },
    /// A verdict and follow ups.
    Verdict {
        verdict: u32,
        words: DocumentText,
        fields: List<ResultField>,
        tasks: List<Chip>,
        proposal: Option<ProposalKey>,
    },
    /// A summary and connector result.
    Connector { summary: DocumentText, part: Part },
    /// Exact connector result kind and raw owner part, before any scoped application routing.
    Change { kind: u16, part: super::OwnerPart, words: DocumentText },
    /// A party choice.
    Choice { option: u32, by: Party, reason: DocumentText },
    /// Whole approved state and its optional explanation.
    Approved { state: Box<[u8]>, words: Option<DocumentText> },
    /// A task that could not complete its contract.
    Failure(DocumentText),
    /// An answer in words.
    Words(DocumentText),
}

/// A task ending and its result, as a card.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Result {
    pub task: Chip,
    pub ending: Ending,
    pub value: Option<ResultValue>,
    /// Completion wall when supplied; closing results may have no completion wall.
    pub at: Option<Wall>,
    /// Complete owner metadata when supplied; partial scripted values retain explicit absence.
    pub details: Option<Box<super::ResultDetails>>,
    pub acts: List<Offered>,
}

/// Counts of tasks folded beyond a tree window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhaseCounts {
    /// Ended chips whose source does not supply a specific outcome.
    pub ended: u32,
    pub waiting: u32,
    pub running: u32,
    pub held: u32,
    pub closing: u32,
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

/// A group of inbox entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InboxGroup {
    /// Questions.
    Questions,
    /// Proposals.
    Proposals,
    /// Held tasks.
    Escalations,
    /// Tasks for parties.
    PersonTasks,
    /// Unread results.
    Results,
    /// Unread chat replies.
    Replies,
}

/// The read position for one conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReadPosition {
    pub task: u64,
    pub through: u64,
}

/// Whole recurring template and retained period facts (pages.md, section 6).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Recurring {
    pub chip: Chip,
    pub key: u32,
    pub batch: List<ProposedTask>,
    pub budget: u64,
    pub overlap: super::RecurringOverlap,
    /// The current period supplied by the engine's funding owner.
    pub current_period: u64,
    /// When its retained ledger opened, if that ledger exists.
    pub began: Option<Wall>,
    /// The period source's next boundary, when supplied.
    pub next: Option<Wall>,
    pub last_period: u64,
    pub pending_period: Option<u64>,
    pub last_batch: List<u64>,
    pub cut: bool,
}

/// The stored writer of a note, without inferred identity or run facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoteAuthor {
    /// A party made or corrected the note.
    Party { party: u64 },
    /// An identified task attempt wrote the note.
    Task { task: u64, attempt: u64 },
}

/// A note with its independently moving revision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Note {
    pub number: u64,
    pub revision: Revision,
    pub scope: NoteScope,
    pub description: DocumentText,
    pub body: DocumentText,
    pub writer: NoteAuthor,
    pub tasks: List<u64>,
    pub changed: Option<Wall>,
    pub acts: List<Offered>,
    /// The supplied reference list is incomplete.
    pub cut: bool,
}

/// Whole activity source and its optional decoded committed turn.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivityLine {
    pub source: Box<super::OwnerActivityLine>,
    pub decoded: Option<ArchivedTurn>,
}

/// Archived metadata and turn, carried by activity or fetched as one complete read.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchivedTurn {
    pub task: u64,
    pub name: TurnName,
    pub cumulative: u64,
    pub read: Option<u64>,
    pub at: Wall,
    pub turn: TurnRead,
}
impl ActivityLine {
    /// A committed archive identity, independent of whether its bytes are available.
    #[must_use]
    pub fn committed_turn(&self) -> Option<TurnName> {
        match self.source.line {
            super::OwnerActivity::Turn { attempt, turn, .. } => Some(TurnName { attempt, number: turn }),
            super::OwnerActivity::Words { .. }
            | super::OwnerActivity::Heard(_)
            | super::OwnerActivity::Notified { .. }
            | super::OwnerActivity::RunBegan { .. }
            | super::OwnerActivity::RunEnded { .. }
            | super::OwnerActivity::Decided { .. }
            | super::OwnerActivity::Kept { .. }
            | super::OwnerActivity::Milestone { .. } => None,
        }
    }
    /// Decoded content paired with its supplied committed identity.
    #[must_use]
    pub fn decoded_turn(&self) -> Option<(TurnName, &super::Turn)> {
        Some((self.committed_turn()?, self.decoded.as_ref()?.turn.decoded()?))
    }
    /// The immutable inbox message named by a words activity line.
    #[must_use]
    pub fn word_message(&self) -> Option<(u64, &Text)> {
        match &self.source.line {
            super::OwnerActivity::Words { message, words, .. } => Some((*message, words)),
            super::OwnerActivity::Heard(_)
            | super::OwnerActivity::Notified { .. }
            | super::OwnerActivity::RunBegan { .. }
            | super::OwnerActivity::RunEnded { .. }
            | super::OwnerActivity::Turn { .. }
            | super::OwnerActivity::Decided { .. }
            | super::OwnerActivity::Kept { .. }
            | super::OwnerActivity::Milestone { .. } => None,
        }
    }
    /// Supplied committed read-through; missing and zero remain distinct.
    #[must_use]
    pub fn read_position(&self) -> Option<u64> {
        match self.source.line {
            super::OwnerActivity::Turn { read, .. } => read,
            super::OwnerActivity::Words { .. }
            | super::OwnerActivity::Heard(_)
            | super::OwnerActivity::Notified { .. }
            | super::OwnerActivity::RunBegan { .. }
            | super::OwnerActivity::RunEnded { .. }
            | super::OwnerActivity::Decided { .. }
            | super::OwnerActivity::Kept { .. }
            | super::OwnerActivity::Milestone { .. } => None,
        }
    }
}

/// Facts about a task phase that a chip alone does not carry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Runtime {
    /// Exact owner activation, including absent observation or actual start.
    Active(super::ActiveRun),
    /// Waiting on dependencies or a named resource hold.
    Waiting { dependencies: List<u64>, hold: Option<Part> },
    /// Idle and waiting on its wake policy.
    Idle,
    /// A run is due, preparing or claimed.
    Starting,
    /// A run is writing or calling a tool.
    Running { attempt: u64, since: Wall, tool: Option<Text> },
    /// A live run keeps its slot while waiting for the person.
    WaitingForWords { attempt: u64, since: Wall },
    /// A failed attempt waits until its retry time.
    BackingOff { at: Wall, attempt: u64, tries: u32, class: FailureClass, reason: Text },
    /// A connector procedure is active, shown in its panel.
    Procedure { panel: Part },
    /// A person task is waiting for its named recipient.
    Person { addressed: Recipient, since: Option<Wall> },
    /// Closing while its children and effects settle.
    Closing(Closing),
    /// Held, with separately retained prior lifecycle when supplied.
    Held { was: Option<super::HeldRuntime> },
    /// Ended, with its outcome in the task result.
    Ended,
}

/// What a steering confirmation says a cancel or close would affect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Impact {
    pub tasks: PhaseCounts,
    pub live_runs: u32,
    pub proposals: u32,
}

/// A resource held for writing and the run holding its writer slot.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WriterSlot {
    pub resource: Part,
    pub task: u64,
    pub attempt: Option<u64>,
}

/// A live chat delegate and the funding a takeover would reserve.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LiveDelegate {
    pub chip: Chip,
    pub takeover_budget: u64,
}

/// Bounded live project tasks in last-change order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tasks {
    pub rows: List<TaskLine>,
    pub more: bool,
}
