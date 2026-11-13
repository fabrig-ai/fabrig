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
                base: false,
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
    let mut world = batched();
    let mut root = Trains::default();
    waiting(&mut world, 3, b"main");
    assert_eq!(world.change(CHANGE).change.landing, None);
    root.restart(&mut world);
    assert_eq!(world.change(CHANGE).change.landing, Some(LANDING));
    root.restart(&mut world);
    assert_eq!(world.change(CHANGE).change.landing, Some(LANDING));
    assert_eq!(world.landing(LANDING).landing.branch.as_ref(), b"main");
    assert_eq!(members(&world, LANDING), [CHANGE, CHANGE + 1, CHANGE + 2]);
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
    assert_eq!(root.held, Vec::<(u64, Box<[u8]>)>::new());
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
    assert_eq!(root.held, Vec::<(u64, Box<[u8]>)>::new());
    root.land(&mut world, LANDING);
    assert_eq!(world.change(53).change.landing, Some(LANDING + 2));
    assert_eq!(root.held, Vec::<(u64, Box<[u8]>)>::new());
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

fn batched() -> World {
    let mut world = world();
    world.batch(3);
    world
}
fn waiting(world: &mut World, count: u64, base: &[u8]) {
    for index in 0..count {
        let task = CHANGE + index;
        landings::ready_on(world, task, format!("temper/{task}").as_bytes(), base);
    }
}
fn members(world: &World, task: u64) -> Vec<u64> {
    world.landing(task).landing.members.iter().map(|member| member.change).collect()
}
#[test]
fn a_batch_takes_three_in_queue_order_and_the_next_takes_the_rest() {
    let mut world = batched();
    let mut root = Trains::default();
    waiting(&mut world, 5, b"main");
    // Queue order is priority, not the changes table's task order.
    world.event(top::Event::Priority { task: CHANGE + 2, priority: 0 });
    world.event(top::Event::Priority { task: CHANGE, priority: 1 });
    world.event(top::Event::Priority { task: CHANGE + 1, priority: 2 });
    world.event(top::Event::Priority { task: CHANGE + 3, priority: 3 });
    world.event(top::Event::Priority { task: CHANGE + 4, priority: 4 });
    root.judge(&mut world);
    assert_eq!(members(&world, LANDING), [CHANGE + 2, CHANGE, CHANGE + 1]);
    for (index, member) in world.landing(LANDING).landing.members.iter().enumerate() {
        assert_eq!(member.unit, u16::try_from(index).expect("member count"));
        let branch = if index == 0 { format!("temper/{LANDING}/l{LANDING}") } else { format!("temper/{LANDING}/l{LANDING}-{}", index + 1) };
        assert_eq!(member.branch.as_ref(), branch.as_bytes());
        let name = top::Name { forge: REPO.forge, repository: REPO.repository, what: top::What::Branch(branch.split('/').map(|part| part.as_bytes().into()).collect()) };
        assert_eq!(world.hold(&name).expect("held from making").task, LANDING);
    }
    assert_eq!(world.change(CHANGE + 3).change.landing, None);
    assert_eq!(world.change(CHANGE + 4).change.landing, None);
    // Forming observes a terminal landing; running its batch is a separate rule.
    world.ended_landing_fixture(LANDING);
    root.judge(&mut world);
    assert_eq!(members(&world, LANDING + 1), [CHANGE + 3, CHANGE + 4]);
}
#[test]
fn a_goal_train_takes_one_unit_even_with_batch_three() {
    let mut world = batched();
    world.produce(b"temper/g1");
    let mut root = Trains::default();
    waiting(&mut world, 3, b"temper/g1");
    root.judge(&mut world);
    assert_eq!(members(&world, LANDING), [CHANGE]);
    assert_eq!(world.change(CHANGE + 1).change.landing, None);
}
#[test]
fn a_formed_batch_is_sealed_against_a_later_ready_change() {
    let mut world = batched();
    let mut root = Trains::default();
    waiting(&mut world, 2, b"main");
    root.judge(&mut world);
    landings::ready_on(&mut world, CHANGE + 2, b"temper/later", b"main");
    root.judge(&mut world);
    assert_eq!(members(&world, LANDING), [CHANGE, CHANGE + 1]);
    assert_eq!(world.change(CHANGE + 2).change.landing, None);
    world.ended_landing_fixture(LANDING);
    root.judge(&mut world);
    assert_eq!(members(&world, LANDING + 1), [CHANGE + 2]);
}
#[test]
fn a_marked_waiting_change_forms_alone_and_the_view_marks_it() {
    let mut world = batched();
    let mut root = Trains::default();
    waiting(&mut world, 3, b"main");
    let mut marked = world.change(CHANGE).clone();
    marked.change.alone = true;
    world.event(top::Event::Change { row: marked });
    // The document is read before forming while the marked row is still waiting.
    let document = world.document(landings::PROJECT, top::changes_subject(landings::PROJECT)).expect("changes");
    let top::Document::Changes(view) = document else { panic!("changes view") };
    assert!(view.queues[0].changes[0].alone);
    root.judge(&mut world);
    assert_eq!(members(&world, LANDING), [CHANGE]);
    assert_eq!(world.change(CHANGE + 1).change.landing, None);
}
#[test]
fn selection_takes_whole_units_and_passes_refusals() {
    struct Case { sizes: &'static [usize], batch: u16, bound: u32, marked: Option<usize>, refused: Option<usize>, take: &'static [u32], hold: &'static [u32] }
    for case in [
        Case { sizes: &[1,1,1], batch: 2, bound: 3, marked: None, refused: None, take: &[0,1], hold: &[] },
        Case { sizes: &[3], batch: 2, bound: 3, marked: None, refused: None, take: &[0], hold: &[] },
        Case { sizes: &[1,2], batch: 2, bound: 3, marked: None, refused: None, take: &[0], hold: &[] },
        Case { sizes: &[1,1], batch: 3, bound: 3, marked: Some(0), refused: None, take: &[0], hold: &[] },
        Case { sizes: &[1,1], batch: 3, bound: 3, marked: Some(1), refused: None, take: &[0], hold: &[] },
        Case { sizes: &[1,1], batch: 3, bound: 3, marked: None, refused: Some(0), take: &[1], hold: &[0] },
        Case { sizes: &[3,1], batch: 2, bound: 2, marked: None, refused: None, take: &[1], hold: &[0] },
    ] {
        let units: Vec<_> = case.sizes.iter().enumerate().map(|(index, size)| top::WaitingUnit { changes: (0..*size as u64).collect(), alone: case.marked == Some(index), refused: case.refused == Some(index) }).collect();
        let got = top::select_units(&units, case.batch, case.bound);
        assert_eq!(got.take.as_ref(), case.take, "{:?}", case.sizes);
        assert_eq!(got.hold.as_ref(), case.hold, "{:?}", case.sizes);
    }
    let world = batched();
    let words = world.unit_words(&[1,2,3,4]);
    assert!(String::from_utf8(words.into_vec()).unwrap().contains("forge.landing_members"));
}
