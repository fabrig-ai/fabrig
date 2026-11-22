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

fn sampled(seed: u64) -> Vec<(u64, u64)> {
    let mut world = World::new(seed);
    world.take_seen();
    let calls = world.calls();
    let writes = world.writes();
    let wall = world.wall();
    let mut counts = Vec::new();
    for number in 1..=128_u64 {
        let row = top::Stored::Binding { address: Box::new([1]), subject: Box::from(number.to_be_bytes()) };
        world.commit(Box::new([save(row.clone())])).expect("valid binding");
        assert_eq!(world.stored().get(&top::Key::Binding(Box::new([1]))), Some(&row));
        counts.push(world.store_counts());
    }
    let (commits, checked) = world.store_counts();
    assert_eq!(commits, 128);
    assert!(checked > 0 && checked < commits);
    assert_eq!(
        world.store_report(),
        format!("store check: {checked} of 128 commits checked (one in 1024, seed {seed})")
    );
    assert_eq!((world.calls(), world.writes(), world.wall()), (calls, writes, wall));
    assert!(world.seen().is_empty(), "the check's outputs stay in its copy");
    counts
}

#[test]
fn the_default_sample_reports_and_replays_without_touching_the_world() {
    // These seeds select commits 87 and 63 in the default one-in-1024 sample.
    let first = sampled(4);
    assert_eq!(first, sampled(4));
    assert_ne!(first, sampled(26));
}
