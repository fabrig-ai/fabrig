//! Task and batch values carried by operations (jig/docs/design/domain/connectors.md, 2).

mod activation;
mod admission;
pub mod party;
mod waits;

pub use activation::{
    Added, CallKey, Carried, DelegateState, Endings, EscalationDecider, EscalationHolder, InvalidResult, Mode, Payer,
    PersonOffer, PersonStallNotice, PersonWait, REASON_NAMES, Reasons, RetryNotice, Revisions, RunContext, Scope,
    Tries,
};
pub use admission::{
    AccountWhy, Capacity, CapacityAction, CapacityCause, LlmFault, Problem, QuestionAnswerer, Refusal, Status,
};
pub use party::{Act, Outcome};
pub use waits::{
    AgentShortfall, AtBound, Figures, HelloRefusal, HostKind, JobShort, JobWhy, Place, TaskRunWait, TaskSourceState,
    TaskWaitRefused, TaskWaitWhy, Thing, Unbounded, Unread, WaitKind, WaitObject,
};

use crate::parts::Part;
use crate::resources::{Holding, Name};
use crate::topics::NewsClass;
use alloc::boxed::Box;
use skein_lib::{Decimal, Duration, Wall};

/// Application-made agent goal delegated by a live task (jig/docs/design/domain/authority.md, 6.3).
#[derive(Hash, Debug)]
pub struct SeededGoal {
    pub charter: u32,
    pub spec: Spec,
    pub contract: Contract,
    pub authority: Authority,
    pub llm: Option<LlmChoice>,
    pub budget: u64,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: Flags,
    pub holdings: Box<[Holding]>,
}

/// People's project membership label; the root translates it to authority policy and performs the
/// action checks. (domain/people/README.md, section 4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    /// `Owner` membership; may start chats here, while broader owner actions remain root policy.
    Owner,
    /// Maintainer membership; may start chats here, with authority checked by the root.
    Maintainer,
    /// Member membership; may start chats here within the root's policy and funding check.
    Member,
    /// Observer membership; creation is refused and its keyed role outcome is saved.
    Observer,
    /// A role numbered by this project's policy, beyond the four defaults.
    Policy { role: u32 },
}

impl Role {
    /// Number used by authority and role-addressed task entries.
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::Owner => 0,
            Self::Maintainer => 1,
            Self::Member => 2,
            Self::Observer => 3,
            Self::Policy { role } => role,
        }
    }

    /// Resolve a configured policy role to a membership label.
    #[must_use]
    pub const fn from_number(number: u32) -> Self {
        match number {
            0 => Self::Owner,
            1 => Self::Maintainer,
            2 => Self::Member,
            3 => Self::Observer,
            role => Self::Policy { role },
        }
    }
}

/// A closed wake rule for task words and subscription hints (domain/tasks/messages.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WakeRule {
    Never,
    Immediate,
    Batch {
        count: u32,
        age: Duration,
    },
    /// At most once in `span` from the task's last run's start (domain/tasks/messages.md, 3).
    Paced {
        span: Duration,
    },
}

/// How delegate result messages wake their requester.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResultsWake {
    Never,
    Each,
    LastOrFailure,
}

/// Inert creator-selected wake policy, bounded at task admission.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WakePolicy {
    pub words: WakeRule,
    pub notices: WakeRule,
    pub news: WakeRule,
    pub results: ResultsWake,
    pub questions: bool,
    pub answers: bool,
    pub timers: bool,
}

impl WakePolicy {
    pub const DEFAULT: Self = Self {
        words: WakeRule::Immediate,
        notices: WakeRule::Immediate,
        news: WakeRule::Immediate,
        results: ResultsWake::LastOrFailure,
        questions: true,
        answers: true,
        timers: true,
    };
}

/// State reported by a task watch, distinct from a terminal result.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NoticeState {
    Held { revision: u64 },
    Done,
    Failed,
    Cancelled,
    Unreadable,
}

/// Kind of a durable task-inbox message (domain/tasks/lifecycle.md, section 7 and domain/tasks/messages.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MessageKind {
    /// Virtual, durable held decision entry projected from the held task.
    Escalation { task: u64, revision: u64 },
    /// Virtual inbox entry for a pending proposal; its reason is in `Word::words`.
    /// The proposal owns its durable state and takes no inbox room.
    Proposal { proposer: u64, proposal: u64, kind: ProposalKind },
    /// A person delegate passed its waiting deadline, delivered once to its requester.
    PersonStalled { task: u64 },
    /// Final accepted or rejected proposal, delivered to its proposer.
    ProposalDecision { proposal: u64, accepted: bool },
    /// A settings proposal outcome, delivered using its reserved inbox room.
    SettingsDecision { proposal: u64, outcome: SettingsOutcome },
    /// Whole words from a person or referenced task.
    Words,
    /// A person settled the numbered uncertain effect as not made.
    EffectNotMade { entry: u64 },
    /// Latest committed amendment, always waking and merging by task.
    Amendment { revision: u64 },
    /// A cancel grace asks its executor to finish, in its own reserved slot.
    FinishRequested { revision: u64 },
    /// A bounded question for which the sender keeps answer room.
    Question,
    /// Answer to a named question previously asked of this sender.
    Answer { question: u64 },
    /// Merged task state hint for one standing interest.
    Notice { subscription: u64, target: u64, state: NoticeState },
    /// Merged timer fire for one standing interest.
    Timer { subscription: u64 },
    /// Merged connector hint routed by its topic.
    News { subscription: u64, class: NewsClass },
    /// A delegate's terminal result, sent with its end.
    Result(ResultKind),
    /// The core's notice that a kept task's last pending dependency is done,
    /// or was removed by an amendment: always waking, one per task, merged
    /// (domain/tasks/lifecycle-endings.md, 1, and domain/tasks/messages.md, 1). `dependency` is the one whose end or
    /// removal emptied the pending set; of several removed at once, the
    /// first in the old list.
    Met { dependency: u64, removed: bool },
    /// The question ended without an answer; its asker may go on.
    QuestionEnded { question: u64 },
    /// Its person answered the question in their inbox; `Word::from` names its asker
    /// (domain/people/README.md, 5; domain/tasks/lifecycle-endings.md, 1).
    QuestionAnswered { question: u64 },
    /// A question offered to another task. The word's number orders the offer;
    /// the question number is the one its answer names.
    PassedQuestion { question: u64 },
    /// A question answered by someone other than the task it was asked of.
    PassedAnswer { question: u64 },
    /// Its requester is owed a question before a done finish (domain/tasks/lifecycle-endings.md, 1).
    QuestionOwed,
    /// Nothing remains to await after a result-ended task parks (domain/tasks/lifecycle.md, 2).
    NothingAwaited,
}

