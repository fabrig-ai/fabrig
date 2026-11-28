//! An owner's model and account choices across its next delegate.
use super::*;

use temper_engine_domain_world::fixtures::{FIRST_LLM_ACCOUNT, SECOND_LLM_ACCOUNT, add_second_account};

fn start_owner(driver: &mut Driver) -> engine::Assignment {
    // One slot for the owner and one for its survey.
    const SLOTS: u32 = 2;
    driver.send(hello_with(Token::new(7), SLOTS, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(driver, 12);
    driver.settle();
    assigned(driver)
}

#[test]
fn an_account_rank_keeps_the_owner_and_changes_its_next_delegate() {
    let ([bootstrap, configuration], mut bounds) = batch_settings(2, 1, 500, false, false, false);
    temper_engine_domain_world::fixtures::two_account_limits(&mut bounds);
    let mut store = temper_engine_domain_world::fixtures::restored_store(bootstrap, &bounds);
    add_second_account(&mut store);
    let mut driver = Driver::cold(store, configuration, &bounds);
    let owner = start_owner(&mut driver);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(80)),
        sign_in: driver.session(),
        key: [80; 16],
        ask: people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::MoveAccount { account: SECOND_LLM_ACCOUNT, place: 0 },
        },
    }));
    driver.settle();
    let child = call_batch(&mut driver, &owner, 90, Box::new([report_delegate(b"survey", Box::new([]))]))[0];
    let mut ranked: Vec<_> = driver
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Accounts(accounts::Stored::Account { number, place, .. })) => {
                Some((*place, *number))
            }
            _ => None,
        })
        .collect();
    ranked.sort_unstable();
    assert_eq!(ranked, [(0, SECOND_LLM_ACCOUNT), (1, FIRST_LLM_ACCOUNT)]);
    assert_eq!(live(&driver, owner.task).kept_account, Some(FIRST_LLM_ACCOUNT));
    assert_eq!(live(&driver, child).kept_account, Some(SECOND_LLM_ACCOUNT));
}

#[test]
fn an_agent_amends_its_delegate_to_the_default_model_without_effort() {
    let (mut configuration, bounds) = batch_settings(2, 1, 500, false, false, false);
    for config in &mut configuration {
        let charter = &mut config.core.defaults.charters[0];
        let mut owner = charter.policy.model.clone();
        owner.name = b"owner".as_slice().into();
        owner.efforts = Box::new([b"high".as_slice().into()]);
        owner.effort = Some(b"high".as_slice().into());
        charter.policy.alternatives = Box::new([owner.clone()]);
        config.core.settings.models = Box::new([charter.policy.model.clone(), owner]);
    }
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    let parent = start_owner(&mut driver);
    let mut member = report_delegate(b"survey", Box::new([]));
    member.llm = Some(tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model: 1, effort: Some(0), speed: None },
        subagents: Box::new([]),
        later: None,
    });
    let child = call_batch(&mut driver, &parent, 90, Box::new([member]))[0];
    let mut amendment = task_amendment(None, None, b"use the default model");
    amendment.llm = Some(tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model: 0, effort: None, speed: None },
        subagents: Box::new([]),
        later: None,
    });
    tool_call(
        &mut driver,
        &parent,
        91,
        engine::Tool::Amend { keep_parameters: false, target: child, revision: 0, amendment },
    );
    assert_eq!(live(&driver, child).revisions.spec, 1);
    assert_eq!(
        live(&driver, child).llm.as_ref().expect("explicit choice").main,
        tasks::ModelEffort { model: 0, effort: None, speed: None }
    );
    assert_eq!(
        driver.answered(91),
        Some(&engine::CallAnswer::Amended {
            task: child,
            revision: 1,
            changed: engine::Changed { tracked: None, fields: engine::Changed::LLM },
        })
    );
}
