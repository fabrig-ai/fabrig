//! Root restarts retain worker preparation, proof and the turn, and reload batch size
//! (temper/docs/design/domain/landing.md, 4.1, 5, 8, 9, 15–16;
//! temper/docs/design/domain/landing-preparing.md, 4–7).
use super::super::{goal_branch, goal_landing};
use super::*;
use temper_protocol_worker_wire::v1::Record as WorkerRecord;

const SLOTS: u32 = 9;

fn worker_hello(world: &World) -> engine::Event {
    let mut event = hello_prepare(SLOTS);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    event
}

fn turn_world() -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            // The provider and store admit four; the initial project's policy below admits one.
            batch: Some(4),
            landing_members: Some(4),
            slots: Some(SLOTS),
            tasks: Some(16),
            authority_batch: Some(4),
            spec_bytes: Some(2048),
            result_bytes: Some(1024),
            inbox_messages: Some(8),
            ..StoryLimits::default()
        },
    )
    .with_landing_batch(1)
    .with_forge_holds(16);
    let mut client = world.env.limits.forge.client;
    client.pending = 8;
    world = world.with_forge_client(client);
    world.adopt();
    world.send(worker_hello(&world));
    world.manual_prepare_jobs();
    world
}

pub(super) fn tick(world: &mut World) {
    // Decisions take no simulated time; idle waits advance to the next peer or root timer.
    let interval = world.tick_interval;
    world.tick_interval = Duration::ZERO;
    if world.events.is_empty() && world.store.pending.is_empty() {
        let next = if world.pending.is_empty() {
            world.root.quiescent().then(|| world.root.next_deadline(&world.env)).flatten()
        } else {
            world.fake.next_deadline()
        };
        if let Some(next) = next {
            world.env.now = world.env.now.max(next);
        }
    }
    world.prepare_tick();
    world.tick_interval = interval;
}

fn restart(world: &mut World) {
    world.drain_cut();
    world.restart_same();
    world.send(worker_hello(world));
}

pub(super) fn pushed_change(world: &mut World, owner: &engine::Assignment, call: u64) -> (u64, engine::Assignment) {
    let (change, producer) = make_change(world, owner, call, b"org/repo");
    let branch = producer.workspace.repositories[0].push.as_ref().expect("producer branch").clone();
    let base = world.fake.branch(b"org/repo", b"main").expect("main");
    world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base }));
    // Distinct paths stack without conflicts and keep successive landings' keys different.
    fake::advance(
        &mut world.fake,
        &world.fake_env,
        b"org/repo",
        &branch,
        format!("fix-{call}").as_bytes(),
        b"fixed",
        1,
    )
    .expect("distinct change");
    (change, producer)
}

fn assert_unheld(world: &World) {
    for row in landing_rows(world) {
        assert!(!matches!(row.landing.phase, forge_top::LandingPhase::Held { .. }), "{row:?}");
        for task in std::iter::once(row.landing.task).chain(row.landing.members.iter().map(|member| member.change)) {
            let task = task_row(world, task).expect("task");
            assert!(!matches!(task.phase, tasks::Phase::Held { .. }), "task {}: {:?}", task.number, task.phase);
        }
    }
}

pub(super) fn await_prepare(world: &mut World) -> u64 {
    for _ in 0..PAIR_TICKS {
        tick(world);
        assert_unheld(world);
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert_eq!(world.jobs.len(), 1, "one worker prepare");
    let job = &world.jobs[0];
    assert_eq!(job.kind, jig_domain_link::JobKind(1));
    let landing = job.workspace.expect("prepare workspace").workstream;
    assert!(world.job_workspaces.contains_key(&job.run.raw()), "worker has the workspace");
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Preparing);
    landing
}

pub(super) fn answer_prepare(world: &mut World) {
    // The world's worker stacks from the assigned workspace and answers through answer_job.
    world.automatic_prepare_jobs();
    tick(world);
    world.manual_prepare_jobs();
    assert!(world.jobs.is_empty(), "worker answered the assigned prepare");
}

