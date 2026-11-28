//! Task and call facts carried by activation snapshots.

use super::{
    Authority, Class, Contract, EndedBy, Executor, LlmChoice, MessageKind, Numbers, Party, PersonAddress, Phase, Spec,
    Word,
};
use crate::resources::SavedResource;
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

/// Stable call identity supplied by a run and scoped by the root's task and
/// claim (domain/engine/decisions.md, section 1.4, and domain/engine/executors-tools.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CallKey {
    pub task: u64,
    pub attempt: u64,
    /// One-based assistant completion.
    pub completion: u32,
    /// Zero-based assistant block position.
    pub position: u32,
}

/// A value a client's request carried, kept in the secrets' store before
/// the core sees the request (`protocol/client/README.md`, section 3): the core
/// gets its number and digest, never the value.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct Carried {
    pub number: u64,
    pub digest: [u8; 32],
}

/// Semantic recipient of one held chat decision. Root verifies current membership
/// and coverage; tasks keeps only this bounded identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EscalationHolder {
    /// Nearest live ancestor task with authority to decide this hold.
    Task(u64),
    /// The chat's person requester, eligible under current policy.
    Person(
        /// Positive requester identity, supplied by root.
        u64,
    ),
    /// Final project policy role; any authenticated current holder may decide
    /// first, and passing further refuses.
    Role {
        /// Project of the held chat, checked by tasks.
        project: u32,
        /// Policy role selected and checked by root, bounded by authority roles.
        role: u32,
    },
}

/// The task or person that decided an escalation while its task remains held
/// (domain/tasks/proposals.md, "An escalation's decider has its kind").
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EscalationDecider {
    /// The live ancestor task that decided.
    Task(u64),
    /// The authenticated person that decided, including through a policy role.
    Person(u64),
}

/// Stop cancels each live run in scope at once; drain lets them finish their turn (4b).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mode {
    Stop,
    Drain,
}

/// A task's subtree, those made later included, or a whole project (4b).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Scope {
    Subtree { task: u64 },
    Project,
}

/// Why a run runs, and the reasons a charter's part is for: one bit each
/// (jig/docs/design/domain/tasks/messages.md, 6; jig/docs/design/protocol/store/records.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reasons(pub u32);

/// Store names of `first` and the choosing kinds, in bit order.
pub const REASON_NAMES: [&[u8]; 14] = [
    b"first",
    b"question",
    b"question_answered",
    b"question_ended",
    b"question_owed",
    b"escalation",
    b"proposal",
    b"person_stalled",
    b"result",
    b"notice",
    b"nothing_awaited",
    b"news",
    b"timer",
    b"words",
];

