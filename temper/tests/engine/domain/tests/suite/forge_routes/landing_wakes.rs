//! Lifecycle wakes in the same root decision as their cause (temper/docs/design/domain/landing-liveness.md, 4).
use super::super::{goal_landing, reviews};
use super::restarts::{answer_prepare, await_prepare, marked_prepare, person, pushed_change, start_prepare};
use super::*;
use temper_engine_domain_forge_landing::{Leave, Member};

const PROJECT: u32 = 1;
// Enough world iterations to drain a lifecycle decision and its immediate fake forge reads.
// The tests of a wake check the commit that saves its cause, not a later poll.
const WAKE_TICKS: u32 = 200;
const STOP_MEMBER: u8 = 101;
const STOP_LANDING: u8 = 102;
const STOP_PREPARE: u8 = 103;
const CANCEL_MEMBER: u8 = 104;
const CANCEL_PREPARE: u8 = 105;

pub(crate) fn stop(world: &mut World, key: u8, task: u64) {
    person(world, key, people::Ask::Stop { project: PROJECT, task, reason: b"Park the task".as_slice().into() });
}

pub(crate) fn cancel(world: &mut World, key: u8, task: u64) {
    person(
        world,
        key,
        people::Ask::Cancel { project: PROJECT, task, reason: b"Withdraw the task".as_slice().into(), grace: None },
    );
}

pub(crate) fn answer_stopped_prepare(world: &mut World, child: u64) {
    let job = world.jobs.pop().expect("outstanding prepare");
    assert_eq!(job.run.raw(), child);
    let terminal = world.protocol.answer_job(
        Token::new(7),
        child,
        job.attempt.raw(),
        host::JobEnding::Failed {
            failure: host::JobFailure::Cancelled(host::Reason::Engine),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(terminal);
}

fn parked(world: &mut World, landing: u64) {
    for _ in 0..WAKE_TICKS {
        world.tick();
        if world.store.pending.is_empty()
            && world.pending.is_empty()
            && world.events.is_empty()
            && task_row(world, landing).expect("parked landing").phase == tasks::Phase::Active(tasks::Active::Idle)
        {
            return;
        }
    }
    panic!("landing did not park: {:?}", task_row(world, landing).expect("landing").phase);
}

pub(crate) fn waiting_prepare() -> (World, u64, u64, u64) {
    let (mut world, change, landing) = marked_prepare();
    let (child, _) = start_prepare(&mut world, landing);
    parked(&mut world, landing);
    (world, change, landing, child)
}

fn await_observation(world: &mut World, what: &str, met: impl Fn(&World) -> bool) {
    for _ in 0..WAKE_TICKS {
        if met(world) {
            return;
        }
        world.tick();
    }
    panic!("no {what} within {WAKE_TICKS} ticks");
}

fn landing_row(world: &World, landing: u64) -> &forge_top::LandingRow {
    landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("committed landing")
}

fn member(world: &World, landing: u64, change: u64) -> &Member {
    landing_row(world, landing).landing.members.iter().find(|row| row.change == change).expect("landing member")
}

fn commit_phase(write: &engine::Write, task: u64) -> Option<&tasks::Phase> {
    let engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) = write else {
        return None;
    };
    (row.number == task).then_some(&row.phase)
}

fn assert_wake(world: &World, task: u64, landing: u64, saved: impl Fn(&tasks::Phase) -> bool) {
    let writes = world
        .store
        .pending
        .iter()
        .find_map(|(_, writes)| {
            writes.iter().any(|write| commit_phase(write, task).is_some_and(&saved)).then_some(writes)
        })
        .expect("lifecycle decision's commit");
    assert!(
        writes.iter().any(|write| commit_phase(write, landing) == Some(&tasks::Phase::Active(tasks::Active::Due))),
        "the decision saving task {task}'s lifecycle change wakes landing {landing}"
    );
}

fn stop_and_check_wake(world: &mut World, change: u64, landing: u64) {
    stop(world, STOP_MEMBER, change);
    assert_wake(world, change, landing, |phase| matches!(phase, tasks::Phase::Held { .. }));
}

fn assert_held_leave(world: &mut World, change: u64, landing: u64, first: Option<skein_lib::Wall>) {
    await_observation(world, "held member leaving", |world| member(world, landing, change).left == Some(Leave::Held));
    assert_eq!(world.root.forge().landing_of(change), None);
    assert_eq!(world.store.change(change).expect("member keeps its place").change.ready_since, first);
}

#[test]
fn holding_the_only_member_wakes_and_ends_its_landing() {
    let (mut world, change, landing, child) = waiting_prepare();
    let first = world.store.change(change).expect("ready member").change.ready_since;
    stop_and_check_wake(&mut world, change, landing);
    assert_held_leave(&mut world, change, landing, first);
    cancel(&mut world, CANCEL_PREPARE, child);
    answer_stopped_prepare(&mut world, child);
    await_observation(&mut world, "landing ending", |world| ended(world, landing));
    world.close();
}

#[test]
fn holding_one_member_wakes_the_landing_for_the_other() {
    let (mut world, _, latch) = gating_pair();
    world.slow_branch = None;
    world.manual_prepare_jobs();
    goal_landing::finish_run(
        &mut world,
        &latch,
        tasks::TaskResult::Report { words: b"Release both".as_slice().into() },
    );
    let landing = await_prepare(&mut world);
    parked(&mut world, landing);
    let [kept, leaving] = landing_row(&world, landing).landing.members.as_ref() else { panic!("two members") };
    let (kept, leaving) = (kept.change, leaving.change);
    let first = world.store.change(leaving).expect("ready member").change.ready_since;
    stop_and_check_wake(&mut world, leaving, landing);
    assert_held_leave(&mut world, leaving, landing, first);
    answer_prepare(&mut world);
    for _ in 0..PAIR_TICKS {
        restarts::tick(&mut world);
        if ended(&world, kept) {
            break;
        }
    }
    let prefix = member(&world, landing, kept).landed.expect("the first member lands without the held suffix");
    // Give the held member's open pull the forge's new base before the restart comparison.
    let pull = member(&world, landing, leaving).pull;
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: api::Hint { repository: forge_world::REPO, change: api::Change::Item(pull), key: None },
    })));
    await_observation(&mut world, "held member's pull reading the new base", |world| {
        let key = Key::Forge(forge_top::Key::Client(skein_forge_client::Key::Live(skein_forge_client::Resource {
            repository: forge_world::REPO,
            what: skein_forge_client::What::Pull(pull),
        })));
        matches!(world.store.rows.get(&key), Some(Record::Forge { row })
            if matches!(row.as_ref(), forge_top::Stored::Client(skein_forge_client::Stored::Live(live))
                if live.cached.pull.as_ref().is_some_and(|pull| pull.base_commit == Some(prefix))))
    });
    world.close();
}

