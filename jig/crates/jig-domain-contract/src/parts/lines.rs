//! Bounded task facts supplied when a connector makes parts of a row.

use super::{Part, RowPart};
use crate::ops::party::{EntryKind, HoldOwner, Whom};
use crate::ops::{
    Active, Added, Circle, Endings, EscalationDecider, EscalationHolder, Executor, Flags, Funder, Hold, Mode, Numbers,
    Party, PersonAddress, PersonWait, ResultField, RetryNotice, Scope, Stage, Status, Tries, WakePolicy,
};
use crate::ops::{AtBound, Unbounded, WaitKind, WaitObject};
use crate::resources::Writer;
use alloc::boxed::Box;
use skein_lib::{List, Token, Wall};

/// One bounded UTF-8 text value, with loss stated separately.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Text {
    pub bytes: Box<[u8]>,
    pub cut: bool,
}

/// A bounded count, with saturation stated.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Count {
    /// Every currently known entry was counted.
    Known(u32),
    /// The bounded cache may omit older entries.
    AtLeast(u32),
}

/// A task's executor kind; its detail lives on the task document.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ExecutorKind {
    /// An application job.
    Job,
    /// An agent with a configured charter.
    Agent,
    /// A connector-owned procedure.
    Procedure,
    /// A party or project role.
    Person,
}

/// A compact typed phase and its immediate waiting facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PhaseLine {
    /// Waiting for live dependencies or resource holdings.
    Waiting { dependencies: List<u64>, more: bool, holds: bool },
    /// Current activation state of an agent or procedure.
    Active(Active),
    /// Waiting for the addressed party or role.
    Person { address: PersonAddress, taken_by: Option<u64>, since: Option<Wall> },
    /// Closing through its ordered obligations.
    Closing(Stage),
    /// Held for this reason at this independent revision.
    Held { reason: Hold, revision: u64 },
    /// Ended outside the live arena.
    Ended,
}

/// A task named in passing, with its bounded first-line title.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Chip {
    pub number: u64,
    pub title: Text,
    pub executor: ExecutorKind,
    pub phase: PhaseLine,
    /// Nonrecursive owner facts and optional fenced live observations.
    pub details: Box<ChipDetails>,
}

/// Latest whole activity name, without transport or provisional content.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ChipWork {
    Preparing,
    Starting,
    Active,
    Writing,
    Tool { call: Box<[u8]>, tool: Box<[u8]> },
    Waiting,
    Ending,
}

/// Fenced host observation; all walls are engine receipt times.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChipObservation {
    pub channel: Token,
    pub work: ChipWork,
    pub since: Wall,
    pub reported_at: Wall,
    pub started: Option<Wall>,
}

/// Facts needed beside every chip, without nesting other chips or histories.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChipDetails {
    pub dependencies: List<Dependency>,
    /// Owner context for a role address, including in a standalone retained chip.
    pub project: u32,
    /// Concrete owner identity; the chip's coarse executor kind remains separate.
    pub executor: Executor,
    pub phase_since: Wall,
    /// Placement/adoption, never inferred to be actual agent start.
    pub run_since: Option<Wall>,
    pub attempt: Option<u64>,
    pub retry: Option<RetryNotice>,
    pub wake: WakePolicy,
    pub person_wait: Option<PersonWait>,
    pub hold: Option<HoldPhaseLine>,
    pub budget: BudgetLine,
    /// Exact immutable activity tail for retained failure/result explanations.
    pub activity: u64,
    pub resources: List<ResourceLine>,
    pub resources_cut: bool,
    /// False for a row-only read: missing holder/writer is then unknown.
    pub resources_observed: bool,
    pub procedure: Option<Part>,
    pub live_delegates: u32,
    /// Connectors the task waits on (domain/engine/views.md, 3), unknown in an unjoined stored closing row.
    pub closing_connectors: Option<List<u16>>,
    pub ending: Option<Ending>,
    pub ended_at: Option<Wall>,
    pub observation: Option<ChipObservation>,
}

/// Contract-shaped result words and an optional connector-owned value.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResultValue {
    /// Whole approved state and optional bounded explanation.
    Approved { state: Box<[u8]>, words: Option<Text> },
    /// Successful report.
    Report(Text),
    /// A selected verdict code with its explanation.
    Verdict { code: u32, fields: Box<[ResultField]>, words: Text },
    /// A connector value with summary words.
    Change { kind: u16, part: Part, words: Text },
    /// A task that could not complete its contract.
    Failure(Text),
}

/// Ending carried while a live task closes, or in an ended result document.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    /// Completed its contract.
    Done(ResultValue),
    /// Failed with these words.
    Failed(Text),
    /// Cancelled, preserving any previously completed result.
    Cancelled { reason: Text, result: Option<ResultValue>, by: Party, at: Wall },
}

/// Dependency facts from its owning task and the prerequisite's live row or stub
/// (client/domain/contract/documents.md, 1; domain/tasks/lifecycle-endings.md, 1 and domain/tasks/changes.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Dependency {
    pub number: u64,
    pub pending: bool,
    pub added: Option<Added>,
    pub keeps: bool,
    pub status: Option<Status>,
}

