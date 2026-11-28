//! Close changes use a job producer (temper/docs/design/domain/forge/change/README.md, 1 and temper/docs/design/domain/forge/change/repairs.md and temper/docs/design/domain/forge/branches-closer.md, 1).
use super::goal_branch::{branch_tip, goal_request};
use super::light_rules::change_row;
use super::*;
use temper_engine_domain_forge_change as change;
use temper_protocol_worker_wire::v1::Record as WorkerRecord;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000;
const REQUEST: &[u8] = br#"{"version":1,"goal":7,"changes":[{"task":9,"landed":true}]}"#;
const COMMAND: &[u8] = b".temper/close";
const CUSTOM_CEILING: u32 = 1_800;
const DEFAULT_CEILING: u32 = 3_600;

fn goal_world(command: bool, authority: bool) -> (World, engine::Assignment, Box<[u8]>, u64) {
    goal_world_with_worker(command, authority, true)
}

fn goal_world_with_worker(
    command: bool,
    authority: bool,
    close_worker: bool,
) -> (World, engine::Assignment, Box<[u8]>, u64) {
    goal_world_with_landing_settings(command, authority, close_worker, None, None)
}

fn goal_world_with_landing_settings(
    command: bool,
    authority: bool,
    close_worker: bool,
    flakes: Option<forge_top::settings::Flakes>,
    hook_ceiling: Option<u32>,
) -> (World, engine::Assignment, Box<[u8]>, u64) {
    let landing_deployment = Box::new([engine::LandingRule {
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
    }]);
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            result_bytes: Some(512),
            close_job: Some((command.then(|| COMMAND.into()), authority)),
            landing_deployment: Some(landing_deployment),
            ..StoryLimits::default()
        },
    );
    if let Some(flakes) = flakes {
        world = world.with_flakes(flakes);
    }
    if let Some(ceiling) = hook_ceiling {
        world = world.with_hook_ceiling(ceiling);
    }
    world.adopt();
    if close_worker {
        hello_close_worker(&mut world);
    } else {
        world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    }
    let request = goal_request(&world, true);
    let goal = assigned_chat(&mut world, 92, request);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    let head = branch_tip(&mut world, &branch);
    (world, goal, branch, head)
}

fn hello_close_worker(world: &mut World) {
    let hosting = world
        .assigned
        .iter()
        .map(|run| protocol::hosting(run.task, run.attempt, jig_domain_link::Phase::Active))
        .collect();
    let mut event = protocol::hello_slots(CHANNEL, 2, hosting);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.jobs = Box::new([close_bound()]);
    world.send(event);
}

fn close_bound() -> jig_domain_link::JobBound {
    jig_domain_link::JobBound {
        kind: jig_domain_link::JobKind(forge_top::CLOSE_KIND),
        spec: 16_384,
        result: 512,
        deadline: Duration::from_secs(86_400),
    }
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
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 256, choices: Box::new([]) };
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
    ];
    if !request.is_empty() {
        parameters.push(tasks::Parameter::Bytes { name: 17, value: request.into() });
    }
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
    finish_job_on(world, CHANNEL, job);
}

