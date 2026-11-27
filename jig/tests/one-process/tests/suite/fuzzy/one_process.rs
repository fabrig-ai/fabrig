use core::convert::Infallible;
use jig_one_process_world::{fixtures, world};
use skein_world::{domain::Verdict, sweep::cases, this_test};

#[test]
fn drawn_kills_and_drains_keep_the_conformance_rules() {
    // Engine submissions from spawn: the key request is kept after 135.
    // Draw a cut in the 20-submission run/commit window starting at 178,
    // before the first transcript shard opens.
    const KEY_KEPT: u32 = 135;
    const RUN_COMMIT_BASE: u32 = KEY_KEPT + 43;
    const RUN_COMMIT_WIDTH: u64 = 20;
    cases(this_test!("drawn_kills_and_drains_keep_the_conformance_rules"), 1..=7, |seed| {
        let mut draw = skein_lib::Rng::new(seed);
        let before = RUN_COMMIT_BASE + u32::try_from(draw.below(RUN_COMMIT_WIDTH)).expect("bounded submission draw");
        for (label, cut) in [("kill", skein_world::Cut::Kill), ("power-loss", skein_world::Cut::PowerLoss)] {
            let outcome = world::crash(fixtures::calm(seed), cut, before);
            assert_eq!(outcome.verdict, Verdict::Passed, "{label} before {before}");
            assert_eq!(outcome.restarts, 1, "{label} before {before}");
            assert!(outcome.drained, "{label} before {before}");
        }
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn text_chats_park_or_retry_across_seeds() {
    cases(this_test!("text_chats_park_or_retry_across_seeds"), 1..=3, |seed| {
        for chat in [fixtures::Chat::OpenText, fixtures::Chat::FiniteText] {
            let mut settings = fixtures::calm(seed);
            settings.chat = chat;
            let outcome = world::run(settings, Box::new([jig_one_process_world::Script::start_chat(b"Hello")]));
            assert_eq!(outcome.verdict, Verdict::Passed, "{chat:?}");
            let evidence = outcome.chat.expect("text scenario evidence");
            assert_eq!(evidence.calls, if chat == fixtures::Chat::OpenText { 1 } else { 2 });
            assert_eq!(evidence.claims, if chat == fixtures::Chat::OpenText { 1 } else { 2 });
        }
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
