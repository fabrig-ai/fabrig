//! Report retention and whole-batch admission (domain/tasks/holds.md, 2).
use super::{fill::signed_in, *};

fn delegate_core() -> (Core, Env<Limits>) {
    with_live_names(0)
}

fn with_live_names(live: u8) -> (Core, Env<Limits>) {
    let mut limits = limits();
    limits.tasks.resource_reports = 8;
    limits.tasks.holdings = 8;
    limits.tasks.hold_wait = Duration::from_nanos(20);
    limits.tasks.batch = 8;
    limits.tasks.delegates = 8;
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.tasks.tree_tasks = 16;
    limits.tasks.inbox_messages = 32;
    limits.tasks.inbox_bytes = 4096;
    limits.authority.batch = 8;
    // Two waiting batches can outlive a third batch's retained refusal.
    limits.call_records = 3;
    let old = core_with_limits(&limits);
    let mut config = config(41);
    config.defaults.rules.ceiling.delegation.tasks = 16;
    config.defaults.rules.ceiling.delegation.depth = 5;
    config.seed_projects[0].policy.policy.ceiling.delegation.tasks = 16;
    config.seed_projects[0].policy.policy.ceiling.delegation.depth = 5;
    let mut given = config.seed_projects[0].policy.policy.roles[0].authority.clone();
    given.delegation.tasks = 16;
    given.delegation.depth = 5;
    set_seed_role_authority(&mut config, given);
    let mut core = seeded_core(config, &limits).expect("fixture core");
    core.counters.deployment.projects = 1;
    core.people = old.people;
    core.accounts = old.accounts;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    signed_in(&mut core, &env);
    let mut parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.authority.delegation.tasks = 8;
    parent.authority.delegation.depth = 4;
    let mut holdings = List::with_capacity(env.limits.tasks.holdings);
    for number in 1..=live {
        drop(report(&mut core, &env, b'z', number));
        holdings.push(tasks::Holding::Write { resource: name(b'z', number), kind: 1 }).unwrap();
    }
    parent.holdings = holdings.into_boxed();
    fixtures::make_batch(&mut core, &env.limits, tasks::Party::Person(1), Box::new([parent]));
    assert!(core.tasks.task(1).is_some());
    core.counters.deployment.tasks = 1;
    core.proofs.insert(1, RunProof::claimed(1, 1, 1, None, fleet::HostKind::Worker)).unwrap();
    (core, env)
}

fn name(prefix: u8, number: u8) -> tasks::Name {
    tasks::Name { connector: 1, path: Box::new([Box::from([prefix]), Box::from([number])]) }
}

fn report(core: &mut Core, env: &Env<Limits>, prefix: u8, number: u8) -> Queue<Request> {
    step(
        core,
        env,
        Event::EffectConnector(connector::Event::Resource {
            project: 1,
            name: name(prefix, number),
            role: connector::ResourceRole::Owned,
            hold: connector::HoldKind::Exclusive { wait: true },
        }),
    )
}

fn delegate(core: &Core) -> Delegate {
    let mut member = new_row(core, 2, tasks::Funder::Task(1), None);
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    Delegate {
        executor: member.executor,
        spec: member.spec,
        contract: member.contract,
        authority: member.authority,
        deadline: member.deadline,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(member.wake),
        tracked: None,
        label: member.label,
        flags: member.flags,
        budget: people::Budget::Named,
    }
}

fn begin(core: &mut Core, env: &Env<Limits>, count: usize) -> Token {
    begin_at(core, env, count, 0)
}

fn begin_at(core: &mut Core, env: &Env<Limits>, count: usize, position: u32) -> Token {
    let mut batch = List::with_capacity(env.limits.tasks.batch);
    for _ in 0..count {
        batch.push(delegate(core)).unwrap();
    }
    let batch = batch.into_boxed();
    let key = CallKey { task: 1, attempt: 1, completion: 1, position };
    let mut out = step(
        core,
        env,
        Event::Delegate {
            to: ReplyTo::new(Token::new(7300_u64.checked_add(u64::from(position)).unwrap())),
            key,
            batch,
        },
    );
    let mut request = None;
    let mut asks = 0_u32;
    while let Some(next) = out.pop() {
        if let Request::Held(ref held) = next {
            panic!("early answer: {held:?}");
        }
        if let Request::Ask { ask: Ask::DelegateHoldings { request: token, .. }, .. } = next {
            request = Some(token);
            asks = asks.checked_add(1).unwrap();
        }
    }
    assert_eq!(asks, 2);
    let request = request.expect("delegation asks holdings");
    let mut holdings = List::with_capacity(env.limits.tasks.batch);
    for _ in 0..count {
        holdings.push(Box::from([])).unwrap();
    }
    drop(step(
        core,
        env,
        Event::DelegateHoldings { request, connector: 2, holdings: Some(holdings.into_boxed()), refused: None },
    ));
    request
}

