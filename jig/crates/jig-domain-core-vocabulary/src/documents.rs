//! Typed document vocabulary and checked allocation bounds
//! (`jig/docs/design/domain/engine/README.md`, 3; `client/domain/contract/documents.md`
//! and `client/domain/contract/README.md`, 5; `domain/engine/views.md`).
//!
//! Text and lists state their loss. Chips retain owner phase clocks, readiness,
//! retries, resource references and endings. Engine receipt clocks in fenced
//! observations are distinct from placement/adoption clocks. Application subjects
//! and connector panels belong to their connectors.

pub mod accounts;
pub mod catalogue;
pub mod chip_phase;
pub mod deployment;
pub mod endings;
pub mod inbox_deployment;
pub mod inbox_selection;
pub mod live_delegates;
pub mod objects;
pub mod offers;
pub mod programs;
pub mod project_note;
pub mod settings;
pub mod settings_review;
pub mod small;
pub mod task_lines;
pub mod task_query;
pub mod task_tree;
pub use task_tree::hold as task_hold;
pub mod task_wait;

pub use accounts::{AccountLine, AccountList, Accounts, LastEnd, SignInLine, SourceLine};
pub use catalogue::{Offered, PayerReadiness, ProviderOffer, Readiness};
pub use chip_phase::{ChipDetails, ChipObservation, ChipWork};
pub use deployment::{
    Account, AccountState, Capacity, Deployment, HelloRefusalLine, Host, HostRun, ProjectCapacity, RunLocation,
    StoreFill,
};
pub use inbox_deployment::{Inbox, InboxEntry, InboxObject};
pub use inbox_selection::{InboxSelection, inbox_selection_bound};
pub use jig_domain_accounts::Use;
pub use live_delegates::LiveDelegate;
pub use objects::{
    AmendmentChange, AmendmentState, EndedDelegate, Escalation, FundingOffer, HistoryLine, MilestoneLine,
    MilestonePhase, PersonTask, Proposal, ProposalAction, ProposalHistory, ProposalHistoryState, ProposalWaiting,
    ProposedTask, Question, ResultDoc, RetainedAction, ShownResult, SpecLine, ended_delegate_from_row,
};
pub use programs::{ProgramHost, ProgramLogin};
pub use project_note::{
    Ledger, Member, Note, NoteLocation, Policy, PolicyRole, Project, ProjectResource, Recurring, Requirement,
    ResourceRole, SpendLine,
};
pub use settings::{Follows, HeldRevision, SettingLine, Settings, SettingsProposal};
pub use settings_review::{ApplicationReview, ReviewItem, SettingsReview, settings_heads};
pub use small::text as bounded_text;
pub use task_lines::{ConversationRead, TaskLine};
pub use task_query::{Lines, PhaseClass, QueryOrder, TaskQuery};
pub use task_tree::{
    AuthorityLine, BudgetLine, ContractLine, Dependency, Ending, EscalationState, FoldedCount, GrantLine, HoldLine,
    NoteScope, Parameter, Part, PartKey, PartKind, PatternEnd, PhaseCounts, PlacedLine, PriorPhase, ProposalRef,
    ResourceLine, ResultValue, Task, TaskLink, TaskSettings, Tree, TreeRow,
};
pub use task_wait::{TaskRunWait, TaskSourceState, TaskWaitRefused, TaskWaitWhy};

pub use crate::runs::waits::WaitsOn;

use crate::Limits;
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_people::ApplicationScope;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, List, Wall};

/// One bounded UTF-8 text value, with loss stated separately.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Text {
    pub bytes: Box<[u8]>,
    pub cut: bool,
}

/// Versions of jig's contract served by this configured deployment.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Versions {
    pub minimum: u32,
    pub maximum: u32,
}

/// Bounds of one constructed document and its configured served versions.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DocumentLimits {
    pub roles: u32,
    pub segments: u32,
    pub grants: u32,
    pub executors: u32,
    pub implications: u32,
    pub rows: u32,
    pub text_bytes: u32,
    pub chips: u32,
    pub lines: u32,
    pub versions: Versions,
}

/// Continuation position in a subject's stable numbered list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cursor {
    /// A task or goal number in the subject's ordered list.
    Number { number: u64 },
    /// Exact identity of one inbox row; number families may overlap.
    Inbox { task: u64, object: InboxObject, at: Option<Wall> },
}

/// The bounded slice requested by a watch or read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Window {
    pub rows: u32,
    pub after: Option<Cursor>,
}

/// Identity of the immutable proposal's maker.
pub use jig_domain_people::Proposer;

