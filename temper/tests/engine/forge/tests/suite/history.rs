//! History reads through the scripted root's durable rows (landing-history.md, 5.1).
use skein_lib::Wall;
use temper_engine_domain_forge::{
    self as forge,
    history::{Cursor, HOUR_NS, HistoryRange, Of, Query, Reply},
};
use temper_engine_forge_world::{World, changes, landings};

fn world() -> World {
    let mut world = World::new(902);
    world.adopt();
    for task in [51, 52] {
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
fn query(of: Of) -> Query {
    Query { project: 5, of, since: None, until: None, after: None }
}
fn wall(hour: u64) -> Wall {
    Wall::from_nanos(hour * HOUR_NS)
}
fn page(reply: Reply) -> (Vec<u64>, Option<Cursor>, u64) {
    let Reply::Page { events, after, upper, .. } = reply else { panic!("history page, got {reply:?}") };
    (events.iter().map(|event| event.number).collect(), after, upper)
}

#[test]
fn project_change_and_landing_read_written_events_through_their_own_ranges() {
    let mut world = world();
    world.history_rows(1);
    world.write_history(51, wall(10), Some(61));
    world.write_history(52, wall(10), Some(62));
    world.write_history(51, wall(11), Some(62));
    assert_eq!(page(world.history_reply(1, query(Of::Project))), (vec![1, 2, 3], None, 3));
    let before = world.history_loads().len();
    assert_eq!(page(world.history_reply(2, query(Of::Change(51)))), (vec![1, 3], None, 3));
    assert!(
        world.history_loads()[before..].iter().all(|(range, _, _)| matches!(
            range,
            HistoryRange::Change { change: 51, .. } | HistoryRange::Numbers { .. }
        ))
    );
    assert!(world.history_loads()[before..].iter().any(|(range, _, _)| matches!(range, HistoryRange::Numbers { .. })));
    assert_eq!(page(world.history_reply(3, query(Of::Landing(62)))), (vec![2, 3], None, 3));
    assert!(
        world.history_loads().iter().any(|(range, _, _)| matches!(range, HistoryRange::Landing { landing: 62, .. }))
    );
}
#[test]
fn forty_events_page_as_thirty_two_then_eight_at_the_first_upper_bound() {
    let mut world = world();
    for _ in 0..40 {
        world.write_history(51, wall(10), Some(61));
    }
    let (numbers, after, upper) = page(world.history_reply(1, query(Of::Project)));
    assert_eq!(numbers, (1..=32).collect::<Vec<_>>());
    assert_eq!(upper, 40);
    assert_eq!(after, Some(Cursor { upper: 40, after: 32 }));
    world.write_history(51, wall(11), Some(61));
    let mut query = query(Of::Project);
    query.after = after;
    assert_eq!(page(world.history_reply(2, query)), ((33..=40).collect(), None, 40));
}
#[test]
fn time_filter_is_half_open_and_project_starts_at_the_since_hour_mark() {
    let mut world = world();
    world.write_history(51, wall(10), None);
    world.write_history(51, wall(11), None);
    world.write_history(51, wall(12), None);
    world.write_history(51, Wall::from_nanos(wall(12).as_nanos() + 5), None);
    world.write_history(51, wall(13), None);
    let mut q = query(Of::Project);
    q.since = Some(Wall::from_nanos(wall(12).as_nanos() + 1));
    q.until = Some(wall(13));
    assert_eq!(page(world.history_reply(1, q)), (vec![4], None, 5));
    assert_eq!(world.history_loads().first().unwrap().1, Some(forge::Key::HistoryEvent { project: 5, number: 2 }));
    q.of = Of::Change(51);
    assert_eq!(page(world.history_reply(2, q)), (vec![4], None, 5));
}
#[test]
fn a_page_cuts_after_two_hundred_fifty_six_examined_rows() {
    let mut world = world();
    for _ in 0..270 {
        world.write_history(51, wall(10), None);
    }
    let mut q = query(Of::Project);
    q.since = Some(wall(11));
    let (events, after, upper) = page(world.history_reply(1, q));
    assert_eq!(events, Vec::<u64>::new());
    assert_eq!(upper, 270);
    assert_eq!(after, Some(Cursor { upper: 270, after: 256 }));
    assert_eq!(world.history_loads().iter().map(|(_, _, most)| most).sum::<u32>(), 256);
    q.after = after;
    assert_eq!(page(world.history_reply(2, q)), (vec![], None, 270));
    // Both the index and the Numbers rows count toward the same bound.
    q.after = None;
    q.of = Of::Change(51);
    assert_eq!(page(world.history_reply(3, q)).1, Some(Cursor { upper: 270, after: 128 }));
}
#[test]
fn a_gone_cursor_names_the_first_survivor_or_none() {
    let mut world = world();
    for _ in 0..4 {
        world.write_history(51, wall(10), Some(61));
    }
    for number in [2, 3] {
        world.erase_history_row(&forge::Key::HistoryEvent { project: 5, number });
        world.erase_history_row(&forge::Key::HistoryChange { project: 5, change: 51, number });
        world.erase_history_row(&forge::Key::HistoryLanding { project: 5, landing: 61, number });
    }
    for (read, of) in [(1, Of::Project), (2, Of::Change(51)), (3, Of::Landing(61))] {
        let mut q = query(of);
        q.after = Some(Cursor { upper: 4, after: 2 });
        assert_eq!(world.history_reply(read, q), Reply::Gone { from: 2, first: Some(4) });
    }
    world.erase_history_row(&forge::Key::HistoryEvent { project: 5, number: 4 });
    let mut q = query(Of::Project);
    q.after = Some(Cursor { upper: 4, after: 4 });
    assert_eq!(world.history_reply(4, q), Reply::Gone { from: 4, first: None });
}
#[test]
fn evicted_change_and_erased_landing_keep_their_history() {
    let mut world = world();
    world.write_history(51, wall(10), Some(61));
    world.evict_history_change(51, 2);
    assert!(!world.stored().contains_key(&forge::Key::Change(51)));
    let mut row = changes::change_row();
    row.pull = Some(1);
    world.keep_history_landing(landings::landing_row(&row));
    world.erase_history_row(&forge::Key::Landing(61));
    world.restart();
    assert_eq!(page(world.history_reply(1, query(Of::Change(51)))), (vec![1], None, 1));
    assert_eq!(page(world.history_reply(2, query(Of::Landing(61)))), (vec![1], None, 1));
}
#[test]
fn sweeping_change_is_empty_project_skips_it_and_its_cursor_is_gone() {
    let mut world = world();
    world.write_history(51, wall(10), Some(61));
    world.write_history(52, wall(10), Some(61));
    world.sweeping_history(51);
    assert_eq!(page(world.history_reply(1, query(Of::Change(51)))), (vec![], None, 2));
    let project = world.history_reply(2, query(Of::Project));
    let Reply::Page { whole_since, .. } = &project else { panic!("project history page") };
    assert_eq!(*whole_since, Some(Wall::from_nanos(1)));
    assert_eq!(page(project), (vec![2], None, 2));
    assert_eq!(page(world.history_reply(3, query(Of::Landing(61)))), (vec![2], None, 2));
    let mut q = query(Of::Project);
    q.after = Some(Cursor { upper: 2, after: 1 });
    assert_eq!(world.history_reply(4, q), Reply::Gone { from: 1, first: Some(2) });
}
#[test]
fn eight_reads_are_admitted_failed_loads_answer_busy_and_late_answers_are_dropped() {
    let mut world = world();
    world.write_history(51, wall(10), None);
    world.hold_history();
    for read in 1..=8 {
        world.event(forge::Event::ReadHistory { read, query: query(Of::Project) });
    }
    assert_eq!(world.history_reply(9, query(Of::Project)), Reply::Busy);
    world.event(forge::Event::HistoryUnloaded { load: 1 });
    assert_eq!(world.finish_history(1), Reply::Busy);
    world.event(forge::Event::HistoryLoaded { load: 1, rows: Box::new([]), next: None });
    assert!(world.take_seen().is_empty(), "no second terminal for retired load");
    // Drop the other reads through their load failures, then deliver all the late successes.
    for load in 2..=8 {
        world.event(forge::Event::HistoryUnloaded { load });
        assert_eq!(world.finish_history(load), Reply::Busy);
    }
    world.release_history();
    for _ in 0..8 {
        assert!(world.step_until(world.now()));
    }
    assert!(world.take_seen().is_empty());
    assert_eq!(page(world.history_reply(10, query(Of::Project))).0, vec![1]);
}
