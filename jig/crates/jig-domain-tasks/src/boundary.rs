//! Root-to-tasks semantic inputs and tasks-to-root persistence/lifecycle outputs
//! (domain/tasks/README.md, sections 2–4 and domain/tasks/lifecycle.md).
//! Inputs have no protocol bytes or connector events. Root authorizes them,
//! owns exact transport replay and commits related rows/effects atomically.
pub use jig_domain_contract::ops::{
    Added, DelegateState, InvalidResult, PersonOffer, PersonStallNotice, PersonWait, Problem, QuestionAnswerer,
    Refusal, Revisions, RunContext, Status,
};
pub use jig_domain_contract::resources::{SavedResource, Writer};

use crate::{Authority, Class, Funder, Numbers, Tries};
use crate::{HoldKind, ResourceKind};
use alloc::boxed::Box;
pub use jig_domain_contract::ops::{
    Active, Bound, ChangeTitle, Circle, Closing, Contract, Deadline, Ending, Exceeded, Executor, FieldRule, Hold,
    HoldSource, JobSpec, Message, MessageKind, New, NoticeState, Parameter, Party, PersonAddress, Phase,
    RecurringOverlap, RecurringTemplate, ResultField, ResultKind, ResultsWake, Spec, Stage, TaskResult, Verdict,
    WakePolicy, WakeRule, Was, Word,
};
pub use jig_domain_contract::procedures::{ProcedureDecision, ProcedureEnd, ProcedureMessage};
pub use jig_domain_contract::resources::{Holding, Name};
pub use jig_domain_contract::topics::NewsClass;
use skein_lib::{Duration, List, ReplyTo, Wall};

/// A connector's procedure whose tasks, made with a pending dependency, are active and
/// kept rather than waiting (jig/docs/design/domain/connectors-procedures.md, 1.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct KeptStart {
    pub connector: u16,
    pub code: u32,
}

/// One bounded standing task, timer or topic interest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SubscriptionKind {
    Task { target: u64, held: bool, result: bool },
    Timer { at: Wall, period: Option<Duration> },
    Topic { connector: u16, topic: u64 },
}

/// Root-issued subscription kept with its owner task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Subscription {
    pub number: u64,
    pub kind: SubscriptionKind,
}

/// The shared inbox classification (domain/tasks/messages.md, sections 1–3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MessageFamily {
    Spoken(Spoken),
    Hint(Hint),
    Slot(Slot),
    Separate(Separate),
    Result,
    Virtual,
}

/// Words admitted from a party or a referenced task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Spoken {
    Words,
    Question,
    Answer { question: u64 },
}

/// Merged subscription hints.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hint {
    Notice { subscription: u64, target: u64, state: NoticeState },
    Timer { subscription: u64 },
    News { subscription: u64, class: NewsClass },
}

/// Control messages entering through reserved inbox room (domain/tasks/messages.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Slot {
    EffectNotMade,
    SettingsDecision,
    ProposalDecision,
    QuestionAnswered,
    QuestionOwed,
    QuestionEnded,
    PersonStalled,
}

/// Messages with separately priced inbox slots (jig/docs/design/domain/tasks/lifecycle-endings.md, 1, and jig/docs/design/domain/tasks/messages.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Separate {
    Amendment,
    FinishRequested,
    Met,
    NothingAwaited,
}

/// An unanswered question, its first answerer, and its current offer if passed up.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QuestionCredit {
    pub number: u64,
    pub answerer: Party,
    /// The logical addressed party.
    pub asked: Party,
    /// Position of the question activity in the owning task record.
    pub line: u64,
    pub offer: Option<QuestionOffer>,
}

/// The current holder and its new waiting interval. Only a task needs a relay word.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QuestionOffer {
    pub holder: crate::EscalationHolder,
    pub since: Wall,
    pub relay: Option<QuestionRelay>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QuestionRelay {
    pub entry: u64,
    pub words: Box<[u8]>,
}

/// Root-numbered notices for one asker's retired questions.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct QuestionNotice {
    pub question: u64,
    pub message: u64,
}

/// The template and period cursor retained with one live recurring task.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RecurringState {
    pub template: RecurringTemplate,
    /// Exact IDs of the last accepted template batch, retained after its members end.
    pub last_batch: Box<[u64]>,
    pub last_period: u64,
    pub pending_period: Option<u64>,
}

/// Bounded working-set pointer to an ended result still named by live work.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Stub {
    pub task: u64,
    pub phase: Status,
    /// `Key::Ended(result.raw())` is the retained result row.
    pub result: skein_lib::Token,
}

/// Root-to-tasks activation terminal, fenced by task/attempt; a terminal may park or retry without
/// ending the task. (domain/tasks/lifecycle.md, section 2, and domain/tasks/lifecycle-holds.md, section 1, and domain/tasks/lifecycle.md, section 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum End {
    /// A configured bound refused the run's model call (domain/hosts/hub.md, 3).
    Exceeded(Exceeded),
    /// Proposed final result; valid finishing with live delegates requires cancelling them or is
    /// refused without admission.
    Finished {
        /** Owned proposed result; invalid bounded shapes consume an `Invalid` failure, while oversized priced terminals refuse before charging. */
        result: TaskResult,
        /** Whether to cancel live descendant work; otherwise a valid finish with live delegates is refused and its attempt remains live. */
        cancel_delegates: bool,
    },
    /// A checked verdict whose follow-up batch is part of the result decision.
    FinishedWithFollowups {
        /// Verdict checked against the task's current result contract.
        result: TaskResult,
        /// Either immediately admitted delegates or one proposal for the whole batch.
        followups: ResultFollowups,
    },
    /// Successful activation park; clears retry/refusal counters and leaves the task idle until words wake it.
    Parked,
    /// Count one classified failure, then back off or hold beyond the configured retry allowance.
    Failed(/** Reported failure category whose counter/backoff policy applies. */ Class),
    /// A classified failure whose bounded cause reaches the retry notice and hold (domain/tasks/lifecycle-holds.md, 1).
    FailedWithReason { class: Class, reason: Box<[u8]> },
    /// Pre-execution refusal pause using the transient delay policy, without spending a failure
    /// try.
    Refused,
    /// Pre-execution refusal with a bounded operator reason and no failure try.
    RefusedWithReason { reason: Box<[u8]> },
    /// Its source or checked placement requires a fresh check; release the run without a try or timer.
    Recheck,
    /// Its worker stopped or restarted; release without a try (domain/engine/executors.md, 6).
    Suspended(Suspended),
}

/// Why an attempt's worker cut it (domain/engine/executors.md, 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SuspendWhy {
    Stopped,
    Restarted,
}

/// A worker's interruption, with the restart window at the end's decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Suspended {
    pub worker: u32,
    pub why: SuspendWhy,
    pub window: Duration,
}

/// The task's open suspension on the engine's wall clock (domain/engine/executors.md, 6.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Suspension {
    pub worker: u32,
    pub since: Wall,
    pub until: Wall,
}

/// Root-authorized follow-ups from one verdict, admitted with its result.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResultFollowups {
    /// A whole batch within the finishing task's authority and allotment.
    Delegates(Box<[New]>),
    /// One pending proposal for a whole batch beyond that authority.
    Proposal(Box<crate::Proposal>),
}

/// The most bytes a wait source can carry.
pub const WAIT_WORDS: u32 = 512;

/// Durable writer slot. A lost run retains the slot until its connector reads afresh.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct WriterSlot {
    pub number: u64,
    pub resource: Name,
    pub writer: Writer,
    pub lost: bool,
}

/// Last durable connector report of one pool's current admission capacity.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PoolSlots {
    pub number: u64,
    pub pool: Name,
    pub slots: u32,
}

/// The first accepted person answer, independent of the temporary role claim.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PersonAnswer {
    pub by: u64,
    pub at: Wall,
}

/// One committed turn's reply reference (domain/people/README.md, section 5).
/// Retained hints form the newest (wall time, reply ordinal) prefix, independently of `last_reply`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReplyHint {
    pub number: u64,
    pub activity: u64,
    pub attempt: u64,
    pub turn: u32,
    pub at: Wall,
}

