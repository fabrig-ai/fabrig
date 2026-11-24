//! The composed comparison's control (temper/docs/design/testing/workloads.md, 9).
use temper_system_world::composed::{World, workload::GOAL_BOUND};
use temper_workload::outcome::End;

#[test]
fn a_changed_composed_outcome_names_its_goal() {
    // Run until every drawn goal is named and its outcome can be read. This
    // control checks the comparison; the fuzzy sweep checks the completed run
    // (temper/docs/design/testing/workloads.md, 9).
    let mut world = World::composed_workload(0);
    for _ in 0..GOAL_BOUND.steps {
        if world.workload().goals.iter().all(|goal| world.workload_names().task(&goal.name).is_some()) {
            break;
        }
        world.tick();
    }
    let before = world.workload_outcome();
    let mut restarted = before.clone();
    restarted.goals.get_mut("g0").expect("drawn goal").end = End::Failed;
    let finding = before.compare(&restarted, &[]).expect_err("a changed outcome is found");
    assert!(finding.contains("compare: g0: end Waiting without restarts, Failed with them"), "{finding}");
}
