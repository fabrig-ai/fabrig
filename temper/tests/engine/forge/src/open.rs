//! Open engine problems' exclusions (`temper/docs/design/testing/engine-checks.md`, 5.2).

use skein_world::faults::Kind;
use skein_world::sweep::{Exclusion, Sweep};

pub const A_WRITE_REFUSED: Kind = Kind { family: "forge", name: "a write refused" };
pub const A_REPAIR_ENDS_WITH_NO_CHANGE: Kind = Kind { family: "runs", name: "a repair ends with no change" };
pub const A_PATH_DELETED_ON_THE_BASE: Kind = Kind { family: "stale", name: "a path deleted on the base" };
pub const A_REPAIR_FINDS_ITS_TESTS_PASSING: Kind = Kind { family: "runs", name: "a repair finds its tests passing" };

/// Adds each engine exclusion whose fault `configured` holds, returning the first refusal.
pub fn exclude(sweep: &mut Sweep, configured: &[Kind]) -> Result<(), String> {
    if configured.contains(&temper_faults::bounds::FORGE_WRITE_ENTRIES) {
        sweep.exclude(Exclusion::new(
            "a_change_waiting_for_room_has_a_bound",
            "bounded waits",
            temper_faults::bounds::FORGE_WRITE_ENTRIES,
            &["a change waiting for room past its stall"],
        ))?;
    }
    if configured.contains(&A_WRITE_REFUSED) {
        sweep.exclude(Exclusion::new(
            "a_refused_release_is_accounted",
            "landing train",
            A_WRITE_REFUSED,
            &["a task's release under way"],
        ))?;
    }
    if configured.contains(&A_REPAIR_ENDS_WITH_NO_CHANGE) {
        sweep.exclude(Exclusion::new(
            "a_no_change_landing_repair_regates_once",
            "a change's reds",
            A_REPAIR_ENDS_WITH_NO_CHANGE,
            &["a landing's gate red"],
        ))?;
    }
    if configured.contains(&temper_blame::STARVED) {
        sweep.exclude(Exclusion::new(
            "a_starved_gate_is_the_runners",
            "a change's reds",
            temper_blame::STARVED,
            &["a landing's gate"],
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
    if configured.contains(&A_REPAIR_FINDS_ITS_TESTS_PASSING) {
        sweep.exclude(Exclusion::new(
            "a_repair_whose_tests_pass_regates_by_itself",
            "a change's reds",
            A_REPAIR_FINDS_ITS_TESTS_PASSING,
            &["a landing's gate red"],
        ))?;
    }
    for kind in [temper_faults::people::ANSWERS_LATE, temper_faults::people::ANSWERS_TWICE] {
        if configured.contains(&kind) {
            sweep.exclude(Exclusion::new(
                "a_held_answer_tells_its_requester",
                "bounded waits",
                kind,
                &["a dependency holds the answer"],
            ))?;
        }
    }
    if configured.contains(&temper_faults::parks::A_CHANGE_WAITS_ON_ANOTHER_GOAL) {
        sweep.exclude(Exclusion::new(
            "a_change_waits_on_another_goal",
            "bounded waits",
            temper_faults::parks::A_CHANGE_WAITS_ON_ANOTHER_GOAL,
            &["its requester parked"],
        ))?;
    }
    if configured.contains(&temper_faults::restarts::ENGINE_CRASH) {
        sweep.exclude(Exclusion::new(
            "a_stored_charter_is_checked_at_start",
            "bounded waits",
            temper_faults::restarts::ENGINE_CRASH,
            &["a stored charter past its room"],
        ))?;
    }
    Ok(())
}
