use skein_lib::Duration;

/// The core's figures for waits with no clock of their own (jig/docs/design/domain/engine/waits.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Figures {
    pub question: Duration,
    pub party: Duration,
    pub news: Duration,
    pub host: Duration,
    pub account: Duration,
    pub step: Duration,
    pub room: Duration,
    pub sign_off: Duration,
}

use crate::documents::TaskRunWait;
use jig_domain_people::EntryKind;
use jig_domain_tasks::{EscalationHolder, Name};
use skein_lib::{List, Wall};

/// One episode of waiting (jig/docs/design/domain/engine/waits.md, 2).
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
    Entry { kind: EntryKind, at: EscalationHolder },
    Question { number: u64, answerer: u64 },
    Task(u64),
    Resource { name: Name, holder: Option<u64> },
    Run(TaskRunWait),
    News { subscriptions: List<u64>, timer: Option<Wall> },
    Procedure { step: u32, things: List<Thing> },
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Waits {
    /// The first wait on a line; it may belong to a delegate in the subtree.
    pub first: Option<Wait>,
    pub nearest: Option<Nearest>,
    /// The task's own waits, in order, within the document's line limit.
    pub own: List<Wait>,
    /// All of its waits, including those beyond the line limit.
    pub count: u32,
}
