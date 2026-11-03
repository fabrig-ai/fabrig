//! Fixtures that drive the core world (`jig/docs/design/domain/testing.md`, 2).
use crate::world::Setup;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use jig_test_profile::fixtures::{CREDENTIAL_RESOURCE, Creating, Goal, account_rows, create_chat, people_member};
use skein_lib::{ReplyTo, Token};

#[must_use]
pub fn live_row(world: &crate::world::World, task: u64) -> &jig_domain_tasks::TaskRecord {
    match world
        .store
        .rows
        .get(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(task))))
    {
        Some(jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(row)))) => row,
        row => panic!("live task row missing: {row:?}"),
    }
}

/// A delegate result and one standing state subscription each reserve a result payload.
#[must_use]
pub fn notices_world(seed: u64) -> crate::world::World {
    let (config, mut limits) = crate::world::fixture(seed, false);
    limits.core.tasks.inbox_bytes = 256;
    crate::world::World::new(Setup { seed, configuration: Some(config), limits, ..Setup::default() })
}

/// Task numbers in a live query page, retaining its requested order.
#[must_use]
pub fn query_numbers(world: &crate::world::World) -> Vec<u64> {
    let jig_domain_core::reads::Answer::Tasks(page) = crate::reads::latest(world) else { panic!("query page") };
    page.live.iter().map(|line| line.chip.number).collect()
}

const HOLDER_REQUEST_BASE: u64 = 8200;
/// Submit an owner or member request with the fixture's request-token range.
pub fn ask_as(world: &mut crate::world::World, sign_in: u64, key: u8, ask: jig_domain_people::Ask) {
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(jig_domain_people::Event::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(HOLDER_REQUEST_BASE + u64::from(key))),
        sign_in,
        key: [key; 16],
        ask,
    })));
}

#[must_use]
pub fn member(world: &crate::world::World, executor: tasks::Executor, words: &[u8]) -> core::Delegate {
    let mut authority = world.assigned.as_ref().expect("task assigned").run.authority.clone();
    authority.budget.spend = 20;
    authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    core::Delegate {
        deadline: jig_domain_tasks::Deadline::Named,
        label: 0,
        flags: jig_domain_tasks::Flags::PLAIN,
        tracked: None,
        budget: jig_domain_people::Budget::Named,
        executor,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: words.into(),
            parameters: Box::new([]),
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

/// A running caller with the supplied authority wall, for delegation deadline stories.
#[must_use]
pub fn run_with_deadline(seed: u64, deadline: Option<skein_lib::Wall>) -> crate::world::World {
    let (config, limits) = crate::faults::plan_fixture(seed);
    let people::Ask::Create { project, mut batch } =
        create_chat(&Creating::new(&config.core, limits.core.tasks.tree_tasks), 1, 0, b"caller".as_slice().into())
    else {
        panic!("caller creation")
    };
    batch[0].authority.deadline = deadline;
    crate::world::World::new(Setup {
        seed,
        configuration: Some(config),
        limits,
        require_assignment: false,
        initial: Some(people::Ask::Create { project, batch }),
        ..Setup::default()
    })
}

/// Assert the last party answer carries this refusal.
pub fn refusal(world: &crate::world::World, why: jig_domain_people::Refusal) {
    use jig_domain_people as people;
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Refused(why))));
}

/// Finish the assigned chat with a report.
pub fn finish_chat(world: &mut crate::world::World, cumulative: u64, words: &[u8]) {
    world.end(
        tasks::End::Finished { result: tasks::TaskResult::Report { words: words.into() }, cancel_delegates: false },
        cumulative,
    );
}

pub fn delegate(world: &mut crate::world::World, run: (u64, u64), completion: u32, member: core::Delegate) -> u64 {
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9000 + u64::from(completion))),
        key: core::CallKey { task: run.0, attempt: run.1, completion, position: 0 },
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    let core::CallPart::Delegated(numbers) = &world.answers.last().expect("delegate answer").2 else {
        panic!("delegation was not admitted: {:?}", world.answers.last());
    };
    numbers[0].task
}