impl Reasons {
    pub const EMPTY: Self = Self(0);
    pub const FIRST: Self = Self(1 << 0);
    pub const QUESTION: Self = Self(1 << 1);
    pub const QUESTION_ANSWERED: Self = Self(1 << 2);
    pub const QUESTION_ENDED: Self = Self(1 << 3);
    pub const QUESTION_OWED: Self = Self(1 << 4);
    pub const ESCALATION: Self = Self(1 << 5);
    pub const PROPOSAL: Self = Self(1 << 6);
    pub const PERSON_STALLED: Self = Self(1 << 7);
    pub const RESULT: Self = Self(1 << 8);
    pub const NOTICE: Self = Self(1 << 9);
    pub const NOTHING_AWAITED: Self = Self(1 << 10);
    pub const NEWS: Self = Self(1 << 11);
    pub const TIMER: Self = Self(1 << 12);
    pub const WORDS: Self = Self(1 << 13);
    /// Every known bit: what a run under a charter with no parts is offered.
    pub const ALL: Self = Self((1 << 14) - 1);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Whether the two share a bit.
    #[must_use]
    pub const fn meets(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// No bit outside `ALL`.
    #[must_use]
    pub const fn is_known(self) -> bool {
        self.0 & !Self::ALL.0 == 0
    }
    /// The bit of a kind that chooses parts; `EMPTY` for an always-offered kind.
    #[must_use]
    pub fn of(kind: &MessageKind) -> Self {
        match kind {
            MessageKind::Question | MessageKind::PassedQuestion { .. } => Self::QUESTION,
            MessageKind::QuestionAnswered { .. } | MessageKind::PassedAnswer { .. } => Self::QUESTION_ANSWERED,
            MessageKind::QuestionEnded { .. } => Self::QUESTION_ENDED,
            MessageKind::QuestionOwed => Self::QUESTION_OWED,
            MessageKind::Escalation { .. } => Self::ESCALATION,
            MessageKind::Proposal { .. } => Self::PROPOSAL,
            MessageKind::PersonStalled { .. } => Self::PERSON_STALLED,
            MessageKind::Result(_) => Self::RESULT,
            MessageKind::Notice { .. } => Self::NOTICE,
            MessageKind::NothingAwaited => Self::NOTHING_AWAITED,
            MessageKind::News { .. } => Self::NEWS,
            MessageKind::Timer { .. } => Self::TIMER,
            MessageKind::Words => Self::WORDS,
            MessageKind::Answer { .. }
            | MessageKind::ProposalDecision { .. }
            | MessageKind::SettingsDecision { .. }
            | MessageKind::EffectNotMade { .. }
            | MessageKind::Met { .. }
            | MessageKind::Amendment { .. }
            | MessageKind::FinishRequested { .. } => Self::EMPTY,
        }
    }
    /// `first` or a choosing kind by its store name; `None` for every other name.
    #[must_use]
    pub fn named(name: &[u8]) -> Option<Self> {
        let mut bit = 1_u32;
        for known in REASON_NAMES {
            if name == known {
                return Some(Self(bit));
            }
            bit = bit.checked_shl(1).expect("fourteen known reason bits");
        }
        None
    }
}

/// The end of a task's financial chain, independent of its requester.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Payer {
    Party(u64),
    Project(u32),
}

/// Owned semantic preparation snapshot emitted to root, bounded by task limits; not a
/// second mutable `TaskRecord` or `funding` ledger. (domain/tasks/README.md, section 3).
/// (domain/engine/executors.md, section 1, and domain/engine/briefs.md, section 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RunContext {
    /// A standing task's wake, rather than a retry or continuation (standing.md, 5.1).
    pub session_wake: bool,
    /// Kinds waking it, excluding entries and words that are always offered.
    pub session_kinds: Reasons,
    /// Why this run runs (jig/docs/design/domain/tasks/messages.md, 6).
    pub reasons: Reasons,
    pub ended_by: EndedBy,
    pub llm: Option<LlmChoice>,
    /// Current financial payer; preparations must recheck it before admission.
    pub payer: Payer,
    pub task: u64,
    /// Newest admitted inbox identity, including words already taken.
    pub last_message: u64,
    /// Whole unread words carried into a newly prepared run brief.
    pub inbox: Box<[Word]>,
    /// Current direct children and their lifecycle at preparation time.
    pub delegates: Box<[DelegateState]>,
    /// Dependency identities whose ended results the root loads; those still pending are in
    /// `pending`.
    pub dependencies: Box<[u64]>,
    /// The dependencies an amendment added, with when and by whom, for the brief's account of
    /// what the task waited for (domain/engine/briefs.md, section 1).
    pub added: Box<[Added]>,
    /// Pending dependencies keeping the task, the oldest first; a procedure steps knowing them
    /// (domain/tasks/lifecycle-endings.md, 1; domain/connectors-procedures.md, 1.1).
    pub pending: Box<[u64]>,
    /// Resources whose next activation resumes saved state.
    pub saved: Box<[SavedResource]>,
    /// Last claimed attempt; a task-wide transcript load must not use turns from a later claim.
    pub previous_attempt: u64,
    /// Whether a committed conversation exists across this task's attempts.
    pub ever_turned: bool,
    /// Failure classes seen before this preparation, for the brief's attempt summary.
    pub tries: Tries,
    /// Contract reason given to this next run after an invalid terminal.
    pub invalid_result: Option<InvalidResult>,
    pub project: u32,
    /// Task executor selected by the root.
    pub executor: Executor,
    /// Bounded owned semantic description; not rendered brief bytes.
    pub spec: Spec,
    /// Bounded result contract root includes in the brief.
    pub contract: Contract,
    /// Requester included in preparation semantics.
    pub requester: Party,
    /// `Exact` carried permission value root translates to authority's independent vocabulary.
    pub authority: Authority,
    /// Borrowed-then-copied authentic financial snapshot for this preparation; root does not mutate
    /// or persist it as a shadow ledger.
    pub numbers: Numbers,
}

