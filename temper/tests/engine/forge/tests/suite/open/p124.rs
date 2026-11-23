//! The change's stall also bounds waiting for write room (engine-checks.md, 5.1).
use skein_lib::{Duration, Wall};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{LIMITS, World, changes};

#[test]
#[ignore = "bounded waits: P124, a wait for room has no bound"]
fn a_change_waiting_for_room_has_a_bound() {
    let mut world = World::with_entries(124, 1);
    world.adopt();
    for task in [51, 52] {
        let branch = format!("temper/{task}");
        world.produce(branch.as_bytes());
        let mut row = changes::change_row();
        row.task = task;
        row.change.task = task;
        row.branch = branch.into_bytes().into();
        row.change.state = change::State::Opening { requested: false };
        row.change.since = world.wall();
        world.event(top::Event::Change { row });
    }
    // The sole entry stays uncertain: its write is lost, and subsequent reconciliation
    // reads cannot reach the forge until well after the waiter's stall.
    world.lose_writes();
    changes::change_step_task(&mut world, 51, 100, change::Status::Passed, Box::new([]), false, false);
    changes::change_step_task(&mut world, 52, 101, change::Status::Passed, Box::new([]), false, false);
    assert!(world.change(52).room.is_some(), "second change must be waiting for write room");
    let since = world.wall();
    world.unavailable(since, Wall::from_nanos(since.as_nanos() + Duration::from_secs(300).as_nanos()));
    world.run_to(Wall::from_nanos(
        since.as_nanos() + LIMITS.change_policy.stall.as_nanos() + Duration::from_secs(1).as_nanos(),
    ));
    assert!(
        matches!(world.change(52).change.state, change::State::Held { .. }),
        "write-room wait exceeded the 30-second change stall without a hold: {:?}",
        world.change(52).change.state
    );
    let words = &world.change(52).effect_account.as_ref().expect("write-room hold has an account").words;
    super::words_name(words, &["room"]);
}