#[must_use]
pub fn two_projects(seed: u64) -> (root::Config, root::Limits) {
    let mut config = jig_test_profile::config(seed);
    let mut limits = jig_test_profile::limits();
    limits.core.authority.projects = 2;
    limits.core.tasks.funders = 8;
    // Both configured projects and the deployment's roles.
    limits.core.people.projects = 3;
    limits.core.people.initial_owners = 2;
    crate::peers::journal(&mut limits);
    let policy = config.core.seed_projects[0].policy.policy.clone();
    config.core.defaults.rules = (authority::Domain::new(config.core.defaults.rules.clone(), limits.core.authority)
        .expect("two projects"))
    .rules()
    .clone();
    config.core.seed_projects = [1, 2]
        .into_iter()
        .map(|number| {
            let mut policy = policy.clone();
            policy.period_spend = if number == 1 { 1000 } else { 700 };
            core::SeedProject {
                number,
                name: format!("Project {number}").into_boxed_str(),
                policy: core::ConfiguredPolicy {
                    policy,
                    permissions: if number == 1 {
                        config.core.seed_projects[0].policy.permissions.clone()
                    } else {
                        Box::new([])
                    },
                    templates: Box::new([]),
                },
            }
        })
        .collect();
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        people::InitialOwner { project: 2, identity: config.core.owners[0].identity.clone() },
    ]);
    (config, limits)
}

const FIXTURE_INBOX_REQUEST: u64 = 8301;
const FIXTURE_SIGN_IN_REQUEST: u64 = 8300;
const FIXTURE_PROJECT: u32 = 1;

/// Read every page of one party inbox group.
pub fn inbox_objects(
    world: &mut crate::world::World,
    sign_in: u64,
    group: jig_domain_core::reads::Group,
) -> Vec<jig_domain_core::documents::InboxObject> {
    use jig_domain_core::reads;
    world.sign_in = Some(sign_in);
    let mut from = None;
    let mut objects = Vec::new();
    for page_number in 0..32 {
        crate::reads::ask(
            world,
            FIXTURE_INBOX_REQUEST + page_number,
            reads::Read::Inbox {
                group,
                filter: reads::Filter { project: Some(FIXTURE_PROJECT), goal: None },
                from: from.take(),
            },
        );
        let reads::Answer::Page(page) = crate::reads::latest(world) else { panic!("inbox page") };
        for row in &page.rows {
            let reads::PageRow::Inbox(entry) = row else { panic!("inbox entry") };
            objects.push(entry.object);
        }
        from.clone_from(&page.next);
        if from.is_none() {
            return objects;
        }
    }
    panic!("bounded inbox pages")
}

/// Sign in a second identity and give it a role, preserving the other holdings.
pub fn second_party(world: &mut crate::world::World, subject: &[u8], role: jig_domain_people::Role) -> u64 {
    use jig_domain_people as people;
    let mut holdings = match world
        .store
        .rows
        .get(&jig_test_domain::Key::Core(jig_domain_core::Key::People(people::Key::Roles(FIXTURE_PROJECT))))
    {
        Some(jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
            holdings,
            ..
        }))) => holdings.to_vec(),
        value => panic!("project roles: {value:?}"),
    };
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::SignIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(FIXTURE_SIGN_IN_REQUEST)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: subject.into() },
            login: b"second".as_slice().into(),
            name: b"Second".as_slice().into(),
        },
    }));
    let Some(people::Reply::SignedIn { sign_in, person, .. }) = world.people_answers.last().cloned() else {
        panic!("second party signed in")
    };
    holdings.retain(|holding| holding.person != person);
    holdings.push(people::Holding { person, role });
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles {
        project: FIXTURE_PROJECT,
        holdings: holdings.into_boxed_slice(),
    })));
    sign_in
}

