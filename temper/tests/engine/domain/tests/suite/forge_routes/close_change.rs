//! Close changes use a job producer (temper/docs/design/domain/forge/change/README.md, 1 and temper/docs/design/domain/forge/change/repairs.md and temper/docs/design/domain/forge/branches.md, 2).
use super::goal_branch::{branch_tip, goal_request};
use super::light_rules::change_row;
use super::*;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000;
const REQUEST: &[u8] = br#"{"version":1,"goal":7,"changes":[{"task":9,"landed":true}]}"#;
const COMMAND: &[u8] = b".temper/close";

fn goal_world(command: bool, authority: bool) -> (World, engine::Assignment, Box<[u8]>, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { result_bytes: Some(512), ..StoryLimits::default() },
    )
    .with_close_job(command.then_some(COMMAND), authority)
    .with_landing_deployment(Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: authority::Last::Open(Box::new([])),
        },
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    world.adopt();
    let mut event = protocol::hello_slots(CHANNEL, 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.jobs = Box::new([jig_domain_link::JobBound {
        kind: jig_domain_link::JobKind(forge_top::CLOSE_KIND),
        spec: 16_384,
        result: 512,
        deadline: Duration::from_secs(600),
    }]);
    world.send(event);
    let request = goal_request(&world, true);
    let goal = assigned_chat(&mut world, 92, request);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    let head = branch_tip(&mut world, &branch);
    (world, goal, branch, head)
}

fn make_change(
    world: &mut World,
    goal: &engine::Assignment,
    branch: &[u8],
    request: &[u8],
    extra: Option<tasks::Parameter>,
) -> u64 {
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(b"Settle goal 7's status.", Box::new([]));
    delegate.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 256 };
    delegate.authority = live_task(world, goal.task).authority.clone();
    delegate.authority.delegation.depth = delegate.authority.delegation.depth.saturating_sub(1);
    delegate.authority.delegation.tasks = 6;
    delegate.authority.budget.spend = 40;
    let mut parameters = vec![
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: branch.into() },
        tasks::Parameter::Bytes { name: 17, value: request.into() },
    ];
    if let Some(extra) = extra {
        parameters.push(extra);
    }
    delegate.spec.parameters = parameters.into_boxed_slice();
    let answer = world.scripted_call(goal, 93, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let engine::CallAnswer::Delegated(made) = answer else { panic!("change admitted: {answer:?}") };
    made[0].task
}

fn next_job(world: &mut World, task: u64) -> host::JobAssignment {
    for _ in 0..STEPS {
        if !world.jobs.is_empty() {
            return world.jobs.remove(0);
        }
        world.tick();
    }
    panic!("close job not assigned: {:?}; errors {:?}", change_row(world, task), world.call_errors)
}

fn finish_job(world: &mut World, job: &host::JobAssignment) {
    let event = world.protocol.answer_job(
        CHANNEL,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished {
            result: br#"{"settled":[]}"#.as_slice().into(),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
}

#[test]
fn a_close_change_runs_the_close_job_at_a_branch_at_its_landing_branchs_tip() {
    use temper_protocol_worker_wire::v1;
    let (mut world, goal, branch, tip) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let job = next_job(&mut world, task);
    assert_eq!(job.kind, jig_domain_link::JobKind(forge_top::CLOSE_KIND));
    assert_eq!(job.deadline, Duration::from_secs(600));
    assert_eq!(live_task(&world, job.run.raw()).requester, tasks::Party::Task(task));
    let decoded = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&job.spec)).expect("close job spec");
    assert_eq!(decoded.command(), b"repo/.temper/close");
    assert_eq!(decoded.request(), REQUEST);
    assert_eq!(change_row(&world, task).close.as_deref(), Some(REQUEST));
    let workspace = job.workspace.as_ref().expect("writable workspace");
    assert_eq!(workspace.workstream, task);
    let items = &world.job_workspaces.get(&job.run.raw()).expect("typed job items").repositories;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].start, engine::ForgeStart::Branch(branch.clone()));
    assert_eq!(items[0].push.as_deref(), Some(change_row(&world, task).branch.as_ref()));
    assert_eq!(branch_tip(&mut world, &branch), tip);
    assert_eq!(world.assigned.len(), 1, "only the goal's agent exists");
    finish_job(&mut world, &job);
    for _ in 0..STEPS {
        if matches!(change_row(&world, task).change.state, change::State::NoChange { .. }) {
            break;
        }
        world.tick();
    }
    world.close();
}

