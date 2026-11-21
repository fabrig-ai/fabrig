//! Step tests of configured faults and pre-injection exclusions
//! (`skein/docs/design/testing/sweeps.md`, 4–6), with hand-filled ledgers.

use skein_lib::Time;

use crate::faults::{FaultLedger, Kind, Struck};
use crate::sweep::{Exclusion, Sweep};

const CANCEL_LANDING: Kind = Kind { family: "landing", name: "cancel" };
const OTHER: Kind = Kind { family: "forge", name: "busy" };
const SCENARIO: &str = "cancelled_landing_frees_its_write_entries";
const DISPATCH: &str = "a gate's dispatch in flight";
const AT: Time = Time::from_nanos(7);

fn configured() -> Sweep {
    let mut sweep = Sweep::new("landing");
    sweep.configure(CANCEL_LANDING);
    sweep
}

fn assert_refused(sweep: &mut Sweep, exclusion: Exclusion, why: &str) {
    let expected = format!("sweep landing: exclusion {:?}: {why}", exclusion.scenario);
    assert_eq!(sweep.exclude(exclusion), Err(expected));
    assert!(sweep.leaves_out(CANCEL_LANDING, &[DISPATCH]).is_none());
}

#[test]
fn configured_fault_must_fall() {
    let mut sweep = configured();
    sweep.configure(OTHER);
    for _ in 0_u64..3 {
        let mut ledger = FaultLedger::new();
        ledger.inject(OTHER, AT, Struck::default());
        sweep.ran(&ledger);
    }
    assert_eq!(sweep.check_fell(), Err("sweep landing: landing/cancel never fell in 3 seeds".into()));
}

#[test]
fn missing_faults_follow_kind_order() {
    let mut sweep = configured();
    sweep.configure(OTHER);
    sweep.configure(CANCEL_LANDING);
    assert_eq!(
        sweep.check_fell(),
        Err("sweep landing: forge/busy never fell in 0 seeds\nsweep landing: landing/cancel never fell in 0 seeds"
            .into())
    );
}

#[test]
fn configured_faults_can_fall_in_different_seeds() {
    let mut sweep = configured();
    sweep.configure(OTHER);
    for kind in [CANCEL_LANDING, OTHER, Kind { family: "extra", name: "unconfigured" }] {
        let mut ledger = FaultLedger::new();
        ledger.inject(kind, AT, Struck::default());
        sweep.ran(&ledger);
    }
    assert_eq!(sweep.check_fell(), Ok(()));
}

#[test]
fn exclusion_requires_scenario_name() {
    let mut sweep = configured();
    for scenario in ["", "Cancel landing", "1landing", "_landing", "cancél", "cancel-landing"] {
        assert_refused(
            &mut sweep,
            Exclusion::new(scenario, "landing train", CANCEL_LANDING, &[DISPATCH]),
            "names no ignored scenario",
        );
    }
    assert_eq!(sweep.exclude(Exclusion::new("cancel_landing_2", "landing train", CANCEL_LANDING, &[DISPATCH])), Ok(()));
}

#[test]
fn exclusion_requires_fixing_work() {
    assert_refused(
        &mut configured(),
        Exclusion::new(SCENARIO, "", CANCEL_LANDING, &[DISPATCH]),
        "names no work that fixes it",
    );
}

#[test]
fn exclusion_requires_conditions() {
    assert_refused(
        &mut configured(),
        Exclusion::new(SCENARIO, "landing train", CANCEL_LANDING, &[]),
        "leaves out the whole fault, not a combination",
    );
}

#[test]
fn exclusion_requires_configured_fault() {
    assert_refused(
        &mut configured(),
        Exclusion::new(SCENARIO, "landing train", OTHER, &[DISPATCH]),
        "fault is not configured",
    );
}

#[test]
fn duplicate_exclusion_is_refused() {
    let mut sweep = configured();
    assert_eq!(sweep.exclude(Exclusion::new(SCENARIO, "landing train", CANCEL_LANDING, &[DISPATCH, "busy"])), Ok(()));
    for when in [&[DISPATCH, "busy"][..], &["busy", DISPATCH][..]] {
        let expected = format!("sweep landing: exclusion {SCENARIO:?}: duplicates the scenario, fault and conditions");
        assert_eq!(sweep.exclude(Exclusion::new(SCENARIO, "other work", CANCEL_LANDING, when)), Err(expected));
    }
    assert_eq!(
        sweep.exclude(Exclusion::new("other_scenario", "landing train", CANCEL_LANDING, &[DISPATCH, "busy"])),
        Ok(())
    );
    assert_eq!(sweep.exclude(Exclusion::new(SCENARIO, "landing train", CANCEL_LANDING, &[DISPATCH])), Ok(()));
    sweep.configure(OTHER);
    assert_eq!(sweep.exclude(Exclusion::new(SCENARIO, "landing train", OTHER, &[DISPATCH, "busy"])), Ok(()));
}

#[test]
fn exclusion_requires_its_fault_and_every_condition() {
    let mut sweep = configured();
    assert_eq!(
        sweep.exclude(Exclusion::new(
            "cancelled_landing_frees_its_write_entries",
            "landing train",
            CANCEL_LANDING,
            &["a gate's dispatch in flight"],
        )),
        Ok(())
    );
    let exclusion = sweep.leaves_out(CANCEL_LANDING, &[DISPATCH]).expect("matching combination is excluded");
    assert_eq!(exclusion.scenario, SCENARIO);
    assert_eq!(exclusion.fixed_by, "landing train");
    assert_eq!(exclusion.fault, CANCEL_LANDING);
    assert_eq!(exclusion.when, &[DISPATCH]);
    assert!(sweep.leaves_out(CANCEL_LANDING, &[]).is_none());
    assert!(sweep.leaves_out(OTHER, &[DISPATCH]).is_none());

    let mut both = configured();
    assert_eq!(both.exclude(Exclusion::new(SCENARIO, "landing train", CANCEL_LANDING, &[DISPATCH, "busy"])), Ok(()));
    assert!(both.leaves_out(CANCEL_LANDING, &[DISPATCH]).is_none());
    assert!(both.leaves_out(CANCEL_LANDING, &["busy"]).is_none());
    assert!(both.leaves_out(CANCEL_LANDING, &["busy", DISPATCH, "extra"]).is_some());
}

#[test]
fn left_out_draws_are_retained_and_counted() {
    let mut sweep = configured();
    let mut first = FaultLedger::new();
    first.leave_out(SCENARIO, CANCEL_LANDING, AT);
    assert_eq!(first.left_out_draws(), &[(SCENARIO, CANCEL_LANDING, AT)]);
    assert!(first.entries().is_empty());
    assert!(first.fell().is_empty());
    sweep.ran(&first);
    assert_eq!(sweep.left_out(SCENARIO), 1);

    let mut second = FaultLedger::new();
    second.leave_out(SCENARIO, CANCEL_LANDING, AT);
    second.leave_out(SCENARIO, OTHER, Time::from_nanos(9));
    second.leave_out("other_scenario", OTHER, AT);
    assert_eq!(
        second.left_out_draws(),
        &[(SCENARIO, CANCEL_LANDING, AT), (SCENARIO, OTHER, Time::from_nanos(9)), ("other_scenario", OTHER, AT)]
    );
    sweep.ran(&second);
    assert_eq!(sweep.left_out(SCENARIO), 3);
    assert_eq!(sweep.left_out("other_scenario"), 1);
    assert_eq!(sweep.left_out("absent"), 0);
    assert_eq!(sweep.check_fell(), Err("sweep landing: landing/cancel never fell in 2 seeds".into()));
}
