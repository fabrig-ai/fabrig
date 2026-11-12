use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core as core;
use jig_domain_link::Hello;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn two_charter_world(seed: u64) -> World {
    let (configuration, mut limits) = jig_test_profile::two_charters(seed);
    limits.core.tasks.tasks = 3;
    limits.core.tasks.project_tasks = 3;
    World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() })
}

fn task_records(world: &World) -> Vec<tasks::Stored> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            root::Record::Core(core::Record::Tasks(record)) => Some(record.clone()),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect()
}

fn research_choice() -> tasks::LlmChoice {
    tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model: 1, effort: Some(0) },
        subagents: Box::new([]),
    }
}

fn research_on_first_charter(world: &World) -> core::Delegate {
    let mut member = jig_core_world::fixtures::member(world, tasks::Executor::Agent { charter: 1 }, b"research");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    member.llm = Some(research_choice());
    member
}

fn research_step(world: &World, task: u64) -> core::ProcedureStep {
    let mut member = research_on_first_charter(world);
    member.authority = jig_core_world::fixtures::live_row(world, task).authority.clone();
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    core::ProcedureStep { batch: Box::new([member]), ..core::ProcedureStep::new(core::StepEnd::Wait) }
}

#[test]
fn a_runs_delegate_is_refused_a_model_outside_its_charter() {
    let mut world = two_charter_world(158);
    let before = task_records(&world);
    let member = research_on_first_charter(&world);
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9001)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    assert!(matches!(
        &world.answers.last().expect("delegation answered").2,
        core::CallPart::DelegationRefused(tasks::Problem {
            why: tasks::Refusal::Llm { model: 1, why: tasks::LlmFault::Charter },
            ..
        })
    ));
    assert_eq!(task_records(&world), before, "the refused delegate writes no task records");
}

#[test]
fn a_partys_create_is_refused_a_model_outside_its_charter() {
    let mut world = two_charter_world(159);
    let before = task_records(&world);
    let mut member = jig_test_profile::fixtures::people_member(b"research", 10);
    member.llm = Some(people::LlmChoice {
        later: None,
        account: None,
        main: people::ModelEffort { model: 1, effort: Some(0) },
        subagents: Box::new([]),
    });
    let sign_in = world.sign_in.expect("owner signed in");
    jig_core_world::fixtures::ask_as(
        &mut world,
        sign_in,
        92,
        people::Ask::Create { project: 1, batch: Box::new([member]) },
    );
    jig_core_world::fixtures::refusal(&world, people::Refusal::Llm { model: 1, why: people::LlmWhy::Charter });
    assert_eq!(task_records(&world), before, "the refused Create writes no task records");
}

