//! One complete composed draw; restart comparison grows here (workloads.md, 5 and 7).
use temper_system_world::composed::World;

#[test]
fn the_composed_workload_ends_within_its_bounds() {
    for seed in [17, 29] {
        eprintln!("seed {seed}");
        let mut world = World::composed_workload(seed);
        eprintln!("workload {:?}", world.workload());
        world.run_workload();
        world.assert_workload();
    }
}
