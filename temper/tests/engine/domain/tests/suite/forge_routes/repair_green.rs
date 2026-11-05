//! Green CI ends an idle repair through jig (temper/docs/design/domain/forge.md, 8.4).
use super::*;
use temper_engine_domain_forge_change::{Repair, State};

const STEPS: u32 = 300; // The world's 100 ms ticks give each durable conversation 30 seconds.
const MADE: u32 = 3; // The change, its producer and its one repair.

fn row(world: &World, change: u64) -> &forge_top::ChangeRow {
    super::questions::row(world, change)
}

fn status(world: &mut World, head: u64, state: raw::Check) {
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
}

fn head(world: &mut World, branch: &[u8]) -> u64 {
    let raw::Answer::Commit(head) = world.external(1, raw::Op::Read(raw::Read::Branch { branch: branch.into() }))
    else {
        panic!("branch head")
    };
    head
}

fn setup(poll: Option<Duration>) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        0,
        Gate::default(),
        None,
        StoryLimits { stall: Some(Duration::from_secs(30 * 60)), ..StoryLimits::default() },
    );
    if let Some(poll) = poll {
        let mut client = world.env.limits.forge.client;
        client.poll = poll;
        client.poll_max = client.poll_max.max(poll);
        world = world.with_forge_client(client);
    }
    change_world_in(world, Change::default())
}

fn repair() -> (World, engine::Assignment, u64, u64) {
    repair_with_poll(None)
}

fn repair_with_poll(poll: Option<Duration>) -> (World, engine::Assignment, u64, u64) {
    let (mut world, _, producer, branch) = setup(poll);
    status(&mut world, 1, raw::Check::Passed);
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    let tasks::Party::Task(change) = live_task(&world, repair.task).requester else { panic!("change owns repair") };
    let head = head(&mut world, &branch);
    (world, repair, change, head)
}

fn answer(world: &mut World, repair: &engine::Assignment, end: tasks::End) {
    let event = world.protocol.answer(Token::new(7), repair.task, repair.attempt, 5, end, engine::Parts::none());
    world.send(event);
}

fn task(world: &World, task: u64) -> &tasks::TaskRecord {
    match ended(world, task) {
        Some(row) => row,
        None => live_task(world, task),
    }
}

fn ended(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) => Some(row),
        _ => None,
    }
}

fn wait_cancelled(world: &mut World, change: u64, repair: u64) {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, repair).is_some() && !matches!(row(world, change).change.state, State::Repairing { .. }) {
            break;
        }
    }
    let ended = ended(world, repair).expect("green head cancelled its repair");
    assert!(matches!(ended.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })), "{:?}", ended.phase);
    assert!(!matches!(row(world, change).change.state, State::Repairing { .. } | State::Held { .. }));
    assert_eq!(task(world, change).made, MADE);
    assert!(row(world, change).producer_failure.is_none());
    let shown = temper_engine_domain_world::tier_task_show::show(world, repair);
    assert_eq!(shown.status, 0);
    assert!(contains(&shown.stdout, b"CI passed at head"), "{}", String::from_utf8_lossy(&shown.stdout));
    assert!(!contains(&shown.stdout, b"Failed"), "{}", String::from_utf8_lossy(&shown.stdout));
}

fn retry(world: &mut World, repair: &engine::Assignment) {
    // Its next brief waits for access, keeping the task between runs while CI is observed.
    world.fail_brief_reads = Some((true, true, skein_forge::api::Error::NotConnected));
    answer(world, repair, tasks::End::Failed(tasks::Class::Run));
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, repair.task).phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })) {
            return;
        }
    }
    panic!("repair waits between attempts: {:?}", live_task(world, repair.task).phase);
}

#[test]
fn a_retrying_repair_ends_on_a_green_head() {
    let (mut world, repair, change, head) = repair();
    retry(&mut world, &repair);
    status(&mut world, head, raw::Check::Passed);
    wait_cancelled(&mut world, change, repair.task);
    assert_eq!(row(&world, change).change.repairs, 1);
    world.close();
}

#[test]
fn a_run_in_progress_finishes_at_a_green_head() {
    let (mut world, repair, change, head) = repair();
    status(&mut world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
    }
    assert!(matches!(live_task(&world, repair.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    assert_eq!(
        row(&world, change).delegate,
        Some((repair.task, temper_engine_domain_forge_change::Delegate::Repair(Repair::Ci)))
    );
    answer(
        &mut world,
        &repair,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"checked".as_slice().into(),
            },
            cancel_delegates: false,
        },
    );
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, repair.task).is_some() && !matches!(row(&world, change).change.state, State::Repairing { .. })
        {
            break;
        }
    }
    assert!(matches!(
        ended(&world, repair.task).expect("repair ended").phase,
        tasks::Phase::Ended(tasks::Ending::Done(_))
    ));
    assert!(!matches!(row(&world, change).change.state, State::Repairing { .. } | State::Held { .. }));
    world.close();
}

