//! Independent branch trains against the fake forge (temper/docs/design/domain/landing.md, 9, 15, 16).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World,
    landings::{self, Trains},
    translate,
};

fn branch(world: &mut World, name: &[u8]) {
    assert_eq!(
        world.external(raw::Op::Git(raw::Git::Create { branch: name.into(), commit: world.branch(b"main") }), 1),
        raw::Answer::Branch(raw::Created::Created)
    );
}

fn ready(world: &mut World, task: u64, destination: &[u8]) {
    let name = format!("temper/c{task}");
    world.produce_file(name.as_bytes(), name.as_bytes(), b"change");
    landings::ready_change_into(world, task, name.as_bytes(), destination);
}

fn until_turn(world: &mut World, task: u64) {
    for entry in task * 100..task * 100 + 80 {
        let decision = landings::step_landing_raw(world, task, entry);
        if matches!(decision, landing::Decision::Prepare { .. }) {
            world.prepare_landing(task, world.landing(task).landing.branch.as_ref() != b"main");
        }
        if matches!(world.landing(task).landing.phase, landing::Phase::Turn { .. }) {
            return;
        }
    }
    panic!("train did not reach its turn: {:?}", world.landing(task));
}

fn task(world: &World, change: u64) -> u64 {
    world.change(change).change.landing.expect("train formed")
}

#[test]
fn drill_13_two_goal_trains_and_main_progress_independently() {
    let mut world = landings::turn_in(World::parallel_trains());
    world.batch(4);
    branch(&mut world, b"temper/g1");
    branch(&mut world, b"temper/g2");
    for (change, base) in
        [(51, b"temper/g1".as_slice()), (52, b"temper/g1"), (53, b"temper/g2"), (54, b"temper/g2"), (55, b"main")]
    {
        ready(&mut world, change, base);
    }
    let mut trains = Trains::default();
    trains.judge(&mut world);
    let (first, second, main) = (task(&world, 51), task(&world, 53), task(&world, 55));
    assert_ne!(first, second);
    assert_ne!(first, main);
    assert_ne!(second, main);
    assert_eq!(world.change(52).change.landing, None);
    assert_eq!(world.change(54).change.landing, None);
    for (landing, change) in [(first, 51), (second, 53), (main, 55)] {
        assert_eq!(world.landing(landing).landing.members.iter().map(|m| m.change).collect::<Vec<_>>(), [change]);
        until_turn(&mut world, landing);
    }
    // Main takes its turn while both goal trains still own theirs.
    trains.land(&mut world, main);
    landings::close_landing(&mut world, main);
    assert_eq!(world.landing(main).landing.phase, landing::Phase::Landed);
    assert!(matches!(world.landing(first).landing.phase, landing::Phase::Turn { .. }));
    assert!(matches!(world.landing(second).landing.phase, landing::Phase::Turn { .. }));
    trains.land(&mut world, second);
    landings::close_landing(&mut world, second);
    let next_second = task(&world, 54);
    assert_ne!(next_second, second);
    assert_eq!(world.change(52).change.landing, None);
    until_turn(&mut world, next_second);
    trains.land(&mut world, next_second);
    landings::close_landing(&mut world, next_second);
    trains.land(&mut world, first);
    landings::close_landing(&mut world, first);
    let next_first = task(&world, 52);
    until_turn(&mut world, next_first);
    trains.land(&mut world, next_first);
    for change in 51..=55 {
        assert!(matches!(world.change(change).change.state, change::State::Landed { .. }));
    }
    assert!(trains.held.is_empty());
    let calls = world.take_calls();
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 1);
    assert_eq!(
        calls
            .iter()
            .filter(|op| matches!(op, api::Op::Write(api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. })))
            .count(),
        5
    );
    assert_ne!(translate::commit(world.branch(b"main")), world.change(51).change.last_head.expect("goal head"));
}