pub(super) fn finish_job_on(world: &mut World, channel: Token, job: &host::JobAssignment) {
    finish_with_on(world, channel, job, br#"{"settled":[]}"#);
}

pub(super) fn finish_with(world: &mut World, job: &host::JobAssignment, result: &[u8]) {
    finish_with_on(world, CHANNEL, job, result);
}

fn finish_with_on(world: &mut World, channel: Token, job: &host::JobAssignment, result: &[u8]) {
    let event = world.protocol.answer_job(
        channel,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished { result: result.into(), work: host::Work { left: None, saved: None } },
    );
    world.send(event);
}

pub(super) const CLOSE_COUNTS: [u8; 32] = [0x33; 32];
pub(super) const CLOSE_FAILURE: &str = "test A::rules rule_00";

pub(super) fn check_refusal(head: [u8; 32], base: [u8; 32]) -> Box<[u8]> {
    refusal_with(head, Some((base, CLOSE_COUNTS)), &[CLOSE_FAILURE], 0)
}

pub(super) fn refusal_with(
    head: [u8; 32],
    against: Option<([u8; 32], [u8; 32])>,
    failures: &[&str],
    more: usize,
) -> Box<[u8]> {
    let head = refusal_commit(head);
    let (base, counts) = match against {
        Some((base, counts)) => (refusal_commit(base), refusal_commit(counts)),
        None => ("null".to_owned(), "null".to_owned()),
    };
    let tree = "2222222222222222222222222222222222222222";
    let failures = serde_json::to_string(failures).expect("failure names as JSON");
    let more = if more == 0 { String::new() } else { format!(",\"more\":{more}") };
    format!("{{\"refused\":{{\"head\":{head},\"tree\":\"{tree}\",\"base\":{base},\"counts\":{counts},\"failures\":{failures}{more}}}}}\n")
        .into_bytes()
        .into_boxed_slice()
}

fn refusal_commit(commit: [u8; 32]) -> String {
    let hex = forge_top::commit_text(&commit);
    let hex = core::str::from_utf8(&hex).expect("commit hex");
    format!("\"{hex}\"")
}

#[test]
fn run_and_close_workspace_items_carry_restored_project_bounds() {
    use forge_top::settings::Flakes;
    for (flakes, expected, ceiling, expected_ceiling) in [
        (Some(Flakes::Never), Some(0), Some(CUSTOM_CEILING), CUSTOM_CEILING),
        (Some(Flakes::Rerun(3)), Some(3), Some(CUSTOM_CEILING), CUSTOM_CEILING),
        (None, None, None, DEFAULT_CEILING),
    ] {
        let (mut world, goal, branch, _) = goal_world_with_landing_settings(true, true, true, flakes, ceiling);
        let change = make_change(&mut world, &goal, &branch, REQUEST, None);
        let job = next_job(&mut world, change);
        let items = world.job_workspaces.get(&job.run.raw()).expect("close job items");
        assert_eq!((goal.workspace.rerun, items.rerun), (expected, expected), "flakes {flakes:?}");
        assert_eq!(goal.workspace.hook_timeout_max_seconds, expected_ceiling);
        assert_eq!(items.hook_timeout_max_seconds, expected_ceiling);
        assert_eq!(job.deadline, Duration::from_secs(u64::from(expected_ceiling)));
        finish_job(&mut world, &job);
        world.close();
    }
}

#[test]
fn a_close_job_starts_at_the_tip_read_before_assignment() {
    use temper_protocol_worker_wire::v1;
    let (mut world, goal, branch, tip) = goal_world_with_worker(true, true, false);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    for _ in 0..STEPS {
        if change_row(&world, task).close_at.is_some() {
            break;
        }
        world.tick();
    }
    let moved = advance_goal_tip(&mut world, &goal, &branch);
    world.send(protocol::lost(CHANNEL));
    hello_close_worker(&mut world);
    let job = next_job(&mut world, task);
    assert_eq!(job.kind, jig_domain_link::JobKind(forge_top::CLOSE_KIND));
    assert_eq!(job.deadline, Duration::from_secs(u64::from(DEFAULT_CEILING)));
    assert_eq!(live_task(&world, job.run.raw()).requester, tasks::Party::Task(task));
    let decoded = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&job.spec)).expect("close job spec");
    assert_eq!(decoded.command.as_ref(), b"repo/.temper/hooks/close");
    assert!(decoded.find && decoded.refusal);
    assert_eq!(decoded.fallback.as_ref(), b"repo/.temper/close");
    assert_eq!(decoded.request.as_ref(), REQUEST);
    assert_eq!(change_row(&world, task).close.as_deref(), Some(REQUEST));
    let workspace = job.workspace.as_ref().expect("writable workspace");
    assert_eq!(workspace.workstream, task);
    let items = &world.job_workspaces.get(&job.run.raw()).expect("typed job items").repositories;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].start, engine::ForgeStart::Commit(translate::commit(tip)));
    assert_eq!(items[0].push.as_deref(), Some(change_row(&world, task).branch.as_ref()));
    assert_eq!(branch_tip(&mut world, &branch), moved);
    assert_ne!(moved, tip);
    assert!(change_row(&world, task).close_at.expect("recorded attempt").asked);
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
    let head = world.worker_commit(tip, b"status", b"settled");
    world.job_push(CHANNEL, &job, &own, head, 2);
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
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::ChangeHot(task))));
    assert_eq!(world.jobs.len(), 0);
    assert_eq!(world.assigned.len(), 1);
    world.close();
}