/// Settings revisions retained live and ended (domain/engine/settings.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SettingsRevision {
    /// The newest revision when the task was made, whatever its parent's.
    pub task: u32,
    /// The revision its agent run was prepared under; none between attempts.
    pub run: Option<u32>,
}

/// Owned durable task state emitted to root storage; root uses `RunContext` for preparation and
/// does not maintain a mutable copy of this ledger. (domain/tasks/README.md, section 3 and domain/tasks/lifecycle.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent durable lifecycle facts are explicit")]
pub struct TaskRecord {
    /// The first concluding party and decision (domain/tasks/gatherings.md, 3).
    pub concluded: Option<crate::Concluded>,
    pub talk: Option<crate::Talk>,
    /// The standing task's current session (jig/docs/design/domain/tasks/standing.md, 7).
    pub session: Option<crate::Session>,
    /// Whether this activation continues across retries and kept parks (standing.md, 5.1).
    pub activation_open: bool,
    /// The live closer's task number (jig/docs/design/domain/tasks/README.md, 3, and jig/docs/design/domain/tasks/lifecycle-endings.md, 3).
    pub closer: Option<u64>,
    /// The live closer-ended task’s most recent judged finish.
    pub parked_result: Option<TaskResult>,
    /// Why this activation runs (jig/docs/design/domain/tasks/messages.md, 6).
    pub reasons: crate::Reasons,
    /// The kinds the claimed attempt is offered (jig/docs/design/domain/tasks/messages.md, 6).
    pub offered: crate::Reasons,
    /// The placed hold holding this task, whatever its phase says (domain/tasks/lifecycle-holds.md, 2).
    pub placed: Option<u64>,
    /// Settings read by this task and its run (domain/engine/settings.md, 3).
    pub settings: SettingsRevision,
    pub charters: u64,
    /// Opaque application label, immutable after creation.
    pub label: u32,
    /// Immutable engine flags.
    pub flags: crate::Flags,
    /// Last words heard, turn committed or delegate ended; creation until then.
    pub last_activity: Wall,
    /// Lifetime ended descendants, summed as each delegate ends; counts saturate.
    pub ended_below: crate::Endings,
    pub ended_delegates: crate::Endings,
    /// Newest question and answer activity positions, at most `inbox_messages`.
    pub question_lines: Box<[crate::QuestionLine]>,
    /// Latest hold milestone; zero before the first hold.
    pub hold: u64,
    /// Newest decided proposal history positions, at most `inbox_messages`.
    pub proposal_rows: Box<[crate::ProposalRow]>,
    /// Agent instruction replacement; absent uses the configured charter default.
    pub instructions: Option<Box<[u8]>>,
    /// An agent task's LLM choice, kept as given and changed only by an amendment
    /// (domain/tasks/README.md, section 3 and domain/tasks/changes.md).
    pub llm: Option<crate::LlmChoice>,
    /// Main account from the last successful checked claim, independent of its preference.
    pub kept_account: Option<u32>,
    /// Messages the current attempt's run refused; untaken by later fences of that attempt.
    pub refused: Box<[u64]>,
    /// A cancel asked this task to finish by a durable deadline.
    pub grace: Option<crate::grace::Grace>,
    /// Independent revisions retained in live and ended records.
    pub revisions: Revisions,
    /// Injected creation time used to order active person tasks in derived inboxes.
    pub created_at: Wall,
    /// Wall time of the last decision that changed this retained row.
    pub changed: Wall,
    /// Injected wall time at which the current lifecycle phase began.
    pub phase_since: Wall,
    /// When the present hold first wrote its line; older rows fall back to `phase_since`.
    pub held_since: Option<Wall>,
    /// First matching task start report: root reports placement or adoption, not agent or tool start.
    /// Retained through holds and the terminal; a new claim clears it.
    pub run_since: Option<Wall>,
    /// The classified failure or pre-execution refusal that last paused this activation.
    pub retry_notice: Option<crate::RetryNotice>,
    /// Injected time of the durable ending, used for newest-first result pages.
    pub ended_at: Option<Wall>,
    /// Last committed semantic change to this task; history rows are keyed by this sequence.
    pub revision: u64,
    /// Last lifecycle milestone, independent of semantic amendments and proposals.
    pub milestone: u64,
    /// Last immutable activity position; restored with live state, never reused.
    pub activity: u64,
    /// Actual activation cause, retained through preparation and restart.
    pub due_reason: crate::Why,
    /// A narrowing or next-turn message stopped the current run; its terminal starts the task anew.
    pub narrowing: bool,
    /// The task's LLM choice changed while this Open run was live: the next message for it
    /// stops the run rather than reach it (domain/tasks/changes.md).
    pub llm_changed: bool,
    /// Root-issued commit order for an ended result; zero while live and until the root saves an ending.
    pub result_position: u64,
    /// Consecutive committed reply ordinal across attempts.
    pub last_reply: u64,
    /// Newest opaque reply hints, bounded by `inbox_messages`; older lines are store-loaded.
    pub replies: Box<[ReplyHint]>,
    /// One bounded semantic held-chat decision and checked revision. Root owns
    /// authentication, routing and retained receipts.
    pub escalation: crate::Escalation,
    /// At most one pending action awaiting a holder; its terminal leaves live memory as history.
    pub proposal: Option<Box<crate::Proposal>>,
    /// A result-created proposal keeps closing at its delegate gate until decided.
    pub result_proposal: bool,
    /// Stable never-reused deployment task identity.
    pub number: u64,
    pub project: u32,
    /// Original current requester topology, distinct from the financial funder.
    pub requester: Party,
    /// Committed cumulative expense of the current attempt; a `new` `Claim` resets only this
    /// attempt baseline, not total direct spend.
    pub run_spent: u64,
    /// Unspent part of this attempt's reserved run allowance, included in `numbers.reserved`.
    pub run_reserved: u64,
    /// Requester-tree root identity, equal to number for a top-level task.
    pub root: u64,
    /// Structural depth below that root, bounded by `Limits::depth`.
    pub depth: u32,
    pub executor: Executor,
    /// Current role-task holder; direct-person tasks never need a claim.
    pub taken_by: Option<u64>,
    /// The first accepted answer, retained after the role claim is cleared.
    pub person_answer: Option<PersonAnswer>,
    /// The interval starts when dependency and resource waits clear.
    pub person_wait: Option<PersonWait>,
    /// Durable core recurring template and last considered period.
    pub recurring: Option<Box<RecurringState>>,
    /// Goal priority retained with the task for project ordering.
    pub tracked: Option<u32>,
    /// Owned bounded specification; live state has empty retained inputs.
    pub spec: Spec,
    pub contract: Contract,
    /// Owned bounded permission carrier checked by root policy, not by tasks.
    pub authority: Authority,
    /// Authentic current-allotment accounting, updated atomically with lifecycle admission.
    pub numbers: Numbers,
    /// Source of the current allotment, kept separate from requester ancestry.
    pub funder: Funder,
    /// Admitted dependency identities, bounded by `Limits::dependencies`: those named at creation,
    /// then those amendments added, the oldest first (domain/tasks/changes.md).
    pub dependencies: Box<[u64]>,
    /// Dependencies started from, in first-marked order, each once (domain/connectors-procedures.md, 1.1).
    pub started_from: Box<[u64]>,
    /// The dependencies an amendment added, each with when and by whom, in their order among
    /// `dependencies`.
    pub added: Box<[Added]>,
    /// `Exact` still-live subset of dependencies, bounded by `Limits::dependencies`; removed once
    /// per dependency end in the same decision. A waiting task waits for them; an active one is
    /// kept by them (domain/tasks/lifecycle-endings.md, 1). Restoration checks fit and required links.
    pub waiting_on: Box<[u64]>,
    /// Resources currently held or awaited; a connector may pass a failed delegate's hold to its tree root during release.
    pub holdings: Box<[Holding]>,
    /// True only after the whole requested set has been taken.
    pub holds_taken: bool,
    /// First wall time at which dependencies cleared but a hold was unavailable: the task's
    /// place in its queues.
    pub hold_wait_since: Option<Wall>,
    /// When the wait bound counts from, if not from `hold_wait_since`: a task that left its
    /// queues for a dependency added or a circle comes back at its place, its bound counted
    /// afresh (domain/tasks/holds.md, 2 and 7).
    pub hold_bound_since: Option<Wall>,
    /// Current live direct delegates, bounded by `Limits::delegates`; requester ending waits until
    /// they are gone.
    pub delegates: Box<[u64]>,
    /// Exact result-created task identities, retained after delegates end or are taken.
    /// A result admits one batch, directly or through its accepted proposal.
    pub result_followups: Box<[u64]>,
    /// Introduced peer tasks this task may message or name as a live dependency.
    pub references: Box<[u64]>,
    /// Open questions with one reserved answer slot apiece.
    pub questions: Box<[QuestionCredit]>,
    /// Incoming procedure questions that still keep space for a notice.
    pub notice_credits: Box<[crate::NoticeCredit]>,
    /// Standing interests that end with the task.
    pub subscriptions: Box<[Subscription]>,
    /// Current wake and batching policy.
    pub wake: WakePolicy,
    /// Latest newly admitted contiguous turn in the current attempt; `new` `Claim` starts at zero.
    pub turn: u32,
    /// Newest message ever admitted, including those taken by committed turns.
    pub last_message: u64,
    /// Whole unread words in increasing root message order.
    pub inbox: Box<[Word]>,
    /// Directing words taken by a run but not recorded, in increasing message order
    /// (jig/docs/design/domain/tasks/messages.md, 2).
    pub unrecorded: Box<[Word]>,
    /// Resource names with saved state, ordered and unique.
    pub saved: Box<[SavedResource]>,
    /// Some attempt of this task committed a turn, so its next preparation must read its transcript.
    pub ever_turned: bool,
    /// Lifetime tasks made in this subtree, including itself, bounded by `Limits::tree_tasks`;
    /// ending delegates does not return capacity.
    pub made: u32,
    /// Latest root-issued attempt identity; `new` claims must strictly increase it.
    pub attempt: u64,
    /// Most recently accepted activation-terminal attempt while the task remains live; not an exact
    /// transport replay receipt.
    pub last_answer: Option<u64>,
    /// Per-class failure history used with the configured retry policy.
    pub tries: Tries,
    /// Last invalid result's contract reason, kept through the next preparation.
    pub invalid_result: Option<InvalidResult>,
    /// Saturating pre-execution refusal counter used for pauses rather than failure tries.
    pub refusals: u32,
    /// Current lifecycle or the retained final outcome.
    pub phase: Phase,
    /// A procedure batch refused for live task room (domain/connectors-procedures.md, 1.2).
    pub room_wait: Option<crate::RoomWait>,
    /// A question-form message to its requester has been admitted; never cleared.
    pub asked_requester: bool,
    /// Newest unrecorded words that woke an idle park; 0 for none, never lowered
    /// (jig/docs/design/domain/tasks/lifecycle.md, 2).
    pub words_woken: u64,
    /// Current attempt (domain/tasks/lifecycle.md, 12.1).
    pub open_attempt: Option<crate::OpenAttempt>,
    /// Worker and window kept until leaving due and preparing (domain/engine/executors.md, 6.2).
    pub suspension: Option<Suspension>,
    /// Nanoseconds spent in closed suspensions.
    pub suspended: u64,
    /// The latest attempt ended suspended because its worker restarted.
    pub restarted: bool,
}

