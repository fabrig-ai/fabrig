//! Task hold facts (client/domain/objects.md, 5 and 11).
use super::{HoldCategory, OwnerChipHold, Revision};
use alloc::boxed::Box;

/// Placed hold facts supplied by the engine (client/domain/contract.md, 5.6).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlacedHold {
    pub number: u64,
    pub owner: super::HoldOwner,
    pub mode: HoldMode,
    pub scope: HoldScope,
    pub reason: Box<[u8]>,
    pub placed_at: skein_lib::Wall,
    pub revision: Revision,
    pub lifted: bool,
    pub may_lift: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoldMode {
    Stop,
    Drain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoldScope {
    Subtree { task: u64 },
    Project,
}

impl OwnerChipHold {
    #[must_use]
    pub fn placed(&self) -> Option<&PlacedHold> {
        self.placed.as_ref()
    }
    /// The independently named hold revision, never the task spec's revision.
    #[must_use]
    pub fn revision(&self) -> Revision {
        Revision { value: self.revision }
    }
    /// Only the supplied uncertain-effect hold requires a repeat decision.
    #[must_use]
    pub fn uncertain(&self) -> bool {
        matches!(self.reason, HoldCategory::Uncertain { .. })
    }
}
