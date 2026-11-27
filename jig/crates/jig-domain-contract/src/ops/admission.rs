//! Task admission refusals and the facts they name.

use super::{Funder, New, Party};
use alloc::boxed::Box;
use skein_lib::{Decimal, Wall, bytes::concat};

/// An answer names its first writer, not the party on whose behalf it was sent.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum QuestionAnswerer {
    Task(u64),
    Person(u64),
}

/// Content-free terminal classification used in observations and dependency readiness.
/// (domain/tasks/lifecycle.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    /// Task completed successfully; dependents can become due after settlement.
    Done,
    /// Task failed; waiting dependents are held.
    Failed,
    /// Task was cancelled; waiting dependents are held.
    Cancelled,
}

/// Terminal bounded admission reason; refusal makes no requested lifecycle or financial mutation.
/// (domain/tasks/README.md, section 4 and domain/tasks/lifecycle.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    /// No delegate joins a concluded gathering (domain/tasks/gatherings.md, 3).
    Concluded {
        at: Wall,
    },
    NotStanding,
    TalkOpen {
        party: u64,
    },
    NoTalk,
    /// A standing placed hold forbids delegation into its scope or detachment from it.
    Placed {
        hold: u64,
    },
    /// The task caller may not lift this placed hold.
    HoldOwner {
        hold: u64,
    },
    /// The named placed hold revision moved.
    HoldRevision {
        revision: u64,
    },
    /// An explicit account preference was refused before the task changes.
    Account {
        account: u32,
        why: AccountWhy,
    },
    /// A cancel gave a grace longer than the configured span.
    Grace,
    /// The project has no room for more live tracked tasks.
    Goals,
    /// No policy role may accept this proposal kind; nothing of it is committed.
    Unproposable,
    /// The address edit names an older person task revision.
    PersonRevision {
        revision: u64,
    },
    /// A required resource is unavailable in this project.
    ResourceUnavailable,
    /// A batch's reports did not all come within `tasks.hold_wait`;
    /// `blocked_by` names its missing members in batch order (domain/tasks/holds.md, 2).
    Unreported,
    /// Startup is restoring, checked or failed; no mutation is admitted.
    NotReady,
    /// Named creator/task is not in the live arena.
    Unknown,
    /// Finite source table has no room before source creation/carving.
    Busy,
    /// Task/source identity is already retained or a task number repeats within the batch.
    Duplicate,
    /// `Make` batch is empty.
    Empty,
    /// `Batch` exceeds the configured direct-child count.
    Batch,
    /// `Batch` cannot fit the live task slab/name set, including slots not yet reclaimed.
    Live,
    /// Creator/project identity or per-project live task count is invalid.
    Project,
    /// Lifetime requester-tree task count would exceed its configured bound.
    Tree,
    /// A `new` child's structural depth would exceed the configured bound.
    Depth,
    /// Creator's live delegate list cannot fit the batch.
    Delegates,
    /// A sender lacks a live delegation or introduced reference to the target.
    Reference,
    /// Subscription count, identity, target or timer shape was refused.
    Subscription,
    /// `Dependency` count, uniqueness or current-live/same-batch identity is invalid.
    /// An unreachable dependency is named in `blocked_by`; count and duplicate refusals name none.
    Dependencies,
    /// The new waits would close a circle of waits; `blocked_by` names its tasks in order, each
    /// waiting for the next, at most `Limits::circle` (domain/tasks/holds.md, 7).
    Circle {
        cut: bool,
    },
    /// A dependency a party's batch names or an amendment adds has ended failed or cancelled;
    /// `blocked_by` names it (jig/docs/design/domain/tasks/README.md, 4).
    DependencyEnded {
        how: Status,
    },
    /// The task is kept by pending dependencies, which `blocked_by` names, the oldest first: a
    /// person's answer, a procedure's step ending it done, or an effect that waits for them
    /// (domain/tasks/lifecycle-endings.md, 1).
    Kept,
    /// Its own unanswered question keeps it (domain/tasks/lifecycle-endings.md, 1).
    KeptQuestion {
        question: u64,
    },
    /// Its person answers this question in their inbox (domain/people/README.md, 5).
    QuestionAnswered {
        question: u64,
        by: QuestionAnswerer,
    },
    PersonAnswers {
        question: u64,
    },
    /// `Agent` charter is absent from the admitted configured table.
    Executor,
    /// Specification count or aggregate owned bytes are invalid.
    Spec,
    /// `TaskResult`-contract shape or a priced terminal's result-byte admission is invalid.
    Contract,
    /// Carried authority collections or aggregate bytes exceed the configured shape bounds.
    AuthorityShape,
    /// Retained inputs exceed their bound or cannot be resolved for the requesting task.
    Inputs,
    /// Current task phase does not permit the requested admission.
    State,
    /// `Attempt` is stale, zero or incompatible with the current phase.
    Attempt,
    /// Valid finishing declines to cancel still-live delegates; activation and charge remain
    /// unadmitted.
    LiveDelegates,
    /// A retained row does not fit the configured bounds.
    /// Ended families stay outside startup.
    Restore,
    /// A turn's read fence was not an offered unread message.
    Read,
    /// `Turn` is not the next contiguous current-attempt turn, or cumulative expense decreases.
    Turn,
    /// Authentic finite source/reservation or eventual funding-chain arithmetic cannot admit the
    /// whole decision.
    Funding,
    /// A task names an unknown, malformed, or mismatched resource kind.
    HoldKind,
    /// A required exclusive resource is busy and its kind refuses waiting.
    HoldTaken,
    /// A task's hold list or a resource's waiting queue exceeds its bound.
    Holds,
    /// An LLM choice names `model`, which it may not, and why (domain/tasks/README.md, sections 3 and 4).
    Llm {
        model: u8,
        why: LlmFault,
    },
}