/// One subject whose current document a party may read or watch.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Subject {
    /// A connector-owned subject within the client's stated role scope.
    Application {
        scope: ApplicationScope,
        connector: u16,
        subject: skein_lib::Token,
    },
    /// The authenticated party's frame.
    Frame {
        party: u64,
    },
    /// A task's overview and controls.
    Task {
        task: u64,
    },
    /// A task's live requester subtree.
    Tree {
        task: u64,
    },
    /// Each named task's compact live status.
    Chips {
        tasks: List<u64>,
    },
    /// The party's waiting objects and unread updates.
    Inbox {
        party: u64,
    },
    /// The party's accounts.
    Accounts {
        party: u64,
    },
    /// Filtered live tasks in one project, newest change first.
    /// A fixed task query, independent of any application shape.
    Query {
        query: TaskQuery,
    },
    /// A task's current attempt.
    Run {
        task: u64,
    },
    /// Hosts, accounts and capacity.
    Deployment,
    Settings,
    /// An immutable pending proposal.
    Proposal {
        proposer: Proposer,
        number: u64,
    },
    /// A held task's escalation.
    Escalation {
        task: u64,
    },
    /// An unanswered question.
    Question {
        task: u64,
        message: u64,
    },
    /// A task addressed to a person or role.
    PersonTask {
        task: u64,
    },
    /// A result still named by the live engine.
    Result {
        task: u64,
    },
    /// A project's resources, policy and funding.
    Project {
        project: u32,
    },
    /// A note retained by its owning child.
    Note {
        note: u64,
    },
}

/// Watch routing categories (`client/domain/contract/README.md`, section 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SubjectShape {
    Application { scope: ApplicationScope, connector: u16, subject: skein_lib::Token },
    Run(u64),
    Retained(u64),
    Chips,
    Other,
}

/// Changes that mark subjects without consulting live child state
/// (`client/domain/contract/README.md`, section 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Touch {
    Party(u64),
    Frames,
    Settings,
    Deployment,
    Accounts(Option<u64>),
    AccountLines(Option<u64>),
}

impl Subject {
    /// Select the watch route without exposing unrelated document subjects.
    #[must_use]
    pub fn shape(&self) -> SubjectShape {
        match self {
            Self::Application { scope, connector, subject } => {
                SubjectShape::Application { scope: *scope, connector: *connector, subject: *subject }
            }
            Self::Run { task } => SubjectShape::Run(*task),
            Self::Task { task } | Self::Tree { task } => SubjectShape::Retained(*task),
            Self::Chips { .. } => SubjectShape::Chips,
            Self::Frame { .. }
            | Self::Inbox { .. }
            | Self::Accounts { .. }
            | Self::Query { .. }
            | Self::Settings
            | Self::Deployment
            | Self::Proposal { .. }
            | Self::Escalation { .. }
            | Self::Question { .. }
            | Self::PersonTask { .. }
            | Self::Result { .. }
            | Self::Project { .. }
            | Self::Note { .. } => SubjectShape::Other,
        }
    }

    /// Whether these changed facts affect this watched document.
    #[must_use]
    pub fn touched_by(&self, touch: Touch) -> bool {
        match self {
            Self::Frame { party: owner } => match touch {
                Touch::Party(party) => party == *owner,
                Touch::Frames => true,
                Touch::Accounts(party) => party.is_none() || party == Some(*owner),
                Touch::Settings | Touch::Deployment | Touch::AccountLines(_) => false,
            },
            Self::Inbox { party } => touch == Touch::Party(*party),
            Self::Settings => touch == Touch::Settings,
            Self::Deployment => match touch {
                Touch::Deployment | Touch::AccountLines(_) => true,
                Touch::Party(_) | Touch::Frames | Touch::Settings | Touch::Accounts(_) => false,
            },
            Self::Accounts { party: owner } => match touch {
                Touch::Accounts(party) | Touch::AccountLines(party) => party.is_none() || party == Some(*owner),
                Touch::Party(_) | Touch::Frames | Touch::Settings | Touch::Deployment => false,
            },
            Self::Application { .. }
            | Self::Task { .. }
            | Self::Tree { .. }
            | Self::Run { .. }
            | Self::Chips { .. }
            | Self::Query { .. }
            | Self::Proposal { .. }
            | Self::Escalation { .. }
            | Self::Question { .. }
            | Self::PersonTask { .. }
            | Self::Result { .. }
            | Self::Project { .. }
            | Self::Note { .. } => false,
        }
    }

