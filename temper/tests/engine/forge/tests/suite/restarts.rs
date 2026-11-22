//! Each restart fault alone, at a named boundary (testing/restarts.md, 5 and 10).
use skein_fake_forge_domain::api as raw;
use skein_forge::api::{Error, Op, Read};
use skein_lib::{Duration, Time, Wall};
use skein_world::faults::{Kind, Struck};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_issues as issues;
use temper_engine_forge_world::{
    REPO, World,
    restarts::{self, BRANCH, GOAL, Injector},
};
use temper_faults::restarts as kinds;

pub(super) fn project(world: &mut World, entry: u64) {
    world.event(top::Event::Project {
        entry,
        repository: REPO,
        view: issues::GoalView {
            phase: issues::Phase::Waiting,
            goal: 42,
            repository: u64::from(REPO.repository),
            title: Box::from("Goal"),
            goal_text: Box::from("Plan"),
            plan: Box::new([]),
            milestones: Box::new([]),
            finished: None,
        },
    });
    world.event(top::Event::Committed { entry });
}

fn watching() -> World {
    let mut world = World::with_entries(377, 2);
    world.adopt();
    world.event(top::Event::Names {
        task: 51,
        resources: Box::new([top::Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: top::What::Branch(Box::new([b"main".as_slice().into()])),
        }]),
    });
    world.run_for(2);
    world.take_seen();
    world.take_calls();
    world
}

fn entry(injector: &Injector, kind: Kind, from: Time, until: Option<Time>) {
    let entries = injector.ledger().entries();
    assert_eq!(entries.len(), 1, "only the named fault");
    let entry = entries.first().expect("injected");
    assert_eq!(entry.kind, kind);
    assert_eq!(entry.from, from);
    assert_eq!(entry.until, until);
    assert_eq!(entry.struck, Struck::default());
}

fn stage_seen(world: &World, stage: top::RestartStage) -> bool {
    world.seen().contains(&top::Request::RestartDone { stage })
}

#[test]
fn crash_drops_the_in_flight_answer_and_finds_the_write_once() {
    let mut world = World::new(23);
    world.adopt();
    world.slow_calls(Duration::from_secs(1));
    project(&mut world, 1);
    let end = world.now().saturating_add(Duration::from_secs(2));
    for _ in 0..100 {
        if world.writes() == 1 {
            break;
        }
        assert!(world.step_until(end));
    }
    assert_eq!(world.writes(), 1, "write in flight, not drained");
    let from = world.now();
    let mut injector = Injector::new(23);
    injector.inject(&mut world, &[kinds::ENGINE_CRASH], None);
    world.run_for(20);
    injector.settled(&world);
    assert!(world.dropped_answers() > 0, "old generation reached no new connector");
    assert_eq!(world.writes(), 1);
    let raw::Answer::Items { items, .. } = world.inspect(&raw::Read::Items {
        order: raw::Order::LeastUpdated,
        since: Time::ZERO,
        page: 1,
        kind: Some(raw::Kind::Issue),
        state: None,
        labels: Box::new([]),
        author: None,
        limit: 8,
    }) else {
        panic!("issues")
    };
    assert_eq!(items.len(), 1, "the fake made the keyed issue once");
    assert!(!world.stored().contains_key(&top::Key::Entry(1)));
    let until = world.restart_settled_at().expect("settled at the boundary");
    assert!(until >= from && until <= world.now());
    entry(&injector, kinds::ENGINE_CRASH, from, Some(until));
}

