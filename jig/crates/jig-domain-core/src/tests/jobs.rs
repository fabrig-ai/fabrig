//! Job admission (jig/docs/design/domain/engine/executors.md, 5).
use crate::documents::{JobWhy, TaskRunWait, TaskWaitRefused, run_wait};
use crate::runs::delegation::job_member;
use crate::tests::new_core;
use crate::tests::*;
use jig_domain_fleet::JobShort;
use jig_domain_link::{Form, JobBound, JobKind, PauseState};

const SHORT: Token = Token::new(11);
const COVERING: Token = Token::new(12);

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
        wake: Some(tasks::WakePolicy::DEFAULT),
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: people::Budget::Named,
    }
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_declared_job_within_its_bounds_takes_its_kinds_result_bound_as_its_contract() {
    let mut member = member();
    assert_eq!(job_member(&[bound()], &mut member), Ok(()));
    assert_eq!(member.contract, tasks::Contract::Job { bytes: 32 });
    assert_eq!(member.llm, None);
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_job_of_an_undeclared_kind_is_refused() {
    assert_eq!(job_member(&[], &mut member()), Err(tasks::Refusal::Executor));
    let mut member = member();
    member.executor = tasks::Executor::Job { kind: 9 };
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Executor));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_job_past_its_kinds_spec_or_deadline_is_refused() {
    let mut member = member();
    member.spec.job.as_mut().expect("job spec").bytes = Box::new([0; 65]);
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Spec));
    member.spec.job.as_mut().expect("job spec").bytes = Box::new([0; 64]);
    member.spec.job.as_mut().expect("job spec").deadline = Duration::from_secs(61);
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Spec));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_job_without_its_spec_is_refused() {
    let mut member = member();
    member.spec.job = None;
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Spec));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_job_naming_an_llm_is_refused() {
    let mut member = member();
    member.llm = Some(tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model: 0, effort: None, speed: None },
        subagents: Box::new([]),
    });
    assert_eq!(job_member(&[bound()], &mut member), Err(tasks::Refusal::Executor));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_jobs_delegation_executor_names_its_kind() {
    let (core, _) = core();
    assert_eq!(core.delegation_executor(1, tasks::Executor::Job { kind: 7 }), Some(authority::Executor::Job(7)));
}

/// The application declares job kinds and their bounds (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn configured_job_bounds_fit_at_the_tasks_childs_limits() {
    let limits = limits();
    let mut config = config(7);
    config.jobs = Box::new([JobBound { spec: limits.tasks.spec_bytes, result: limits.tasks.result_bytes, ..bound() }]);
    let core = new_core(config, &limits).expect("job bounds fit the tasks child");
    assert_eq!(
        core.jobs.as_ref(),
        &[JobBound { spec: limits.tasks.spec_bytes, result: limits.tasks.result_bytes, ..bound() }]
    );
}

fn due_job() -> (Core, Limits) {
    due_job_with_holds(Box::new([]))
}

fn due_job_with_holds(holds: Box<[tasks::Name]>) -> (Core, Limits) {
    let (mut core, limits) = due_job_without_host(holds);
    job_hello(&mut core, &limits, COVERING, 1, Box::new([bound()]));
    (core, limits)
}

fn due_job_without_host(holds: Box<[tasks::Name]>) -> (Core, Limits) {
    make_due_job(holds, &limits())
}

