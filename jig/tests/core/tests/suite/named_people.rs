//! Outside person names must link at the next start (jig/docs/design/domain/engine.md, 5.1 and 6).
use jig_core_world::fixtures::{ask, live_row};
use jig_core_world::world::{Setup, World};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const SEED: u64 = 29_501;
const PROJECT: u32 = 1;
const PERSON: u64 = 1;
const PERSON_TASK: u64 = 2;
const DELEGATE: u64 = 3;
const ROLE: u32 = 0;

fn configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_test_profile::effect_fixture(SEED, false);
    // The fixture offers a charter, a procedure and the owner's person role.
    // One root, the live person task used as an absent person, and the tested delegate.
    limits.core.tasks.tasks = 3;
    limits.core.tasks.project_tasks = 3;
    limits.core.tasks.executor_kinds = 3;
    limits.core.authority.executors = 3;
    limits.core.documents.executors = 3;
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1), authority::Executor::Role(ROLE)]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    (config, limits)
}

fn world() -> World {
    let (config, limits) = configuration();
    let mut world = World::new(Setup { seed: SEED, configuration: Some(config), limits, ..Setup::default() });
    let mut member = jig_test_profile::fixtures::people_member(b"check the answer", 0);
    member.executor = people::TaskExecutor::Person(people::PersonAddress::Role(ROLE));
    ask(&mut world, 51, people::Ask::Create { project: PROJECT, batch: Box::new([member]) });
    world
}

fn amendment(person: u64) -> people::Amendment {
    people::Amendment {
        addressed: Some(people::AddressChange { revision: 0, address: people::PersonAddress::Person(person) }),
        tracked: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"name the person".as_slice().into(),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    Person(people::Refusal),
    Task(tasks::Problem),
    Unexpected(String),
}

fn amend(world: &mut World, person: u64) -> Option<Refusal> {
    let revision = live_row(world, PERSON_TASK).revisions.spec;
    ask(
        world,
        52,
        people::Ask::Amend {
            project: PROJECT,
            task: PERSON_TASK,
            revision,
            amendment: amendment(person),
            release: None,
        },
    );
    match world.people_answers.last() {
        Some(people::Reply::Outcome(people::Outcome::Amended { .. })) => None,
        Some(people::Reply::Outcome(people::Outcome::Refused(why))) => Some(Refusal::Person(why.clone())),
        other => Some(Refusal::Unexpected(format!("{other:?}"))),
    }
}

fn agent(world: &mut World, person: u64) -> Option<Refusal> {
    let mut member = jig_core_world::fixtures::member(
        world,
        tasks::Executor::Person(tasks::PersonAddress::Person(person)),
        b"ask the person",
    );
    member.authority.tools = tasks::Tools(0);
    member.authority.models = tasks::Models::NONE;
    member.authority.budget.spend = 0;
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9052)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    match world.answers.last().map(|answer| &answer.2) {
        Some(core::CallPart::Delegated(_)) => None,
        Some(core::CallPart::DelegationRefused(problem)) => Some(Refusal::Task(problem.clone())),
        other => Some(Refusal::Unexpected(format!("{other:?}"))),
    }
}

// The application has no caller for raw task batches. These rows drive the
// core event through its root; admission's output is checked in the core's
// named-people unit table because the root has no raw-batch reply fixture.
fn raw_batch(world: &mut World, person: u64, field: NamedLink) -> Option<Refusal> {
    let number = core::fresh(&mut world.domain.core_mut().counters, core::Family::Task).expect("task number");
    let source = live_row(world, PERSON_TASK);
    let mut row = tasks::New {
        number,
        project: PROJECT,
        label: 0,
        flags: tasks::Flags::PLAIN,
        executor: tasks::Executor::Person(tasks::PersonAddress::Role(ROLE)),
        llm: None,
        spec: source.spec.clone(),
        contract: source.contract.clone(),
        authority: source.authority.clone(),
        deadline: tasks::Deadline::Named,
        numbers: tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: PROJECT, period: 1 },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    };
    let mut creator = tasks::Party::Deployment { project: PROJECT };
    match field {
        NamedLink::Executor => row.executor = tasks::Executor::Person(tasks::PersonAddress::Person(person)),
        NamedLink::Funder => row.funder = tasks::Funder::Pool { project: PROJECT, person, period: 1 },
        NamedLink::Requester => creator = tasks::Party::Person(person),
    }
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(Token::new(9053)),
        creator,
        batch: Box::new([row]),
    })));
    None
}

#[derive(Clone, Copy)]
enum NamedLink {
    Executor,
    Funder,
    Requester,
}

fn executor_batch(world: &mut World, person: u64) -> Option<Refusal> {
    raw_batch(world, person, NamedLink::Executor)
}
fn funder_batch(world: &mut World, person: u64) -> Option<Refusal> {
    raw_batch(world, person, NamedLink::Funder)
}
fn requester_batch(world: &mut World, person: u64) -> Option<Refusal> {
    raw_batch(world, person, NamedLink::Requester)
}

fn linked(row: &tasks::TaskRecord, named: u64, field: NamedLink) -> bool {
    match field {
        NamedLink::Executor => row.executor == tasks::Executor::Person(tasks::PersonAddress::Person(named)),
        NamedLink::Funder => row.funder == tasks::Funder::Pool { project: PROJECT, person: named, period: 1 },
        NamedLink::Requester => row.requester == tasks::Party::Person(named),
    }
}

fn task_rows(world: &World) -> Vec<tasks::Stored> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(row)) => Some(row.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_person_named_from_outside_is_held_or_refused_and_the_start_links() {
    type Input = fn(&mut World, u64) -> Option<Refusal>;
    let routes: [(&str, Input, u64, Option<Refusal>, NamedLink); 5] = [
        ("party amendment", amend, PERSON_TASK, Some(Refusal::Person(people::Refusal::Unknown)), NamedLink::Executor),
        ("agent person task", agent, DELEGATE, Some(Refusal::Task(tasks::Problem::new(None, tasks::Refusal::Unknown))), NamedLink::Executor),
        ("connector batch executor", executor_batch, DELEGATE, None, NamedLink::Executor),
        ("connector batch funder", funder_batch, DELEGATE, None, NamedLink::Funder),
        ("connector batch requester", requester_batch, DELEGATE, None, NamedLink::Requester),
    ];
    let mut failures = Vec::new();
    for (route, input, task, refusal, field) in routes {
        for (named, held) in [(PERSON, true), (PERSON_TASK, false)] {
            let result = std::panic::catch_unwind(|| {
            let mut world = world();
            let before = task_rows(&world);
            let answer = input(&mut world, named);
            let after = task_rows(&world);
            world.restart_with(configuration().0);
            assert_eq!(world.domain.start_failure(), None, "{route}, named {named}");
            assert!(world.domain.ready(), "{route}, named {named}: {:?}", world.domain.start_phase());
            assert_eq!(world.domain.start_phase(), None, "{route} completed Open, named {named}");
            if held {
                assert_eq!(answer, None, "{route}");
                assert!(linked(live_row(&world, task), named, field), "{route}: named {named}");
            } else {
                assert_eq!(answer, refusal, "{route}");
                assert_eq!(after, before, "{route} changes nothing");
            }
            assert!(
                !world.store.rows.values().any(|row| matches!(row,
                    root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))
                        if linked(row, PERSON_TASK, field)
                )),
                "{route} saves no absent person"
            );
            });
            if result.is_err() { failures.push((route, named)); }
        }
    }
    assert!(failures.is_empty(), "failed rows: {failures:?}");
}