#[test]
fn a_retry_after_green_is_found_within_a_poll() {
    // A long poll leaves room for the fixture's 100 ms bookkeeping ticks inside one poll.
    let (mut world, repair, change, head) = repair_with_poll(Some(Duration::from_secs(60)));
    status(&mut world, head, raw::Check::Passed);
    let green_by =
        world.env.now.saturating_add(world.env.limits.forge.client.poll_max).saturating_add(Duration::from_secs(5));
    while world.env.now <= green_by && row(&world, change).result.checks.bytes.as_ref() != b"build passed" {
        world.tick();
    }
    assert_eq!(row(&world, change).result.checks.bytes.as_ref(), b"build passed");
    let half_poll = Duration::from_nanos(world.env.limits.forge.client.poll.as_nanos() / 2);
    let retry_at = world.env.now.saturating_add(half_poll);
    while world.env.now < retry_at {
        world.tick();
    }
    retry(&mut world, &repair);
    let deadline = world.env.now.saturating_add(world.env.limits.forge.client.poll);
    while world.env.now <= deadline && ended(&world, repair.task).is_none() {
        world.tick();
    }
    assert!(
        ended(&world, repair.task).is_some(),
        "retry found within poll: phase={:?}, poll={:?}, now={:?}, deadline={:?}",
        world.root.tasks().task(repair.task).map(|row| &row.phase),
        world.env.limits.forge.client.poll,
        world.env.now,
        deadline
    );
    wait_cancelled(&mut world, change, repair.task);
    world.close();
}

#[test]
fn a_cancelled_repair_released_at_a_green_head_makes_no_fresh_one() {
    let (mut world, repair, change, head) = repair();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner"),
        key: [95; 16],
        ask: people::Ask::Cancel { project: 1, task: repair.task, reason: b"try again".as_slice().into(), grace: None },
    }));
    answer(&mut world, &repair, tasks::End::Parked);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    let revision = live_task(&world, change).revisions.hold;
    status(&mut world, head, raw::Check::Passed);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(960)),
        sign_in: world.signed_in.expect("owner"),
        key: [96; 16],
        ask: people::Ask::Release { project: 1, task: change, revision, again: None },
    }));
    for _ in 0..STEPS {
        world.tick();
    }
    assert_eq!(task(&world, change).made, MADE);
    assert!(row(&world, change).delegate.is_none());
    assert_eq!(row(&world, change).change.repairs, 0);
    assert!(!matches!(row(&world, change).change.state, State::Repairing { .. } | State::Held { .. }));
    world.close();
}

#[test]
fn a_failed_repair_at_a_green_head_holds_nothing() {
    let (mut world, repair, change, head) = repair();
    status(&mut world, head, raw::Check::Passed);
    answer(
        &mut world,
        &repair,
        tasks::End::Finished {
            result: tasks::TaskResult::Failure { reason: b"nothing to repair".as_slice().into() },
            cancel_delegates: false,
        },
    );
    for _ in 0..STEPS {
        world.tick();
    }
    assert!(
        !matches!(row(&world, change).change.state, State::Repairing { .. } | State::Held { .. }),
        "{:?}",
        row(&world, change)
    );
    assert_eq!(task(&world, change).made, MADE);
    world.close();
}

#[test]
fn a_repair_held_for_its_brief_ends_on_a_green_head() {
    let (mut world, _, producer, branch) = setup(None);
    status(&mut world, 1, raw::Check::Passed);
    world.fail_brief_reads = Some((true, true, skein_forge::api::Error::InvalidAnswer));
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1).expect("pushed");
    status(&mut world, head, raw::Check::Failed);
    answer(
        &mut world,
        &producer,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
    );
    let mut found = None;
    for _ in 0..STEPS {
        world.tick();
        found = world.store.rows.values().find_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
                if task.spec.words.starts_with(b"Repair CI at ") && matches!(task.phase, tasks::Phase::Held { .. }) =>
            {
                let tasks::Party::Task(change) = task.requester else { panic!("repair owner") };
                Some((change, task.number))
            }
            _ => None,
        });
        if found.is_some() {
            break;
        }
    }
    let (change, repair) = found.expect("brief holds its repair");
    assert!(matches!(row(&world, change).change.state, State::Repairing { .. }));
    world.fail_brief_reads = None;
    status(&mut world, head, raw::Check::Passed);
    wait_cancelled(&mut world, change, repair);
    world.close();
}
