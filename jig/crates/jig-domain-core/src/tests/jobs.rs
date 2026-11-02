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