/// Current held-decision recipient and any durable rejection words.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EscalationState {
    /// The hold is being routed to its eligible holder in this decision.
    Routing,
    /// An identified holder may decide it.
    Waiting { holder: EscalationHolder, entry: u64, since: Wall },
    /// Decided to leave it held.
    Rejected { by: EscalationDecider, reason: Text },
}

/// A hold's reason, revision and prior lifecycle, with its escalation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HoldPhaseLine {
    pub placed: Option<PlacedLine>,
    /// Connector-supplied account retained with the hold, never reconstructed by a client.
    pub words: Option<Text>,
    pub circle: Option<Circle>,
    /// The connector's account, independently of the resource or outbox origin.
    pub account: Option<Part>,
    pub source: Option<Part>,
    pub reason: Hold,
    pub revision: u64,
    pub before: PriorPhase,
    pub escalation: Option<EscalationState>,
    pub tries: Tries,
    pub hold_wait_since: Option<Wall>,
}

/// A placed hold and the reader's ownership admission (client/domain/contract/documents.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PlacedLine {
    pub number: u64,
    pub owner: HoldOwner,
    pub mode: Mode,
    pub scope: Scope,
    pub reason: Box<[u8]>,
    pub placed_at: Wall,
    pub revision: u64,
    pub lifted: bool,
    pub may_lift: bool,
}

/// The lifecycle preserved by a hold.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PriorPhase {
    /// Waiting for dependencies or resource holdings.
    Waiting,
    /// An activation's phase.
    Active(Active),
    /// Closing through this obligation.
    Closing(Stage),
}

/// A task's concrete allotment and its financial source.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BudgetLine {
    pub numbers: Numbers,
    pub left: u64,
    pub funder: Funder,
    pub deadline: Option<Wall>,
}

/// A resource the task holds or awaits, and its authentic writer if present.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResourceLine {
    pub part: Part,
    pub kind: u16,
    pub taken: bool,
    pub holder: Option<u64>,
    pub writer: Option<Writer>,
    pub writer_lost: bool,
}

/// Phase totals of every task counted, independent of visible rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PhaseCounts {
    pub waiting: u32,
    pub idle: u32,
    pub starting: u32,
    pub running: u32,
    pub backing_off: u32,
    pub closing: u32,
    pub held: u32,
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

impl PhaseCounts {
    /// Begin a bounded phase census.
    pub const ZERO: Self = Self {
        waiting: 0,
        idle: 0,
        starting: 0,
        running: 0,
        backing_off: 0,
        closing: 0,
        held: 0,
        done: 0,
        failed: 0,
        cancelled: 0,
    };
}

/// The requesting party's position over a conversation's committed turns.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConversationRead {
    pub seen: u64,
    pub unread: Count,
}

/// One record for every task list (client/domain/contract/documents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskLine {
    pub chip: Chip,
    pub project: u32,
    pub requester: Party,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: Flags,
    pub changed: Wall,
    pub last_activity: Wall,
    pub ended_at: Option<Wall>,
    pub budget: BudgetLine,
    pub waits_on: WaitsOn,
    pub wait: Option<Wait>,
    pub nearest: Option<Nearest>,
    pub phases: PhaseCounts,
    /// Direct delegates ended done (jig/docs/design/client/domain/contract/documents.md, 1).
    pub delegates_done: u32,
    /// Direct delegates live or ended, cancelled ones left out.
    pub delegates_total: u32,
    pub held_below: u32,
    pub conversation: Option<ConversationRead>,
    pub parts: List<RowPart>,
}

/// What a task waits on, computed for each document and never stored.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum WaitsOn {
    /// Its oldest unanswered question to its requester (domain/engine/views.md, 3).
    Question { number: u64 },
    /// Pending prerequisites, oldest first, bounded by document lines (domain/engine/views.md, 3).
    Dependencies { pending: List<u64>, count: u32 },
    /// The oldest waiting entry and the number of waiting entries in the subtree.
    Party { whom: Whom, entries: u32, task: u64, kind: EntryKind },
    /// Live delegates, with no party waiting in the subtree.
    Delegates,
    /// A message, a run's resumption or a procedure's facts.
    News,
    /// A working run, a phase that explains its own wait, or an ended task.
    Nothing,
}

impl PhaseCounts {
    /// Ended goals use their stored subtree census (protocol/store/loads.md, 3).
    #[must_use]
    pub const fn from_endings(endings: Endings) -> Self {
        Self { done: endings.done, failed: endings.failed, cancelled: endings.cancelled, ..Self::ZERO }
    }
}

/// One episode of waiting (jig/docs/design/domain/engine/waits.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Wait {
    pub task: u64,
    pub kind: WaitKind,
    pub object: WaitObject,
    pub since: Wall,
    pub bound: Bound,
    pub hears: Option<EscalationHolder>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Bound {
    At { at: Wall, then: AtBound },
    None(Unbounded),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Nearest {
    pub at: Wall,
    pub then: AtBound,
    pub kind: WaitKind,
}
