//! Procedure-only job admission (jig/docs/design/domain/engine.md, 7.5).
use jig_core_world::{
    fixtures::{delegate, job_world, live_row, member},
    world::World,
};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

fn job(world: &World, kind: u32, bytes: usize, seconds: u64) -> core::Delegate {
    jig_core_world::fixtures::job(world, kind, vec![0; bytes].into_boxed_slice(), Duration::from_secs(seconds))
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

fn bound(kind: u32) -> host::JobBound {
    host::JobBound { kind: host::JobKind(kind), spec: 64, result: 32, deadline: Duration::from_secs(60) }
}

fn start(seed: u64) -> (World, u64, u64) {
    let (mut world, procedure) = job_world(seed, true);
    let child = job(&world, 7, 64, 60);
    let event = step(&world, procedure, child);
    world.send(root::Event::Core(event));
    let task = live_row(&world, procedure).delegates[0];
    (world, procedure, task)
}

fn proof(world: &World, task: u64) -> &core::RunProof {
    let key = root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)));
    let Some(root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof)))) = world.store.rows.get(&key)
    else {
        panic!("job claim proof")
    };
    proof
}

fn ended(world: &World, task: u64) -> &tasks::TaskRecord {
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)));
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) = world.store.rows.get(&key) else {
        panic!("ended job row")
    };
    row
}

fn answer(world: &mut World, ending: host::JobEnding) {
    let (channel, job) = world.jobs.last().expect("job assignment");
    world.answer_job(*channel, job.run, job.attempt, ending);
}

fn finished() -> host::JobEnding {
    host::JobEnding::Finished { result: b"result".as_slice().into(), work: host::Work { left: None, saved: None } }
}

fn failed(failure: host::JobFailure) -> host::JobEnding {
    host::JobEnding::Failed { failure, detail: Box::new([]), work: host::Work { left: None, saved: None } }
}

fn finish_procedure(world: &mut World, procedure: u64) {
    let (step, connector, code, context) = world.procedure_inputs.last().expect("next procedure input");
    assert_eq!(context.task, procedure);
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: procedure,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep::new(core::StepEnd::Result(tasks::TaskResult::Report { words: Box::new([]) })),
    }));
    assert!(matches!(ended(world, procedure).phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
}

/// A job claims without an account, waits for its kind, and returns its bytes to its procedure
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_procedures_job_waits_for_a_worker_declaring_its_kind_and_its_result_reaches_the_next_step() {
    let (mut world, procedure, task) = start(907);
    let row = live_row(&world, task);
    assert_eq!(row.executor, tasks::Executor::Job { kind: 7 });
    assert_eq!(row.contract, tasks::Contract::Job { bytes: 32 });
    assert_eq!(row.numbers.reserved, 0);
    assert!(matches!(row.phase, tasks::Phase::Active(tasks::Active::Claimed { .. })));
    assert!(proof(&world, task).accounts.is_empty());
    assert_eq!(proof(&world, task).window, None);
    assert_eq!(proof(&world, task).conversation, proof(&world, task).attempt);
    assert!(world.jobs.is_empty());
    world.job_hello(Token::new(8), 1, Box::new([bound(9)]));
    assert!(world.jobs.is_empty());
    world.job_hello(Token::new(9), 2, Box::new([bound(7)]));
    assert_eq!(world.jobs.len(), 1);
    let (channel, assignment) = &world.jobs[0];
    assert_eq!(*channel, Token::new(9));
    assert_eq!(assignment.run, Token::new(task));
    assert_eq!(assignment.kind, host::JobKind(7));
    assert_eq!(assignment.spec.as_ref(), [0; 64]);
    assert_eq!(assignment.deadline, Duration::from_secs(60));
    assert!(assignment.grants.is_empty());
    assert_eq!(assignment.workspace, None);
    answer(&mut world, finished());
    assert_eq!(
        ended(&world, task).phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Job { bytes: b"result".as_slice().into() }))
    );
    let context = &world.procedure_inputs.last().expect("procedure woken").3;
    assert_eq!(context.task, procedure);
    assert!(context.inbox.iter().any(
        |word| word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job) && word.words.as_ref() == b"result"
    ));
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)))));
    finish_procedure(&mut world, procedure);
}

/// Job deadlines retry by the run class, then escalate to the procedure
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_failing_job_is_held_past_its_tries_and_its_procedure_hears_it() {
    let (mut world, procedure, task) = start(908);
    world.job_hello(Token::new(8), 1, Box::new([bound(7)]));
    let retries = world.env().limits.core.tasks.retries.run;
    for count in 1..=retries.retries + 1 {
        let attempt = world.jobs.last().expect("job assigned").1.attempt;
        answer(&mut world, failed(host::JobFailure::Deadline));
        assert_eq!(live_row(&world, task).tries.run, count);
        if count <= retries.retries {
            let assigned = world.jobs.len();
            assert!(matches!(live_row(&world, task).phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
            world.advance(retries.max);
            assert_eq!(world.jobs.len(), assigned + 1);
            assert!(world.jobs.last().expect("retry assigned").1.attempt > attempt);
        }
    }
    assert!(matches!(
        live_row(&world, task).phase,
        tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Run), .. }
    ));
    let context = &world.procedure_inputs.last().expect("procedure hears escalation").3;
    assert_eq!(context.task, procedure);
    assert!(
        context
            .inbox
            .iter()
            .any(|word| matches!(word.kind, tasks::MessageKind::Escalation { task: held, .. } if held == task))
    );
}