fn make_due_job(holds: Box<[tasks::Name]>, limits: &Limits) -> (Core, Limits) {
    let mut config = config(41);
    config.jobs = Box::new([bound()]);
    let mut core = core_with_config(config, limits);
    let grant = authority::Grant {
        connector: 1,
        kind: 5,
        pattern: authority::Pattern { segments: Box::new([]), last: authority::Last::Open(Box::new([])) },
    };
    let mut rules = core.authority.rules().clone();
    let mut policy = core.authority.policy(1).expect("fixture project").clone();
    rules.ceiling.grants = Box::new([grant.clone()]);
    policy.roles[0].authority.grants = Box::new([grant.clone()]);
    policy.ceiling.grants = Box::new([grant]);
    core.authority = authority::Domain::new(rules, limits.authority).expect("fixture grants");
    let mut out = Queue::uncapped();
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    for resource in &holds {
        let hold = tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits };
        send_task(&mut core, limits, tasks::Event::Resource { name: resource.clone(), hold });
        send_task(
            &mut core,
            limits,
            tasks::Event::ResourceAccess {
                project: 1,
                name: resource.clone(),
                access: tasks::ResourceAccess::Writable,
            },
        );
    }
    send_task(&mut core, limits, tasks::Event::Check);
    send_task(&mut core, limits, tasks::Event::Restored);
    send_task(
        &mut core,
        limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(900)), project: 1, period: 1, budget: 1000 },
    );
    let mut parent = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    let mut holdings = List::with_capacity(limits.tasks.holdings);
    for resource in holds {
        holdings.push(tasks::Holding::Write { resource, kind: 1 }).expect("fixture holdings");
    }
    parent.holdings = holdings.into_boxed();
    fixtures::make_batch(&mut core, limits, tasks::Party::Person(1), Box::new([parent]));
    assert!(core.tasks.task(2).is_some(), "procedure fixture");
    let mut row = new_row(&core, 1, tasks::Funder::Task(2), None);
    let job = member();
    row.executor = job.executor;
    row.spec = job.spec;
    row.spec.job.as_mut().expect("job payload").deadline = Duration::from_secs(1);
    row.contract = tasks::Contract::Job { bytes: 32 };
    row.flags = tasks::Flags::PLAIN;
    row.llm = None;
    row.numbers.budget = 0;
    row.authority.budget.spend = 0;
    fixtures::make_batch(&mut core, limits, tasks::Party::Task(2), Box::new([row]));
    assert!(core.tasks.task(1).is_some(), "job fixture");
    core.opened = true;
    (core, *limits)
}

/// A job's preparation and claim need no agent, account, brief or transcript
/// (jig/docs/design/domain/engine/executors.md, 5).
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
            guide_limit: 0,
            task: 1,
            attempt,
            holds: Box::new([]),
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
                assert_eq!(deadline, Duration::from_secs(1));
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
/// (jig/docs/design/domain/engine/executors.md, 5).
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
/// (jig/docs/design/domain/engine/executors.md, 5).
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
            guide_limit: 0,
            task: 1,
            attempt,
            holds: Box::new([]),
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
            guide_limit: 0,
            task: 1,
            attempt: next,
            holds: Box::new([]),
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
    assert_eq!(grants[0].form, Form::Application);
    assert_eq!(grants[0].account, 7);
    let proof = core.proofs.get(&1).expect("job proof");
    assert_eq!(proof.credentials.as_ref(), [7]);
    assert!(proof.accounts.is_empty());
}

/// A job's deadline holds even when a credential would make it wait
/// (jig/docs/design/domain/engine/executors.md, 5).
#[test]
fn a_jobs_past_deadline_holds_before_credential_readiness() {
    let (mut core, limits) = due_job();
    let mut row = core.tasks.task(1).expect("due job").clone();
    row.authority.budget.deadline = Some(Wall::EPOCH);
    let parent = core.tasks.task(2).expect("procedure").clone();
    fixtures::restore(&mut core, &limits, Box::new([parent, row]));
    let context = core.tasks.run_context(1).expect("due job");
    core.workspace_credentials.insert(1, Box::new([7])).expect("credential room");
    assert_eq!(
        crate::routing::run::jobs::admission(&core, &context, Wall::from_nanos(1), Box::new([])),
        Err(RunAdmission::Hold(tasks::Hold::Deadline))
    );
}

/// A job's application bytes reach its requester unchanged
/// (jig/docs/design/domain/engine/executors.md, 5; jig/docs/design/domain/tasks/lifecycle.md, 7).
#[test]
fn a_jobs_result_notice_preserves_its_bytes() {
    assert_eq!(
        crate::routing::task::result_notice(tasks::Ending::Done(tasks::TaskResult::Job {
            bytes: Box::new([0, 255, 7])
        })),
        (tasks::ResultKind::Job, Box::<[u8]>::from([0, 255, 7]))
    );
}