pub use crate::index::Key;

/// Tasks-to-root persistence row or root-to-tasks startup restore input; live tasks,
/// funding, writer slots and pool counts are restored before admission.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Stored {
    Summary(Box<crate::TaskSummary>),
    Retention(crate::RetentionRow),
    /// A standing hold, or a lifted hold still named by live tasks.
    PlacedHold(Box<crate::PlacedHold>),
    /// One party's durable update index.
    Update(crate::Update),
    Result(crate::ResultRow),
    EndedByParty(crate::EndedByParty),
    EndedByProject(crate::EndedByProject),
    EndedDelegate(crate::Edge),
    Delegate(crate::Edge),
    /// Earlier activity loaded on demand, never restored as live state.
    Activity(crate::ActivityLine),
    Attempt(crate::AttemptRow),
    Hold(crate::HoldRow),
    /// Pending person-origin goal proposal.
    PersonProposal(Box<crate::PersonProposal>),
    /// A decided person-origin goal proposal, loaded on demand.
    DecidedProposal(Box<crate::PersonProposal>),
    /// A closed funding source, loaded on demand.
    ClosedLedger(crate::FundingRecord),
    /// Committed change read on demand, never restored as live state.
    History(crate::History),
    /// Current bounded live task state, emitted on mutation and admitted once at startup.
    Live(/** Owned boxed live task row; fit and link checks precede activation. */ Box<TaskRecord>),
    /// The live task's cold part, put only when its fields change (protocol/store/parts.md, 2).
    LiveCold {
        number: u64,
        spec: Box<Spec>,
        contract: Contract,
        authority: Authority,
        instructions: Option<Box<[u8]>>,
    },
    /// Ended task state; root loads it for authenticated result reads, never as
    /// live startup state.
    Ended(
        /** Owned boxed ending retained in root storage; never a live restore input or a replayed result delivery. */
        Box<TaskRecord>,
    ),
    /// Compact durable pointer to a retained result named by live work.
    Stub(Stub),
    /// Finite external source accounting; default startup checks its reservations.
    Ledger(
        /** Finite period/pool accounting owned by tasks and admitted at startup under `Limits::funders`. */
        crate::FundingRecord,
    ),
    /// Occupied writer slot, restored before claims are adopted.
    Writer(WriterSlot),
    /// Last reported pool capacity, restored before hold readiness runs.
    Pool(PoolSlots),
}

impl Stored {
    /// Pure fixed-size projection of this row's logical `key`; no allocation, mutation, persistence
    /// or lifecycle output.
    #[must_use]
    pub const fn key(&self) -> Key {
        match self {
            Stored::Summary(row) => Key::Summary(row.number),
            Stored::Retention(row) => row.key(),
            Stored::PlacedHold(row) => Key::PlacedHold(row.number),
            Stored::Update(row) => row.key(),
            Stored::EndedByParty(row) => row.key(),
            Stored::EndedByProject(row) => row.key(),
            Stored::EndedDelegate(row) => Key::EndedDelegate { requester: row.requester, task: row.task },
            Stored::Delegate(row) => Key::Delegate { requester: row.requester, task: row.task },
            Stored::Result(row) => Key::Result { party: row.party, position: row.position },
            Stored::PersonProposal(row) => Key::PersonProposal(row.number),
            Stored::DecidedProposal(row) => Key::DecidedProposal(row.number),
            Stored::ClosedLedger(row) => Key::ClosedLedger(row.funder),
            Stored::History(row) => Key::History { task: row.task, revision: row.revision },
            Stored::Activity(row) => Key::Activity { task: row.task, position: row.position },
            Stored::Attempt(row) => Key::Attempt { task: row.task, attempt: row.attempt },
            Stored::Hold(row) => Key::Hold { task: row.task, position: row.position },
            Stored::Live(record) => Key::Live(record.number),
            Stored::LiveCold { number, .. } => Key::LiveCold(*number),
            Stored::Ended(record) => Key::Ended(record.number),
            Stored::Stub(stub) => Key::Stub(stub.task),
            Stored::Ledger(record) => Key::Ledger(record.funder),
            Stored::Writer(slot) => Key::Writer(slot.number),
            Stored::Pool(row) => Key::Pool(row.number),
        }
    }
}

/// Lifecycle acknowledgement classification, not an exact payload replay proof; exact transport
/// replay is the root's responsibility. (domain/tasks/lifecycle.md, section 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Accepted {
    /// This event newly admitted the task transition or turn.
    New,
    /// `Unpriced` terminal attempt equals the live task's recorded `last_answer`; root remains
    /// responsible for exact payload proof.
    Already,
}

