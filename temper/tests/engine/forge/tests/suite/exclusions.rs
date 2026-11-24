//! Engine exclusions leave out only configured faults in their named combinations.
use skein_world::faults::Kind;
use skein_world::sweep::Sweep;
use temper_engine_forge_world::open::{self, *};

// Each row pins a distinct open problem, its fault vocabulary and fixing work.
const CASES: &[(Kind, &str, &str, &str, &str)] = &[
    (
        temper_faults::bounds::FORGE_WRITE_ENTRIES,
        "the engine's own bounds/forge write entries",
        "a_change_waiting_for_room_has_a_bound",
        "bounded waits",
        "a change waiting for room past its stall",
    ),
    (
        temper_faults::bounds::BRANCH_HOLDS,
        "the engine's own bounds/branch holds",
        "a_prepare_is_admitted_when_branch_holds_are_released",
        "failure worlds: repaired as a red",
        "the holds and pending prepares, including the new change or train, exceed capacity",
    ),
    (
        A_WRITE_REFUSED,
        "forge/a write refused",
        "a_refused_release_is_accounted",
        "landing train",
        "a task's release under way",
    ),
    (
        A_REPAIR_ENDS_WITH_NO_CHANGE,
        "runs/a repair ends with no change",
        "a_no_change_landing_repair_regates_once",
        "a change's reds",
        "a landing's gate red",
    ),
    (temper_blame::STARVED, "ci/starved", "a_starved_gate_is_the_runners", "a change's reds", "a landing's gate"),
    (
        A_PATH_DELETED_ON_THE_BASE,
        "stale/a path deleted on the base",
        "a_resolution_keeping_a_deleted_path_is_refused",
        "a change's reds",
        "a resolution under way",
    ),
    (
        A_REPAIR_FINDS_ITS_TESTS_PASSING,
        "runs/a repair finds its tests passing",
        "a_repair_whose_tests_pass_regates_by_itself",
        "a change's reds",
        "a landing's gate red",
    ),
    (
        temper_faults::parks::A_CHANGE_WAITS_ON_ANOTHER_GOAL,
        "runs that park, resume and move/a change waits on another goal",
        "a_change_waits_on_another_goal",
        "bounded waits",
        "its requester parked",
    ),
    (
        temper_faults::people::CANCELS_A_GOAL,
        "people and parties/cancelling a goal",
        "a_goal_cancelled_while_its_branch_is_being_made_ends_cancelled",
        "failure worlds: repaired as a red",
        "a goal branch making in flight",
    ),
    (
        temper_faults::restarts::ENGINE_CRASH,
        "restarts/engine crash",
        "a_stored_charter_is_checked_at_start",
        "bounded waits",
        "a stored charter past its room",
    ),
];

#[test]
fn engine_exclusions_require_the_configured_fault_and_its_conditions() {
    let configured: Vec<_> = CASES.iter().map(|row| row.0).collect();
    let mut sweep = Sweep::new("all open engine faults");
    for kind in &configured {
        sweep.configure(*kind);
    }
    open::exclude(&mut sweep, &configured).expect("all faults configured");
    let mut empty = Sweep::new("no open engine faults");
    open::exclude(&mut empty, &[]).expect("no exclusions needed");

    for &(kind, vocabulary, scenario, work, condition) in CASES {
        assert_eq!(format!("{}/{}", kind.family, kind.name), vocabulary);
        let exclusion = sweep.leaves_out(kind, &[condition]).expect(scenario);
        assert_eq!(exclusion.scenario, scenario);
        assert_eq!(exclusion.fixed_by, work);
        assert_eq!(exclusion.fault, kind);
        assert_eq!(exclusion.when, &[condition]);
        assert_eq!(sweep.leaves_out(kind, &["unrelated state", condition]).expect(scenario).scenario, scenario);
        assert!(sweep.leaves_out(kind, &[]).is_none(), "{scenario}: condition missing");
        assert!(sweep.leaves_out(kind, &["unrelated state"]).is_none(), "{scenario}: wrong condition");
        assert!(empty.leaves_out(kind, &[condition]).is_none(), "{scenario}: no configured faults");

        let mut single = Sweep::new("one open engine fault");
        single.configure(kind);
        open::exclude(&mut single, &[kind]).expect("only the selected fault needs configuring");
        assert_eq!(single.leaves_out(kind, &[condition]).expect(scenario).scenario, scenario);
        for &(other, _, _, _, other_condition) in CASES {
            if other != kind {
                assert!(single.leaves_out(other, &[other_condition]).is_none());
            }
        }
    }
}

#[test]
fn engine_exclusions_return_the_first_refusal_without_adding_later_rows() {
    let mut sweep = Sweep::new("missing configured fault");
    sweep.configure(A_WRITE_REFUSED);
    let refusal = open::exclude(&mut sweep, &[temper_faults::bounds::FORGE_WRITE_ENTRIES, A_WRITE_REFUSED])
        .expect_err("the first fault is absent from the sweep");
    assert_eq!(
        refusal,
        "sweep missing configured fault: exclusion \"a_change_waiting_for_room_has_a_bound\": fault is not configured"
    );
    assert!(sweep.leaves_out(A_WRITE_REFUSED, &["a task's release under way"]).is_none());
}