#[test]
fn a_procedure_hears_its_step_refused_for_a_model_outside_its_charter() {
    const SEED: u64 = 160;
    let mut world = two_charter_world(SEED);
    world.hold_procedures = true;
    let mut procedure = jig_test_profile::fixtures::people_member(b"procedure", 20);
    procedure.executor = people::TaskExecutor::Procedure { connector: 1, code: 1 };
    procedure.authority.delegation =
        people::Delegation { kinds: Box::new([people::Executor::Charter(1)]), tasks: 1, depth: 1 };
    let sign_in = world.sign_in.expect("owner signed in");
    jig_core_world::fixtures::ask_as(
        &mut world,
        sign_in,
        93,
        people::Ask::Create { project: 1, batch: Box::new([procedure]) },
    );
    let (step, connector, code, context) = world.procedure_inputs.last().expect("procedure started");
    let (task, step, connector, code) = (context.task, *step, *connector, u32::from(*code));
    let decision = research_step(&world, task);

    // Inspect the core's refusal before the root translates it to the connector's vocabulary.
    let (configuration, _) = jig_test_profile::two_charters(SEED);
    let env = world.env().with(world.env().limits.core);
    let mut recovered = super::start_links::recovered(&world, configuration.core, &env);
    let mut requests =
        core::step(&mut recovered, &env, core::Event::ProcedureStep { task, step, connector, code, decision });
    let mut refusal = None;
    while let Some(request) = requests.pop() {
        match request {
            core::Request::Ask { connector, ask: core::Ask::ProcedureHoldings { .. } } => {
                let mut answered = core::step(
                    &mut recovered,
                    &env,
                    core::Event::ProcedureHoldings {
                        task,
                        step,
                        connector,
                        holdings: Some(Box::new([Box::new([])])),
                        refused: None,
                    },
                );
                while let Some(request) = answered.pop() {
                    requests.push(request);
                }
            }
            core::Request::Ask {
                ask: core::Ask::StepRefused { task: refused_task, step: refused_step, why }, ..
            } => {
                assert_eq!((refused_task, refused_step), (task, step));
                refusal = Some(why);
            }
            core::Request::ProjectMade { .. }
            | core::Request::Write(_)
            | core::Request::Ask { .. }
            | core::Request::Held(_)
            | core::Request::Now(_) => {}
        }
    }
    assert!(
        matches!(
            refusal,
            Some(core::StepRefusal::Refused(tasks::Problem {
                why: tasks::Refusal::Llm { model: 1, why: tasks::LlmFault::Charter },
                ..
            }))
        ),
        "expected model 1 refused by its charter, got {refusal:?}"
    );

    let before = task_records(&world);
    let decision = research_step(&world, task);
    world.send(root::Event::Core(core::Event::ProcedureStep { task, step, connector, code, decision }));
    assert_eq!(task_records(&world), before, "the refused step writes no task records");
    let record =
        jig_test_connector::Record::ProcedureDelegateOutcome { task, step, child: None, revisions: Box::new([]) };
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: connector, key: record.key() }),
        Some(&root::Record::Connector { number: connector, record }),
        "the procedure receives its refused-step outcome"
    );
}

#[test]
fn a_delegate_on_another_charter_runs_its_instructions_and_account() {
    let (config, limits) = jig_test_profile::two_charters(141);
    let mut world = World::new(Setup { seed: 141, configuration: Some(config), limits, ..Setup::default() });
    assert_eq!(world.assigned.as_ref().expect("chat").charter, 1);
    world.grant_account(2);
    world.delegate_agent(2, b"find the regression");
    let assignment = world.assigned.as_ref().expect("delegate assigned");
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
    assert_eq!(assignment.grants[0].account, 2);
}

#[test]
fn a_task_waits_while_its_charters_account_is_unusable() {
    let (config, limits) = jig_test_profile::two_charters(142);
    let mut world = World::new(Setup { seed: 142, configuration: Some(config), limits, ..Setup::default() });
    let chat = world.assigned.as_ref().expect("chat").task;
    world.delegate_agent(2, b"wait for the account");
    assert_eq!(world.assigned.as_ref().expect("chat stays assigned").task, chat);
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
        if task.executor == tasks::Executor::Agent { charter: 2 } && task.attempt == 0)));
    world.grant_account(2);
    assert_eq!(world.assigned.as_ref().expect("delegate after grant").charter, 2);
}

#[test]
fn a_delegate_waits_for_a_host_declaring_its_charters_model() {
    use jig_domain_fleet as fleet;
    let (mut config, mut limits) = jig_test_profile::two_charters(151);
    limits.core.fleet.workers = 2;
    config.core.defaults.charters[1].policy.model.name = b"specialist".as_slice().into();
    config.core.defaults.charters[1].policy.model.endpoint = 2;
    let specialist = config.core.defaults.charters[1].policy.model.clone();
    config.core.settings.models = Box::new([config.core.settings.models[0].clone(), specialist]);
    config.core.settings.endpoints = Box::new([
        core::EndpointFacts { direct: Box::new([]), providers: Box::new([]), number: 1, dialect: 3, program: None },
        core::EndpointFacts { direct: Box::new([]), providers: Box::new([]), number: 2, dialect: 3, program: None },
    ]);
    let mut world = World::new(Setup { seed: 151, configuration: Some(config), limits, ..Setup::default() });
    let chat = world.assigned.as_ref().expect("chat").task;
    world.grant_account(2);
    world.delegate_agent(2, b"wait for a specialist host");
    assert_eq!(world.assigned.as_ref().expect("chat stays assigned").task, chat);
    let mut bounds = jig_test_profile::fixtures::bounds();
    bounds.endpoints[0].endpoint = 2;
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 1,
        channel: Token::new(8),
        hello: Hello {
            jobs: Box::default(),
            bounds,
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let assignment = world.assigned.as_ref().expect("delegate after suitable hello");
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.model.name.as_ref(), b"specialist");
}