/// `Refusal` location and reason returned to the root; batch-wide failures may have no single task
/// identity. (domain/tasks/README.md, section 4 and domain/tasks/lifecycle.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Problem {
    /// Offending task identity when one exists; `None` denotes a batch/source/global failure.
    pub task: Option<u64>,
    /// Bounded structural, lifecycle, financial or restore admission reason.
    pub why: Refusal,
    /// The tasks the refusal names: the live delegates that prevented a finish, in the requester's
    /// current order; a circle's tasks; an unreachable dependency; an ended dependency;
    /// omitted dependencies marked started from; or the
    /// pending dependencies keeping the task; or `Unreported` members whose resources
    /// lack reports, in batch order. Absent for every other refusal.
    pub blocked_by: Option<Box<[u64]>>,
    /// Exact room check metadata; absent for shape, state and other refusals.
    pub capacity: Option<Capacity>,
}

impl QuestionAnswerer {
    #[must_use]
    pub fn party(self) -> Party {
        match self {
            Self::Task(task) => Party::Task(task),
            Self::Person(person) => Party::Person(person),
        }
    }
}

impl Problem {
    /// A refusal with no live-delegate blockers.
    #[must_use]
    pub const fn new(task: Option<u64>, why: Refusal) -> Self {
        Self { task, why, blocked_by: None, capacity: None }
    }

    /// An unchanged refusal carrying owner-proven capacity metadata.
    #[must_use]
    pub const fn capacity(task: Option<u64>, why: Refusal, capacity: Capacity) -> Self {
        Self { task, why, blocked_by: None, capacity: Some(capacity) }
    }

    /// A finish refused until these named delegates end or are cancelled.
    #[must_use]
    pub fn live_delegates(task: u64, delegates: &[u64]) -> Self {
        Self::naming(Some(task), Refusal::LiveDelegates, delegates)
    }

    /// A refusal naming these tasks.
    #[must_use]
    pub fn naming(task: Option<u64>, why: Refusal, tasks: &[u64]) -> Self {
        Self { task, why, blocked_by: Some(Box::from(tasks)), capacity: None }
    }
}

impl Refusal {
    /// Render the refusal itself, when the caller does not carry a whole `Problem`.
    #[must_use]
    pub fn words(self) -> Box<[u8]> {
        let reason: &[u8] = match self {
            Self::Concluded { .. } => b"concluded",
            Self::NotStanding => b"the task does not stand",
            Self::TalkOpen { .. } => b"another party has an open talk",
            Self::NoTalk => b"no talk of yours to leave",
            Self::Placed { .. } => b"held by a placed hold",
            Self::HoldOwner { .. } => b"only the placed hold's owner may release it",
            Self::HoldRevision { .. } => b"placed hold revision changed",
            Self::Unproposable => b"no proposal could be accepted",
            Self::PersonRevision { .. } => b"person revision",
            Self::ResourceUnavailable => b"resource unavailable",
            Self::Unreported => b"unreported",
            Self::NotReady => b"not ready",
            Self::Unknown => b"unknown",
            Self::Busy => b"busy",
            Self::Duplicate => b"duplicate",
            Self::Empty => b"empty",
            Self::Batch => b"batch",
            Self::Live => b"live",
            Self::Project => b"project",
            Self::Tree => b"tree",
            Self::Depth => b"depth",
            Self::Delegates => b"delegates",
            Self::Reference => b"reference",
            Self::Subscription => b"subscription",
            Self::Dependencies => b"dependencies",
            Self::Circle { .. } => b"these tasks would wait for one another",
            Self::DependencyEnded { .. } => b"a dependency has ended",
            Self::QuestionAnswered { question, by } => return concat(&[b"Refused: ", &by.answered_words(question)]),
            Self::PersonAnswers { .. } => b"the person answers this question",
            Self::Kept | Self::KeptQuestion { .. } => b"kept",
            Self::Executor => b"executor",
            Self::Spec => b"spec",
            Self::Contract => b"contract",
            Self::AuthorityShape => b"authority shape",
            Self::Inputs => b"inputs",
            Self::State => b"state",
            Self::Goals => b"goals",
            Self::Grace => b"grace",
            Self::Attempt => b"attempt",
            Self::LiveDelegates => b"live delegates",
            Self::Restore => b"restore",
            Self::Read => b"read",
            Self::Turn => b"turn",
            Self::Funding => b"funding",
            Self::HoldKind => b"hold kind",
            Self::HoldTaken => b"hold taken",
            Self::Holds => b"holds",
            Self::Account { account, why } => {
                let why: &[u8] = match why {
                    AccountWhy::Unknown => b"account not found",
                    AccountWhy::Provider => b"account belongs to another provider",
                    AccountWhy::Uses => b"account does not serve the payer",
                    AccountWhy::Limited => b"kept account is limited",
                };
                return concat(&[b"Refused account ", Decimal::of(u64::from(account)).as_bytes(), b": ", why]);
            }
            Self::Llm { why, .. } => match why {
                LlmFault::Unconfigured => b"model not configured",
                LlmFault::Effort => b"effort not declared",
                LlmFault::Charter => b"model outside the charter",
                LlmFault::Authority => b"model outside authority",
                LlmFault::Procedure => b"no model runs this task",
                LlmFault::Repeated => b"model repeated",
            },
        };
        concat(&[b"Refused: ", reason])
    }
}