fn answer(core: &mut Core, env: &Env<Limits>, request: Token, count: u8) -> Queue<Request> {
    answer_names(core, env, request, b'c', count)
}

fn answer_names(core: &mut Core, env: &Env<Limits>, request: Token, prefix: u8, count: u8) -> Queue<Request> {
    let mut holdings = List::with_capacity(env.limits.tasks.batch);
    for number in 1..=count {
        holdings.push(Box::from([tasks::Holding::Write { resource: name(prefix, number), kind: 1 }])).unwrap();
    }
    let holdings = holdings.into_boxed();
    step(core, env, Event::DelegateHoldings { request, connector: 1, holdings: Some(holdings), refused: None })
}

fn call_answer(mut out: Queue<Request>) -> Option<CallPart> {
    let mut answer = None;
    while let Some(next) = out.pop() {
        if let Request::Held(held) = next
            && let Held::CallAnswer { part, .. } = *held
        {
            assert!(answer.replace(part).is_none(), "one answer");
        }
    }
    answer
}

#[test]
fn a_full_report_table_keeps_members_reported_before_the_holdings_answer() {
    let (mut core, env) = delegate_core();
    for number in 1..=8 {
        drop(report(&mut core, &env, b'z', number));
    }
    let request = begin(&mut core, &env, 4);
    for number in 1..=4 {
        drop(report(&mut core, &env, b'c', number));
    }
    let answer = call_answer(answer(&mut core, &env, request, 4));
    let Some(CallPart::Delegated(made)) = answer else {
        panic!("batch answered delegated, got {answer:?}; awaiting {}", core.awaiting_resources_flights());
    };
    assert_eq!(made.len(), 4);
    for number in 2..=5 {
        assert!(core.tasks.task(number).is_some());
    }
    assert_eq!(core.awaiting_resources_flights(), 0);
    assert_eq!(core.tasks.pinned_reports(), 0);
}

#[test]
fn a_waiting_batch_pins_its_reports_through_foreign_reports() {
    let (mut core, env) = delegate_core();
    for number in 1..=8 {
        drop(report(&mut core, &env, b'z', number));
    }
    let request = begin(&mut core, &env, 4);
    for number in 1..=3 {
        drop(report(&mut core, &env, b'c', number));
    }
    assert_eq!(call_answer(answer(&mut core, &env, request, 4)), None);
    assert_eq!(core.awaiting_resources_flights(), 1);
    assert_eq!(core.tasks.pinned_reports(), 4);
    for number in 1..=20 {
        drop(report(&mut core, &env, b'a', number));
    }
    for number in 1..=3 {
        assert!(core.tasks.resource_hold(&name(b'c', number)).is_some());
    }
    let Some(CallPart::Delegated(made)) = call_answer(report(&mut core, &env, b'c', 4)) else {
        panic!("the missing report completes the batch");
    };
    assert_eq!(made.len(), 4);
    assert_eq!(core.awaiting_resources_flights(), 0);
    assert_eq!(core.tasks.pinned_reports(), 0);
}

fn report_capacity(problem: tasks::Problem) {
    assert_eq!(problem.task, None);
    assert_eq!(problem.why, tasks::Refusal::Busy);
    assert_eq!(problem.blocked_by, None);
    assert_eq!(problem.capacity.unwrap().cause, tasks::CapacityCause::ResourceReports { limit: 8 });
}

