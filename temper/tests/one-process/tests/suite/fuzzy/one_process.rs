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
