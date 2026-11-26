//! Open engine problems' exclusions (`temper/docs/design/testing/engine-checks.md`, 5.2).

use skein_world::faults::Kind;
use skein_world::sweep::{Exclusion, Sweep};

pub const A_PATH_DELETED_ON_THE_BASE: Kind = Kind { family: "stale", name: "a path deleted on the base" };

/// Adds each engine exclusion whose fault `configured` holds, returning the first refusal.
pub fn exclude(sweep: &mut Sweep, configured: &[Kind]) -> Result<(), String> {
    if configured.contains(&temper_faults::bounds::BRANCH_HOLDS) {
        sweep.exclude(Exclusion::new(
            "a_prepare_is_admitted_when_branch_holds_are_released",
            "failure worlds: repaired as a red",
            temper_faults::bounds::BRANCH_HOLDS,
            &["the holds and pending prepares, including the new change or train, exceed capacity"],
        ))?;
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
    Ok(())
}
