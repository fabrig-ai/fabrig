//! People/accounts joins and a pool seed's financial payer (domain/accounts-uses.md, 1.2).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "unit tests build fixtures and inspect selected child events"
)]
mod pool_seed;

use crate::tests::{
    documents::accounts_document::{account, fixture, source},
    *,
};
const ALICE: u64 = 1;
const BOB: u64 = 2;
const SERVICE: u64 = 3;
const ACCOUNT: u32 = 20;
const SOURCE: u32 = 21;
const PROJECT: u32 = 0;

fn prepared(plan: bool, holder: accounts::Holder, uses: Option<Vec<Use>>) -> (Core, Env<Limits>) {
    let mut row = account(ACCOUNT, holder);
    if let accounts::Stored::Account { uses: kept, .. } = &mut row {
        *kept = uses.map(Vec::into_boxed_slice);
    }
    let (core, env) =
        fixture(vec![row, source(ACCOUNT, SOURCE, 0, if plan { accounts::Kind::SignIn } else { accounts::Kind::Key })]);
    (core, env)
}
fn serves(core: &Core, payer: Use) -> bool {
    crate::runs::coverage::serves(&core.people, core.accounts.account(ACCOUNT).unwrap(), payer, None)
}
fn roster(core: &mut Core, env: &Env<Limits>, project: u32, members: &[u64]) {
    let mut limits = env.limits.people;
    // Holder, another person and one service; the shared roster contains all three.
    limits.people = 3;
    limits.holdings = 3;
    core.people = people::Domain::new(&limits, Box::new([]), core.settings.deployment_provider, Box::new([]));
    core.people.start_restore(Wall::EPOCH);
    let people_env = env.with(limits);
    let mut out = Queue::uncapped();
    for party in [ALICE, BOB, SERVICE] {
        let provider = if party == SERVICE { core.settings.deployment_provider } else { 0 };
        let identity = people::Identity {
            key: people::IdentityKey { provider, subject: party.to_be_bytes().into() },
            login: Box::new([]),
            name: Box::new([]),
        };
        people::step(
            &mut core.people,
            &people_env,
            people::Event::Restore { record: people::Stored::Person { number: party, identity } },
            &mut out,
        );
    }
    let holdings =
        members.iter().map(|person| people::Holding { person: *person, role: people::Role::Owner }).collect();
    people::step(
        &mut core.people,
        &people_env,
        people::Event::Restore { record: people::Stored::Roles { project, holdings } },
        &mut out,
    );
    finish_people(&mut core.people, &people_env);
}

#[test]
fn default_keys_and_plans_cover_only_their_allowed_payers() {
    use accounts::{Holder, Use};
    for (plan, holder, party, project) in [
        (false, Holder::Party(ALICE), false, false),
        (false, Holder::Deployment, true, true),
        (true, Holder::Party(ALICE), false, true),
    ] {
        let (core, _) = prepared(plan, holder, None);
        assert!(serves(&core, Use::Party(ALICE)));
        assert_eq!(serves(&core, Use::Party(BOB)), party);
        assert_eq!(serves(&core, Use::Project(PROJECT)), project);
    }
}

#[test]
fn explicit_uses_replace_defaults_and_never_bypass_plan_rules() {
    use accounts::{Holder, Use};
    for (plan, uses, alice, bob) in [
        (false, vec![], false, false),
        (false, vec![Use::Party(BOB)], false, true),
        (true, vec![Use::Party(BOB)], false, false),
        (true, vec![Use::Party(ALICE)], true, false),
    ] {
        let (core, _) = prepared(plan, Holder::Party(ALICE), Some(uses));
        assert_eq!((serves(&core, Use::Party(ALICE)), serves(&core, Use::Party(BOB))), (alice, bob));
    }
}

