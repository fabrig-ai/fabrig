//! References to connector-owned values (jig/docs/design/domain/connectors.md, 2).

use alloc::boxed::Box;
use skein_lib::{List, ReplyTo, Time, Token, Wall, Writer};

mod lines;
pub use lines::{
    Bound, BudgetLine, Chip, ChipDetails, ChipObservation, ChipWork, ConversationRead, Count, Dependency, Ending,
    EscalationState, ExecutorKind, HoldPhaseLine, Nearest, PhaseCounts, PhaseLine, PlacedLine, PriorPhase,
    ResourceLine, ResultValue, TaskLine, Text, Wait, WaitsOn,
};

use crate::ops::party::Proposer;
use crate::requests::ApplicationScope;
use crate::{effects, ops};

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

/// The context of a run's brief, shared with each connector that plans sections
/// (jig/docs/design/domain/engine/briefs.md, 1.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Preparation {
    pub task: u64,
    pub project: u32,
    /// A task requester is the task's parent.
    pub requester: ops::Party,
    pub root: u64,
    /// The charter's documents with slots, in charter order.
    pub documents: Box<[Document]>,
    /// The core's sections, in brief order.
    pub core: Box<[Claim]>,
    /// Slots left beside the core's and documents' sections.
    pub slots: u32,
    pub budget: u32,
}

/// One core section's claim on the brief budget.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Claim {
    pub size: u32,
    pub limit: u32,
    pub priority: u16,
    pub required: bool,
}

/// A charter's document (domain/engine/settings.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub connector: u16,
    /// The resource name, within the authority's pattern bounds.
    pub name: effects::Name,
    /// A bounded nonempty path.
    pub path: Box<[u8]>,
    /// A bounded nonempty heading when present.
    pub heading: Option<Box<[u8]>>,
}

/// A section a connector offers to hold for the brief.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Section {
    /// Unique among the sections this connector holds.
    pub token: Token,
    /// The connector's section kind, carried through to the assignment.
    pub kind: u16,
    /// The section's fixed name in a failure.
    pub name: &'static [u8],
    /// Expected size of the gathered section, or zero if unknown.
    pub size: u32,
    pub limit: u32,
    /// Lower priorities keep room first.
    pub priority: u16,
    pub required: bool,
    /// Index into `Preparation::documents` for a document section.
    pub document: Option<u16>,
}

/// How a brief's failure counts (domain/engine/briefs.md, 1.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fate {
    /// Nothing is wrong but room: the task pauses, no try spent.
    Capacity,
    /// Counted in the tasks child's transient class.
    Transient,
    /// Counted in the tasks child's infrastructure class.
    Infrastructure,
    /// Counted in the tasks child's permanent class.
    Permanent,
}

/// Why a section or a brief could not be prepared, in fixed words (domain/engine/briefs.md, 1.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Failure {
    pub fate: Fate,
    /// The section's name, or `b"brief"`.
    pub section: &'static [u8],
    pub words: Box<[u8]>,
}

impl Failure {
    /// `SECTION: WORDS`; whoever keeps it holds it to its own bound.
    #[must_use]
    pub fn reason(&self) -> Box<[u8]> {
        let size = self.section.len().saturating_add(b": ".len()).saturating_add(self.words.len());
        let mut writer = Writer::new(size);
        writer.put(self.section).expect("measured section name");
        writer.put(b": ").expect("measured separator");
        writer.put(&self.words).expect("measured failure words");
        writer.finish()
    }
}

/// A charter document section's priority (domain/engine/settings.md, 9).
pub const DOCUMENT_PRIORITY: u16 = 5;

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
    /// Plan this connector's sections, answered by `Up::Planned`.
    Plan(Box<Preparation>),
    /// Forget a plan still being made; no answer.
    Abandon { task: u64 },
    /// Gather a section by the deadline, answered by `Up::Sized` or `Up::Failed`.
    Gather { task: u64, section: Token, budget: u32, deadline: Time },
    /// Cut a held section, answered by `Up::Sized` or `Up::Failed`.
    Cut { task: u64, section: Token, size: u32 },
    /// Release a section, planned or held; no answer.
    Drop { task: u64, section: Token },
}

/// A parts connector's answers and change marks.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// The document was made, or no visible document remains.
    Made {
        watcher: Token,
        outcome: Made,
    },
    /// Row-major parts, each row in the ask's kind order.
    Lines {
        token: Token,
        parts: List<MadePart>,
    },
    /// A public document, if available.
    Public {
        reply: ReplyTo,
        answer: Option<PublicApplication>,
    },
    /// Unasked: the tasks whose parts changed.
    Marked {
        tasks: List<u64>,
    },
    Planned {
        task: u64,
        sections: Box<[Section]>,
    },
    /// A section has been gathered or cut and is held at this size.
    Sized {
        task: u64,
        section: Token,
        size: u32,
    },
    Failed {
        task: u64,
        section: Token,
        failure: Failure,
    },
}
