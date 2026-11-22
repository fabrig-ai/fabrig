use jig_domain_accounts::{Held, Stored};
use temper_engine_domain::Record;
use temper_system_world::composed::{BILLING_ACCOUNT, FIRST_SOURCE, SECOND_SOURCE, World, report_script};
use temper_worker_domain::agent;

#[test]
fn each_partys_goal_uses_its_own_account_credential() {
    use temper_system_world::composed::accounts::{FIRST_ACCOUNT, OTHER_ACCOUNT, credential};
    let mut world = World::two_accounts(941);
    world.run_account_goals();
    for (goal, account) in world.account_goals().into_iter().zip([FIRST_ACCOUNT, OTHER_ACCOUNT]) {
        let queries: Vec<_> = world.account_queries().iter().filter(|(task, _)| *task == goal).collect();
        assert!(!queries.is_empty(), "each goal calls its provider");
        assert!(
            queries.iter().all(|(_, query)| query.credential == Some(credential(account))),
            "party's account {account}"
        );
        assert!(world.account_goal_done(goal));
    }
}

#[test]
#[ignore = "provider failures: P26 must choose another account of the same holder"]
fn a_limited_account_frees_its_tasks() {
    use skein_fake_llm_domain::api::Strike;
    use temper_system_world::composed::accounts::{FIRST_ACCOUNT, OTHER_ACCOUNT};
    let mut world = World::same_holder_accounts(942);
    world.plan_account_fault(FIRST_ACCOUNT, Strike::Spent, 1, None);
    world.finish_account_goal().expect("a limited account frees its tasks");
    assert_eq!(world.first_goal_account(), Some(OTHER_ACCOUNT));
}

#[test]
fn spent_credits_stop_placement() {
    use skein_fake_llm_domain::api::Strike;
    use temper_system_world::composed::accounts::FIRST_ACCOUNT;
    const GOAL_MINUTES: u64 = 3;
    const GOAL_STEPS: u32 = 20_000;
    let mut world = World::two_accounts(943);
    world.plan_account_fault(FIRST_ACCOUNT, Strike::Spent, 1, None);
    let bound = (skein_lib::Duration::from_secs(60 * GOAL_MINUTES), GOAL_STEPS);
    world
        .run_account_case(FIRST_ACCOUNT, false, bound, bound)
        .expect("the other account's goal ends while this one waits");
    assert!(world.spent_stopped_placement(FIRST_ACCOUNT), "spent credits mark the account and stop new placements");
    world.judge_account_runs().expect("spent credits spend no task try");
}

#[test]
#[ignore = "provider failures: P126 an unserved model must narrow the account and choose again"]
fn an_unserved_model_moves_the_task() {
    use skein_fake_llm_domain::api::Strike;
    use temper_system_world::composed::accounts::{FIRST_ACCOUNT, OTHER_ACCOUNT};
    let mut world = World::same_holder_accounts(944);
    world.plan_account_fault(FIRST_ACCOUNT, Strike::Unserved { model: b"fake-1".as_slice().into() }, 1, None);
    let ended = world.finish_account_goal();
    assert!(
        world.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Accounts(Stored::Account { number: FIRST_ACCOUNT, list: Some(list), .. }))
            if list.read.is_some() && list.names & 1 == 0)),
        "the refused model is removed from the account's model list"
    );
    ended.expect("an unserved model moves the task");
    assert_eq!(
        world.first_goal_account(),
        Some(OTHER_ACCOUNT),
        "the refused model is not placed on that account again"
    );
}

#[test]
fn a_worker_limit_keeps_its_source_identity() {
    let mut world = World::two_source_chat(918, report_script(true));
    world.until_call();
    world.notice(agent::Up::Limited { account: FIRST_SOURCE, reset: None });
    // Drain the process notice, both roots and the durable account decision.
    for _ in 0..world.notice_steps() {
        world.tick();
        if world.store.rows.values().any(|row| {
            matches!(
                row,
                Record::Core(jig_domain_core::Record::Accounts(Stored::Source {
                    number: FIRST_SOURCE,
                    held: Some(Held::Limited { reset: None }),
                    ..
                }))
            )
        }) {
            break;
        }
    }
    let sources: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::Accounts(Stored::Source { account, number, held, .. })) = row {
                Some((*account, *number, *held))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        sources,
        vec![
            (BILLING_ACCOUNT, FIRST_SOURCE, Some(Held::Limited { reset: None })),
            (BILLING_ACCOUNT, SECOND_SOURCE, None),
        ]
    );
}