/// Root-issued semantic task/finance inputs after authorization and transport fencing;
/// notifications have no reply destination and may be ignored if stale. (domain/tasks/README.md, section 4 and domain/tasks/lifecycle.md).
#[derive(PartialEq, Eq, Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "proposed operations stay inline and are priced by size_of")]
pub enum Event {
    /// Record a live task's kept words. Root checks that the caller is its run
    /// (jig/docs/design/domain/tasks/messages.md, 2).
    RecordWords {
        reply_to: ReplyTo,
        task: u64,
        messages: Box<[u64]>,
    },
    /// Root-authorized concluding word (domain/tasks/gatherings.md, 3).
    Conclude {
        reply_to: ReplyTo,
        by: u64,
        task: u64,
    },
    /// Close project rows (jig/docs/design/domain/tasks/changes.md).
    CloseProject {
        project: u32,
    },
    /// Root-checked placement; the root numbers it and judges standing, the child judges room.
    Place {
        reply_to: ReplyTo,
        hold: crate::PlacedHold,
    },
    /// Root-checked lift naming the placed hold revision (domain/tasks/lifecycle-holds.md, 2).
    Lift {
        reply_to: ReplyTo,
        by: Party,
        hold: u64,
        revision: u64,
        with_tasks: bool,
    },
    /// Move eligible live agent tasks on `from`, or every older revision, to the newest (domain/engine/settings.md, 4).
    MoveToNewest {
        from: Option<u32>,
    },
    /// Replace held charter versions and name the newest settings after a commit or start
    /// (domain/engine/settings.md, 2 and 3).
    Charters {
        newest: u32,
        charters: Box<[crate::CharterModels]>,
        efforts: Box<[u8]>,
    },
    /// One project's party operations, admitted in order and made whole.
    Compound {
        reply_to: ReplyTo,
        by: Party,
        project: u32,
        operations: List<crate::Operation>,
        period: u64,
        pool_budget: u64,
        period_budget: u64,
    },
    /// Latest connector admission rule for this individual resource.
    Resource {
        name: Name,
        hold: HoldKind,
    },
    /// Current adoption access for a named resource in one project.
    ResourceAccess {
        project: u32,
        name: Name,
        access: ResourceAccess,
    },
    /// Install one connector's bounded resource-kind configuration.
    Kinds {
        connector: u16,
        kinds: Box<[ResourceKind]>,
    },
    /// Connector reports its current slot count for one opaque pool.
    Slots {
        pool: Name,
        slots: u32,
    },
    /// Connector reports that a holder's allocation in this pool vanished.
    AllocationGone {
        pool: Name,
        task: u64,
    },
    /// Connector facts changed for one idle procedure; a due step is offered once.
    WakeProcedure {
        task: u64,
    },
    /// Root admits a person's goal proposal after validating policy and shape.
    ProposePerson {
        reply_to: ReplyTo,
        proposal: crate::PersonProposal,
    },
    /// Withdraw the proposing party's pending proposal.
    WithdrawPersonProposal {
        reply_to: Option<ReplyTo>,
        proposer: u64,
        proposal: u64,
    },
    /// Root commits a policy holder's decision on a person-origin goal proposal.
    DecidePersonProposal {
        reply_to: ReplyTo,
        proposer: u64,
        proposal: u64,
        by: Party,
        message: Option<u64>,
        decision: crate::ProposalDecision,
        revision: Option<u32>,
    },
    /// A newly opened project period makes the core recurring procedure due.
    TickRecurring {
        task: u64,
        period: u64,
    },
    /// A subscribed top-level procedure receives its next period's task and spend allotment.
    RenewStanding {
        task: u64,
        period: u64,
    },
    /// Root supplies fresh identities for the template batch requested by the core procedure.
    RecurringBatch {
        task: u64,
        period: u64,
        numbers: Box<[u64]>,
    },
    /// Root supplies the fresh identity of the task's own closer (domain/tasks/lifecycle-endings.md, 3).
    CloserBatch {
        task: u64,
        number: u64,
    },
    /// Fenced decision from the owner of one due procedure task.
    Procedure {
        /// The core verified that the task tree's requester reads its project at admission.
        reads_project: bool,
        reply_to: ReplyTo,
        task: u64,
        step: u64,
        /// Last message in the prefix offered to this step, or none for an empty inbox.
        read: Option<u64>,
        decision: ProcedureDecision,
    },
    /// Root-authenticated role holder takes an active person task from every role inbox.
    TakePerson {
        reply_to: ReplyTo,
        task: u64,
        person: u64,
    },
    /// The current holder returns a role task to the role inbox.
    HandBackPerson {
        reply_to: ReplyTo,
        task: u64,
        person: u64,
    },
    /// Root-authenticated addressee answers a live person task under its result contract.
    AnswerPerson {
        reply_to: ReplyTo,
        task: u64,
        person: u64,
        result: TaskResult,
    },
    /// Root-authorized bounded proposal and resolved first holder.
    Propose {
        reply_to: ReplyTo,
        proposal: crate::Proposal,
    },
    /// Root-checked current holder resolves a pending proposal after executing an acceptance.
    DecideProposal {
        reply_to: ReplyTo,
        proposer: u64,
        proposal: u64,
        /// Fresh root-issued message for acceptance/rejection, absent for pass.
        message: Option<u64>,
        by: Party,
        decision: crate::ProposalDecision,
    },
    /// Root completes a settings proposal atomically with its revision or stale withdrawal.
    DecideSettingsProposal {
        reply_to: Option<ReplyTo>,
        decision: crate::SettingsDecision,
    },
    /// Proposer withdraws its own pending action before it closes.
    WithdrawProposal {
        reply_to: ReplyTo,
        proposer: u64,
        proposal: u64,
    },
    /// Root routes one pending proposal only from the holder and revision it checked.
    StalledProposal {
        proposer: u64,
        proposal: u64,
        from: crate::ProposalHolder,
        revision: u64,
        holder: crate::ProposalHolder,
    },
    /// Root-checked live ancestor controls a delegate and its subtree.
    Cancel {
        reply_to: ReplyTo,
        by: Party,
        operation: crate::Cancel,
    },
    /// Application-only failure of a live task and cancellation of its subtree (domain/tasks/lifecycle.md, 13).
    Fail {
        reply_to: ReplyTo,
        by: Party,
        operation: crate::Fail,
    },
    /// Root-checked release of a live task.
    Release {
        reply_to: ReplyTo,
        by: Party,
        operation: crate::Release,
    },
    /// Root-checked release of a held procedure for fresh measurement.
    Measure {
        reply_to: ReplyTo,
        by: Party,
        task: u64,
    },
    /// Root-checked amendment of a live delegate, with a fresh commit-order message number.
    Amend {
        /// The core verified that the task tree's requester reads its project at admission.
        reads_project: bool,
        /// Ended dependencies and the ended requesters used to judge their reach.
        places: Box<[crate::DependencyPlace]>,
        reply_to: ReplyTo,
        by: Party,
        operation: crate::Amend,
    },
    /// Root-authorized person adoption of one live task, with its reservation moved to that person's pool.
    Move {
        reply_to: ReplyTo,
        person: u64,
        period: u64,
        pool_budget: u64,
        period_budget: u64,
        operation: crate::Move,
    },
    /// Install one root-numbered standing interest for a current task.
    Subscribe {
        /// The core verified that the task tree's requester reads its project at admission.
        reads_project: bool,
        reply_to: ReplyTo,
        task: u64,
        subscription: Subscription,
    },
    /// Root-authorized connector topic interest with the same durable task subscription.
    SubscribeTopic {
        reply_to: ReplyTo,
        task: u64,
        subscription: Subscription,
    },
    /// Remove one interest owned by a current task.
    Unsubscribe {
        reply_to: ReplyTo,
        task: u64,
        subscription: u64,
    },
    /// End task watches by reading in trees requested by parties who lost their project role.
    ReadingLost {
        project: u32,
        parties: Box<[u64]>,
    },
    /// Root places a merged state/timer/connector hint in a subscribed inbox.
    Notice {
        task: u64,
        word: Word,
    },
    /// Give two live tasks referenced by the introducer reciprocal references.
    Introduce {
        reply_to: ReplyTo,
        by: u64,
        left: u64,
        right: u64,
    },
    /// Admit authenticated person words to a live chat and wake or relay after their commit.
    Message {
        reply_to: ReplyTo,
        project: u32,
        operation: Message,
    },
    /// Admit a question offered to the addressed task's person (domain/people/README.md, 5).
    PersonQuestion {
        reply_to: ReplyTo,
        project: u32,
        task: u64,
        person: u64,
        word: Word,
    },
    /// Deliver an authenticated inbox answer and its notice in one decision.
    PassQuestion {
        asker: u64,
        question: u64,
        offer: QuestionOffer,
    },
    /// Core-admitted answer at a party placement, including a role's holder.
    AnswerOfferedQuestion {
        reply_to: ReplyTo,
        project: u32,
        asker: u64,
        word: Word,
    },
    PersonQuestionAnswer {
        reply_to: ReplyTo,
        project: u32,
        asked: u64,
        person: u64,
        question: u64,
        notice: u64,
        word: Word,
    },
    /// A person's words whose project standing was admitted by the core; retains that actor.
    AuthorizedMessage {
        reply_to: ReplyTo,
        project: u32,
        operation: Message,
    },
    Talk {
        reply_to: ReplyTo,
        project: u32,
        operation: Message,
    },
    Leave {
        reply_to: ReplyTo,
        project: u32,
        task: u64,
        party: u64,
        maintainer: bool,
    },
    /// Root delivers a just-ended delegate's result to its task requester in
    /// the same decision that archived the delegate.
    DelegateResult {
        task: u64,
        word: Word,
        wake: bool,
    },
    /// Root verified a newly named retained input against its ended store row.
    RememberStub {
        stub: Stub,
    },
    /// Root `SetRoles` preflight: inspect only current person-requested `Waiting`
    /// contexts without mutation; one terminal even before readiness.
    InspectEscalations {
        /// Stage-local root right consumed once by `EscalationsInspected`
        reply_to: ReplyTo,
        /// Root-validated project whose current `Waiting` tasks are inspected.
        project: u32,
    },
    /// Root after successful `ApplyRoles` in the same synchronous decision; emit
    /// existing `EscalationNeeded` for `Waiting` only and one completion.
    RecheckEscalations {
        /// Stage-local root right consumed by `EscalationsRechecked`
        reply_to: ReplyTo,
        /// Same preflight project; rejected/currently unheld tasks stay inert
        project: u32,
    },
    /// Root resolves the just-created routing obligation before commitment;
    /// stale notification changes nothing.
    RoutedEscalation {
        /// Held person chat.
        task: u64,
        /// Positive semantic revision.
        revision: u64,
        /// Root-verified eligible requester or final policy role.
        holder: crate::EscalationHolder,
        entry: u64,
    },
    /// Root has authenticated the current recipient and checked authority;
    /// one `EscalationDecided` terminal follows.
    DecideEscalation {
        /// Root-owned correlation owed one semantic terminal.
        reply_to: ReplyTo,
        task: u64,
        /// Waiting revision, never a root transport receipt.
        revision: u64,
        /// Positive authenticated deciding task or person.
        by: crate::EscalationDecider,
        /// Root-numbered next holder entry on pass only.
        entry: Option<u64>,
        /// Bounded authorized release/reject/pass choice.
        decision: crate::EscalationDecision,
    },
    /// The period source supplied a next boundary; no cadence is inferred.
    /// Update its current ledger, preserve the original opening wall, and ignore stale notices.
    PeriodNext {
        project: u32,
        period: u64,
        next: Option<Wall>,
    },
    /// `Open` one finite monotonic project period after root authorization; emits ledger `Save`
    /// plus `Done`, or one `Refused` with no mutation.
    OpenPeriod {
        /// Root-issued destination owed one `Done` or `Refused` terminal.
        reply_to: ReplyTo,
        project: u32,
        /// Fresh period identity, strictly greater than retained periods for this project.
        period: u64,
        /// Finite root-authorized deployment-unit budget; source capacity is checked before
        /// mutation.
        budget: u64,
    },
    /// Reserve one finite person's pool from its original period after root authorization; emits
    /// both ledger Saves plus `Done`, or `Refused` atomically.
    CarvePool {
        /// Root-issued destination owed one `Done` or `Refused` terminal.
        reply_to: ReplyTo,
        project: u32,
        /// Person whose current role and allotment root authenticated and checked.
        person: u64,
        /// Existing original project period whose available budget is reserved for this pool.
        period: u64,
        /// Complete finite pool budget, atomically reserved from the original period.
        budget: u64,
    },
    /// Post the connector's maximum price in the effect's deciding commit.
    /// Root already checked authority and the current funding balance.
    ChargeEffect {
        funder: Funder,
        maximum: u64,
    },
    /// Change an existing current-period person's pool while preserving reservations and spending.
    ResizePool {
        reply_to: ReplyTo,
        project: u32,
        person: u64,
        period: u64,
        budget: u64,
    },
    /// Change the open period's budget after root authorization, its reservations and
    /// spending kept (domain/authority-budgets.md, 1); emits the ledger's `Save` plus `Done`, or one
    /// `Refused` with no mutation.
    ResizePeriod {
        reply_to: ReplyTo,
        project: u32,
        period: u64,
        budget: u64,
    },
    /// Admit the next contiguous turn, offered inbox read and checked expense delta atomically;
    /// stale turns/attempts and arithmetic failures refuse without debit.
    Turn {
        /// Root-issued destination owed one `TurnAcknowledged` or `Refused` terminal.
        reply_to: ReplyTo,
        /// `Live` task whose claimed attempt and next turn are being admitted.
        task: u64,
        /// Nonzero current attempt fence; stale attempts cannot charge or advance turns.
        attempt: u64,
        /// Exactly the next contiguous nonzero turn in this attempt; root handles exact replay
        /// before this child input.
        turn: u32,
        /// Highest committed inbox message read by this turn; taking is atomic with the charge.
        read: Option<u64>,
        /// Offered messages still in the inbox that this turn refuses.
        refused: Box<[u64]>,
        /// Highest message this exact run was offered by its assignment or committed relay.
        offered: Option<u64>,
        /// Exact unconsumed offers authenticated by root for this attempt, at most `run_inbox`.
        outstanding: Box<[u64]>,
        /// Whole priced attempt expense; tasks posts only the checked delta above `run_spent`
        /// from this attempt's reserved run allowance.
        cumulative: u64,
    },
    /// A run's successful write of its own Now, in the note's saving decision
    /// (jig/docs/design/domain/tasks/standing.md, 6). Stale/nonstanding attempts are ignored.
    NowWritten {
        task: u64,
        attempt: u64,
        line: Box<[u8]>,
    },
    /// Root-validated answer refusals, kept before its activation in the same decision.
    /// Stale attempts or names outside the offered inbox are ignored.
    Refusals {
        task: u64,
        attempt: u64,
        refused: Box<[u64]>,
        offered: Option<u64>,
    },
    /// Create an authorized whole batch and reservations atomically, producing `Made` or
    /// `Refused` plus bounded persistence/lifecycle outputs.
    Make {
        /// The core verified that the task tree's requester reads its project at admission.
        reads_project: bool,
        /// Party behind a Delegate; reach and history are theirs, topology remains the creator's
        /// (jig/docs/design/domain/tasks/README.md, 4).
        maker: Option<u64>,
        /// Root-issued destination owed one `Made` or `Refused` terminal for the whole batch.
        reply_to: ReplyTo,
        /// Loaded ended places for a person creator (jig/docs/design/domain/tasks/README.md, 4).
        places: Box<[crate::DependencyPlace]>,
        /// Root-verified creator/requester of every `new` task; task creator and `funding`
        /// ancestry are checked separately.
        creator: Party,
        /// Owned nonempty batch bounded by `Limits::batch`; all graph, payload and reservation
        /// checks precede mutation.
        batch: Box<[New]>,
    },
    /// Admit the accepted batch of a result-created proposal while its proposer closes.
    MakeResultFollowups {
        /// The core verified that the task tree's requester reads its project at admission.
        reads_project: bool,
        /// Destination owed one `Made` or `Refused` terminal for the whole batch.
        reply_to: ReplyTo,
        /// The closing task whose result made this proposal.
        proposer: u64,
        /// Exact still-pending result proposal being accepted in this decision.
        proposal: u64,
        /// Whole authorized batch with its final funding source.
        batch: Box<[New]>,
    },
    /// `Due`-to-`Preparing` admission, producing `Done` or `Refused`.
    Prepare {
        /// Destination owed one `Done` or `Refused` terminal.
        reply_to: ReplyTo,
        /// `Live` `Due` task to enter `Preparing`; other phases refuse without mutation.
        task: u64,
    },
    /// `Preparing`-to-`Claimed` admission with a strictly increasing root attempt, producing `Done`
    /// or `Refused`.
    Claim {
        /// The session verdict prepared at a standing task's wake, committed only by this claim.
        session_end: Option<crate::SessionEnd>,
        /// The prepared run's reasons (jig/docs/design/domain/tasks/messages.md, 6).
        reasons: crate::Reasons,
        /// The kinds its preparation offers (jig/docs/design/domain/tasks/messages.md, 6).
        offered: crate::Reasons,
        /// Destination owed one `Done` or `Refused` terminal.
        reply_to: ReplyTo,
        /// `Live` `Preparing` task whose attempt is being claimed.
        task: u64,
        /// Root-issued nonzero identity strictly greater than the latest claim; success resets
        /// attempt turn/expense baselines.
        attempt: u64,
        /// Whether the prepared run includes an earlier committed transcript.
        resumed: bool,
        /// Why the claim's preparation began fresh with an earlier window.
        fresh: Option<crate::Fresh>,
        /// Root-checked allowance reserved from the task at this claim.
        budget: u64,
        /// Checked main account; success keeps it in this same claim decision.
        account: Option<u32>,
        /// Sources of the prepared agent run; none for a job.
        sources: Option<crate::ClaimSources>,
        /// Complete set of held resources this run will write, in one atomic claim.
        writes: Box<[Name]>,
    },
    /// The host's guide report for the indicated attempt.
    RunGuides {
        task: u64,
        attempt: u64,
        guides: Box<[crate::Guide]>,
    },
    /// A connector has read a lost run's resource afresh; its slot may now be freed.
    ReadAfresh {
        resource: Name,
    },
    /// Claim the writer slot for a connector-owned effect until it settles.
    EffectInFlight {
        reply_to: ReplyTo,
        task: u64,
        resource: Name,
        entry: u64,
    },
    /// A connector-owned effect's terminal frees its matching writer slot.
    EffectSettled {
        resource: Name,
        entry: u64,
    },
    /// Notification changing only the matching `Claimed` attempt to `Running`; absent/stale inputs
    /// are ignored and no reply is owed.
    Started {
        /// `Live` claimed task the root reports as started.
        task: u64,
        /// Matching claimed attempt; absent tasks and stale phases/attempts are ignored.
        attempt: u64,
    },
    /// Admit a root-fenced activation terminal with explicit expense provenance; emits one
    /// `Acknowledged` or `Refused` and atomic resulting saves.
    Activation {
        /// The core verified that the task tree's requester reads its project at admission.
        reads_project: bool,
        /// Destination owed one `Acknowledged` or `Refused` terminal; root delays exposure until
        /// the decision is durable.
        reply_to: ReplyTo,
        /// `Live` task whose activation terminal is admitted.
        task: u64,
        /// Root-fenced current attempt, including a stopped run preserved while held or closing.
        attempt: u64,
        /// Bounded root-supplied activation outcome; tasks normalizes invalid outcomes. A
        /// park/retry need not end the task.
        end: End,
        /// Full set of resource names with saved state; absent preserves the previous set.
        saved: Option<Box<[SavedResource]>>,
        /// `Priced` worker cumulative expense or unpriced topology/readiness cause; root owns exact
        /// replay fencing.
        cause: Cause,
    },
    /// Notification pausing only a `Preparing` task; does not consume a failure try and has no
    /// reply.
    /// Preparation became stale or waits for an account/host; return to due without a retry timer.
    PreparationWaiting {
        task: u64,
    },
    /// Pause preparation without spending a try, retaining its reason.
    PreparationPaused {
        task: u64,
        reason: Box<[u8]>,
    },
    /// Count a preparation failure, retaining its reason through backoff and hold.
    PreparationCounted {
        task: u64,
        class: Class,
        reason: Box<[u8]>,
    },
    /// Authorized notification preserving prior state and issuing `Stop` for any live run;
    /// absent tasks and unchanged reasons have no output.
    Hold {
        /// `Live` task to hold while retaining its prior phase; a changed reason
        /// replaces the current hold, and advances its revision.
        task: u64,
        /// Authorized or root-classified hold reason; tasks judges no policy here.
        why: Hold,
    },
    /// Connector-classified hold with its exact opaque resource or effect identity.
    HoldFrom {
        task: u64,
        why: Hold,
        source: Box<HoldSource>,
    },
    /// Root notification that prior effects settled; matching `Effects` stages advance.
    EffectsSettled {
        /// Closing task whose prior effects have settled.
        task: u64,
    },
    /// A connector kept one failed task's resource for its tree root until that root closes.
    Retained {
        task: u64,
        root: u64,
        holding: Holding,
    },
    /// Root notification after cleanup finishes; matching `Releases` stages advance,
    /// including held prior closing, without lifting a hold.
    Settled {
        /// Task whose root-owned cleanup obligations settled; only `Releases` stages advance,
        /// including preserved closing while held.
        task: u64,
    },
    /// Root selects the next party and numbers the news at the first stall only.
    PersonStallNotice {
        task: u64,
        message: Option<u64>,
        offer: Option<PersonAddress>,
    },
    /// Root projects the clock again for a restored wait whose notice already ran.
    RearmPersonStall {
        task: u64,
        may_pass: bool,
    },
    /// Root numbers each unanswered-question notice once.
    QuestionEndedNotice {
        task: u64,
        notices: Box<[QuestionNotice]>,
    },
    /// Root numbers the reminder for a run that finished before asking.
    QuestionOwedNotice {
        task: u64,
        message: u64,
    },
    /// Core-numbered notice after a park with nothing pending.
    NothingAwaitedNotice {
        task: u64,
        message: u64,
    },
    /// Recheck left-alone parks after routing the change's own wakes.
    RecheckStuck,
    /// Core supplies the oldest open question's bounded reason; no message is numbered.
    QuestionParked {
        task: u64,
        question: u64,
        reason: Option<Box<[u8]>>,
    },
    /// A caller cancels its own unanswered question.
    CancelQuestion {
        reply_to: ReplyTo,
        task: u64,
        question: u64,
    },
    /// Root numbers the notice that a kept task goes on (domain/tasks/lifecycle-endings.md, 1, and domain/tasks/messages.md, 1).
    MetNotice {
        task: u64,
        dependency: u64,
        removed: bool,
        by: Party,
        message: u64,
    },
    /// Admit one `Live`/`Ledger` row during restoring; invalid input permanently fails restoration
    /// with `RestoreRefused`.
    Restore {
        /// One bounded `Live` or `Ledger` row while restoring; `Ended` and
        /// invalid/duplicate rows fail restoration.
        record: Stored,
    },
    /// From the parent, once every record its checks read is restored,
    /// the connectors' among them, and before the store switches
    /// (domain/engine/restart.md): the whole set's checks, its links, stubs,
    /// writers, waits and funders. A broken set fails the start; a
    /// waiter left on a circle of waits is held, its writes landing in
    /// the copy (domain/tasks/holds.md, 7).
    Check,
    /// Replay bounded activations/adoptions/closing outputs after `Check` passed.
    /// Before `Check`, refuses as out of order. Repetition after ready/failed is ignored.
    Restored,
}

