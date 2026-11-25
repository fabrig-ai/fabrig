//! P58 residual: a no-test repair should regate without a person's release.
//! temper/docs/design/domain/forge/change/repairs.md, A landing repair.
use super::*;
use crate::forge_routes::{landing::produced_turn_change_with, reviews::finish_review};

// Includes the prepare, gate, repair delivery and one new gate dispatch.
const SETTLE_TICKS: u32 = 1200;

fn dispatches(world: &World) -> usize {
    world.writes.iter().filter(|write| matches!(write, skein_forge::api::Write::Dispatch { .. })).count()
}

fn finish_no_test(world: &mut World, repair: &engine::Assignment) {
    finish_review(world, repair, tasks::TaskResult::Verdict {
        code: forge::REPAIR_NO_TEST,
        fields: Box::new([]),
        words: b"The gate's jobs name no failing test. The named tests pass at the unchanged head; nothing needs changing.".as_slice().into(),
    });
}

#[test]
#[ignore = "a change's reds: P58, passing repair regates without release"]
fn a_repair_whose_tests_pass_regates_by_itself() {
    let (mut world, task) = produced_turn_change_with(&StoryLimits {
        turn: true,
        dispatch_passes: Some(0),
        spec_bytes: Some(4096),
        ..StoryLimits::default()
    });
    let mut repair = None;
    for _ in 0..SETTLE_TICKS {
        world.prepare_tick();
        if let Some((child, change::Delegate::Repair(why))) = change_row(&world, task).delegate {
            assert!(matches!(why, change::Repair::Landing { .. } | change::Repair::LandingAt { .. }));
            if let Some(run) = world.assigned.iter().find(|run| run.task == child) {
                repair = Some(run.clone());
                break;
            }
        }
    }
    let repair = repair.expect("failed landing gate assigns a repair");
    assert!(
        task_row(&world, repair.task)
            .expect("repair task")
            .spec
            .words
            .windows(b"No test:".len())
            .any(|w| w == b"No test:"),
        "the repair offers the typed no-test verdict"
    );
    let row = change_row(&world, task);
    let head = world.fake.branch(b"org/repo", &row.branch).expect("repair head");
    let before = dispatches(&world);
    assert_eq!(before, 1, "one failed landing gate");
    finish_no_test(&mut world, &repair);
    for _ in 0..SETTLE_TICKS {
        world.prepare_tick();
        if dispatches(&world) > before {
            break;
        }
    }
    assert_eq!(world.fake.branch(b"org/repo", &row.branch), Some(head), "no new commit");
    assert_eq!(
        dispatches(&world),
        before + 1,
        "a passing no-test repair must regate once without a person's release; state: {:?}",
        change_row(&world, task).change.state
    );
    assert!(!matches!(task_row(&world, task).expect("change").phase, tasks::Phase::Held { .. }));
    world.close();
}