/// A permanently unprepared workspace spends a permanent try and backs off
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_job_whose_workspace_could_not_be_prepared_for_good_fails_permanent() {
    let (mut world, _, task) = start(909);
    world.job_hello(Token::new(8), 1, Box::new([bound(7)]));
    answer(&mut world, failed(host::JobFailure::Unprepared(host::Preparation::Permanent { resource: None })));
    assert_eq!(live_row(&world, task).tries, tasks::Tries { permanent: 1, ..tasks::Tries::NONE });
    assert!(matches!(live_row(&world, task).phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
}

/// A subtree cancel immediately stops its job without a grace word
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn cancelling_the_requesters_subtree_ends_a_running_job_at_once() {
    let (mut world, procedure, task) = start(910);
    world.job_hello(Token::new(8), 1, Box::new([bound(7)]));
    let attempt = world.jobs[0].1.attempt.raw();
    let ancestor = world.assignments[0].0;
    crate::read_support::request(
        &mut world,
        92,
        jig_domain_people::Ask::Cancel {
            project: 1,
            task: ancestor,
            reason: b"cancel subtree".as_slice().into(),
            grace: None,
        },
    );
    assert!(world.cancellations.contains(&(task, attempt)));
    assert!(live_row(&world, task).inbox.is_empty());
    answer(&mut world, failed(host::JobFailure::Cancelled(host::Reason::Engine)));
    assert!(matches!(ended(&world, task).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    assert!(matches!(ended(&world, procedure).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
}

fn restart(world: &mut World, seed: u64) {
    let (mut config, _) = jig_core_world::faults::plan_fixture(seed);
    config.core.worker_names = Box::new([
        b"caller".as_slice().into(),
        b"first job worker".as_slice().into(),
        b"second job worker".as_slice().into(),
    ]);
    config.core.jobs = Box::new([bound(7)]);
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(1), authority::Executor::Job(7)]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    config.core.seed_projects[0].policy.policy.ceiling.delegation.kinds.clone_from(&kinds);
    config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.kinds = kinds;
    world.restart_with(config);
}

fn hello_hosting(world: &mut World, task: u64, attempt: u64) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 1,
        channel: Token::new(9),
        hello: fleet::Hello {
            jobs: Box::new([bound(7)]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([fleet::Hosting {
                run: Token::new(task),
                attempt: Token::new(attempt),
                phase: fleet::Phase::Active,
            }]),
        },
    })));
}

/// A listed claimed job survives a cold restart and its answer finishes it
/// (jig/docs/design/domain/engine.md, 7.5 and section 6).
#[test]
fn a_restart_adopts_a_claimed_job_its_worker_still_lists() {
    let (mut world, procedure, task) = start(911);
    world.job_hello(Token::new(8), 1, Box::new([bound(7)]));
    let attempt = world.jobs[0].1.attempt.raw();
    restart(&mut world, 911);
    hello_hosting(&mut world, task, attempt);
    assert!(world.cancellations.is_empty());
    assert_eq!(proof(&world, task).attempt, attempt);
    world.answer_job(Token::new(9), Token::new(task), Token::new(attempt), finished());
    assert!(matches!(ended(&world, task).phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    finish_procedure(&mut world, procedure);
}

/// An unlisted job is lost past the restart grace and retries with a new attempt
/// (jig/docs/design/domain/engine.md, 7.5 and section 6).
#[test]
fn a_claimed_job_no_worker_lists_after_a_restart_is_lost_and_tried_again() {
    let (mut world, procedure, task) = start(912);
    world.job_hello(Token::new(8), 1, Box::new([bound(7)]));
    let attempt = world.jobs[0].1.attempt;
    restart(&mut world, 912);
    let grace = world.env().limits.core.fleet.grace;
    world.advance(Duration::from_nanos(grace.as_nanos() - 1));
    assert_eq!(live_row(&world, task).tries.lost, 0);
    world.advance(Duration::from_nanos(1));
    assert_eq!(live_row(&world, task).tries.lost, 1);
    world.job_hello(Token::new(9), 1, Box::new([bound(7)]));
    world.advance(world.env().limits.core.tasks.retries.lost.max);
    assert_eq!(world.jobs.len(), 2);
    assert!(world.jobs[1].1.attempt > attempt);
    answer(&mut world, finished());
    finish_procedure(&mut world, procedure);
}
