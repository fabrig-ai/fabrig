//! The composed comparison's control (temper/docs/design/testing/workloads.md, 9).
use temper_system_world::composed::World;
use temper_workload::outcome::End;

#[test]
fn a_changed_composed_outcome_names_its_goal() {
    let mut world = World::composed_workload(0);
    world.run_workload();
    world.assert_workload();
    let before = world.workload_outcome();
    let mut restarted = before.clone();
    restarted.goals.get_mut("g0").expect("drawn goal").end = End::Failed;
    let finding = before.compare(&restarted, &[]).expect_err("a changed outcome is found");
    assert!(finding.contains("compare: g0: end Done without restarts, Failed with them"), "{finding}");
}
