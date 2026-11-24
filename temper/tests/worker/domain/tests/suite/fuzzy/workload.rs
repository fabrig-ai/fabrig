//! Workloads and their two ledgers (`temper/docs/design/testing/workloads.md`, 8.1).
use skein_world::sweep::Sweep;
use temper_faults::{bounds, parks};
use temper_worker_domain_world::whole::workload;

#[test]
fn worker_runs_park_resume_and_move() {
    let mut sweep = Sweep::new("worker runs park resume and move");
    sweep.configure(parks::DELIVERED_THEN_PARKED);
    sweep.configure(parks::RESUMED_ON_ANOTHER_WORKER);
    // Seed 0 parks a delivery; seed 1 resumes on another worker.
    for seed in 0..2 {
        eprintln!("worker runs park resume and move: seed {seed}");
        let ledger = workload::run(seed, true).expect("worker workload finding");
        sweep.ran(&ledger);
    }
    sweep.check_fell().expect("each park fell");
}

#[test]
fn the_workers_tables_reach_their_bounds() {
    let mut sweep = Sweep::new("the workers tables reach their bounds");
    sweep.configure(bounds::RUN_SLOTS);
    sweep.configure(bounds::BUILD_PLACES);
    // WORKSPACES: four slots, but at most three concurrent runs.
    // PREPARE_PERMITS: four permits, but at most three concurrent prepares.
    // PER_RUN_SCRATCH: eight repositories, but each of the three runs has only one.
    // The build place fills in seed 0; the first full run-slot table is seed 3.
    for seed in 0..4 {
        eprintln!("the workers tables reach their bounds: seed {seed}");
        let ledger = workload::run(seed, false).expect("worker workload finding");
        sweep.ran(&ledger);
    }
    sweep.check_fell().expect("each reachable bound fell");
}
