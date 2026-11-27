//! Effect identities, descriptions and verdicts (jig/docs/design/domain/connectors.md, 2).

use crate::ops::{CallKey, Party, Proposal};
use alloc::boxed::Box;
use skein_lib::{ReplyTo, Wall};

/// The purpose fixes the key independently of retry or outbox numbering.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectPurpose {
    Call {
        attempt: u64,
        completion: u32,
        position: u32,
    },
    Procedure {
        purpose: u64,
    },
    /// A connector's stable goal projection purpose.
    Projection {
        purpose: u64,
    },
}

/// Full deployment identity and the effect's stable purpose.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EffectKey {
    pub deployment: [u8; 16],
    pub task: u64,
    pub origin: EffectPurpose,
    pub purpose: u64,
}

/// What a connector's effect does to its resource.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectForm {
    Creation,
    Transition,
    Set,
}

/// Promise available after an effect's uncertain outcome.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Recovery {
    Keyed,
    Conditional,
    Idempotent,
    Unrecoverable,
}

/// One connector's declared effect kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EffectKind {
    pub kind: u16,
    pub form: EffectForm,
    pub recovery: Recovery,
    pub waits_for_dependencies: bool,
}

/// The inspectable part of a connector-owned effect.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EffectDescription {
    /// Connector number assigned by the application root.
    pub connector: u16,
    /// Stable purpose within the asking task.
    pub purpose: u64,
    /// Authority's resource and state description.
    pub effect: Effect,
}

/// Why a described effect was not admitted, including refusals before authority checked it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectRefusal {
    Authority,
    Obsolete,
    Description,
    Capacity,
    Dependencies,
    Proposal,
    Unavailable,
}

/// An outbox terminal or uncertain write.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OutboxOutcome {
    Made,
    Failed,
    Withdrawn,
    /// A person settled an uncertain effect as not made.
    NotMade,
    Uncertain,
    /// The connector cannot safely retry this entry; a person must resolve it.
    Held {
        entry: u64,
    },
}

/// Ordered from least to most strict: deciding never clears a fact failure.
/// Terminal result of one pure check, ordered by severity; it neither executes the action nor
/// commits a durable decision. (domain/authority-checks.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Answer {
    /// Checks permit the action; the caller still owns execution and atomic commitment.
    Allow,
    /// Required facts, budget, account or writer condition are not ready; no action is authorized
    /// yet.
    Wait,
    /// Task authority is insufficient but hard ceilings permit seeking an eligible accepter.
    Propose,
    /// A hard requirement, admission bound or permission failed.
    Refuse,
}

/// Authority layer that supplied a ceiling or permission for a finding. (domain/authority.md, section 6, and domain/authority-checks.md, section 1
/// ).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Source {
    /// The acting or creating task's authority.
    Task,
    /// The root-verified person's project role.
    Role,
    /// The live project's ceiling.
    Project,
    /// Immutable deployment rules.
    Deployment,
}

/// Fixed-size reason emitted by a pure check into caller-reserved bounded room; multiple
/// independent reasons may accompany one answer. (domain/authority-checks.md, section 1, and domain/authority.md, sections 9–10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Finding {
    /// Input shape or payload exceeds admission limits.
    Oversized,
    /// No live policy exists for the named project.
    UnknownProject,
    /// The live policy has no named role.
    UnknownRole,
    /// Componentwise authority deficits against this source.
    Authority {
        /** Authority layer against which this condition failed. */
        source: Source,
        /** Independent component deficits against that source. */
        lacks: Lacks,
    },
    /// A child executor is not permitted by this source.
    Executor {
        /** Authority layer against which this condition failed. */
        source: Source,
    },
    /// Aggregate lifetime task capacity does not fit this source.
    Tasks {
        /** Authority layer against which this condition failed. */
        source: Source,
    },
    /// Aggregate reserved spend does not fit this source.
    Spend {
        /** Authority layer against which this condition failed. */
        source: Source,
    },
    /// Required accounting or aggregate arithmetic is unrepresentable.
    Arithmetic,
    /// Offered run budget is not above the minimum or available funding is insufficient.
    RunBudget,
    /// Offered run budget exceeds deployment or task authority cap.
    RunCap,
    /// Task deadline is past the supplied wall time.
    Deadline,
    /// A required model account is unusable.
    Account,
    /// A written resource is held by another task or has a pending hold.
    Writer,
    /// Configured family is absent from an applicable authority value.
    Tool,
    /// This source has no grant covering the requested operation.
    Grant {
        /** Authority layer against which this condition failed. */
        source: Source,
    },
    /// The effect's maximum price exceeds funding available at this source.
    Price { source: Source },
    /// The adopted resource is context only or current access forbids a write.
    ResourceAccess,
    /// A connector reports a resource unavailable in the project.
    ResourceUnavailable,
    /// The caller has no verified standing to message the task.
    Reference,
    /// The note scope is absent from this source.
    Scope {
        /** Authority layer against which this condition failed. */
        source: Source,
    },
    /// A pinned required fact is missing, unknown or pending.
    Required { judge: Judge },
    /// A matching pinned required fact failed.
    Failed { judge: Judge },
    /// The effect kind cannot guard a requirement marked as requiring a guard.
    Unguarded { judge: Judge },
    /// The person's role does not allow this request kind.
    Unpermitted,
    /// The person's role may not accept this proposal kind.
    Undecidable,
    /// A funding request's pool budget exceeds the role's period ceiling.
    PeriodSpend,
}

