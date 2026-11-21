//! A restart reads the held goal branch before settling and makes it after
//! (temper/docs/design/domain/forge/branches.md, 1).
use skein_forge::api::{Op, Write};
use skein_forge_client::{Condition, Effect, Entry};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{REPO, World};

const GOAL: u64 = 77;
const BRANCH: &[u8] = b"temper/77/g77";
const ENDED: u64 = 63;
const FRESH: u64 = 3;

fn name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([
            b"temper".as_slice().into(),
            b"77".as_slice().into(),
            b"g77".as_slice().into(),
        ])),
    }
}

fn before_restart() -> World {
    let mut world = World::with_entries(377, 2);
    world.adopt();
    world.hold_task(GOAL);
    world.store_before_restart(top::Stored::Hold(top::Hold {
        name: name(),
        task: GOAL,
        writer: None,
        beside: None,
        design_writers: Box::new([]),
        drift: None,
    }));
    world.take_seen();
    world.take_calls();
    world.observe_entry_saves();
    assert!(!world.has_branch(BRANCH));
    world
}

fn cancelled_landing() -> top::LandingRow {
    top::LandingRow {
        last_event: None,
        turn_since: None,
        waiting: None,
        room: None,
        job: None,
        landing: landing::Landing {
            last_left: None,
            searched: None,
            task: ENDED,
            project: 5,
            phase: landing::Phase::Cancelled,
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
        flake_reruns: Box::new([]),
    }
}

fn held_dispatch(number: u64) -> top::Stored {
    top::Stored::Entry {
        entry: Entry {
            number,
            task: ENDED,
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

fn restart_has_read_then_settled(world: &mut World) -> usize {
    world.restart();
    world.run_for(2);
    let seen = world.seen();
    let read = seen
        .iter()
        .position(|request| *request == top::Request::RestartDone { stage: top::RestartStage::ReadAfresh })
        .expect("reading afresh ends even if the outbox is full");
    let settled = seen
        .iter()
        .position(|request| *request == top::Request::RestartDone { stage: top::RestartStage::Settled })
        .expect("the restored outbox settles");
    assert!(read < settled);
    assert!(
        seen[..=settled].iter().all(|request| !matches!(
            request,
            top::Request::ContinueGoalBranch { task: GOAL }
                | top::Request::Save { record: top::Stored::Entry { entry: Entry { task: GOAL, .. }, .. }, .. }
        )),
        "no making is asked or saved before the settle: {seen:?}"
    );
    assert!(!world.has_branch(BRANCH));
    assert!(matches!(
        world.stored().get(&top::Key::Hold(name())),
        Some(top::Stored::Hold(top::Hold { writer: None, .. }))
    ));
    settled
}

fn finish_making(world: &mut World) {
    assert_eq!(
        world.seen().iter().filter(|request| **request == top::Request::ContinueGoalBranch { task: GOAL }).count(),
        1,
        "the line asks the making once"
    );
    world.event(top::Event::ContinueGoalBranch { task: GOAL, entry: FRESH });
    world.run_for(2);
    assert!(world.seen().iter().any(|request| matches!(
        request,
        top::Request::Save { record: top::Stored::Entry { entry: Entry { number: FRESH, task: GOAL, .. }, .. }, .. }
    )));
    // The world's scripted root commits the saved entry through its `release`.
    assert_eq!(world.branch(BRANCH), world.branch(b"main"));
    assert_eq!(world.writes(), 1);
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
}

#[test]
fn a_full_outbox_delays_a_restarted_goal_making_but_not_reading_afresh() {
    let mut world = before_restart();
    // Closing tasks stay held until their outbox entries have settled.
    world.hold_task(ENDED);
    world.store_before_restart(top::Stored::Landing(cancelled_landing()));
    for number in [1, 2] {
        world.store_before_restart(held_dispatch(number));
    }
    restart_has_read_then_settled(&mut world);
    assert!(!world.seen().contains(&top::Request::ContinueGoalBranch { task: GOAL }));
    for number in [1, 2] {
        assert!(matches!(world.stored().get(&top::Key::Entry(number)), Some(top::Stored::Entry { held: true, .. })));
    }
    world.event(top::Event::Resolve { entry: 1, again: false });
    finish_making(&mut world);
}

#[test]
fn a_restarted_goal_making_with_room_is_asked_only_after_the_settle() {
    let mut world = before_restart();
    let settled = restart_has_read_then_settled(&mut world);
    let asked = world
        .seen()
        .iter()
        .position(|request| *request == top::Request::ContinueGoalBranch { task: GOAL })
        .expect("room admits the making after the settle");
    assert!(settled < asked);
    finish_making(&mut world);
}
