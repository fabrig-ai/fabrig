use skein_fake_peers::forge::Delivery;
use skein_world::domain::Verdict;
use temper_one_process_world::referee::{restart, secrets};
use temper_one_process_world::{fixtures, world};
#[test]
fn a_fresh_goals_unencodable_review_releases_the_host_and_cancels() {
    temper_one_process_world::review_delivery::walk(0);
}
#[test]
fn a_changes_unencodable_review_after_two_restarts_cancels_and_closes() {
    temper_one_process_world::review_delivery::walk(2);
}
#[test]
fn a_signal_mid_run_drains_and_the_next_start_settles_the_claims() {
    let outcome = world::drain_mid_run(fixtures::calm(fixtures::SEED), true);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.drained);
    assert_eq!(outcome.restarts, 1);
    assert_eq!(outcome.claims_settled_at_restart, 1);
    assert_eq!(outcome.settings_revision, 2, "the hosted ctl change is retained");
}
#[test]
fn a_secret_in_a_root_fails_the_referee_naming_it() {
    let root = format!("root {:?}", fixtures::LLM_TOKEN);
    for (name, bytes, expected) in [
        ("formatted root", root.into_bytes(), Some("LLM access token")),
        ("binary frame", [b"\xff".as_slice(), fixtures::LLM_TOKEN, b"\x80"].concat(), Some("LLM access token")),
        ("interrupted token", [b"test-\xfftoken".as_slice()].concat(), None),
    ] {
        assert_eq!(secrets::in_bytes(&bytes), expected, "{name}");
    }
}
#[test]
fn a_restart_showing_part_of_a_commit_fails_the_referee() {
    let records = fixtures::two_forge_records();
    let uncertain: restart::Records =
        records.iter().map(|row| (temper_engine_domain_forge::stored_key(row), row.clone())).collect();
    let partial: restart::Records =
        records.iter().take(1).map(|row| (temper_engine_domain_forge::stored_key(row), row.clone())).collect();
    assert_eq!(
        restart::check(&restart::Records::new(), Some(&uncertain), &partial),
        Err(temper_engine_domain_forge::Key::Names(2))
    );
}
#[test]
fn a_person_signs_in_by_the_printed_link_and_a_chat_is_answered() {
    for story in [world::Story::Chat { words: b"Hello" }, world::Story::ModelsList] {
        let outcome = world::run(fixtures::calm(fixtures::SEED), story);
        assert_eq!(outcome.verdict, Verdict::Passed);
        assert_eq!(outcome.signed_in, 1);
        if let Some(list) = outcome.model_list {
            assert_eq!(list.requests, [Some(0)], "the fake accepted the added key once");
            assert_eq!(list.agents, 1, "one hosted agent-list process");
            assert_eq!(list.queries, 0, "a list is not a run");
            assert!(list.new_in_me, "the list's new name reached the person's me");
            assert!(list.accounting_unchanged, "no attempt, spend or budget moved");
            assert_eq!(outcome.chats_answered, 0);
        } else {
            assert_eq!(outcome.chats_answered, 1);
        }
    }
}
#[test]
fn a_person_on_a_deployment_with_no_welcome_finds_the_inbox() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::NoWelcome);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
}
#[test]
fn a_person_keeps_the_sign_in_held_by_a_fresh_printed_link() {
    let starts = world::sign_in_again(fixtures::calm(fixtures::SEED));
    for outcome in starts {
        assert_eq!(outcome.verdict, Verdict::Passed);
        assert_eq!(outcome.signed_in, 1);
    }
}
#[test]
fn an_adopted_repository_gets_its_change_pushed_through_git() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Adopt);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.adopted, 1);
    assert_eq!(outcome.pushed.len(), 1);
    assert_eq!(outcome.deliveries, [Delivery::NoAddress, Delivery::Written]);
}
