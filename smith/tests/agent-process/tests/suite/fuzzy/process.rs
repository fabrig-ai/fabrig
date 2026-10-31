//! Seeded process schedules across full answers and cancellation after admission.

use smith_agent_process_world::{World, charter};
use smith_channel::{RunFailure, RunResult};

fn settle(world: &mut World) {
    assert!(world.settle(), "the agent answered");
    assert_eq!(world.observed().iter().filter(|frame| frame.kind == 0x0110).count(), 1);
    world.assert_agent_clean();
}

#[test]
fn seeded_completed_runs_answer_once_and_release_inherited_descriptors() {
    let charter = charter();
    for seed in 0..64_u64 {
        skein_world::domain::assert_replays(seed, seed + 1, |seed| {
            let mut world = World::new(seed, &charter);
            settle(&mut world);
            assert!(world.peer_replied(), "seed {seed} received model response");
            (world.trace(), world.answer())
        });
    }
}

#[test]
fn seeded_cancelled_runs_answer_once_and_release_inherited_descriptors() {
    let charter = charter();
    for seed in 0..64_u64 {
        skein_world::domain::assert_replays(seed, seed + 1, |seed| {
            let mut world = World::new(seed, &charter);
            world.signal();
            settle(&mut world);
            let answer = world.answer().expect("terminal answer");
            assert!(
                matches!(answer.result(), RunResult::Failed(failed) if failed.reason() == &RunFailure::Cancelled),
                "seed {seed} cancelled"
            );
            (world.trace(), world.answer())
        });
    }
}
