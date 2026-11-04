//! Deployment procedures spend at the effect (jig/docs/design/domain/authority.md, 6–7).
use core::effects::connector;
use jig_core_world::world::fixture;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

const PROJECT: u32 = 1;
const PERSON: u64 = 1;
const PERIOD: u64 = 1;
const CONNECTOR: u16 = 1;
const PROCEDURE: u32 = 1;
const CHARTER: u32 = 1;
const PERIOD_BUDGET: u64 = 1000; // The core world's configured project spend.
const EFFECT_PRICE: u64 = 3; // The core world's connector effect maximum.
const EFFECT_KIND: u16 = 5;
const PERIOD_FUNDER: tasks::Funder = tasks::Funder::Period { project: PROJECT, period: PERIOD };
const POOL_FUNDER: tasks::Funder = tasks::Funder::Pool { project: PROJECT, person: PERSON, period: PERIOD };
const EXECUTOR: tasks::Executor = tasks::Executor::Procedure { connector: CONNECTOR, code: PROCEDURE };

fn outputs(domain: &mut core::Core, env: &Env<core::Limits>, event: core::Event) -> Vec<core::Request> {
    let mut out = core::step(domain, env, event);
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    requests
}

fn task_event(domain: &mut core::Core, env: &Env<core::Limits>, event: tasks::Event) -> Vec<core::Request> {
    outputs(domain, env, core::Event::Tasks(event))
}

fn seeded_core() -> (core::Core, Env<core::Limits>) {
    let (config, limits) = fixture(1900, false);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.core };
    let domain = super::start_links::first(config.core, &env);
    (domain, env)
}

fn reserved_period(pool: u64) -> (core::Core, Env<core::Limits>) {
    let (mut domain, env) = seeded_core();
    task_event(
        &mut domain,
        &env,
        tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(1)),
            project: PROJECT,
            period: PERIOD,
            budget: PERIOD_BUDGET,
        },
    );
    task_event(
        &mut domain,
        &env,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(2)),
            project: PROJECT,
            person: PERSON,
            period: PERIOD,
            budget: pool,
        },
    );
    (domain, env)
}

fn spec() -> tasks::Spec {
    tasks::Spec {
        job: None,
        asks: false,
        words: b"deployment procedure".as_slice().into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
    }
}

fn admit(
    domain: &mut core::Core,
    env: &Env<core::Limits>,
    seed: connector::RepairSeed,
    executor: tasks::Executor,
) -> u64 {
    let number = seed.number();
    outputs(
        domain,
        env,
        core::Event::ConnectorRepair {
            seed: Box::new(seed),
            executor,
            spec: spec(),
            contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
            holdings: Box::new([]),
        },
    );
    number
}

fn procedure(domain: &mut core::Core, env: &Env<core::Limits>) -> u64 {
    let seed = domain.procedure_seed(PROJECT).expect("policy and task number exist");
    admit(domain, env, seed, EXECUTOR)
}

fn step(
    domain: &mut core::Core,
    env: &Env<core::Limits>,
    task: u64,
    decision: core::ProcedureStep,
) -> Vec<core::Request> {
    let (_, _, step) = domain.tasks.procedure_due(task).expect("procedure is due");
    outputs(domain, env, core::Event::ProcedureStep { task, step, connector: CONNECTOR, code: PROCEDURE, decision })
}

fn numbers(domain: &core::Core, funder: tasks::Funder) -> tasks::Numbers {
    domain.tasks.funding(funder).expect("opened funding source").numbers
}

#[test]
fn a_procedure_task_with_no_agent_is_made_while_the_period_is_reserved_whole() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let before = [numbers(&domain, PERIOD_FUNDER), numbers(&domain, POOL_FUNDER)];
    let task = procedure(&mut domain, &env);
    let row = domain.tasks.task(task).expect("zero-budget procedure admitted");
    assert_eq!(row.numbers, tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 });
    assert_eq!(row.authority.budget.spend, 0);
    assert_eq!(row.requester, tasks::Party::Deployment { project: PROJECT });
    assert_eq!(row.funder, PERIOD_FUNDER);
    assert_eq!(row.executor, EXECUTOR);
    step(&mut domain, &env, task, core::ProcedureStep::new(core::StepEnd::Wait));
    assert_eq!(domain.tasks.task(task).expect("waiting procedure").attempt, 1);
    task_event(&mut domain, &env, tasks::Event::WakeProcedure { task });
    let result = tasks::TaskResult::Report { words: b"done".as_slice().into() };
    step(&mut domain, &env, task, core::ProcedureStep::new(core::StepEnd::Result(result.clone())));
    task_event(&mut domain, &env, tasks::Event::EffectsSettled { task });
    let ended = task_event(&mut domain, &env, tasks::Event::Settled { task });
    assert!(ended.iter().any(|request| matches!(request,
        core::Request::Write(core::Write::Save(core::Record::Tasks(tasks::Stored::Ended(row))))
            if row.number == task && row.phase == tasks::Phase::Ended(tasks::Ending::Done(result.clone()))
    )));
    assert_eq!([numbers(&domain, PERIOD_FUNDER), numbers(&domain, POOL_FUNDER)], before);
    assert_eq!(before[0], tasks::Numbers { budget: PERIOD_BUDGET, spent: 0, spent_below: 0, reserved: PERIOD_BUDGET });
}