#[test]
fn a_batch_that_cannot_fit_beside_live_names_is_decided_whole() {
    let (mut core, env) = with_live_names(7);
    drop(report(&mut core, &env, b'z', 8));
    let request = begin(&mut core, &env, 2);
    let Some(CallPart::DelegationRefused(problem)) = call_answer(answer(&mut core, &env, request, 2)) else {
        panic!("report capacity refuses the named call");
    };
    report_capacity(problem);
    for number in 2..=3 {
        assert!(core.tasks.task(number).is_none());
    }
    assert_eq!(core.awaiting_resources_flights(), 0);
    assert_eq!(core.tasks.pinned_reports(), 0);
    let key = CallKey { task: 1, attempt: 1, completion: 1, position: 0 };
    assert!(!core.pending_calls.contains_key(&key));
    assert!(core.call_parts.contains_key(&key));
    let batch = Box::new([delegate(&core)]);
    let retry = step(&mut core, &env, Event::Delegate { to: ReplyTo::new(Token::new(7301)), key, batch });
    assert_eq!(call_answer(retry), core.call_parts.get(&key).cloned());
}

#[test]
fn waiting_batches_share_the_report_bound_without_displacing_each_others_pins() {
    let (mut core, env) = delegate_core();
    let first = begin(&mut core, &env, 4);
    assert_eq!(call_answer(answer(&mut core, &env, first, 4)), None);
    let too_large = begin_at(&mut core, &env, 5, 1);
    let Some(CallPart::DelegationRefused(problem)) = call_answer(answer_names(&mut core, &env, too_large, b'd', 5))
    else {
        panic!("the other batch's pins consume room");
    };
    report_capacity(problem);
    assert_eq!(core.tasks.pinned_reports(), 4);
    let fits = begin_at(&mut core, &env, 4, 2);
    assert_eq!(call_answer(answer_names(&mut core, &env, fits, b'd', 4)), None);
    assert_eq!(core.tasks.pinned_reports(), 8);
    assert_eq!(core.awaiting_resources_flights(), 2);
}

#[test]
fn missing_reports_refuse_at_the_first_parks_deadline_and_release_their_room() {
    let (mut core, mut env) = delegate_core();
    // No sign-in alarm can precede the batch's alarm in this fixture.
    let request = begin(&mut core, &env, 4);
    drop(report(&mut core, &env, b'c', 1));
    drop(report(&mut core, &env, b'c', 3));
    assert_eq!(call_answer(answer(&mut core, &env, request, 4)), None);
    let deadline = env.now.saturating_add(env.limits.tasks.hold_wait);
    assert_eq!(core.next_deadline(env.now, env.wall), Some(deadline));
    env.now = Time::from_nanos(deadline.as_nanos().checked_sub(1).unwrap());
    assert_eq!(call_answer(fire(&mut core, &env, Timer::Resources)), None);
    assert_eq!(core.awaiting_resources_flights(), 1);
    // A report of an already present member does not restart the wait.
    drop(report(&mut core, &env, b'c', 1));
    assert_eq!(core.next_deadline(env.now, env.wall), Some(deadline));
    env.now = deadline;
    assert_eq!(core.due(env.now, env.wall), Some(Timer::Resources));
    assert_eq!(
        call_answer(fire(&mut core, &env, Timer::Resources)),
        Some(CallPart::DelegationRefused(tasks::Problem::naming(Some(3), tasks::Refusal::Unreported, &[3, 5])))
    );
    assert_eq!(core.awaiting_resources_flights(), 0);
    assert_eq!(core.tasks.pinned_reports(), 0);
    for number in 2..=5 {
        assert!(core.tasks.task(number).is_none());
    }
    let key = CallKey { task: 1, attempt: 1, completion: 1, position: 0 };
    assert!(!core.pending_calls.contains_key(&key));
    assert!(core.call_parts.contains_key(&key));
    let next = begin_at(&mut core, &env, 8, 1);
    assert_eq!(call_answer(answer_names(&mut core, &env, next, b'd', 8)), None);
    assert_eq!(core.tasks.pinned_reports(), 8);
}

