//! The restart sweep is separate from the crash and upgrade sweeps.
use skein_world::sweep::Sweep;
use temper_faults::restarts::SIMULATED;
use temper_one_process_world::restarts;

#[test]
#[expect(clippy::single_element_loop, reason = "one pinned seed is enough for both configured kinds to fall")]
fn drawn_unit_signals_and_older_layouts_keep_the_worlds_rules() {
    let mut sweep = Sweep::new("one-process restarts");
    for kind in SIMULATED {
        sweep.configure(kind);
    }
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/dev");
    for seed in [5] {
        eprintln!("one-process restarts seed {seed}");
        let outcome = restarts::drawn(seed, &directory);
        sweep.ran(&outcome.ledger);
    }
    sweep.check_fell().expect("both simulated restart faults fell");
}
