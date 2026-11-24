//! The engine's wait records as supplied to clients (jig/docs/design/client/domain/contract/documents.md, 1).
use super::{EscalationHolder, TaskRunWait, WaitedFor};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// One episode of waiting, possibly in a descendant of the task on the line.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Wait {
    pub task: u64,
    pub kind: WaitKind,
    pub object: WaitObject,
    pub since: Wall,
    pub bound: Bound,
    pub hears: Option<EscalationHolder>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WaitKind {
    Party,
    Question,
    Dependency,
    Resource,
    Account,
    Host,
    Delegate,
    News,
    Step,
    Room,
    Turn,
    SignOff,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum WaitObject {
    Entry { kind: WaitedFor, number: u64, at: EscalationHolder },
    Question { number: u64, answerer: u64 },
    Task(u64),
    Resource { name: ResourceName, holder: Option<u64> },
    Run(TaskRunWait),
    News { subscriptions: List<u64>, timer: Option<Wall> },
    Procedure { step: u32, things: List<Thing> },
}

/// The connector and the opaque byte segments of a literal resource name.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResourceName {
    pub connector: u16,
    pub path: List<Box<[u8]>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Thing {
    pub what: u32,
    pub number: u64,
    pub owner: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Bound {
    At { at: Wall, then: AtBound },
    None(Unbounded),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AtBound {
    Hold,
    PassUp,
    Overdue,
    GiveUp,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unbounded {
    NoDeadline,
    Standing,
    Behind(u64),
    OperatorHold,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Nearest {
    pub at: Wall,
    pub then: AtBound,
    pub kind: WaitKind,
}
