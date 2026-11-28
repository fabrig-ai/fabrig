//! Application-made goals use their requester's money and identity (domain/authority.md, 6.3).
use super::*;
use jig_domain_contract as contract;

const TASK: u64 = 1;
const BUDGET: u64 = 4_000;
const CLOSER: tasks::Closer = tasks::Closer { connector: 1, code: 6 };

fn fixture() -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 2, budget: 20_000 },
    );
    let mut parent = new_row(&core, TASK, tasks::Funder::Period { project: 1, period: 2 }, None);
    parent.numbers.budget = 10_000;
    parent.authority.budget.spend = 10_000;
    parent.authority.delegation = tasks::Delegation {
        kinds: Box::new([tasks::AuthorityExecutor::Charter(1), tasks::AuthorityExecutor::Procedure(6)]),
        tasks: 8,
        depth: 4,
    };
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
    assert!(core.tasks.task(TASK).is_some());
    core.counters.deployment.tasks = TASK;
    (core, limits)
}

fn goal(context: &tasks::DelegationContext, budget: u64) -> connector::SeededGoal {
    connector::SeededGoal {
        charter: 1,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"repair".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 32 },
        authority: context.authority.clone(),
        llm: Some(tasks::LlmChoice {
            account: None,
            main: tasks::ModelEffort { model: 0, effort: Some(1), speed: None },
            subagents: Box::new([]),
            later: None,
        }),
        budget,
        tracked: Some(7),
        label: 9,
        flags: tasks::Flags { listed: true, ended_by: tasks::EndedBy::Closer(CLOSER), ..tasks::Flags::PLAIN },
        holdings: Box::new([]),
    }
}

#[test]
fn an_application_goal_is_one_agent_delegate_with_the_named_budget_and_settings() {
    let (mut core, limits) = fixture();
    let context = core.tasks.delegation(TASK).expect("live requester");
    let seed = core.repair_seed(1).expect("project repair seed");
    let number = seed.number();
    let resource = tasks::Name { connector: 1, path: Box::new([Box::from(*b"g")]) };
    send_task(&mut core, &limits, tasks::Event::Resource { name: resource.clone(), hold: tasks::HoldKind::Shared });
    let mut asked = goal(&context, BUDGET);
    asked.holdings = Box::new([tasks::Holding::Write { resource, kind: 1 }]);
    let spec = asked.spec.clone();
    let contract = asked.contract.clone();
    let llm = asked.llm.clone();
    let flags = asked.flags;
    let holdings = asked.holdings.clone();
    let mut authority = asked.authority.clone();
    authority.budget.spend = BUDGET;
    authority.delegation.depth = 3;
    let event = seed.goal(TASK, &context, asked).expect("goal fits its requester");
    let tasks::Event::Make { creator, ref batch, .. } = event else { panic!("one making") };
    assert_eq!(creator, tasks::Party::Task(TASK));
    assert_eq!(batch.len(), 1);
    let mut out = Queue::uncapped();
    tasks::step(&mut core.tasks, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks }, event, &mut out);
    while let Some(request) = out.pop() {
        if let tasks::Request::Refused { problem, .. } = request {
            panic!("goal refused: {problem:?}");
        }
    }
    let row = core.tasks.task(number).expect("agent goal admitted");
    assert_eq!(row.executor, tasks::Executor::Agent { charter: 1 });
    assert_eq!(row.requester, tasks::Party::Task(TASK));
    assert_eq!(row.funder, tasks::Funder::Task(TASK));
    assert_eq!(row.numbers, tasks::Numbers { budget: BUDGET, spent: 0, spent_below: 0, reserved: 0 });
    assert_eq!(row.authority, authority);
    assert_eq!((&row.spec, &row.contract, &row.llm), (&spec, &contract, &llm));
    assert_eq!((row.tracked, row.label, row.flags), (Some(7), 9, flags));
    assert_eq!(row.holdings, holdings);
    assert_eq!(row.wake, tasks::WakePolicy::DEFAULT);
    let parent = core.tasks.task(TASK).expect("requester live");
    assert_eq!(parent.numbers.reserved, context.numbers.reserved + BUDGET);
    assert_eq!(&*parent.delegates, &[number]);
}

#[test]
fn a_seeded_goal_never_cuts_its_budget_to_fit_any_of_its_requesters_four_numbers() {
    let (mut core, _) = fixture();
    let mut context = core.tasks.delegation(TASK).expect("live requester");
    // Each debit alone takes the remaining amount from the exact fit to one short.
    for numbers in [
        tasks::Numbers { budget: BUDGET - 1, spent: 0, spent_below: 0, reserved: 0 },
        tasks::Numbers { budget: BUDGET, spent: 1, spent_below: 0, reserved: 0 },
        tasks::Numbers { budget: BUDGET, spent: 0, spent_below: 1, reserved: 0 },
        tasks::Numbers { budget: BUDGET, spent: 0, spent_below: 0, reserved: 1 },
    ] {
        context.numbers = numbers;
        let seed = core.repair_seed(1).expect("seed");
        assert_eq!(seed.goal(TASK, &context, goal(&context, BUDGET)).expect_err("one short"), tasks::Refusal::Funding);
    }
    context.numbers = tasks::Numbers { budget: 20_000, spent: 2_000, spent_below: 3_000, reserved: 5_000 };
    let seed = core.repair_seed(1).expect("seed");
    assert_eq!(
        seed.goal(TASK, &context, goal(&context, 12_000)).expect_err("only 10,000 left"),
        tasks::Refusal::Funding
    );
    let seed = core.repair_seed(1).expect("seed");
    assert!(seed.goal(TASK, &context, goal(&context, 10_000)).is_ok(), "exact remaining balance fits");
}

