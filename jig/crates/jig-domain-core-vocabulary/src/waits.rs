pub use jig_domain_contract::ops::{AtBound, Thing, Unbounded, WaitKind, WaitObject};
pub use jig_domain_contract::parts::{Bound, Nearest, Wait};
pub use jig_domain_contract::procedures::{ReportRefusal, Reported, ReportedBound};

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

use skein_lib::{List, Wall};

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
pub struct Waits {
    /// The first wait on a line; it may belong to a delegate in the subtree.
    pub first: Option<Wait>,
    pub nearest: Option<Nearest>,
    /// The task's own waits, in order, within the document's line limit.
    pub own: List<Wait>,
    /// All of its waits, including those beyond the line limit.
    pub count: u32,
}