#[test]
fn a_repair_that_runs_an_agent_still_waits_for_unreserved_spend() {
    let (mut domain, _) = reserved_period(PERIOD_BUDGET);
    assert!(domain.repair_seed(PROJECT).is_none());
}

#[test]
fn a_repair_with_unreserved_spend_keeps_its_funded_budget() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET - EFFECT_PRICE);
    let seed = domain.repair_seed(PROJECT).expect("unreserved spend funds a repair");
    let task = admit(&mut domain, &env, seed, EXECUTOR);
    let row = domain.tasks.task(task).expect("funded repair admitted");
    assert_eq!(row.numbers.budget, EFFECT_PRICE);
    assert_eq!(row.authority.budget.spend, EFFECT_PRICE);
    assert_eq!(numbers(&domain, PERIOD_FUNDER).reserved, PERIOD_BUDGET);
    let before = numbers(&domain, PERIOD_FUNDER);
    let made = effect(&mut domain, &env, task);
    assert!(made.iter().any(|request| matches!(
        request,
        core::Request::Ask { ask: core::Ask::Effect(connector::Ask::Keep { .. }), .. }
    )));
    assert_eq!(domain.tasks.task(task).expect("funded repair").numbers.spent, EFFECT_PRICE);
    assert_eq!(numbers(&domain, PERIOD_FUNDER), before);
}

#[test]
fn a_zero_budget_procedure_cannot_delegate_a_run_that_spends() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let task = procedure(&mut domain, &env);
    let mut given = domain.tasks.task(task).expect("procedure").authority.clone();
    given.budget.spend = EFFECT_PRICE;
    given.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let member = core::Delegate {
        executor: tasks::Executor::Agent { charter: CHARTER },
        spec: spec(),
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        authority: given,
        deadline: tasks::Deadline::Named,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(tasks::WakePolicy::DEFAULT),
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: people::Budget::Named,
    };
    let context = domain.tasks.delegation(task).expect("procedure delegation");
    let mut findings = Queue::uncapped();
    let mut authority = domain.authority.policy(PROJECT).expect("policy").ceiling.clone();
    authority.budget.spend = EFFECT_PRICE;
    authority.delegation = authority::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let checked = domain.connector_batch_admit(
        PROJECT,
        &context.authority,
        context.numbers,
        context.tasks_left,
        Box::new([authority::Delegate {
            executor: authority::Executor::Charter(CHARTER),
            authority,
            symbolic: Box::new([]),
        }]),
        &mut findings,
    );
    assert_ne!(checked.answer, authority::Answer::Allow);
    assert!(
        std::iter::from_fn(|| findings.pop())
            .any(|finding| matches!(finding, authority::Finding::Spend { source: authority::Source::Task }))
    );
    let before = numbers(&domain, PERIOD_FUNDER);
    let requests = step(
        &mut domain,
        &env,
        task,
        core::ProcedureStep { batch: Box::new([member]), ..core::ProcedureStep::new(core::StepEnd::Wait) },
    );
    assert!(
        requests.iter().any(|request| matches!(request, core::Request::Ask { ask: core::Ask::StepRefused { .. }, .. }))
    );
    assert!(domain.tasks.task(task).expect("procedure").delegates.is_empty());
    assert_eq!(numbers(&domain, PERIOD_FUNDER), before);
}

#[test]
fn a_procedure_seed_cannot_admit_an_agent_executor() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let seed = domain.procedure_seed(PROJECT).expect("procedure seed");
    let task = admit(&mut domain, &env, seed, tasks::Executor::Agent { charter: CHARTER });
    assert!(domain.tasks.task(task).is_none());
}

fn description() -> connector::EffectDescription {
    connector::EffectDescription {
        waits_for_dependencies: false,
        connector: CONNECTOR,
        purpose: 19,
        form: connector::EffectForm::Set,
        recovery: connector::Recovery::Keyed,
        effect: authority::Effect {
            connector: CONNECTOR,
            kind: EFFECT_KIND,
            name: authority::Name { segments: Box::new([Box::from([1]), Box::from([1])]) },
            state: [13; 32],
            price: Some(EFFECT_PRICE),
            access: authority::EffectAccess::Owned,
            additional: Box::new([]),
            guards: Box::new([]),
        },
    }
}

