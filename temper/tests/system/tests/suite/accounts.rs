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
        query.system.windows(spec.len()).any(|part| part == spec)
            || query
                .messages
                .iter()
                .flat_map(|message| &message.parts)
                .any(|part| matches!(part, Part::Text { text } if text.windows(spec.len()).any(|part| part == spec))),
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