impl QuestionAnswerer {
    /// The first answer's author, in tool answer words.
    #[must_use]
    pub fn answered_words(self, question: u64) -> Box<[u8]> {
        let (kind, number) = match self {
            Self::Task(task) => (b"task ".as_slice(), task),
            Self::Person(person) => (b"person ".as_slice(), person),
        };
        concat(&[
            b"question ",
            Decimal::of(question).as_bytes(),
            b" was answered by ",
            kind,
            Decimal::of(number).as_bytes(),
        ])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CapacityAction {
    Batch { first: Option<u64>, members: u32 },
    Proposal { number: u64 },
    PersonProposal { number: u64 },
    Message { task: u64, number: u64 },
    Source { funder: Funder },
    Introduce { left: u64, right: u64 },
    Subscription { number: u64 },
    Effect { entry: u64 },
    Claim { attempt: u64 },
    Release,
    Place { hold: u64 },
    Move,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CapacityCause {
    PlacedHolds { limit: u32 },
    ProjectPlacedHolds { limit: u32 },
    PartyPlacedHolds { limit: u32 },
    Exemptions { limit: u32 },
    Tasks { limit: u32 },
    CreationFlights { limit: u32 },
    ResourceReports { limit: u32 },
    ProjectTasks { limit: u32 },
    Tree { limit: u32 },
    Delegates { limit: u32 },
    Inbox { task: u64, messages: u32, bytes: u32 },
    Holdings { limit: u32 },
    HoldWaiters { limit: u32 },
    Proposals { limit: u32 },
    References { limit: u32 },
    Subscriptions { limit: u32 },
    Funders { limit: u32 },
    Writers { limit: u32 },
}

/// Metadata attached only by the exact failed room check, not inferred from Busy.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Capacity {
    pub project: Option<u32>,
    pub by: Party,
    pub action: CapacityAction,
    pub cause: CapacityCause,
}

impl Capacity {
    /// Describe a whole refused atomic batch without retaining its specifications.
    #[must_use]
    pub fn batch(by: Party, batch: &[New], task: Option<u64>, cause: CapacityCause) -> Self {
        let mut first_number = None;
        let mut project = None;
        if let Some(first) = batch.first() {
            first_number = Some(first.number);
            project = Some(first.project);
            for member in batch {
                if member.project != first.project {
                    project = None;
                }
            }
        }
        if let Some(task) = task {
            for member in batch {
                if member.number == task {
                    project = Some(member.project);
                }
            }
        }
        Self {
            project,
            by,
            action: CapacityAction::Batch {
                first: first_number,
                members: u32::try_from(batch.len()).unwrap_or(u32::MAX),
            },
            cause,
        }
    }
}

/// Why an LLM choice is refused (domain/tasks/README.md, section 4 and domain/tasks/changes.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LlmFault {
    /// The model is not configured.
    Unconfigured,
    /// The effort is not one the model declares, or none is named for a model that declares some.
    Effort,
    /// The model is not one of the task's charter's.
    Charter,
    /// The task's authority does not allow the model (domain/authority.md, section 3).
    Authority,
    /// The task is a procedure's or a person's, whose executor runs no model.
    Procedure,
    /// The sub-agents name the model twice.
    Repeated,
}

/// Why an explicit account cannot fund the prospective task (domain/accounts-uses.md, 1.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountWhy {
    Unknown,
    Provider,
    Uses,
    Limited,
}
