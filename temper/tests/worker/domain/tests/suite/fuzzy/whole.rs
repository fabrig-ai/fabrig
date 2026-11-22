//! Deterministic whole-worker boundary replay against the forge and disk.
use skein_world::sweep::Sweep;
use temper_worker_domain_world::whole::faults;
use temper_worker_domain_world::whole::{Case, run};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT, PLACEMENT_BUSY};

#[test]
fn seeded_worker_delivery_and_contact_stories_replay() {
    for seed in 0..16 {
        for case in [Case::Delivered, Case::CancelledDelivery, Case::ContactWithin, Case::ContactPast] {
            assert_eq!(
                run(seed, case).expect("world checks"),
                run(seed, case).expect("world checks"),
                "seed {seed}, case {case:?}"
            );
        }
    }
}

#[test]
fn seeded_worker_faults_fall_and_the_checks_hold() {
    let mut sweep = Sweep::new("worker domain faults");
    for kind in [FETCH_CUT, DISK_FULL, PLACEMENT_BUSY] {
        sweep.configure(kind);
    }
    for seed in 0..8 {
        eprintln!("worker domain faults: seed {seed}");
        let ledger = faults::seeded(seed).expect("worker domain finding");
        sweep.ran(&ledger);
    }
    sweep.check_fell().expect("all configured faults fell");
}
