//! Procedure-only job admission (jig/docs/design/domain/engine.md, 7.5).
use jig_core_world::{
    fixtures::{delegate, job_world, live_row, member},
    world::World,
};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

fn job(world: &World, kind: u32, bytes: usize, seconds: u64) -> core::Delegate {
    let mut child = member(world, tasks::Executor::Job { kind }, b"");
    child.spec.job = Some(Box::new(tasks::JobSpec {
        bytes: vec![0; bytes].into_boxed_slice(),
        deadline: Duration::from_secs(seconds),
    }));
    child.contract = tasks::Contract::Job { bytes: 0 };
    child.authority.budget.spend = 0;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    child
}

fn step(world: &World, task: u64, child: core::Delegate) -> core::Event {
    let (step, connector, code, context) = world.procedure_inputs.last().expect("procedure input");
    assert_eq!(context.task, task);
    core::Event::ProcedureStep {
        task,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep { batch: Box::new([child]), ..core::ProcedureStep::new(core::StepEnd::Wait) },
    }
}

fn no_child(world: &World, task: u64) {
    assert!(live_row(world, task).delegates.is_empty());
    let record = connector::Record::ProcedureDelegateOutcome {
        task,
        step: world.procedure_inputs.last().expect("procedure input").0,
        child: None,
        revisions: Box::new([]),
    };
    assert_eq!(
        world.store.rows.get(&root::Key::Connector { number: 1, key: record.key() }),
        Some(&root::Record::Connector { number: 1, record })
    );
}

// Inspect the core's refusal before the root translates its diagnostics into the
// connector's refusal vocabulary, then deliver the same refused step through the root.
fn decide(world: &mut World, task: u64, child: core::Delegate) -> core::StepRefusal {
    let event = step(world, task, child.clone());
    let env = world.env().with(world.env().limits.core);
    let mut requests = core::step(world.domain.core_mut(), &env, event);
    let Some(core::Request::Ask { connector: 1, ask: core::Ask::StepRefused { task: named, why, .. } }) =
        requests.pop()
    else {
        panic!("core refused the procedure step: {requests:?}");
    };
    assert_eq!(named, task);
    assert!(requests.is_empty());
    let event = step(world, task, child);
    world.send(root::Event::Core(event));
    no_child(world, task);
    why
}

fn refused(why: &core::StepRefusal, task: u64, expected: tasks::Refusal) {
    assert_eq!(
        why,
        &core::StepRefusal::Refused(tasks::Problem {
            task: Some(task),
            why: expected,
            blocked_by: None,
            capacity: None,
        })
    );
}

/// An undeclared kind refuses the whole step (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_procedures_job_of_an_undeclared_kind_is_refused_and_its_procedure_told() {
    let (mut world, procedure) = job_world(901, true);
    let child = job(&world, 9, 64, 60);
    refused(&decide(&mut world, procedure, child), procedure, tasks::Refusal::Executor);
}

/// Each kind's spec and deadline are bounded (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_procedures_job_past_its_kinds_bounds_is_refused() {
    let (mut world, procedure) = job_world(902, true);
    let child = job(&world, 7, 65, 60);
    refused(&decide(&mut world, procedure, child), procedure, tasks::Refusal::Spec);
    let child = job(&world, 7, 64, 61);
    refused(&decide(&mut world, procedure, child), procedure, tasks::Refusal::Spec);
}

fn without_job_delegation(world: &mut World, procedure: u64) {
    let mut authority = live_row(world, procedure).authority.clone();
    authority.delegation.kinds = Box::new([]);
    let run = world.assignments[0];
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9002)),
        key: core::CallKey { task: run.0, attempt: run.1, completion: 2, position: 0 },
        action: core::NamedAction::Amend {
            target: procedure,
            revision: 0,
            amendment: tasks::Amendment { authority: Some(authority), ..jig_test_profile::fixtures::unchanged(b"") },
        },
    }));
    assert!(live_row(world, procedure).authority.delegation.kinds.is_empty());
}

