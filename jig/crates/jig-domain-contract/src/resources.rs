//! Resource names, roles and holdings (jig/docs/design/domain/connectors.md, 2).

use alloc::boxed::Box;

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
