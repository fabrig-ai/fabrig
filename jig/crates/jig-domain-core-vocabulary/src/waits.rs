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

impl WaitKind {
    pub const ALL: [Self; 12] = [
        Self::Party,
        Self::Question,
        Self::Dependency,
        Self::Resource,
        Self::Account,
        Self::Host,
        Self::Delegate,
        Self::News,
        Self::Step,
        Self::Room,
        Self::Turn,
        Self::SignOff,
    ];

    const fn slot(self) -> u32 {
        match self {
            Self::Party => 0,
            Self::Question => 1,
            Self::Dependency => 2,
            Self::Resource => 3,
            Self::Account => 4,
            Self::Host => 5,
            Self::Delegate => 6,
            Self::News => 7,
            Self::Step => 8,
            Self::Room => 9,
            Self::Turn => 10,
            Self::SignOff => 11,
        }
    }
}

/// One wall time per kind (jig/docs/design/domain/engine/waits.md, 7),
/// kept inline so copying the deployment header doesn't allocate.
/// A presence bit distinguishes a floor at the epoch from an unset floor.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WaitFloors {
    at: [Wall; WaitKind::ALL.len()],
    present: u16,
}

impl WaitFloors {
    pub const EMPTY: Self = Self { at: [Wall::EPOCH; WaitKind::ALL.len()], present: 0 };

    #[must_use]
    pub fn floor(&self, kind: WaitKind) -> Option<Wall> {
        let slot = kind.slot();
        let mask = 1_u16.checked_shl(slot).expect("wait kind fits its presence bits");
        if self.present & mask == 0 { None } else { self.at.get(skein_lib::usize_of(slot)).copied() }
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.present.count_ones()
    }

    /// Keep the first wall time for this kind; return false if it is already kept.
    pub fn set_once(&mut self, kind: WaitKind, at: Wall) -> bool {
        let slot = kind.slot();
        let mask = 1_u16.checked_shl(slot).expect("wait kind fits its presence bits");
        if self.present & mask != 0 {
            return false;
        }
        *self.at.get_mut(skein_lib::usize_of(slot)).expect("one floor slot per wait kind") = at;
        self.present |= mask;
        true
    }
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
