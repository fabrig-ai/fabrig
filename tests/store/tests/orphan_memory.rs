//! Count only this binary: the orphan queue costs what it holds, not
//! the budget's most orphan tasks (skein's draft/dynamic-memory.md, 2).
use jig_store_world::World;
use skein_heap::{Counting, Meter};

#[global_allocator]
static HEAP: Counting = Counting;

// Measured 173,330 bytes, including the world's simulator and file drivers;
// about twice that peak, far below the old 481,877,984-byte orphan estimate
// (32 bytes * (268,435,456 / 18 + (3 + 1 + 3) * 20,801) tasks).
const OPEN_HEAP: u64 = 352 * 1024;

#[test]
fn opening_a_store_reserves_nothing_for_orphans() {
    // jig's small profile: its orphan estimate alone is hundreds of MiB.
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).store;
    let meter = Meter::new();
    meter.start();
    let mut world = World::unopened_observed(18, limits, None);
    let opened = world.open();
    let measured = meter.end();
    let peak = meter.check(measured, OPEN_HEAP, &"small store construction through open");
    assert_eq!(opened, Some(0), "the small profile opens an empty store");
    eprintln!("small store construction through open: {peak} bytes (ceiling {OPEN_HEAP})");
}
