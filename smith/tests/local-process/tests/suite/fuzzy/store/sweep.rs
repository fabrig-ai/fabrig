use super::{Cut, Initial, configured_mode, run};

pub(super) fn cuts(seed_base: u64, fresh: bool, initial: Initial) -> u32 {
    let baseline = run(configured_mode(seed_base, fresh, initial, skein_sim::Config::calm()));
    for before in 0..baseline.procs[0].policy.submitted {
        let seed = seed_base + u64::from(before);
        let mut world = configured_mode(seed, fresh, initial, skein_sim::Config::calm());
        world.cut(0, before, Cut::PowerLoss);
        let outcome = run(world);
        assert!(outcome.procs[0].policy.reserved.is_some(), "{initial:?}, seed {seed}, cut {before}");
        assert_eq!(outcome.machine.files().open_handles(), 0);
    }
    let cuts = baseline.procs[0].policy.submitted;
    skein_checks::count("seed", u64::from(cuts) + 1);
    cuts
}