pub(super) fn await_landed(world: &mut World, landing: u64) {
    for _ in 0..PAIR_TICKS {
        tick(world);
        assert_unheld(world);
        assert!(world.jobs.is_empty(), "no second prepare job");
        let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
        if ended(world, landing) && row.landing.members.iter().all(|member| ended(world, member.change)) {
            return;
        }
    }
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    panic!(
        "landing {landing} did not finish at {:?}: {:?}; members {:?}",
        world.env.wall,
        row.landing.phase,
        row.landing
            .members
            .iter()
            .map(|member| (member.change, member.landed.is_some(), member.told, ended(world, member.change)))
            .collect::<Vec<_>>()
    );
}

fn assert_landed(world: &World, landing: u64, changes: &[u64]) {
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(row.landing.preparation, 1);
    assert_eq!(row.landing.members.iter().map(|member| member.change).collect::<Vec<_>>(), changes);
    for member in &row.landing.members {
        assert!(member.landed.is_some() && member.told);
        assert_eq!(member.left, None);
        assert!(ended(world, member.change));
        let change = goal_landing::row(world, member.change);
        assert_eq!(change.change.repairs, 0);
        assert!(matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }));
    }
    assert_eq!(
        world
            .job_workspaces
            .values()
            .filter(|workspace| workspace.repositories.iter().any(|item| item.prefixes.iter().any(|branch| row
                .landing
                .members
                .iter()
                .any(|member| member.branch == *branch))))
            .count(),
        1,
        "one prepare assignment per landing"
    );
    let top = row.landing.members.last().expect("top");
    assert_eq!(world.fake.branch(b"org/repo", b"main"), fake::commit_count(&top.landed.expect("top landed")));
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Dispatch { branch, .. } if *branch == top.branch))
            .count(),
        1,
        "one dispatch at the top"
    );
    assert_eq!(
        world.writes.iter().filter(|write| matches!(write, api::Write::Merge { number, how: api::MergeHow::FastForwardOnly, .. } if Some(*number) == row.landing.round.pull)).count(),
        1,
        "one fast-forward"
    );
}

// Expiring the worker's git identity fences its claim, not the atomic admission.
fn marked_prepare() -> (World, u64, u64) {
    let mut world = turn_world();
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let (change, producer) = pushed_change(&mut world, &chat, 95);
    world.send(engine::Event::Credential { number: 1, generation: 2, valid: Duration::ZERO });
    finish_producer(&mut world, &producer, forge_world::REPO);
    for _ in 0..PAIR_TICKS {
        tick(&mut world);
        if let Some(row) = landing_rows(&world).into_iter().find(|row| {
            row.landing.phase == forge_top::LandingPhase::Preparing
                && row.landing.members.iter().all(|member| member.asked == Some(1))
        }) {
            let landing = row.landing.task;
            assert_eq!(row.landing.preparation, 1);
            assert!(row.job.is_none(), "job history starts at the first claim");
            let children = prepare_children(&world, landing);
            assert_eq!(children.len(), 1, "marks and child stored together");
            assert_eq!(task_row(&world, landing).expect("landing task").delegates.as_ref(), children);
            let child = task_row(&world, children[0]).expect("prepare child");
            assert!(
                child.open_attempt.as_ref().is_none_or(|attempt| attempt.claimed.is_none()),
                "no claim: {:?}",
                child.open_attempt
            );
            assert!(world.jobs.is_empty(), "no prepare assigned");
            assert!(world.store.pending.is_empty(), "marks committed");
            assert!(world.pending.is_empty(), "no outstanding forge submissions at the cut");
            return (world, change, landing);
        }
    }
    panic!("prepare marks not saved");
}

fn prepare_children(world: &World, landing: u64) -> Vec<u64> {
    // Include ended children: a replacement must not hide the earlier admission.
    let mut children: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.requester == tasks::Party::Task(landing)
                    && matches!(row.executor, tasks::Executor::Job { .. }) =>
            {
                Some(row.number)
            }
            _ => None,
        })
        .collect();
    children.sort_unstable();
    children
}