/// A connector's current write relationship for this exact effect. Participating writes are
/// permitted as a participant, subject to the same grant and requirement checks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectAccess {
    Owned,
    Participant,
    Context,
    Unavailable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EffectResource {
    pub name: Name,
    pub access: EffectAccess,
}

/// A read or effect of one connector, pinned to the state it names.
/// Root-translated connector read or effect with a literal resource and exact state pin, bounded on
/// admission. (domain/authority-checks.md, section 1.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Effect {
    pub connector: u16,
    /// Requested exact kind; grant implications do not remove requirements on this kind.
    pub kind: u16,
    /// Full literal resource name, bounded by segment and byte limits.
    pub name: Name,
    pub state: [u8; 32],
    /// Maximum deployment-unit charge, posted in the effect's deciding commit.
    pub price: Option<u64>,
    /// Connector-verified current access to the resource and object being written.
    pub access: EffectAccess,
    /// Other resources touched by this same effect, each with its own current access.
    pub additional: Box<[EffectResource]>,
    /// Judges this effect kind's condition checks when it is applied.
    pub guards: Box<[Judge]>,
}

/// A connector's judgement for the exact state of the checked effect.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Verdict {
    /// The requirement held when judged.
    Met,
    /// The judge cannot yet say whether it holds.
    Wait,
    /// The requirement failed.
    Refuse,
}

/// Every independent component that failed; a check can name all of them
/// in one bounded finding without losing the stricter answer.
/// Independent component deficits summarized in one fixed-size value; ordering is permission
/// inclusion, not structural equality. (domain/authority.md, sections 4–5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "these are independent component deficits, not lifecycle state")]
pub struct Lacks {
    pub tools: bool,
    /// Some requested grant is not covered.
    pub grants: bool,
    pub executors: bool,
    /// Requested lifetime task capacity does not fit.
    pub tasks: bool,
    /// Requested descendant depth does not fit.
    pub depth: bool,
    /// Requested spend does not fit current available funding or authority.
    pub spend: bool,
    /// Requested deadline is later than permitted.
    pub deadline: bool,
    pub notes: bool,
    /// Some requested model is not among the other's.
    pub models: bool,
}

impl Lacks {
    /// Pure query returning whether this bounded value has no entries or deficits; emits no output
    /// and allocates nothing.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        !(self.tools
            || self.grants
            || self.executors
            || self.tasks
            || self.depth
            || self.spend
            || self.deadline
            || self.notes
            || self.models)
    }
}

/// A judge selected by policy. Its parameters name configuration held by that connector.
/// (domain/authority.md, section 10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Judge {
    pub connector: u16,
    pub requirement: u16,
    pub parameters: u32,
}

/// A connector's resource path. Bytes and segment boundaries are literal:
/// empty segments are permitted, with no parsing or normalization here.
/// Owned literal resource segments; callers admit segment count and bytes before value queries
/// that do not perform admission themselves. (domain/authority.md, sections 3–5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name {
    /// Literal ordered segments, bounded by `Limits::segments` and `Limits::segment_bytes` when
    /// admitted.
    pub segments: Box<[Box<[u8]>]>,
}

/// Who requested the connector-owned effect and when a call must answer.
#[derive(Hash, Debug)]
pub enum EffectOrigin {
    /// A connector-owned goal projection, independent of executable task grants.
    Projection {
        goal: u64,
        entry: Option<u64>,
    },
    Call {
        to: ReplyTo,
        key: CallKey,
        deadline: Wall,
    },
    /// An explicit proposal keeps its connector payload without making it.
    Propose {
        to: ReplyTo,
        key: CallKey,
        reason: Box<[u8]>,
        as_holder: bool,
    },
    /// A holder decision already authenticated by the core's proposal route.
    Accept {
        to: ReplyTo,
        key: Option<CallKey>,
        proposal: Box<Proposal>,
        by: Party,
    },
    Procedure {
        task: u64,
        step: u64,
        entry: Option<u64>,
    },
}

impl EffectOrigin {
    #[must_use]
    pub fn task(&self) -> u64 {
        match self {
            Self::Projection { goal, .. } => *goal,
            Self::Call { key, .. } | Self::Propose { key, .. } => key.task,
            Self::Accept { proposal, .. } => proposal.proposer,
            Self::Procedure { task, .. } => *task,
        }
    }
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
