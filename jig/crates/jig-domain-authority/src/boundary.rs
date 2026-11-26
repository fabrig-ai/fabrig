//! Owned questions and answers (domain/authority-checks.md, section 1, and domain/authority.md, section 9).

use alloc::boxed::Box;
pub use jig_domain_contract::effects::{Answer, Effect, EffectAccess, EffectResource, Finding, Source, Verdict};

use skein_lib::Wall;

use crate::{Authority, Executor, Judge, Numbers, ProposalKind, ResourceScope, Tools};

/// Pure check result and optional replacement funding snapshot; the root commits allowed numbers
/// together with the action once. (domain/authority-budgets.md, section 1, and domain/authority-checks.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Checked {
    /// Strictest answer after all applicable checks.
    pub answer: Answer,
    /// Replacement funding numbers only on `Allow`; other answers carry `None` and authorize no
    /// debit.
    pub numbers: Option<Numbers>,
}

/// Root-translated directly created task's executor and complete authority, admitted under batch
/// and authority limits. (domain/authority-checks.md, section 1.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Delegate {
    /// Executor permission that the creator must have.
    pub executor: Executor,
    /// Complete child authority, bounded by the configured authority limits.
    pub authority: Authority,
    /// Open-prefix grants narrowed with this delegate's number after the batch is admitted.
    pub symbolic: Box<[crate::Grant]>,
}

/// Root-gathered creation question over one coherent creator and funding snapshot; no tasks or
/// reservations are retained here. (domain/authority-checks.md, section 1.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BatchAsk {
    pub project: u32,
    /// Creator's current authority, bounded by `Limits`.
    pub creator: Authority,
    /// Creator's current allotment snapshot, not a copy of its static ceiling.
    pub numbers: Numbers,
    /// Lifetime task capacity still available, kept separately by tasks. Current lifetime task
    /// capacity available from the tasks ledger.
    pub tasks_left: u32,
    /// Whole all-or-nothing batch, bounded by `Limits::batch`; each child costs one slot plus its
    /// descendant capacity.
    pub tasks: Box<[Delegate]>,
}

/// Root-gathered permission and authentic-fact question for one pinned effect; this value makes no
/// connector call. (domain/authority-checks.md, section 1.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EffectAsk {
    pub project: u32,
    /// Acting task or accepter authority, admitted under `Limits`.
    pub authority: Authority,
    /// Acting task or accepter's current four funding numbers.
    pub numbers: Numbers,
    /// Single pinned connector operation being checked.
    pub effect: Effect,
    /// Wall time of this decision, used for observed verdict freshness.
    pub now: Wall,
}

/// One named judgement and the wall time at which its facts were observed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Given {
    pub judge: Judge,
    pub verdict: Verdict,
    pub at: Wall,
    /// The exact effect state the connector was asked to judge.
    pub state: [u8; 32],
}

/// Root-verified writer-hold relationship for a run's written resource; authority performs no
/// ownership discovery. (domain/authority-checks.md, section 1.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Writer {
    /// Hold belongs to the running task.
    Task,
    /// Hold belongs to a root-verified ancestor of the running task.
    Ancestor,
    /// Hold is unresolved; run admission waits.
    Pending,
    /// Another task holds the writer; run admission waits.
    Other,
}

/// One workspace resource and its root-verified writer relationship, admitted under run write/name
/// limits. (domain/authority-checks.md, section 1.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Write {
    /// Written resource and required connector kind, including saved workspace writes.
    pub effect: Effect,
    /// Root-verified current writer hold.
    pub held: Writer,
}

/// Root-gathered run admission snapshot; no claim, worker, clock or account state is held by this
/// question. (domain/authority-checks.md, section 1.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RunAsk {
    pub project: u32,
    /// Current task authority, bounded by `Limits`.
    pub authority: Authority,
    /// Current funding snapshot used to compute available run spend.
    pub numbers: Numbers,
    /// Offered run budget in the deployment unit; must exceed the minimum and fit all caps.
    pub budget: u64,
    /// Spend needed for the reserve and one most costly completion.
    pub reserve: u64,
    /// Root-supplied current wall time for the deadline check.
    pub wall: Wall,
    /// One status for each model account its charter will use. One usability status per model
    /// account used by the charter, bounded by `Limits::accounts`.
    pub main_provider: u32,
    pub accounts: Box<[AccountCheck]>,
    /// Application credentials are independent of selected LLM sources.
    pub credentials: Box<[bool]>,
    /// Every resource its workspace will write, including saved work. Every written resource,
    /// including saved work, bounded by `Limits::writes`.
    pub writes: Box<[Write]>,
}

/// Root-translated person action after membership verification; authority returns a decision over
/// values, not task mutations. (domain/authority-checks.md, section 1.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PersonRequest {
    /// Check role permission and all-or-nothing batch funding.
    Create(
        /** Whole directly created batch, bounded by `Limits::batch` and each child's authority limits. */
        Box<[Delegate]>,
    ),
    /// A party's batch beneath a task, checked within the role without its pool,
    /// then within the task's remaining (jig/docs/design/domain/authority-checks.md, 1.4).
    Delegate { creator: Authority, tasks: Box<[Delegate]> },
    /// The complete future authority/allotment to fund, not an unchecked delta. Check role
    /// permission and the complete future allotment.
    Allot(/** Complete future authority/allotment to reserve, bounded by authority limits. */ Authority),
    /// Check role acceptance and rights for one proposed action.
    Accept(/** Single proposed action; effect acceptance still needs its pinned-fact check. */ Action),
    /// Check the whole authority left by a widening and what it needs from the giver's pool.
    Amend {
        /// Complete authority after the amendment, bounded by authority limits.
        authority: Authority,
        /// Budget already funded by this pool, or spend retained by the task when funded anew.
        credit: u64,
    },
    /// Check role permission and the authority/allotment funding the move.
    Move(/** Complete authority/allotment to fund for the moved task, bounded by authority limits. */ Authority),
    /// Check specific cancellation permission; does not cancel here.
    Cancel,
    /// Check specific release permission; does not release here.
    Release,
    /// Check specific watch permission; does not create a view here.
    Watch,
    /// Check specific policy-change permission; does not replace policy here.
    Policy,
}

