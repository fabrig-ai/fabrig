//! A restart makes a missing goal branch from the beginning of catching up
//! (temper/docs/design/domain/forge/branches.md, 1).
use skein_forge::api::{Op, Write};
use skein_forge_client::{Condition, Effect, Entry};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World,
    restarts::{BRANCH, GOAL, goal_name as name, synthetic_missing_branch},
};

const OWNER: u64 = 63;
const FRESH: u64 = 3;
const SLOW: &[u8] = b"slow-read";

pub(super) fn before_restart() -> World {
    let mut world = World::with_entries(377, 2);
    world.adopt();
    synthetic_missing_branch(&mut world);
    world.store_before_restart(top::Stored::Names {
        task: 88,
        resources: Box::new([top::Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: top::What::Branch(Box::new([SLOW.into()])),
        }]),
    });
    world.script_branch_error(SLOW, Some(skein_forge::api::Error::Unavailable));
    world.take_seen();
    world.take_calls();
    world.observe_entry_saves();
    assert!(!world.has_branch(BRANCH));
    world
}

pub(super) fn held_landing() -> top::LandingRow {
    top::LandingRow {
        read_hold: None,
        last_event: None,
        turn_since: None,
        waiting: Some(landing::Awaited::Release),
        room: None,
        job: None,
        landing: landing::Landing {
            last_left: None,
            searched: None,
            task: OWNER,
            project: 5,
            phase: landing::Phase::Held { was: Box::new(landing::Phase::Waiting), why: landing::Hold::Prepare },
            members: Box::new([]),
            preparation: 0,
            stale_proofs: 0,
            branch: b"main".as_slice().into(),
            round: landing::Round::new(b"temper/63/l63".as_slice().into()),
            last_prepare: None,
        },
        asks: Box::new([]),
        entries: [1, 2]
            .map(|entry| top::LandingEntry {
                preparation: None,
                base: None,
                entry,
                member: 0,
                gate: 0,
                operation: top::Operation::Dispatch,
                run: None,
            })
            .into(),
        follows: None,
        flake_reruns: Box::new([]),
    }
}

pub(super) fn held_dispatch(number: u64) -> top::Stored {
    top::Stored::Entry {
        entry: Entry {
            number,
            task: OWNER,
            repository: REPO,
            effect: Effect {
                write: Write::Dispatch {
                    workflow: b"gate.yml".as_slice().into(),
                    branch: b"main".as_slice().into(),
                    inputs: Box::new([]),
                },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
        held: true,
    }
}

fn restart_with_other_fresh_reads_out(world: &mut World) {
    world.restart();
    world.run_for(2);
    assert!(world.seen().contains(&top::Request::RestartDone { stage: top::RestartStage::Settled }));
    assert!(!world.seen().contains(&top::Request::RestartDone { stage: top::RestartStage::ReadAfresh }));
    assert!(!world.has_branch(BRANCH));
    assert!(matches!(
        world.stored().get(&top::Key::Hold(name())),
        Some(top::Stored::Hold(top::Hold { writer: None, .. }))
    ));
}

fn finish_making(world: &mut World) {
    assert_eq!(
        world.seen().iter().filter(|request| **request == top::Request::ContinueGoalBranch { task: GOAL }).count(),
        1,
        "the line asks the making once"
    );
    world.push_file(b"main", b"new-tip", b"tip after restart");
    world.event(top::Event::ContinueGoalBranch { task: GOAL, entry: FRESH });
    world.run_for(2);
    assert!(world.seen().iter().any(|request| matches!(
        request,
        top::Request::Save { record: top::Stored::Entry { entry: Entry { number: FRESH, task: GOAL, .. }, .. }, .. }
    )));
    // The world's scripted root commits the saved entry through its `release`.
    assert_eq!(world.branch(BRANCH), world.branch(b"main"));
    assert_eq!(world.writes(), 1);
    assert!(
        !world.seen().contains(&top::Request::RestartDone { stage: top::RestartStage::ReadAfresh }),
        "a making is made while other resources still owe fresh reads"
    );
    assert_eq!(
        world
            .take_calls()
            .iter()
            .filter(|op| matches!(
                op, Op::Write(Write::CreateBranch { branch, .. }) if branch.as_ref() == BRANCH
            ))
            .count(),
        1
    );
    assert!(world.seen().iter().all(|request| !matches!(request, top::Request::MakeFailed { .. })));
    world.script_branch_error(SLOW, None);
    world.run_for(8);
    assert_eq!(
        world
            .seen()
            .iter()
            .filter(|request| **request == top::Request::RestartDone { stage: top::RestartStage::ReadAfresh })
            .count(),
        1,
        "only the other resource held the fresh pass open"
    );
}

#[test]
fn a_full_outbox_lines_up_a_restarted_goal_making_until_a_holder_is_released() {
    let mut world = before_restart();
    // Waiting tasks stay held until their outbox entries have settled.
    world.hold_task(OWNER);
    world.store_before_restart(top::Stored::Landing(held_landing()));
    for number in [1, 2] {
        world.store_before_restart(held_dispatch(number));
    }
    restart_with_other_fresh_reads_out(&mut world);
    assert!(!world.seen().contains(&top::Request::ContinueGoalBranch { task: GOAL }));
    for number in [1, 2] {
        assert!(matches!(world.stored().get(&top::Key::Entry(number)), Some(top::Stored::Entry { held: true, .. })));
    }
    world.event(top::Event::Resolve { entry: 1, again: false });
    finish_making(&mut world);
}

#[test]
fn a_restarted_goal_making_with_room_is_made_while_other_fresh_reads_are_out() {
    let mut world = before_restart();
    restart_with_other_fresh_reads_out(&mut world);
    assert!(
        world.seen().contains(&top::Request::ContinueGoalBranch { task: GOAL }),
        "room admits the making from the beginning of catching up"
    );
    finish_making(&mut world);
}
