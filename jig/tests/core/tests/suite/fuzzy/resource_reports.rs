//! Every creation call is decided within its report wait, even over a full
//! table and interleaved connector traffic (jig/docs/design/domain/tasks/holds.md, 2).
use jig_core_world::{
    faults::plan_fixture,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_core::engine::{self, Admit};
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Rng, Token, Wall};

fn member(authority: &tasks::Authority, resource: u64) -> core::Delegate {
    let mut authority = authority.clone();
    authority.budget.spend = 10;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    core::Delegate {
        deadline: tasks::Deadline::Named,
        label: 0,
        flags: tasks::Flags::PLAIN,
        tracked: None,
        budget: jig_domain_people::Budget::Named,
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"resource work".as_slice().into(),
            parameters: Box::new([tasks::Parameter::Resource { name: 1, connector: 1, resource }]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 128 },
        authority,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(tasks::WakePolicy::DEFAULT),
    }
}

fn delegate(world: &mut World, parent: (u64, u64), completion: u32, members: Vec<core::Delegate>) {
    world.send(root::Event::Core(core::Event::Delegate {
        to: ReplyTo::new(Token::new(9000 + u64::from(completion))),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion, position: 0 },
        batch: members.into_boxed_slice(),
    }));
}

fn foreign(world: &mut World, number: u64) {
    world.send(root::Event::Core(core::Event::EffectConnector(core::effects::connector::Event::Resource {
        project: 1,
        name: tasks::Name { connector: 1, path: Box::new([Box::from(b"z".as_slice()), number.to_be_bytes().into()]) },
        role: core::effects::connector::ResourceRole::Owned,
        hold: core::effects::connector::HoldKind::Exclusive { wait: true },
    })));
}

fn world(seed: u64, capacity: u32, bound: u64) -> (World, root::Limits) {
    let (mut config, mut limits) = plan_fixture(seed);
    limits.core.tasks.resource_reports = capacity;
    limits.core.tasks.hold_wait = Duration::from_secs(1);
    limits.core.tasks.tasks = 32;
    limits.core.tasks.project_tasks = 32;
    limits.core.tasks.tree_tasks = 32;
    limits.core.tasks.delegates = 16;
    limits.core.tasks.batch = u32::try_from(bound).expect("small batch bound");
    limits.core.authority.batch = u32::try_from(bound).expect("small batch bound");
    limits.core.tasks.inbox_messages = 32;
    limits.core.tasks.inbox_bytes = 4096;
    limits.core.views.runs = 32;
    limits.core.fleet.attempts = 32;
    limits.connector.resources = 32;
    config.core.defaults.rules.ceiling.delegation.tasks = 32;
    config.core.seed_projects[0].policy.policy.ceiling.delegation.tasks = 32;
    config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.tasks = 16;
    config.first.resources = (1..=12)
        .map(|number| connector::ResourceSpec {
            credential: None,
            path: jig_test_connector_kit::path(1, number),
            hold: connector::Hold::Exclusive { mode: connector::HoldMode::Wait },
            writable: true,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    config.first.pools = Box::new([]);
    let world = World::new(Setup { seed, configuration: Some(config), limits, ..Setup::default() });
    (world, limits)
}

// A connector emits one report burst before its holdings answer. Interleaving
// begins after that answer pins the batch; delayed reports can then arrive singly.
fn holdings_burst(
    world: &mut World,
    rng: &mut Rng,
    first: u64,
    count: u64,
    missing: &mut Vec<u64>,
) -> Vec<(u64, core::Event)> {
    let mut delayed = Vec::new();
    let mut events = Vec::new();
    let mut member_at = 0_u64;
    while let Some(event) = world.domain.pop_holdings() {
        if matches!(event, core::Event::EffectConnector(core::effects::connector::Event::Resource { .. })) {
            if rng.below(3) == 0 {
                missing.push(first + member_at);
                delayed.push((first + member_at, event));
            } else {
                events.push(root::Event::Core(event));
            }
            member_at += 1;
        } else {
            events.push(root::Event::Core(event));
        }
    }
    assert_eq!(member_at, count);
    world.send_all(events);
    delayed
}

fn outcome(world: &World, seed: u64, first: u64, count: u64, live: u64, capacity: u32, missing: &[u64]) -> usize {
    match &world.answers.last().expect("batch decided").2 {
        core::CallPart::Delegated(made) => {
            assert_eq!(missing, &[] as &[u64]);
            assert_eq!(made.len(), usize::try_from(count).expect("small batch"));
            for member in made {
                assert!(
                    world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(member.task))))
                );
            }
            0
        }
        core::CallPart::DelegationRefused(problem) => {
            let outcome = match problem.why {
                tasks::Refusal::Busy => {
                    assert_eq!(problem.task, None);
                    assert_eq!(
                        problem.capacity.expect("capacity refusal has its bound").cause,
                        tasks::CapacityCause::ResourceReports { limit: capacity }
                    );
                    assert!(live + count > u64::from(capacity));
                    1
                }
                tasks::Refusal::Unreported => {
                    assert_ne!(missing, &[] as &[u64]);
                    assert_eq!(problem.task, missing.first().copied());
                    assert_eq!(problem.blocked_by.as_deref(), Some(missing));
                    assert_eq!(problem.capacity, None);
                    2
                }
                why => panic!("seed {seed}: unexpected refusal {why:?}"),
            };
            for number in first..first + count {
                assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(number)))));
            }
            outcome
        }
        answer => panic!("seed {seed}: unexpected answer {answer:?}"),
    }
}

