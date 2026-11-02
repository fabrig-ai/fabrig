//! Retained tier policy rows (temper/docs/design/domain/after-landing.md, 10).
use crate::{Domain, TierRow};
use alloc::boxed::Box;
use core::mem::size_of;
use temper_engine_domain_forge_tiers as p;

impl Domain {
    #[must_use]
    pub fn tier_task(&self, _task: u64) -> Option<&TierRow> {
        None
    }
    #[must_use]
    pub fn describe_tier_effect(
        &self,
        _task: u64,
        _entry: u64,
    ) -> Option<(skein_forge::api::Repository, crate::What, crate::AgentEffect)> {
        None
    }
}
pub(crate) fn tests_bytes(tests: &[Box<[u8]>]) -> Option<u64> {
    let mut held = u64::try_from(tests.len()).ok()?.checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    for test in tests {
        held = held.checked_add(u64::try_from(test.len()).ok()?)?;
    }
    Some(held)
}
pub(crate) fn failures_bytes(failures: &p::Failures) -> Option<u64> {
    match failures {
        p::Failures::Many => Some(0),
        p::Failures::Tests(tests) => tests_bytes(tests),
    }
}
pub(crate) fn blame_bytes(blame: &p::Blame) -> Option<u64> {
    match blame {
        p::Blame::Cleared => Some(0),
        p::Blame::Owns(f) | p::Blame::Suspect(f) => failures_bytes(f),
    }
}
pub(crate) fn row_bytes(row: &TierRow) -> Option<u64> {
    let mut held = u64::try_from(row.tier.len()).ok()?.checked_add(failures_bytes(&row.record.failing)?)?.checked_add(
        u64::try_from(row.record.flakes.len()).ok()?.checked_mul(u64::try_from(size_of::<p::Flake>()).ok()?)?,
    )?;
    for flake in &row.record.flakes {
        held = held.checked_add(u64::try_from(flake.test.len()).ok()?)?;
    }
    if let Some(run) = &row.record.run {
        held = held.checked_add(
            u64::try_from(run.landings.len()).ok()?.checked_mul(u64::try_from(size_of::<p::Landing>()).ok()?)?,
        )?;
        match &run.part {
            p::Part::Whole => {}
            p::Part::Rerun { failed } => held = held.checked_add(tests_bytes(failed)?)?,
            p::Part::Probe { tests, finding, .. } => {
                match tests {
                    p::Selection::All => {}
                    p::Selection::Tests(tests) => held = held.checked_add(tests_bytes(tests)?)?,
                }
                held = held
                    .checked_add(failures_bytes(&finding.failing)?)?
                    .checked_add(tests_bytes(&finding.flakes)?)?
                    .checked_add(
                        u64::try_from(finding.blamed.len())
                            .ok()?
                            .checked_mul(u64::try_from(size_of::<p::Blame>()).ok()?)?,
                    )?;
                for blame in &finding.blamed {
                    held = held.checked_add(blame_bytes(blame)?)?;
                }
            }
        }
    }
    Some(held)
}
