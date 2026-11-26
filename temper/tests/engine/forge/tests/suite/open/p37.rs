//! Empty gate reports do not name a failure to repair (landing-proof.md, 2.4).
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World, changes,
    landings::{self, CHANGE, LANDING},
};

#[test]
fn a_starved_gate_is_the_runners() {
    const SEED: u64 = 31;
    check_starved_gate(&mut super::reported_gate(SEED, &[], false));
}

pub(crate) fn check_starved_gate(world: &mut World) {
    // The fake reports no failing ids. Its Lines format has no unobserved-tests field.
    world.gate_policy(landing::Project { gate_retries: 1, ..landing::Project::DEFAULT });
    world.take_calls();
    // Eighty root turns exceed the two-second workflow and its one permitted retry.
    for entry in 100..180 {
        if matches!(
            landings::step_landing(world, LANDING, entry),
            landing::Decision::Hold(_) | landing::Decision::Leave { .. } | landing::Decision::Finish
        ) {
            break;
        }
    }
    let decision = changes::change_step_task(world, CHANGE, 300, change::Status::Passed, Box::new([]), false, false);
    let calls = world.take_calls();
    assert!(
        calls.iter().any(|op| matches!(op, api::Op::Read(api::Read::Artifact { .. }))),
        "the failed workflow's report was read"
    );
    assert!(
        !world.stored().iter().any(|(_, row)| matches!(row, top::Stored::Receipt { receipt, .. } if !receipt.passed)),
        "no receipt for an empty report"
    );
    let retries = calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Rerun { .. }))).count();
    assert!(
        !matches!(decision, change::Decision::Delegate(change::Delegate::Repair(_))),
        "a report with no failed test never asks repair: {decision:?}; retries={retries}"
    );
    assert_eq!(retries, 1, "retry the failed job once at the same head");
    assert_eq!(world.change(CHANGE).change.repairs, 0);
    assert!(
        matches!(world.landing(LANDING).landing.phase, landing::Phase::Held { why: landing::Hold::Faults { .. }, .. }),
        "{:?}",
        world.landing(LANDING)
    );
    let words = world.landing_hold_words(LANDING).expect("held gate words");
    super::words_name(&words, &["gate gate", "check broke", "report names no failed test", "change wasn't judged"]);
}
