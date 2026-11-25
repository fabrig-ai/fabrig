use jig_domain_accounts::{Held, Stored};
use jig_domain_core::{documents as docs, reads::Read};
use jig_domain_tasks as tasks;
use skein_fake_llm_domain::api::{Error, Part, Query};
use skein_lib::Duration;
use temper_engine_domain::Record;
use temper_engine_domain_world::fixtures::{FIRST_LLM_SOURCE, SECOND_LLM_SOURCE};
use temper_system_world::composed::{BILLING_ACCOUNT, FIRST_SOURCE, SECOND_SOURCE, World, report_script};
use temper_worker_domain::agent;

#[test]
fn every_survey_completion_uses_the_first_ranked_accounts_grant() {
    // The survey says one message and then finishes: two provider completions.
    const SURVEY_COMPLETIONS: usize = 2;
    let mut world =
        World::ranking_chat(919, temper_system_world::composed::text_script(b"Ready."), report_script(true));
    world.run();
    world.rank_second_account();
    world.request_survey();
    world.run();
    let survey = world.task_agent(2);
    assert_eq!(survey.prompts().len(), SURVEY_COMPLETIONS);
    assert_eq!(survey.completions().len(), SURVEY_COMPLETIONS);
    for completion in survey.completions() {
        assert_eq!(completion.grant, Some(smith_domain::GrantName { account: SECOND_LLM_SOURCE, generation: 1 }));
    }
    // Both sources are usable: the owner still requests the first after the rank.
    assert!(world.task_agent(1).completions().iter().all(
        |completion| completion.grant == Some(smith_domain::GrantName { account: FIRST_LLM_SOURCE, generation: 1 })
    ));
}

#[test]
fn a_provider_limit_resumes_the_same_work_after_its_reset_without_an_act() {
    let mut world = World::provider_hold(950, Error::Exhausted { retry_after: Some(Duration::from_secs(2)) });
    world.until_account_held();
    let waiting = task(&world, 2).clone();
    assert_eq!(waiting.phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(world.observed.limits, [(1, Some(Duration::from_secs(2)))]);
    let reset = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Core(jig_domain_core::Record::Accounts(Stored::Source {
                held: Some(Held::Limited { reset }),
                ..
            })) => *reset,
            _ => None,
        })
        .expect("provider supplied its reset");
    let docs::Document::Task(document) = world.provider_document(Read::Task { task: 2 }) else {
        panic!("task document")
    };
    assert_eq!(
        document.run_wait,
        Some(docs::TaskRunWait::Account {
            account: Some(0),
            source: Some(1),
            why: docs::TaskWaitWhy::Source(docs::TaskSourceState::Limited { until: Some(reset) }),
        })
    );
    let docs::Document::Deployment(document) = world.provider_document(Read::Deployment) else {
        panic!("deployment document")
    };
    let account = document.accounts.iter().find(|a| a.number == 0).expect("kept account");
    assert_eq!(account.state, docs::AccountState::Later { until: Some(reset) });
    assert!(!account.usable);
    let source = account.sources.iter().find(|s| s.number == 1).expect("kept source");
    assert_eq!(source.state, docs::AccountState::Later { until: Some(reset) });
    assert!(!source.usable);
    let failed_queries = world.observed.queries.len();
    assert_spec(&world.observed.queries.last().expect("failed query").2, 0);
    world.advance(Duration::from_secs(1));
    assert_eq!(world.observed.queries.len(), failed_queries, "no query before reset");
    world.advance(Duration::from_secs(2));
    world.finish_provider_work();
    assert!(task(&world, 2).ended_at.expect("result time") > reset);
    let (_, at, query) = world.observed.queries.iter().rev().find(|(task, _, _)| *task == 2).expect("resumed query");
    assert!(at.as_nanos() > reset.as_nanos(), "the resumed query follows the reset");
    assert_spec(query, 0);
    assert_same_work(&world, &waiting);
}

#[test]
fn a_provider_refusal_resumes_the_same_work_on_the_replaced_key() {
    let mut world = World::provider_hold(951, Error::Unauthorized);
    world.until_account_held();
    let waiting = task(&world, 2).clone();
    assert_eq!(waiting.phase, tasks::Phase::Active(tasks::Active::Due));
    let docs::Document::Task(document) = world.provider_document(Read::Task { task: 2 }) else {
        panic!("task document")
    };
    assert_eq!(
        document.run_wait,
        Some(docs::TaskRunWait::Account {
            account: Some(0),
            source: Some(1),
            why: docs::TaskWaitWhy::Source(docs::TaskSourceState::Refused),
        })
    );
    let docs::Document::Deployment(document) = world.provider_document(Read::Deployment) else {
        panic!("deployment document")
    };
    let account = document.accounts.iter().find(|a| a.number == 0).expect("kept account");
    assert_eq!(account.state, docs::AccountState::SignInAgain);
    assert!(!account.usable);
    let before = world.observed.queries.len();
    assert!(world.observed.queries.iter().all(|(_, _, query)| query.credential == Some(0)));
    assert_spec(&world.observed.queries.last().expect("refused query").2, 0);
    world.advance(Duration::from_secs(1));
    assert_eq!(world.observed.queries.len(), before, "a refused key needs the person");
    world.replace_provider_key();
    world.finish_provider_work();
    assert_spec(
        &world.observed.queries.iter().skip(before).find(|(task, _, _)| *task == 2).expect("resumed query").2,
        1,
    );
    assert_same_work(&world, &waiting);
}

fn task(world: &World, number: u64) -> &tasks::TaskRecord {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.number == number =>
            {
                Some(row)
            }
            _ => None,
        })
        .expect("original task")
}

fn assert_spec(query: &Query, credential: u32) {
    assert_eq!(query.credential, Some(credential));
    let spec = b"@held Retain this specification after the account resumes.";
    assert!(
        skein_lib::bytes::find(&query.system, spec).is_some()
            || query
                .messages
                .iter()
                .flat_map(|message| &message.parts)
                .any(|part| matches!(part, Part::Text { text } if skein_lib::bytes::find(text, spec).is_some())),
        "resumed query keeps the spec"
    );
}

fn assert_same_work(world: &World, waiting: &tasks::TaskRecord) {
    let ended = task(world, waiting.number);
    assert_eq!(waiting.revisions.spec, 0);
    assert_eq!(ended.revisions.spec, waiting.revisions.spec);
    assert_eq!(ended.kept_account, waiting.kept_account);
    assert_eq!(ended.kept_account, Some(0));
    assert!(ended.delegates.is_empty());
    let result = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
        words: b"The held work reported.".as_slice().into(),
    }));
    assert_eq!(ended.phase, result);
    assert_eq!(task(world, 1).phase, result);
    assert!(world.observed.queries.iter().all(|(task, _, _)| [1, 2].contains(task)), "no other agent");
    assert_eq!(task(world, 1).made, 2);
}

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
