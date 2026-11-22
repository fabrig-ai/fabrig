//! The checkout fault sweep (temper/docs/design/testing/worker.md, 4).

use skein_lib::Duration;
use skein_world::streams::{Stream, Streams};
use skein_world::sweep::Sweep;
use temper_worker_checkout_world::client::{Pick, Plan};
use temper_worker_checkout_world::{Settings, World};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT};

#[test]
fn seeded_worker_faults_fall_and_the_checks_hold() {
    let mut sweep = Sweep::new("worker checkout faults");
    sweep.configure(FETCH_CUT);
    sweep.configure(DISK_FULL);
    for seed in 0..8 {
        eprintln!("worker checkout faults seed {seed}");
        let streams = Streams::new(seed);
        let mut world = World::with_faults(Settings::calm(seed), streams);
        let mut workload = streams.rng(Stream::Workload, "clients");
        for _ in 0..32 {
            let at = world.now().saturating_add(Duration::from_millis(workload.between(0, 180_000)));
            let plan = Plan {
                workstream: u32::try_from(workload.below(4)).expect("four workstreams"),
                start: Some(if workload.chance(500) { Pick::Base } else { Pick::Branch }),
                pushes: u32::try_from(workload.between(0, 2)).expect("few pushes"),
                save: workload.chance(300),
                ..Plan::simple(0)
            };
            world.submit(at, plan);
        }
        let ledger = world.run_faults(100_000);
        sweep.ran(&ledger);
    }
    sweep.check_fell().expect("each configured checkout fault fell");
}