impl MessageKind {
    /// Borrow the delegate's terminal shape, when this is a result message.
    #[must_use]
    pub fn result(&self) -> Option<&ResultKind> {
        match self {
            Self::Result(result) => Some(result),
            Self::Escalation { .. }
            | Self::Proposal { .. }
            | Self::PassedQuestion { .. }
            | Self::PassedAnswer { .. }
            | Self::PersonStalled { .. }
            | Self::ProposalDecision { .. }
            | Self::SettingsDecision { .. }
            | Self::Words
            | Self::EffectNotMade { .. }
            | Self::Amendment { .. }
            | Self::FinishRequested { .. }
            | Self::Question
            | Self::Answer { .. }
            | Self::Notice { .. }
            | Self::Timer { .. }
            | Self::News { .. }
            | Self::Met { .. }
            | Self::QuestionEnded { .. }
            | Self::QuestionAnswered { .. }
            | Self::QuestionOwed
            | Self::NothingAwaited => None,
        }
    }
}

/// Terminal shape carried with a delegate's bounded message words.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResultKind {
    Job,
    /// Exact approved state; presence distinguishes absent from empty explanation words.
    Approved {
        state: Box<[u8]>,
        words_present: bool,
    },
    /// Successful conclusion that the requested change does not apply.
    NoChange,
    Report,
    Verdict {
        code: u32,
        fields: Box<[ResultField]>,
    },
    Change {
        connector: u16,
        kind: u16,
        resource: u64,
    },
    Failed,
    Cancelled,
}

/// One durable whole message in a task's bounded inbox
/// (domain/tasks/messages.md, section 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Word {
    /// Root-issued commit-order message identity.
    pub number: u64,
    /// Root-verified sender, checked against requester or live delegate.
    pub from: Party,
    /// Reason this message entered the inbox.
    pub kind: MessageKind,
    /// Whole bounded words, never cut while in the inbox.
    pub words: Box<[u8]>,
    /// Injected time of admission, for oldest-first reads.
    pub at: Wall,
    /// Number of hints merged into this whole inbox entry.
    pub hits: u32,
    /// A batching threshold already reached, retained across restart.
    pub eligible: bool,
}

/// Root-verified requester/creator identity; requester topology and financial source are
/// separate values. (domain/tasks/README.md, sections 2–3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Party {
    /// Task requester/creator whose delegation topology this child owns.
    Task(
        /** Deployment task number of the creator/requester; task requesters must remain live until their delegates end. */
         u64,
    ),
    /// Person requester/creator authenticated by the root.
    Person(/** Authenticated deployment person number supplied by the root. */ u64),
    /// Deployment-owned work for one project, authorized by the root.
    Deployment {
        /** Project on whose behalf deployment configuration starts the task. */
        project: u32,
    },
}

impl Party {
    /// The numeric party label carried by a host message (domain/hosts/README.md, 7).
    #[must_use]
    pub fn sender(self) -> Box<[u8]> {
        match self {
            Self::Task(number) | Self::Person(number) => Box::from(Decimal::of(number).as_bytes()),
            Self::Deployment { .. } => Box::from(b"deployment".as_slice()),
        }
    }
}

/// Opaque person-task destination; root authenticates the identity or role membership.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PersonAddress {
    /// One named person may answer without a role claim.
    Person(u64),
    /// Any current holder of a project role may take and answer.
    Role(u32),
}

/// Task executor selected by the root. The procedure's owner interprets its opaque code.
/// (domain/tasks/README.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Executor {
    /// Configured agent charter.
    Agent {
        /** Configured agent charter number, checked against `Domain`'s admitted charter table. */
        charter: u32,
    },
    /// Connector or core procedure stepped by its owner through the root.
    Procedure { connector: u16, code: u32 },
    /// A person or any current holder of a project role answers in the people inbox.
    Person(PersonAddress),
    /// Application job kind (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
    Job { kind: u32 },
}

/// What to do when a new period arrives before the preceding batch has ended.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RecurringOverlap {
    /// Let the due period pass without another batch.
    Skip,
    /// Remember the latest due period and make its batch after the old one ends.
    Wait,
}

/// A core procedure's durable batch template, with local member numbers starting at one.
/// The live procedure's authority is its per-period budget ceiling.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RecurringTemplate {
    /// Stable deployment configuration identity within a project, used to fence restarts.
    pub key: u32,
    pub batch: Box<[New]>,
    pub overlap: RecurringOverlap,
}

/// One addressed word admitted to a task inbox (jig/docs/design/domain/tasks/messages.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Message {
    pub task: u64,
    pub word: Word,
}