/// Tasks-to-root persistence, lifecycle and terminal reply outputs; root groups resulting
/// saves/erases with effects and delays outward replies until durability. (domain/tasks/lifecycle.md).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// Numbers recorded, and numbers naming none of this task's unrecorded words
    /// (jig/docs/design/domain/tasks/messages.md, 2).
    WordsRecorded {
        reply_to: ReplyTo,
        task: u64,
        recorded: Box<[u64]>,
        unknown: Box<[u64]>,
    },
    /// Placed hold lifted; count of tasks released from their held phase.
    Lifted {
        reply_to: ReplyTo,
        released: u32,
    },
    /// The selected live agent tasks moved to the newest, and those that could not move.
    MovedToNewest {
        moved: u32,
        stayed: u32,
    },
    /// The last live holder of this settings revision ended in this step.
    Released {
        revision: u32,
    },
    ReleasedProjectCharters {
        project: u32,
        revision: u64,
    },
    /// Nothing was made: the first failed operation and its owner diagnostic.
    CompoundRefused {
        reply_to: ReplyTo,
        at: u32,
        problem: Problem,
    },
    /// No part of this procedure step changed task state.
    ProcedureRefused {
        task: u64,
        step: u64,
        problem: Problem,
    },
    /// The complete requested set became held in this commit.
    Taken {
        task: u64,
        holdings: Box<[Holding]>,
    },
    /// One task entered or changed position in a resource's waiting queue.
    Waiting {
        task: u64,
        resource: Name,
        place: u32,
    },
    /// A writer slot was busy; the caller may pause and retry without a failed attempt.
    WriterWaiting {
        reply_to: ReplyTo,
        task: Option<u64>,
        resource: Name,
    },
    /// Terminal for one admitted person-origin goal proposal.
    PersonProposed {
        reply_to: ReplyTo,
        proposal: u64,
    },
    /// Terminal for one person-origin goal decision.
    PersonProposalDecided {
        reply_to: Option<ReplyTo>,
        proposer: u64,
        number: u64,
        outcome: crate::ProposalOutcome,
    },
    /// The core procedure needs one whole root-numbered template batch in this period.
    RecurringDue {
        task: u64,
        period: u64,
        members: u32,
    },
    /// A changed requester tree asks root to recheck the current proposal recipient from the nearest holder.
    ProposalRerouteNeeded {
        proposer: u64,
        proposal: u64,
    },
    /// Root supplies one message number for the elapsed person-task wait.
    PersonStallDue {
        task: u64,
    },
    /// A restored wait with a notice may still have a party to pass up to.
    PersonStallRestored {
        task: u64,
    },
    /// One continuation for each asker whose questions ended unanswered.
    QuestionEndedDue {
        task: u64,
        questions: Box<[u64]>,
    },
    /// Root supplies the next obligation reminder number.
    QuestionOwedDue {
        task: u64,
    },
    /// Number the always-waking notice for an idle result-ended task.
    NothingAwaitedDue {
        task: u64,
    },
    /// Route the change's own wakes before judging the affected parks.
    RecheckStuck,
    /// Finish an exhausted park with words from the core's question cache.
    QuestionParked {
        task: u64,
        question: crate::OpenQuestion,
        chain: Box<[crate::StuckLink]>,
    },
    /// Make the idle task's closer instead of numbering a notice (domain/tasks/lifecycle-endings.md, 3).
    CloserDue {
        task: u64,
    },
    /// Root supplies one message number for the notice that a kept task goes on (domain/tasks/lifecycle-endings.md, 1, and domain/tasks/messages.md, 1).
    MetDue {
        task: u64,
        dependency: u64,
        removed: bool,
        by: Party,
    },
    /// One nonfinal held decision passed its configured wait.
    EscalationStalled {
        task: u64,
        revision: u64,
        holder: crate::EscalationHolder,
    },
    /// One nonfinal holder's wait bound elapsed; root chooses the next holder.
    ProposalStalled {
        proposer: u64,
        proposal: u64,
        holder: crate::ProposalHolder,
    },
    /// Exact semantic proposal decision terminal for a named call or keyed person request.
    ProposalDecided {
        reply_to: ReplyTo,
        proposer: u64,
        number: u64,
        outcome: crate::ProposalOutcome,
    },
    /// Root allocates a commit-order message number for this subscribed state change.
    Notify {
        task: u64,
        subscription: u64,
        target: u64,
        state: NoticeState,
        words: Box<[u8]>,
    },
    /// Root allocates a commit-order message number for this due timer.
    Timer {
        task: u64,
        subscription: u64,
    },
    /// An ending task releases a connector topic interest in the same decision.
    EndTopic {
        task: u64,
        subscription: u64,
        connector: u16,
    },
    /// Accepted person words; root answers the keyed request after the inbox write commits.
    Sent {
        reply_to: ReplyTo,
        task: u64,
        word: Word,
    },
    /// Resolve an unrecoverable outbox entry with the release in one decision.
    ResolveUncertain {
        task: u64,
        entry: u64,
        again: bool,
    },
    /// Root relays this whole committed message through the fleet to the current run.
    Relay {
        task: u64,
        attempt: u64,
        previous: Option<u64>,
        word: Word,
    },
    /// Terminal read-only `SetRoles` preflight; snapshot is owned transient root
    /// context, not retained state.
    EscalationsInspected {
        /// Echoed stage-local right.
        reply_to: ReplyTo,
        /// At most `Limits::tasks` `Waiting` contexts without owned rejection
        /// reasons; only `NotReady` can refuse.
        result: Result<Box<[crate::EscalationContext]>, Refusal>,
    },
    /// Terminal for `RecheckEscalations` after its existing `EscalationNeeded` outputs; root
    /// completes the keyed role flight after routing them.
    EscalationsRechecked {
        /// Echoed stage-local right, consumed once.
        reply_to: ReplyTo,
        /// Success or `NotReady` before any recheck output
        result: Result<(), Refusal>,
    },
    /// Tasks asks root to resolve its new held person chat in this atomic
    /// decision; no outward notice precedes commitment.
    EscalationNeeded {
        /// Bounded held context; root drops it after routing
        context: Box<crate::EscalationContext>,
    },
    /// One semantic decision terminal; root commits accepted child writes,
    /// its history and people's keyed answer together.
    EscalationDecided {
        /// Echoed root correlation, consumed once.
        reply_to: ReplyTo,
        task: u64,
        revision: u64,
        /// Accepted semantic transition or unchanged refusal.
        outcome: crate::EscalationOutcome,
    },
    /// One successful `Make` terminal reporting the complete batch.
    Made {
        /** Original `Make` destination for its one successful terminal reply. */
        reply_to: ReplyTo,
        /** All `new` task numbers in input order, bounded by `Limits::batch`; no partial creation is reported. */
        tasks: Box<[u64]>,
    },
    /// One reply-bearing entrance/lifecycle/financial refusal; no requested mutation was admitted.
    Refused {
        /** Original reply-bearing event destination, consumed once on refusal. */
        reply_to: ReplyTo,
        /** Admission location/reason; requested mutation was not made. */
        problem: Problem,
    },
    /// One successful `funding`, preparation or claim terminal.
    Done {
        /** Original successful `funding`/preparation/claim destination, consumed once. */
        reply_to: ReplyTo,
    },
    /// One activation-terminal reply; durability and exact transport replay proof belong to root.
    Acknowledged {
        /** Original activation-terminal destination; root durability barrier precedes outward acknowledgement. */
        reply_to: ReplyTo,
        /** Task whose lifecycle terminal was admitted or already recorded. */
        task: u64,
        /** Fenced activation attempt. */
        attempt: u64,
        /** New lifecycle admission or existing last-answer identity; exact payload replay is separately verified by root. */
        accepted: Accepted,
        /// The newly admitted terminal parks; an already admitted terminal retains its proof.
        parked: bool,
    },
    /// One newly admitted turn reply; root saves its transcript with the charged task state before
    /// outward acknowledgement.
    TurnAcknowledged {
        /** Original `new`-turn destination; root commits transcript and charged task row together before acknowledging. */
        reply_to: ReplyTo,
        /** Task whose turn and expense delta were admitted. */
        task: u64,
        /** Current claimed attempt identity. */
        attempt: u64,
        /** New contiguous turn committed in task state. */
        turn: u32,
        /** Current charged-turn route emits `Accepted::New`; root owns replay acknowledgement without re-entering this route. */
        accepted: Accepted,
    },
    /// Bounded semantic preparation request to root, not a rendered brief or a raw mutable
    /// task peek.
    Activate {
        /** Owned bounded semantic preparation snapshot; root selects the route and drops the snapshot on claim/failure. */
        context: Box<RunContext>,
    },
    /// Ask root/fleet to stop one exact live attempt; its lifecycle terminal is still owed.
    Stop {
        /** Task whose live activation must be stopped by root/fleet. */
        task: u64,
        /** `Exact` live attempt to stop; its terminal remains owed before closing can advance. */
        attempt: u64,
    },
    /// Ask root/fleet to reconcile a restored claim and highest committed turn.
    Adopt {
        /** `Restored` live task whose claim the root must reconcile with fleet. */
        task: u64,
        /** `Restored` committed attempt identity to adopt; no `new` attempt is minted here. */
        attempt: u64,
        /** Highest committed turn in the restored attempt, supplied for fleet reconciliation. */
        kept: u32,
    },
    /// Ask root to settle prior effects, then return `EffectsSettled`; emitted only
    /// after delegates and funded allocations settle.
    Close {
        /** Task whose delegates/financial allocations have settled and root closing obligations must finish. */
        task: u64,
        /** Bounded pending final ending; root uses it to plan cleanup after effect settlement. */
        ending: Ending,
    },
    /// Ask root to release this task's resources after prior effects settle.
    Release {
        task: u64,
        ending: Ending,
    },
    /// One newly ended requester-identified notification emitted with the ended record and exact
    /// financial posting; current root exposes person result notices, with no task-requester inbox
    /// route or child delivery credit.
    Ended {
        /** Task removed from the live arena and saved as a retained ended row. */
        task: u64,
        /** Requester identifying this notification; the parent routes person result notices. */
        requester: Party,
        /** Bounded final result/reason; emitted with ended/funder writes in one root decision. */
        ending: Ending,
        /** Whether the delegate result can wake the requester as usual. */
        wake: bool,
    },
    /// Owned persistence output for the parent's current atomic decision.
    Save {
        /** Owned row joining the root's atomic decision; not an `IO` submission or a durable acknowledgement by itself. */
        record: Stored,
    },
    /// Logical-row removal for the parent's current atomic decision.
    Erase {
        /** Row removal joining the same atomic decision as related saves and outputs. */
        key: Key,
    },
    /// Terminal startup failure; this instance cannot become ready through more
    /// `Restore`/`Restored` inputs.
    RestoreRefused {
        /** Terminal startup validation failure; this domain instance remains unready. */
        problem: Problem,
    },
}

