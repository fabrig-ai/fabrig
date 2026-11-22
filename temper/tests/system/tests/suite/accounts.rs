use jig_domain_accounts::{Held, Stored};
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