/// Bounded semantic parameter carrier; tasks does not interpret names or resources, and the
/// current root chat constructor supplies no parameters. (domain/tasks/README.md, section 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Parameter {
    /// Numeric specification carrier; current chat construction supplies no parameters.
    Number {
        /** Opaque semantic parameter name, carried without interpretation by tasks. */
        name: u32,
        /** Numeric parameter value, carried without interpretation here. */
        value: u64,
    },
    /// Bounded UTF-8 text specification carrier; current chat construction supplies no parameters.
    Bytes {
        /** Opaque semantic parameter name, carried without interpretation by tasks. */
        name: u32,
        /** Owned UTF-8 text bytes; these and `Spec::words` share `Limits::spec_bytes`. */
        value: Box<[u8]>,
    },
    /// Connector/resource carrier; no connector lookup or effect is performed by tasks.
    Resource {
        /** Opaque semantic parameter name, carried without interpretation by tasks. */
        name: u32,
        /** Connector identity carried without lookup or interpretation here. */
        connector: u16,
        /** Opaque semantic resource identifier; tasks does not load connector data. */
        resource: u64,
    },
}

/// Owned task description supplied by root, bounded at admission (domain/tasks/README.md, section 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spec {
    /// Application bytes, present exactly for a job executor (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
    pub job: Option<Box<JobSpec>>,
    /// Owned UTF-8 specification words; combined with text-valued parameters, bounded by
    /// `Limits::spec_bytes`.
    pub words: Box<[u8]>,
    /// Parameters bounded by `Limits::parameters`; tasks does not interpret their names, and
    /// current root chat construction supplies an empty slice.
    pub parameters: Box<[Parameter]>,
    /// Retained input identities, bounded by `Limits::inputs` on creation and restore.
    pub inputs: Box<[u64]>,
    /// Immutable obligation to ask its requester a question (domain/tasks/lifecycle-endings.md, 1).
    pub asks: bool,
}

/// A job task's application bytes and per-attempt deadline (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobSpec {
    pub bytes: Box<[u8]>,
    pub deadline: Duration,
}

/// One required named machine field and its value byte cap.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldRule {
    pub name: Box<[u8]>,
    pub max: u32,
}

/// A named opaque machine value retained with a verdict.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResultField {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}

/// One admitted result choice with a unique code and a per-choice word-byte bound.
/// (domain/tasks/README.md, section 3, and domain/tasks/lifecycle.md, section 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Verdict {
    /// Distinct permitted choice code within its contract.
    pub code: u32,
    /// Maximum word bytes for this choice, no greater than `Limits::result_bytes`.
    pub words: u32,
    /// Maximum whole-batch follow-up delegates this choice permits.
    pub followups: u32,
    /// Required closed-list machine fields, whose values remain opaque.
    pub fields: Box<[FieldRule]>,
}

/// Root-supplied bounded result shape; tasks validates returned results against it before terminal
/// admission. (domain/tasks/README.md, section 3, and domain/tasks/lifecycle.md, section 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Contract {
    /// Maximum application result bytes (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
    Job { bytes: u32 },
    /// Approval of exact opaque connector state (domain/people/README.md, section 7).
    Approval { state: Box<[u8]>, part: Part },
    /// Bounded report result.
    Report {
        /** Maximum report bytes, no greater than `Limits::result_bytes`. */
        words: u32,
    },
    /// One of a nonempty admitted list of distinct verdict codes and byte caps.
    Verdict {
        /** Nonempty distinct-code choices, bounded by `Limits::contract_choices`, each with a result-byte bound. */
        choices: Box<[Verdict]>,
    },
    /// Connector/kind-matching result with an opaque resource and bounded report.
    Change {
        /** Required connector identity in a successful change result. */
        connector: u16,

        kind: u16,
        /** Maximum change-report bytes, no greater than `Limits::result_bytes`. */
        words: u32,
        /// Successful no-resource verdicts; distinct codes bounded as a verdict contract's choices.
        choices: Box<[Verdict]>,
    },
}

/// Explicit change-title bytes, boxed beside the result to keep inline carriers small.
/// Admission bounds the words by the result contract (jig/docs/design/domain/tasks/README.md, 3, and jig/docs/design/domain/tasks/lifecycle.md, 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeTitle {
    pub words: Box<[u8]>,
}

impl ChangeTitle {
    /// Own a producer's title; the task entrance checks its byte bound.
    #[must_use]
    pub fn new(words: Box<[u8]>) -> Box<Self> {
        Box::new(Self { words })
    }
}

/// Owned executor outcome checked against the admitted task contract before lifecycle admission;
/// connector interpretation and outward delivery remain with root. (domain/tasks/lifecycle.md, section 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TaskResult {
    /// Opaque application result (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
    Job { bytes: Box<[u8]> },
    /// Exact state approved by the addressed person, with optional explanation.
    Approved { state: Box<[u8]>, words: Option<Box<[u8]>> },
    /// Successful bounded report matching a report contract.
    Report {
        /** Report bytes, bounded by both the admitted contract and `Limits::result_bytes`. */
        words: Box<[u8]>,
    },
    /// Successful admitted verdict code with bounded explanation.
    Verdict {
        code: u32,
        /// Each required field exactly once; values are never read by tasks.
        fields: Box<[ResultField]>,
        /** Explanation bytes bounded by that choice's word cap and `Limits::result_bytes`. */
        words: Box<[u8]>,
    },
    /// Successful matching connector/kind result; tasks performs no connector effect.
    Change {
        /** Connector identity that must equal the admitted change contract. */
        connector: u16,
        /** Connector-defined kind that must equal the admitted change contract. */
        kind: u16,
        /** Opaque resulting resource identifier; root/connector interpretation remains outside tasks. */
        resource: u64,
        /// Producer-supplied title, bounded independently by the contract and `Limits::result_bytes`.
        title: Box<ChangeTitle>,
        /** Change-report bytes bounded by the admitted contract and `Limits::result_bytes`. */
        words: Box<[u8]>,
    },
    /// Successful no-change conclusion under a change contract; claims no resulting resource.
    NoChange {
        /// Explanation bounded by the admitted contract and `Limits::result_bytes`.
        words: Box<[u8]>,
    },
    /// Bounded failure result allowed by every contract; produces an `Ending::Failed`.
    Failure {
        /** Failure-result reason, allowed by every contract and bounded by `Limits::result_bytes`. */
        reason: Box<[u8]>,
    },
}

