//! Restart-family coverage over the follows workload (testing/restarts.md, 9).
use skein_world::sweep::Sweep;
use temper_faults::restarts::ENGINE;

#[test]
fn fuzzy_engine_restarts() {
    let mut sweep = Sweep::new("engine restarts over follows");
    for kind in ENGINE {
        sweep.configure(kind);
    }
    // Two start classes plus a second signal cover ENGINE in three runs:
    // each class draws its prerequisite crash, and the second signal its stop.
    // Reuse the settled follows world, keeping each seed's draws and ledger separate.
    for ledger in super::follows::run_with_restarts(&[0, 2, 3]) {
        eprintln!("fell: {:?}", ledger.fell());
        sweep.ran(&ledger);
    }
    sweep.check_fell().expect("every engine restart kind fell");
}
