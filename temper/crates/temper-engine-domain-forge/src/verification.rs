//! Retained landing verdicts (temper/docs/design/domain/after-landing.md, 10).
use crate::{Domain, Verification};
use core::mem::size_of;
impl Domain {
    #[must_use]
    pub fn change_verification(&self, _task: u64) -> temper_engine_domain_forge_change::Verification {
        temper_engine_domain_forge_change::Verification::Unfollowed
    }
}
pub(crate) fn row_bytes(row: &Verification) -> Option<u64> {
    let mut held = u64::try_from(row.changes.len()).ok()?.checked_mul(8)?.checked_add(
        u64::try_from(row.tiers.len()).ok()?.checked_mul(u64::try_from(size_of::<crate::TierVerification>()).ok()?)?,
    )?;
    for tier in &row.tiers {
        held = held.checked_add(u64::try_from(tier.tier.len()).ok()?)?;
        if let Some(verdict) = &tier.verdict {
            held = held.checked_add(crate::tiers::blame_bytes(&verdict.blame)?)?;
        }
    }
    Some(held)
}
