//! Read pages and the owner-keyed store transport (`client/domain/contract/README.md`, section 4).
//! Cursors name a query and an owner key. The client only echoes them.
use crate::{
    CoreRecord, Key, Record,
    documents::{self, Document, Proposer},
};
use alloc::boxed::Box;
use core::mem::{size_of, size_of_val};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall, length::len_u64};

/// Which inbox group the page selects.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Group {
    /// Objects awaiting the party, oldest first.
    Needs,
    /// Unread outcomes and replies, newest first.
    Updates,
    /// Pending proposals, oldest first.
    Proposals,
    /// Pending escalations, oldest first.
    Escalations,
    /// Unanswered questions, oldest first.
    Questions,
    /// Active person tasks, oldest first.
    PersonTasks,
    /// Unread results, newest first.
    Results,
    /// Unread chat replies, newest first.
    Replies,
}
impl Group {
    #[must_use]
    pub fn updates(self) -> bool {
        match self {
            Self::Updates | Self::Results | Self::Replies => true,
            Self::Needs | Self::Proposals | Self::Escalations | Self::Questions | Self::PersonTasks => false,
        }
    }

    #[must_use]
    pub fn includes(self, object: documents::InboxObject) -> bool {
        match object {
            documents::InboxObject::Proposal { .. } => {
                self == Self::Needs || self == Self::Proposals || self == Self::Updates
            }
            documents::InboxObject::Escalation { .. } => self == Self::Needs || self == Self::Escalations,
            documents::InboxObject::Question { .. } => self == Self::Needs || self == Self::Questions,
            documents::InboxObject::PersonTask => self == Self::Needs || self == Self::PersonTasks,
            documents::InboxObject::Result { .. } => self == Self::Updates || self == Self::Results,
            documents::InboxObject::Reply { .. } => self == Self::Updates || self == Self::Replies,
        }
    }
}
/// Optional filters on the party's cross-project inbox.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Filter {
    pub project: Option<u32>,
    pub goal: Option<u64>,
}
/// Identity of a store query, independent of its continuation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Scope {
    /// Ended task records in key order for whole-task erasure (jig/docs/design/protocol/store/README.md, 6.1;
    /// jig/docs/design/domain/people/acts.md, 2).
    EndedTasks,
    /// History, milestones, activity, terminals, decisions, ended reads and the
    /// party proposal and update rows whose task link names this owner
    /// (jig/docs/design/protocol/store/README.md, 6.1; jig/docs/design/domain/people/acts.md, 2).
    TaskRows { task: u64 },
    /// Dense inclusive revision interval; held newest and tag-9 predecessors.
    SettingsHistory { first: u32, last: u32, newest: u32, whole: bool },
    /// A task query's live group and its indexed ended group.
    Tasks { query: documents::TaskQuery },
    /// Source text from a live or retained task, in bounded byte chunks.
    TaskText { task: u64, field: super::TaskTextField, from: u64 },
    /// One retained task.
    Task { task: u64 },
    /// One immutable proposal and its final routing.
    Proposal { proposer: Proposer, proposal: u64 },
    /// Latest kept decision for one escalation.
    Escalation { task: u64 },
    /// Original question and any retained answer.
    Question { task: u64, message: u64 },
    /// One retained person task.
    PersonTask { task: u64 },
    /// One retained result.
    Result { task: u64 },
    /// Exact outward activity named by an inbox reply.
    Reply { task: u64, activity: u64 },
    /// An immutable opaque turn archive.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// Ended direct delegates in task-key order.
    EndedDelegates { task: u64 },
    /// Semantic changes and lifecycle milestones in owner-key order.
    History { task: u64, subtree: bool },
    /// Immutable task activity, newest position first.
    Activity { task: u64 },
    /// One filtered inbox group in its semantic order.
    Inbox { group: Group, filter: Filter },
    /// Goal-scoped notes selected by their owning project, without bodies.
    Notes { project: u32 },
    /// Exact notes owner scope and optional byte substring of its descriptions.
    ScopedNotes { scope: notes::Scope, description: Option<Box<[u8]>> },
    /// Ended period ledgers; project zero selects the deployment's projects.
    Spend { project: u32 },
}
/// Owner key in this query's store order. Inbox indices preserve exact identity
/// and wall ordering across projects without a second live cache.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Position {
    /// An owner's physical record key.
    Owner(Key),
    /// A virtual owner index for an exact inbox object.
    Inbox { at: Option<Wall>, task: u64, object: documents::InboxObject, key: Key },
}
/// Opaque continuation of this exact query.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cursor {
    /// Remaining live rows, before entering the store range.
    Live { scope: Scope, after: documents::Cursor, store_after: Option<Position> },
    /// Exclusive owner key; `None` starts the stored portion after live rows.
    Store { scope: Scope, after: Option<Position> },
}
/// Which side of an activity line to read (client/domain/contract/README.md, section 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Earlier,
    Later,
}
/// One document or bounded page, without a request key.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "the read carries two independent inline owner cursors, priced in the read bounds"
)]
pub enum Read {
    Settings,
    Setting {
        setting: crate::settings::Setting,
        revision: u32,
        proposal: Option<(Proposer, u64)>,
    },
    Revision {
        number: u32,
    },
    Revisions {
        before: Option<u32>,
    },
    /// A task query's independent live and ended groups.
    Tasks {
        query: documents::TaskQuery,
        live: Option<Cursor>,
        ended: Option<Cursor>,
    },
    /// A connector-owned document within the client's stated role scope.
    Application {
        scope: jig_domain_people::ApplicationScope,
        connector: u16,
        read: skein_lib::Token,
        window: documents::Window,
    },
    /// The authenticated party's frame.
    Frame,
    /// Source words with an explicit continuation byte offset.
    TaskText {
        task: u64,
        field: super::TaskTextField,
        from: u64,
    },
    /// A task overview, live or ended.
    Task {
        task: u64,
    },
    /// An immutable proposal.
    Proposal {
        proposer: Proposer,
        proposal: u64,
    },
    /// A held task and its kept decision.
    Escalation {
        task: u64,
    },
    /// Whole original question and any kept answer.
    Question {
        task: u64,
        message: u64,
    },
    /// A person form, live or ended.
    PersonTask {
        task: u64,
    },
    /// One result.
    Result {
        task: u64,
    },
    /// Whole outward words at the exact child activity key.
    Reply {
        task: u64,
        activity: u64,
    },
    /// One committed opaque turn.
    Turn {
        task: u64,
        attempt: u64,
        turn: u32,
    },
    /// Ended direct delegates.
    EndedDelegates {
        task: u64,
        from: Option<Cursor>,
    },
    /// Task or subtree history.
    History {
        task: u64,
        subtree: bool,
        from: Option<Cursor>,
    },
    /// Activity from its end, in decreasing committed position.
    Activity {
        task: u64,
        before: Option<Cursor>,
    },
    /// The page next to a line of a task's activity, that line left out (client/domain/contract/README.md, section 4).
    ActivityBeside {
        task: u64,
        position: u64,
        side: Side,
    },
    /// One filtered inbox group.
    Inbox {
        group: Group,
        filter: Filter,
        from: Option<Cursor>,
    },
    /// A project's current resources, policy and funding.
    Project {
        project: u32,
    },
    /// Goal-scoped project notes from their owning child's rows, without bodies.
    Notes {
        project: u32,
        from: Option<Cursor>,
    },
    /// Notes in one exact owner scope, optionally searched by description.
    ScopedNotes {
        scope: notes::Scope,
        description: Option<Box<[u8]>>,
        from: Option<Cursor>,
    },
    /// Ended period spending; project zero selects the deployment's projects.
    Spend {
        project: u32,
        from: Option<Cursor>,
    },
    /// Deployment state.
    Deployment,
}
/// One store request. The transport selects the owner's range, never a cache.
/// `after` is exclusive in the stated order. Ended rows carry their owner
/// and bounded durable requester ancestry for visibility and subtree checks.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Range {
    pub party: u64,
    pub scope: Scope,
    pub after: Option<Position>,
    pub backwards: bool,
    pub rows: u32,
    pub bytes: u64,
}
/// A raw owner row with the facts needed to authenticate it without restoration.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Row {
    pub value: Record,
    pub owner: Option<Box<tasks::TaskRecord>>,
    /// Requester ancestors nearest first, copied from durable task records.
    pub ancestors: Box<[u64]>,
    /// Related immutable evidence, such as a decision or answered question.
    pub related: Option<Record>,
    /// A page turn whose bytes could not be carried (protocol/store/loads.md, 3).
    pub unjoined: Option<UnjoinedTurn>,
    /// The ended read position joined for this ended conversation's requester party.
    pub ended_read: Option<u64>,
    /// Existing ancestor, dependency and ended-subtree rows requested by this query.
    pub linked: Box<[Box<tasks::TaskRecord>]>,
    /// The selected reference when an owner row has several waiting objects.
    pub inbox: Option<documents::Cursor>,
}
impl Row {
    /// Complete ownership of this transport row and its requested evidence.
    #[must_use]
    pub fn owned_bytes(&self) -> Option<u64> {
        row_bytes(self)
    }
}
/// The one terminal event of a store load.
#[derive(Hash, Debug)]
pub enum Loaded {
    /// Rows within the range's row and byte caps; continuation is exclusive.
    Page { rows: Box<[Row]>, next: Option<Position> },
    /// Evidence for a single turn, authenticated against its durable owner.
    TurnProblem { task: u64, attempt: u64, turn: u32, owner: Box<tasks::TaskRecord>, why: TurnFailure },
    /// Transport admission was full and retained nothing.
    Busy,
    /// This range is no longer available.
    Gone,
    /// The first row needs more joins than the whole read bound (protocol/store/loads.md 3).
    TooLinked { joins: u32 },
}
/// A selected frame could not be supplied; visibility is checked before its reason leaves.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TurnFailure {
    /// Size charged past the read budget: opaque bytes without frame machines,
    /// or a complete frame when frame machines are configured (protocol/store/transcripts.md 6).
    TooLarge { bytes: u64 },
    /// The frame could not be read whole.
    Malformed,
}
impl TurnFailure {
    /// Complete ownership of a failure carrier and its durable task owner.
    #[must_use]
    pub fn owned_bytes(owner: &tasks::TaskRecord) -> Option<u64> {
        u64::try_from(size_of::<Loaded>()).ok()?.checked_add(tasks::task_bytes(owner)?)
    }
}
/// An ended object's document and its durable ending facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ended {
    pub value: EndedValue,
    pub at: Option<Wall>,
    pub by: Option<tasks::Party>,
}
/// Typed retained values, made by the same owners as live documents.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EndedValue {
    /// Full ended task overview and retained outcome.
    Task { document: Box<documents::Task>, result: Box<documents::ResultDoc> },
    /// Immutable task proposal and final routing state.
    Proposal(documents::ProposalHistory),
    /// A retained person-origin proposal and its first decision evidence.
    PersonProposal { proposal: Box<tasks::PersonProposal>, decision: Option<crate::ProposalDecisionRecord> },
    /// Kept escalation decision, its task and prior lifecycle story.
    Escalation {
        task: Box<documents::Task>,
        decision: crate::EscalationDecisionRecord,
        hold: Option<documents::MilestoneLine>,
    },
    /// Original question and any retained answer or task ending.
    Question {
        question: documents::Question,
        answer: Option<tasks::ActivityLine>,
        ending: Option<Box<documents::ResultDoc>>,
    },
    /// Retained person form and its outcome.
    PersonTask { document: Box<documents::PersonTask>, result: Box<documents::ResultDoc> },
    /// Whole outward words retained by their producing task.
    Reply(tasks::Word),
    /// Immutable opaque transcript bytes, not decoded by the core.
    Turn(crate::TurnRecord),
    /// A turn carried to a party other than its task's requester, projected before it leaves
    /// (domain/engine/briefs.md, 2.1).
    WithheldTurn(crate::TurnRecord),
}
/// A transcript frame not carried beside its activity line (protocol/store/loads.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UnjoinedTurn {
    TooLarge { task: u64, attempt: u64, turn: u32, spent: u64, read: Option<u64>, at: Wall, bytes: u64 },
    Unread { task: u64, attempt: u64, turn: u32 },
}
/// The turn supplied within an activity page's bounds (protocol/store/loads.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PageTurn {
    Carried(crate::TurnRecord),
    /// A turn carried to a party other than its task's requester, projected before it leaves
    /// (domain/engine/briefs.md, 2.1).
    Withheld(crate::TurnRecord),
    TooLarge {
        spent: u64,
        read: Option<u64>,
        at: Wall,
        bytes: u64,
    },
    Unread,
}
/// One semantic row of a read page.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PageRow {
    /// The common task line.
    Line(documents::TaskLine),
    /// An ended delegate's status and outcome.
    EndedDelegate(documents::EndedDelegate),
    /// A semantic task change.
    History(documents::HistoryLine),
    /// A lifecycle transition.
    Milestone(documents::MilestoneLine),
    /// Whole original words, run boundaries and exact archive references.
    Activity { line: tasks::ActivityLine, turn: Option<PageTurn> },
    /// One waiting object or unread update.
    Inbox(documents::InboxEntry),
    /// One loaded note.
    Note(documents::Note),
    /// Ended period accounting, never inferred from live roots.
    Spend(tasks::FundingRecord),
}
/// One bounded list page; no continuation means the range ended.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Page {
    pub rows: List<PageRow>,
    /// Conversation prefix continuation; other scopes supply none.
    pub before: Option<Cursor>,
    pub next: Option<Cursor>,
    pub cut: bool,
    /// Complete original membership contexts for Inbox rows only, when supplied.
    pub inbox_selection: Option<List<documents::InboxSelection>>,
}
/// One task query's two independently continued groups (domain/engine/views.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskPage {
    pub live: List<documents::TaskLine>,
    pub live_next: Option<Cursor>,
    pub ended: EndedGroup,
}
/// An ended index's rows, or no index for these query keys.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "the ended group carries its inline owner cursor, priced in the answer bound"
)]
pub enum EndedGroup {
    Rows { rows: List<documents::TaskLine>, next: Option<Cursor>, cut: bool },
    None,
}
/// Exactly one terminal, held until its preceding decisions are durable.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    /// One source text chunk.
    TaskText(Box<super::TaskText>),
    Setting(Box<SettingDocument>),
    Revision(Box<RevisionAnswer>),
    Revisions(Box<RevisionsPage>),
    /// A task query's two groups.
    Tasks(Box<TaskPage>),
    /// A current document.
    Document(Box<Document>),
    /// An ended or decided object, with kept actor and wall time.
    Ended(Box<Ended>),
    /// A bounded list page.
    Page(Box<Page>),
    /// No value is available.
    None(Why),
}
/// Refusal facts which reveal no invisible object identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    /// Missing, or invisible to the party.
    Unknown,
    /// A construction or load bound prevents admission.
    Busy,
    /// Required retained rows are no longer retained.
    Gone,
    /// The visible turn could not fit the read budget; original payload bytes.
    TooLarge { bytes: u64 },
    /// The first row needs more joins than the whole read bound.
    TooLinked { joins: u32 },
    /// The visible turn frame could not be read whole.
    Malformed,
    /// The read's sign-in is unknown to a ready engine, ended or expired.
    SignedOut,
}
#[must_use]
pub fn record_bytes(value: &Record) -> Option<u64> {
    let carrier = u64::try_from(size_of::<Record>()).ok()?;
    let owned = match value {
        Record::Tasks(value) => tasks::stored_bytes(value)?,
        Record::Core(CoreRecord::Turn(value)) => len_u64(value.transcript.len()),
        Record::Core(CoreRecord::Terminal(value)) => tasks::terminal_bytes(&value.end)?,
        Record::Core(CoreRecord::EscalationDecision(value)) => match &value.decision {
            jig_domain_people::EscalationDecision::Reject { reason } => len_u64(reason.len()),
            jig_domain_people::EscalationDecision::Release | jig_domain_people::EscalationDecision::Pass => 0,
        },
        Record::Core(CoreRecord::ProposalDecision(_)) => 0,
        Record::Notes(notes::Record::Entry(entry)) => len_u64(entry.body.len())
            .checked_add(len_u64(entry.description.len()))?
            .checked_add(List::<u64>::price(entry.references.len())?)?
            .checked_add(match &entry.scope {
                notes::Scope::Party { .. }
                | notes::Scope::Deployment
                | notes::Scope::Project { .. }
                | notes::Scope::Goal { .. } => 0,
                notes::Scope::Resources { pattern, .. } => {
                    let mut total = u64::try_from(size_of_val(pattern)).ok()?;
                    total = total.checked_add(u64::try_from(size_of_val(pattern.segments.as_ref())).ok()?)?;
                    total = total.checked_add(match &pattern.last {
                        notes::Last::Exact(bytes) | notes::Last::Open(bytes) => len_u64(bytes.len()),
                    })?;
                    for segment in &pattern.segments {
                        total = total.checked_add(len_u64(segment.len()))?;
                    }
                    total
                }
            })?,
        Record::Core(
            CoreRecord::Settings(_)
            | CoreRecord::Deployment(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Projection(_),
        )
        | Record::People(_)
        | Record::Notes(notes::Record::Line(_))
        | Record::Accounts(_) => return None,
    };
    carrier.checked_add(owned)
}
#[must_use]
pub fn row_bytes(row: &Row) -> Option<u64> {
    let mut total = u64::try_from(size_of::<Row>())
        .ok()?
        .checked_add(record_bytes(&row.value)?)?
        .checked_add(match &row.owner {
            Some(owner) => tasks::task_bytes(owner)?,
            None => 0,
        })?
        .checked_add(u64::try_from(size_of_val(row.ancestors.as_ref())).ok()?)?
        .checked_add(match &row.related {
            Some(value) => record_bytes(value)?,
            None => 0,
        })?
        .checked_add(u64::try_from(size_of_val(row.linked.as_ref())).ok()?)?;
    for linked in &row.linked {
        total = total.checked_add(tasks::task_bytes(linked)?)?;
    }
    Some(total)
}
/// One setting's value, or an application setting's state and size (jig/docs/design/client/domain/contract/README.md, 4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingDocument {
    pub overlay: Option<Box<crate::settings::Overlay>>,
    pub setting: crate::settings::Setting,
    pub revision: u32,
    pub value: Option<crate::settings::Value>,
    pub application: Option<crate::settings::application::State>,
    pub proposed: Option<crate::settings::application::SizedEdit>,
}
/// Revision values leave through the protocol's single-value byte admission.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RevisionAnswer {
    Revision { revision: crate::settings::Revision, value_bytes: u64 },
    Forgotten { oldest: u32 },
    Unknown,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RevisionEdit {
    Set,
    Removed,
    Reset,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RevisionItemHead {
    pub setting: crate::settings::Setting,
    pub edit: RevisionEdit,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RevisionRow {
    pub head: crate::settings::Head,
    pub items: List<RevisionItemHead>,
    pub more: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RevisionsPage {
    pub rows: List<RevisionRow>,
    pub next: Option<u32>,
    pub forgotten: bool,
}