fn waiting_review() -> (World, u64, u64, u64, engine::Assignment) {
    let mut world = restarts::turn_world_configured(Gate { agent: true, ..Gate::default() }, None);
    let ask = walking::goal_from(
        world.chat(PROJECT, b"Make a reviewed change into main"),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 250, priority: 3, deadline: None },
    );
    let goal = assigned_chat(&mut world, 92, ask);
    let (change, producer) = pushed_change(&mut world, &goal, 95);
    finish_producer(&mut world, &producer, forge_world::REPO);
    let head_review = reviews::next_assignment(&mut world, 2);
    reviews::finish_review(&mut world, &head_review, approved());
    let landing = await_prepare(&mut world);
    answer_prepare(&mut world);
    let prefix_review = reviews::next_assignment(&mut world, 3);
    parked(&mut world, landing);
    (world, goal.task, change, landing, prefix_review)
}

fn approved() -> tasks::TaskResult {
    tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"Approved".as_slice().into() }
}

fn cancelled_departures(world: &World, change: u64, landing: u64) -> usize {
    world
        .store
        .rows
        .values()
        .filter(|record| {
            let Record::Forge { row } = record else { return false };
            matches!(row.as_ref(), forge_top::Stored::HistoryEvent(event)
            if event.change == change && event.landing.is_some_and(|within| within.task == landing)
                && matches!(event.what, forge_top::history::What::Departed { .. }))
        })
        .count()
}

