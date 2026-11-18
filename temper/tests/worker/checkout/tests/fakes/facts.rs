//! The facts-on-the-side property, shared by the named scenarios and the sweep.

use temper_worker_checkout_world::{Settings, Told, World, submit_noisily};
use temper_worker_domain_checkout::Limits;

// The sweep's generous fact sink; each run asserts that this keeps every fact.
const FACT_CAPACITY: u32 = 4096;
const ITERATIONS: u32 = 100_000;

pub(super) fn assert_unchanged(settings: Settings) -> Told {
    let seed = settings.seed;
    let run = |facts| {
        let settings = Settings { checkout: Limits { facts, ..settings.checkout }, ..settings };
        let mut world = World::new(settings);
        submit_noisily(&mut world, &settings, seed);
        world.run(ITERATIONS);
        (world.trace().to_vec(), world.stats(), world.told())
    };
    let (trace, stats, (told, lost)) = run(FACT_CAPACITY);
    assert_eq!(lost, 0, "seed {seed}: room for every fact");
    assert!(told.started > 0, "seed {seed}: facts were told");
    let (silent, same, (none, dropped)) = run(0);
    assert!(silent == trace && same == stats, "seed {seed}: the same requests, at the same times");
    assert_eq!((none, dropped > 0), (Told::default(), true), "seed {seed}: every fact dropped");
    told
}
