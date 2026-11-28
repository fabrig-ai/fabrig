//! Each case replays its story at the case seed (skein/docs/design/testing/sweeps.md, 4.2).

use crate::support::{Story, run};
use skein_world::sweep::{Case, cases};
use std::convert::Infallible;

#[test]
fn seeded_sign_ins_faults_and_clocks_settle_and_replay() -> Result<(), Case<Infallible>> {
    cases(skein_world::this_test!("seeded_sign_ins_faults_and_clocks_settle_and_replay"), 0..32, |seed| {
        let story = match seed % 3 {
            0 => Story::SignIn,
            1 => Story::Expired,
            _ => Story::AnotherKey,
        };
        assert_eq!(run(seed, story), run(seed, story), "seed {seed}");
        Ok(())
    })
}