#[test]
fn a_close_change_is_refused_without_close_job_authority() {
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
fn a_close_change_ignores_legacy_hook_parameter_twenty() {
    use temper_protocol_worker_wire::v1;
    for legacy in [None, Some(forge_top::CLOSE_HOOK), Some(b"/ignored")] {
        let (mut world, goal, branch, _) = goal_world(false, true);
        let parameter = legacy.map(|path| tasks::Parameter::Bytes { name: 20, value: path.into() });
        let task = make_change(&mut world, &goal, &branch, REQUEST, parameter);
        let job = next_job(&mut world, task);
        let spec = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&job.spec)).expect("close job spec");
        assert_eq!(spec.command.as_ref(), b"repo/.temper/hooks/close");
        assert_eq!(spec.request.as_ref(), REQUEST);
        assert!(spec.find && spec.refusal);
        assert!(spec.fallback.is_empty());
        world.close();
    }
}

fn fail_close_until_held(
    world: &mut World,
    task: u64,
    mut job: host::JobAssignment,
    failure: host::JobFailure,
    detail: &[u8],
) {
    let retries = match failure {
        host::JobFailure::Failed | host::JobFailure::Deadline => world.env.limits.tasks.retries.run.retries,
        host::JobFailure::Cancelled(host::Reason::Contact) => world.env.limits.tasks.retries.lost.retries,
        host::JobFailure::Undelivered(host::DeliveryOutcome::Refused) => {
            world.env.limits.tasks.retries.permanent.retries
        }
        unexpected => panic!("unexpected close fault fixture: {unexpected:?}"),
    };
    let child = job.run.raw();
    for attempt in 0..=retries {
        if attempt != 0 {
            job = next_job(world, task);
            assert_eq!(job.run.raw(), child, "the retry belongs to the original job");
        }
        let event = world.protocol.answer_job(
            CHANNEL,
            job.run.raw(),
            job.attempt.raw(),
            host::JobEnding::Failed { failure, detail: detail.into(), work: host::Work { left: None, saved: None } },
        );
        world.send(event);
    }
    for _ in 0..STEPS {
        if matches!(live_task(world, task).escalation, tasks::Escalation::Waiting { .. })
            && change_row(world, task)
                .producer_failure
                .as_deref()
                .is_some_and(|words| words.windows(detail.len()).any(|part| part == detail))
        {
            break;
        }
        world.tick();
    }
    assert!(matches!(
        change_row(world, task).change.state,
        change::State::Held { hold: change::Hold { cause: change::Cause::Failed, .. }, .. }
    ));
    assert!(matches!(live_task(world, task).phase, tasks::Phase::Held { .. }));
    assert!(
        change_row(world, task)
            .producer_failure
            .as_deref()
            .expect("failure words")
            .windows(detail.len())
            .any(|part| part == detail)
    );
}

fn release_change(world: &mut World, goal: &engine::Assignment, task: u64) {
    let revision = live_task(world, task).revisions.hold;
    let answer = world.scripted_call(goal, 95, engine::Tool::Release { target: task, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
}

fn advance_goal_tip(world: &mut World, goal: &engine::Assignment, branch: &[u8]) -> u64 {
    let tip = branch_tip(world, branch);
    super::goal_delivery::deliver(world, goal, tip, b"docs/design/close.md")
}

fn assert_fresh_workspace(
    world: &World,
    task: u64,
    job: &host::JobAssignment,
    previous: &[engine::ForgeRepository],
    tip: u64,
) {
    assert_eq!(job.workspace.as_ref().expect("writable workstream").workstream, task);
    let fresh = &world.job_workspaces.get(&job.run.raw()).expect("fresh workspace").repositories;
    let ([fresh], [previous]) = (fresh.as_ref(), previous) else { panic!("one writable repository per job") };
    assert!(fresh.save.is_some());
    assert_ne!(fresh.save, previous.save, "a fresh job has its own saved-work branch");
    let mut expected = previous.clone();
    expected.start = engine::ForgeStart::Commit(translate::commit(tip));
    expected.save.clone_from(&fresh.save);
    assert_eq!(fresh, &expected);
}

#[test]
fn a_fresh_close_job_keeps_its_changes_writable_workspace() {
    let (mut world, goal, branch, tip) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    let items = world.job_workspaces.get(&child).expect("first workspace").repositories.clone();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].start, engine::ForgeStart::Commit(translate::commit(tip)));
    assert_eq!(items[0].push.as_deref(), Some(change_row(&world, task).branch.as_ref()));
    fail_close_until_held(&mut world, task, first, host::JobFailure::Failed, b"exit 1: close: unknown tag");
    let tip = advance_goal_tip(&mut world, &goal, &branch);
    release_change(&mut world, &goal, task);
    let retry = next_job(&mut world, task);
    assert_ne!(retry.run.raw(), child);
    assert_fresh_workspace(&world, task, &retry, &items, tip);
    finish_job(&mut world, &retry);
    world.close();
}

