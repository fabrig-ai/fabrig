//! Resource names, roles and holdings (jig/docs/design/domain/connectors.md, 2).

use crate::ops::{Executor, Refusal, Spec};
use alloc::boxed::Box;
use skein_lib::Token;

/// What one adopted resource permits this project to do.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    /// The deployment owns the resource.
    Owned,
    /// It writes as a participant of a shared resource.
    Participant,
    /// It may read, but may not write.
    Context,
    /// It is known but unavailable.
    Unavailable,
}

/// How a named resource is held while a task uses it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldKind {
    /// No task hold is needed.
    Shared,
    /// One task holds the resource at a time.
    Exclusive { wait: bool },
    /// At most the current number of slots may be held.
    Pooled { slots: u32, wait: bool },
}

/// A connector resource name opaque to the tasks hub (domain/tasks/holds.md, 1).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Name {
    /// Connector number assigned by the application's root.
    pub connector: u16,
    /// Literal path segments with no connector-specific interpretation here.
    pub path: Box<[Box<[u8]>]>,
}

/// One resource a task needs before it may become active.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Holding {
    /// Exclusive write resource and its connector-defined kind.
    Write { resource: Name, kind: u16 },
    /// Counted pool resource and its connector-defined kind.
    Slot { pool: Name, kind: u16 },
}

/// Whether a busy resource refuses admission or queues its next holder.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Taken {
    /// Refuse a batch needing this resource and name its holder.
    Refuses,
    /// Admit the task with no holds until the complete set is free.
    Waits,
}

/// Connector-configured admission rule for one resource kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rule {
    /// The connector orders writes itself; no task hold is taken.
    Shared,
    /// One task holds the resource.
    Exclusive { taken: Taken },
    /// Up to the connector's current number of slots hold the pool.
    Pooled { taken: Taken },
}

/// One connector's declared resource kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceKind {
    pub kind: u16,
    pub hold: Rule,
}

/// Why a connector refuses a member's holdings (domain/tasks/holds.md, 1 and 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldingsRefusal {
    /// The member names a symbol this connector does not offer it.
    Symbol { symbol: u16 },
    /// The member would write a resource this task holds, outside its subtree.
    Held { holder: u64 },
}

/// One member's concrete identity passed to a connector for its own holdings.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HoldingsNeed {
    pub project: u32,
    pub root: u64,
    pub number: u64,
    pub executor: Executor,
    pub spec: Spec,
    pub tracked: bool,
    /// This connector's symbols of the member, in the member's order.
    pub symbols: Box<[u16]>,
}

/// A connector resource on which a run left state for its next activation.
/// The path is opaque to tasks and ordered by literal segment bytes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SavedResource {
    pub connector: u16,
    pub path: Box<[Box<[u8]>]>,
}

/// The one writer of an exclusively held resource (domain/tasks/holds.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Writer {
    /// A claimed agent run, fenced by its attempt.
    Run { task: u64, attempt: u64 },
    /// A connector-owned outbox effect.
    Effect { entry: u64 },
}

/// What the engine asks of a resource connector.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Notification: a task was erased and its names need not be kept; no answer.
    Unname { task: u64 },
    /// Notification: a task took a write hold under this outbox entry; no answer.
    Hold { task: u64, resource: Name, entry: u64 },
    /// Notification: a fenced attempt was lost; its named writes need fresh reads.
    Lost { task: u64, attempt: u64, resources: Box<[Name]> },
    /// Judgement: which holdings a task needs, answered by `Up::Holdings`.
    Holdings {
        request: Token,
        project: u32,
        root: u64,
        number: u64,
        executor: Executor,
        spec: Spec,
        tracked: bool,
        /// This connector's symbols of the member, in the member's order.
        symbols: Box<[u16]>,
    },
    /// Judgement: holdings for a named delegate batch, answered by `Up::DelegateHoldings`.
    DelegateHoldings { request: Token, from: u64, members: Box<[HoldingsNeed]> },
    /// Judgement: holdings for a procedure's delegate batch, answered by `Up::ProcedureHoldings`.
    ProcedureHoldings { task: u64, step: u64, members: Box<[HoldingsNeed]> },
    /// Notification: a repair was refused; no answer.
    RepairRefused { repair: Option<u64>, number: u64, why: Refusal },
    /// Notification: a delegation was refused; no answer.
    DelegateRefused { task: Option<u64> },
}

/// A resource connector's holdings answers and observations.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// The task's holdings, or their refusal.
    Holdings { request: Token, holdings: Option<Box<[Holding]>>, refused: Option<HoldingsRefusal> },
    /// Batch holdings in member order, or their refusal.
    DelegateHoldings { request: Token, holdings: Option<Box<[Box<[Holding]>]>>, refused: Option<HoldingsRefusal> },
    /// Procedure batch holdings in member order, or their refusal.
    ProcedureHoldings {
        task: u64,
        step: u64,
        holdings: Option<Box<[Box<[Holding]>]>>,
        refused: Option<HoldingsRefusal>,
    },
    /// Unasked: a resource's role and hold kind.
    Resource { project: u32, name: Name, role: ResourceRole, hold: HoldKind },
    /// Unasked: a pool's slot count.
    PoolSlots { name: Name, slots: u32 },
    /// Unasked: a task's resource drifted.
    Drift { task: u64, resource: Name },
    /// Unasked: a resource was read afresh after a lost attempt.
    ReadAfresh { resource: Name },
}