/// Jobs claim their connectors' held names, distinct from authority paths
/// (jig/docs/design/domain/engine/executors.md, 1, step 3 and 7.5).
#[test]
fn a_job_claims_the_names_its_root_prepared() {
    let held = tasks::Name { connector: 1, path: Box::new([Box::from(b"slot".as_slice())]) };
    let checked = tasks::Name { connector: 1, path: Box::new([Box::from(b"authority".as_slice())]) };
    let (mut core, limits) = due_job_with_holds(Box::new([held.clone()]));
    let context = core.tasks.run_context(1).expect("due job");
    drop(watch_outputs(&mut core, &limits, Event::Activate { context, ready: true }));
    let attempt = core.workspace_flight(1).expect("workspace request").1;
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            guide_limit: 0,
            task: 1,
            attempt,
            writes: Some(Box::new([authority::Write {
                effect: authority::Effect {
                    connector: checked.connector,
                    kind: 5,
                    name: authority::Name { segments: checked.path.clone() },
                    state: [0; 32],
                    price: None,
                    access: authority::EffectAccess::Owned,
                    additional: Box::new([]),
                    guards: Box::new([]),
                },
                held: authority::Writer::Ancestor,
            }])),
            holds: Box::new([held.clone(), held.clone()]),
            credentials: Box::new([]),
            workspace: None,
        },
    );
    assert_eq!(core.tasks.task(1).expect("job").phase, tasks::Phase::Active(tasks::Active::Claimed { attempt }));
    assert_eq!(core.tasks.writer(&held).expect("prepared writer slot").writer, tasks::Writer::Run { task: 1, attempt });
    assert!(core.tasks.writer(&checked).is_none(), "authority paths do not name writer slots");
    let mut writers = List::with_capacity(limits.tasks.holdings);
    for request in out {
        if let Request::Write(Write::Save(Record::Tasks(tasks::Stored::Writer(slot)))) = request {
            writers.push(slot.resource).expect("bounded writer slots");
        }
    }
    assert_eq!(writers.as_slice(), [held], "the prepared names are deduplicated");
}

#[test]
fn a_full_store_work_lane_does_not_delay_a_job() {
    for full in [false, true] {
        let (mut core, limits) = due_job();
        if full {
            for task in 0..limits.load_slots {
                fixture_window(&mut core, u64::from(task) + 100, (1, 1));
            }
            assert!(!core.loads.room(&limits));
        }
        let context = core.tasks.run_context(1).expect("due job");
        let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true });
        assert_eq!(core.tasks.task(1).expect("job").phase, tasks::Phase::Active(tasks::Active::Preparing));
        assert!(core.due.is_empty());
        assert!(core.transcripts.is_empty());
        let mut workspaces = 0_u32;
        for request in out {
            if let Request::Now(now) = request {
                let Now::WorkspaceRequest { task: 1, context, .. } = *now else {
                    panic!("only job workspace preparation leaves activation");
                };
                assert_eq!(context.executor, tasks::Executor::Job { kind: 7 });
                workspaces += 1;
            }
        }
        assert_eq!(workspaces, 1);
    }
}

fn job_fleet(core: &mut Core, limits: &Limits, event: fleet::Event) -> Box<[Request]> {
    watch_outputs(core, limits, Event::Fleet(event))
}