#[test]
fn cancelling_a_member_wakes_its_landing_before_effects_settle() {
    const ONE_LEAVE: usize = 1;
    let (mut world, goal, change, landing, reviewer) = waiting_review();
    let words = format!("landing {landing}: change {change} left: cancelled");
    cancel(&mut world, CANCEL_MEMBER, change);
    assert_wake(&world, change, landing, |phase| matches!(phase, tasks::Phase::Closing(_)));
    await_observation(&mut world, "closing member leaving", |world| {
        member(world, landing, change).left == Some(Leave::Cancelled)
    });
    assert!(matches!(
        task_row(&world, change).expect("closing member").phase,
        tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Delegates, .. })
    ));
    assert!(matches!(
        task_row(&world, reviewer.task).expect("worker not yet answered").phase,
        tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Run { .. }, .. })
    ));
    await_observation(&mut world, "the goal hearing the leave", |world| {
        news::news(world, goal, landing).contains(&words)
    });
    world.send(protocol::shutdown(Token::new(7), reviewer.task, reviewer.attempt, 5));
    await_observation(&mut world, "cancelled member settling its effects", |world| {
        matches!(task_row(world, change).expect("member").phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. }))
    });
    world.drain_cut();
    assert_eq!(cancelled_departures(&world, change, landing), ONE_LEAVE);
    assert_eq!(news::news(&world, goal, landing).iter().filter(|line| **line == words).count(), ONE_LEAVE);
    world.close();
}

// Inject an expiring repair assignment so the real findings route issues tasks::Event::Fail.
fn expire_repair(world: &mut World, change: u64) {
    let finding = forge_top::Finding {
        project: PROJECT,
        number: 1,
        kind: forge_top::FindingKind::Red,
        severity: forge_top::Severity::High,
        area: None,
        state: forge_top::FindingState::Routed,
        title: b"Repair deadline".as_slice().into(),
        body: b"End the assigned repair at its deadline".as_slice().into(),
        filer: forge_top::Filer::Party(1),
        at: world.env.wall,
        why: None,
        hander: None,
        goal: Some(change),
        reason: None,
        signature: None,
        seen: None,
        blame: None,
        cleared: None,
        repair: Some(forge_top::FindingRepair {
            goal: change,
            made: true,
            attempts: 1,
            cancelled: false,
            failing: Box::new([]),
            until: Some(world.env.wall),
            judging: None,
        }),
    };
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Restore {
        record: forge_top::Stored::Finding(finding),
    })));
}

#[test]
fn a_failed_member_leaves_its_landing_as_cancelled() {
    let (mut world, _, change, landing, reviewer) = waiting_review();
    expire_repair(&mut world, change);
    await_observation(&mut world, "failed member leaving", |world| {
        member(world, landing, change).left == Some(Leave::Cancelled)
    });
    assert!(matches!(
        task_row(&world, change).expect("failed member").phase,
        tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Delegates, ending: tasks::Ending::Failed { .. } })
    ));
    world.send(protocol::shutdown(Token::new(7), reviewer.task, reviewer.attempt, 5));
    await_observation(&mut world, "failed member settling its effects", |world| {
        matches!(task_row(world, change).expect("member").phase, tasks::Phase::Ended(tasks::Ending::Failed { .. }))
    });
    world.close();
}

#[test]
fn holding_a_prepare_job_wakes_and_holds_its_landing() {
    let (mut world, _, landing, child) = waiting_prepare();
    stop(&mut world, STOP_PREPARE, child);
    assert_wake(&world, child, landing, |phase| matches!(phase, tasks::Phase::Held { .. }));
    await_observation(&mut world, "prepare hold reaching landing", |world| {
        matches!(task_row(world, landing).map(|row| &row.phase), Some(tasks::Phase::Held { source: Some(source), .. })
            if matches!(source.as_ref(), tasks::HoldSource::Account { words, .. } if words.starts_with(b"prepare job:")))
    });
    answer_stopped_prepare(&mut world, child);
    world.close();
}

#[test]
fn waking_a_held_landing_leaves_it_held() {
    let (mut world, change, landing, child) = waiting_prepare();
    stop(&mut world, STOP_LANDING, landing);
    world.drain_cut();
    let before = task_row(&world, landing).expect("held landing").phase.clone();
    assert!(matches!(before, tasks::Phase::Held { .. }));
    stop(&mut world, STOP_MEMBER, change);
    world.drain_cut();
    assert_eq!(task_row(&world, landing).expect("still held").phase, before);
    assert_eq!(member(&world, landing, change).left, None);
    answer_stopped_prepare(&mut world, child);
    world.close();
}