fn close_ending(world: &mut World, task: u64) -> tasks::Ending {
    for _ in 0..STEPS {
        if let Some(ended) = super::goal_landing::ended(world, task) {
            let tasks::Phase::Ended(ending) = &ended.phase else { panic!("terminal phase") };
            return ending.clone();
        }
        world.tick();
    }
    panic!("close change did not end: {:?}", change_row(world, task))
}

#[test]
fn a_fresh_close_jobs_push_opens_and_lands_its_change() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    fail_close_until_held(&mut world, task, first, host::JobFailure::Failed, b"exit 1: close: unknown tag");
    let tip = advance_goal_tip(&mut world, &goal, &branch);
    release_change(&mut world, &goal, task);
    let retry = next_job(&mut world, task);
    assert_ne!(retry.run.raw(), child);
    let own = change_row(&world, task).branch.clone();
    let head = world.worker_commit(tip, b"status", b"settled");
    world.job_push(CHANNEL, &retry, &own, head, 2);
    finish_job(&mut world, &retry);
    assert!(matches!(close_ending(&mut world, task), tasks::Ending::Done(tasks::TaskResult::Change { .. })));
    assert!(matches!(change_row(&world, task).change.state, change::State::Landed { .. }));
    assert_eq!(world.merges, 1);
    let number = change_row(&world, task).pull.expect("fresh job opened pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.base, branch);
    assert_eq!(pull.commit, head);
    assert_eq!(pull.merged, Some(branch_tip(&mut world, &branch)));
    world.close();
}

#[test]
fn a_fresh_close_job_with_nothing_to_push_ends_its_change() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    fail_close_until_held(&mut world, task, first, host::JobFailure::Failed, b"exit 1: close: unknown tag");
    advance_goal_tip(&mut world, &goal, &branch);
    release_change(&mut world, &goal, task);
    let retry = next_job(&mut world, task);
    assert_ne!(retry.run.raw(), child);
    finish_job(&mut world, &retry);
    assert_eq!(
        close_ending(&mut world, task),
        tasks::Ending::Done(tasks::TaskResult::NoChange { words: b"Nothing to settle.".as_slice().into() })
    );
    assert!(change_row(&world, task).pull.is_none());
    assert_eq!(world.merges, 0);
    world.close();
}

#[test]
fn a_fresh_close_job_that_fails_holds_its_change_with_the_new_words() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    fail_close_until_held(&mut world, task, first, host::JobFailure::Failed, b"exit 1: close: unknown tag");
    let revision = live_task(&world, task).revisions.hold;
    advance_goal_tip(&mut world, &goal, &branch);
    release_change(&mut world, &goal, task);
    let retry = next_job(&mut world, task);
    assert_ne!(retry.run.raw(), child);
    fail_close_until_held(&mut world, task, retry, host::JobFailure::Failed, b"exit 1: close: another unknown tag");
    assert!(live_task(&world, task).revisions.hold > revision);
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, task).phase else {
        panic!("change held with account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("failure account") };
    assert!(core::str::from_utf8(words).expect("hold words").contains("exit 1: close: another unknown tag"));
    assert_eq!(change_row(&world, task).delegate, None);
    world.close();
}

#[test]
fn releasing_a_broken_close_rechecks_unchanged_tips() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    let items = world.job_workspaces.get(&child).expect("first workspace").repositories.clone();
    assert_eq!(live_task(&world, child).authority.budget.spend, 0, "close jobs reserve no spend");
    assert_eq!(live_task(&world, task).numbers.reserved, 0);
    fail_close_until_held(&mut world, task, first, host::JobFailure::Failed, b"exit 1: close: unknown tag");
    let old = super::goal_landing::ended(&world, child).expect("failed job already ended before release").clone();
    let tasks::Phase::Ended(tasks::Ending::Cancelled { reason, .. }) = &old.phase else {
        panic!("failed job: {:?}", old.phase)
    };
    assert_eq!(reason.as_ref(), b"its change holds on its failure");
    let before = change_row(&world, task).close_at.expect("first job recorded its tips");
    release_change(&mut world, &goal, task);
    let fresh = next_job(&mut world, task);
    assert_ne!(fresh.run.raw(), child);
    assert_eq!(super::goal_landing::ended(&world, child), Some(&old));
    let fresh_items = &world.job_workspaces.get(&fresh.run.raw()).expect("fresh writable workspace").repositories;
    assert_eq!(fresh_items.len(), 1);
    assert_eq!(fresh_items[0].start, items[0].start);
    assert_eq!(fresh_items[0].push, items[0].push);
    assert_eq!(live_task(&world, fresh.run.raw()).authority.budget.spend, 0);
    assert_eq!(change_row(&world, task).delegate, Some((fresh.run.raw(), change::Delegate::Close)));
    assert_eq!(change_row(&world, task).close_at, Some(before));
    finish_job(&mut world, &fresh);
    assert!(matches!(close_ending(&mut world, task), tasks::Ending::Done(tasks::TaskResult::NoChange { .. })));
    world.close();
}