fn assert_prepare_history(world: &World, change: u64, landing: u64, expected: &[(u64, u64)]) {
    use forge_top::history as h;
    let jobs: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::HistoryEvent(event)
                    if event.change == change && matches!(event.what, h::What::Job { job: h::Job::Prepare, .. }) =>
                {
                    Some(event)
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect();
    assert_eq!(jobs.len(), expected.len());
    for (event, (child, attempt)) in jobs.iter().zip(expected) {
        assert_eq!(event.what, h::What::Job { job: h::Job::Prepare, task: *child });
        assert_eq!(u64::from(event.attempt), *attempt);
        assert_eq!(event.operation, *child);
        assert_eq!(event.landing.map(|within| (within.task, within.round)), Some((landing, 1)));
        let attempts: Vec<_> =
            expected.iter().filter_map(|(task, attempt)| (*task == *child).then_some(*attempt)).collect();
        assert_eq!(claimed_attempts(world, *child), attempts);
        assert!(matches!(task_row(world, *child).expect("settled prepare child").phase, tasks::Phase::Ended(_)));
    }
    assert_unheld(world);
    assert!(!world.store.rows.values().any(|record| matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
            if matches!(row.executor, tasks::Executor::Person(_)) || !row.questions.is_empty()
                || row.inbox.iter().any(|word| matches!(word.kind, tasks::MessageKind::Question)))));
}

#[test]
fn a_restart_after_prepare_admission_claims_that_child_once() {
    let (mut world, change, landing) = marked_prepare();
    let children = prepare_children(&world, landing);
    restart(&mut world);
    assert_eq!(await_prepare(&mut world), landing);
    let job = (world.jobs[0].run.raw(), world.jobs[0].attempt.raw());
    assert_eq!(job.0, children[0]);
    assert_prepare_record(&world, landing, job);
    assert_eq!(claimed_attempts(&world, job.0), [job.1]);
    answer_prepare(&mut world);
    await_landed(&mut world, landing);
    assert_landed(&world, landing, &[change]);
    assert_eq!(prepare_children(&world, landing), children);
    assert_prepare_history(&world, change, landing, &[job]);
    world.close();
}

fn assert_prepare_record(world: &World, landing: u64, job: (u64, u64)) {
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Preparing);
    assert_eq!(row.landing.members[0].asked, Some(1));
    let kept = row.job.as_ref().expect("claimed prepare");
    assert_eq!((kept.step.task, u64::from(kept.step.attempt)), job);
    assert_eq!(kept.round, 1);
    assert_eq!(kept.members.as_ref(), [0]);
}

fn claimed_attempts(world: &World, child: u64) -> Vec<u64> {
    let mut attempts: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row)))
                if row.task == child && row.claimed.is_some() =>
            {
                Some(row.attempt)
            }
            _ => None,
        })
        .collect();
    if let Some(attempt) = &task_row(world, child).expect("child").open_attempt
        && attempt.claimed.is_some()
    {
        attempts.push(attempt.attempt);
    }
    attempts.sort_unstable();
    attempts.dedup();
    attempts
}

fn start_prepare(world: &mut World, landing: u64) -> (u64, u64) {
    world.send(engine::Event::Credential { number: 1, generation: 3, valid: Duration::from_secs(3600) });
    assert_eq!(await_prepare(world), landing);
    let job = (world.jobs[0].run.raw(), world.jobs[0].attempt.raw());
    assert_prepare_record(world, landing, job);
    assert_eq!(prepare_children(world, landing), [job.0]);
    assert_eq!(claimed_attempts(world, job.0), [job.1]);
    job
}

fn member_branch(world: &World, landing: u64) -> Box<[u8]> {
    landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing").landing.members[0]
        .branch
        .clone()
}

#[test]
fn a_lost_prepare_worker_retries_the_same_child_after_its_writers_settle() {
    lost_prepare(false);
}