/// Durable final outcome, or a pending closing outcome; requester notification follows
/// settlement. (domain/tasks/lifecycle.md, sections 1 and 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    /// Checked successful result proposed for final ending; retained while closing and made
    /// final only after settlement.
    Done(/** Successful checked result retained in pending closing or in the retained end record. */ TaskResult),
    /// Failure ending with its bounded reason, pending while closing or final in the retained
    /// record.
    Failed {
        /** Owned failure reason bounded by `Limits::result_bytes`. */
        reason: Box<[u8]>,
    },
    /// Cancellation of descendant work, optionally preserving an already-completed result; no
    /// arbitrary public cancel route is exposed.
    Cancelled {
        /// Party whose decision cancelled this work.
        by: Party,
        /// Wall time when cancellation was decided.
        at: Wall,
        /** Owned cancellation reason bounded by `Limits::result_bytes`. */
        reason: Box<[u8]>,
        /** Optional already-completed result retained with cancellation, separately bounded and checked against the contract. */
        result: Option<TaskResult>,
    },
}

/// Current agent-task activation phase; task and attempt identity fence every external transition.
/// (domain/tasks/lifecycle.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Active {
    /// `Activation` parked; no current wake input is exposed by this contracted child.
    Idle,
    /// An activation is requested and root owns preparation.
    Due,
    /// Root is gathering the brief/resources; a claim or preparation failure follows.
    Preparing,
    /// Root committed the fenced attempt before assignment/start.
    Claimed {
        /** Committed root-issued strictly increasing nonzero attempt identity. */
        attempt: u64,
    },
    /// Root reported the exact claimed attempt started.
    Running {
        /** Current claimed attempt reported as started. */
        attempt: u64,
    },
    /// `Retry`/refusal pause with one projected deadline.
    BackingOff {
        /** Saved wall-time retry threshold, projected once to a monotonic deadline and reprojected on restore. */
        until: Wall,
    },
}

/// Ordered closing obligation; the root confirms effect settlement before resource release.
/// (domain/tasks/lifecycle.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stage {
    /// Wait for the exact live attempt's terminal.
    Run {
        /** `Live` attempt whose terminal must be heard before closing advances. */
        attempt: u64,
    },
    /// Wait for all live delegates and incoming financial allocations to settle.
    Delegates,
    /// `Close` output is owed settlement of all prior effects from root.
    Effects,
    /// `Release` output is owed settlement of resource cleanup from root.
    Releases,
    /// Root closing obligations settled; final financial posting and retained ending can commit.
    /// Unheld `Settled` is transient and refused on live restore; held prior closing may retain it.
    Settled,
}

/// Owned pending ending and closing stage retained until every obligation and financial allocation
/// settles. (domain/tasks/lifecycle.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Closing {
    /// Next unsettled closing obligation.
    pub stage: Stage,
    /// Bounded pending final outcome, preserved through holds and cancellation.
    pub ending: Ending,
}

/// Lifecycle preserved while held; terminal/settlement notifications can update it without lifting
/// the hold. (domain/tasks/lifecycle.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Was {
    /// Prior dependency-waiting phase.
    Waiting,
    /// Prior agent activation phase.
    Active(/** Prior agent activation phase, including its attempt/backoff fence. */ Active),
    /// Prior pending closing state.
    Closing(/** Prior closing stage and owned pending ending. */ Closing),
}

/// Task's complete lifecycle value; ended values occur in retained durable rows rather than the
/// live arena. (domain/tasks/lifecycle.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    /// `Some` live dependency remains; readiness becomes `Due` when `waiting_on` empties.
    Waiting,
    /// `Agent` activation is idle, due, preparing, claimed, running or backing off.
    Active(Active),
    /// Pending ending waits for ordered activation/delegate/root settlement.
    Closing(/** Owned pending ending and next closing obligation. */ Closing),
    /// Prior lifecycle preserved until a release or amendment lifts the hold.
    Held {
        /** Prior phase retained and updated while held; the hold is not lifted by a terminal. */
        was: Was,

        why: Hold,
        /// Exact cause, retained in lifecycle milestones.
        source: Option<Box<HoldSource>>,
    },
    /// Ended outcome; default restoration refuses it in the live arena.
    Ended(/** Final ending; this phase is excluded from the live restore arena. */ Ending),
}

/// What explains a hold beyond its reason: a connector account, a failure cause,
/// a circle of waits, or the waits at a bound (client/domain/objects.md, 5; domain/tasks/lifecycle-holds.md, 1, and domain/tasks/holds.md, 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HoldSource {
    /// Why preparation or activation failed past its allowance, only on `Hold::Failures`.
    Preparation {
        words: Box<[u8]>,
    },
    Resource(Name),
    Effect {
        connector: u16,
        entry: u64,
    },
    Circle(Circle),
    /// The connector's retained account, including an effect's operation, resource and outcome.
    Account {
        connector: u16,
        entry: Option<u64>,
        words: Box<[u8]>,
    },
    /// What the task waited for at its bound, in UTF-8.
    Wait {
        words: Box<[u8]>,
    },
}

