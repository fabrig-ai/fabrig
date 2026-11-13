//! Whole committed documents, made by the engine and owned by this client.
//! See client/domain/contract/documents.md, and client/domain/objects.md, 3–8.
//! List capacities and Text byte bounds are validated by the protocol before entry;
//! the working set keeps only what its own configured limits admit.
use super::{
    Actor, ApplicationPart, ChipExecutor, DocumentText, Ending, NoteScope, Party, Phase, ProposalKey, Refusal,
    Revision, TaskLine, Text, TurnName, TurnRead,
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
    pub executor: ChipExecutor,
    pub phase: Phase,
    /// Whole live phase facts; compact `phase` remains the document's filter value.
    pub details: Box<ChipPhaseFacts>,
}

/// Exact producer phase with its mandatory owner metadata.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChipPhaseFacts {
    pub phase: super::ChipPhase,
    pub details: Box<super::ChipDetails>,
}

/// The settings revisions an agent task's runs read their charter from
/// (client/domain/contract/documents.md, 1; domain/engine/settings.md, 3).
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
    pub talk: Option<Talk>,
    pub session: Option<Session>,
    /// None for a procedure's or a person's task.
    pub settings: Option<TaskSettings>,
    pub label: u32,
    pub flags: super::TaskFlags,
    pub last_activity: Wall,
    /// When the task was made: its record's `created_at` (jig/docs/design/client/domain/contract/documents.md, 1).
    pub made: Wall,
    pub waits_on: super::WaitsOn,
    pub chip: Chip,
    /// The actual goal this task serves, including itself when it is a goal.
    pub goal: Option<u64>,
    pub priority: Option<u32>,
    pub lineage_chips: List<Chip>,
    pub inputs: List<super::TaskLink>,
    pub project: u32,
    pub requester: super::Requester,
    pub lineage: List<u64>,
    pub spec: super::SpecificationDocument,
    pub spec_revision: Revision,
    /// The independent person-task revision, when supplied by its owner.
    pub person_revision: Option<Revision>,
    /// Exact current taker identity; a display name is not required.
    pub taken_by: Option<u64>,
    /// Supplied instruction text; absence supplies no effective text.
    pub instructions: Option<DocumentText>,
    pub wake: super::WakePolicy,
    /// Standing interests supplied separately from activation policy.
    pub subscriptions: super::Subscriptions,
    pub result_contract: super::TaskContractDocument,
    pub dependencies: List<super::TaskLink>,
    pub dependency_facts: List<super::Dependency>,
    pub references: List<super::TaskLink>,
    /// At least one task list was omitted; this does not identify which list.
    pub cut: bool,
    pub resources: List<super::ResourceLine>,
    pub budget: super::Budget,
    pub authority: super::AuthorityDocument,
    pub hold: Option<Box<super::ChipHold>>,
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

/// Whole separately supplied task context (client/domain/contract/documents.md, 1).
/// Absence supplies no runtime, impact, spend, panel or writer-slot facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskContext {
    pub parts: List<ApplicationPart>,
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

/// Allocated offer slots for the closed act vocabulary (client/domain/contract/documents.md, 3).
/// Offers are independent of paged rows; their nested values keep their own bounds.
pub const MAX_OFFERED_ACTS: u32 = 27;

/// An engine offered act kind, without view words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActKind {
    /// Open or continue a standing task's talk.
    Talk,
    /// Leave a standing task's talk.
    Leave,
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
    /// Remove a project (jig/docs/design/domain/people/acts.md, 2).
    RemoveProject,
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
        grants: List<super::Grant>,
        /// Complete configured connector kind order; equality is implicit.
        implications: List<super::KindImplication>,
        /// Complete numbers and lists for directly made widening.
        authority: Box<super::AuthorityDocument>,
        /// Hard envelope of supported form choices.
        ceiling: Box<super::AuthorityDocument>,
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
    Drift { part: ApplicationPart },
    /// A connector effect failed for good.
    EffectFailed { part: ApplicationPart },
    /// An unrecoverable effect may have happened.
    EffectUncertain { part: ApplicationPart },
    /// Out of budget.
    Budget { spent: u64, limit: u64 },
    /// Past a deadline.
    Deadline { at: Wall },
    /// Stopped by a party.
    Stopped { by: Party, at: Wall },
    /// Waited too long for a hold.
    Resource { resource: ApplicationPart, holder: Actor },
    /// A pool lost allocation.
    Pool { part: ApplicationPart },
    /// A procedure needs a decision.
    Procedure { part: ApplicationPart },
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
    pub executor: super::Executor,
    pub contract: super::TaskContractDocument,
    pub authority: super::AuthorityDocument,
    pub deadline: super::NewDeadline,
    pub budget: u64,
    pub funder: super::Funder,
    pub dependencies: List<u64>,
    pub resources: List<super::Part>,
    pub wake: super::WakePolicy,
    pub priority: Option<u32>,
    pub cut: bool,
}

