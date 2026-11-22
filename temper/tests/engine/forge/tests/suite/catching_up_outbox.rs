//! An uncertain entry's find fences its lane, not other repositories or fresh reads.
use skein_forge::api::{Error, Op, Read, Write};
use skein_forge_client::{Condition, Effect, Entry};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{REPO, World, changes, translate};

const STUCK: &[u8] = b"stuck";
const BEHIND: &[u8] = b"behind";
const CHANGE: u64 = 52;
const BRANCH: &[u8] = b"temper/52";

fn branch_entry(world: &World, number: u64, branch: &[u8]) -> Entry {
    Entry {
        number,
        task: 70,
        repository: REPO,
        effect: Effect {
            write: Write::CreateBranch { branch: branch.into(), commit: translate::commit(world.branch(b"main")) },
            condition: Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}

fn no_settle(world: &World) {
    assert!(
        !world.seen().contains(&top::Request::RestartDone { stage: top::RestartStage::Settled }),
        "a find that keeps failing owes its first look"
    );
    assert!(!world.has_branch(BEHIND), "the later write on the uncertain entry's lane waits");
}

#[test]
fn a_restored_failing_find_holds_its_lane_but_not_a_change_on_another_repository() {
    let mut world = World::with_second_repository(378);
    world.adopt();
    world.produce_second(BRANCH);
    let mut row = changes::change_row();
    row.task = CHANGE;
    row.change.task = CHANGE;
    row.repository.repository = 3;
    row.branch = BRANCH.into();
    row.change.state = change::State::Opening { requested: false };
    world.event(top::Event::Change { row });
    world.script_branch_error(STUCK, Some(Error::Unavailable));
    world.lose_writes();
    world.event(top::Event::Enqueue { entry: branch_entry(&world, 1, STUCK) });
    world.event(top::Event::Committed { entry: 1 });
    world.run_for(1);
    assert!(matches!(
        world.stored().get(&top::Key::Entry(1)),
        Some(top::Stored::Entry { entry: Entry { attempt: Some(_), .. }, held: false })
    ));
    world.answer_writes();
    world.take_seen();
    world.take_calls();
    world.restart();
    world.run_for(2);
    assert!(world.seen().contains(&top::Request::RestartDone { stage: top::RestartStage::ReadAfresh }));
    no_settle(&world);
    world.event(top::Event::Enqueue { entry: branch_entry(&world, 2, BEHIND) });
    world.event(top::Event::Committed { entry: 2 });
    world.event(top::Event::StepChange {
        merge_job: false,
        task: CHANGE,
        entry: 3,
        heard: change::Heard {
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Passed,
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([]),
        kept: false,
    });
    world.run_for(1);
    no_settle(&world);
    assert!(world.seen().iter().any(|request| matches!(
        request,
        top::Request::ChangeDecision {
            task: CHANGE,
            decision: change::Decision::Effect(change::Effect::Open),
            entry: Some(3),
            ..
        }
    )));
    world.event(top::Event::Committed { entry: 3 });
    world.run_for(1);
    assert!(world.change(CHANGE).pull.is_some(), "the other repository's change opened its pull");
    assert!(!world.stored().contains_key(&top::Key::Entry(3)), "the change's write is made");
    world.run_for(8);
    no_settle(&world);
    let calls = world.take_calls();
    assert!(
        calls.iter().filter(|op| **op == Op::Read(Read::Branch { branch: STUCK.into() })).count() >= 3,
        "the find keeps failing, not merely unanswered once"
    );
    assert!(calls.iter().any(|op| matches!(op, Op::Write(Write::OpenPull { head, .. }) if head.as_ref() == BRANCH)));
    assert!(
        !calls
            .iter()
            .any(|op| matches!(op, Op::Write(Write::CreateBranch { branch, .. }) if branch.as_ref() == BEHIND)),
        "the same-lane write was never sent"
    );
}