/// A circle of waits: tasks each waiting for the next, the last for the first, so that none of
/// them could end (domain/tasks/holds.md, 7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Circle {
    /// The circle's tasks in order, from the one whose new wait would close it, at most
    /// `Limits::circle`.
    pub tasks: Box<[u64]>,
    /// The circle has more tasks than these.
    pub cut: bool,
}

/// Root-reported or task-derived reason to preserve a task's prior lifecycle while stopping its
/// live activation. (domain/tasks/lifecycle-holds.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hold {
    /// A configured bound refused the run's model call (domain/hosts/hub.md, 3).
    Exceeded(Exceeded),
    /// Configured retry allowance was exhausted.
    Failures(/** Failure class whose retry allowance was exhausted. */ Class),
    /// Executor made no progress within its configured stall bound.
    Stalled,
    /// A required dependency failed or was cancelled.
    Dependency(/** `Dependency` that ended failed or cancelled, causing this dependent to be held. */ u64),
    /// Root reports a person's stop or another authorized stop decision.
    Stopped,
    /// The placed hold that holds this task, when no cause of 5.5 does.
    Placed { hold: u64 },
    /// Root reports resource drift requiring a decision; tasks owns no connector drift detection.
    Drift,
    /// Root reports a permanently failed effect requiring a decision.
    Effects,
    /// A connector effect failed for good.
    EffectFailed,
    /// An unrecoverable effect may or may not have reached its system.
    Uncertain { entry: u64 },
    /// Available funding is too small for another run or effect.
    Budget,
    /// Root reports that the permitted deadline prevents another activation.
    Deadline,
    /// A task waited beyond the bound for its required resource holds.
    HoldsWaited,
    /// A connector reports that a named resource is no longer available.
    ResourceUnavailable,
    /// Current adoption forbids a running workspace write.
    ResourceAccess,
    /// The numbered pool lost its allocation while this task held or awaited it.
    PoolLost { pool: u64 },
    /// A procedure asked its requester for a decision.
    Procedure,
    /// Joining a queue, or the waits a release gives back, would close a circle of waits; the
    /// hold's source names the circle, and the hold lifts once it breaks (domain/tasks/holds.md, 7).
    Circle,
    /// A wait reached its bound.
    Waited,
}

impl Hold {
    /// Identity of the independently placed hold, when this is a placed reason.
    #[must_use]
    pub const fn placed(self) -> Option<u64> {
        match self {
            Self::Placed { hold } => Some(hold),
            Self::Exceeded(_)
            | Self::Failures(_)
            | Self::Stalled
            | Self::Dependency(_)
            | Self::Stopped
            | Self::Drift
            | Self::Effects
            | Self::EffectFailed
            | Self::Uncertain { .. }
            | Self::Budget
            | Self::Deadline
            | Self::HoldsWaited
            | Self::ResourceUnavailable
            | Self::ResourceAccess
            | Self::PoolLost { .. }
            | Self::Procedure
            | Self::Circle
            | Self::Waited => None,
        }
    }
}

/// The proposal deadline retained until the task-making decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Deadline {
    Named,
    Span(Duration),
}

/// Root-authorized member of an atomic creation batch; tasks preflights the whole graph, payloads,
/// capacity and finite reservations before mutation. (domain/tasks/README.md, sections 3–4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct New {
    /// Opaque application label, immutable after creation.
    pub label: u32,
    /// Immutable engine flags.
    pub flags: Flags,
    /// Fresh never-reused task number supplied by the root; live or within-batch duplicates refuse.
    pub number: u64,
    /// Policy project authenticated by root and checked against creator/project structural limits.
    pub project: u32,
    /// `Agent` charter executor, checked against admitted configuration.
    pub executor: Executor,
    /// An agent task's LLM choice, checked against the configured models, its charter and its
    /// authority; absent for a procedure's or a person's task, and where the root resolves the
    /// default (domain/tasks/README.md, sections 3 and 4).
    pub llm: Option<LlmChoice>,
    /// Owned specification within configured counts/bytes, with empty retained inputs.
    pub spec: Spec,
    /// Admitted bounded result contract.
    pub contract: Contract,
    /// Complete root-approved authority carrier; tasks checks bounded shape, not policy inclusion.
    pub authority: Authority,
    /// Deadline selection retained in pending proposals.
    pub deadline: Deadline,
    /// Fresh allotment: budget equals authority spend, with all spend/reservation counters zero.
    pub numbers: Numbers,
    /// Finite source; task sources must be live ancestors of the task creator and external
    /// sources must belong to this project.
    pub funder: Funder,
    /// Distinct immutable dependencies, at most `Limits::dependencies`: members of this batch or
    /// the creator's current live delegates.
    pub dependencies: Box<[u64]>,
    /// Connector resources taken together when the task first becomes active.
    pub holdings: Box<[Holding]>,
    /// Creator-selected policy for wakes and batching.
    pub wake: WakePolicy,
    /// Present only for the core recurring procedure; template members cannot recur.
    pub recurring: Option<Box<RecurringTemplate>>,
    /// Goal priority when people track this task; absent for ordinary work.
    pub tracked: Option<u32>,
}

/// A configured bound a run's model call met, and that bound's configured
/// value in its own unit: never a measured size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Exceeded {
    pub bound: Bound,
    pub value: u64,
}

