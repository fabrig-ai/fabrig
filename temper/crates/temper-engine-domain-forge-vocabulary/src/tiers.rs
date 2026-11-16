//! Tiers follow their configured branches (temper/docs/design/domain/after-landing.md, 2, 3, 4 and 10).
use crate::{TierLanding, TierRow};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api;
use temper_engine_domain_forge_tiers as p;

/// Logged members plus live changes whose landing rotated out of the ring.
pub fn posting_comments(limits: &crate::Limits) -> Option<u32> {
    limits.landings_kept.checked_mul(limits.landing_members)?.checked_add(limits.changes)
}
pub(crate) fn tests_bytes(tests: &[Box<[u8]>]) -> Option<u64> {
    let mut held = skein_lib::len_u64(tests.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    for test in tests {
        held = held.checked_add(skein_lib::len_u64(test.len()))?;
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
pub fn row_bytes(row: &TierRow) -> Option<u64> {
    let mut held = skein_lib::len_u64(row.tier.len())
        .checked_add(skein_lib::len_u64(row.observations.len()))?
        .checked_add(failures_bytes(&row.record.failing)?)?
        .checked_add(
            skein_lib::len_u64(row.record.flakes.len()).checked_mul(u64::try_from(size_of::<p::Flake>()).ok()?)?,
        )?;
    for flake in &row.record.flakes {
        held = held.checked_add(skein_lib::len_u64(flake.test.len()))?;
    }
    held = held.checked_add(
        skein_lib::len_u64(row.pending.len()).checked_mul(u64::try_from(size_of::<TierLanding>()).ok()?)?,
    )?;
    held = held.checked_add(
        skein_lib::len_u64(row.goals.len()).checked_mul(u64::try_from(size_of::<crate::TierGoal>()).ok()?)?,
    )?;
    for goal in &row.goals {
        held = held.checked_add(skein_lib::len_u64(goal.branch.len()))?.checked_add(failures_bytes(&goal.failing)?)?;
    }
    if let Some(flight) = &row.flight {
        if let Some(report) = &flight.report {
            held = held.checked_add(skein_lib::len_u64(report.url.len()))?;
        }
        held = held.checked_add(
            skein_lib::len_u64(flight.pending.len()).checked_mul(u64::try_from(size_of::<api::JobAttempt>()).ok()?)?,
        )?;
    }
    if let Some(result) = &row.result {
        held = held
            .checked_add(skein_lib::len_u64(result.run.url.len()))?
            .checked_add(skein_lib::len_u64(result.logs.len()))?
            .checked_add(crate::landing::receipt_job_bytes(result)?)?;
        if let Some(failures) = &result.failures {
            held = held.checked_add(failures_bytes(failures)?)?;
        }
    }
    if let Some(posting) = &row.posting {
        held = held
            .checked_add(u64::try_from(size_of::<crate::TierPosting>()).ok()?)?
            .checked_add(skein_lib::len_u64(posting.link.len()))?
            .checked_add(skein_lib::len_u64(posting.description.len()))?
            .checked_add(
                skein_lib::len_u64(posting.landings.len())
                    .checked_mul(u64::try_from(size_of::<crate::TierPostedLanding>()).ok()?)?,
            )?
            .checked_add(
                skein_lib::len_u64(posting.comments.len())
                    .checked_mul(u64::try_from(size_of::<crate::TierComment>()).ok()?)?,
            )?;
        for landing in &posting.landings {
            held = held.checked_add(blame_bytes(&landing.blame)?)?;
        }
    }
    if let Some(hold) = &row.hold {
        held = held.checked_add(skein_lib::len_u64(hold.reason.len()))?;
    }
    if let Some(run) = &row.record.run {
        held = held.checked_add(
            skein_lib::len_u64(run.landings.len()).checked_mul(u64::try_from(size_of::<p::Landing>()).ok()?)?,
        )?;
        match &run.part {
            p::Part::Whole => {}
            p::Part::Rerun { failed } => held = held.checked_add(tests_bytes(failed)?)?,
            p::Part::Probe { tests, waiting, finding, .. } => {
                held = held.checked_add(
                    skein_lib::len_u64(waiting.len()).checked_mul(u64::try_from(size_of::<p::Group>()).ok()?)?,
                )?;
                for group in waiting {
                    held = held.checked_add(
                        skein_lib::len_u64(group.tests.len()).checked_mul(u64::try_from(size_of::<u16>()).ok()?)?,
                    )?;
                }
                match tests {
                    p::Selection::All => {}
                    p::Selection::Tests(tests) => held = held.checked_add(tests_bytes(tests)?)?,
                }
                held = held
                    .checked_add(failures_bytes(&finding.failing)?)?
                    .checked_add(tests_bytes(&finding.flakes)?)?
                    .checked_add(
                        skein_lib::len_u64(finding.blamed.len())
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

/// A person's tier control (temper/docs/design/domain/after-landing.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierActRequest {
    pub project: u32,
    pub repository: Option<crate::DraftRepository>,
    pub tier: Box<[u8]>,
    pub act: TierAct,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TierAct {
    Hold { reason: Box<[u8]> },
    Release,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TierRefusal {
    NoRepository,
    NoTier,
    Held { party: u64, reason: Box<[u8]> },
    NotHeld,
    Words,
}
