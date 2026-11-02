//! One-terminal reads, bounded pages and opaque cursors; client/domain/contract.md, 6.
//! A cursor is never decoded by the client. Live watches may overlap retained read pages.
use super::{
    Bytes, DecodedActivityLine, HistoryRow, InboxGroup, Me, Note, NoteScope, ProposalKey, Question, Result, Task, Text,
};
use alloc::boxed::Box;
use skein_lib::List;
/// An opaque page continuation within its byte bound.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub value: Bytes,
}
impl Cursor {
    /// Takes a complete opaque continuation within the client's cursor bound.
    pub fn new(bytes: Box<[u8]>, limit: u32) -> core::result::Result<Self, Box<[u8]>> {
        let value = Bytes::new(bytes, limit)?;
        Ok(Self { value })
    }
}

/// One inbox query order; the two groups carry independent opaque cursors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InboxReadGroup {
    /// Work waiting on the party, oldest first.
    Needs,
    /// Results and chat replies, newest first.
    Updates,
}
impl InboxReadGroup {
    /// A kind belongs to exactly one of the two query groups.
    #[must_use]
    pub const fn of(kind: InboxGroup) -> Self {
        match kind {
            InboxGroup::Questions | InboxGroup::Proposals | InboxGroup::Escalations | InboxGroup::PersonTasks => {
                Self::Needs
            }
            InboxGroup::Results | InboxGroup::Replies => Self::Updates,
        }
    }
}

/// A document or a bounded page requested without a key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Read {
    Settings,
    Setting(super::ReadSetting),
    Revision(super::ReadRevision),
    Revisions(super::ReadRevisions),
    /// A task query with independent group continuations.
    Tasks {
        query: super::TaskQuery,
        live: Option<Cursor>,
        ended: Option<Cursor>,
    },
    /// The frame.
    Me,
    /// One committed turn, with the same access as activity.
    Turn {
        task: u64,
        attempt: u64,
        turn: u32,
    },
    /// One chunk of stored specification words or effective instructions.
    TaskText {
        task: u64,
        field: TaskTextField,
        from: u64,
    },
    /// One task.
    Task {
        task: u64,
    },
    /// One proposal.
    Proposal {
        proposal: ProposalKey,
    },
    /// One escalation.
    Escalation {
        task: u64,
    },
    /// One question.
    Question {
        task: u64,
        message: u64,
    },
    /// One person task.
    PersonTask {
        task: u64,
    },
    /// One result.
    Result {
        task: u64,
    },
    /// Ended delegates.
    Ended {
        task: u64,
        after: Option<Cursor>,
    },
    /// Task or subtree history.
    History {
        task: u64,
        subtree: bool,
        after: Option<Cursor>,
    },
    /// Activity read from the end.
    Activity {
        task: u64,
        before: Option<Cursor>,
    },
    /// One filtered inbox group, with its own continuation. A kind implies its group.
    Inbox {
        group: InboxReadGroup,
        project: Option<u32>,
        kind: Option<InboxGroup>,
        goal: Option<u64>,
        after: Option<Cursor>,
    },
    /// Project spend by period, with an opaque continuation.
    Spend {
        project: u32,
        after: Option<Cursor>,
    },
    /// One project.
    Project {
        project: u32,
    },
    /// Scoped notes.
    Notes {
        scope: NoteScope,
        description: Option<Text>,
        after: Option<Cursor>,
    },
    /// Deployment state.
    Deployment,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivityPage {
    /// The source page is incomplete, independent of row loss and continuations.
    pub cut: bool,
    pub rows: List<DecodedActivityLine>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HistoryPage {
    /// The source page is incomplete, independent of row loss and continuations.
    pub cut: bool,
    pub rows: List<HistoryRow>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NotePage {
    /// The source page is incomplete, independent of row loss and continuations.
    pub cut: bool,
    pub rows: List<Note>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A read document or page.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReadValue {
    /// Source text with explicit byte continuation.
    TaskText(Box<TaskText>),
    Settings(Box<super::SettingsDocument>),
    Setting(Box<super::SettingDocument>),
    Revision(Box<super::RevisionAnswer>),
    Revisions(Box<super::RevisionsPage>),
    /// A task query's two groups.
    Tasks(Box<super::TaskPage>),
    /// Whole authentic project owner document.
    Project(Box<super::Project>),
    /// Whole ended facts for one existing object query.
    Ended(Box<super::Ended>),
    /// The frame.
    Me(Box<Me>),
    /// A complete committed turn archive.
    Turn(Box<super::ArchivedTurn>),
    /// A task.
    Task(Box<Task>),
    /// Whole supplied pending owner proposal.
    Proposal(Box<super::Proposal>),
    /// A task owner’s whole escalation document.
    Escalation(Box<super::Escalation>),
    /// A question.
    Question(Box<Question>),
    /// Pending person work with owner contract and optional shown results.
    PersonTask(Box<super::PersonTask>),
    /// A result.
    Result(Box<Result>),
    /// Retained delegate endings without synthetic result details.
    EndedPage(Box<super::EndedPage>),
    /// History lines.
    History(Box<HistoryPage>),
    /// Committed activity.
    Activity(Box<ActivityPage>),
    /// Exact owner Inbox rows selected under the original query.
    Inbox(Box<super::InboxPage>),
    /// Retained whole funding records without inferred ledger facts.
    Spend(Box<super::SpendPage>),
    /// Notes.
    Notes(Box<NotePage>),
    /// Whole deployment-owned facts and diagnostics.
    Deployment(Box<super::Deployment>),
}

/// Why one read has no value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReadNone {
    /// Original turn payload bytes, past the engine or client read bound.
    TooLarge { bytes: u64 },
    /// The first row needs more joins than the whole read bound.
    TooLinked { joins: u32 },
    /// The visible turn could not be read or decoded whole.
    Malformed,
    /// No object the party may see.
    Unknown,
    /// Not admitted.
    Refused,
    /// No longer retained.
    Gone,
    /// The attempt lost its connection.
    Unreachable,
    /// The engine loads are full.
    Busy,
    /// The sign in ended.
    SignedOut,
}

/// The one terminal of a read.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReadResult {
    /// The committed value.
    Value(ReadValue),
    /// Why no value came.
    None(ReadNone),
}

/// The independent task text sources available through bounded reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskTextField {
    /// The task's specification words.
    Specification,
    /// Override or charter instructions at the task's recorded settings revision.
    Instructions,
}

/// A UTF-8 chunk, byte offsets and the complete source length.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskText {
    pub task: u64,
    pub field: TaskTextField,
    pub from: u64,
    pub total: u64,
    pub next: Option<u64>,
    pub text: Text,
}
