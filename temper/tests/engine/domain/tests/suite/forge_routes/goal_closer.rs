//! Goal closer procedure stories (temper/docs/design/domain/forge/branches.md, 2).
use super::goal_branch::{branch_tip, forge_hold, goal_request};
use super::goal_landing::{ended, finish_run, mine, row, status};
use super::*;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_closer as closer;

#[path = "closer_questions.rs"]
mod questions;

const STEPS: u32 = 1000;
const CHANNEL: Token = Token::new(7);

fn goal_world(close: bool, branch: bool) -> (World, engine::Assignment, Box<[u8]>) {
    goal_world_in(close, branch, None, false)
}
fn goal_world_in(
    close: bool,
    branch: bool,
    tier: Option<temper_engine_domain_world::tier_runs::Tier>,
    main_ci: bool,
) -> (World, engine::Assignment, Box<[u8]>) {
    goal_world_with_ci(close, branch, tier, main_ci, true, STEPS, None)
}
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "stories vary the close job, goal branch, landing rule and CI silence independently"
)]
fn goal_world_with_ci(
    close: bool,
    branch: bool,
    tier: Option<temper_engine_domain_world::tier_runs::Tier>,
    main_ci: bool,
    silent_ci: bool,
    passes: u32,
    commits: Option<u32>,
) -> (World, engine::Assignment, Box<[u8]>) {
    goal_world_with_updates(close, branch, tier, main_ci, silent_ci, passes, commits, false, None, 1)
}

#[expect(
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools,
    reason = "the update stories reuse the closer fixture with an independent main containment rule"
)]
fn goal_world_with_updates(
    close: bool,
    branch: bool,
    tier: Option<temper_engine_domain_world::tier_runs::Tier>,
    main_ci: bool,
    silent_ci: bool,
    passes: u32,
    commits: Option<u32>,
    up_to_date: bool,
    stall: Option<Duration>,
    attempts: u8,
) -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        silent_ci,
        passes,
        Gate::default(),
        None,
        StoryLimits {
            inbox_messages: Some(16),
            inbox_bytes: Some(16_384),
            tier,
            slots: Some(4),
            tasks: Some(32),
            tree_tasks: Some(64),
            task_delegates: Some(16),
            authority_batch: Some(8),
            tasks_batch: Some(8),
            result_bytes: Some(4096),
            forge_commits: commits,
            stall,
            ..StoryLimits::default()
        },
    )
    .with_close_job(close.then_some(b".temper/close".as_slice()), true)
    .with_goal_closer()
    .with_landing_attempts(attempts)
    .with_landing_deployment(Box::new([
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: Box::new([
                    b"forge".as_slice().into(),
                    b"forge.example".as_slice().into(),
                    b"org".as_slice().into(),
                    b"repo".as_slice().into(),
                    b"branch".as_slice().into(),
                ]),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: main_ci,
            up_to_date,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: Box::new([
                    b"forge".as_slice().into(),
                    b"forge.example".as_slice().into(),
                    b"org".as_slice().into(),
                    b"repo".as_slice().into(),
                    b"branch".as_slice().into(),
                ]),
                last: authority::Last::Open(b"temper".as_slice().into()),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
    ]));
    if commits.is_some() {
        let client = skein_forge_client::Limits { rows: 8, ..world.env.limits.forge.client };
        world = world.with_forge_client(client);
    }
    world.adopt();
    let mut event = protocol::hello_slots(CHANNEL, 4, Box::new([]));
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
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal request") };
    batch[0].words = b"TITLE\n\nBuild PATH.\nDraft: PATH".as_slice().into();
    batch[0].contract = people::Contract::Report { words: 4096 };
    batch[0].flags.ended_by = people::EndedBy::Closer(people::Closer { connector: 0, code: 6 });
    if !branch {
        batch[0].symbols = Box::new([]);
    }
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = if branch {
        format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice()
    } else {
        b"main".as_slice().into()
    };
    (world, goal, branch)
}