#[test]
fn a_key_naming_a_project_pays_for_a_persons_work_in_it_and_nowhere_else() {
    let (mut core, _) = prepared(false, accounts::Holder::Deployment, Some(vec![Use::Project(PROJECT)]));
    with_model(&mut core);
    let pool = tasks::Funder::Pool { project: PROJECT, person: ALICE, period: 1 };
    let payer = core.tasks.prospective_payer(pool, &[]).expect("a named pool has a financial payer");
    assert_eq!(payer, tasks::Payer::Party(ALICE));
    assert!(!serves(&core, crate::runs::coverage::payer(payer)));
    assert!(serves(&core, Use::Project(PROJECT)));
    assert_eq!(core.account_preference(Some(&preferred(ACCOUNT)), payer, PROJECT, None), Ok(()));
    assert_eq!(
        core.account_preference(Some(&preferred(ACCOUNT)), payer, PROJECT + 1, None),
        Err(tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses })
    );
    assert!(core.account_coverage(payer, Some(PROJECT)).get(0).unwrap().serves);
    assert!(!core.account_coverage(payer, Some(PROJECT + 1)).get(0).unwrap().serves);
}

#[test]
fn a_pool_seeds_work_is_its_persons_and_their_plan_pays_in_a_shared_project() {
    use pool_seed::{OTHER_ACCOUNT, POOL, POOL_PROJECT, fixture, repair};
    let (mut core, env) = fixture(6000, 7000, 2000);
    let seed = core.pool_seed(POOL_PROJECT, ALICE).expect("a member's pool funds automatic work");
    let number = seed.number();
    let mut expected = task_authority(&core.authority.policy(POOL_PROJECT).unwrap().ceiling);
    expected.budget.spend = 2000;
    assert_eq!(seed.authority, expected, "only the ceiling's spend is narrowed");
    drop(step(&mut core, &env, repair(seed)));
    let row = core.tasks.task(number).expect("procedure made from the pool seed");
    assert_eq!(row.funder, POOL);
    assert_eq!(row.requester, tasks::Party::Deployment { project: POOL_PROJECT });
    assert_eq!(row.flags, tasks::Flags::PLAIN);
    assert_eq!(row.tracked, None);
    assert_eq!(core.tasks.payer(number), Some(tasks::Payer::Party(ALICE)));
    let coverage = core.account_coverage(tasks::Payer::Party(ALICE), Some(POOL_PROJECT));
    assert_eq!(
        coverage.as_slice(),
        &[
            accounts::Coverage { account: ACCOUNT, serves: true },
            accounts::Coverage { account: OTHER_ACCOUNT, serves: false },
        ]
    );
    assert!(core.accounts.account(ACCOUNT).unwrap().plan());
    assert!(core.accounts.account(OTHER_ACCOUNT).unwrap().plan());
}

#[test]
fn a_plan_naming_a_project_still_does_not_cover_another_persons_work_in_it() {
    let (mut core, _) = prepared(true, accounts::Holder::Party(ALICE), Some(vec![Use::Project(PROJECT)]));
    with_model(&mut core);
    assert!(serves(&core, Use::Project(PROJECT)));
    assert_eq!(
        core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(BOB), PROJECT, None),
        Err(tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses })
    );
}

#[test]
fn plan_coverage_rechecks_current_project_membership() {
    use accounts::{Holder, Use};
    let (mut core, env) = prepared(true, Holder::Party(ALICE), Some(vec![Use::Project(PROJECT), Use::Party(SERVICE)]));
    roster(&mut core, &env, PROJECT, &[ALICE, SERVICE]);
    assert!(serves(&core, Use::Project(PROJECT)) && serves(&core, Use::Party(SERVICE)));
    roster(&mut core, &env, PROJECT, &[ALICE, SERVICE, BOB]);
    assert!(!serves(&core, Use::Project(PROJECT)));
    assert!(serves(&core, Use::Party(SERVICE)));
}

fn preferred(number: u32) -> tasks::LlmChoice {
    tasks::LlmChoice {
        later: None,
        account: Some(number),
        main: tasks::ModelEffort { model: 0, effort: Some(0), speed: None },
        subagents: Box::new([]),
    }
}
fn with_model(core: &mut Core) {
    core.settings.models[0].provider = core.settings.llm_providers.providers[0].number;
}