#[test]
fn a_childs_funding_refusal_releases_a_pinned_batch() {
    let (mut core, env) = delegate_core();
    let request = begin(&mut core, &env, 4);
    assert_eq!(call_answer(answer(&mut core, &env, request, 4)), None);
    assert_eq!(core.tasks.pinned_reports(), 4);
    let mut other = new_row(&core, 6, tasks::Funder::Task(1), None);
    other.numbers.budget = 90;
    other.authority.budget.spend = 90;
    fixtures::make_batch(&mut core, &env.limits, tasks::Party::Task(1), Box::new([other]));
    assert!(core.tasks.task(6).is_some());
    for number in 1..=3 {
        drop(report(&mut core, &env, b'c', number));
    }
    let Some(CallPart::DelegationRefused(problem)) = call_answer(report(&mut core, &env, b'c', 4)) else {
        panic!("the tasks child refuses unfunded work");
    };
    assert_eq!(problem.why, tasks::Refusal::Funding);
    assert_eq!(core.tasks.pinned_reports(), 0);
    assert_eq!(core.awaiting_resources_flights(), 0);
}

#[test]
fn one_report_decides_every_waiting_batch_even_when_the_first_is_refused() {
    let (mut core, env) = delegate_core();
    let first = begin(&mut core, &env, 1);
    assert_eq!(call_answer(answer(&mut core, &env, first, 1)), None);
    let second = begin_at(&mut core, &env, 1, 1);
    assert_eq!(call_answer(answer(&mut core, &env, second, 1)), None);
    assert_eq!(core.awaiting_resources_flights(), 2);
    let mut out = step(
        &mut core,
        &env,
        Event::EffectConnector(connector::Event::Resource {
            project: 1,
            name: name(b'c', 1),
            role: connector::ResourceRole::Unavailable,
            hold: connector::HoldKind::Exclusive { wait: true },
        }),
    );
    let mut refused = List::with_capacity(2);
    while let Some(next) = out.pop() {
        if let Request::Held(held) = next
            && let Held::CallAnswer { part: CallPart::DelegationRefused(problem), .. } = *held
        {
            assert_eq!(problem.why, tasks::Refusal::ResourceUnavailable);
            refused.push(problem.task.expect("refused member")).unwrap();
        }
    }
    assert_eq!(refused.as_slice(), &[2, 3]);
    assert_eq!(core.awaiting_resources_flights(), 0);
    assert_eq!(core.tasks.pinned_reports(), 0);
}

#[test]
fn overlapping_pins_count_names_once_and_retries_and_releases_are_idempotent() {
    let (mut core, env) = with_live_names(7);
    let mut first = new_row(&core, 2, tasks::Funder::Task(1), None);
    first.holdings = Box::new([
        tasks::Holding::Write { resource: name(b'z', 1), kind: 1 },
        tasks::Holding::Write { resource: name(b'c', 1), kind: 1 },
    ]);
    let mut second = new_row(&core, 3, tasks::Funder::Task(1), None);
    second.holdings = Box::new([tasks::Holding::Slot { pool: name(b'c', 1), kind: 2 }]);
    let batch = [first, second];
    let owner = Token::new(7000);
    let other = Token::new(7001);
    assert_eq!(core.tasks.pin_reports(owner, &batch), Ok(()));
    assert_eq!(core.tasks.pin_reports(owner, &batch), Ok(()));
    assert_eq!(core.tasks.pin_reports(other, &batch), Ok(()));
    assert_eq!(core.tasks.pinned_reports(), 2);
    drop(report(&mut core, &env, b'c', 1));
    for number in 1..=9 {
        drop(report(&mut core, &env, b'a', number));
    }
    assert!(core.tasks.resource_hold(&name(b'c', 1)).is_some());
    core.tasks.unpin_reports(owner);
    core.tasks.unpin_reports(owner);
    assert_eq!(core.tasks.pinned_reports(), 2);
    core.tasks.unpin_reports(other);
    core.tasks.unpin_reports(other);
    assert_eq!(core.tasks.pinned_reports(), 0);
}

#[test]
fn reporting_a_name_again_refreshes_its_eviction_order() {
    let (mut core, env) = delegate_core();
    drop(report(&mut core, &env, b'c', 1));
    for number in 1..=7 {
        drop(report(&mut core, &env, b'z', number));
    }
    drop(report(&mut core, &env, b'c', 1));
    drop(report(&mut core, &env, b'a', 1));
    assert!(core.tasks.resource_hold(&name(b'c', 1)).is_some());
    assert!(core.tasks.resource_hold(&name(b'z', 1)).is_none());
}