fn make_closer(world: &mut World, goal: &engine::Assignment, call: u64) -> u64 {
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(
        b"Close your requester: nothing more will come.",
        Box::new([]),
    );
    delegate.executor = tasks::Executor::Procedure { connector: 0, code: 6 };
    delegate.contract = live_task(world, goal.task).contract.clone();
    delegate.authority = live_task(world, goal.task).authority.clone();
    delegate.authority.delegation.depth = delegate.authority.delegation.depth.saturating_sub(1);
    delegate.authority.delegation.tasks = 32;
    delegate.authority.budget.spend = 120;
    let answer = world.scripted_call(goal, call, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let engine::CallAnswer::Delegated(made) = answer else { panic!("closer admitted: {answer:?}") };
    made[0].task
}
pub(super) fn closer_row(world: &World, task: u64) -> Option<&forge_top::CloserRow> {
    match world.store.rows.get(&Key::Forge(forge_top::Key::Closer(task))) {
        Some(Record::Forge { row }) => match row.as_ref() {
            forge_top::Stored::Closer(row) => Some(row),
            _ => panic!("closer row"),
        },
        _ => None,
    }
}
pub(super) fn wait_ended(world: &mut World, task: u64) {
    for _ in 0..4000 {
        if ended(world, task).is_some() {
            return;
        }
        world.tick();
    }
    panic!(
        "task {task} did not end; closer {:?}; errors {:?}; phase {:?}",
        closer_row(world, task),
        world.call_errors,
        live_task(world, task).phase
    )
}
pub(super) fn result_words(world: &World, task: u64) -> &[u8] {
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) =
        &ended(world, task).expect("closer ended").phase
    else {
        panic!("closer report")
    };
    words
}
fn add_change(
    world: &mut World,
    goal: &engine::Assignment,
    branch: &[u8],
    call: u64,
    title: &[u8],
    report: &[u8],
) -> u64 {
    let assigned = world.assigned.len();
    let answer = mine(world, goal, call, br#"{"words":"Add to the goal"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else {
        panic!("change admitted: {answer:?}; errors {:?}", world.call_errors)
    };
    let task = made[0].task;
    let producer = reviews::next_assignment(world, assigned);
    produce(world, task, branch, &producer, title, report);

    task
}
pub(super) fn close_job(world: &mut World) -> host::JobAssignment {
    for _ in 0..STEPS {
        if !world.jobs.is_empty() {
            return world.jobs.remove(0);
        }
        world.tick();
    }
    panic!("close job was not assigned; errors {:?}", world.call_errors)
}
pub(super) fn finish_close(world: &mut World, task: u64, branch: &[u8], job: &host::JobAssignment) {
    let own = row(world, task).branch.clone();
    let tip = branch_tip(world, branch);
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: own.clone(), commit: tip })),
        raw::Answer::Branch(raw::Created::Created)
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &own, b"status", &job.run.raw().to_be_bytes(), 2)
        .expect("close job pushed");
    let event = world.protocol.answer_job(
        CHANNEL,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished {
            result: br#"{"settled":["status"]}"#.as_slice().into(),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
}
pub(super) fn wait_landing(world: &mut World, closer: u64) -> u64 {
    for _ in 0..STEPS {
        if let Some(task) = closer_row(world, closer).and_then(|row| row.closer.landing)
            && world.store.rows.contains_key(&Key::Forge(forge_top::Key::Change(task)))
        {
            return task;
        }
        world.tick();
    }
    panic!(
        "landing not made; closer {:?}; phase {:?}; inbox {:?}",
        closer_row(world, closer),
        live_task(world, closer).phase,
        live_task(world, closer).inbox.iter().map(|w| (&w.kind, core::str::from_utf8(&w.words))).collect::<Vec<_>>()
    )
}

pub(super) fn producer_for(world: &mut World, task: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        let child = row(world, task).delegate.map(|(child, _)| child);
        if let Some(run) = world.assigned.iter().find(|run| Some(run.task) == child) {
            return run.clone();
        }
        world.tick();
    }
    panic!("producer for {task}: {:?}; {:?}", row(world, task).change.state, live_task(world, task).phase)
}
fn assigned(world: &mut World, task: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        if let Some(run) = world.assigned.iter().find(|run| run.task == task) {
            return run.clone();
        }
        world.tick();
    }
    panic!("assignment for {task}")
}
fn produce(world: &mut World, task: u64, branch: &[u8], producer: &engine::Assignment, title: &[u8], report: &[u8]) {
    produce_with_status(world, task, branch, producer, title, report, Some(raw::Check::Passed));
}
fn produce_with_status(
    world: &mut World,
    task: u64,
    branch: &[u8],
    producer: &engine::Assignment,
    title: &[u8],
    report: &[u8],
    check: Option<raw::Check>,
) {
    let own = row(world, task).branch.clone();
    let tip = branch_tip(world, branch);
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: own.clone(), commit: tip })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let contents = if title == b"failures" { temper_engine_domain_world::tier_runs::FAILURES } else { b"added" };
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &own, title, contents, 1)
        .expect("producer pushed");
    if let Some(check) = check {
        status(world, head, check);
    }
    finish_run(
        world,
        producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(title.into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: report.into(),
        },
    );
    wait_ended(world, task);
}
fn ordinary_delegate(world: &World, goal: &engine::Assignment, words: &[u8]) -> engine::Delegate {
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(words, Box::new([]));
    delegate.authority = live_task(world, goal.task).authority.clone();
    delegate.authority.delegation = tasks::Delegation { tasks: 0, depth: 0, kinds: Box::new([]) };
    delegate.authority.budget.spend = 10;
    delegate
}
fn numbered_history(world: &mut World, goal: &engine::Assignment, branch: &[u8]) {
    // Reserve the earlier tree tasks so the history's task numbers are 9, 10 and 12.
    for call in 200..208 {
        let delegate = ordinary_delegate(world, goal, b"Earlier work");
        let engine::CallAnswer::Delegated(made) =
            world.scripted_call(goal, call, engine::Tool::Delegate { batch: Box::new([delegate]) })
        else {
            panic!("earlier work admitted")
        };
        let task = made[0].task;
        assert!(task <= 8);
        let run = assigned(world, task);
        finish_run(world, &run, tasks::TaskResult::Report { words: b"done".as_slice().into() });
        wait_ended(world, task);
        if task == 8 {
            break;
        }
    }
    let mut batch = Vec::new();
    for words in [b"First".as_slice(), b"Cancelled", b"Unrelated report", b"Last"] {
        let mut delegate = ordinary_delegate(world, goal, words);
        if words != b"Unrelated report" {
            delegate.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
            delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 4096 };
            delegate.authority = live_task(world, goal.task).authority.clone();
            delegate.authority.delegation.depth -= 1;
            delegate.authority.delegation.tasks = 6;
            delegate.authority.budget.spend = 60;
            delegate.spec.parameters = Box::new([
                tasks::Parameter::Resource {
                    name: 1,
                    connector: 0,
                    resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                },
                tasks::Parameter::Bytes { name: 2, value: branch.into() },
            ]);
        }
        batch.push(delegate);
    }
    let answer = world.scripted_call(goal, 300, engine::Tool::Delegate { batch: batch.into_boxed_slice() });
    let engine::CallAnswer::Delegated(made) = answer else { panic!("history batch: {answer:?}") };
    assert_eq!(made.iter().map(|task| task.task).collect::<Vec<_>>(), [9, 10, 11, 12]);
    let report = assigned(world, 11);
    finish_run(world, &report, tasks::TaskResult::Report { words: b"done".as_slice().into() });
    let first = producer_for(world, 9);
    produce(world, 9, branch, &first, b"First", b"Implemented.\n\n## Left:\nOne item\n# Checks\npassed");
    let cancelled = producer_for(world, 10);
    let answer = world.scripted_call(
        goal,
        301,
        engine::Tool::Cancel { target: 10, reason: b"drop this change".as_slice().into() },
    );
    assert!(matches!(answer, engine::CallAnswer::Controlled), "{answer:?}");
    finish_run(world, &cancelled, tasks::TaskResult::NoChange { words: b"cancelled".as_slice().into() });
    wait_ended(world, 10);
    let last = producer_for(world, 12);
    produce(world, 12, branch, &last, b"Last", b"No left section");
}

