//! References to connector-owned values (jig/docs/design/domain/connectors.md, 2).

use alloc::boxed::Box;
use skein_lib::{List, ReplyTo, Token, Wall};

mod lines;
pub use lines::{
    Bound, BudgetLine, Chip, ChipDetails, ChipObservation, ChipWork, ConversationRead, Count, Dependency, Ending,
    EscalationState, ExecutorKind, HoldPhaseLine, Nearest, PhaseCounts, PhaseLine, PlacedLine, PriorPhase,
    ResourceLine, ResultValue, TaskLine, Text, Wait, WaitsOn,
};

use crate::ops::party::Proposer;
use crate::requests::ApplicationScope;

/// Connector-owned value's kind, carried only as a reference.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PartKind {
    /// A named resource or its state.
    Resource,
    /// A counted resource pool.
    Pool,
    /// A task's connector result.
    Result,
    /// A procedure's panel or kind.
    Procedure,
    /// A proposed effect's description.
    Effect,
    /// An existing connector-owned outbox entry.
    Outbox,
    /// The connector's account of drift or a failed effect for the numbered held task.
    Hold,
}

/// The core's key for a connector-owned part; keys are never shortened.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartKey {
    /// Literal resource path segments.
    Path(List<Box<[u8]>>),
    /// A numbered kind, resource, task or proposal.
    Number(u64),
}

/// A reference to application data; no application value is copied.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Part {
    pub connector: u16,
    pub kind: PartKind,
    pub key: PartKey,
}

/// An application's record and the internal task numbers it names.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PublicApplication {
    pub tasks: Box<[u64]>,
    pub record: Box<[u8]>,
}

/// The bounded slice requested by a watch or read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Window {
    pub rows: u32,
    pub after: Option<Cursor>,
}

/// Continuation position in a subject's stable numbered list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cursor {
    /// A task or goal number in the subject's ordered list.
    Number { number: u64 },
    /// Exact identity of one inbox row; number families may overlap.
    Inbox { task: u64, object: InboxObject, at: Option<Wall> },
}

/// The object one inbox row opens; its own document carries content and acts.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InboxObject {
    Question { message: u64 },
    Proposal { proposer: Proposer, number: u64 },
    Escalation { revision: u64 },
    PersonTask,
    Reply { message: u64 },
    Result { position: u64 },
}

/// Why a watch ended.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum End {
    /// Its watcher left.
    Unwatched,
    /// Its subject left the engine.
    Gone,
    /// Its party may no longer see the subject.
    NotAllowed,
    /// Its party signed out.
    SignedOut,
    /// Its parent is stopping.
    Stopping,
}

/// The parent's result of making a document; its content stays with the parent.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Made {
    /// A whole bounded document is ready.
    Ready,
    /// No visible document remains.
    Gone { why: End },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartAsk {
    pub owner: u16,
    pub kind: u32,
    pub version: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PartFailure {
    Busy,
    TooLarge,
    Unreadable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RowPart {
    Made(Box<[u8]>),
    Unknown,
    Failed(PartFailure),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MadePart {
    Made(Box<[u8]>),
    Failed(PartFailure),
}

/// Row-major request; each row's kinds retain the query's order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PartsAsk {
    pub token: Token,
    pub party: u64,
    pub kinds: List<PartAsk>,
    pub rows: List<TaskLine>,
}

/// What the engine asks of a document and parts connector.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Judgement: make the party's document, answered by `Up::Made`.
    Document { watcher: Token, scope: ApplicationScope, subject: Token, party: u64, window: Window },
    /// Notification: drop the subject's binding; no answer.
    DropDocument { subject: Token },
    /// Notification: answer the party's read with the connector's own held output, not an `Up`.
    Read { reply: ReplyTo, scope: ApplicationScope, read: Token, party: u64, window: Window },
    /// Judgement: make application parts for task lines, answered by `Up::Lines`.
    Lines(PartsAsk),
    /// Judgement: make a public document, answered by `Up::Public`.
    Public { reply: ReplyTo, kind: u16, version: u16, project: u32, task: Option<u64>, public: Box<[u64]> },
}

/// A parts connector's answers and change marks.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// The document was made, or no visible document remains.
    Made { watcher: Token, outcome: Made },
    /// Row-major parts, each row in the ask's kind order.
    Lines { token: Token, parts: List<MadePart> },
    /// A public document, if available.
    Public { reply: ReplyTo, answer: Option<PublicApplication> },
    /// Unasked: the tasks whose parts changed.
    Marked { tasks: List<u64> },
}
