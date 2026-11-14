//! Whole-change retention through stored rows and the forge's wall deadline (landing-history.md, 4.5).
use skein_lib::{Token, Wall};
use temper_engine_domain_forge::{
    self as forge,
    history::{HOUR_NS, HistoryRange, Project},
};
use temper_engine_forge_world::{World, changes};

fn day(day: u64) -> Wall {
    Wall::from_nanos(day * 24 * HOUR_NS)
}
fn world(events: u64) -> World {
    let mut world = World::new(907);
    world.history_limits(14, events);
    world.adopt();
    for task in [51, 52, 53] {
        let mut row = changes::change_row();
        row.task = task;
        row.change.task = task;
        row.change.state = temper_engine_domain_forge_change::State::Ended;
        row.ending = Some(forge::Ending::NoChange);
        world.event(forge::Event::Change { row });
    }
    world.take_seen();
    world
}
fn project(world: &World) -> &Project {
    let Some(forge::Stored::HistoryProject(row)) = world.stored().get(&forge::Key::HistoryProject { project: 5 })
    else {
        panic!("project history")
    };
    row
}
fn numbers(world: &World, change: u64) -> Vec<u64> {
    world.history(change).iter().map(|event| event.number).collect()
}
fn gone(world: &World, change: u64, erased: &[u64]) {
    assert_eq!(numbers(world, change), []);
    for number in erased {
        assert!(!world.stored().contains_key(&forge::Key::HistoryEvent { project: 5, number: *number }));
        assert!(!world.stored().contains_key(&forge::Key::HistoryChange { project: 5, change, number: *number }));
        assert!(!world.stored().contains_key(&forge::Key::HistoryLanding { project: 5, landing: 61, number: *number }));
        assert!(!world.stored().contains_key(&forge::Key::HistoryLast { project: 5, change, number: *number }));
    }
}

#[test]
fn hourly_age_sweep_takes_old_changes_whole_and_leaves_recent_history() {
    let mut world = world(131_072);
    for change in [51, 51, 52, 52] {
        world.write_history(change, if change == 51 { day(0) } else { day(10) }, Some(61));
    }
    world.history_hour(day(14));
    world.run_for(0);
    assert_eq!(numbers(&world, 51), [1, 2], "exactly fourteen days is not older");
    assert!(world.history_deadline().is_some_and(|due| due <= day(15)));
    world.history_hour(day(15));
    world.run_for(0);
    gone(&world, 51, &[1, 2]);
    assert_eq!(numbers(&world, 52), [3, 4]);
    assert_eq!(world.landing_history(61).iter().map(|event| event.number).collect::<Vec<_>>(), [3, 4]);
    assert_eq!((project(&world).kept, project(&world).whole_since), (2, Some(day(0))));
    assert!(world.stored().contains_key(&forge::Key::HistoryLast { project: 5, change: 52, number: 4 }));
}

#[test]
fn count_sweep_orders_by_last_event_not_first_and_never_splits_a_change() {
    for interleaved in [false, true] {
        let mut world = world(10);
        let order = if interleaved {
            vec![51, 52, 52, 52, 52, 51, 51, 51, 53, 53, 53, 53]
        } else {
            vec![51, 51, 51, 51, 52, 52, 52, 52, 53, 53, 53, 53]
        };
        for change in order {
            world.write_history(change, day(0), Some(61));
        }
        world.run_for(0);
        assert_eq!(project(&world).kept, 8);
        if interleaved {
            gone(&world, 52, &[2, 3, 4, 5]);
            assert_eq!(numbers(&world, 51), [1, 6, 7, 8]);
        } else {
            gone(&world, 51, &[1, 2, 3, 4]);
            assert_eq!(numbers(&world, 52), [5, 6, 7, 8]);
        }
        assert_eq!(numbers(&world, 53), [9, 10, 11, 12]);
    }
}