/// A real admitted hold spans four segments while short document lists hold two.
#[must_use]
pub fn long_resource_world(seed: u64) -> (crate::world::World, u64, jig_test_connector::Path) {
    let (mut configuration, mut limits) = crate::world::fixture(seed, false);
    limits.core.documents.lines = 2;
    limits.core.fleet.slots = 2;
    limits.connector.path_segments = limits.core.tasks.hold_segments;
    let mut path = skein_lib::List::with_capacity(limits.core.tasks.hold_segments);
    for segment in jig_test_connector_world::path(1, 1).segments() {
        path.push(segment.clone()).expect("prefix fits");
    }
    for segment in [b"third".as_slice(), b"last".as_slice()] {
        path.push(segment.into()).expect("four admitted segments");
    }
    let path = jig_test_connector::Path::new(path, 128).expect("bounded resource");
    configuration.first.resources[0].path = path.clone();
    crate::peers::journal(&mut limits);
    let mut world =
        crate::world::World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    world.delegate_on_path(CREDENTIAL_RESOURCE, path.clone());
    let core::CallPart::Delegated(numbers) = &world.answers.last().expect("delegation answer").2 else {
        panic!("resource delegate admitted: {:?}", world.answers.last())
    };
    let child = numbers[0].task;
    world.reads = Some(crate::reads::Parties::new(world.store.applied));
    (world, child, path)
}

#[must_use]
pub fn chat(world: &crate::world::World, project: u32, words: Box<[u8]>) -> people::Ask {
    chat_for(world, project, 0, words)
}

#[must_use]
pub fn goal(
    world: &crate::world::World,
    project: u32,
    words: Box<[u8]>,
    executor: people::TaskExecutor,
    terms: Goal,
) -> people::Ask {
    let people::Ask::Create { mut batch, .. } = chat(world, project, words) else { unreachable!() };
    let member = &mut batch[0];
    member.executor = executor;
    member.budget = people::Budget::Named;
    member.authority.spend = terms.budget;
    member.authority.deadline = terms.deadline;
    member.tracked = Some(terms.priority);
    member.flags.conversation = false;
    member.flags.listed = true;
    member.label = jig_client_test_records::GOAL;
    if matches!(executor, people::TaskExecutor::Procedure { .. }) {
        member.authority.tools = 0;
    }
    people::Ask::Create { project, batch }
}

#[must_use]
pub fn chat_for(world: &crate::world::World, project: u32, role: u32, words: Box<[u8]>) -> people::Ask {
    let people::Ask::Create { mut batch, .. } = create_chat(&world.creating, project, role, words) else {
        unreachable!()
    };
    if let Some(root::Record::Core(core::Record::People(people::Stored::Policy { value, .. }))) =
        world.store.rows.get(&root::Key::Core(core::Key::People(people::Key::Policy(project))))
        && let Some(current) = value.roles.iter().find(|value| value.number == role)
    {
        let task_cap = world.creating.tasks;
        batch[0].authority = current.authority.clone();
        batch[0].authority.spend = 0;
        batch[0].authority.delegation.tasks = current.authority.delegation.tasks.min(task_cap).saturating_sub(1);
        batch[0].authority.delegation.depth = current.authority.delegation.depth.saturating_sub(1);
    }
    people::Ask::Create { project, batch }
}

pub mod deadlines;

