//! Seeded pacing preserves the stepped world's run (skein/docs/design/testing/sweeps.md, 4.2–4.5).

use std::convert::Infallible;

use skein_world::{
    sweep::{Case, cases},
    this_test,
};

use crate::support;

#[test]
fn sliced_steps_replay_across_seeds() -> Result<(), Case<Infallible>> {
    cases(this_test!("sliced_steps_replay_across_seeds"), 0..1024, |seed| {
        support::compare(seed);
        Ok(())
    })
}