#[test]
fn a_restart_restores_a_task_on_the_second_charter() {
    let (config, limits) = jig_test_profile::two_charters(143);
    let mut world = World::new(Setup { seed: 143, configuration: Some(config), limits, ..Setup::default() });
    world.delegate_agent(2, b"research across restart");
    let task = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
                if task.executor == tasks::Executor::Agent { charter: 2 } =>
            {
                Some(task.number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable delegate");
    world.restart_with(jig_test_profile::two_charters(143).0);
    world.grant_account(2);
    rejoin(&mut world);
    let assignment = world.assigned.as_ref().expect("restored delegate assigned");
    assert_eq!(assignment.task, task);
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
    assert_eq!(assignment.grants[0].account, 2);
}

fn rejoin(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: skein_lib::Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}

fn set_goal(world: &mut World, charter: u32) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(9000)),
        sign_in: world.sign_in.expect("signed in party"),
        key: [31; 16],
        ask: jig_core_world::fixtures::goal(
            world,
            1,
            b"Research this goal".as_slice().into(),
            people::TaskExecutor::Agent { charter },
            jig_test_profile::fixtures::Goal { budget: 20, priority: 1, ..Default::default() },
        ),
    })));
}

#[test]
fn a_goal_on_the_second_charter_is_admitted_and_runs_it() {
    let (config, limits) = jig_test_profile::two_charters(144);
    let mut world = World::new(Setup { seed: 144, configuration: Some(config), limits, ..Setup::default() });
    world.grant_account(2);
    set_goal(&mut world, 2);
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::Outcome(people::Outcome::Created { .. })))
    );
    let assignment = world.assigned.as_ref().expect("goal assigned");
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
    assert_eq!(assignment.grants[0].account, 2);
}

#[test]
fn a_goal_naming_an_unoffered_charter_is_refused_not_offered() {
    let (config, limits) = jig_test_profile::two_charters(145);
    let mut world = World::new(Setup { seed: 145, configuration: Some(config), limits, ..Setup::default() });
    set_goal(&mut world, 9);
    assert!(
        world.people_answers.iter().any(|reply| matches!(
            reply,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotOffered))
        ))
    );
}

fn host_world(seed: u64, kinds: jig_domain_fleet::Kinds) -> World {
    use jig_core_world::peers::{Peers, fixture};
    use jig_fake_parties::{Act, Ask, Party, Role};
    use jig_fake_workers::{Script, Worker};
    let (mut config, limits) = fixture(seed, 1);
    config.core.defaults.charters[0].hosts = kinds;
    let workers = [0, 7].into_iter().map(|channel| Worker::new(channel, 1, vec![Box::new([Script::Wait])])).collect();
    let party = Party::new(
        1,
        Role::Owner,
        10_000,
        Box::new([Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() }]),
    );
    let mut world = World::new(Setup {
        configuration: Some(config),
        limits,
        peers: Some(Peers::new(workers, vec![party])),
        ..Setup::default()
    });
    world.peers.as_mut().expect("peers").parties[0]
        .extend(Box::new([Act::Request { key: [1; 16], ask: Ask::Chat { words: Box::from(&b"hello"[..]) } }]));
    world.drain();
    world
}

#[test]
fn a_charter_for_workers_never_runs_in_the_engines_slots() {
    let world = host_world(146, jig_domain_fleet::Kinds::Workers);
    let peers = world.peers.as_ref().expect("peers");
    assert!(peers.workers[0].seen.is_empty(), "engine slot stays free");
    assert_eq!(peers.workers[1].seen.len(), 1, "worker received the assignment");
}

