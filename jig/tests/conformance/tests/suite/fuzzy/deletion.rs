use jig_conformance::Cut;
use jig_conformance_world::deletion::{self, FAMILIES};

#[test]
fn every_family_deletion_survives_every_commit_and_seeded_random_cuts() {
    for seed in [71, 233] {
        for family in FAMILIES {
            let (first, last) = {
                let mut baseline = deletion::prepared(family, seed);
                let first = baseline.store.applied + 1;
                deletion::remove(&mut baseline);
                (first, baseline.store.applied)
            };
            for cut in (first..=last).map(Cut::AfterCommit).chain([Cut::Random { seed }]) {
                let world = deletion::run(family, seed, cut);
                if let Cut::AfterCommit(number) = cut {
                    assert!(world.outcome().crashes.contains(&number), "{family:?}, seed {seed}, cut {number}");
                }
            }
        }
    }
}
