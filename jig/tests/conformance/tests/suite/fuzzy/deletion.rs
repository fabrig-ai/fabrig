use std::convert::Infallible;

use jig_conformance::Cut;
use jig_conformance_world::deletion::{self, FAMILIES};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};

#[test]
fn every_family_deletion_survives_every_commit_and_seeded_random_cuts() -> Result<(), Case<Infallible>> {
    cases(this_test!("every_family_deletion_survives_every_commit_and_seeded_random_cuts"), [71, 232], |seed| {
        for family in FAMILIES {
            let mut baseline = deletion::prepared(family, seed);
            let first = baseline.store.applied + 1;
            baseline.cut(Cut::EachCommitFrom(first));
            deletion::remove(&mut baseline);
            if let Some(number) = baseline.forked_at() {
                assert!(baseline.outcome().crashes.contains(&number), "{family:?}, seed {seed}, cut {number}");
            }
            jig_conformance::cut_passed();
            let last = baseline.store.applied;
            assert_eq!(baseline.outcome().forked, (first..=last).collect::<Vec<_>>(), "{family:?}, seed {seed}");
            let cut = Cut::Random { seed };
            eprintln!("deletion {family:?}, seed {seed}, cut {cut:?}");
            let _world = deletion::run(family, seed, cut);
        }
        Ok(())
    })
}