fn denied(why: core::StepRefusal, expected: authority::Answer, source: authority::Source) {
    let core::StepRefusal::Denied { answer, findings } = why else { panic!("authority denial: {why:?}") };
    assert_eq!(answer, expected);
    assert!(findings.contains(&authority::Finding::Executor { source }));
}

/// A task may propose a kind covered by its ceilings (jig/docs/design/domain/engine.md, 7.5;
/// jig/docs/design/domain/authority.md, 8.1).
#[test]
fn a_job_kind_its_procedures_delegation_lacks_is_answered_propose() {
    let (mut world, procedure) = job_world(903, true);
    without_job_delegation(&mut world, procedure);
    let child = job(&world, 7, 64, 60);
    denied(decide(&mut world, procedure, child), authority::Answer::Propose, authority::Source::Task);
}

/// A project's ceiling cannot be proposed past (jig/docs/design/domain/engine.md, 7.5;
/// jig/docs/design/domain/authority.md, 8.1).
#[test]
fn a_job_kind_its_projects_ceiling_lacks_is_answered_refuse() {
    let (mut world, procedure) = job_world(904, false);
    let child = job(&world, 7, 64, 60);
    denied(decide(&mut world, procedure, child), authority::Answer::Refuse, authority::Source::Project);
}

/// An agent cannot make a job (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn an_agents_delegate_naming_a_job_is_refused() {
    let (mut world, procedure) = job_world(905, true);
    let before = live_row(&world, world.assignments[0].0).delegates.clone();
    let run = world.assignments[0];
    let child = job(&world, 7, 64, 60);
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9002)),
        key: core::CallKey { task: run.0, attempt: run.1, completion: 2, position: 0 },
        batch: Box::new([child]),
        stubs: Box::new([]),
    }));
    assert_eq!(
        world.answers.last().expect("delegation refusal").2,
        core::CallPart::DelegationRefused(tasks::Problem {
            task: None,
            why: tasks::Refusal::Executor,
            blocked_by: None,
            capacity: None,
        })
    );
    assert_eq!(live_row(&world, run.0).delegates, before);
    assert!(live_row(&world, procedure).delegates.is_empty());
}

/// A connector's job action reaches core admission through the test kits
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_connector_procedure_programmed_with_an_undeclared_job_hears_its_step_refused() {
    let seed = 906;
    let (mut configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    configuration.core.jobs = Box::new([jig_domain_host::JobBound {
        kind: jig_domain_host::JobKind(7),
        spec: 64,
        result: 32,
        deadline: Duration::from_secs(60),
    }]);
    configuration.first.procedures[0].actions = Box::new([
        connector::ProcedureAction::Job { kind: 9, spec: Box::new([0; 64]), deadline: Duration::from_secs(60) },
        connector::ProcedureAction::Finish { result: connector::ProcedureResult::Succeeded },
    ]);
    configuration.first.procedures[0].max_steps = 2;
    let mut world = World::new(jig_core_world::world::Setup {
        seed,
        configuration: Some(configuration),
        limits,
        ..jig_core_world::world::Setup::default()
    });
    let run = world.assignments[0];
    let mut procedure = member(&world, tasks::Executor::Procedure { connector: 1, code: 1 }, b"program");
    procedure.authority.budget.spend = 0;
    procedure.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let task = delegate(&mut world, run, 2, procedure);
    no_child(&world, task);
    let key = root::Key::Connector { number: 1, key: connector::RecordKey::Procedure(task) };
    let Some(root::Record::Connector { record: connector::Record::Procedure(state), .. }) = world.store.rows.get(&key)
    else {
        panic!("procedure program committed")
    };
    assert_eq!(state.steps, 1);
    assert_eq!(state.index, 0);
    assert!(!state.awaiting);
    // Close the waiting procedure before the restart oracle resumes live work.
    crate::read_support::request(
        &mut world,
        91,
        jig_domain_people::Ask::Cancel { project: 1, task, reason: b"story complete".as_slice().into(), grace: None },
    );
}