#[test]
fn a_lost_prepare_worker_retries_the_same_child_across_restart() {
    lost_prepare(true);
}

#[expect(
    clippy::too_many_lines,
    reason = "one recovery couples loss, a restart cut, writer settlement and the retained prefix"
)]
fn lost_prepare(reboot: bool) {
    let (mut world, change, landing) = marked_prepare();
    let first = start_prepare(&mut world, landing);
    let branch = member_branch(&world, landing);
    let writer = tasks::Writer::Run { task: first.0, attempt: first.1 };
    assert_eq!(goal_branch::writer(&world, &branch), Some(writer));
    let _report = world.push_prepare();
    let prefix = world.fake.branch(b"org/repo", &branch).expect("lost worker's prefix");
    world.jobs.clear();
    world.forge_reads = Some(Vec::new());
    let mut held = Vec::new();
    world.send(protocol::lost(Token::new(7)));
    world.env.now = world.env.now.saturating_add(world.env.limits.fleet.grace).saturating_add(Duration::from_secs(1));
    for _ in 0..PAIR_TICKS {
        held.extend(take_member_answers(&mut world, &branch));
        world.tick();
        assert_unheld(&world);
        assert!(world.jobs.is_empty(), "no prepare while its writer awaits a read");
        assert_eq!(prepare_children(&world, landing), [first.0]);
        assert_eq!(claimed_attempts(&world, first.0), [first.1]);
        if !held.is_empty()
            && world.store.rows.values().any(|record| {
                matches!(record,
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row)))
                    if row.task == first.0 && row.attempt == first.1 && row.ended.is_some())
            })
        {
            break;
        }
    }
    assert!(!held.is_empty(), "loss asks a fresh read of the member's branch");
    assert_eq!(goal_branch::writer(&world, &branch), Some(writer), "loss retains the slot until read-back");
    let ended = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row)))
                if row.task == first.0 && row.attempt == first.1 =>
            {
                Some(row)
            }
            _ => None,
        })
        .expect("first attempt ended");
    let ended_at = ended.ended.expect("first ended before second claim");
    assert!(
        matches!(ended.end, Some(tasks::EndKind::Failed { class: tasks::Class::Lost, .. })),
        "worker lost past fleet grace"
    );
    if reboot {
        for _ in 0..PAIR_TICKS {
            if world.events.is_empty() && world.store.pending.is_empty() && world.pending.is_empty() {
                break;
            }
            held.extend(take_member_answers(&mut world, &branch));
            world.tick_without_resume();
        }
        assert_eq!(claimed_attempts(&world, first.0), [first.1], "retry claims only after restart");
        world.restart_same();
        // Answers withheld from the stopped process do not answer startup's fresh reads.
        held.clear();
        world.forge_reads = Some(Vec::new());
        world.events.retain(|event| {
            !matches!(event, engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { .. }, .. }))
        });
    }
    let mut hello = worker_hello(&world);
    if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut hello
    {
        hello.hosting = world
            .assigned
            .iter()
            .filter(|run| {
                matches!(task_row(&world, run.task).expect("assigned task").phase,
            tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == run.attempt)
            })
            .map(|run| protocol::hosting(run.task, run.attempt, jig_domain_link::Phase::Active))
            .collect();
    }
    world.send(hello);
    // Past the lost retry's backoff, a read still owns the first attempt's slot.
    world.env.now =
        world.env.now.saturating_add(world.env.limits.tasks.retries.lost.max).saturating_add(Duration::from_secs(1));
    for _ in 0..PAIR_TICKS {
        held.extend(take_member_answers(&mut world, &branch));
        world.tick();
        assert_eq!(goal_branch::writer(&world, &branch), Some(writer));
        assert_eq!(claimed_attempts(&world, first.0), [first.1], "no overlapping writer claims");
        if !held.is_empty() && world.events.is_empty() && world.store.pending.is_empty() && world.pending.is_empty() {
            break;
        }
    }
    assert!(!held.is_empty(), "the member's branch is read before retrying");
    for answer in held {
        world.send(answer);
    }
    for _ in 0..PAIR_TICKS {
        world.tick();
        assert_unheld(&world);
        if goal_branch::writer(&world, &branch) == Some(writer) {
            assert_eq!(claimed_attempts(&world, first.0), [first.1], "first writer excludes a second claim");
        }
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert_eq!(world.jobs.len(), 1, "one retry assignment after read-back");
    let second = (world.jobs[0].run.raw(), world.jobs[0].attempt.raw());
    assert_eq!(second.0, first.0);
    assert_ne!(second.1, first.1);
    assert_eq!(goal_branch::writer(&world, &branch), Some(tasks::Writer::Run { task: second.0, attempt: second.1 }));
    assert_eq!(claimed_attempts(&world, first.0), [first.1, second.1]);
    assert!(
        task_row(&world, second.0)
            .expect("retried child")
            .open_attempt
            .as_ref()
            .expect("second attempt")
            .claimed
            .expect("claimed")
            >= ended_at
    );
    answer_prepare(&mut world);
    await_landed(&mut world, landing);
    assert_landed(&world, landing, &[change]);
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landed");
    assert_eq!(row.landing.members[0].landed, Some(translate::commit(prefix)), "recovered prefix stays");
    assert_eq!(prepare_children(&world, landing), [first.0]);
    assert_prepare_history(&world, change, landing, &[first, second]);
    world.close();
}