#[test]
fn measuring_a_refused_close_runs_one_fresh_job_at_unchanged_tips() {
    let (mut world, goal, branch, tip) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let landing = translate::commit(branch_tip(&mut world, b"main"));
    finish_with(&mut world, &first, &check_refusal(translate::commit(tip), landing));
    super::goal_closer::wait_held(&mut world, task, goal.task);
    let revision = live_task(&world, task).revisions.hold;
    let tool = engine::Tool::Measure { target: task, revision };
    let answer = world.scripted_call(&goal, 95, tool);
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    let fresh = next_job(&mut world, task);
    assert_ne!(fresh.run.raw(), first.run.raw());
    let row = change_row(&world, task);
    let accepted = row.task_release.expect("measurement release was kept");
    assert_eq!((accepted.revision, accepted.measure), (revision, true));
    assert_eq!(
        row.close_at,
        Some(change::CloseAt { branch: translate::commit(tip), landing, asked: true, refused: None, outdated: None })
    );
    finish_job(&mut world, &fresh);
    assert!(matches!(close_ending(&mut world, task), tasks::Ending::Done(tasks::TaskResult::NoChange { .. })));
    assert!(world.jobs.is_empty(), "one accepted measurement made one fresh job");
    world.close();
}

#[test]
fn moving_main_alone_rechecks_a_refused_close() {
    let (mut world, goal, branch, tip) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    let landing = branch_tip(&mut world, b"main");
    finish_with(&mut world, &first, &check_refusal(translate::commit(tip), translate::commit(landing)));
    super::goal_closer::wait_held(&mut world, task, goal.task);
    let landing = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"counts", b"updated", 1)
        .expect("main moved");
    release_change(&mut world, &goal, task);
    let fresh = next_job(&mut world, task);
    assert_ne!(fresh.run.raw(), child);
    assert_eq!(
        change_row(&world, task).close_at,
        Some(change::CloseAt {
            branch: translate::commit(tip),
            landing: translate::commit(landing),
            asked: true,
            refused: None,
            outdated: None,
        })
    );
    finish_job(&mut world, &fresh);
    world.close();
}

#[test]
fn a_fresh_close_job_keeps_its_changes_writable_workspace_after_restart() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let first = next_job(&mut world, task);
    let child = first.run.raw();
    let items = world.job_workspaces.get(&child).expect("first workspace").repositories.clone();
    fail_close_until_held(&mut world, task, first, host::JobFailure::Failed, b"exit 1: close: unknown tag");
    let tip = advance_goal_tip(&mut world, &goal, &branch);
    world.drain_cut();
    world.restart_same();
    // The reconnecting worker advertises the same job kind as before restart.
    for event in &mut world.events {
        if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = event
        {
            hello.jobs = Box::new([close_bound()]);
        }
    }
    world.until(Until::Ready);
    release_change(&mut world, &goal, task);
    let retry = next_job(&mut world, task);
    assert_ne!(retry.run.raw(), child);
    assert_fresh_workspace(&world, task, &retry, &items, tip);
    finish_job(&mut world, &retry);
    assert!(matches!(close_ending(&mut world, task), tasks::Ending::Done(tasks::TaskResult::NoChange { .. })));
    world.close();
}

#[test]
fn a_broken_close_holds_after_its_run_tries() {
    for (failure, detail, expected) in [
        (
            host::JobFailure::Failed,
            b"exit 1: close: command failed".as_slice(),
            "the close check broke and didn't judge the goal: exit 1: close: command failed; its release runs it again.",
        ),
        (
            host::JobFailure::Deadline,
            b"command timed out".as_slice(),
            "the close check broke and didn't judge the goal: command timed out; its release runs it again.",
        ),
    ] {
        let (mut world, goal, branch, _) = goal_world(true, true);
        let task = make_change(&mut world, &goal, &branch, REQUEST, None);
        let job = next_job(&mut world, task);
        fail_close_until_held(&mut world, task, job, failure, detail);
        super::goal_closer::assert_hold_words(&world, task, expected);
        world.close();
    }
}