#[test]
fn a_closer_closes_lands_and_reports() {
    let (mut world, goal, branch) =
        goal_world_in(true, true, Some(temper_engine_domain_world::tier_runs::Tier::default()), false);
    numbered_history(&mut world, &goal, &branch);
    let closer = make_closer(&mut world, &goal, 400);
    let job = close_job(&mut world);
    let close = live_task(&world, job.run.raw()).requester;
    let tasks::Party::Task(close) = close else { panic!("job's close change") };
    let expected = format!(
        "{{\"version\":1,\"goal\":{},\"changes\":[{{\"task\":9,\"landed\":true}},{{\"task\":10,\"landed\":false}},{{\"task\":12,\"landed\":true}}]}}",
        goal.task
    );
    assert_eq!(row(&world, close).close.as_deref(), Some(expected.as_bytes()));
    finish_close(&mut world, close, &branch, &job);
    wait_ended(&mut world, close);
    let tip = branch_tip(&mut world, &branch);
    status(&mut world, tip, raw::Check::Passed);
    let landing = wait_landing(&mut world, closer);
    assert_eq!(row(&world, landing).branch, branch);
    assert_eq!(row(&world, landing).base.as_ref(), b"main");
    assert_eq!(row(&world, landing).body.as_ref(), b"TITLE\n\nDraft: PATH");
    assert_eq!(row(&world, landing).title.as_ref(), b"TITLE");
    assert_eq!(forge_hold(&world, &branch).expect("goal holds g").task, goal.task);
    assert!(live_task(&world, closer).holdings.is_empty());
    assert!(live_task(&world, landing).holdings.is_empty());
    assert_eq!(row(&world, landing).delegate, None);
    wait_ended(&mut world, closer);
    assert!(closer_row(&world, closer).expect("closer retained").done);
    let change::State::Cleared { merge } = row(&world, landing).change.state else { panic!("landing cleared") };
    let landed = [9, 12, close].map(|task| {
        let change = row(&world, task);
        closer::Landed {
            task,
            title: change.title.clone(),
            pull: change.pull.expect("landed pull"),
            branch: branch.clone(),
            left: if task == 9 { Some(b"## Left:\nOne item".as_slice().into()) } else { None },
        }
    });
    let expected = closer::Report {
        goal: goal.task,
        title: b"TITLE".as_slice().into(),
        landed: landed.into(),
        unlanded: Box::new([closer::Unlanded {
            task: 10,
            title: b"Cancelled".as_slice().into(),
            why: closer::Why::Cancelled,
        }]),
        landing: closer::Landing::Landed {
            pull: row(&world, landing).pull.expect("landing pull"),
            base: b"main".as_slice().into(),
            commit: forge_top::commit_text(&merge),
            repairs: Box::new([]),
            resolutions: Box::new([]),
        },
        selections: Box::new([closer::Selection {
            pull: row(&world, landing).pull.expect("landing pull"),
            line: None,
        }]),
    };
    assert_eq!(result_words(&world, closer), closer::words(&expected, 4096).as_ref());
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

#[test]
fn a_far_diverged_goal_closes_lands_and_reports_without_comparisons() {
    use skein_forge::api::Read;
    let (mut world, goal, branch) = goal_world_with_ci(true, true, None, false, true, STEPS, Some(64));
    world.forge_reads = Some(Vec::new());
    let mut goal_tips = vec![translate::commit(branch_tip(&mut world, &branch))];
    for n in 0_u8..21 {
        let tip = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"goal", &[n], 1)
            .expect("goal advances");
        goal_tips.push(translate::commit(tip));
    }
    for n in 0_u8..22 {
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"main", &[n], 1)
            .expect("main advances independently");
    }
    let main = translate::commit(branch_tip(&mut world, b"main"));
    let closer = make_closer(&mut world, &goal, 400);
    let job = close_job(&mut world);
    let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    finish_close(&mut world, close, &branch, &job);
    wait_ended(&mut world, close);
    goal_tips.push(translate::commit(branch_tip(&mut world, &branch)));
    let landing = wait_landing(&mut world, closer);
    wait_ended(&mut world, closer);
    let change::State::Landed { merge, .. } = row(&world, landing).change.state else { panic!("landing merged") };
    assert_eq!(translate::commit(branch_tip(&mut world, b"main")), merge);
    assert!(!result_words(&world, closer).is_empty(), "closer reported");
    assert!(
        !world.forge_reads.as_ref().expect("recorded reads").iter().any(|(_, read)| matches!(read,
            Read::Compare { before, after } if
                (*before == main && goal_tips.contains(after)) ||
                (*after == main && goal_tips.contains(before)) ||
                (goal_tips.contains(before) && goal_tips.contains(after))
        )),
        "containment reads no full comparison"
    );
    world.close();
}