#[test]
fn a_prepare_answer_kept_before_restart_is_read_without_asking_again() {
    let (mut world, change, landing) = marked_prepare();
    let job = start_prepare(&mut world, landing);
    let branch = member_branch(&world, landing);
    world.forge_reads = Some(Vec::new());
    answer_prepare(&mut world);
    let prefix = world.fake.branch(b"org/repo", &branch).expect("answered prefix");
    let mut held = Vec::new();
    for _ in 0..PAIR_TICKS {
        held.extend(take_member_answers(&mut world, b"main"));
        world.tick();
        if !held.is_empty()
            && ended(&world, job.0)
            && goal_branch::writer(&world, &branch).is_none()
            && world.events.is_empty()
            && world.store.pending.is_empty()
            && world.pending.is_empty()
        {
            break;
        }
    }
    assert!(!held.is_empty(), "landing's base read waits after the answer is saved");
    assert_eq!(goal_branch::writer(&world, &branch), None, "ended answer frees its writer before restart");
    assert!(ended(&world, job.0));
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Preparing);
    let owner = task_row(&world, landing).expect("landing task");
    assert!(
        owner
            .inbox
            .iter()
            .any(|word| word.from == tasks::Party::Task(job.0) && matches!(word.kind, tasks::MessageKind::Result(_))),
        "answer kept unread before a landing decision"
    );
    assert_eq!(row.landing.members[0].candidate, None, "answer not decided yet");
    assert_eq!(claimed_attempts(&world, job.0), [job.1]);
    assert_eq!(prepare_children(&world, landing), [job.0]);
    assert!(world.jobs.is_empty(), "worker ended");
    // The inbox answer survives; startup replaces the old process's withheld base read.
    world.restart_same();
    world.send(worker_hello(&world));
    await_landed(&mut world, landing);
    assert_landed(&world, landing, &[change]);
    assert_eq!(landing_rows(&world)[0].landing.members[0].landed, Some(translate::commit(prefix)));
    assert_eq!(prepare_children(&world, landing), [job.0]);
    assert_eq!(claimed_attempts(&world, job.0), [job.1]);
    assert_prepare_history(&world, change, landing, &[job]);
    world.close();
}

fn person(world: &mut World, key: u8, ask: people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [key; 16],
        ask,
    }));
}