#[test]
fn a_named_account_refusal_keeps_its_number() {
    let (mut core, _) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    assert_eq!(
        core.account_preference(Some(&preferred(999)), tasks::Payer::Party(ALICE), PROJECT, None),
        Err(tasks::Refusal::Account { account: 999, why: tasks::AccountWhy::Unknown })
    );
    assert_eq!(
        core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(BOB), PROJECT, None),
        Err(tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses })
    );
    core.settings.llm_providers.providers[0].number = 4;
    assert_eq!(
        core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(ALICE), PROJECT, None),
        Err(tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Provider })
    );
}

#[test]
fn an_unavailable_named_account_is_admitted_for_its_payer() {
    let (mut core, _) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    assert!(!core.accounts.account_views().get(0).unwrap().usable);
    assert_eq!(core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(ALICE), PROJECT, None), Ok(()));
}

#[test]
fn a_batch_preference_uses_the_prospective_financial_chain() {
    let (mut core, env) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    let first = new_row(&core, 2, tasks::Funder::Pool { project: PROJECT, person: BOB, period: 1 }, None);
    let mut child = new_row(&core, 3, tasks::Funder::Task(2), None);
    child.llm = Some(preferred(ACCOUNT));
    let mut batch = [first, child];
    assert_eq!(
        core.resolve_llm(&env.limits, &batch, &[]),
        Err(tasks::Problem::new(Some(3), tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses }))
    );
    batch[0].funder = tasks::Funder::Pool { project: PROJECT, person: ALICE, period: 1 };
    assert_eq!(core.resolve_llm(&env.limits, &batch, &[]), Ok(()));
}

#[test]
fn acceptance_rechecks_a_preference_against_the_new_funder() {
    let (mut core, env) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    let alice = tasks::Funder::Pool { project: PROJECT, person: ALICE, period: 1 };
    let mut new = new_row(&core, 2, alice, None);
    new.llm = Some(preferred(ACCOUNT));
    assert_eq!(core.funded_accounts(&[new.clone()], alice), Ok(()));
    let proposal = tasks::PersonProposal {
        number: 1,
        proposer: ALICE,
        project: PROJECT,
        action: tasks::PersonAction::Batch(Box::new([new])),
        reason: Box::new([]),
        state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
    };
    assert_eq!(
        core.goal_accept_allowed(&env.limits, &proposal, PROJECT, BOB, people::Role::Owner, 1, Wall::EPOCH),
        Err(people::Refusal::Account { account: ACCOUNT, why: people::AccountWhy::Uses })
    );
}

#[test]
fn a_limited_kept_account_refuses_a_new_account_of_its_provider() {
    let (mut core, env) = fixture(vec![
        account(ACCOUNT, accounts::Holder::Party(ALICE)),
        source(ACCOUNT, SOURCE, 0, accounts::Kind::Key),
        account(30, accounts::Holder::Party(ALICE)),
        source(30, 31, 0, accounts::Kind::Key),
    ]);
    with_model(&mut core);
    let mut out = Queue::uncapped();
    let account_env = env.with(env.limits.accounts);
    accounts::step(
        &mut core.accounts,
        &account_env,
        accounts::Event::Limited { account: SOURCE, reset: None },
        &mut out,
    );
    assert_eq!(
        core.account_preference(Some(&preferred(30)), tasks::Payer::Party(ALICE), PROJECT, Some(ACCOUNT)),
        Err(tasks::Refusal::Account { account: 30, why: tasks::AccountWhy::Limited })
    );
    accounts::step(
        &mut core.accounts,
        &account_env,
        accounts::Event::Change {
            request: Token::new(1),
            holder: accounts::Holder::Party(ALICE),
            change: accounts::Change::Resume { source: SOURCE },
        },
        &mut out,
    );
    assert_eq!(
        core.account_preference(Some(&preferred(30)), tasks::Payer::Party(ALICE), PROJECT, Some(ACCOUNT)),
        Ok(())
    );
}
