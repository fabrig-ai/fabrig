//! Job admission (jig/docs/design/domain/engine.md, 7.5).
use crate::runs::delegation::job_member;
use crate::tests::*;
use jig_domain_link::{JobBound, JobKind};

fn bound() -> JobBound {
    JobBound { kind: JobKind(7), spec: 64, result: 32, deadline: Duration::from_secs(60) }
}

fn member() -> Delegate {
    Delegate {
        executor: tasks::Executor::Job { kind: 7 },
        spec: tasks::Spec {
            words: Box::new([]),
            parameters: Box::new([]),
            inputs: Box::new([]),
            asks: false,
            job: Some(Box::new(tasks::JobSpec { bytes: Box::new([0; 64]), deadline: Duration::from_secs(60) })),
        },
        contract: tasks::Contract::Report { words: 100 },
        authority: task_authority(&authority_value(0, Box::new([]))),
        deadline: tasks::Deadline::Named,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: people::Budget::Named,
    }
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_declared_job_within_its_bounds_takes_its_kinds_result_bound_as_its_contract() {
    let mut member = member();
    assert_eq!(job_member(&[bound()], &mut member), Ok(()));
    assert_eq!(member.contract, tasks::Contract::Job { bytes: 32 });
    assert_eq!(member.llm, None);
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_job_of_an_undeclared_kind_is_refused() {
    assert_eq!(job_member(&[], &mut member()), Err(tasks::Refusal::Executor));
    let mut member = member();
    member.executor = tasks::Executor::Job { kind: 9 };
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Executor));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_job_past_its_kinds_spec_or_deadline_is_refused() {
    let mut member = member();
    member.spec.job.as_mut().expect("job spec").bytes = Box::new([0; 65]);
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Spec));
    member.spec.job.as_mut().expect("job spec").bytes = Box::new([0; 64]);
    member.spec.job.as_mut().expect("job spec").deadline = Duration::from_secs(61);
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Spec));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_job_without_its_spec_is_refused() {
    let mut member = member();
    member.spec.job = None;
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Spec));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_job_naming_an_llm_is_refused() {
    let mut member = member();
    member.llm = Some(tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model: 0, effort: None },
        subagents: Box::new([]),
    });
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Executor));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_jobs_delegation_executor_names_its_kind() {
    let (core, _) = core();
    assert_eq!(core.delegation_executor(1, tasks::Executor::Job { kind: 7 }), Some(authority::Executor::Job(7)));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn configured_job_bounds_fit_at_the_tasks_childs_limits() {
    let limits = limits();
    let mut config = config(7);
    config.jobs = Box::new([JobBound { spec: limits.tasks.spec_bytes, result: limits.tasks.result_bytes, ..bound() }]);
    let core = Core::new(config, &limits).expect("job bounds fit the tasks child");
    assert_eq!(core.jobs.len(), 1);
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
#[should_panic(expected = "job kinds are ascending and distinct")]
fn repeated_job_kinds_are_not_configuration() {
    let mut config = config(7);
    config.jobs = Box::new([bound(), bound()]);
    drop(Core::new(config, &limits()));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
#[should_panic(expected = "job kinds are ascending and distinct")]
fn descending_job_kinds_are_not_configuration() {
    let mut config = config(7);
    config.jobs = Box::new([bound(), JobBound { kind: JobKind(6), ..bound() }]);
    drop(Core::new(config, &limits()));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
#[should_panic(expected = "job specs fit the tasks child")]
fn a_job_spec_bound_past_the_tasks_child_is_not_configuration() {
    let mut config = config(7);
    config.jobs = Box::new([JobBound { spec: limits().tasks.spec_bytes + 1, ..bound() }]);
    drop(Core::new(config, &limits()));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine.md, 7.5).
#[test]
#[should_panic(expected = "job results fit the tasks child")]
fn a_job_result_bound_past_the_tasks_child_is_not_configuration() {
    let mut config = config(7);
    config.jobs = Box::new([JobBound { result: limits().tasks.result_bytes + 1, ..bound() }]);
    drop(Core::new(config, &limits()));
}

fn due_job() -> (Core, Limits) {
    let (mut core, limits) = core();
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(900)), project: 1, period: 1, budget: 1000 },
    );
    let mut parent = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
    assert!(core.tasks.task(2).is_some(), "procedure fixture");
    let mut row = new_row(&core, 1, tasks::Funder::Task(2), None);
    let job = member();
    row.executor = job.executor;
    row.spec = job.spec;
    row.contract = tasks::Contract::Job { bytes: 32 };
    row.flags = tasks::Flags::PLAIN;
    row.llm = None;
    row.numbers.budget = 0;
    row.authority.budget.spend = 0;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(2), Box::new([row]));
    assert!(core.tasks.task(1).is_some(), "job fixture");
    core.opened = true;
    (core, limits)
}