#[test]
fn releasing_a_stopped_landing_replaces_its_cancelled_prepare_once() {
    let (mut world, change, landing) = marked_prepare();
    let first = start_prepare(&mut world, landing);
    person(&mut world, 96, people::Ask::Stop { project: 1, task: landing });
    for _ in 0..PAIR_TICKS {
        world.tick();
        if matches!(task_row(&world, landing).expect("stopped landing").phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(matches!(task_row(&world, landing).expect("stopped landing").phase, tasks::Phase::Held { .. }));
    assert_eq!(landing_rows(&world)[0].landing.phase, forge_top::LandingPhase::Preparing);
    person(
        &mut world,
        97,
        people::Ask::Cancel {
            project: 1,
            task: first.0,
            reason: b"end prepare while its landing is stopped".as_slice().into(),
            grace: None,
        },
    );
    let job = world.jobs.pop().expect("stopped worker's job");
    let event = world.protocol.answer_job(
        Token::new(7),
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Failed {
            failure: host::JobFailure::Cancelled(host::Reason::Engine),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
    for _ in 0..PAIR_TICKS {
        world.tick();
        if matches!(
            task_row(&world, first.0).expect("child").phase,
            tasks::Phase::Ended(tasks::Ending::Cancelled { .. })
        ) {
            break;
        }
    }
    assert!(
        matches!(task_row(&world, first.0).expect("child").phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })),
        "person's cancel ends the child while the landing stays stopped"
    );
    assert!(matches!(task_row(&world, landing).expect("stopped landing").phase, tasks::Phase::Held { .. }));
    let row = landing_rows(&world)[0];
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Preparing);
    assert_eq!(row.landing.members[0].asked, Some(1));
    assert_eq!(
        row.job.as_ref().expect("cancelled prepare").step.outcome,
        Some(forge_top::history::JobOutcome::Cancelled)
    );
    assert_eq!(prepare_children(&world, landing), [first.0]);
    assert_eq!(claimed_attempts(&world, first.0), [first.1]);
    assert!(world.jobs.is_empty(), "prepare absent at release");
    let revision = task_row(&world, landing).expect("held landing").revisions.hold;
    person(&mut world, 98, people::Ask::Release { project: 1, task: landing, revision, again: None });
    assert_eq!(await_prepare(&mut world), landing);
    let replacement = (world.jobs[0].run.raw(), world.jobs[0].attempt.raw());
    assert_ne!(replacement.0, first.0);
    assert_eq!(prepare_children(&world, landing), [first.0, replacement.0]);
    assert_eq!(claimed_attempts(&world, replacement.0), [replacement.1]);
    assert_prepare_record(&world, landing, replacement);
    answer_prepare(&mut world);
    let prefix = world.fake.branch(b"org/repo", &member_branch(&world, landing)).expect("replacement's prefix");
    await_landed(&mut world, landing);
    let row = landing_rows(&world)[0];
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(row.landing.preparation, 1);
    assert_eq!(row.landing.members.len(), 1);
    let member = &row.landing.members[0];
    assert_eq!(member.change, change);
    assert_eq!(member.left, None);
    assert!(member.told && ended(&world, change));
    assert_eq!(member.landed, Some(translate::commit(prefix)), "replacement's prefix lands");
    assert_eq!(goal_landing::row(&world, change).change.repairs, 0);
    assert_eq!(world.fake.branch(b"org/repo", b"main"), Some(prefix));
    assert_eq!(world.job_workspaces.len(), 2, "one assignment per child");
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Dispatch { branch, .. } if *branch == member.branch))
            .count(),
        1
    );
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Merge { number, how: api::MergeHow::FastForwardOnly, .. } if Some(*number) == row.landing.round.pull)).count(), 1);
    assert_prepare_history(&world, change, landing, &[first, replacement]);
    world.close();
}

fn take_member_answers(world: &mut World, branch: &[u8]) -> Vec<engine::Event> {
    let mut held = Vec::new();
    while let Some(index) = world.events.iter().position(|event| {
        matches!(event,
        engine::Event::Forge(event) if matches!(event.as_ref(),
            forge_top::Event::Client(skein_forge_client::Event::Answered { call, .. })
                if world.forge_reads.as_ref().expect("recorded reads").iter().any(|(issued, read)|
                    issued == call && matches!(read, api::Read::Branch { branch: name } if name.as_ref() == branch))))
    }) {
        held.push(world.events.remove(index).expect("selected branch answer"));
    }
    held
}