#[test]
fn full_tables_and_interleaved_reports_never_leave_a_call_past_its_deadline() {
    let mut outcomes = [0_u32; 3];
    for seed in 301..=316 {
        eprintln!("resource reports seed {seed}");
        let mut rng = Rng::new(seed);
        let capacity = u32::try_from(2 + rng.below(7)).unwrap();
        let live = rng.below(u64::from(capacity.min(4)));
        let bound = 1 + rng.below(4);
        let count = 1 + rng.below(bound);
        let (mut world, limits) = world(seed, capacity, bound);
        let (now, wall) = world.clocks();
        world.set_clock(now, wall);
        let parent = world.assignments[0];
        let authority = world.assigned.as_ref().unwrap().run.authority.clone();
        for number in 1..=12 {
            world.send(root::Event::Connector {
                number: 1,
                event: connector::Event::Adopt {
                    project: 1,
                    resource: jig_test_connector_kit::path(1, number),
                    role: connector::ResourceRole::Owned,
                },
            });
        }
        // Separate live tasks keep a random portion of the table before it fills.
        for resource in 1..=live {
            delegate(&mut world, parent, u32::try_from(resource).unwrap(), vec![member(&authority, resource)]);
            assert!(matches!(world.answers.last().unwrap().2, core::CallPart::Delegated(_)));
        }
        for number in 1..=u64::from(capacity) {
            foreign(&mut world, number);
        }
        world.domain.script_holdings();
        let batch = (1..=count).map(|at| member(&authority, live + at)).collect();
        let first = parent.0 + live + 1;
        let completion = u32::try_from(live + 1).unwrap();
        let answers = world.answers.len();
        delegate(&mut world, parent, completion, batch);
        let mut missing = Vec::new();
        let parked = world.clocks().0;
        let delayed = holdings_burst(&mut world, &mut rng, first, count, &mut missing);
        let (now, wall) = world.clocks();
        let deadline = if world.domain.core().awaiting_resources_flights() == 1 {
            let deadline = parked.saturating_add(limits.core.tasks.hold_wait);
            assert!(
                engine::next_deadline(world.domain.engine(), now, wall, Admit::All).expect("report deadline armed")
                    <= deadline
            );
            for (number, event) in delayed {
                // More than a full table's foreign reports cannot evict these pins.
                for at in 0..capacity + u32::try_from(rng.below(3)).unwrap() {
                    foreign(&mut world, 1000 + u64::from(at));
                }
                let (now, wall) = world.clocks();
                let by = Duration::from_nanos(rng.below(1_000_000_001));
                world.fire_due(now.saturating_add(by), Wall::from_nanos(wall.as_nanos() + by.as_nanos()));
                if world.answers.len() == answers && rng.below(2) == 0 {
                    world.send(root::Event::Core(event));
                    missing.retain(|missing| *missing != number);
                }
            }
            if world.answers.len() == answers {
                let (now, wall) = world.clocks();
                let at = deadline.max(now).saturating_add(Duration::from_nanos(rng.below(10_000_000)));
                world.fire_due(at, Wall::from_nanos(wall.as_nanos() + at.as_nanos() - now.as_nanos()));
            }
            deadline
        } else {
            now
        };
        assert_eq!(world.domain.core().awaiting_resources_flights(), 0, "seed {seed}, deadline {deadline:?}");
        assert_eq!(world.answers.len(), answers + 1, "seed {seed}: every call answers");
        let outcome = outcome(&world, seed, first, count, live, capacity, &missing);
        outcomes[outcome] += 1;
    }
    assert!(outcomes.iter().all(|count| *count > 0), "all terminal outcomes exercised: {outcomes:?}");
}
