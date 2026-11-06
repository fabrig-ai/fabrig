//! A live delegate's wait and the result it delivers (temper/docs/design/domain/forge.md, 8.4).
use super::early_start::{goal_world, make, observe_push, push, stop};
use super::questions::{finish_producing, next_producer, row};
use super::*;
use temper_engine_domain_forge_change::State;

#[test]
fn a_live_producer_past_the_stall_delivers_its_change() {
    let mut held = Vec::new();
    for dependency in [false, true] {
        let (mut world, goal, _, main) = goal_world();
        let after = if dependency {
            let first = make(&mut world, &goal, b"Dependency", &[]);
            let producer = next_producer(&mut world, first);
            Some((first, producer))
        } else {
            None
        };
        let change = make(
            &mut world,
            &goal,
            b"Slow producer",
            &after.as_ref().map(|(task, _)| *task).into_iter().collect::<Vec<_>>(),
        );
        if let Some((_first, producer)) = after {
            let until = world.env.now.saturating_add(Duration::from_secs(20));
            while world.env.now < until {
                world.tick();
            }
            assert_eq!(row(&world, change).change.state, State::Producing { requested: false });
            push(&mut world, &producer, main, b"dependency");
            observe_push(&mut world, &producer);
        }
        let producer = next_producer(&mut world, change);
        observe_push(&mut world, &producer);
        for _ in 0..40 {
            world.tick();
        }
        let requested = row(&world, change).change.since;
        let stall = world.env.limits.forge.change_policy.stall;
        let until = requested.as_nanos().saturating_add(stall.as_nanos());
        while world.env.wall.as_nanos() < until || !world.store.pending.is_empty() {
            world.tick();
        }
        // Observe the deadline's first completed step, including asynchronous reads.
        for _ in 0..400 {
            if matches!(row(&world, change).change.state, State::Held { .. }) {
                break;
            }
            world.tick();
        }
        let before = row(&world, change).change.state.clone();
        finish_producing(&mut world, change, &producer);
        for _ in 0..100 {
            world.tick();
        }
        held.push(matches!(before, State::Held { .. }));
        assert!(row(&world, change).pull.is_some(), "the result opens its pull without a release");
        assert!(row(&world, change).change.last_head.is_some());
        assert!(!matches!(row(&world, change).change.state, State::Held { .. }));
        // The small profile's sign-in expires during the long simulated run.
        world.signed_in = None;
        world.sign_in();
        stop(&mut world, &goal);
    }
    assert_eq!(held, [false, false], "a live producer does not stall, with or without a dependency");
}
