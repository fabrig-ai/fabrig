//! Joined closes and aborts across moves under small intakes (runtime/io/README.md, 5).
use skein_io_world::joined::random;
use skein_sim::Config;

#[test]
fn joined_closes_and_aborts_hold_over_many_seeds() {
    for seed in 0..150 {
        let outcome = random(seed, Config::chaos()).run();
        assert!(outcome.trace.is_empty(), "seed {seed}: joined pairs never reach the simulator");
    }
}