    /// Borrow the bounded task names of a chips watch.
    #[must_use]
    pub fn chips(&self) -> Option<&List<u64>> {
        match self {
            Self::Chips { tasks } => Some(tasks),
            Self::Application { .. }
            | Self::Task { .. }
            | Self::Tree { .. }
            | Self::Run { .. }
            | Self::Frame { .. }
            | Self::Inbox { .. }
            | Self::Accounts { .. }
            | Self::Query { .. }
            | Self::Settings
            | Self::Deployment
            | Self::Proposal { .. }
            | Self::Escalation { .. }
            | Self::Question { .. }
            | Self::PersonTask { .. }
            | Self::Result { .. }
            | Self::Project { .. }
            | Self::Note { .. } => None,
        }
    }
}

/// The subjects a watch may name (`protocol/client/README.md`, section 5).
/// Own subjects take their party from the authenticated sign-in in the core.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Watched {
    Application { scope: ApplicationScope, connector: u16, subject: skein_lib::Token },
    Frame,
    Task { task: u64 },
    Tree { task: u64 },
    Chips { tasks: List<u64> },
    Inbox,
    Accounts,
    Query { query: TaskQuery },
    Run { task: u64 },
    Deployment,
    Settings,
}

/// Why no live document is available.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing {
    /// No visible subject is known by this name.
    Unknown,
    /// The supplied party may not see this subject.
    NotAllowed,
    /// The subject belongs to stored history.
    Ended,
    /// The requested whole value exceeds construction bounds.
    TooLarge,
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
    Active(tasks::Active),
    /// Waiting for the addressed party or role.
    Person { address: tasks::PersonAddress, taken_by: Option<u64>, since: Option<Wall> },
    /// Closing through its ordered obligations.
    Closing(tasks::Stage),
    /// Held for this reason at this independent revision.
    Held { reason: tasks::Hold, revision: u64 },
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

/// A held project role and the current funding facts available to the party.
/// Each project has its kept name (domain/model.md, 6.1); unopened pools have no ledger.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FrameProjectRole {
    /// Catalogue models beyond the chat charter's bounds (domain/engine/settings.md, 11).
    pub left_out: u32,
    pub project: u32,
    pub role: people::Role,
    pub name: Text,
    pub pool: Option<Ledger>,
    /// Direct creating bounds for this party and project (client/domain/contract/documents.md, 3).
    pub creating: Option<crate::acts::Creating>,
    /// The catalogue models the party's tasks here may use (domain/authority.md, section 3).
    pub models: jig_domain_authority::Models,
    /// Configured templates with this project's policy overrides applied.
    pub templates: List<crate::Template>,
    /// Current usable models and accounts (domain/engine/views.md, 1).
    pub offer: List<Offered>,
    /// Effective templates' current previews, without account reservations.
    pub defaults: List<(u32, tasks::LlmChoice)>,
    /// Whether the project is public (jig/docs/design/domain/people/public-reading.md, 2).
    pub public: bool,
}

/// Party identity, current roles, bounded counts and contract configuration.
/// Deployment membership is supplied independently of the bounded project rows
/// (`client/domain/contract/README.md`, section 2 and `client/domain/contract/documents.md` and `client/domain/contract/README.md`, section 5.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Frame {
    pub party: u64,
    pub kind: people::Kind,
    pub login: Text,
    pub name: Text,
    pub deployment_role: Option<people::Role>,
    pub projects: List<FrameProjectRole>,
    pub needs: u32,
    /// Exact kinds, including needs beyond the recent inbox prefix.
    pub need_counts: people::Needs,
    pub updates: Count,
    pub results_seen: u64,
    pub retention: Duration,
    pub cut: bool,
    /// Public configured provider offers within the document line bound.
    pub providers: List<ProviderOffer>,
    /// Whether additional provider offers were omitted.
    pub providers_cut: bool,
    /// Startup key ceiling, including the client carried-value bound.
    pub key_bytes: u32,
    /// The driver's redirect bound: the most bytes a pasted sign-in address may have.
    pub redirect_bytes: u32,
    /// The most bytes a party's words may have: a task's spec, a message, an answer.
    pub words_bytes: u32,
}

/// A live attempt's committed progress; provisional writing is supplied by watches.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Run {
    pub task: u64,
    pub attempt: Option<u64>,
    pub committed_turn: u32,
    pub writing: Option<jig_domain_views::Writing>,
    /// Current content call, independently of committed or absent main writing.
    pub call: Option<jig_domain_views::Call>,
}