/// Words addressed by one task message.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Question {
    pub task: Chip,
    pub from: Option<super::Requester>,
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
    Approval { state: Text, part: ApplicationPart },
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
    Connector { summary: DocumentText, part: ApplicationPart },
    /// Exact connector result kind and raw owner part, before any scoped application routing.
    Change { kind: u16, part: super::Part, words: DocumentText },
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
pub struct ImpactCounts {
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
pub struct DecodedActivityLine {
    pub source: Box<super::ActivityLine>,
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
/// What a line weighs in a window: its bytes, and this for it and for each
/// message, block, object and part it holds (client/domain/client/conversations.md).
pub const LINE_ROOM: u32 = 64;
impl DecodedActivityLine {
    /// Retained payload bytes and structural room (client/domain/client/conversations.md).
    #[must_use]
    pub fn weight(&self) -> u64 {
        let mut weight = u64::from(LINE_ROOM).saturating_add(activity_weight(&self.source.line));
        if let Some(archive) = &self.decoded {
            match &archive.turn {
                TurnRead::Decoded(turn) => {
                    if let Some(llm) = &turn.llm {
                        weight = weight.saturating_add(byte_weight(llm.model.bytes()));
                        if let Some(effort) = &llm.effort {
                            weight = weight.saturating_add(byte_weight(effort.bytes()));
                        }
                    }
                    for message in &turn.messages {
                        weight = weight.saturating_add(u64::from(LINE_ROOM));
                        for block in &message.blocks {
                            weight = weight.saturating_add(u64::from(LINE_ROOM));
                            match block {
                                super::Block::Text(text) => weight = weight.saturating_add(byte_weight(text.bytes())),
                                super::Block::Opaque { dialect, .. } => {
                                    weight = weight.saturating_add(byte_weight(dialect.bytes()));
                                }
                                super::Block::Result { summary, .. } => {
                                    weight = weight.saturating_add(byte_weight(summary.bytes()));
                                }
                                super::Block::Call(call) => {
                                    weight = weight.saturating_add(byte_weight(call.tool.bytes()));
                                    weight = weight.saturating_add(byte_weight(call.input.bytes()));
                                    if let Some(outcome) = &call.outcome {
                                        weight = weight.saturating_add(byte_weight(outcome.bytes()));
                                    }
                                    weight = weight.saturating_add(
                                        u64::from(call.objects.len()).saturating_mul(u64::from(LINE_ROOM)),
                                    );
                                    for part in &call.parts {
                                        weight = weight.saturating_add(part_weight(part));
                                    }
                                }
                            }
                        }
                    }
                }
                TurnRead::Undecodable { .. } => {}
            }
        }
        weight
    }
    /// A committed archive identity, independent of whether its bytes are available.
    #[must_use]
    pub fn committed_turn(&self) -> Option<TurnName> {
        match self.source.line {
            super::Activity::Turn { attempt, turn, .. } => Some(TurnName { attempt, number: turn }),
            super::Activity::Words { .. }
            | super::Activity::Heard(_)
            | super::Activity::Notified { .. }
            | super::Activity::RunBegan { .. }
            | super::Activity::RunEnded { .. }
            | super::Activity::Decided { .. }
            | super::Activity::NowWritten { .. }
            | super::Activity::Stood { .. }
            | super::Activity::TalkOpened { .. }
            | super::Activity::TalkEnded { .. }
            | super::Activity::SessionEnded { .. }
            | super::Activity::Kept { .. }
            | super::Activity::Milestone { .. } => None,
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
            super::Activity::Words { message, words, .. } => Some((*message, words)),
            super::Activity::Heard(_)
            | super::Activity::Notified { .. }
            | super::Activity::RunBegan { .. }
            | super::Activity::RunEnded { .. }
            | super::Activity::Turn { .. }
            | super::Activity::Decided { .. }
            | super::Activity::NowWritten { .. }
            | super::Activity::Stood { .. }
            | super::Activity::TalkOpened { .. }
            | super::Activity::TalkEnded { .. }
            | super::Activity::SessionEnded { .. }
            | super::Activity::Kept { .. }
            | super::Activity::Milestone { .. } => None,
        }
    }
    /// Supplied committed read-through; missing and zero remain distinct.
    #[must_use]
    pub fn read_position(&self) -> Option<u64> {
        match self.source.line {
            super::Activity::Turn { read, .. } => read,
            super::Activity::Words { .. }
            | super::Activity::Heard(_)
            | super::Activity::Notified { .. }
            | super::Activity::RunBegan { .. }
            | super::Activity::RunEnded { .. }
            | super::Activity::Decided { .. }
            | super::Activity::NowWritten { .. }
            | super::Activity::Stood { .. }
            | super::Activity::TalkOpened { .. }
            | super::Activity::TalkEnded { .. }
            | super::Activity::SessionEnded { .. }
            | super::Activity::Kept { .. }
            | super::Activity::Milestone { .. } => None,
        }
    }
}

/// Facts about a task phase that a chip alone does not carry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Runtime {
    /// Exact owner activation, including absent observation or actual start.
    Active(super::ActiveRun),
    /// Waiting on dependencies or a named resource hold.
    Waiting { dependencies: List<u64>, hold: Option<ApplicationPart> },
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
    Procedure { panel: ApplicationPart },
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
    pub tasks: ImpactCounts,
    pub live_runs: u32,
    pub proposals: u32,
}

/// A resource held for writing and the run holding its writer slot.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WriterSlot {
    pub resource: ApplicationPart,
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

fn byte_weight(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("retained byte length fits u64")
}
fn activity_weight(activity: &super::Activity) -> u64 {
    match activity {
        super::Activity::Words { words, kind, .. } => byte_weight(words.bytes()).saturating_add(kind_weight(kind)),
        super::Activity::Heard(word) | super::Activity::Notified { word, .. } => {
            u64::from(LINE_ROOM).saturating_add(byte_weight(word.words.bytes())).saturating_add(kind_weight(&word.kind))
        }
        super::Activity::Stood { words, .. } => byte_weight(words.bytes()),
        super::Activity::NowWritten { line, .. } => byte_weight(line.bytes()),
        super::Activity::Kept { pending, .. } => u64::from(pending.len()).saturating_mul(u64::from(LINE_ROOM)),
        super::Activity::TalkOpened { .. }
        | super::Activity::TalkEnded { .. }
        | super::Activity::SessionEnded { .. }
        | super::Activity::RunBegan { .. }
        | super::Activity::RunEnded { .. }
        | super::Activity::Turn { .. }
        | super::Activity::Decided { .. }
        | super::Activity::Milestone { .. } => 0,
    }
}
fn kind_weight(kind: &super::MessageKind) -> u64 {
    match kind {
        super::MessageKind::Result(result) => match result {
            super::WordResult::Approved { state, .. } => byte_weight(state.bytes()),
            super::WordResult::Verdict { fields, .. } => {
                let mut weight = 0_u64;
                for field in fields {
                    weight = weight
                        .saturating_add(u64::from(LINE_ROOM))
                        .saturating_add(byte_weight(&field.name))
                        .saturating_add(byte_weight(&field.value));
                }
                weight
            }
            super::WordResult::Report
            | super::WordResult::Change { .. }
            | super::WordResult::Failed
            | super::WordResult::Cancelled
            | super::WordResult::Job => 0,
        },
        super::MessageKind::QuestionOwed
        | super::MessageKind::NothingAwaited
        | super::MessageKind::SettingsDecision { .. }
        | super::MessageKind::Escalation { .. }
        | super::MessageKind::Proposal { .. }
        | super::MessageKind::PersonStalled { .. }
        | super::MessageKind::ProposalDecision { .. }
        | super::MessageKind::QuestionAnswered { .. }
        | super::MessageKind::QuestionEnded { .. }
        | super::MessageKind::Words
        | super::MessageKind::EffectNotMade { .. }
        | super::MessageKind::Amendment { .. }
        | super::MessageKind::Question
        | super::MessageKind::Answer { .. }
        | super::MessageKind::Notice { .. }
        | super::MessageKind::Timer { .. }
        | super::MessageKind::News { .. }
        | super::MessageKind::Met { .. } => 0,
    }
}
fn part_weight(part: &ApplicationPart) -> u64 {
    let mut weight = u64::from(LINE_ROOM);
    match part {
        ApplicationPart::Resource { path, .. } => {
            for segment in path {
                weight = weight.saturating_add(byte_weight(segment));
            }
        }
        ApplicationPart::Connector { .. }
        | ApplicationPart::Kind { .. }
        | ApplicationPart::Panel { .. }
        | ApplicationPart::Result { .. }
        | ApplicationPart::Effect { .. }
        | ApplicationPart::OwnerEffect { .. }
        | ApplicationPart::Outbox { .. }
        | ApplicationPart::Hold { .. }
        | ApplicationPart::Project { .. }
        | ApplicationPart::Deployment { .. } => {}
    }
    weight
}

/// A standing task's talk (domain/tasks/standing.md, 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Talk {
    pub party: u64,
    pub opened: Wall,
    pub last: Wall,
}
/// A standing task's current session (domain/tasks/standing.md, 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Session {
    pub first: u64,
    pub last: Wall,
}