#[test]
fn forge_down_until_refuses_every_call_then_reads_and_settles() {
    let mut world = watching();
    let from = world.now();
    let until = Wall::from_nanos(from.saturating_add(Duration::from_secs(5)).as_nanos());
    let mut injector = Injector::new(377);
    injector.inject(&mut world, &[kinds::FORGE_DOWN_UNTIL], Some(until));
    world.run_for(4);
    assert!(!stage_seen(&world, top::RestartStage::ReadAfresh));
    let calls = world.take_calls();
    assert_ne!(calls, []);
    assert_eq!(calls.len(), world.restart_answers().len(), "every outage call refused");
    assert!(world.restart_answers().iter().all(|(at, _, error)| *at < until && *error == Error::Unavailable));
    world.run_for(20);
    assert!(world.take_calls().iter().any(|op| matches!(op, Op::Read(Read::Branch { .. }))));
    assert!(stage_seen(&world, top::RestartStage::ReadAfresh));
    assert!(stage_seen(&world, top::RestartStage::Settled));
    entry(&injector, kinds::FORGE_DOWN_UNTIL, from, Some(Time::from_nanos(until.as_nanos())));
}

#[test]
fn forge_down_for_good_refuses_through_the_horizon() {
    let mut world = watching();
    let from = world.now();
    let mut injector = Injector::new(377);
    injector.inject(&mut world, &[kinds::FORGE_DOWN_FOR_GOOD], None);
    world.run_for(20);
    let calls = world.take_calls();
    assert!(calls.len() > 1, "the start retries");
    assert_eq!(calls.len(), world.restart_answers().len());
    assert!(world.restart_answers().iter().all(|(_, _, error)| *error == Error::Unavailable));
    assert!(!stage_seen(&world, top::RestartStage::ReadAfresh));
    assert_eq!(world.now(), from.saturating_add(Duration::from_secs(20)));
    entry(&injector, kinds::FORGE_DOWN_FOR_GOOD, from, None);
}

#[test]
fn ended_owners_fill_the_restored_outbox_and_a_new_making_waits() {
    let mut world = watching();
    let from = world.now();
    let mut injector = Injector::new(377);
    injector.inject(&mut world, &[kinds::ENTRIES_HELD_BY_ENDED], None);
    world.run_for(2);
    assert!(stage_seen(&world, top::RestartStage::Settled));
    for number in [1, 2] {
        let Some(top::Stored::Entry { entry, held: true }) = world.stored().get(&top::Key::Entry(number)) else {
            panic!("every entry stays held after restoration");
        };
        assert_eq!(entry.task, restarts::ENDED);
        assert!(matches!(entry.effect.write, skein_forge::api::Write::Dispatch { .. }));
    }
    assert_eq!(world.landing(restarts::ENDED).landing.phase, temper_engine_domain_forge_landing::Phase::Cancelled);
    world.event(top::Event::Hold { entry: 3, task: GOAL, resource: restarts::goal_name(), from: None });
    world.run_for(2);
    assert!(!world.has_branch(BRANCH));
    assert!(!world.stored().contains_key(&top::Key::Entry(3)), "no entry place admitted the write");
    assert!(world.take_calls().iter().all(|op| !matches!(op, Op::Write(_))));
    entry(&injector, kinds::ENTRIES_HELD_BY_ENDED, from, Some(from));
}