#[test]
fn a_charter_for_the_engine_never_runs_on_a_worker() {
    let mut world = host_world(147, jig_domain_fleet::Kinds::Engine);
    let peers = world.peers.as_ref().expect("peers");
    assert_eq!(peers.workers[0].seen.len(), 1, "engine received the assignment");
    assert!(peers.workers[1].seen.is_empty(), "worker slot stays free");
    // Engine 6 step 3 settles the dead engine/in-process claim: phase, retry, reserve and activity change durably.
    world.broken();
}

#[test]
fn a_restart_settles_an_engine_charters_claim_at_once() {
    let (mut config, mut limits) = jig_core_world::world::fixture(148, false);
    config.core.defaults.charters[0].hosts = jig_domain_fleet::Kinds::Engine;
    limits.core.fleet.engine_slots = 1;
    let mut world = World::new(Setup { seed: 148, configuration: Some(config), limits, ..Setup::default() });
    let assignment = world.assigned.as_ref().expect("engine assigned");
    let task = assignment.task;
    let attempt = assignment.attempt;
    world.send(root::Event::Turn {
        opens_window: false,
        refused: Box::new([]),
        channel: Token::new(0),
        task,
        attempt,
        turn: 1,
        cumulative: 2,
        read: None,
        transcript: b"engine turn".as_slice().into(),
    });
    assert!(world.store.rows.values().any(|record| matches!(record,
        root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof)))
        if proof.task == task && proof.host == jig_domain_fleet::HostKind::Engine)));
    let mut config = jig_core_world::world::fixture(148, false).0;
    config.core.defaults.charters[0].hosts = jig_domain_fleet::Kinds::Engine;
    world.restart_with(config);
    assert!(world.domain.ready(), "restart does not wait for a worker hello");
    assert!(world.store.rows.values().any(|record| matches!(record,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))
        if row.number == task && row.tries.lost == 1)));
}

fn worker_with_affinity(world: &mut World, task: u64) {
    use jig_domain_fleet as fleet;
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(8),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([task]),
            hosting: Box::new([]),
        },
    })));
}

fn both_host_world(seed: u64) -> World {
    let (mut config, mut limits) = jig_test_profile::two_charters(seed);
    config.core.defaults.charters[1].hosts = jig_domain_fleet::Kinds::Both;
    limits.core.fleet.engine_slots = 1;
    limits.core.fleet.slots = 1;
    let mut world = World::new(Setup { seed, configuration: Some(config), limits, ..Setup::default() });
    world.grant_account(2);
    world
}

fn delegate_task(world: &World) -> u64 {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
                if task.executor == tasks::Executor::Agent { charter: 2 } =>
            {
                Some(task.number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("delegate in store")
}

fn first_claim_host(world: &World) -> jig_domain_fleet::HostKind {
    let task = delegate_task(world);
    world
        .commits
        .iter()
        .flatten()
        .find_map(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))))
                if proof.task == task =>
            {
                Some(proof.host)
            }
            root::Write::Save(_)
            | root::Write::DropTranscript { .. }
            | root::Write::Wires
            | root::Write::Erase(_)
            | root::Write::CopySettingsRevision(_) => None,
        })
        .expect("first claim submitted")
}

fn current_claim_host(world: &World) -> jig_domain_fleet::HostKind {
    let task = delegate_task(world);
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &world.store.rows[&key] else {
        panic!("durable claim proof")
    };
    proof.host
}

#[test]
fn a_charter_both_kinds_may_run_claims_the_engine_when_only_its_slot_is_free() {
    let mut world = both_host_world(149);
    world.delegate_agent(2, b"use the free engine slot");
    assert_eq!(first_claim_host(&world), jig_domain_fleet::HostKind::Engine);
    // Engine 6 step 3 ends the engine claim; engine 11.5 recomputes Cancel impact from the changed live runs.
    world.broken();
}

#[test]
fn a_claim_for_the_engine_is_placed_only_on_the_engine() {
    let mut world = both_host_world(150);
    let assigned = world.assignments.len();
    world.store.fault(jig_core_world::Fault::Hold { commits: 2 });
    world.delegate_agent(2, b"keep the claimed kind");
    world.release_commits();
    assert_eq!(world.assignments.len(), assigned, "claim waits for its completion");
    let task = delegate_task(&world);
    worker_with_affinity(&mut world, task);
    world.release_commits();
    assert_eq!(world.assigned.as_ref().expect("delegate assigned").task, task);
    assert_eq!(current_claim_host(&world), jig_domain_fleet::HostKind::Engine);
    // Engine 6 step 3 ends the engine claim; engine 11.5 recomputes Cancel impact from the changed live runs.
    world.broken();
}