/// A dependency an amendment added to a live task (domain/tasks/changes.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Added {
    pub task: u64,
    /// When the amendment was committed.
    pub at: Wall,
    /// The amendment's maker: a party, or a task amending one below it.
    pub by: Party,
}

/// One direct child's current lifecycle in its requester's next brief.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DelegateState {
    pub task: u64,
    pub phase: Phase,
    pub revisions: Revisions,
}

/// Revisions of the individual things a party decides, independent of history,
/// spend and executor progress (client/domain/contract/documents.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Revisions {
    /// Moves when a hold begins or its reason changes, never when it passes up.
    pub hold: u64,
    /// Moves when an amendment changes the task's specification or policy.
    pub spec: u64,
    /// Moves when a person task's question or displayed results change.
    pub person: u64,
}

/// Why an agent's proposed result failed its current admitted contract.
/// The next run receives this durable reason in its attempt summary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InvalidResult {
    /// The successful result used a different contract form.
    Form,
    /// A verdict named a code absent from the current closed choices.
    Verdict,
    /// The result's words exceeded their admitted bound.
    Words,
    /// A change named a different connector or result kind.
    Change,
    /// The verdict proposed more follow-up tasks than its contract permits.
    Followups,
}

/// One waiting interval of a person task, retained through claims and restart.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PersonWait {
    pub since: Wall,
    pub stall: Duration,
    pub deadline: Wall,
    pub notice: Option<PersonStallNotice>,
    pub offer: Option<PersonOffer>,
}

/// The current offer, alongside (not instead of) the task's first address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PersonOffer {
    pub holder: PersonAddress,
    pub since: Wall,
}

/// One numbered news item issued when a person task passed its wait.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PersonStallNotice {
    pub message: u64,
    pub at: Wall,
    /// Exact immutable outgoing activity in this task's ordered tail.
    pub activity: u64,
}

impl PersonWait {
    /// The stall is counted from arrival at the current holder.
    #[must_use]
    pub fn stall_deadline(self) -> Wall {
        match self.offer {
            Some(offer) => Wall::from_nanos(offer.since.as_nanos().saturating_add(self.stall.as_nanos())),
            None => self.deadline,
        }
    }
}

/// The decision responsible for the most recent retry pause.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RetryNotice {
    /// A classified failure, with its counter and configured allowance.
    Failed { class: Class, count: u32, allowance: u32, reason: Option<Box<[u8]>> },
    /// Preparation or placement refused before execution; no failure try spent.
    Refused { count: u32, reason: Option<Box<[u8]>> },
}

/// Task's per-class failure counters, updated with saturation; counts describe history rather
/// than a retry permission. (domain/tasks/lifecycle-holds.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Tries {
    pub transient: u32,
    /// Count of failures awaiting an external correction.
    pub permanent: u32,
    pub run: u32,
    pub agent: u32,
    pub lost: u32,
    /// Count of results that violated the task contract.
    pub invalid: u32,
    pub infrastructure: u32,
}

impl Tries {
    /// All failure counters zero; a `new` task and a successful park start with this value.
    pub const NONE: Tries =
        Tries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 };

    /// Pure selection of the counter or retry policy for `class`; no allocation, mutation, output
    /// or lifecycle transition.
    #[must_use]
    pub const fn of(&self, class: Class) -> u32 {
        match class {
            Class::Transient => self.transient,
            Class::Permanent => self.permanent,
            Class::Run => self.run,
            Class::Agent => self.agent,
            Class::Lost => self.lost,
            Class::Invalid => self.invalid,
            Class::Infrastructure => self.infrastructure,
        }
    }
}

/// Lifetime ended descendants' outcomes, saturating (domain/tasks/README.md, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Endings {
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}

impl Endings {
    pub const ZERO: Self = Self { done: 0, failed: 0, cancelled: 0 };
}