/// Root's expense provenance for a lifecycle terminal; tasks owns authentic delta/accounting
/// admission while root owns exact transport replay proofs.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    /// Worker terminal with authentic cumulative expense: preflight lifecycle and eventual
    /// financial arithmetic before admitting the `new` delta together.
    Priced {
        /// Authentic whole attempt expense, monotonic relative to `run_spent` and preflighted
        /// against eventual funding-chain representability.
        cumulative: u64,
    },
    /// Loss, fleet refusal or invalid-answer normalization supplied by root; performs lifecycle
    /// admission without charging and still owes one terminal reply.
    Unpriced,
    /// The core cancelled a waiting attempt at a turn boundary.
    Parked { cumulative: Option<u64> },
}

/// Authority and accounting view for a run's delegation call.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DelegationContext {
    pub project: u32,
    pub requester: Party,
    /// A held or closing ancestor cannot take a proposal decision.
    pub deciding: bool,
    pub authority: Authority,
    pub numbers: Numbers,
    pub tasks_left: u32,
}

/// Current connector access, independent of the resource's hold kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceAccess {
    /// Owned or participating writes can be checked against grants.
    Writable,
    /// Reads remain available, but writes are forbidden.
    Context,
    /// Neither effects nor holds are available.
    Unavailable,
}