#[test]
fn a_closer_without_a_close_job_skips_the_close_step() {
    let (mut world, goal, branch) = goal_world(false, true);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    let closer = make_closer(&mut world, &goal, 94);
    wait_ended(&mut world, closer);
    let row = closer_row(&world, closer).expect("closer row");
    assert!(row.close_changes.is_empty());
    assert_eq!(row.landing_changes.len(), 1);
    assert!(world.jobs.is_empty());
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

#[test]
fn a_closer_of_a_goal_with_no_goal_branch_only_reports() {
    let (mut world, goal, branch) = goal_world(true, false);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    let closer = make_closer(&mut world, &goal, 94);
    wait_ended(&mut world, closer);
    let row = closer_row(&world, closer).expect("closer row");
    assert!(row.close_changes.is_empty());
    assert!(row.landing_changes.is_empty());
    let output = core::str::from_utf8(result_words(&world, closer)).expect("report UTF-8");
    assert!(output.contains("into main\n"), "{output}");
    assert!(output.contains("The landing: none, no goal branch."), "{output}");
    assert!(output.ends_with(": unknown\n"), "{output}");
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

pub(super) fn wait_held(world: &mut World, task: u64, goal: u64) {
    for _ in 0..STEPS {
        if matches!(live_task(world, task).phase, tasks::Phase::Held { .. })
            && matches!(live_task(world, task).escalation, tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), .. } if holder == goal)
        {
            return;
        }
        world.tick();
    }
    panic!(
        "T{task} did not escalate to goal T{goal}: {:?}; {:?}",
        live_task(world, task).phase,
        live_task(world, task).escalation
    )
}

#[test]
fn a_closers_failed_close_change_holds_it_and_its_release_retries() {
    let (mut world, goal, branch) = goal_world(true, true);
    let closer = make_closer(&mut world, &goal, 93);
    let mut close = None;
    for _ in 0..=world.env.limits.tasks.retries.permanent.retries {
        let job = close_job(&mut world);
        let tasks::Party::Task(task) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
        close = Some(task);
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
    }
    let close = close.expect("close attempted");
    wait_held(&mut world, close, goal.task);
    assert!(matches!(row(&world, close).change.state, change::State::Held { why: change::Hold::Failed, .. }));
    assert!(matches!(
        world.scripted_call(
            &goal,
            94,
            engine::Tool::Cancel { target: close, reason: b"retry the close step".as_slice().into() }
        ),
        engine::CallAnswer::Controlled
    ));
    wait_ended(&mut world, close);
    wait_held(&mut world, closer, goal.task);
    assert_eq!(
        closer_row(&world, closer).expect("held closer").closer.hold,
        Some(closer::Hold::ClosingStep { step: closer::Step::Close, change: close })
    );
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, closer).phase else {
        panic!("hold account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("close account") };
    assert!(words.starts_with(format!("the close step's change T{close} ended cancelled").as_bytes()));
    let revision = live_task(&world, closer).revisions.hold;
    let answer = world.scripted_call(&goal, 95, engine::Tool::Release { target: closer, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    let job = close_job(&mut world);
    let tasks::Party::Task(second) = live_task(&world, job.run.raw()).requester else { panic!("second close") };
    assert_ne!(second, close);
    assert_eq!(closer_row(&world, closer).expect("retry row").close_changes.as_ref(), &[close, second]);
    finish_close(&mut world, second, &branch, &job);
    wait_ended(&mut world, closer);
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

#[test]
fn a_closers_landing_held_escalates_to_the_goal() {
    let (mut world, goal, branch) = goal_world_in(false, true, None, true);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    let mut head = branch_tip(&mut world, &branch);
    status(&mut world, head, raw::Check::Pending);
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    for number in 0..world.env.limits.forge.change_policy.repairs {
        let before = world.assigned.len();
        status(&mut world, head, raw::Check::Failed);
        let repair = reviews::next_assignment(&mut world, before);
        assert_eq!(live_task(&world, repair.task).requester, tasks::Party::Task(landing));
        head =
            fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"repair", &number.to_be_bytes(), 1)
                .expect("repair pushed g");
        status(&mut world, head, raw::Check::Pending);
        finish_run(
            &mut world,
            &repair,
            tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Repair".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"repair pushed".as_slice().into(),
            },
        );
        for _ in 0..STEPS {
            if row(&world, landing).change.state == (change::State::Checking { head: translate::commit(head) }) {
                break;
            }
            world.tick();
        }
    }
    status(&mut world, head, raw::Check::Failed);
    wait_held(&mut world, landing, goal.task);
    assert!(matches!(row(&world, landing).change.state, change::State::Held { why: change::Hold::Repairs, .. }));
    assert!(matches!(live_task(&world, closer).phase, tasks::Phase::Active(_)));
    let name = forge_top::Name {
        forge: forge_world::REPO.forge,
        repository: forge_world::REPO.repository,
        what: forge_top::What::Pull(row(&world, landing).pull.expect("landing pull")),
    };
    let pull = Key::Forge(forge_top::Key::PullState(name));
    let mut watched = false;
    for _ in 0..STEPS {
        if let Some(Record::Forge { row }) = world.store.rows.get(&pull)
            && let forge_top::Stored::PullState(pull) = row.as_ref()
        {
            watched = pull.head == translate::commit(head) && pull.ci == skein_forge::api::Ci::Failed;
        }
        if watched {
            break;
        }
        world.tick();
    }
    assert!(watched, "the pull watch catches the gathered head before the restart oracle");
    world.close();
}

pub(super) fn updating_landing(close: bool) -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    updating_landing_with_stall(close, None)
}

pub(super) fn updating_landing_with_stall(
    close: bool,
    stall: Option<Duration>,
) -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    let (mut world, goal, branch) = goal_world_with_updates(close, true, None, true, true, STEPS, None, true, stall, 1);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    let main = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"base", b"new main", 1)
        .expect("main advances");
    status(&mut world, main, raw::Check::Passed);
    let closer = make_closer(&mut world, &goal, 94);
    if close {
        let job = close_job(&mut world);
        let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
        finish_close(&mut world, close, &branch, &job);
        wait_ended(&mut world, close);
        assert!(matches!(row(&world, close).change.state, change::State::Landed { .. }));
    }
    let landing = wait_landing(&mut world, closer);
    for _ in 0..STEPS {
        world.tick();
        if matches!(row(&world, landing).change.state, change::State::Updating { .. }) {
            break;
        }
    }
    assert!(
        matches!(row(&world, landing).change.state, change::State::Updating { .. }),
        "{:?}",
        row(&world, landing).change.state
    );
    assert_eq!(world.writes.iter().filter(|write| matches!(write, skein_forge::api::Write::Update { .. })).count(), 0);
    (world, goal, branch, closer, landing)
}