#[test]
fn a_lost_close_holds_after_its_runner_tries() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let job = next_job(&mut world, task);
    fail_close_until_held(
        &mut world,
        task,
        job,
        host::JobFailure::Cancelled(host::Reason::Contact),
        b"worker contact lost",
    );
    super::goal_closer::assert_hold_words(
        &world,
        task,
        "the close check's runner was lost and didn't judge the goal: worker contact lost; its release runs it again.",
    );
    world.close();
}

#[test]
fn an_unrecognized_finished_close_holds_without_another_try() {
    const OTHER_HEAD: [u8; 32] = [0x12; 32];
    let wrong_head = check_refusal(OTHER_HEAD, OTHER_HEAD);
    for (result, reason) in [
        (br#"{"other":1}"#.as_slice(), r#"{"other":1}"#),
        (b"not JSON".as_slice(), "not JSON"),
        (wrong_head.as_ref(), r#"{"refused":{"head":"12121212121212121212121212121212121212121212"#),
    ] {
        let (mut world, goal, branch, _) = goal_world(true, true);
        let task = make_change(&mut world, &goal, &branch, REQUEST, None);
        let job = next_job(&mut world, task);
        finish_with(&mut world, &job, result);
        super::goal_closer::wait_held(&mut world, task, goal.task);
        let expected = format!(
            "the close check broke and didn't judge the goal: it finished with {reason}; its release runs it again."
        );
        super::goal_closer::assert_hold_words(&world, task, &expected);
        assert!(matches!(
            change_row(&world, task).change.state,
            change::State::Held { hold: change::Hold { cause: change::Cause::Failed, .. }, .. }
        ));
        assert_eq!(change_row(&world, task).close_at.expect("close attempt").refused, None);
        assert!(world.jobs.is_empty(), "a finished invalid result must not spend another try");
        world.close();
    }
}

#[test]
fn a_failed_close_job_holds_the_change() {
    let (mut world, goal, branch, _) = goal_world(true, true);
    let task = make_change(&mut world, &goal, &branch, REQUEST, None);
    let job = next_job(&mut world, task);
    let child = job.run.raw();
    super::goal_closes::park_goal(&mut world, &goal);
    fail_close_until_held(
        &mut world,
        task,
        job,
        host::JobFailure::Undelivered(host::DeliveryOutcome::Refused),
        b"close delivery refused",
    );
    for _ in 0..STEPS {
        if super::goal_landing::ended(&world, child).is_some() && world.assigned.len() > 1 {
            break;
        }
        world.tick();
    }
    let ended = super::goal_landing::ended(&world, child).expect("failed close job must end, not stay held");
    let tasks::Phase::Ended(tasks::Ending::Cancelled { reason, .. }) = &ended.phase else {
        panic!("failed close job: {:?}", ended.phase)
    };
    assert_eq!(reason.as_ref(), b"its change holds on its failure");
    assert!(!matches!(ended.escalation, tasks::Escalation::Waiting { .. }));
    let row = change_row(&world, task);
    let change::State::Held { was, hold } = &row.change.state else { panic!("close held at producing") };
    assert_eq!(**was, change::State::Producing { requested: true });
    assert_eq!(hold.cause, change::Cause::Failed);
    assert_eq!(row.delegate, None);
    assert_eq!(row.delegate_status, change::Status::Failed);
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, task).phase else {
        panic!("change held with account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("failure account") };
    assert_eq!(Some(words.as_ref()), row.producer_failure.as_deref());
    assert!(matches!(live_task(&world, task).escalation, tasks::Escalation::Waiting {
        holder: tasks::EscalationHolder::Task(holder), ..
    } if holder == goal.task));
    assert_eq!(world.merges, 0);
    assert_eq!(world.assigned.len(), 2);
    let wake = &world.assigned[1];
    assert_eq!(wake.task, goal.task);
    let escalations: Vec<_> = wake
        .messages
        .iter()
        .filter_map(|word| match word.kind {
            tasks::MessageKind::Escalation { task, .. } => Some(task),
            _ => None,
        })
        .collect();
    assert_eq!(escalations, [task]);
    world.close();
}
