//! A cancelled delegate's account and release (temper/docs/design/domain/forge.md, 8.4).
use super::*;
use temper_engine_domain_forge_change::{Hold, State};

const STEPS: u32 = 200;
const REASON: &[u8] = b"try a different approach";

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn change_task(world: &World, delegate: u64) -> u64 {
    let tasks::Party::Task(task) = live_task(world, delegate).requester else { panic!("change's delegate") };
    task
}

fn cancel(world: &mut World, run: &engine::Assignment, change: u64, reason: &[u8]) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [95; 16],
        ask: people::Ask::Cancel { project: 1, task: run.task, reason: reason.into(), grace: None },
    }));
    // The scripted host acknowledges the cancelled run, releasing its writer.
    let event =
        world.protocol.answer(Token::new(7), run.task, run.attempt, 5, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, change).phase, tasks::Phase::Held { .. }) {
            return;
        }
    }
    panic!("cancelled delegate did not hold its change: {:?}", change_row(world, change));
}

fn account(world: &World, change: u64, expected: tasks::Hold) -> &[u8] {
    let tasks::Phase::Held { why, source: Some(source), .. } = &live_task(world, change).phase else {
        panic!("hold with account: {:?}", live_task(world, change).phase)
    };
    assert_eq!(*why, expected);
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("delegate's account") };
    words
}

fn cancelled_repair(reason: &[u8]) -> (World, u64, u64) {
    let (mut world, _, producer, branch) = change_world(false, 0, None);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: 1,
            context: b"build".as_slice().into(),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    let change = change_task(&world, repair.task);
    cancel(&mut world, &repair, change, reason);
    (world, change, repair.task)
}

#[test]
fn a_cancelled_repair_holds_saying_it_was_cancelled() {
    for reason in [REASON, b"".as_slice()] {
        let (mut world, change, repair) = cancelled_repair(reason);
        let opening = format!("its repair, task {repair}, was cancelled");
        let expected =
            if reason.is_empty() { opening.into_bytes() } else { [opening.as_bytes(), b": ", reason].concat() };
        assert_eq!(account(&world, change, tasks::Hold::Procedure), expected.as_slice());
        assert!(matches!(&change_row(&world, change).change.state, State::Held { was, why: Hold::Failed }
            if matches!(was.as_ref(), State::Repairing { .. })));
        assert_eq!(change_row(&world, change).change.repairs, 1, "cancel keeps the attempted count");
        let shown = temper_engine_domain_world::tier_task_show::show(&mut world, change);
        assert_eq!(shown.status, 0);
        assert!(contains(&shown.stdout, &expected), "{}", String::from_utf8_lossy(&shown.stdout));
        assert!(!contains(&shown.stdout, b"An effect failed permanently"));
    }
}

#[test]
fn a_released_cancelled_repair_asks_again_at_a_head_still_failing() {
    let (mut world, change, repair) = cancelled_repair(REASON);
    let revision = live_task(&world, change).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(960)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [96; 16],
        ask: people::Ask::Release { project: 1, task: change, revision, again: None },
    }));
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, change).delegate.is_some() {
            break;
        }
    }
    let row = change_row(&world, change);
    let (fresh, _) = row.delegate.expect("replacement repair");
    assert_ne!(fresh, repair);
    assert_eq!(row.change.repairs, 1, "replacement spends the reset allowance");
    assert!(row.producer_failure.is_none(), "fresh delegation clears the cancellation account");
    assert!(matches!(row.change.state, State::Repairing { .. }));
    for _ in 0..STEPS {
        world.tick();
    }
    assert_eq!(change_row(&world, change).delegate.expect("same replacement").0, fresh);
    assert_eq!(live_task(&world, change).made, 4, "change, producer and exactly two repairs");
}

#[test]
fn a_cancelled_producer_says_it_was_cancelled() {
    let (mut world, _, producer, _) = change_world(false, 1000, None);
    let change = change_task(&world, producer.task);
    cancel(&mut world, &producer, change, REASON);
    let expected = format!("its producer, task {}, was cancelled: try a different approach", producer.task);
    assert_eq!(account(&world, change, tasks::Hold::EffectFailed), expected.as_bytes());
    assert!(matches!(&change_row(&world, change).change.state, State::Held { was, why: Hold::Failed }
        if matches!(was.as_ref(), State::Producing { .. })));
}