/// One whole typed document, within its configured limits.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Document {
    /// The party's frame.
    Frame(Frame),
    /// Compact status of each named task.
    Chips(List<Chip>),
    /// Current attempt and committed turn.
    Run(Run),
    /// One task page's current overview.
    Task(Box<Task>),
    /// A bounded live subtree with folds.
    Tree(Tree),
    /// A filtered live project task list.
    /// A query window over live tasks.
    Lines(Lines),
    /// A pending immutable proposal.
    Proposal(Box<Proposal>),
    /// A held task and its preserved story.
    Escalation(Escalation),
    /// Whole unanswered question text.
    Question(Question),
    /// A current person task and its form.
    PersonTask(Box<PersonTask>),
    /// An outcome retained in a live closing row.
    Result(Box<ResultDoc>),
    /// Project resources, roles, policy and ledgers.
    Project(Box<Project>),
    /// A note loaded by its owning child.
    Note(Note),
    /// Party-only bounded waiting and unread groups.
    Inbox(Inbox),
    /// The party's own accounts and sign-ins.
    Accounts(Box<Accounts>),
    /// Fleet placement, accounts and finite capacities.
    Deployment(Box<Deployment>),
    Settings(Box<Settings>),
}

/// Checked allocation bound excluding live delegates and task-limit context;
/// `document_with_context_bound` prices those. Parents price each retained delivery separately.
#[must_use]
pub fn document_bound(limits: &DocumentLimits) -> Option<u64> {
    if limits.rows == 0
        || limits.text_bytes == 0
        || limits.chips == 0
        || limits.lines == 0
        || limits.versions.minimum == 0
        || limits.versions.minimum > limits.versions.maximum
    {
        return None;
    }
    let chip = chip_phase::heap_bound(limits)?;
    let chips = List::<Chip>::price(limits.chips)?.checked_add(u64::from(limits.chips).checked_mul(chip)?)?;
    let charter = List::<u32>::price(limits.lines)?;
    let creating = charter
        .checked_add(List::<crate::acts::CreateProcedure>::price(limits.lines)?)?
        .checked_add(List::<u32>::price(limits.roles)?)?
        .checked_add(u64::try_from(size_of::<AuthorityLine>()).ok()?)?
        .checked_add(task_tree::authority_bound(limits)?)?;
    let frame = List::<FrameProjectRole>::price(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_mul(creating)?)?
        .checked_add(u64::from(limits.lines).checked_mul(u64::from(limits.text_bytes))?)?
        .checked_add(u64::from(limits.text_bytes).checked_mul(2)?)?
        .checked_add(catalogue::offers_bound(limits)?)?;
    u64::try_from(size_of::<Document>()).ok()?.checked_add(
        chips
            .max(frame)
            .max(u64::from(limits.text_bytes).checked_mul(3)?)
            .max(task_tree::document_bound(limits)?)
            .max(task_query::document_bound(limits)?)
            .max(objects::document_bound(limits)?)
            .max(project_note::document_bound(limits)?)
            .max(deployment::document_bound(limits)?)
            .max(accounts::document_bound(limits)?),
    )
}

/// Bound of a document including live delegates up to `tasks.delegates`, their
/// chip heaps, whole settings-proposal reasons and the inbox's requester selection context.
#[must_use]
pub fn document_with_context_bound(limits: &Limits, kinds: &[crate::settings::Kind]) -> Option<u64> {
    document_bound(&limits.documents)?
        .max(project_note::note_bound(limits)?)
        .checked_add(List::<LiveDelegate>::price(limits.tasks.delegates)?)?
        .checked_add(u64::from(limits.tasks.delegates).checked_mul(chip_phase::heap_bound(&limits.documents)?)?)?
        .checked_add(settings::document_bound(limits, kinds)?)?
        .checked_add(settings_review::document_bound(limits)?)?
        .checked_add(u64::from(limits.people.words))?
        .checked_add(inbox_selection_bound(limits.documents.rows, limits.tasks.depth)?)?
        .checked_add(u64::from(limits.documents.lines).checked_mul(crate::settings::templates::list_bound(limits)?)?)?
        .checked_add(u64::from(limits.documents.lines).checked_mul(frame_offer_bound(limits)?)?)
}

fn frame_offer_bound(limits: &Limits) -> Option<u64> {
    let offers = List::<Offered>::price(tasks::MODELS)?
        .checked_add(u64::from(tasks::MODELS).checked_mul(List::<u32>::price(limits.accounts.accounts)?)?)?;
    let defaults = List::<(u32, tasks::LlmChoice)>::price(limits.templates)?
        .checked_add(u64::from(limits.templates).checked_mul(List::<tasks::ModelEffort>::price(tasks::MODELS)?)?)?;
    offers.checked_add(defaults)
}
#[must_use]
pub fn chip_heap_bound(limits: &DocumentLimits) -> Option<u64> {
    chip_phase::heap_bound(limits)
}

/// Construct a bounded chip from a loaded owner row, without live observations
/// or restoring it. Whole opaque resource keys must fit the receiving limits.
pub fn chip_from_row(limits: &Limits, row: &tasks::TaskRecord) -> Result<Chip, Missing> {
    small::chip_from_row(limits, row)
}
