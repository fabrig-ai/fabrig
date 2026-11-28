//! Pool-funded automatic work (jig/docs/design/domain/authority.md, 6.2).
use super::*;

pub(super) const POOL_PROJECT: u32 = 1;
pub(super) const OTHER_ACCOUNT: u32 = 30;
pub(super) const POOL: tasks::Funder = tasks::Funder::Pool { project: POOL_PROJECT, person: ALICE, period: 0 };
const PERIOD: tasks::Funder = tasks::Funder::Period { project: POOL_PROJECT, period: 0 };
const PERIOD_BUDGET: u64 = 10_000;

pub(super) fn fixture(ceiling: u64, allotment: u64, run_cap: u64) -> (Core, Env<Limits>) {
    let (mut core, env) = super::fixture(vec![
        account(ACCOUNT, accounts::Holder::Party(ALICE)),
        source(ACCOUNT, SOURCE, 0, accounts::Kind::SignIn),
        account(OTHER_ACCOUNT, accounts::Holder::Party(BOB)),
        source(OTHER_ACCOUNT, 31, 0, accounts::Kind::SignIn),
    ]);
    roster(&mut core, &env, POOL_PROJECT, &[ALICE, BOB]);
    let mut rules = core.authority.rules().clone();
    rules.ceiling.budget.spend = PERIOD_BUDGET;
    rules.period_spend = PERIOD_BUDGET;
    rules.maximum_run_spend = run_cap;
    let mut policy = core.authority.policy(POOL_PROJECT).unwrap().clone();
    policy.ceiling.budget.spend = ceiling;
    policy.period_spend = PERIOD_BUDGET;
    policy.roles[0].authority.budget.spend = ceiling.min(500);
    policy.roles[0].period_spend = allotment;
    core.authority = authority::Domain::new(rules, env.limits.authority).expect("pool fixture rules");
    let mut out = Queue::uncapped();
    authority::step(&mut core.authority, authority::Event::Policy { project: POOL_PROJECT, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: POOL_PROJECT }));
    send_task(&mut core, &env.limits, tasks::Event::Check);
    send_task(&mut core, &env.limits, tasks::Event::Restored);
    (core, env)
}

pub(super) fn repair(seed: connector::RepairSeed) -> Event {
    Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: 1, code: 1 },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"one person's automatic work".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
        holdings: Box::new([]),
    }
}

fn open_period(core: &mut Core, env: &Env<Limits>, budget: u64) {
    send_task(
        core,
        &env.limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: POOL_PROJECT, period: 0, budget },
    );
}

fn carve_pool(core: &mut Core, env: &Env<Limits>, person: u64, budget: u64) {
    send_task(
        core,
        &env.limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(2)),
            project: POOL_PROJECT,
            person,
            period: 0,
            budget,
        },
    );
}

#[test]
fn a_pool_seed_opens_missing_funding_in_the_make_decision() {
    for (period_open, pool_open) in [(false, false), (true, false), (true, true)] {
        let (mut core, env) = fixture(6000, 7000, 2000);
        if period_open {
            open_period(&mut core, &env, if pool_open { 7000 } else { PERIOD_BUDGET });
        }
        if pool_open {
            carve_pool(&mut core, &env, ALICE, 7000);
        }
        let seed = core.pool_seed(POOL_PROJECT, ALICE).expect("the pool, not the period, has room");
        let number = seed.number();
        assert_eq!(seed.opens.0.is_some(), !period_open);
        assert_eq!(seed.opens.1.is_some(), !pool_open);
        assert_eq!(core.tasks.funding(PERIOD).is_some(), period_open, "seeding writes no ledger");
        assert_eq!(core.tasks.funding(POOL).is_some(), pool_open);
        let out = step(&mut core, &env, repair(seed));
        let writes: Vec<_> = out
            .into_iter()
            .filter_map(|request| {
                if let Request::Write(Write::Save(Record::Tasks(record))) = request { Some(record) } else { None }
            })
            .collect();
        assert!(
            writes
                .iter()
                .any(|record| { if let tasks::Stored::Live(row) = record { row.number == number } else { false } })
        );
        for (funder, changed) in [(PERIOD, !pool_open), (POOL, true)] {
            assert_eq!(
                writes.iter().any(|record| {
                    if let tasks::Stored::Ledger(row) = record { row.funder == funder } else { false }
                }),
                changed,
                "only changed ledgers join the task's decision"
            );
        }
        let budget = core.tasks.task(number).expect("made in the decision opening the pool").numbers.budget;
        assert_eq!(budget, 2000);
        assert_eq!(core.tasks.funding(POOL).unwrap().numbers.reserved, budget);
        assert_eq!(
            core.tasks.funding(PERIOD).unwrap().numbers.reserved,
            7000,
            "only the pool is carved from the period"
        );
    }
}