fn effect(domain: &mut core::Core, env: &Env<core::Limits>, task: u64) -> Vec<core::Request> {
    let (_, _, step) = domain.tasks.procedure_due(task).expect("effect's procedure is due");
    let owner = domain.effect_owner();
    outputs(
        domain,
        env,
        core::Event::EffectStart {
            owner,
            connector: CONNECTOR,
            origin: core::EffectOrigin::Procedure { task, step, entry: None },
        },
    );
    outputs(
        domain,
        env,
        core::Event::EffectConnector(connector::Event::Described { owner, description: Box::new(description()) }),
    )
}

#[test]
fn a_zero_budget_procedures_priced_effect_waits_then_spends_only_the_unreserved_period() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let task = procedure(&mut domain, &env);
    let before = [numbers(&domain, PERIOD_FUNDER), numbers(&domain, POOL_FUNDER)];
    let waiting = effect(&mut domain, &env, task);
    assert!(waiting.iter().any(|request| matches!(
        request,
        core::Request::Ask { ask: core::Ask::Effect(connector::Ask::Drop { answer: authority::Answer::Wait, .. }), .. }
    )));
    assert!(
        !waiting.iter().any(|request| matches!(
            request,
            core::Request::Ask { ask: core::Ask::Effect(connector::Ask::Keep { .. }), .. }
        )),
        "{waiting:?}"
    );
    assert_eq!([numbers(&domain, PERIOD_FUNDER), numbers(&domain, POOL_FUNDER)], before);
    // Free exactly the effect's price; the people's remaining reservation stays intact.
    task_event(
        &mut domain,
        &env,
        tasks::Event::ResizePool {
            reply_to: ReplyTo::new(Token::new(3)),
            project: PROJECT,
            person: PERSON,
            period: PERIOD,
            budget: PERIOD_BUDGET - EFFECT_PRICE,
        },
    );
    task_event(&mut domain, &env, tasks::Event::WakeProcedure { task });
    let made = effect(&mut domain, &env, task);
    assert!(
        made.iter().any(|request| matches!(
            request,
            core::Request::Ask { ask: core::Ask::Effect(connector::Ask::Keep { .. }), .. }
        )),
        "{made:?}"
    );
    assert_eq!(
        numbers(&domain, PERIOD_FUNDER),
        tasks::Numbers {
            budget: PERIOD_BUDGET,
            spent: EFFECT_PRICE,
            spent_below: 0,
            reserved: PERIOD_BUDGET - EFFECT_PRICE,
        }
    );
    assert_eq!(
        numbers(&domain, POOL_FUNDER),
        tasks::Numbers { budget: PERIOD_BUDGET - EFFECT_PRICE, spent: 0, spent_below: 0, reserved: 0 }
    );
    assert_eq!(
        domain.tasks.task(task).expect("procedure").numbers,
        tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 }
    );
    assert!(
        made.iter().any(|request| matches!(request,
            core::Request::Write(core::Write::Save(core::Record::Tasks(tasks::Stored::Ledger(row))))
                if row.funder == PERIOD_FUNDER && row.numbers.spent == EFFECT_PRICE
        )),
        "the effect's decision saves its period charge"
    );
}

#[test]
fn a_procedure_opens_its_period_without_reserving_any_spend() {
    let (mut domain, env) = seeded_core();
    let task = procedure(&mut domain, &env);
    let initial = tasks::Funder::Period { project: PROJECT, period: 0 };
    assert_eq!(domain.tasks.task(task).expect("admitted procedure").funder, initial);
    assert_eq!(numbers(&domain, initial), tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 });
}

#[test]
fn a_procedure_seed_needs_a_project_policy() {
    const UNKNOWN_PROJECT: u32 = 2; // Only project 1 is configured.
    let (mut domain, _) = seeded_core();
    assert!(domain.procedure_seed(UNKNOWN_PROJECT).is_none());
}

#[test]
fn a_procedure_seed_needs_an_unused_task_number() {
    let (mut domain, _) = seeded_core();
    let mut deployment = domain.counters.deployment();
    deployment.tasks = u64::MAX;
    domain.counters = core::Counters::new(deployment);
    assert!(domain.procedure_seed(PROJECT).is_none());
}

