//! The composed driver's opening referee, including same-instant steps.

use super::{World, report_script};
use temper_engine_domain_world::ends::OPENING;

#[test]
fn a_composed_restart_opens_within_the_shared_bound() {
    let mut world = World::absence_chat(941, report_script(false), b"@system", 1);
    world.until_model_held();
    world.cut_link();
    world.stop_engine();
    world.reopen_engine();
    assert!(!world.opened, "the old root's opening is not the new one's");
    let restarted = world.iteration;
    world.join_link();
    assert!(world.opened && world.root.ready());
    assert!(world.iteration - restarted < OPENING.steps);
    world.ends.close(world.env.now, &world.store).expect("the restarted root opened");
}

#[test]
fn an_unopened_composed_root_is_found_without_moving_time() {
    let mut world = World::chat(941, report_script(false));
    // Deliberately withhold the opening input, not a refusal exempted by no_opening.
    world.root_events.clear();
    for _ in 1..OPENING.steps {
        world.tick_at_now().expect("still within the opening bound");
    }
    let finding = world.tick_at_now().expect_err("same-instant work still exhausts OPENING");
    assert!(finding.contains("engine opening: did not open within 1 minutes and 1000 steps"), "{finding}");
    assert_eq!(world.env.now, super::Time::ZERO);
    assert_eq!(world.try_run(), Err(finding), "run retains the first opening finding");
}