#[test]
fn a_pool_seeds_budget_is_the_least_of_ceiling_pool_room_and_run_cap() {
    for (ceiling, allotment, cap, expected) in [
        (6000, 5000, 2000, Some(2000)),
        (1000, 5000, 2000, Some(1000)),
        (6000, 500, 2000, Some(500)),
        (0, 5000, 2000, None),
        (6000, 0, 2000, None),
    ] {
        let (mut core, _) = fixture(ceiling, allotment, cap);
        let before = core.counters.deployment;
        let seed = core.pool_seed(POOL_PROJECT, ALICE);
        assert_eq!(seed.as_ref().map(|seed| seed.budget), expected);
        assert_eq!(seed.as_ref().map(|seed| seed.authority.budget.spend), expected);
        if expected.is_none() {
            assert_eq!(core.counters.deployment, before);
        }
    }
}

#[test]
fn a_reserved_pool_leaves_only_its_remaining_room_and_none_takes_no_number() {
    let (mut core, env) = fixture(6000, 3000, 2000);
    let first = core.pool_seed(POOL_PROJECT, ALICE).unwrap();
    drop(step(&mut core, &env, repair(first)));
    let last = core.pool_seed(POOL_PROJECT, ALICE).unwrap();
    assert_eq!(last.budget, 1000, "reservations leave less than the run cap");
    drop(step(&mut core, &env, repair(last)));
    let before = core.counters.deployment;
    let funding = *core.tasks.funding(POOL).unwrap();
    assert_eq!(funding.numbers.reserved, funding.numbers.budget);
    assert!(core.pool_seed(POOL_PROJECT, ALICE).is_none());
    assert_eq!(core.counters.deployment, before);
    assert_eq!(core.tasks.funding(POOL), Some(&funding));
}

#[test]
fn a_pool_seed_requires_an_open_project_policy_and_a_current_role() {
    let (mut core, env) = fixture(6000, 7000, 2000);
    // An existing pool is not a role: a removed member still has no seed.
    open_period(&mut core, &env, PERIOD_BUDGET);
    carve_pool(&mut core, &env, SERVICE, 7000);
    let before = core.counters.deployment;
    core.start.consumed = false;
    assert!(core.pool_seed(POOL_PROJECT, SERVICE).is_none());
    assert!(core.start.consumed, "even a refused seed is an operational input");
    assert_eq!(core.counters.deployment, before);
    core.projects.clear();
    assert!(core.pool_seed(POOL_PROJECT, ALICE).is_none());
    assert_eq!(core.counters.deployment, before);
    core.projects.push(POOL_PROJECT).unwrap();
    core.authority = authority::Domain::new(core.authority.rules().clone(), env.limits.authority).unwrap();
    assert!(core.pool_seed(POOL_PROJECT, ALICE).is_none());
    assert_eq!(core.counters.deployment, before);
    assert!(core.tasks.task(before.tasks.saturating_add(1)).is_none());

    let (mut exhausted, _) = fixture(6000, 7000, 2000);
    exhausted.counters.deployment.tasks = u64::MAX;
    assert!(exhausted.pool_seed(POOL_PROJECT, ALICE).is_none());
    assert_eq!(exhausted.counters.deployment.tasks, u64::MAX);
    assert!(exhausted.tasks.funding(PERIOD).is_none());
    assert!(exhausted.tasks.funding(POOL).is_none());
}

#[test]
fn a_pool_the_period_cannot_carve_refuses_the_repair_make() {
    let (mut core, env) = fixture(6000, 7000, 2000);
    core.connectors = Box::new([1]);
    open_period(&mut core, &env, PERIOD_BUDGET);
    carve_pool(&mut core, &env, BOB, 7000);
    let period_before = *core.tasks.funding(PERIOD).unwrap();
    let seed = core.pool_seed(POOL_PROJECT, ALICE).expect("an unopened pool reads its role's allotment");
    let number = seed.number();
    let out = step(&mut core, &env, repair(seed));
    for request in &out {
        if let Request::Ask { ask: Ask::RepairRefused { .. }, .. } = request {
            panic!("the configured connector does not declare resources");
        }
    }
    assert!(core.tasks.task(number).is_none());
    assert!(core.tasks.funding(POOL).is_none());
    assert_eq!(core.tasks.funding(PERIOD), Some(&period_before));
}
