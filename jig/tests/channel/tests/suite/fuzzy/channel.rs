use jig_channel_world::stories;
use skein_world::{domain::Verdict, sweep::cases, this_test};
use std::convert::Infallible;

#[test]
fn drawn_channel_worlds_keep_the_referee_s_rules() -> Result<(), String> {
    cases(this_test!("drawn_channel_worlds_keep_the_referee_s_rules"), 1..=12, |seed| {
        let outcome = stories::drawn(seed);
        assert_eq!(outcome.verdict, Verdict::Passed);
        assert_eq!(outcome.answers_acknowledged, 1);
        Ok::<(), Infallible>(())
    })
    .map_err(|case| case.to_string())
}

#[test]
fn a_replaced_grant_s_roots_are_checked_at_every_step() {
    // These named grant-replacement stories check roots at every step on a fixed seed.
    const SEED: u64 = 7;
    for story in [stories::Story::Key { replaced: true }, stories::Story::ApplicationGeneration] {
        let outcome = stories::every_root_step(SEED, story);
        assert_eq!(outcome.verdict, Verdict::Passed, "{story:?}, seed {SEED}");
    }
}
