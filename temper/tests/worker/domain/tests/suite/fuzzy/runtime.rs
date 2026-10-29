//! Small retained-turn windows and body budgets; deterministic seeds.
use temper_worker_domain_world::runtime::{Close, Settings, World};

// Measured peak: 34,264 bytes; 25% headroom, rounded up to two significant digits.
const RUNTIME_HEAP_CEILING: u64 = 43_000;

#[test]
fn runtime_random_turn_sizes_and_tiny_windows_settle() {
    for seed in 0..48 {
        let turns = u32::try_from(seed % 3 + 1).expect("tiny window");
        let byte_slots = u32::try_from(seed / 3 % 3 + 1).expect("tiny budget");
        let close = match seed % 5 {
            0 => Close::Abandoned,
            1 => Close::Unacknowledged,
            2 => Close::Reopened,
            _ => Close::Committed,
        };
        let settings = Settings { seed, turns, byte_slots, merging: seed % 2 == 0, close };
        let result = std::panic::catch_unwind(|| {
            let mut world = World::new(settings, RUNTIME_HEAP_CEILING);
            world.run();
            // Stopping also takes the independent agent window already sent.
            assert!(world.stats().peak_retained <= turns * 2);
        });
        assert!(result.is_ok(), "runtime seed {seed}, settings {settings:?}");
    }
}