impl PersonRequest {
    /// Funding already covered for a widening (domain/authority-budgets.md, section 1, and domain/authority-checks.md, section 1.4).
    #[must_use]
    pub fn funding_credit(&self) -> u64 {
        match self {
            Self::Amend { credit, .. } => *credit,
            Self::Create(_)
            | Self::Delegate { .. }
            | Self::Allot(_)
            | Self::Accept(_)
            | Self::Move(_)
            | Self::Cancel
            | Self::Release
            | Self::Watch
            | Self::Policy => 0,
        }
    }
}

/// Root-gathered person-role question with the funding snapshot and separate available
/// lifetime capacity. (domain/authority-checks.md, section 1.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PersonAsk {
    pub project: u32,
    pub role: u32,
    /// Funding snapshot: the task's remaining for Delegate, otherwise the person-period pool.
    /// Pool-funded requests check its budget against role period spend.
    pub pool: Numbers,
    /// Current lifetime task capacity available to this funding decision.
    pub tasks_left: u32,
    /// Single translated action checked for specific role permission.
    pub request: PersonRequest,
}

/// Root-translated tool-call requirements beyond its configured family bit; serving the call
/// remains with the root. (domain/authority-checks.md, section 1.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Call {
    /// Family permission alone is checked here.
    Tool,
    /// Family permission and resource read grant are checked.
    Read(/** Pinned resource read requiring a covered grant and bounded name. */ Effect),
    /// Family permission and root-verified task reference are checked.
    Message {
        /** Whether the root verified a reference giving standing to message this task. */
        referenced: bool,
    },
    /// Family permission and one supported scope are checked.
    Note(/** Exactly one note scope, including a connector resource pattern. */ NoteScope),
}

/// Where one note write applies, relative to the acting task and project.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NoteScope {
    /// The acting task's goal.
    Goal,
    /// The task's project.
    Project,
    /// The deployment.
    Deployment,
    /// One connector's resource pattern.
    Resources(ResourceScope),
}

/// Root-gathered call permission question over one configured family and admitted
/// authority/resource values. (domain/authority-checks.md, section 1.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CallAsk {
    pub project: u32,
    /// Current caller authority, admitted under `Limits`.
    pub authority: Authority,
    /// Exactly one family bit, assigned by configuration. Exactly one tool-family bit selected by
    /// the root's registry; other shapes refuse.
    pub family: Tools,
    pub call: Call,
}

/// Owned proposed action used for needs and acceptance checks; caller bounds its payloads and
/// commits acceptance with execution. (domain/authority.md, section 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    /// Propose an all-or-nothing creation batch.
    Batch(/** Whole delegated batch, bounded by batch and authority limits. */ Box<[Delegate]>),
    /// Propose one exact pinned connector effect.
    Effect(/** One pinned connector effect with a bounded resource name. */ Effect),
    /// Propose wider authority for a task.
    Widen(/** Complete requested wider authority, bounded at admission. */ Authority),
    /// Propose authority newly needed by an amendment.
    Amend(/** Authority newly needed by the amendment, bounded at admission. */ Authority),
    /// Topology grants standing; release may additionally need authority. Propose release based on
    /// standing and any additionally needed authority.
    Escalate {
        /// Authority newly needed for release, or `None` when standing alone suffices.
        release: Option<Authority>,
    },
}

/// Root-verified eligible proposal accepter with current funding and task capacity; standing and
/// distance are not discovered here. (domain/authority.md, section 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Holder {
    /// The caller has verified that this task stands above the proposer. Ancestor task whose
    /// standing the caller has verified.
    Task {
        project: u32,
        /// Ancestor's admitted current authority.
        authority: Authority,
        /// Ancestor's current funding snapshot.
        numbers: Numbers,
        /// Ancestor's remaining lifetime task capacity.
        tasks_left: u32,
    },
    /// Person whose role membership and proposal standing the root has verified.
    Person {
        project: u32,
        role: u32,
        /// Proposal kind this role must be allowed to decide.
        proposal: ProposalKind,
        /// Person-period funding snapshot.
        pool: Numbers,
        /// Current lifetime task capacity available to acceptance.
        tasks_left: u32,
    },
}

/// Exact secret-free evidence retained by an allowed run and its durable claim.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AccountBinding {
    pub provider: u32,
    pub account: u32,
    pub source: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AccountCheck {
    pub binding: AccountBinding,
    pub usable: bool,
    pub host_eligible: bool,
}
/// Run-specific answer; shared checks keep their payload-free verdict.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RunAnswer {
    Allow { spend: u64, accounts: Box<[AccountBinding]> },
    Wait,
    Propose,
    Refuse,
}
impl RunAnswer {
    #[must_use]
    pub const fn verdict(&self) -> Answer {
        match self {
            Self::Allow { .. } => Answer::Allow,
            Self::Wait => Answer::Wait,
            Self::Propose => Answer::Propose,
            Self::Refuse => Answer::Refuse,
        }
    }
}