#[test]
fn a_task_created_without_an_llm_choice_keeps_none_and_its_run_uses_the_default() {
    let world = World::new(Setup { seed: 153, ..Setup::default() });
    let assignment = world.assigned.as_ref().expect("chat assigned");
    let chat = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) if task.number == assignment.task => {
                Some(task)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable chat");
    assert_eq!(chat.llm, None, "its run check resolves the default");
    assert_eq!(assignment.run.policy.model.name.as_ref(), b"scripted");
    assert_eq!(assignment.run.policy.model.effort.as_deref(), Some(b"low".as_slice()), "its first declared effort");
}

fn coordinator_wake() -> tasks::WakePolicy {
    tasks::WakePolicy {
        news: tasks::WakeRule::Paced { span: skein_lib::Duration::from_millis(1_800_000) },
        results: tasks::ResultsWake::Each,
        ..tasks::WakePolicy::DEFAULT
    }
}

fn wake_world(seed: u64) -> World {
    let (mut configuration, limits) = jig_core_world::world::fixture(seed, false);
    configuration.core.defaults.charters[0].wake = coordinator_wake();
    let creating = jig_test_profile::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
    let people::Ask::Create { project, mut batch } =
        jig_test_profile::fixtures::create_chat(&creating, 1, 0, b"coordinate".as_slice().into())
    else {
        panic!("initial chat")
    };
    batch[0].wake = None;
    World::new(Setup {
        seed,
        configuration: Some(configuration),
        limits,
        initial: Some(people::Ask::Create { project, batch }),
        ..Setup::default()
    })
}

fn task_wake(world: &World, number: u64) -> tasks::WakePolicy {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) if task.number == number => {
                Some(task.wake)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable task wake policy")
}

#[test]
fn a_party_task_naming_no_wake_takes_its_charters_default() {
    let world = wake_world(154);
    let task = world.assigned.as_ref().expect("coordinator assigned").task;
    assert_eq!(task_wake(&world, task), coordinator_wake());
}

fn delegate_wake(world: &mut World, wake: Option<tasks::WakePolicy>) -> u64 {
    let assignment = world.assigned.as_ref().expect("caller assigned");
    let run = (assignment.task, assignment.attempt);
    let mut member = jig_core_world::fixtures::member(world, tasks::Executor::Agent { charter: 1 }, b"research");
    member.wake = wake;
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    jig_core_world::fixtures::delegate(world, run, 1, member)
}

#[test]
fn a_delegate_naming_no_wake_takes_its_charters_default() {
    let mut world = wake_world(155);
    let task = delegate_wake(&mut world, None);
    assert_eq!(task_wake(&world, task), coordinator_wake());
}

#[test]
fn a_named_wake_policy_is_kept_over_the_charters() {
    let mut world = wake_world(156);
    let task = delegate_wake(&mut world, Some(tasks::WakePolicy::DEFAULT));
    assert_eq!(task_wake(&world, task), tasks::WakePolicy::DEFAULT);
}

#[test]
fn a_procedure_naming_no_wake_takes_the_default() {
    let mut world = wake_world(157);
    world.hold_procedures = true;
    let mut member = jig_test_profile::fixtures::people_member(b"procedure", 10);
    member.executor = people::TaskExecutor::Procedure { connector: 1, code: 1 };
    member.wake = None;
    let sign_in = world.sign_in.expect("owner signed in");
    jig_core_world::fixtures::ask_as(
        &mut world,
        sign_in,
        91,
        people::Ask::Create { project: 1, batch: Box::new([member]) },
    );
    let (_, _, _, context) = world.procedure_inputs.last().expect("procedure created");
    assert_eq!(task_wake(&world, context.task), tasks::WakePolicy::DEFAULT);
}