fn running_proof(world: &mut World, landing: u64) -> u64 {
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("proving landing");
    let candidate = row.landing.members[0].candidate.expect("candidate");
    let raw::Answer::Runs(runs) = world.external(
        1,
        raw::Op::Read(raw::Read::Runs {
            workflow: Some(b"gate.yml".as_slice().into()),
            commit: Some(fake::commit_count(&candidate).expect("fake candidate")),
            number: None,
            page: 1,
            limit: 2,
        }),
    ) else {
        panic!("dispatched runs")
    };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state, raw::RunState::Running, "the proof is in flight at the cut");
    runs[0].id
}

#[derive(Clone, Copy)]
enum Cut {
    Preparing,
    Proving,
    Turn,
}

#[test]
fn an_engine_restarts_in_each_turn_landing_phase() {
    let mut world = turn_world();
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    for (call, cut) in [(95, Cut::Preparing), (96, Cut::Proving), (97, Cut::Turn)] {
        let (change, producer) = pushed_change(&mut world, &chat, call);
        finish_producer(&mut world, &producer, forge_world::REPO);
        let landing = await_prepare(&mut world);
        let mut proof = None;
        if matches!(cut, Cut::Preparing) {
            let job = (world.jobs[0].run, world.jobs[0].attempt);
            restart(&mut world);
            assert_eq!(world.jobs.len(), 1);
            assert_eq!((world.jobs[0].run, world.jobs[0].attempt), job);
            answer_prepare(&mut world);
        } else {
            answer_prepare(&mut world);
            if matches!(cut, Cut::Turn) {
                world.hold_write_answers = Some(HoldWrite::Merge);
            }
            for _ in 0..PAIR_TICKS {
                tick(&mut world);
                assert_unheld(&world);
                assert!(world.jobs.is_empty(), "no second prepare");
                let ready = match cut {
                    Cut::Preparing => unreachable!(),
                    Cut::Proving => world.writes.iter().any(|write| matches!(write, api::Write::Dispatch { branch, .. } if *branch == landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing").landing.round.branch)),
                    Cut::Turn => !world.held_write_answers.is_empty(),
                };
                if ready {
                    break;
                }
            }
            world.drain_cut();
            let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("cut landing");
            match cut {
                Cut::Preparing => unreachable!(),
                Cut::Proving => {
                    assert_eq!(row.landing.phase, forge_top::LandingPhase::Proving);
                    proof = Some(running_proof(&mut world, landing));
                }
                Cut::Turn => {
                    assert!(matches!(row.landing.phase, forge_top::LandingPhase::Turn { .. }));
                    assert_eq!(world.held_write_answers.len(), 1, "merge made, answer held");
                }
            }
            world.restart_same();
            world.forge_reads = Some(Vec::new());
            world.send(worker_hello(&world));
            world.release_write_answers();
        }
        await_landed(&mut world, landing);
        assert_landed(&world, landing, &[change]);
        if let Some(run) = proof {
            assert!(
                world
                    .forge_reads
                    .as_ref()
                    .expect("recorded reads")
                    .iter()
                    .any(|(_, read)| matches!(read, api::Read::Run { run: found } if *found == run)),
                "restart finds the same dispatched run"
            );
        }
    }
    assert_eq!(landing_rows(&world).len(), 3);
    assert_eq!(world.job_workspaces.len(), 3);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 3);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Merge { .. })).count(), 3);
    assert_eq!(world.assigned.len(), 4, "one chat and three producers, no repairs");
    world.close();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one restart couples four goals' readiness, preparation and per-goal landing news"
)]
fn four_goals_land_together_after_loading_batch_four() {
    let mut world = turn_world();
    world.env.limits.tasks.project_goals = 4;
    world.env.limits.people.requests = 12;
    world.drain_cut();
    world.restart_with_batch(4);
    world.send(worker_hello(&world));
    world.until(Until::Ready);
    let mut goals = Vec::new();
    let mut changes = Vec::new();
    let mut producers = Vec::new();
    for key in 92..96 {
        let ask = walking::goal_from(
            world.chat(1, b"Make one fix into main"),
            people::TaskExecutor::Agent { charter: 1 },
            jig_test_profile::fixtures::Goal { budget: 100, priority: 3, deadline: None },
        );
        let goal = assigned_chat(&mut world, key, ask);
        let (change, producer) = pushed_change(&mut world, &goal, 100 + u64::from(key));
        goals.push(goal);
        changes.push(change);
        producers.push(producer);
    }
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("latch chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(20) };
    let latch_run = assigned_chat(&mut world, 96, people::Ask::Create { project, batch });
    let latch = latch_run.task;
    for (index, change) in changes.iter().enumerate() {
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(300 + *change)),
            sign_in: world.signed_in.expect("owner"),
            key: [u8::try_from(index + 100).expect("key"); 16],
            ask: people::Ask::Amend {
                project: 1,
                task: *change,
                revision: live_task(&world, *change).revisions.spec,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: Some(Box::new([latch])),
                    authority: None,
                    llm: None,
                    reason: b"Ready together".as_slice().into(),
                },
            },
        }));
        world.until(Until::Ready);
        assert_eq!(live_task(&world, *change).waiting_on.as_ref(), [latch]);
    }
    for producer in &producers {
        finish_producer(&mut world, producer, forge_world::REPO);
    }
    for _ in 0..PAIR_TICKS {
        tick(&mut world);
        if changes.iter().all(|change| {
            matches!(
                goal_landing::row(&world, *change).change.state,
                temper_engine_domain_forge_change::State::Gating { .. }
            )
        }) {
            break;
        }
    }
    assert!(
        changes.iter().all(|change| matches!(
            goal_landing::row(&world, *change).change.state,
            temper_engine_domain_forge_change::State::Gating { .. }
        )),
        "all four await the same readiness decision behind the latch: {:?}",
        changes.iter().map(|change| goal_landing::row(&world, *change).change.state).collect::<Vec<_>>()
    );
    assert!(landing_rows(&world).is_empty(), "the latch keeps all four changes");
    goal_landing::finish_run(
        &mut world,
        &latch_run,
        tasks::TaskResult::Report { words: b"Ready together".as_slice().into() },
    );
    let landing = await_prepare(&mut world);
    let job = &world.jobs[0];
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&job.spec),
    )
    .expect("prepare spec");
    let request: serde_json::Value = serde_json::from_slice(spec.request.as_ref()).expect("prepare request");
    assert_eq!(request["members"].as_array().expect("stack").len(), 4);
    // Equal goal priorities leave first-ready time, then task number, to order the queue.
    let mut queue = changes.clone();
    queue.sort_by_key(|change| (goal_landing::row(&world, *change).change.ready_since.expect("ready"), *change));
    answer_prepare(&mut world);
    await_landed(&mut world, landing);
    assert_eq!(landing_rows(&world).len(), 1);
    assert_landed(&world, landing, &queue);
    let row = landing_rows(&world)[0];
    for (goal, change) in goals.iter().zip(changes.iter()) {
        assert_eq!(goal_landing::row(&world, *change).goal, Some(goal.task));
        let member = row.landing.members.iter().find(|member| member.change == *change).expect("goal's member");
        let prefix = String::from_utf8(engine::commit_text(&member.landed.expect("prefix")).into_vec()).expect("hex");
        let expected = format!("landing {landing}: change {} landed at {prefix}", member.change);
        assert_eq!(
            live_task(&world, goal.task)
                .inbox
                .iter()
                .filter(|message| matches!(message.kind, tasks::MessageKind::News { .. })
                    && message.words.as_ref() == expected.as_bytes())
                .count(),
            1,
            "each goal hears its own prefix"
        );
    }
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 1);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Merge { .. })).count(), 1);
}