fn forty() -> World {
    let mut world = world(40);
    world.history_rows(64);
    for _ in 0..40 {
        world.write_history(51, day(0), Some(61));
    }
    world.hold_history();
    world.write_history(52, day(0), None);
    world.answer_history(); // Last: A's snapshot ends at 40.
    world.answer_history(); // Change: its first 32, leaving the next page in flight.
    world
}
#[test]
fn sweep_pages_erase_at_most_thirty_two_and_keep_a_concurrent_new_event() {
    let mut world = forty();
    assert_eq!((project(&world).kept, project(&world).sweeping), (9, Some(51)));
    assert_eq!(numbers(&world, 51), (33..=40).collect::<Vec<_>>());
    assert_eq!(world.history_loads().last().unwrap().2, 32);
    world.write_history(51, day(1), Some(61));
    world.answer_history();
    assert_eq!((project(&world).kept, project(&world).sweeping), (2, None));
    assert_eq!(numbers(&world, 51), [42]);
    assert_eq!(numbers(&world, 52), [41]);
    assert!(world.stored().contains_key(&forge::Key::HistoryLast { project: 5, change: 51, number: 42 }));
    assert!(!world.stored().contains_key(&forge::Key::HistoryLast { project: 5, change: 51, number: 40 }));
    world.release_history();
    world.run_for(0);
    assert_eq!(numbers(&world, 51), [42]);

    // A new row visible to the next Change load is still past the sweep's loaded last number.
    let mut world = self::world(40);
    for _ in 0..40 {
        world.write_history(51, day(0), Some(61));
    }
    world.hold_history();
    world.write_history(52, day(0), None);
    world.answer_history();
    world.write_history(51, day(1), Some(61));
    world.answer_history();
    world.answer_history();
    assert_eq!((project(&world).kept, project(&world).sweeping), (2, None));
    assert_eq!(numbers(&world, 51), [42]);
    world.release_history();
    world.run_for(0);
}

#[test]
fn restart_drops_the_page_in_flight_and_finishes_the_partial_change_on_the_hour() {
    let mut world = forty();
    assert_eq!(project(&world).kept, 9);
    world.restart();
    assert_eq!(project(&world).sweeping, Some(51));
    world.release_history(); // Answers from the old generation cannot erase the same page twice.
    world.history_hour(day(1));
    world.run_for(0);
    gone(&world, 51, &(1..=40).collect::<Vec<_>>());
    assert_eq!(numbers(&world, 52), [41]);
    assert_eq!((project(&world).kept, project(&world).sweeping), (1, None));
    world.history_hour(day(2));
    world.run_for(0);
    assert_eq!(project(&world).kept, 1);
}

#[test]
fn project_release_sweeps_every_change_and_survives_a_restart_midway() {
    let mut world = world(131_072);
    for _ in 0..40 {
        world.write_history(51, day(0), Some(61));
    }
    world.write_history(52, day(0), Some(61));
    world.hold_history();
    world.event(forge::Event::ReleaseProject { project: 5, request: Token::new(100) });
    assert!(project(&world).removing);
    world.answer_history();
    world.answer_history();
    assert_eq!((project(&world).kept, project(&world).sweeping), (9, Some(51)));
    world.restart();
    assert!(project(&world).removing);
    world.release_history();
    world.history_hour(day(1));
    world.run_for(0);
    gone(&world, 51, &(1..=40).collect::<Vec<_>>());
    gone(&world, 52, &[41]);
    assert_eq!(world.landing_history(61), []);
    assert!(!world.stored().contains_key(&forge::Key::HistoryProject { project: 5 }));
}

#[test]
fn sweep_keeps_the_last_mark_before_whole_since_hour_and_every_later_mark() {
    let mut world = world(131_072);
    for hour in [1, 2, 3, 4] {
        world.write_history(51, Wall::from_nanos(hour * HOUR_NS), Some(61));
    }
    world.write_history(52, day(10), Some(61));
    world.history_hour(day(15));
    world.run_for(0);
    assert_eq!(project(&world).whole_since, Some(Wall::from_nanos(4 * HOUR_NS)));
    assert_eq!(project(&world).marks.iter().map(|mark| mark.first).collect::<Vec<_>>(), [3, 4, 5]);
}

#[test]
fn refused_sweep_load_retries_on_the_next_write_or_hour_without_parallel_sweeps() {
    let mut world = world(1);
    world.write_history(51, day(0), None);
    world.hold_history();
    world.write_history(52, day(0), None);
    let loads = world.history_loads().len();
    world.write_history(53, day(0), None);
    assert_eq!(world.history_loads().len(), loads, "one sweep in flight per project");
    world.refuse_history();
    assert_eq!(project(&world).kept, 3);
    world.history_hour(day(1));
    world.release_history();
    world.run_for(0);
    assert_eq!(project(&world).kept, 1);
    assert_eq!(numbers(&world, 53), [3]);
    assert!(
        world.history_loads().iter().any(|(range, _, most)| matches!(range, HistoryRange::Last { .. }) && *most == 1)
    );
}
