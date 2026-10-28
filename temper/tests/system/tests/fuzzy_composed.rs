//! Seeded process completion times and delayed commits cross both roots.
use temper_system_world::composed::{World, report_script};

#[test]
fn seeded_system_runs_replay_turn_credit_calls_and_prepared_mounts() {
    for seed in 0..24 {
        let run = || {
            let mut world = if seed % 3 == 0 {
                World::with_repository(seed, report_script(true))
            } else {
                World::chat(seed, report_script(seed % 2 == 0))
            };
            world.run();
            assert_eq!(world.observed.turns, world.observed.turn_acks);
            assert_eq!(world.observed.relays, world.observed.replies);
            world
        };
        let first = run();
        let second = run();
        assert_eq!(first.observed, second.observed, "seed {seed}");
        assert_eq!(first.store.rows, second.store.rows, "seed {seed}");
        assert_eq!(first.agent().turns(), second.agent().turns(), "seed {seed}");
    }
}
