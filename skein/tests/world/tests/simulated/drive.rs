use skein_world_tests::hosted::Script;

#[test]
fn worlds_drain_once_per_iteration_in_the_simulator() {
    let mut simulated = skein_world::World::new(
        7,
        skein_sim::Config::calm(),
        skein_world_tests::hosted::Judge,
        skein_world::Memory::Unchecked,
    );
    simulated.spawn(|| Script::new(&[]));
    let outcome = simulated.run();
    assert_eq!(outcome.procs[0].iterations, outcome.procs[0].drains);
    assert!(outcome.procs[0].drains > 0);
}
