//! Completed changes stop reading their work (domain/forge/keeping-up.md, 3).
use super::*;
use skein_forge::api::{Change as HintChange, Hint, Op, Read};
use temper_engine_domain_forge_change as change;

const TASK: u64 = 51;
const BRANCH: &[u8] = b"temper/51";

fn watched() -> forge_world::World {
    let mut world = forge_world::World::new(17);
    world.adopt();
    world.produce(BRANCH);
    let pull = world.open_pull(BRANCH, b"main");
    let mut row = forge_world::changes::change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(translate::commit(world.branch(BRANCH)));
    row.change.state = change::State::Checking { head: row.change.last_head.expect("produced head") };
    world.event(forge_top::Event::Change { row });
    world.event(forge_top::Event::Names {
        task: TASK,
        resources: Box::new([forge_top::Name {
            forge: forge_world::REPO.forge,
            repository: forge_world::REPO.repository,
            what: forge_top::What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
        }]),
    });
    world.run_for(2);
    world
}

fn move_after_completion(world: &mut forge_world::World) {
    world.take_calls();
    world.push(BRANCH, b"later work");
    world.event(forge_top::Event::Hint {
        hint: Hint { repository: forge_world::REPO, change: HintChange::Branch(BRANCH.into()), key: None },
    });
    world.push_file(b"main", b"base", b"later base");
    world.event(forge_top::Event::Hint {
        hint: Hint { repository: forge_world::REPO, change: HintChange::Branch(b"main".as_slice().into()), key: None },
    });
    world.run_for(20);
    forge_world::changes::change_step(world, 91, change::Status::Passed, Box::new([]));
    let calls = world.take_calls();
    assert!(
        calls.iter().any(|op| matches!(op, Op::Read(Read::Branch { branch }) if branch.as_ref() == BRANCH)),
        "the watched move was read"
    );
    assert_eq!(
        calls.iter().filter(|op| matches!(op, Op::Read(Read::Compare { .. }))).count(),
        0,
        "no comparison after completion: {calls:?}"
    );
    assert_eq!(
        calls.iter().filter(|op| matches!(op, Op::Read(Read::File { .. }))).count(),
        0,
        "no raw file read after completion: {calls:?}"
    );
}

#[test]
fn an_ended_changes_branch_move_reads_no_comparison_or_file() {
    let mut world = watched();
    world.event(forge_top::Event::StepChange {
        task: TASK,
        entry: 90,
        merge_job: false,
        heard: change::Heard {
            delegate: change::Status::Passed,
            delegate_running: false,
            delegate_live: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: true,
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: false,
    });
    world.run_for(2);
    assert_eq!(world.change(TASK).change.state, change::State::Ended);
    move_after_completion(&mut world);
}

#[test]
fn a_branch_comparison_in_flight_at_landing_is_dropped() {
    let mut world = watched();
    world.event(forge_top::Event::Subscribe {
        subscription: forge_top::Subscriber {
            task: 7,
            number: 1,
            topic: forge_top::Topic::Landings { repository: forge_world::REPO, branch: BRANCH.into() },
            own_change: None,
            goal_tasks: None,
            paths: Box::new([]),
        },
    });
    world.take_calls();
    world.slow_calls(Duration::from_secs(3));
    world.push(BRANCH, b"work before landing");
    world.event(forge_top::Event::Hint {
        hint: Hint { repository: forge_world::REPO, change: HintChange::Branch(BRANCH.into()), key: None },
    });
    world.run_for(4);
    let calls = world.take_calls();
    assert_eq!(
        calls.iter().filter(|op| matches!(op, Op::Read(Read::Compare { .. }))).count(),
        1,
        "a comparison has been sent and still awaits its answer: {calls:?}"
    );
    world.slow_calls(Duration::ZERO);
    let pull = world.change(TASK).pull.expect("open pull");
    let merge = world.merge(pull, BRANCH);
    forge_world::changes::change_step(&mut world, 90, change::Status::Passed, Box::new([]));
    assert!(matches!(world.change(TASK).change.state, change::State::Landed { merge: landed, .. } if landed == merge));
    world.take_seen();
    world.take_calls();
    world.run_for(20);
    assert!(
        !world.seen().iter().any(|request| matches!(request, forge_top::Request::News { .. })),
        "the late comparison publishes no facts: {:?}",
        world.seen()
    );
    let calls = world.take_calls();
    assert_eq!(
        calls.iter().filter(|op| matches!(op, Op::Read(Read::Compare { .. } | Read::File { .. }))).count(),
        0,
        "no read follows the late answer: {calls:?}"
    );
}

#[test]
fn a_landed_changes_branch_move_reads_no_comparison_or_file() {
    let mut world = watched();
    let pull = world.change(TASK).pull.expect("open pull");
    let merge = world.merge(pull, BRANCH);
    forge_world::changes::change_step(&mut world, 90, change::Status::Passed, Box::new([]));
    assert!(matches!(world.change(TASK).change.state, change::State::Landed { merge: landed, .. } if landed == merge));
    move_after_completion(&mut world);
}
