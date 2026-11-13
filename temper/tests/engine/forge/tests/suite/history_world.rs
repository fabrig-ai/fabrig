//! Reading the world's indexed history (domain/landing-history.md, 2.1 and 4.1).
use temper_engine_domain_forge::{self as top, history};
use temper_engine_forge_world::{REPO, World, changes};

const A: u64 = 51;
const B: u64 = 52;
const L: u64 = 71;

fn world() -> World {
    let mut world = World::new(2570);
    world.adopt();
    for task in [A, B] {
        let mut row = changes::change_row();
        row.task = task;
        row.change.task = task;
        world.event(top::Event::Change { row });
    }
    world
}

fn step(landing: Option<u64>, what: history::What) -> history::Step {
    history::Step {
        what,
        landing: landing.map(|task| history::InLanding { task, round: 2, unit: 1 }),
        head: None,
        base: None,
        operation: history::Operation::Own,
        attempt: 0,
        source_at: None,
    }
}

fn saved(world: &World, project: u32, number: u64) -> history::Event {
    let Some(top::Stored::HistoryEvent(event)) = world.stored().get(&top::Key::HistoryEvent { project, number }) else {
        panic!("written event exists")
    };
    event.clone()
}

#[test]
fn reads_written_events_through_change_and_landing_indexes() {
    let mut world = world();
    assert_eq!(world.history(A), []);
    assert_eq!(world.landing_history(L), []);
    assert_eq!(world.write_event(A, step(None, history::What::Ready)), Some(1));
    assert_eq!(world.write_event(A, step(Some(L), history::What::Selected)), Some(2));
    assert_eq!(world.write_event(B, step(None, history::What::Pushed)), Some(3));
    assert_eq!(world.write_event(A, step(None, history::What::Landed)), Some(4));

    assert_eq!(world.history(A), [saved(&world, 5, 1), saved(&world, 5, 2), saved(&world, 5, 4)]);
    assert_eq!(world.history(B), [saved(&world, 5, 3)]);
    assert_eq!(world.landing_history(L), [saved(&world, 5, 2)]);
    assert_eq!(world.history(99), []);
    assert_eq!(world.landing_history(99), []);
}

#[test]
fn skips_erased_events_without_losing_surviving_indexed_events() {
    let mut world = world();
    assert_eq!(world.write_event(A, step(Some(L), history::What::Selected)), Some(1));
    assert_eq!(world.write_event(A, step(Some(L), history::What::Landed)), Some(2));
    assert_eq!(world.write_event(B, step(Some(72), history::What::Selected)), Some(3));
    world.erase_history_event(5, 1);
    assert!(world.stored().contains_key(&top::Key::HistoryChange { project: 5, change: A, number: 1 }));
    assert!(world.stored().contains_key(&top::Key::HistoryLanding { project: 5, landing: L, number: 1 }));
    assert_eq!(world.history(A), [saved(&world, 5, 2)]);
    assert_eq!(world.landing_history(L), [saved(&world, 5, 2)]);
    world.erase_history_event(5, 2);
    assert_eq!(world.history(A), []);
    assert_eq!(world.landing_history(L), []);
}

fn project(world: &mut World, project: u32) {
    let Some(top::Stored::Repository(row)) = world.stored().get(&top::Key::Repository(REPO)) else {
        panic!("adopted repository")
    };
    let mut row = row.clone();
    row.project = project;
    world.event(top::Event::Restore { record: top::Stored::Repository(row) });
    // Each project's fixture starts with no last-event pointer into another project.
    for task in [A, B] {
        let mut row = world.change(task).clone();
        row.history = None;
        world.event(top::Event::Restore { record: top::Stored::Change(Box::new(row)) });
    }
}

#[test]
fn orders_events_by_number_across_projects() {
    let mut world = world();
    assert_eq!(world.write_event(A, step(Some(L), history::What::Ready)), Some(1));
    assert_eq!(world.write_event(B, step(None, history::What::Ready)), Some(2));
    project(&mut world, 6);
    assert_eq!(world.write_event(B, step(None, history::What::Pushed)), Some(1));
    assert_eq!(world.write_event(A, step(Some(L), history::What::Selected)), Some(2));
    project(&mut world, 5);
    assert_eq!(world.write_event(A, step(Some(L), history::What::Landed)), Some(3));

    let ordered = [saved(&world, 5, 1), saved(&world, 6, 2), saved(&world, 5, 3)];
    assert_eq!(world.history(A), ordered);
    assert_eq!(world.landing_history(L), ordered);
}
