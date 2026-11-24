//! The worker's restart stream is separate from its existing story seeds.
use skein_world::sweep::Sweep;
use temper_faults::restarts::WORKER;
use temper_worker_domain_world::whole::restarts;

#[test]
fn worker_restarts_sweep_delivery_and_contact_stories() {
    let mut sweep = Sweep::new("worker restarts");
    for kind in WORKER {
        sweep.configure(kind);
    }
    // One seed per kind: twelve is the minimum for one restart draw per seed.
    for seed in [0, 1, 2, 3, 5, 7, 14, 15, 18, 20, 51, 55] {
        eprintln!("worker restarts seed {seed}");
        let ledger = restarts::stories(seed);
        sweep.ran(&ledger);
    }
    sweep.check_fell().expect("every worker restart kind fell");
}