/// A configured bound on a model call.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Bound {
    /// The model's reasoning, per item or in all: bytes.
    Reasoning,
    /// The model's other replay metadata: bytes.
    Metadata,
    /// The items of the model's answer: a count.
    Items,
    /// The model's answer: bytes.
    Answer,
    /// The tools offered to the model: a count.
    Tools,
    /// The messages and blocks one request carries: a count of both.
    HistoryItems,
    /// The run's retained history, which compaction could not bring under
    /// one of its configured ceilings: on messages, on blocks or on bytes.
    /// The value is that ceiling, and the bound doesn't name its unit.
    History,
    /// The model's window: tokens.
    Window,
    /// The model's declared output ceiling: tokens.
    Output,
}

/// Close a whole subtree with a common bounded reason (domain/tasks/changes.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Cancel {
    pub task: u64,
    pub reason: Box<[u8]>,
    pub grace: Option<Duration>,
    pub message: Option<u64>,
}

/// Whether an amendment tracks a task at a priority, or untracks it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tracking {
    /// A goal from this decision, at this priority.
    Tracked { priority: u32 },
    /// No goal from this decision.
    Untracked,
}

/// Whole optional task changes; root checks any widening against its holder.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Amendment {
    /// Change goal tracking independently of the specification.
    pub tracked: Option<Tracking>,
    /// Whole person address change, independent of spec and hold revisions.
    pub addressed: Option<AddressChange>,
    /// Replace the charter's instructions, delivered whole as this amendment's message.
    pub instructions: Option<Box<[u8]>>,
    /// Release this exact hold with the change; a later hold remains in place.
    pub release: Option<u64>,
    pub spec: Option<Spec>,
    pub wake: Option<WakePolicy>,
    /// Whole replacement dependency list, at any mutable phase (domain/tasks/lifecycle-endings.md, 1 and domain/tasks/changes.md).
    pub dependencies: Option<Box<[u64]>>,
    pub authority: Option<Authority>,
    /// A new LLM choice for an agent task's next run; `None` keeps its choice (domain/tasks/changes.md
    /// ).
    pub llm: Option<LlmChoice>,
    pub reason: Box<[u8]>,
}

/// A replacement address naming the person task revision it decides about.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AddressChange {
    pub revision: u64,
    pub address: PersonAddress,
}

/// One kind of durable task-tree change.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Change {
    Made,
    Concluded,
    Cancelled,
    Released,
    Amended,
    Moved,
    Proposed,
    ProposalPassed,
    ProposalAccepted,
    ProposalRejected,
    ProposalWithdrawn,
}

/// Amend a task with its root-issued message (domain/tasks/changes.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Amend {
    /// The giver's source for a widening or an accepted proposal (domain/authority-budgets.md, 1, and domain/authority.md, 9).
    pub funder: Option<Funder>,
    pub task: u64,
    pub message: u64,
    pub stop_run: bool,
    pub amendment: Box<Amendment>,
}

/// Root-reported activation failure category; each category has its own bounded retry policy and
/// counter. (domain/tasks/lifecycle-holds.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    /// Transient peer/read/transcript failure reported by the root.
    Transient,
    /// Failure requiring an external correction, such as an unavailable resource or invalid
    /// assignment.
    Permanent,
    /// `Run` itself reported execution failure.
    Run,
    /// `Agent` crashed, violated its channel contract or stalled.
    Agent,
    /// Worker was lost.
    Lost,
    /// Returned result violated the admitted task contract.
    Invalid,
    /// A workspace or required brief section could not be prepared for now.
    Infrastructure,
}

/// The flags a task is made with; amendments never change them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    /// Turns are entries for its requesting party; only agent tasks have turns.
    pub conversation: bool,
    /// Ended tasks are indexed by project and label.
    pub listed: bool,
    /// How long the ended task is kept.
    pub retention: Retention,
    /// Which authority may end this task, fixed at creation.
    pub ended_by: EndedBy,
}

impl Flags {
    /// Ordinary work: no conversation, not listed, kept.
    pub const PLAIN: Flags =
        Flags { ended_by: EndedBy::Result, conversation: false, listed: false, retention: Retention::Kept };
}

/// How long an ended task is retained (domain/people/README.md, section 5; protocol/store/growth.md, section 1.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    /// Kept to the deployment's retention horizon.
    Kept,
    /// Erased once read by its requesting party and named by no live task.
    ForgottenOnceRead,
}

/// The immutable authority that may end a task (domain/tasks/lifecycle-endings.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndedBy {
    /// Its run's judged result ends it.
    Result,
    /// Only its requester ends it; a run's raw finish parks it (5.8).
    Requester,
    /// A named procedure closes an agent task (jig/docs/design/domain/tasks/lifecycle-endings.md, 3).
    Closer(Closer),
    /// A standing agent task ends only by cancel (jig/docs/design/domain/tasks/standing.md, 2).
    Standing,
}

/// The procedure named to close a task (jig/docs/design/domain/tasks/lifecycle-endings.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Closer {
    pub connector: u16,
    pub code: u32,
}

/// One model and its effort (domain/tasks/README.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModelEffort {
    /// The catalogue number's position, starting at zero.
    pub model: u8,
    /// One of the efforts the model declares, in its order; `None` is the LLM provider's default,
    /// the only effort of a model that declares none.
    pub effort: Option<u8>,
}

impl ModelEffort {
    /// The index of a kept choice whose model, or the effort it named, left the catalogue:
    /// no configured model has it (jig/docs/design/protocol/store/records.md, 1).
    pub const GONE: u8 = u8::MAX;
}