#[test]
fn a_priced_procedure_effect_waits_when_one_unit_is_missing() {
    const SHORT_BY: u64 = 1; // The unavailable side of the effect's exact price.
    let (mut domain, env) = reserved_period(PERIOD_BUDGET - EFFECT_PRICE + SHORT_BY);
    let task = procedure(&mut domain, &env);
    let before = numbers(&domain, PERIOD_FUNDER);
    let requests = effect(&mut domain, &env, task);
    assert!(requests.iter().any(|request| matches!(
        request,
        core::Request::Ask { ask: core::Ask::Effect(connector::Ask::Drop { answer: authority::Answer::Wait, .. }), .. }
    )));
    assert_eq!(numbers(&domain, PERIOD_FUNDER), before);
}

#[test]
fn a_zero_budget_agent_or_person_seed_cannot_make_a_task_or_open_a_period() {
    for executor in
        [tasks::Executor::Agent { charter: CHARTER }, tasks::Executor::Person(tasks::PersonAddress::Person(PERSON))]
    {
        let (mut domain, env) = seeded_core();
        let seed = domain.procedure_seed(PROJECT).expect("procedure seed");
        let task = admit(&mut domain, &env, seed, executor);
        assert!(domain.tasks.task(task).is_none());
        assert!(domain.tasks.funding(tasks::Funder::Period { project: PROJECT, period: 0 }).is_none());
    }
}

#[test]
fn a_procedures_effect_checks_both_price_ceilings_before_period_funds() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let task = procedure(&mut domain, &env);
    let mut description = description();
    description.effect.price = Some(PERIOD_BUDGET + 1);
    let mut findings = Queue::uncapped();
    assert_eq!(
        domain.connector_effect_admit(task, &description, env.wall, &[], &mut findings),
        authority::Answer::Refuse
    );
    let findings: Vec<_> = std::iter::from_fn(|| findings.pop()).collect();
    for source in [authority::Source::Project, authority::Source::Deployment] {
        assert!(findings.contains(&authority::Finding::Price { source }), "{findings:?}");
    }
    assert_eq!(domain.tasks.task(task).expect("procedure").authority.budget.spend, 0);
}

#[test]
fn a_procedures_effect_still_needs_a_grant_while_period_funds_are_short() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let task = procedure(&mut domain, &env);
    let mut description = description();
    description.effect.kind = EFFECT_KIND + 1;
    let mut findings = Queue::uncapped();
    assert_eq!(
        domain.connector_effect_admit(task, &description, env.wall, &[], &mut findings),
        authority::Answer::Refuse
    );
    let findings: Vec<_> = std::iter::from_fn(|| findings.pop()).collect();
    for source in [authority::Source::Task, authority::Source::Project, authority::Source::Deployment] {
        assert!(findings.contains(&authority::Finding::Grant { source }), "{findings:?}");
    }
}

#[test]
fn an_unpriced_procedure_effect_needs_no_unreserved_period_spend() {
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let task = procedure(&mut domain, &env);
    let before = numbers(&domain, PERIOD_FUNDER);
    let mut description = description();
    description.effect.price = None;
    let mut findings = Queue::uncapped();
    assert_eq!(
        domain.connector_effect_admit(task, &description, env.wall, &[], &mut findings),
        authority::Answer::Allow
    );
    assert!(findings.is_empty());
    assert_eq!(numbers(&domain, PERIOD_FUNDER), before);
}

#[test]
fn a_procedures_priced_effect_uses_the_current_period() {
    const NEXT_PERIOD: u64 = PERIOD + 1;
    let (mut domain, env) = reserved_period(PERIOD_BUDGET);
    let task = procedure(&mut domain, &env);
    let before = numbers(&domain, PERIOD_FUNDER);
    let opened = task_event(
        &mut domain,
        &env,
        tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(4)),
            project: PROJECT,
            period: NEXT_PERIOD,
            budget: EFFECT_PRICE,
        },
    );
    assert!(opened.iter().any(|request| matches!(request,
        core::Request::Write(core::Write::Save(core::Record::Tasks(tasks::Stored::ClosedLedger(row))))
            if row.funder == PERIOD_FUNDER && row.numbers.spent == before.spent
    )));
    assert!(domain.tasks.funding(PERIOD_FUNDER).is_none());
    let made = effect(&mut domain, &env, task);
    assert!(made.iter().any(|request| matches!(
        request,
        core::Request::Ask { ask: core::Ask::Effect(connector::Ask::Keep { .. }), .. }
    )));
    assert_eq!(domain.tasks.task(task).expect("procedure").funder, PERIOD_FUNDER);
    assert_eq!(numbers(&domain, tasks::Funder::Period { project: PROJECT, period: NEXT_PERIOD }).spent, EFFECT_PRICE);
    assert_eq!(domain.tasks.task(task).expect("procedure").numbers.spent, 0);
}
