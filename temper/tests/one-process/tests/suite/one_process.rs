use skein_fake_peers::forge::Delivery;
use skein_world::domain::Verdict;
use temper_one_process_world::{fixtures, world};

#[test]
fn a_restart_over_a_live_change_answers_a_person() {
    let outcome = world::restart_held_change(fixtures::calm(fixtures::SEED));
    assert_eq!(outcome.signed_in, 1);
}

#[test]
fn an_adoption_saves_a_pending_window_before_the_engine_commit() {
    // This story pins seed 1's saved pending window, independently of the shared seed.
    const ADOPTION_SEED: u64 = 1;
    let outcome = world::adoption(fixtures::calm(ADOPTION_SEED));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.adopted, 1);
    assert!(
        outcome.adoption_window.is_some_and(|(start, end)| start < end),
        "saved pending adoption window precedes the engine commit: {:?}",
        outcome.adoption_window
    );
}

#[test]
fn a_refused_restore_ends_with_its_in_process_worker() {
    let outcome = world::refused_restore(fixtures::calm(fixtures::SEED));
    assert_eq!(
        outcome.end,
        Some(temper_service::End::Engine(temper_service::EngineEnd::Stopped(
            temper_service::EngineStop::StartRefused {
                phase: Some(jig_domain_core::start::Phase::LoadCore),
                failure: jig_domain_core::start::Failure::Settings {
                    failure: jig_domain_core::settings::Failure {
                        setting: jig_domain_core::settings::Setting::Policy(1),
                        fault: jig_domain_core::settings::Fault::Shape,
                        detail: None
                    },
                    changed: false,
                },
            }
        )))
    );
}
#[test]
fn a_signal_mid_run_interrupts_a_chats_run_and_the_next_start_takes_it_again() {
    let outcome =
        world::drain_mid_run(fixtures::calm(fixtures::SEED), world::Story::ChatReadThenWait { words: b"Hello" }, true);
    assert_eq!(outcome.settings_revision, 2, "the hosted ctl change is retained");
    assert_restarted_run(outcome);
}
#[test]
fn a_signal_mid_run_interrupts_a_coding_run_and_the_next_start_takes_it_again() {
    let outcome = world::drain_mid_run(fixtures::calm(fixtures::SEED), world::Story::CodingAtStop, true);
    let stopped = assert_restarted_run(outcome);
    assert!(stopped.workspace.is_some(), "prepared workspace at agent Start");
    let [before, after] = stopped.change.expect("coding run's change");
    assert_eq!(before, after);
    assert_eq!(stopped.saved_head[0], stopped.saved_head[1], "saved branch is not rewound");
    assert_eq!(stopped.checkout_head, stopped.saved_head[0], "checkout starts at the saved head");
    assert_eq!(stopped.checkout_file.as_deref(), Some(b"fixed\n".as_slice()));
    assert!(!stopped.saved[0].is_empty(), "producer parked and saved before the stop");
    assert!(!stopped.writers.is_empty(), "producer owns its writer slot");
}
fn assert_restarted_run(outcome: world::Outcome) -> world::Stopped {
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.drained);
    assert_eq!(outcome.restarts, 1);
    let stopped = outcome.stopped.expect("signal found a run");
    assert_eq!(
        (stopped.tries[1].lost, stopped.interrupted),
        (0, 1),
        "no lost try and one Recheck terminal; {stopped:?}"
    );
    assert_eq!(outcome.claims_settled_at_restart, 0);
    assert_eq!(stopped.tries[0], stopped.tries[1]);
    assert_eq!(stopped.saved[0], stopped.saved[1]);
    let next = stopped.next.expect("new attempt started");
    assert!(next > stopped.attempt);
    assert!(stopped.writers.iter().all(|&(attempt, lost)| attempt == next && !lost));
    stopped
}
#[test]
fn a_person_signs_in_by_the_printed_link_and_a_chat_is_answered() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Chat { words: b"Hello" });
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_answered, 1);
}
#[test]
fn a_providers_list_is_read_by_the_workers_hosted_program() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::ModelsList);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    let list = outcome.model_list.expect("provider's list outcome");
    assert_eq!(list.requests, [Some(0)], "the fake accepted the added key once");
    assert_eq!(list.agents, 1, "one hosted agent-list process");
    assert_eq!(list.queries, 0, "a list is not a run");
    assert!(list.new_in_me, "the list's new name reached the person's me");
    assert!(list.accounting_unchanged, "no attempt, spend or budget moved");
    assert_eq!(outcome.chats_answered, 0);
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
    assert!(outcome.deliveries.starts_with(&[Delivery::NoAddress, Delivery::Written]));
}

#[test]
fn one_signed_push_gets_one_204_from_the_engine_listener() {
    let (deliveries, receipt) = world::signed_push(fixtures::calm(fixtures::SEED));
    // The fixture's topic was prepared before the listener had an address.
    assert_eq!(deliveries, [Delivery::NoAddress, Delivery::Written]);
    assert!(receipt.signed_push, "the only write is a signed push to /hooks/1");
    assert_eq!(receipt.accepted, 1);
    assert_eq!(receipt.responses, [(1, 204)], "one response on that accepted connection");
}

#[test]
fn ctl_adopts_the_forgejo_home_as_owned() {
    let (adopted, repositories) = world::ctl_adoption(fixtures::calm(fixtures::SEED));
    assert_eq!(adopted.exit, skein_io::kernel::Exit::Code(0), "{}", String::from_utf8_lossy(&adopted.error));
    assert!(
        adopted.output.starts_with(b"Adopted acme/app on forge 1 as owned.\n"),
        "{}",
        String::from_utf8_lossy(&adopted.output)
    );
    let [row] = repositories.as_slice() else { panic!("one durable repository: {repositories:?}") };
    assert_eq!(row.project, 1);
    assert!(row.home);
    assert_eq!(row.provider.forge, 1);
    assert_eq!(row.provider.repository, temper_playground::documents::REPOSITORY);
    assert_eq!(row.owner.as_ref(), b"acme");
    assert_eq!(row.name.as_ref(), b"app");
    assert_eq!(row.landing.as_ref(), b"main");
    assert_eq!(row.role, temper_engine_domain_forge::Role::Owned);
}
