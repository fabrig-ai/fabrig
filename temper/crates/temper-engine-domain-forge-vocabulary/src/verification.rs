//! Retained landing verdicts (temper/docs/design/domain/after-landing.md, 5.3, 6 and 10).
use crate::{TierVerification, Verification};
use alloc::boxed::Box;
use core::mem::size_of;

/// What the forge knows of a landing a change names as the red it repairs (temper/docs/design/domain/agent/tools.md, 1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LandingRed {
    /// No live `Verification` row of temper's has this commit in the repository.
    Unknown,
    /// Its changes landed into another branch than the one asked.
    Elsewhere {
        branch: Box<[u8]>,
    },
    /// No tier holds it red: no verdict `Owns` or `Suspect` without its `repaired`.
    NotRed,
    Red,
}
pub(crate) fn row_bytes(row: &Verification) -> Option<u64> {
    let mut held = skein_lib::len_u64(row.changes.len()).checked_mul(8)?.checked_add(
        skein_lib::len_u64(row.tiers.len()).checked_mul(u64::try_from(size_of::<TierVerification>()).ok()?)?,
    )?;
    for tier in &row.tiers {
        held = held.checked_add(skein_lib::len_u64(tier.tier.len()))?;
        if let Some(verdict) = &tier.verdict {
            held = held.checked_add(crate::tiers::blame_bytes(&verdict.blame)?)?;
        }
    }
    Some(held)
}
