//! Open engine problems' exclusions (`temper/docs/design/testing/engine-checks.md`, 5.2).

use skein_world::faults::Kind;
use skein_world::sweep::{Exclusion, Sweep};
use temper_engine_domain_forge::LandingRow;
use temper_engine_domain_forge_landing::Combine;
use temper_faults::forge::MERGE_LATE;

pub const A_PATH_DELETED_ON_THE_BASE: Kind = Kind { family: "stale", name: "a path deleted on the base" };

pub const BRANCH_HOLDS_CAPACITY: Exclusion = Exclusion::new(
    "a_prepare_is_admitted_when_branch_holds_are_released",
    "failure worlds: repaired as a red",
    temper_faults::bounds::BRANCH_HOLDS,
    &["the holds and pending prepares, including the new change or train, exceed capacity"],
);

/// A late manual record of an already-landed squash can strand its train.
pub const MERGE_LATE_RECORD: Exclusion = Exclusion::new(
    "a_late_manual_merge_record_finishes_the_landed_batch",
    "landing train",
    MERGE_LATE,
    &["recording a squashed member already landed on its base"],
);

/// Whether a manual merge call is recording a squashed member landed on its base.
#[must_use]
pub fn recording_landed_squash(row: &LandingRow, number: u64, commit: skein_forge::api::Commit) -> bool {
    row.landing.members.iter().any(|member| {
        member.pull == number
            && member.combined == Some(Combine::Squash)
            && member.landed == Some(commit)
            && !member.recorded
    })
}

/// Adds each engine exclusion whose fault `configured` holds, returning the first refusal.
pub fn exclude(sweep: &mut Sweep, configured: &[Kind]) -> Result<(), String> {
    if configured.contains(&BRANCH_HOLDS_CAPACITY.fault) {
        sweep.exclude(BRANCH_HOLDS_CAPACITY)?;
    }
    if configured.contains(&A_PATH_DELETED_ON_THE_BASE) {
        sweep.exclude(Exclusion::new(
            "a_resolution_keeping_a_deleted_path_is_refused",
            "a change's reds",
            A_PATH_DELETED_ON_THE_BASE,
            &["a resolution under way"],
        ))?;
    }
    if configured.contains(&temper_faults::people::CANCELS_A_GOAL) {
        sweep.exclude(Exclusion::new(
            "a_goal_cancelled_while_its_branch_is_being_made_ends_cancelled",
            "failure worlds: repaired as a red",
            temper_faults::people::CANCELS_A_GOAL,
            &["a goal branch making in flight"],
        ))?;
    }
    if configured.contains(&MERGE_LATE) {
        sweep.exclude(MERGE_LATE_RECORD)?;
    }
    Ok(())
}