/// What an agent task's runs' LLMs are, including its creator's optional account preference
/// (domain/tasks/README.md, section 3; domain/accounts-uses.md, section 1.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LlmChoice {
    /// Creator preference, distinct from the task's successfully claimed account.
    pub account: Option<u32>,
    /// The model its runs use, and their check holds its most costly completion (domain/authority-checks.md,
    /// section 1.3).
    pub main: ModelEffort,
    /// The models its runs' sub-agents may use, each named once, at most `MODELS`, among which
    /// smith chooses within a run.
    pub subagents: Box<[ModelEffort]>,
    pub later: Option<LlmLater>,
}

/// The choice every run whose reasons hold no `first` takes
/// (jig/docs/design/domain/tasks/README.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LlmLater {
    pub main: ModelEffort,
    /// Each named once, at most `MODELS`.
    pub subagents: Box<[ModelEffort]>,
}

/// A task action waiting for a holder's authority and funding.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "a stored proposed amendment stays inline and is priced by size_of")
)]
pub enum ProposalAction {
    /// Settings values stay in the core (domain/engine/settings-proposals.md, 1).
    Settings { against: u32, items: Box<[SettingHead]>, projects: Box<[u32]> },
    /// Whole batch with IDs allocated on proposal admission, before reservation.
    Batch(Box<[New]>),
    /// Connector-owned payload retained under this proposal's number. Only
    /// the needed authority and original call purpose cross the child boundary.
    Effect { connector: u16, authority: Authority, attempt: u64, completion: u32, position: u32, purpose: u64 },
    /// A procedure-origin effect keeps its genuine stable purpose and step.
    ProcedureEffect { connector: u16, authority: Authority, step: u64, purpose: u64 },
    /// Change an existing live task after its holder accepts the needed authority.
    Amend { task: u64, revision: u64, amendment: Amendment },
    /// Widen a task's current authority.
    Widen { task: u64, authority: Authority },
    /// Release a held task whose decision needs the holder.
    Release { task: u64 },
    /// Release from a placed hold, naming what the proposer saw.
    ReleasePlaced { task: u64, hold: u64, revision: u64 },
}

/// Current decision recipient, selected by root from the nearest covering holder.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProposalHolder {
    /// Ancestor task.
    Task(u64),
    /// Person requester above the root.
    Person(u64),
    /// People whose project role permits this kind of decision; there is no further holder.
    Policy { project: u32, kind: ProposalKind },
}

/// Decision family used for the final policy recipient.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProposalKind {
    Settings,
    Batch,
    Effect,
    Amend,
    Widen,
    Release,
}

/// One proposal's current or retained decision state.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalState {
    /// No holder has decided yet.
    Pending { holder: ProposalHolder, since: Wall },
    /// Accepted by a covered holder in the same commit as its action.
    Accepted { by: Party },
    /// Rejected with words delivered to the proposer.
    Rejected { by: Party, reason: Box<[u8]> },
    /// Proposer withdrew it or its live basis closed.
    Withdrawn,
}

/// Root-numbered proposal whose action and reason remain durable.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Proposal {
    pub number: u64,
    pub proposer: u64,
    pub project: u32,
    pub action: ProposalAction,
    pub reason: Box<[u8]>,
    pub as_holder: bool,
    pub state: ProposalState,
}

/// The tasks child retains only settings item heads (domain/engine/settings-proposals.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Setting {
    Model(u32),
    Worker(u32),
    Provider(u32),
    Charter(u32),
    Template(u32),
    Policy(u32),
    Rules,
    Limits,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingEdit {
    Set,
    Removed,
    Reset,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SettingHead {
    pub setting: Setting,
    pub edit: SettingEdit,
}

/// Settings decisions name the made revision, or the revision that made them stale.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingsOutcome {
    Accepted { revision: u32 },
    Rejected,
    Stale { newest: u32 },
}

/// Task-owned permission carrier supplied by the root after authority checks; tasks admits its
/// bounded shape but does not judge permission inclusion. (domain/tasks/README.md, sections 2–3).
/// (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Authority {
    /// Configured permitted tool-family bits; no tool is served by tasks.
    pub tools: Tools,
    /// Configured models the task's LLM choice may name (domain/authority.md, section 3).
    pub models: Models,
    /// Owned resource grants, bounded by `Limits::authority_grants` and the aggregate authority-
    /// byte limit.
    pub grants: Box<[Grant]>,
    /// `Executor`, lifetime task-count and depth ceiling supplied after the root's policy check.
    pub delegation: Delegation,
    /// Static spend/time permission, distinct from current accounting.
    pub budget: Budget,
    /// Permitted note-scope bits carried without judging them here.
    pub notes: Scopes,
    /// Connector resource patterns where this task may write notes.
    pub note_resources: Box<[ResourceScope]>,
}

/// Root-configured tool-family bit set carried with a task, independent of authority's vocabulary.
/// (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tools(/** Up to 64 configured family bits; tasks assigns no family meanings. */ pub u64);

/// Configured-model bit set carried with a task, in configuration's order: bit `i` is the `i`th
/// configured model (domain/tasks/README.md, section 3; domain/authority.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Models {
    /// Every model, including models configured later.
    All,
    /// A frozen configured-model mask, including all 64 bits or none.
    Named(u64),
}

impl Models {
    /// Reordering and removal affect effective authority, never a retained ask's identity.
    #[must_use]
    pub fn remap(self, moved: &[u64; 64]) -> Self {
        match self {
            Self::All => Self::All,
            Self::Named(bits) => {
                let mut result = 0;
                for (bit, today) in moved.iter().enumerate() {
                    if bits.checked_shr(u32::try_from(bit).expect("bounded bit")).unwrap_or(0) & 1 == 1 {
                        result |= today;
                    }
                }
                Self::Named(result)
            }
        }
    }