fn pending_landing() -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    // Hold/release stories judge each ended landing, rather than automatic attempts.
    pending_landing_with_attempts(1)
}

fn pending_landing_with_attempts(attempts: u8) -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    let (mut world, goal, branch) =
        goal_world_with_updates(false, true, None, true, true, STEPS, None, false, None, attempts);
    add_change(&mut world, &goal, &branch, 93, b"First", b"done");
    let head = branch_tip(&mut world, &branch);
    status(&mut world, head, raw::Check::Pending);
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    (world, goal, branch, closer, landing)
}

fn cancel_landing(world: &mut World, goal: &engine::Assignment, landing: u64) {
    let answer = world.scripted_call(
        goal,
        95,
        engine::Tool::Cancel { target: landing, reason: b"end this landing".as_slice().into() },
    );
    assert!(matches!(answer, engine::CallAnswer::Controlled), "{answer:?}");
    wait_ended(world, landing);
}

pub(super) fn assert_hold_words(world: &World, task: u64, expected: &str) {
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(world, task).phase else { panic!("held account") };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold words") };
    assert_eq!(core::str::from_utf8(words).expect("hold UTF-8"), expected);
}

fn next_landing(world: &mut World, closer: u64, count: usize) -> u64 {
    for _ in 0..STEPS {
        if closer_row(world, closer).expect("closer").landing_changes.len() == count {
            break;
        }
        world.tick();
    }
    assert_eq!(
        closer_row(world, closer).expect("closer").landing_changes.len(),
        count,
        "an ended attempt below the bound must retry without holding"
    );
    closer_row(world, closer).expect("closer").closer.landing.expect("next landing")
}

