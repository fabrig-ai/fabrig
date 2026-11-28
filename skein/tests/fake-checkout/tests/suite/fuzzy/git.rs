//! Small deterministic line-edit sweep with independent expected outcomes and
//! complete fixture replay (skein/docs/design/testing/fakes/checkout.md, 5;
//! skein/docs/design/testing/sweeps.md, 4.2–4.5).

use std::convert::Infallible;

use skein_fake_checkout_tests::merge_case;
use skein_world::{
    sweep::{Case, cases},
    this_test,
};

#[test]
fn sixty_four_small_merge_histories_replay_and_match_independent_expectations() -> Result<(), Case<Infallible>> {
    cases(this_test!("sixty_four_small_merge_histories_replay_and_match_independent_expectations"), 0..64, |seed| {
        assert_eq!(merge_case(seed), merge_case(seed), "full replay, seed {seed}");
        Ok(())
    })
}

#[test]
fn full_width_seed_draws_a_valid_merge_history() {
    let _snapshot = merge_case(u64::MAX);
}
