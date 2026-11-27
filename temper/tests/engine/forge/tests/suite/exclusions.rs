//! Engine exclusions leave out only configured faults in their named combinations.
use skein_world::faults::Kind;
use skein_world::sweep::Sweep;
use temper_engine_forge_world::open::{self, *};

// Each row pins a distinct open problem, its fault vocabulary and fixing work.
const CASES: &[(Kind, &str, &str, &str, &str)] = &[
    (
        temper_faults::bounds::BRANCH_HOLDS,
        "the engine's own bounds/branch holds",
        "a_prepare_is_admitted_when_branch_holds_are_released",
        "failure worlds: repaired as a red",
        "the holds and pending prepares, including the new change or train, exceed capacity",
    ),
    (
        A_PATH_DELETED_ON_THE_BASE,
        "stale/a path deleted on the base",
        "a_resolution_keeping_a_deleted_path_is_refused",
        "a change's reds",
        "a resolution under way",
    ),
    (
        temper_faults::forge::CLOSE_REFUSED,
        "refusing/close_refused",
        "a_failed_capability_refresh_reads_again",
        "failure worlds: repaired as a red",
        "the refused close occupies a slot in a full forge client call table",
    ),
    (
        temper_faults::people::CANCELS_A_GOAL,
        "people and parties/cancelling a goal",
        "a_goal_cancelled_while_its_branch_is_being_made_ends_cancelled",
        "failure worlds: repaired as a red",
        "a goal branch making in flight",
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
    let kind = temper_faults::bounds::FORGE_WRITE_ENTRIES;
    sweep.configure(kind);
    open::exclude(&mut sweep, &[kind]).expect("room bound's fault is configured");
    assert!(sweep.leaves_out(kind, &["a change waiting for room past its stall"]).is_none());
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
fn a_crash_with_an_unfit_stored_charter_is_not_excluded() {
    let kind = temper_faults::restarts::ENGINE_CRASH;
    let mut sweep = Sweep::new("engine crash with an unfit stored charter");
    sweep.configure(kind);
    open::exclude(&mut sweep, &[kind]).expect("engine crash is configured");
    assert!(sweep.leaves_out(kind, &["a stored charter past its room"]).is_none());
}

#[test]
fn engine_exclusions_return_the_first_refusal_without_adding_later_rows() {
    let mut sweep = Sweep::new("missing configured fault");
    sweep.configure(A_PATH_DELETED_ON_THE_BASE);
    let refusal = open::exclude(&mut sweep, &[temper_faults::bounds::BRANCH_HOLDS, A_PATH_DELETED_ON_THE_BASE])
        .expect_err("the first fault is absent from the sweep");
    assert_eq!(
        refusal,
        "sweep missing configured fault: exclusion \"a_prepare_is_admitted_when_branch_holds_are_released\": fault is not configured"
    );
    assert!(sweep.leaves_out(A_PATH_DELETED_ON_THE_BASE, &["a resolution under way"]).is_none());
}