#[test]
fn check_make_refuses_the_same_model_as_the_making_without_changing_its_requester() {
    let (mut core, limits) = fixture();
    let context = core.tasks.delegation(TASK).expect("seeded goal fixture");
    let before = core.tasks.task(TASK).expect("seeded goal fixture").clone();
    let seed = core.repair_seed(1).expect("seeded goal fixture");
    let valid = seed.goal(TASK, &context, goal(&context, BUDGET)).expect("seeded goal fixture");
    assert_eq!(core.check_make(&limits, &valid), Ok(()));
    let seed = core.repair_seed(1).expect("seeded goal fixture");
    let number = seed.number();
    let mut asked = goal(&context, BUDGET);
    asked.authority.models = tasks::Models::NONE;
    let made = seed.goal(TASK, &context, asked).expect("seeded goal fixture");
    let why = tasks::Refusal::Llm { model: 0, why: tasks::LlmFault::Authority };
    assert_eq!(core.check_make(&limits, &made), Err(why));
    assert_eq!(core.tasks.task(TASK), Some(&before));
    assert!(core.tasks.task(number).is_none());
    let output = step(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, Event::Tasks(made));
    let mut refused = false;
    for request in &output {
        if let Request::Now(now) = request
            && let Now::Link(LinkNow::RefusedPayload { problem, .. }) = now.as_ref()
        {
            assert_eq!(problem.why, why);
            refused = true;
        }
    }
    assert!(refused, "the making refuses the preflighted model");
    assert_eq!(core.tasks.task(TASK), Some(&before));
    assert!(core.tasks.task(number).is_none());
}

#[test]
fn check_make_requires_the_resource_report_before_saying_a_goal_can_be_made() {
    let (mut core, limits) = fixture();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let context = core.tasks.delegation(TASK).expect("seeded goal fixture");
    let seed = core.repair_seed(1).expect("seeded goal fixture");
    let number = seed.number();
    let resource = tasks::Name { connector: 1, path: Box::new([Box::from(*b"g")]) };
    let mut asked = goal(&context, BUDGET);
    asked.holdings = Box::new([tasks::Holding::Write { resource: resource.clone(), kind: 1 }]);
    let make = seed.goal(TASK, &context, asked).expect("fits the requester");
    assert_eq!(core.check_make(&env.limits, &make), Err(tasks::Refusal::Unreported));
    connector_step(
        &mut core,
        &env,
        resource.connector,
        contract::Up::Resources(contract::resources::Up::Resource {
            project: 1,
            name: resource.clone(),
            role: connector::ResourceRole::Unavailable,
            hold: connector::HoldKind::Shared,
        }),
    );
    assert_eq!(core.check_make(&env.limits, &make), Err(tasks::Refusal::ResourceUnavailable));
    connector_step(
        &mut core,
        &env,
        resource.connector,
        contract::Up::Resources(contract::resources::Up::Resource {
            project: 1,
            name: resource,
            role: connector::ResourceRole::Owned,
            hold: connector::HoldKind::Shared,
        }),
    );
    assert_eq!(core.check_make(&env.limits, &make), Ok(()));
    step(&mut core, &env, Event::Tasks(make));
    let row = core.tasks.task(number).expect("reported making fits");
    assert!(row.holdings.is_empty(), "both paths normalize shared resources");
}

#[test]
fn a_seeded_goal_requires_lifetime_room_and_a_delegation_level() {
    let (mut core, _) = fixture();
    let mut context = core.tasks.delegation(TASK).expect("live requester");
    context.tasks_left = 0;
    let seed = core.repair_seed(1).expect("seed");
    assert_eq!(seed.goal(TASK, &context, goal(&context, BUDGET)).expect_err("no task left"), tasks::Refusal::Tree);
    context.tasks_left = 1;
    context.authority.delegation.depth = 0;
    let seed = core.repair_seed(1).expect("seed");
    assert_eq!(seed.goal(TASK, &context, goal(&context, BUDGET)).expect_err("no level left"), tasks::Refusal::Depth);
    context.authority.delegation.depth = 1;
    let seed = core.repair_seed(1).expect("seed");
    let tasks::Event::Make { batch, .. } = seed.goal(TASK, &context, goal(&context, BUDGET)).expect("last level fits")
    else {
        panic!("making")
    };
    assert_eq!(batch.first().expect("one goal").authority.delegation.depth, 0);
}
