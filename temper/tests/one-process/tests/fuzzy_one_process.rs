use skein_world::domain::Verdict;
use temper_one_process_world::{fixtures, world};
#[test]
fn drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once() {
    for seed in 1..=4 {
        let mut draw = skein_lib::Rng::new(seed);
        let (before, outcome) = world::crash(fixtures::calm(seed), skein_world::Cut::Kill, &mut draw);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}, before {before}");
        assert_eq!(outcome.restarts, 1, "seed {seed}, before {before}");
        assert_eq!(outcome.adopted, 1, "seed {seed}, before {before}");
        world::require_pending_at_cut(&outcome, seed, before);
        assert!(outcome.adoption_pending_at_restart, "seed {seed}, before {before}: pending adoption at cut");
        assert!(outcome.adoptions_answered <= 1, "seed {seed}, before {before}: one answer per key");
    }
}
#[test]
fn drawn_power_losses_keep_the_conformance_rules() {
    for seed in 1..=4 {
        let mut draw = skein_lib::Rng::new(seed);
        let (before, outcome) = world::crash(fixtures::calm(seed), skein_world::Cut::PowerLoss, &mut draw);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}, before {before}");
        assert_eq!(outcome.restarts, 1, "seed {seed}, before {before}");
        assert_eq!(outcome.adopted, 1, "seed {seed}, before {before}");
        world::require_pending_at_cut(&outcome, seed, before);
        assert!(outcome.adoption_pending_at_restart, "seed {seed}, before {before}: pending adoption at cut");
        assert!(outcome.drained, "seed {seed}, before {before}");
    }
}

#[test]
fn a_chat_reads_then_replies_and_waits() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::ChatReadThenWait { words: b"Hello" });
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_answered, 1);
}

#[test]
fn a_merge_in_progress_is_pushed_with_both_parents() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Merge);
    assert_eq!(outcome.verdict, Verdict::Passed);
    let [pushed] = outcome.pushed.as_slice() else { panic!("one push: {:?}", outcome.pushed) };
    assert_eq!(pushed.parents, [fixtures::BRANCH_HEAD, fixtures::MOVED_BASE]);
}