#[test]
fn cancelled_landings_retry_to_the_project_bound_and_release_adds_one_attempt() {
    let (mut world, goal, _, closer, first) = pending_landing_with_attempts(3);
    let spend = live_task(&world, first).authority.budget.spend;
    let runs = world.assigned.iter().filter(|run| run.task == goal.task).count();
    let mut landing = first;
    for count in 1..=3 {
        cancel_landing(&mut world, &goal, landing);
        assert_eq!(row(&world, landing).change.ended_held, None);
        if count < 3 {
            landing = next_landing(&mut world, closer, count + 1);
            assert_eq!(
                live_task(&world, landing).authority.budget.spend,
                spend,
                "an unspent cancelled attempt returns its half"
            );
            assert_eq!(live_task(&world, closer).revisions.hold, 0);
            assert_eq!(world.assigned.iter().filter(|run| run.task == goal.task).count(), runs);
        }
    }
    wait_held(&mut world, closer, goal.task);
    assert_hold_words(
        &world,
        closer,
        &format!("the landing T{landing} ended cancelled; attempts ended without landing: 3, the project's bound 3"),
    );
    let revision = live_task(&world, closer).revisions.hold;
    let answer = world.scripted_call(&goal, 96, engine::Tool::Release { target: closer, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    landing = next_landing(&mut world, closer, 4);
    cancel_landing(&mut world, &goal, landing);
    wait_held(&mut world, closer, goal.task);
    assert_hold_words(
        &world,
        closer,
        &format!("the landing T{landing} ended cancelled; attempts ended without landing: 4, the project's bound 3"),
    );
    world.close();
}

#[test]
fn cancelled_close_changes_count_their_attempts_alone() {
    let (mut world, goal, _) = goal_world_with_updates(true, true, None, true, true, STEPS, None, false, None, 3);
    let closer = make_closer(&mut world, &goal, 93);
    let mut close = 0;
    for _ in 0..3 {
        let job = close_job(&mut world);
        let tasks::Party::Task(task) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
        close = task;
        let answer = world.scripted_call(
            &goal,
            95,
            engine::Tool::Cancel { target: close, reason: b"Retry close".as_slice().into() },
        );
        assert!(matches!(answer, engine::CallAnswer::Controlled), "{answer:?}");
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
        wait_ended(&mut world, close);
    }
    wait_held(&mut world, closer, goal.task);
    assert_hold_words(
        &world,
        closer,
        &format!(
            "the close step's change T{close} ended cancelled; attempts ended without landing: 3, the project's bound 3"
        ),
    );
    assert_eq!(closer_row(&world, closer).expect("closer").close_changes.len(), 3);
    assert!(closer_row(&world, closer).expect("closer").landing_changes.is_empty());
    world.close();
}

fn base_holds_goal(world: &mut World, branch: &[u8]) {
    let head = branch_tip(world, branch);
    let old = branch_tip(world, b"main");
    assert_eq!(
        world.external(
            1,
            raw::Op::Git(raw::Git::Push { branch: b"main".as_slice().into(), commit: head, expected: Some(old) })
        ),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
}

fn release_ended_landing(base_holds: bool, restart: bool) {
    let (mut world, goal, branch, closer, landing) = pending_landing();
    cancel_landing(&mut world, &goal, landing);
    wait_held(&mut world, closer, goal.task);
    assert_eq!(
        closer_row(&world, closer).expect("held closer").closer.hold,
        Some(closer::Hold::ClosingStep { step: closer::Step::Land, change: landing })
    );
    if restart {
        let mut cut = false;
        for _ in 0..STEPS {
            if temper_engine_domain_world::tier_runs::restart_cut(&world) {
                cut = true;
                break;
            }
            world.tick();
        }
        assert!(cut, "restart between hold and release");
        world.restart_same();
        world.until(Until::Ready);
        wait_held(&mut world, closer, goal.task);
    }
    if base_holds {
        base_holds_goal(&mut world, &branch);
    }
    world.fake_env.limits.latency_min = Duration::from_secs(1);
    world.fake_env.limits.latency_max = Duration::from_secs(1);
    let revision = live_task(&world, closer).revisions.hold;
    let answer = world.scripted_call(&goal, 96, engine::Tool::Release { target: closer, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("released closer").closer.retry == Some(closer::Step::Land) {
            break;
        }
        world.tick();
    }
    let progress = &closer_row(&world, closer).expect("released closer").closer;
    assert_eq!(progress.retry, Some(closer::Step::Land), "release survives the containment read");
    assert_eq!(progress.hold, None);
    world.fake_env.limits.latency_min = Duration::ZERO;
    world.fake_env.limits.latency_max = Duration::ZERO;
    if base_holds {
        wait_ended(&mut world, closer);
        let output = core::str::from_utf8(result_words(&world, closer)).expect("report UTF-8");
        assert!(output.contains("The landing: none, its base already held the goal's branch."), "{output}");
        assert_eq!(closer_row(&world, closer).expect("reported closer").landing_changes.as_ref(), &[landing]);
    } else {
        for _ in 0..STEPS {
            if closer_row(&world, closer).expect("retry closer").landing_changes.len() == 2 {
                break;
            }
            world.tick();
        }
        let row = closer_row(&world, closer).expect("retry closer");
        assert_eq!(row.landing_changes.len(), 2, "release makes a new landing, not another hold: {row:?}");
        assert_ne!(row.closer.landing, Some(landing));
        assert_eq!(row.closer.hold, None);
    }
    world.close();
}

#[test]
fn a_cancelled_landing_release_makes_a_new_landing() {
    release_ended_landing(false, false);
}

#[test]
fn a_cancelled_landing_release_makes_its_new_landing_at_the_moved_head() {
    let (mut world, goal, branch, closer, landing) = pending_landing();
    let old = branch_tip(&mut world, &branch);
    for _ in 0..STEPS {
        if row(&world, landing).change.last_head == Some(translate::commit(old)) {
            break;
        }
        world.tick();
    }
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(old)));
    cancel_landing(&mut world, &goal, landing);
    wait_held(&mut world, closer, goal.task);
    let moved = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"later", b"new work", 1)
        .expect("goal branch advances while its closer holds");
    assert_ne!(moved, old);
    status(&mut world, moved, raw::Check::Pending);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(branch.clone()),
            key: None,
        },
    })));
    let revision = live_task(&world, closer).revisions.hold;
    let answer = world.scripted_call(&goal, 96, engine::Tool::Release { target: closer, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
    for _ in 0..STEPS {
        world.tick();
        if let Some(new) = closer_row(&world, closer).and_then(|row| row.closer.landing)
            && new != landing
            && world.store.rows.contains_key(&Key::Forge(forge_top::Key::Change(new)))
            && row(&world, new).change.last_head.is_some()
        {
            break;
        }
    }
    let new = closer_row(&world, closer).expect("released closer").closer.landing.expect("new landing");
    assert_ne!(new, landing);
    assert_eq!(
        row(&world, new).change.last_head,
        Some(translate::commit(moved)),
        "release reads the moved goal tip, not the ended landing's head"
    );
    world.close();
}

#[test]
fn a_cancelled_landing_release_reports_if_main_now_holds_the_goal() {
    release_ended_landing(true, false);
}

#[test]
fn an_ended_landing_release_survives_restart() {
    for base_holds in [false, true] {
        release_ended_landing(base_holds, true);
    }
}

#[test]
fn a_failed_landing_reports_without_holding_if_main_holds_the_goal() {
    let (mut world, goal, branch, closer, landing) = pending_landing();
    base_holds_goal(&mut world, &branch);
    world.fake_env.limits.latency_min = Duration::from_secs(1);
    world.fake_env.limits.latency_max = Duration::from_secs(1);
    let answer = world.scripted_call(
        &goal,
        95,
        engine::Tool::Cancel { target: landing, reason: b"stop the failed landing".as_slice().into() },
    );
    assert!(matches!(answer, engine::CallAnswer::Controlled), "{answer:?}");
    wait_ended(&mut world, landing);
    assert!(!closer_row(&world, closer).expect("closer still deciding").done);
    // Supply the failed terminal account while the closer reads containment.
    world.send(engine::Event::Forge(Box::new(forge_top::Event::ChangeEnding {
        task: landing,
        ending: forge_top::Ending::Failed { line: b"landing failed".as_slice().into() },
    })));
    for _ in 0..STEPS {
        if matches!(row(&world, landing).ending, Some(forge_top::Ending::Failed { .. })) {
            break;
        }
        world.tick();
    }
    assert!(matches!(row(&world, landing).ending, Some(forge_top::Ending::Failed { .. })));
    world.fake_env.limits.latency_min = Duration::ZERO;
    world.fake_env.limits.latency_max = Duration::ZERO;
    wait_ended(&mut world, closer);
    let output = core::str::from_utf8(result_words(&world, closer)).expect("report UTF-8");
    assert!(output.contains("The landing: none, its base already held the goal's branch."), "{output}");
    assert_eq!(ended(&world, closer).expect("closer ended").revisions.hold, 0, "the closer never held");
    world.close();
}

#[test]
fn a_closer_survives_restart() {
    let (mut world, goal, branch) = goal_world(true, true);
    let closer = make_closer(&mut world, &goal, 93);
    let job = close_job(&mut world);
    let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    finish_close(&mut world, close, &branch, &job);
    let mut cut = false;
    for _ in 0..STEPS {
        if matches!(row(&world, close).change.state, change::State::Landed { .. })
            && closer_row(&world, closer).expect("closer").closer.landing.is_none()
            && temper_engine_domain_world::tier_runs::restart_cut(&world)
        {
            cut = true;
            break;
        }
        world.tick();
    }
    assert!(cut, "cut between close landing and goal landing");
    let saved = closer_row(&world, closer).expect("closer durable").clone();
    world.restart_same();
    wait_ended(&mut world, closer);
    let restored = closer_row(&world, closer).expect("closer restored");
    assert_eq!(restored.close_changes, saved.close_changes);
    assert_eq!(restored.landing_changes.len(), 1);
    assert!(result_words(&world, closer).starts_with(b"Goal T"));
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

#[test]
fn a_closer_waits_for_work_and_closes_again() {
    let (mut world, goal, branch) = goal_world_in(true, true, None, true);
    let closer = make_closer(&mut world, &goal, 93);
    let job = close_job(&mut world);
    let tasks::Party::Task(first) = live_task(&world, job.run.raw()).requester else { panic!("first close") };
    finish_close(&mut world, first, &branch, &job);
    let landing = wait_landing(&mut world, closer);
    let before = world.assigned.len();
    let answer = mine(&mut world, &goal, 94, br#"{"words":"New work"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else {
        panic!("new work: {answer:?}; errors {:?}", world.call_errors)
    };
    let work = made[0].task;
    let producer = reviews::next_assignment(&mut world, before);
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"Wait for new work");
    amendment.dependencies = Some(Box::new([work]));
    let revision = live_task(&world, closer).revisions.spec;
    let answer = world.scripted_call(
        &goal,
        95,
        engine::Tool::Amend { target: closer, revision, amendment, keep_parameters: false },
    );
    assert!(matches!(answer, engine::CallAnswer::Amended { .. }), "{answer:?}");
    assert_eq!(live_task(&world, closer).waiting_on.as_ref(), &[work]);
    assert_eq!(closer_row(&world, closer).expect("kept closer").close_changes.as_ref(), &[first]);
    produce(&mut world, work, &branch, &producer, b"New work", b"done");
    let tip = branch_tip(&mut world, &branch);
    let job = close_job(&mut world);
    let tasks::Party::Task(second) = live_task(&world, job.run.raw()).requester else { panic!("second close") };
    assert_ne!(second, first);
    // The branch observation and the next close job are independent.
    for _ in 0..STEPS {
        if row(&world, landing).change.last_head == Some(translate::commit(tip)) {
            break;
        }
        world.tick();
    }
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(tip)));
    assert_eq!(closer_row(&world, closer).expect("reclosed").close_changes.as_ref(), &[first, second]);
    finish_close(&mut world, second, &branch, &job);
    wait_ended(&mut world, second);
    let head = branch_tip(&mut world, &branch);
    status(&mut world, head, raw::Check::Passed);
    wait_ended(&mut world, closer);
    let output = core::str::from_utf8(result_words(&world, closer)).expect("report UTF-8");
    assert!(output.contains(&format!("- T{first} Settle goal")), "{output}");
    assert!(output.contains(&format!("- T{second} Settle goal")), "{output}");
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

