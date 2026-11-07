use jig_channel_world::stories;
use skein_world::domain::Verdict;

#[test]
fn drawn_channel_worlds_keep_the_referee_s_rules() {
    for seed in 1..=12 {
        let outcome = stories::drawn(seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "drawn seed {seed}");
        assert_eq!(outcome.answers_acknowledged, 1, "drawn seed {seed}");
    }
}

#[test]
fn a_replaced_grant_s_roots_are_checked_at_every_step() {
    const SEED: u64 = 7;
    for story in [stories::Story::Key { replaced: true }, stories::Story::ApplicationGeneration] {
        let outcome = stories::every_root_step(SEED, story);
        assert_eq!(outcome.verdict, Verdict::Passed, "{story:?}, seed {SEED}");
    }
}