/// A job's preparation and claim need no agent, account, brief or transcript
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_due_job_claims_without_an_agent_or_transcript_and_hands_its_start_to_connectors() {
    let (mut core, limits) = due_job();
    let context = core.tasks.run_context(1).expect("due job");
    let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true });
    let mut attempt = None;
    for request in out {
        match request {
            Request::Now(now) => {
                let Now::WorkspaceRequest { task: 1, attempt: fresh, context } = *now else {
                    panic!("job emitted agent preparation: {now:?}");
                };
                assert_eq!(context.executor, tasks::Executor::Job { kind: 7 });
                attempt = Some(fresh);
            }
            Request::Write(_) => {}
            other @ (Request::ProjectMade { .. } | Request::Ask { .. } | Request::Held(_)) => {
                panic!("unexpected job activation: {other:?}");
            }
        }
    }
    let attempt = attempt.expect("workspace requested");
    assert_eq!(core.tasks.task(1).expect("job").phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert!(core.transcripts.is_empty());
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt,
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: Some(Box::new([1, 2, 3])),
        },
    );
    let row = core.tasks.task(1).expect("claimed job");
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Claimed { attempt }));
    assert_eq!(row.numbers.reserved, 0);
    assert_eq!(row.kept_account, None);
    let proof = core.proofs.get(&1).expect("job proof");
    assert_eq!(proof.conversation, attempt);
    assert!(proof.accounts.is_empty());
    assert!(proof.credentials.is_empty());
    assert_eq!(proof.window, None);
    assert_eq!(proof.placement, None);
    assert_eq!(proof.host, fleet::HostKind::Worker);
    assert!(core.transcripts.is_empty());
    let mut starts = 0;
    let mut saved = false;
    for request in out {
        match request {
            Request::Ask { ask, .. } => {
                let Ask::JobClaimed { task: 1, attempt: named, kind, spec, deadline, result, grants, workspace } = ask
                else {
                    panic!("unexpected job handoff: {ask:?}");
                };
                assert!(saved, "proof saved before connector handoff");
                assert_eq!(named, attempt);
                assert_eq!(kind, JobKind(7));
                assert_eq!(spec.as_ref(), [0; 64]);
                assert_eq!(deadline, Duration::from_secs(60));
                assert_eq!(result, 32);
                assert!(grants.is_empty());
                assert_eq!(workspace.as_deref(), Some([1, 2, 3].as_slice()));
                starts += 1;
            }
            Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(_)))) => saved = true,
            Request::Write(_) => {}
            other @ (Request::ProjectMade { .. } | Request::Held(_) | Request::Now(_)) => {
                panic!("unexpected job claim: {other:?}");
            }
        }
    }
    assert_eq!(starts, core.connectors.len());
    assert!(core.job_workspaces.is_empty());
}

/// A blocked start remains due, without preparing agent state
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_due_job_is_remembered_until_decisions_open_and_preparation_is_ready() {
    for opened in [false, true] {
        let (mut core, limits) = due_job();
        core.opened = opened;
        let context = core.tasks.run_context(1).expect("due job");
        let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: !opened });
        assert!(out.is_empty());
        assert_eq!(core.due.len(), 1);
        assert!(core.due_ready());
        assert!(core.contexts.is_empty());
        assert!(core.transcripts.is_empty());
    }
}

/// A job waits for its application's credentials, independently of LLM accounts
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_jobs_credential_wait_keeps_it_due_and_its_claim_grants_only_the_application() {
    let (mut core, limits) = due_job();
    let context = core.tasks.run_context(1).expect("due job");
    drop(watch_outputs(&mut core, &limits, Event::Activate { context, ready: true }));
    let attempt = core.workspace_flight(1).expect("workspace flight").1;
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt,
            writes: Some(Box::new([])),
            credentials: Box::new([7]),
            workspace: None,
        },
    ));
    assert_eq!(core.tasks.task(1).expect("waiting job").phase, tasks::Phase::Active(tasks::Active::Due));
    assert!(!core.proofs.contains_key(&1));
    // The root delivers the new activation from PreparationWaiting.
    let context = core.tasks.run_context(1).expect("waiting job context");
    drop(watch_outputs(&mut core, &limits, Event::Activate { context, ready: true }));
    assert_eq!(core.due.len(), 1);
    assert!(!core.due_ready());
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Credential { number: 7, generation: 1, valid: Duration::from_secs(60) }),
    ));
    let context = core.take_due().expect("credential releases due job");
    drop(watch_outputs(&mut core, &limits, Event::Activate { context, ready: true }));
    let next = core.workspace_flight(1).expect("new workspace flight").1;
    assert!(next > attempt);
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt: next,
            writes: Some(Box::new([])),
            credentials: Box::new([7]),
            workspace: None,
        },
    );
    let mut selected = None;
    for request in &out {
        let Request::Ask { ask: Ask::JobClaimed { grants, .. }, .. } = request else { continue };
        selected = Some(grants);
    }
    let grants = selected.expect("job start with application grant");
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].form, accounts::Form::Application);
    assert_eq!(grants[0].account, 7);
    let proof = core.proofs.get(&1).expect("job proof");
    assert_eq!(proof.credentials.as_ref(), [7]);
    assert!(proof.accounts.is_empty());
}

/// A job's deadline holds even when a credential would make it wait
/// (jig/docs/design/domain/engine.md, 7.5).
#[test]
fn a_jobs_past_deadline_holds_before_credential_readiness() {
    let (mut core, _) = due_job();
    let mut context = core.tasks.run_context(1).expect("due job");
    context.authority.budget.deadline = Some(Wall::EPOCH);
    core.workspace_credentials.insert(1, Box::new([7])).expect("credential room");
    assert_eq!(
        crate::routing::run::jobs::admission(&core, &context, Wall::from_nanos(1), Box::new([])),
        Err(RunAdmission::Hold(tasks::Hold::Deadline))
    );
}

/// A job's application bytes reach its requester unchanged
/// (jig/docs/design/domain/engine.md, 7.5; jig/docs/design/domain/tasks.md, 5.6).
#[test]
fn a_jobs_result_notice_preserves_its_bytes() {
    assert_eq!(
        crate::routing::task::result_notice(tasks::Ending::Done(tasks::TaskResult::Job {
            bytes: Box::new([0, 255, 7])
        })),
        (tasks::ResultKind::Job, Box::<[u8]>::from([0, 255, 7]))
    );
}