fn job_hello(core: &mut Core, limits: &Limits, channel: Token, worker: u32, jobs: Box<[JobBound]>) {
    let out = job_fleet(
        core,
        limits,
        fleet::Event::Hello {
            worker,
            channel,
            hello: Hello {
                fresh: false,
                reads_usage: false,
                jobs,
                bounds: fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
    );
    assert!(core.fleet.program_endpoint(channel, 1).is_some(), "worker not admitted: {out:?}");
}

fn activate_job(core: &mut Core, limits: &Limits) {
    let context = core.tasks.run_context(1).expect("due job");
    drop(watch_outputs(core, limits, Event::Activate { context, ready: true }));
}

fn prepared_job(core: &mut Core, limits: &Limits, attempt: u64) -> Box<[Request]> {
    watch_outputs(
        core,
        limits,
        Event::WorkspacePrepared {
            guide_limit: 0,
            task: 1,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: Some(Box::new([1])),
        },
    )
}

fn start_job(core: &mut Core, limits: &Limits, attempt: u64) {
    let payload = JobBound { deadline: Duration::from_secs(1), ..bound() };
    drop(job_fleet(
        core,
        limits,
        fleet::Event::StartJob {
            reply_to: ReplyTo::new(Token::new(1)),
            run: Token::new(1),
            attempt: Token::new(attempt),
            workstream: 1,
            job: payload,
            adopt: false,
        },
    ));
}

fn resume_job(core: &mut Core, limits: &Limits) -> Queue<Request> {
    resume_fleet(core, &Env { limits: *limits, now: Time::ZERO, wall: Wall::EPOCH })
}

fn assert_due(core: &Core, wait: TaskRunWait) {
    let row = core.tasks.task(1).expect("job");
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(row.tries, tasks::Tries::NONE);
    assert!(core.workspace_flight(1).is_none());
    assert_eq!(run_wait(core, row), Some(wait));
}

/// A host must cover the declaration even when this payload needs less
/// (jig/docs/design/domain/engine/fleet.md, 2).
#[test]
fn a_job_waits_for_its_declared_kind_and_deadline_then_places_on_the_covering_worker() {
    let offered = JobBound { deadline: Duration::from_secs(10), ..bound() };
    for has_kind in [false, true] {
        let (jobs, why, figures): (Box<[JobBound]>, _, _) = if has_kind {
            (Box::new([offered]), JobShort::Deadline, Some(fleet::Figures { required: 60_000, offered: 10_000 }))
        } else {
            (Box::new([]), JobShort::Kind, None)
        };
        let mut limits = limits();
        limits.fleet.workers = 2;
        let (mut core, limits) = make_due_job(Box::new([]), &limits);
        job_hello(&mut core, &limits, SHORT, 0, jobs);
        activate_job(&mut core, &limits);
        assert_due(
            &core,
            TaskRunWait::Refused(TaskWaitRefused::Job { worker: Some(1), kind: 7, why: JobWhy::Short(why), figures }),
        );
        assert!(!core.proofs.contains_key(&1));
        assert!(core.take_due().is_none());
        job_hello(&mut core, &limits, COVERING, 1, Box::new([bound()]));
        let context = core.take_due().expect("covering hello makes the job ready");
        drop(watch_outputs(&mut core, &limits, Event::Activate { context, ready: true }));
        let attempt = core.workspace_flight(1).unwrap().1;
        drop(prepared_job(&mut core, &limits, attempt));
        assert_eq!(core.tasks.task(1).unwrap().phase, tasks::Phase::Active(tasks::Active::Claimed { attempt }));
        start_job(&mut core, &limits, attempt);
        let mut assigned = false;
        for request in &resume_job(&mut core, &limits) {
            if let Request::Held(held) = request
                && let Held::Assign { channel, run, attempt: named, .. } = **held
            {
                assert_eq!((channel, run.raw(), named.raw()), (COVERING, 1, attempt));
                assigned = true;
            }
        }
        assert!(assigned, "the covering worker receives the assignment");
    }
}

/// Host capacity and pause are readiness, not a shortfall (fleet.md, 2).
#[test]
fn a_job_waits_for_the_only_covering_hosts_room_and_resume() {
    let (mut core, limits) = due_job_without_host(Box::new([]));
    activate_job(&mut core, &limits);
    assert_due(&core, TaskRunWait::Host);
    let busy = Token::new(9);
    let env = Env { limits: limits.fleet, now: Time::ZERO, wall: Wall::EPOCH };
    job_hello(&mut core, &limits, COVERING, 1, Box::new([bound()]));
    let mut out = Queue::uncapped();
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::StartJob {
            reply_to: ReplyTo::new(busy),
            run: busy,
            attempt: busy,
            workstream: 9,
            job: bound(),
            adopt: false,
        },
        &mut out,
    );
    fleet::resume(&mut core.fleet, &env, true, &mut out);
    assert_eq!(out.pop(), Some(fleet::Request::AssignJob { channel: COVERING, run: busy, attempt: busy }));
    assert_due(&core, TaskRunWait::Host);
    assert!(!core.due_ready());
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Answer {
            channel: COVERING,
            run: busy,
            attempt: busy,
            answer: fleet::Answer::Ended,
            payload: busy,
        },
        &mut out,
    );
    drop(job_fleet(&mut core, &limits, fleet::Event::Acknowledge { run: busy, attempt: busy }));
    assert!(core.due_ready(), "free slot releases the due job");
    for state in [PauseState::Paused, PauseState::Draining, PauseState::Taking] {
        drop(job_fleet(&mut core, &limits, fleet::Event::Paused { channel: COVERING, state }));
        if state == PauseState::Taking {
            assert!(core.due_ready(), "taking work releases the due job");
        } else {
            assert_due(&core, TaskRunWait::Host);
            assert!(!core.due_ready());
        }
    }
}