#[test]
fn a_close_job_that_pushed_opens_and_lands_its_change() {
    let (mut world, goal, branch, tip) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let job = next_job(&mut world, task);
    let own = change_row(&world, task).branch.clone();
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: own.clone(), commit: tip })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &own, b"status", b"settled", 2)
        .expect("close job pushed");
    finish_job(&mut world, &job);
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
    for _ in 0..STEPS {
        if world.store.rows.contains_key(&key) {
            break;
        }
        world.tick();
    }
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) = world.store.rows.get(&key)
    else {
        panic!("change ended: {:?}", change_row(&world, task))
    };
    assert!(matches!(ended.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. }))));
    assert_eq!(world.merges, 1);
    assert!(matches!(change_row(&world, task).change.state, change::State::Landed { .. }));
    let number = change_row(&world, task).pull.expect("pull opened");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.base.as_ref(), branch.as_ref());
    assert_eq!(pull.commit, head);
    assert_eq!(pull.merged, Some(branch_tip(&mut world, &branch)));
    assert_eq!(world.assigned.len(), 1);
    world.close();
}

#[test]
fn a_close_job_that_pushed_nothing_ends_the_change_with_no_change() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let job = next_job(&mut world, task);
    finish_job(&mut world, &job);
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
    for _ in 0..STEPS {
        if world.store.rows.contains_key(&key) {
            break;
        }
        world.tick();
    }
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) = world.store.rows.get(&key)
    else {
        panic!("close ended: {:?}", change_row(&world, task))
    };
    assert!(
        matches!(&ended.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { words })) if words.as_ref() == b"Nothing to settle.")
    );
    assert!(change_row(&world, task).pull.is_none());
    assert_eq!(world.merges, 0);
    assert_eq!(world.assigned.len(), 1);
    world.close();
}

fn refused(command: bool, authority: bool, request: &[u8], extra: Option<tasks::Parameter>, expected: &[u8]) {
    let (mut world, goal, branch, _) = goal_world(command, authority);
    let task = make_change(&mut world, &goal, &branch, request, extra);
    for _ in 0..STEPS {
        if matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }) {
            break;
        }
        world.tick();
    }
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, task).phase else {
        panic!("start refused")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("refusal account") };
    assert!(words.starts_with(expected), "{}", core::str::from_utf8(words).expect("refusal text"));
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Change(task))));
    assert!(world.jobs.is_empty());
    assert_eq!(world.assigned.len(), 1);
    world.close();
}

#[test]
fn a_close_change_is_refused_without_a_close_job() {
    refused(false, true, REQUEST, None, b"change's close: the project names no close_job.");
}

#[test]
fn a_close_change_is_refused_without_the_close_kind() {
    refused(true, false, REQUEST, None, b"change's close: its authority does not delegate to job kind 4.");
}

#[test]
fn a_malformed_close_request_refuses_the_start() {
    for (request, extra) in [
        (b"not a close request".as_slice(), None),
        (REQUEST, Some(tasks::Parameter::Bytes { name: 16, value: forge_top::commit_text(&translate::commit(1)) })),
        (REQUEST, Some(tasks::Parameter::Bytes { name: 3, value: b"temper/handed".as_slice().into() })),
    ] {
        refused(true, true, request, extra, b"malformed close request parameter 17.");
    }
}

#[test]
fn a_failed_close_job_holds_the_change() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let mut last_job = None;
    for _ in 0..=world.env.limits.tasks.retries.permanent.retries {
        let job = next_job(&mut world, task);
        let event = world.protocol.answer_job(
            CHANNEL,
            job.run.raw(),
            job.attempt.raw(),
            host::JobEnding::Failed {
                failure: host::JobFailure::Undelivered(host::DeliveryOutcome::Refused),
                detail: b"close delivery refused".as_slice().into(),
                work: host::Work { left: None, saved: None },
            },
        );
        world.send(event);
        last_job = Some(job);
    }
    let job = last_job.expect("first attempt");
    for _ in 0..STEPS {
        if matches!(live_task(&world, task).escalation, tasks::Escalation::Waiting { .. }) {
            break;
        }
        world.tick();
    }
    assert!(matches!(
        live_task(&world, job.run.raw()).phase,
        tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Permanent), .. }
    ));
    assert!(matches!(change_row(&world, task).change.state, change::State::Held { why: change::Hold::Failed, .. }));
    assert!(matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }));
    assert!(
        matches!(live_task(&world, task).escalation, tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), .. } if holder == goal.task),
        "job escalation {:?}; change phase {:?}; policy {:?}",
        live_task(&world, job.run.raw()).escalation,
        live_task(&world, task).phase,
        change_row(&world, task).change.state
    );
    assert_eq!(world.merges, 0);
    assert_eq!(world.assigned.len(), 1);
    world.close();
}
