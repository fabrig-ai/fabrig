//! A live delegate's wait and the result it delivers (temper/docs/design/domain/forge.md, 8.4).
use super::early_start::{goal_world, make, observe_push, push};
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
        let made_at = world.env.wall;
        let change = make(&mut world, &goal, b"Slow producer", &after.as_ref().map(|(task, _)| *task).into_iter().collect::<Vec<_>>());
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
        for _ in 0..40 { world.tick(); }
        let requested = row(&world, change).change.since;
        let stall = world.env.limits.forge.change_policy.stall;
        let assigned = world.assigned.len();
        let messages = live_task(&world, goal.task).inbox.len();
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
        eprintln!("dependency={dependency} made={made_at:?} requested={requested:?} observed={:?} since={:?} state={before:?} head={:?} phase={:?} escalation={:?} goal_assignments={} goal_messages={}", world.env.wall, row(&world, change).change.since, row(&world, change).change.last_head, task_state(&world, change), (), world.assigned[assigned..].iter().filter(|run| run.task == goal.task).count(), live_task(&world, goal.task).inbox.len() - messages);
        finish_producing(&mut world, change, &producer);
        for _ in 0..100 {
            world.tick();
        }
        eprintln!("after result: state={:?} head={:?} phase={:?} escalation={:?} goal_assignments={} goal_messages={}", row(&world, change).change.state, row(&world, change).change.last_head, task_state(&world, change), (), world.assigned[assigned..].iter().filter(|run| run.task == goal.task).count(), live_task(&world, goal.task).inbox.len() - messages);
        held.push(matches!(before, State::Held { .. }));
        world.close();
    }
    assert_eq!(held, [false, false], "a live producer does not stall, with or without a dependency");
}

fn task_state(world: &World, task: u64) -> (&tasks::Phase, &tasks::Escalation) {
    let record = world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))
        .or_else(|| world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))));
    match record {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) => (&row.phase, &row.escalation),
        _ => panic!("change task"),
    }
}