/// The generic inbox inputs consumed by a connector procedure (domain/tasks/messages.md, 1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProcedureInput {
    News,
    Result(ResultKind),
}

/// Classify the message for inbox admission, activity and batching.
#[must_use]
pub fn message_family(kind: &MessageKind) -> MessageFamily {
    match kind {
        MessageKind::Words => MessageFamily::Spoken(Spoken::Words),
        MessageKind::Question => MessageFamily::Spoken(Spoken::Question),
        MessageKind::Answer { question } => MessageFamily::Spoken(Spoken::Answer { question: *question }),
        MessageKind::Notice { subscription, target, state } => {
            MessageFamily::Hint(Hint::Notice { subscription: *subscription, target: *target, state: *state })
        }
        MessageKind::Timer { subscription } => MessageFamily::Hint(Hint::Timer { subscription: *subscription }),
        MessageKind::News { subscription, class } => {
            MessageFamily::Hint(Hint::News { subscription: *subscription, class: *class })
        }
        MessageKind::PersonStalled { .. } => MessageFamily::Slot(Slot::PersonStalled),
        MessageKind::Amendment { .. } => MessageFamily::Separate(Separate::Amendment),
        MessageKind::FinishRequested { .. } => MessageFamily::Separate(Separate::FinishRequested),
        MessageKind::Met { .. } => MessageFamily::Separate(Separate::Met),
        MessageKind::NothingAwaited => MessageFamily::Separate(Separate::NothingAwaited),
        MessageKind::Result(_) => MessageFamily::Result,
        MessageKind::EffectNotMade { .. } => MessageFamily::Slot(Slot::EffectNotMade),
        MessageKind::SettingsDecision { .. } => MessageFamily::Slot(Slot::SettingsDecision),
        MessageKind::ProposalDecision { .. } => MessageFamily::Slot(Slot::ProposalDecision),
        MessageKind::QuestionAnswered { .. } => MessageFamily::Slot(Slot::QuestionAnswered),
        MessageKind::QuestionOwed => MessageFamily::Slot(Slot::QuestionOwed),
        MessageKind::QuestionEnded { .. } => MessageFamily::Slot(Slot::QuestionEnded),
        MessageKind::Escalation { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::PassedQuestion { .. }
        | MessageKind::PassedAnswer { .. } => MessageFamily::Virtual,
    }
}

/// Select connector news and delegate results from a procedure's inbox.
#[must_use]
pub fn procedure_input(kind: MessageKind) -> Option<ProcedureInput> {
    match kind {
        MessageKind::News { .. } => Some(ProcedureInput::News),
        MessageKind::Result(result) => Some(ProcedureInput::Result(result)),
        MessageKind::Escalation { .. }
        | MessageKind::Proposal { .. }
        | MessageKind::PassedQuestion { .. }
        | MessageKind::PassedAnswer { .. }
        | MessageKind::SettingsDecision { .. }
        | MessageKind::ProposalDecision { .. }
        | MessageKind::EffectNotMade { .. }
        | MessageKind::Words
        | MessageKind::Met { .. }
        | MessageKind::FinishRequested { .. }
        | MessageKind::Amendment { .. }
        | MessageKind::Question
        | MessageKind::Answer { .. }
        | MessageKind::Notice { .. }
        | MessageKind::QuestionAnswered { .. }
        | MessageKind::QuestionOwed
        | MessageKind::NothingAwaited
        | MessageKind::QuestionEnded { .. }
        | MessageKind::Timer { .. }
        | MessageKind::PersonStalled { .. } => None,
    }
}

/// Whether this message asks an agent to finish (domain/hosts/README.md, section 7).
#[must_use]
pub fn finish_requested(word: &Word) -> bool {
    message_family(&word.kind) == MessageFamily::Separate(Separate::FinishRequested)
}

/// Whole admitted payload, including the opaque state of an approval result.
#[must_use]
pub fn word_payload_bytes(word: &Word) -> Option<usize> {
    match word.kind.result() {
        Some(ResultKind::Approved { state, words_present }) => {
            if !words_present && !word.words.is_empty() {
                return None;
            }
            state.len().checked_add(word.words.len())
        }
        Some(ResultKind::Verdict { fields, .. }) => crate::verdict::fields_bytes(fields)?.checked_add(word.words.len()),
        Some(
            ResultKind::Job
            | ResultKind::NoChange
            | ResultKind::Report
            | ResultKind::Change { .. }
            | ResultKind::Failed
            | ResultKind::Cancelled,
        )
        | None => Some(word.words.len()),
    }
}

pub(crate) const fn valid_hold(hold: Hold) -> bool {
    match hold {
        Hold::Dependency(task) => task != 0,
        Hold::Placed { hold } => hold != 0,
        Hold::Uncertain { entry } => entry != 0,
        Hold::PoolLost { pool } => pool != 0,
        Hold::Exceeded(_)
        | Hold::Failures(_)
        | Hold::Stalled
        | Hold::Stopped
        | Hold::Drift
        | Hold::ResourceUnavailable
        | Hold::ResourceAccess
        | Hold::Effects
        | Hold::EffectFailed
        | Hold::Budget
        | Hold::Deadline
        | Hold::HoldsWaited
        | Hold::Procedure
        | Hold::Circle
        | Hold::Waited => true,
    }
}
