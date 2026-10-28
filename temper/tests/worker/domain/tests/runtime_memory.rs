//! All root entry points, including owned conflict paths, retained opaque
//! bodies, repeated hello copies and retained answers, measured serially.
use skein_world::domain::heap;
use temper_worker_domain_world::runtime::{Close, Settings, World};

#[global_allocator]
static HEAP: heap::Counting = heap::Counting;

#[test]
fn runtime_heap_stays_bounded_at_one_and_three_retained_turns() {
    for turns in [1, 3] {
        for merging in [false, true] {
            let mut world =
                World::new(Settings { seed: 91, turns, byte_slots: turns, merging, close: Close::Committed });
            world.fill_turns();
            world.run();
            assert_eq!(world.stats().peak_retained, turns);
            assert!(world.stats().peak_heap > 0, "the counting allocator observed domain memory");
        }
    }
}
