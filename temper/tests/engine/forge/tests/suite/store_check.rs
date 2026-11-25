//! The forge's committed store is checked against the start's Link phase.
use jig_conformance::store_check::StoreChecks;
use temper_engine_domain_forge as top;
use temper_engine_forge_world::{World, changes};

fn save(record: top::Stored) -> top::Request {
    top::Request::Save { record, release: None, read_afresh: None }
}

fn every() -> World {
    let mut world = World::new(19);
    world.store_checks(StoreChecks::every());
    world.adopt();
    world
}

#[test]
fn a_lone_change_part_is_found_at_its_commit() {
    let mut world = every();
    let before = world.store_counts().0;
    let (cold, _) = changes::change_row().split();
    let finding = world.commit(Box::new([save(top::Stored::ChangeCold(Box::new(cold)))])).expect_err("lone part");
    assert_eq!(finding, format!("store check: commit {}: start refused at Link: Row(ChangeCold(51))", before + 1));
    assert_eq!(world.store_counts(), (before + 1, before + 1));
}

#[test]
fn a_new_change_commits_both_parts_together_and_checks_no_empty_batch() {
    let mut world = every();
    let before = world.store_counts().0;
    world.event(top::Event::Change { row: changes::change_row() });
    assert_eq!(world.store_counts(), (before + 1, before + 1));
    assert!(world.stored().contains_key(&top::Key::ChangeCold(51)));
    assert!(world.stored().contains_key(&top::Key::ChangeHot(51)));
    world.commit(Box::new([])).expect("empty batch");
    assert_eq!(world.store_counts(), (before + 1, before + 1));
}

fn subscriber() -> top::Subscriber {
    top::Subscriber {
        task: 91,
        number: 1,
        topic: top::Topic::Findings { project: 5 },
        own_change: None,
        goal_tasks: None,
        paths: Box::new([]),
    }
}

#[test]
fn a_record_cannot_supply_its_own_missing_task() {
    let mut world = every();
    let before = world.store_counts().0;
    let finding = world.commit(Box::new([save(top::Stored::Subscription(subscriber()))])).expect_err("absent task");
    assert!(
        finding.starts_with(&format!("store check: commit {}: start refused at Link: Link(", before + 1)),
        "{finding}"
    );
    assert!(finding.contains("Subscription { task: 91"), "{finding}");
    assert!(finding.contains("need: Task { task: 91, retired: None }"), "{finding}");
}

#[test]
fn a_scripted_task_makes_its_subscription_link_and_erasure_pass() {
    let mut world = every();
    let before = world.store_counts().0;
    world.event(top::Event::Subscribe { subscription: subscriber() });
    assert_eq!(world.store_counts(), (before + 1, before + 1));
    world
        .commit(Box::new([top::Request::Erase { key: top::Key::Subscription { task: 91, topic: subscriber().topic } }]))
        .expect("erase commits");
    assert_eq!(world.store_counts(), (before + 2, before + 2));
}

#[test]
fn an_erased_scripted_task_no_longer_supplies_a_link() {
    let mut world = every();
    world.event(top::Event::Subscribe { subscription: subscriber() });
    world.event(top::Event::Unsubscribe { task: 91, topic: subscriber().topic });
    world.event(top::Event::Unnamed { task: 91 });
    let finding = world.commit(Box::new([save(top::Stored::Subscription(subscriber()))])).expect_err("erased task");
    assert!(finding.contains("need: Task { task: 91, retired: None }"), "{finding}");
}

#[test]
fn the_kept_batch_refuses_a_missing_task_even_if_it_is_handed_over_later() {
    // Seed 7 keeps the second of these three batches.
    let mut world = World::new(7);
    world.commit(Box::new([save(binding(1))])).expect("valid binding");
    world.commit(Box::new([save(top::Stored::Subscription(subscriber()))])).expect("check deferred until end");
    world.hold_task(91);
    world
        .commit(Box::new([top::Request::Erase { key: top::Key::Subscription { task: 91, topic: subscriber().topic } }]))
        .expect("erase commits");
    assert_eq!(world.store_counts(), (3, 0));
    let finding = world.finish().expect_err("task was absent at the kept batch");
    assert!(finding.starts_with("store check: commit 2: start refused at Link: Link("), "{finding}");
    assert!(finding.contains("Subscription { task: 91"), "{finding}");
    assert!(finding.contains("need: Task { task: 91, retired: None }"), "{finding}");
    assert_eq!(world.store_counts(), (3, 1));
}

#[test]
fn the_kept_batch_links_a_task_that_is_forgotten_later() {
    let mut world = World::new(19);
    world.hold_task(91);
    world.commit(Box::new([save(top::Stored::Subscription(subscriber()))])).expect("valid subscription");
    world.event(top::Event::Unnamed { task: 91 });
    assert_eq!(world.store_counts(), (1, 0));
    world.finish().expect("task was held at the kept batch");
    assert_eq!(world.store_counts(), (1, 1));
}

const BINDINGS: u64 = 128;

fn binding(number: u64) -> top::Stored {
    top::Stored::Binding { address: Box::new([1]), subject: Box::from(number.to_be_bytes()) }
}

fn bindings(world: &mut World) {
    for number in 1..=BINDINGS {
        let row = binding(number);
        world.commit(Box::new([save(row.clone())])).expect("valid binding");
        assert_eq!(world.stored().get(&top::Key::Binding(Box::new([1]))), Some(&row));
    }
}

fn sampled(seed: u64) -> u64 {
    let mut world = World::new(seed);
    let mut control = World::new(seed);
    control.store_checks(StoreChecks::none("unchecked control for the binding story"));
    bindings(&mut control);
    bindings(&mut world);
    assert_eq!(world.store_counts(), (BINDINGS, 0));
    world.finish().expect("valid kept binding");
    assert_eq!(world.store_counts(), (BINDINGS, 1));
    let report = world.store_report();
    let prefix = format!("store check: 1 of {BINDINGS} commits checked (one per run, seed {seed}, commit ");
    let commit = report.strip_prefix(&prefix).and_then(|suffix| suffix.strip_suffix(')')).expect(&report);
    let commit = commit.parse().expect("kept batch number");
    assert!((1..=BINDINGS).contains(&commit), "{report}");
    assert_eq!((world.calls(), world.writes(), world.wall()), (control.calls(), control.writes(), control.wall()));
    assert_eq!(world.seen(), control.seen(), "the check's outputs stay in its copy");
    commit
}

#[test]
fn the_default_sample_reports_and_replays_without_touching_the_world() {
    let first = sampled(4);
    assert_eq!(first, sampled(4));
    assert_ne!(first, sampled(26));
}
