//! Derived trains against the independent forge (domain/landing.md, 4.1, 9, 15).
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, CHANGE, LANDING, Trains},
    public,
};

fn world() -> World {
    let mut world = World::trains();
    world.adopt();
    assert!(world.project_landing(
        landings::PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: REPO,
            tiers: Box::new([]),
            gates: Box::new([landing::Gate {
                name: Box::from(b"gate".as_slice()),
                workflow: Box::from(b"gate.yml".as_slice()),
                ignored: Box::new([]),
                blocking: true,
            }]),
        }])
    ));
    world
}
fn prove(world: &mut World, task: u64) {
    for position in 0..80 {
        let entry = task * 100 + position * 2;
        landings::step_landing(world, task, entry);
        let member = world.landing(task).landing.members[0].change;
        changes::change_step_task(world, member, entry + 1, change::Status::Passed, Box::new([]), false, false);
        if matches!(world.landing(task).landing.phase, landing::Phase::Turn { .. }) {
            return;
        }
    }
    panic!("train never took its own turn: {:?}", world.landing(task));
}
#[test]
fn one_train_forms_one_landing_and_its_next_without_a_new_ready_event() {
    let mut world = world();
    let mut root = Trains::default();
    world.produce(b"temper/51");
    root.ready(&mut world, CHANGE, b"temper/51");
    world.produce(b"temper/52");
    root.ready(&mut world, 52, b"temper/52");
    assert_eq!(world.change(CHANGE).change.landing, Some(LANDING));
    assert_eq!(world.landing(LANDING).landing.members.len(), 1);
    assert_eq!(world.change(52).change.landing, None);
    root.land(&mut world, LANDING);
    assert_eq!(world.change(52).change.landing, Some(LANDING + 1));
    root.land(&mut world, LANDING + 1);
    assert_eq!(world.landing(LANDING + 1).landing.phase, landing::Phase::Landed);
}
#[test]
fn a_goals_branch_and_main_take_their_turns_side_by_side() {
    let mut world = world();
    let mut root = Trains::default();
    world.produce(b"temper/g1");
    landings::ready_on(&mut world, CHANGE, b"temper/51", b"temper/g1");
    landings::ready_on(&mut world, 52, b"temper/52", b"main");
    root.judge(&mut world);
    let goal = world.change(CHANGE).change.landing.expect("goal train");
    let main = world.change(52).change.landing.expect("main train");
    assert_ne!(goal, main);
    assert_eq!(world.landing(goal).landing.branch.as_ref(), b"temper/g1");
    prove(&mut world, goal);
    prove(&mut world, main);
    root.land(&mut world, goal);
    root.land(&mut world, main);
    assert_eq!(world.landing(goal).landing.phase, landing::Phase::Landed);
    assert_eq!(world.landing(main).landing.phase, landing::Phase::Landed);
}
#[test]
fn two_repositories_mains_take_their_turns_side_by_side() {
    let mut world = public::world(1000);
    let mut root = Trains::default();
    public::ready(&mut world, public::CHANGE, public::BRANCH);
    public::ready(&mut world, public::SECOND, public::OTHER_BRANCH);
    root.judge(&mut world);
    let first = world.change(public::CHANGE).change.landing.expect("first train");
    let second = world.change(public::SECOND).change.landing.expect("second train");
    assert_ne!(first, second);
    prove(&mut world, first);
    prove(&mut world, second);
    root.land(&mut world, first);
    root.land(&mut world, second);
    assert_eq!(world.landing(first).landing.phase, landing::Phase::Landed);
    assert_eq!(world.landing(second).landing.phase, landing::Phase::Landed);
}
#[test]
fn a_restart_forms_a_free_train_from_its_waiting_change() {
    let mut world = world();
    let mut root = Trains::default();
    landings::ready_on(&mut world, CHANGE, b"temper/51", b"main");
    assert_eq!(world.change(CHANGE).change.landing, None);
    root.restart(&mut world);
    assert_eq!(world.change(CHANGE).change.landing, Some(LANDING));
    root.restart(&mut world);
    assert_eq!(world.change(CHANGE).change.landing, Some(LANDING));
    assert_eq!(world.landing(LANDING).landing.branch.as_ref(), b"main");
    root.land(&mut world, LANDING);
}
#[test]
fn a_joined_unit_is_held_again_on_release_and_the_train_goes_on() {
    let mut world = public::world(1000);
    let mut root = Trains::default();
    public::ready(&mut world, public::CHANGE, public::BRANCH);
    public::ready(&mut world, public::SECOND, public::OTHER_BRANCH);
    root.dependencies.push(top::ChangeDependencies { change: public::CHANGE, pending: Box::new([public::SECOND]) });
    root.judge(&mut world);
    assert_eq!(world.change(public::CHANGE).change.landing, None);
    assert_eq!(world.change(public::SECOND).change.landing, None);
    let words = root.held.iter().find(|(task, _)| *task == public::CHANGE).expect("first change held").1.clone();
    let words = String::from_utf8(words.into_vec()).expect("hold text");
    assert!(words.starts_with("unit of 2 changes: C"));
    assert!(words.contains(&public::CHANGE.to_string()));
    assert!(words.contains(&public::SECOND.to_string()));
    assert!(words.ends_with("(temper/docs/design/draft/merge-queue.md, 13)"));
    root.release(&mut world, public::CHANGE);
    assert!(root.held.iter().any(|(task, _)| *task == public::CHANGE));
    assert_eq!(world.change(public::CHANGE).change.landing, None);
    landings::ready_on(&mut world, 53, b"temper/53", b"main");
    root.judge(&mut world);
    assert!(world.change(53).change.landing.is_some(), "refused unit does not block its train");
}
#[test]
fn a_dependency_given_after_forming_does_not_join_anything() {
    let mut world = public::world(1000);
    let mut root = Trains::default();
    public::ready(&mut world, public::CHANGE, public::BRANCH);
    root.judge(&mut world);
    let first = world.change(public::CHANGE).change.landing.expect("formed");
    let before = world.landing(first).clone();
    public::ready(&mut world, public::SECOND, public::OTHER_BRANCH);
    root.dependencies.push(top::ChangeDependencies { change: public::CHANGE, pending: Box::new([public::SECOND]) });
    root.judge(&mut world);
    assert_eq!(world.landing(first), &before);
    let second = world.change(public::SECOND).change.landing.expect("its own sealed unit");
    assert_ne!(second, first);
    assert!(root.held.is_empty());
}
#[test]
fn full_live_slots_wait_without_holding_and_form_when_a_slot_frees() {
    let mut world = world();
    let mut root = Trains::default();
    world.produce(b"temper/g1");
    world.produce(b"temper/g2");
    for (task, branch, base) in [
        (51, b"temper/51".as_slice(), b"main".as_slice()),
        (52, b"temper/52", b"temper/g1"),
        (53, b"temper/53", b"temper/g2"),
    ] {
        landings::ready_on(&mut world, task, branch, base);
        root.judge(&mut world);
    }
    assert_eq!(world.change(53).change.landing, None);
    assert!(root.held.is_empty());
    root.land(&mut world, LANDING);
    assert_eq!(world.change(53).change.landing, Some(LANDING + 2));
    assert!(root.held.is_empty());
}

#[test]
fn an_aged_low_priority_change_is_next_ahead_of_a_later_high_priority_change() {
    let mut world = world();
    let mut root = Trains::default();
    landings::ready_on(&mut world, CHANGE, b"temper/51", b"main");
    root.judge(&mut world);
    landings::ready_on(&mut world, 52, b"temper/52", b"main");
    world.event(top::Event::Priority { task: 52, priority: 9 });
    world.run_for(31);
    landings::ready_on(&mut world, 53, b"temper/53", b"main");
    world.event(top::Event::Priority { task: 53, priority: 0 });
    root.judge(&mut world);
    assert_eq!(world.change(52).change.landing, None);
    assert_eq!(world.change(53).change.landing, None);
    root.land(&mut world, LANDING);
    assert_eq!(world.change(52).change.landing, Some(LANDING + 1));
    assert_eq!(world.change(53).change.landing, None);
}