    pub const NONE: Self = Self::Named(0);
    pub const ALL: Self = Self::All;

    #[must_use]
    pub const fn bits(self) -> u64 {
        match self {
            Self::All => u64::MAX,
            Self::Named(bits) => bits,
        }
    }

    #[must_use]
    pub const fn is_all(self) -> bool {
        match self {
            Self::All => true,
            Self::Named(_) => false,
        }
    }

    /// Preserve the selector tag across a vocabulary boundary.
    #[must_use]
    pub const fn from_parts(bits: u64, all: bool) -> Self {
        if all { Self::All } else { Self::Named(bits) }
    }
}

/// Task-relative note-scope bits carried as semantic data; the authority child checks scope
/// permission through the root. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1
/// ).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Scopes(/** Goal 1, project 2 and deployment 4 scope bits. */ pub u8);

impl Scopes {
    /// Goal-relative note-scope bit carried with authority.
    pub const GOAL: Scopes = Scopes(1);

    /// Project-relative note-scope bit carried with authority.
    pub const PROJECT: Scopes = Scopes(2);

    /// Deployment-relative note-scope bit carried with authority.
    pub const DEPLOYMENT: Scopes = Scopes(4);
}

/// One connector resource pattern carried with a task's note permissions.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResourceScope {
    pub connector: u16,
    pub pattern: Pattern,
}

/// Owned connector permission carried by tasks; connector interpretation and authority decisions
/// remain outside this child. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1
/// ).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub connector: u16,
    /// Granted connector kind; implication checks belong to authority.
    pub kind: u16,
    /// Covered resources, bounded under the task authority shape limits.
    pub pattern: Pattern,
}

/// Literal base segments and terminal coverage rule carried without connector parsing or
/// normalization. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pattern {
    /// `Exact` literal base segments, bounded per grant by `Limits::authority_segments`; bytes
    /// count toward the aggregate authority-byte limit.
    pub segments: Box<[Box<[u8]>]>,
    /// Terminal rule whose bytes also count toward the aggregate authority-byte limit.
    pub last: Last,
}

/// Terminal resource coverage carrier; tasks bounds bytes while authority interprets coverage.
/// (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Last {
    /// Requires one exact additional segment alone, as interpreted by authority.
    Exact(/** Literal additional segment; its bytes count toward `Limits::authority_bytes`. */ Box<[u8]>),
    /// Requires an additional segment with this prefix and covers its descendants, as interpreted
    /// by authority.
    Open(/** Literal prefix of an additional segment; its bytes count toward `Limits::authority_bytes`. */ Box<[u8]>),
}

/// Permitted executor identities carried in authority; these are not additional implemented task
/// execution routes. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AuthorityExecutor {
    /// Permission to delegate to an application job kind (jig/docs/design/domain/authority.md, 3).
    Job(u32),
    /// Permission to delegate to an agent charter.
    Charter(u32),
    /// `Procedure` permission carrier; the contracted task API exposes agent execution only.
    Procedure(/** `Procedure` permission number; not a current task `Executor` variant. */ u32),
    /// Person-role permission carrier; the contracted task API exposes agent execution only.
    Role(/** Project-role permission number; not a current task `Executor` variant. */ u32),
}

/// `Authority`'s permitted executors, lifetime task capacity and depth carried with a task; current
/// topology limits are checked separately. (domain/tasks/README.md, sections 2–3).
/// (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Delegation {
    /// Permitted executor entries, bounded by `Limits::executor_kinds`.
    pub kinds: Box<[AuthorityExecutor]>,
    /// Lifetime descendant task allowance; it is not the current live delegate count.
    pub tasks: u32,
    /// Maximum descendant depth carried as permission, distinct from the child's structural depth
    /// bound.
    pub depth: u32,
}

/// Task's permission ceiling for deployment-unit spend and latest wall time; available `funding` is
/// tracked separately in `Numbers`. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1
/// ).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Budget {
    /// Maximum allotment in the root-configured deployment spending unit.
    pub spend: u64,
    /// Latest permitted ending wall time; `None` denotes no finite deadline.
    pub deadline: Option<Wall>,
}

/// Concrete accounting of one current allotment, owned by tasks: direct spend, settled funded spend
/// and still-open reservations. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1
/// ).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Numbers {
    /// Full amount of this allotment reserved against its funder.
    pub budget: u64,
    /// Direct expense posted here, within this allotment's budget.
    pub spent: u64,
    /// Expense of funded allotments already settled here, posted once when each task ends.
    pub spent_below: u64,
    /// Full budgets of directly funded allotments still open; availability is not recreated by
    /// opening another period.
    pub reserved: u64,
}

/// Financial source, distinct from requester topology; original period identities survive
/// later period openings. (domain/tasks/README.md, sections 2–3). (domain/authority.md, sections 3–6, and domain/authority-budgets.md, section 1
/// ).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Funder {
    /// Financial reservation against a live task in the creator's verified ancestry.
    Task(/** `Live` `funding` task number; the financial parent may differ from the requester. */ u64),
    /// Financial reservation against a retained person's project-period pool.
    Pool {
        project: u32,
        /** Person whose original project-period pool funds the task. */
        person: u64,
        /** Original project period of the pool, unchanged by later openings. */
        period: u64,
    },
    /// Per-period allotment of a live core recurring procedure, from its project's period.
    Recurring { project: u32, task: u64, period: u64 },
    /// Financial reservation directly against a retained project period.
    Period {
        project: u32,
        /** Monotonically opened period identity within that project. */
        period: u64,
    },
}