/// A lost host before claim prevents commitment; loss after claim causes Recheck (fleet.md, 2).
#[test]
fn a_job_whose_host_is_lost_goes_due_without_spending_a_try() {
    for claimed in [false, true] {
        let (mut core, limits) = due_job();
        activate_job(&mut core, &limits);
        let attempt = core.workspace_flight(1).unwrap().1;
        if claimed {
            drop(prepared_job(&mut core, &limits, attempt));
            assert_eq!(core.tasks.task(1).unwrap().phase, tasks::Phase::Active(tasks::Active::Claimed { attempt }));
            start_job(&mut core, &limits, attempt);
        }
        drop(job_fleet(&mut core, &limits, fleet::Event::Lost { channel: COVERING }));
        if claimed {
            let mut dropped = false;
            for request in &resume_job(&mut core, &limits) {
                if let Request::Now(now) = request
                    && let Now::DropAssignment { task: 1 } = **now
                {
                    dropped = true;
                }
            }
            assert!(dropped);
            assert_eq!(core.fleet.waiting(), 0);
        } else {
            let out = prepared_job(&mut core, &limits, attempt);
            let mut returned = false;
            for request in out {
                if let Request::Now(now) = &request
                    && let Now::RunPreparationFailed { task: 1 } = **now
                {
                    returned = true;
                }
                if let Request::Ask { ask: Ask::JobClaimed { .. }, .. } = request {
                    panic!("no handoff when the host is lost during preparation");
                }
            }
            assert!(returned);
            assert!(!core.proofs.contains_key(&1));
        }
        assert_due(&core, TaskRunWait::Host);
        activate_job(&mut core, &limits);
        assert!(core.take_due().is_none());
    }
}

/// An untranslated job keeps only its task off the channel until it is lost (layouts.md, 5.1).
#[test]
fn an_untranslated_job_is_due_after_the_pause_and_waits_for_a_new_channel() {
    let (mut core, limits) = due_job();
    activate_job(&mut core, &limits);
    let attempt = core.workspace_flight(1).expect("prepared job").1;
    drop(prepared_job(&mut core, &limits, attempt));
    start_job(&mut core, &limits, attempt);
    drop(resume_job(&mut core, &limits));
    let run = Token::new(1);
    let attempt = Token::new(attempt);
    let payload = Token::new(99);
    drop(job_fleet(
        &mut core,
        &limits,
        fleet::Event::Answer { channel: COVERING, run, attempt, answer: fleet::Answer::Untranslated, payload },
    ));
    // The hub link decodes the component's local refusal as RefusedWithReason.
    let reason = b"the worker's layout cannot keep jig-channel Job.workspace: field".as_slice();
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::AnswerPayload {
            run,
            attempt,
            payload,
            cumulative: 0,
            end: tasks::End::RefusedWithReason { reason: reason.into() },
            saved: None,
            invalid_saved: false,
            unusable_history: false,
            untooled: false,
            read: None,
            refused: Box::new([]),
        },
    ));
    let row = core.tasks.task(1).unwrap();
    assert_eq!(row.retry_notice, Some(tasks::RetryNotice::Refused { count: 1, reason: Some(reason.into()) }));
    let tasks::Phase::Active(tasks::Active::BackingOff { until }) = row.phase else { panic!("refusal pause") };
    let env = Env { limits, now: core.tasks.next_deadline().unwrap(), wall: until };
    drop(fire(&mut core, &env, Timer::Tasks));
    assert_due(
        &core,
        TaskRunWait::Refused(TaskWaitRefused::Job {
            worker: Some(2),
            kind: 7,
            why: JobWhy::Untranslated,
            figures: None,
        }),
    );
    let context = core.tasks.run_context(1).unwrap();
    drop(step(&mut core, &env, Event::Activate { context, ready: true }));
    assert!(!core.due_ready());
    drop(job_fleet(&mut core, &limits, fleet::Event::Lost { channel: COVERING }));
    core.reclaim();
    job_hello(&mut core, &limits, SHORT, 1, Box::new([bound()]));
    assert!(core.take_due().is_some(), "replacement channel has no exclusion");
}
