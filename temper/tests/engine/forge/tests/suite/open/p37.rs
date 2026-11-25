//! Empty gate reports belong to the runner, not repair (landing-proof.md, 2.4).
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    changes,
    landings::{self, CHANGE, LANDING},
};

#[test]
#[ignore = "a change's reds: P37, an empty red asks repair"]
fn a_starved_gate_is_the_runners() {
    // The fake reports no failing ids. Its Lines format has no unobserved-tests field.
    let mut world = super::reported_gate(&[]);
    world.gate_policy(landing::Project { gate_retries: 1, ..landing::Project::DEFAULT });
    world.take_calls();
    // Eighty root turns exceed the two-second workflow and its one permitted retry.
    for entry in 100..180 {
        if matches!(
            landings::step_landing(&mut world, LANDING, entry),
            landing::Decision::Hold(_) | landing::Decision::Leave { .. } | landing::Decision::Finish
        ) {
            break;
        }
    }
    let decision =
        changes::change_step_task(&mut world, CHANGE, 300, change::Status::Passed, Box::new([]), false, false);
    let calls = world.take_calls();
    assert!(
        calls.iter().any(|op| matches!(op, api::Op::Read(api::Read::Artifact { .. }))),
        "the failed workflow's report was read"
    );
    if let Some(report) = world.stored().iter().find_map(|(_, row)| match row {
        top::Stored::Receipt { receipt, .. } if !receipt.passed => Some(&receipt.report),
        _ => None,
    }) {
        assert_eq!(report, &top::ReceiptReport::Tests(Box::new([])), "no failing test in the report");
    }
    let retries = calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Rerun { .. }))).count();
    assert!(
        !matches!(decision, change::Decision::Delegate(change::Delegate::Repair(_))),
        "a red with no failed test is the runner's, not {decision:?}; retries={retries}"
    );
    assert_eq!(retries, 1, "retry the failed job once at the same head");
    assert_eq!(world.change(CHANGE).change.repairs, 0);
    assert!(
        matches!(world.landing(LANDING).landing.phase, landing::Phase::Held { why: landing::Hold::Faults { .. }, .. }),
        "{:?}",
        world.landing(LANDING)
    );
}
