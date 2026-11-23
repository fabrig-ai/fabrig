//! A permanent refusal in one repository leaves another goal's work alone.
use super::*;
use temper_engine_domain_world::ends::{Bound, Expect};

const BOUND: Bound = Bound { minutes: 10, steps: 4000 };

fn goal(world: &mut World, key: u8) -> engine::Assignment {
    let people::Ask::Create { project, mut batch } = adoption_chat(world) else { panic!("goal batch") };
    batch[0].flags = people::Flags { listed: true, ..people::Flags::PLAIN };
    batch[0].label = 2;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
    batch[0].contract = people::Contract::Report { words: 128 };
    assigned_chat(world, key, people::Ask::Create { project, batch })
}

#[test]
fn a_goal_in_another_repository_ends_untouched_by_a_permanent_refusal() {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, across: true, slots: Some(4), ..StoryLimits::default() },
    );
    world.adopt();
    adopt_library(&mut world);
    world.send(hello_prepare(4));
    let struck = goal(&mut world, 92);
    let (a, producer_a) = make_change(&mut world, &struck, 95, b"org/repo");
    // The same open refusal as the held story, scoped to the first repository.
    world.fail_open_writes = true;
    world.fail_open_in = Some(forge_world::REPO);
    world.ends.faults_stop(world.env.now);
    world.ends.expect(a, Expect::Held("permission"), BOUND).expect("nonzero bound");
    push_change(&mut world, &producer_a, b"org/repo");
    finish_producer(&mut world, &producer_a, forge_world::REPO);
    for _ in 0..4000 {
        world.prepare_tick();
        if matches!(live_task(&world, a).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(matches!(live_task(&world, a).phase, tasks::Phase::Held { .. }));
    let untouched = goal(&mut world, 93);
    world.ends.expect(untouched.task, Expect::Untouched, BOUND).expect("nonzero bound");
    let (b, producer_b) = make_change(&mut world, &untouched, 96, b"org/lib");
    push_change(&mut world, &producer_b, b"org/lib");
    finish_producer(&mut world, &producer_b, LIB);
    for _ in 0..4000 {
        world.prepare_tick();
        if matches!(live_task(&world, a).phase, tasks::Phase::Held { .. }) && goal_landing::ended(&world, b).is_some() {
            break;
        }
    }
    assert!(matches!(live_task(&world, a).phase, tasks::Phase::Held { .. }));
    assert!(goal_landing::ended(&world, b).is_some(), "other repository's change landed");
    goal_landing::finish_run(
        &mut world,
        &untouched,
        tasks::TaskResult::Report { words: b"The library change landed.".as_slice().into() },
    );
    goal_closer::wait_ended(&mut world, untouched.task);
    world.ends.close(world.env.now, &world.store).expect("other repository's goal ended");
    assert!(world.fail_open_writes && world.fail_open_in == Some(forge_world::REPO));
    world.close();
}