#[test]
fn missing_held_branch_is_read_before_it_is_made() {
    let mut world = watching();
    world.observe_entry_saves();
    let from = world.now();
    let mut injector = Injector::new(377);
    injector.inject(&mut world, &[kinds::BRANCH_THE_FORGE_LACKS], None);
    assert!(!world.has_branch(BRANCH));
    assert!(
        !world.stored().iter().any(|(_, row)| matches!(row, top::Stored::Entry { entry, .. } if entry.task == GOAL))
    );
    let read = Read::Branch { branch: BRANCH.into() };
    world.run_until_read(&read);
    let op = Op::Read(read);
    assert!(world.awaiting_forge_reply(&op), "the branch read is in flight");
    let end = world.now().saturating_add(Duration::from_secs(2));
    for _ in 0..100 {
        assert!(
            !world.seen().contains(&top::Request::ContinueGoalBranch { task: GOAL }),
            "the making waits for the branch read's answer"
        );
        if !world.awaiting_forge_reply(&op) {
            break;
        }
        assert!(world.step_until(end));
    }
    assert!(!world.awaiting_forge_reply(&op), "the branch read answered");
    let answered = world.seen().len();
    world.run_for(2);
    let seen = world.seen();
    let settle = seen
        .iter()
        .position(|r| *r == top::Request::RestartDone { stage: top::RestartStage::Settled })
        .expect("settle");
    let making = seen.iter().position(|r| *r == top::Request::ContinueGoalBranch { task: GOAL }).expect("making");
    assert!(answered <= making, "the branch read answers before the making request");
    assert!(settle < making, "the outbox settles before the making request");
    assert!(
        seen.iter().take(making).all(|request| !matches!(request,
            top::Request::Save { record: top::Stored::Entry { entry, .. }, .. } if entry.task == GOAL
        )),
        "no creation entry before the making request"
    );
    assert!(!world.has_branch(BRANCH));
    world.event(top::Event::ContinueGoalBranch { task: GOAL, entry: 3 });
    world.run_for(2);
    assert_eq!(world.branch(BRANCH), world.branch(b"main"));
    assert_eq!(world.writes(), 1);
    assert!(stage_seen(&world, top::RestartStage::ReadAfresh));
    entry(&injector, kinds::BRANCH_THE_FORGE_LACKS, from, Some(from));
}

#[test]
fn lost_made_writes_find_is_refused_after_restart() {
    let mut world = watching();
    world.lose_made_write_answers();
    project(&mut world, 1);
    let end = world.now().saturating_add(Duration::from_secs(5));
    for _ in 0..100 {
        if world.made_without_answer() == 1 {
            break;
        }
        assert!(world.step_until(end));
    }
    assert_eq!(world.made_without_answer(), 1);
    assert!(world.stored().contains_key(&top::Key::Entry(1)));
    world.take_calls();
    let from = world.now();
    let mut injector = Injector::new(377);
    injector.inject(&mut world, &[kinds::FIND_KEEPS_FAILING], None);
    world.run_for(20);
    let calls = world.take_calls();
    let finds: Vec<_> = calls
        .iter()
        .filter(|op| matches!(op, Op::Read(Read::Items { kind: Some(skein_forge::api::Kind::Issue), .. })))
        .collect();
    assert!(finds.len() > 1, "find retried");
    assert_eq!(finds.len(), world.restart_answers().len(), "every find refused");
    assert!(world.restart_answers().iter().all(|(_, _, error)| *error == Error::Unavailable));
    assert!(
        world.seen().contains(&top::Request::RestartDone { stage: top::RestartStage::ReadAfresh }),
        "other reads finish; only the uncertain write's find fails"
    );
    assert_eq!(world.writes(), 1, "made write not repeated");
    assert!(world.stored().contains_key(&top::Key::Entry(1)));
    entry(&injector, kinds::FIND_KEEPS_FAILING, from, None);
}

#[test]
fn startup_read_is_rate_limited_on_every_attempt() {
    let mut world = watching();
    let from = world.now();
    let mut injector = Injector::new(377);
    injector.inject(&mut world, &[kinds::READ_ALWAYS_LIMITED], None);
    world.run_for(20);
    let calls = world.take_calls();
    let reads: Vec<_> =
        calls.iter().filter(|op| **op == Op::Read(Read::Branch { branch: b"main".as_slice().into() })).collect();
    assert!(reads.len() > 1, "read retried");
    assert_eq!(reads.len(), world.restart_answers().len());
    assert!(
        world
            .restart_answers()
            .iter()
            .all(|(_, _, error)| matches!(error, Error::RateLimited { after } if *after > Duration::ZERO))
    );
    assert!(!stage_seen(&world, top::RestartStage::ReadAfresh));
    entry(&injector, kinds::READ_ALWAYS_LIMITED, from, None);
}