#[test]
fn a_red_after_the_closers_landing_is_news_for_the_goal() {
    use temper_engine_domain_world::tier_runs;
    let (mut world, goal, branch) = goal_world_in(false, true, Some(tier_runs::Tier::default()), false);
    add_change(&mut world, &goal, &branch, 93, b"failures", b"done");
    let tip = branch_tip(&mut world, &branch);
    status(&mut world, tip, raw::Check::Passed);
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    for _ in 0..4000 {
        if !tier_runs::red_news(&world, goal.task).is_empty()
            && tier_runs::tier(&world).record.run.is_none()
            && matches!(row(&world, landing).change.state, change::State::Red { .. })
        {
            break;
        }
        tier_runs::tick(&mut world);
    }
    let news = tier_runs::red_news(&world, goal.task);
    assert!(
        !news.is_empty(),
        "red reaches goal; landing {:?}; tiers {:?}",
        row(&world, landing).change.state,
        world
            .store
            .rows
            .values()
            .filter_map(|r| match r {
                Record::Forge { row } => match row.as_ref() {
                    forge_top::Stored::Tier(tier) => Some(&tier.record),
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .collect::<Vec<_>>()
    );
    assert!(
        matches!(row(&world, landing).change.state, change::State::Red { .. }),
        "landing {:?}; news {:?}",
        row(&world, landing).change.state,
        news.iter().map(|word| String::from_utf8_lossy(&word.words)).collect::<Vec<_>>()
    );
    assert!(!live_task(&world, closer).inbox.iter().any(|word| word.words.starts_with(b"Red after landing:")));
    let landings = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Subscription(sub) if sub.task == goal.task => match &sub.topic {
                    forge_top::Topic::Landings { branch, .. } => Some(branch.as_ref()),
                    _ => None,
                },
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect::<Vec<_>>();
    assert!(!landings.contains(&b"main".as_slice()), "no main landings while the closer verifies: {landings:?}");
    assert!(landings.contains(&branch.as_ref()), "the goal keeps its branch's topic");
    world.close();
}

fn selection_change(world: &mut World, goal: &engine::Assignment, branch: &[u8], call: u64, title: &[u8]) {
    let assigned = world.assigned.len();
    let Some(engine::CallAnswer::Delegated(made)) =
        mine(world, goal, call, br#"{"words":"Implement or repair the goal"}"#)
    else {
        panic!("goal change admitted")
    };
    let producer = reviews::next_assignment(world, assigned);
    produce_with_status(world, made[0].task, branch, &producer, title, b"done", None);
}
fn selection_world() -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    let (mut world, goal, branch) = goal_world_with_ci(false, true, None, false, false, 0, None);
    fake::set_opening(&mut world.fake, b"org/repo", b"build", b"selection: full\n...");
    selection_change(&mut world, &goal, &branch, 93, b"source");
    let closer = make_closer(&mut world, &goal, 94);
    let landing = wait_landing(&mut world, closer);
    (world, goal, branch, closer, landing)
}

fn hold_selection_answer(world: &mut World, failed: bool) -> engine::Event {
    for _ in 0..STEPS {
        if let Some(index) = world.events.iter().position(|event| matches!(event,
            engine::Event::Forge(event) if matches!(event.as_ref(), forge_top::Event::Client(skein_forge_client::Event::Answered { result, .. }) if match result { Ok(skein_forge::api::Answer::Job { .. }) => !failed, Err(skein_forge::api::Error::Refused) => failed, _ => false }))) {
            return world.events.remove(index).expect("independent forge job answer");
        }
        world.tick();
    }
    panic!("selection job answer not read")
}

#[test]
fn a_goals_report_quotes_each_main_pulls_ci_selection() {
    let (mut world, goal, _, closer, landing) = selection_world();
    let answer = hold_selection_answer(&mut world, false);
    assert!(ended(&world, landing).is_some(), "repair follows the landing");
    assert!(ended(&world, closer).is_none(), "report waits for selection reads");
    fake::set_opening(&mut world.fake, b"org/repo", b"build", b"cargo ...\n");
    let assigned = world.assigned.len();
    let mut delegate = ordinary_delegate(&world, &goal, b"Repair main");
    delegate.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 4096 };
    delegate.authority = live_task(&world, goal.task).authority.clone();
    delegate.authority.delegation.depth -= 1;
    delegate.authority.delegation.tasks = 6;
    delegate.authority.budget.spend = 60;
    delegate.spec.parameters = Box::new([
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
    ]);
    let delegated = world.scripted_call(&goal, 95, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let engine::CallAnswer::Delegated(made) = delegated else { panic!("main repair admitted: {delegated:?}") };
    let repair = made[0].task;
    let producer = reviews::next_assignment(&mut world, assigned);
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"Wait for the main repair");
    amendment.dependencies = Some(Box::new([repair]));
    let revision = live_task(&world, closer).revisions.spec;
    let amended = world.scripted_call(
        &goal,
        97,
        engine::Tool::Amend { target: closer, revision, amendment, keep_parameters: false },
    );
    assert!(matches!(amended, engine::CallAnswer::Amended { .. }), "{amended:?}");
    world.events.push_front(answer);
    produce_with_status(&mut world, repair, b"main", &producer, b"Repair main", b"repaired", None);
    assert_eq!(row(&world, repair).base.as_ref(), b"main");
    assert!(row(&world, repair).change.last_head.is_some());
    wait_ended(&mut world, closer);
    let landing_pull = row(&world, landing).pull.expect("landing pull");
    let repair_pull = row(&world, repair).pull.expect("repair pull");
    let output = core::str::from_utf8(result_words(&world, closer)).expect("report UTF-8");
    assert!(
        output.ends_with(&format!(
            "CI selection:\n- pull request #{landing_pull}: selection: full\n- pull request #{repair_pull}: unknown\n"
        )),
        "{output}"
    );
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}

#[test]
fn a_failed_log_read_reads_unknown() {
    let (mut world, goal, _, closer, landing) = selection_world();
    world.fail_job_reads = true;
    let answer = hold_selection_answer(&mut world, true);
    assert!(ended(&world, closer).is_none(), "report waits for the failed job read");
    world.events.push_front(answer);
    wait_ended(&mut world, closer);
    let pull = row(&world, landing).pull.expect("landing pull");
    let output = core::str::from_utf8(result_words(&world, closer)).expect("report UTF-8");
    assert!(output.ends_with(&format!("CI selection:\n- pull request #{pull}: unknown\n")), "{output}");
    let words = result_words(&world, closer).into();
    finish_run(&mut world, &goal, tasks::TaskResult::Report { words });
    wait_ended(&mut world, goal.task);
    for _ in 0..300 {
        world.tick();
    }
    world.close();
}