/// A lower-world story supplies the exact durable source it will grant later.
#[must_use]
pub fn waiting_on_source(seed: u64, source: u32) -> crate::world::World {
    let (config, limits) = crate::world::fixture(seed, false);
    let mut rows = account_rows(&config.core.settings.llm_providers).into_vec();
    for row in &mut rows {
        match row {
            root::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Source { number, .. })) => {
                *number = source;
            }
            root::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Next { number })) => {
                *number = source + 1;
            }
            root::Record::Core(_) | root::Record::Connector { .. } => {}
        }
    }
    configured_sources(seed, config, limits, rows.into_boxed_slice())
}
#[must_use]
pub fn configured_sources(
    seed: u64,
    config: root::Config,
    limits: root::Limits,
    rows: Box<[root::Record]>,
) -> crate::world::World {
    let initial = create_chat(
        &Creating::new(&config.core, limits.core.tasks.tree_tasks),
        1,
        0,
        b"make an effect".as_slice().into(),
    );
    crate::world::World::new(Setup {
        seed,
        configuration: Some(config),
        limits,
        require_assignment: false,
        initial: Some(initial),
        rows: Some(rows),
        ..Setup::default()
    })
}

/// Restore project policies from a world's durable rows before its tasks.
pub fn restore_project_policies(core: &mut core::Core, limits: &core::Limits, store: &crate::Store) {
    for row in store.rows.values() {
        if let root::Record::Core(core::Record::People(people::Stored::Policy { project, name, value })) = row {
            assert_eq!(
                core.restore(
                    &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: *limits },
                    core::Record::People(people::Stored::Policy {
                        project: *project,
                        name: name.clone(),
                        value: value.clone()
                    }),
                ),
                core::Restore::Kept,
                "durable policy restores"
            );
        }
    }
}

/// A delegate that writes one of the resource fixture's holds.
#[must_use]
pub fn resource_member(world: &crate::world::World, id: u64, words: &[u8]) -> core::Delegate {
    let mut child = member(world, tasks::Executor::Agent { charter: 1 }, words);
    child.spec.parameters = Box::new([tasks::Parameter::Resource { name: 1, connector: 1, resource: id }]);
    child.authority.budget.spend = 10;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    child
}

/// Adopt the exclusive and pooled resources used by writer and hold stories.
pub fn configure_resources(world: &mut crate::world::World) {
    for id in [1, 2] {
        world.send(root::Event::Connector {
            number: 1,
            event: jig_test_connector::Event::Adopt {
                project: 1,
                resource: jig_test_connector_world::path(1, id),
                role: jig_test_connector::ResourceRole::Owned,
            },
        });
    }
    world.configure_holds(
        1,
        Box::new([
            tasks::Kind { connector: 1, kind: 1, hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits } },
            tasks::Kind { connector: 1, kind: 2, hold: tasks::HoldKind::Pooled { taken: tasks::Taken::Waits } },
        ]),
    );
}

/// A procedure with one declared job kind (jig/docs/design/domain/engine.md, 7.5).
#[must_use]
pub fn job_world(seed: u64, ceiling: bool) -> (crate::world::World, u64) {
    let (mut configuration, mut limits) = crate::faults::plan_fixture(seed);
    limits.core.fleet.workers = 3;
    configuration.core.worker_names = Box::new([
        b"caller".as_slice().into(),
        b"first job worker".as_slice().into(),
        b"second job worker".as_slice().into(),
    ]);
    limits.core.authority.executors = 3;
    limits.core.tasks.executor_kinds = 3;
    limits.core.documents.executors = 3;
    configuration.core.jobs = Box::new([jig_domain_host::JobBound {
        kind: jig_domain_host::JobKind(7),
        spec: 64,
        result: 32,
        deadline: skein_lib::Duration::from_secs(60),
    }]);
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1), authority::Executor::Job(7)]);
    configuration.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut configuration.core.seed_projects[0].policy.policy;
    if ceiling {
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        policy.roles[0].authority.delegation.kinds = kinds;
    }
    crate::peers::journal(&mut limits);
    let mut world =
        crate::world::World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    world.hold_procedures = true;
    let run = world.assignments[0];
    let mut procedure = member(&world, tasks::Executor::Procedure { connector: 1, code: 1 }, b"procedure");
    if ceiling {
        procedure.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Job(7)]);
    } else {
        procedure.authority.delegation.kinds = Box::new([]);
    }
    let task = delegate(&mut world, run, 1, procedure);
    (world, task)
}
