//! Blocking judgement at the round prefix (domain/landing.md, 3, 15).
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{World, changes, landings::{self, CHANGE, LANDING}, translate};
const MEMBER: &[u8] = b"temper/51";
const GATE: u64 = 9;
fn reviewed(kind: change::GateKind, freshness: change::Freshness) -> World { reviewed_history(kind, freshness, false) }
fn reviewed_history(kind: change::GateKind, freshness: change::Freshness, history: bool) -> World {
    let mut world = landings::turn_world();
    world.produce(MEMBER);
    let earlier = translate::commit(world.branch(MEMBER));
    world.push_file(b"main", b"base", b"base moved");
    if history { world.resolve(MEMBER, b"change"); }
    let head = translate::commit(world.branch(MEMBER));
    let mut row = changes::change_row();
    row.pull = Some(world.open_pull(MEMBER, b"main"));
    row.change.last_head = Some(head);
    row.change.ready_since = Some(world.wall());
    row.change.state = change::State::Ready { head, ready_since: world.wall() };
    row.change.gates = Box::new([change::Gate { number: GATE, kind, blocking: true, freshness, eager: true }]);
    row.change.clean = if history { Box::new([earlier]) } else { Box::new([]) };
    row.verdicts = Box::new([change::GateReport { number: GATE, head: if history { earlier } else { head }, status: change::Status::Passed }]);
    world.event(top::Event::Change { row });
    assert_eq!(world.change(CHANGE).change.gates.len(), 1);
    let row = landings::landing_row(world.change(CHANGE));
    world.event(top::Event::Landing { row });
    world
}
fn request_judgement(world: &mut World, first: u64) -> api::Commit {
    for entry in first..first + 40 {
        let decision = landings::step_landing(world, LANDING, entry);
        if decision == (landing::Decision::Judge { member: 0 }) {
            return world.landing(LANDING).landing.members[0].candidate.expect("prefix");
        }
        assert_ne!(decision, landing::Decision::Finish, "exact judgement is asked at the prefix");
    }
    panic!("no prefix judgement: {:?}", world.landing(LANDING));
}
fn change_step(world: &mut World, entry: u64) -> change::Decision {
    changes::change_step_task(world, CHANGE, entry, change::Status::Passed, Box::new([]), false, false)
}
fn ask_gate(world: &mut World, prefix: api::Commit, entry: u64) {
    assert_eq!(change_step(world, entry), change::Decision::Delegate(change::Delegate::Gate { number: GATE, head: prefix }));
    world.event(top::Event::Delegated { task: CHANGE, child: 71, kind: change::Delegate::Gate { number: GATE, head: prefix } });
}
fn verdict(world: &mut World, passed: bool) {
    world.event(top::Event::DelegateResult {
        task: CHANGE, child: 71, status: if passed { change::Status::Passed } else { change::Status::Failed },
        failed: false, cancelled: false, no_change: false, words: b"reviewed the prefix".as_slice().into(),
    });
}
#[test]
fn a_clean_review_at_the_members_head_carries_to_its_prefix_without_an_ask() {
    let mut world = reviewed(change::GateKind::Agent, change::Freshness::Clean);
    for entry in 100..160 {
        let decision = landings::step_landing(&mut world, LANDING, entry);
        assert_ne!(decision, landing::Decision::Judge { member: 0 }, "clean review carries without a delegate");
        if decision == landing::Decision::Finish {
            assert!(world.landing(LANDING).landing.members[0].landed.is_some());
            return;
        }
    }
    panic!("clean judgement did not land");
}
#[test]
fn exact_reviews_and_checks_wait_at_the_round_pull_beside_tests_and_survive_restart() {
    for (kind, freshness) in [(change::GateKind::Agent, change::Freshness::Exact), (change::GateKind::Check, change::Freshness::Clean)] {
        let mut world = reviewed(kind, freshness);
        let prefix = request_judgement(&mut world, 100);
        ask_gate(&mut world, prefix, 200);
        let expected = world.change(CHANGE).change.state.clone();
        let change::State::Judging { target, .. } = &expected else { panic!("judging is durable") };
        let row = &world.landing(LANDING).landing;
        assert_eq!((target.landing, target.round, Some(target.pull), &target.branch, &target.base, target.prefix),
            (LANDING, row.preparation, row.round.pull, &row.round.branch, &row.branch, prefix));
        assert_ne!(Some(target.pull), world.change(CHANGE).pull);
        let workspace = world.workspace_item(top::items::Seed { modify: false, task: 71, parent: Some(CHANGE), repository: temper_engine_forge_world::REPO, run_branch: Box::new([]), saved_branch: None }).expect("review checkout");
        assert_eq!(workspace.start, top::items::Start::Branch(target.branch.clone()));
        assert_eq!(workspace.push, None);
        let brief = String::from_utf8(world.pull_brief(CHANGE, prefix).into_vec()).expect("pull brief");
        assert!(brief.contains(&format!("Pull request: #{}\nBranch: {}\nBase: main\nHead: ", target.pull, String::from_utf8_lossy(&target.branch))), "{brief}");
        world.restart();
        assert_eq!(world.change(CHANGE).change.state, expected);
        for entry in 210..225 {
            assert_ne!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Finish, "judgement blocks after tests");
        }
        assert!(world.take_calls().iter().any(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))));
        assert!(world.landing(LANDING).landing.members[0].landed.is_none());
        verdict(&mut world, true);
        change_step(&mut world, 230);
        assert!(matches!(world.change(CHANGE).change.state, change::State::Ready { .. }));
        landings::drive_landing(&mut world, LANDING);
        assert_eq!(world.landing(LANDING).landing.members[0].landed, Some(prefix));
    }
}
#[test]
fn a_refused_prefix_review_repairs_at_the_changes_own_pull_and_the_member_leaves() {
    let mut world = reviewed(change::GateKind::Agent, change::Freshness::Exact);
    let ready = world.change(CHANGE).change.ready_since;
    let own_pull = world.change(CHANGE).pull;
    let prefix = request_judgement(&mut world, 100);
    ask_gate(&mut world, prefix, 200);
    verdict(&mut world, false);
    assert_eq!(change_step(&mut world, 210), change::Decision::Delegate(change::Delegate::Repair(change::Repair::Gate(GATE))));
    assert_eq!(world.change(CHANGE).review_target().expect("repair pull").0, own_pull.expect("own pull"));
    assert_eq!(landings::step_landing(&mut world, LANDING, 211), landing::Decision::Leave { member: 0, why: landing::Leave::Refused { gate: GATE } });
    landings::drive_landing(&mut world, LANDING);
    assert_eq!(world.landing(LANDING).landing.members[0].left, Some(landing::Leave::Refused { gate: GATE }));
    assert_eq!(world.change(CHANGE).change.ready_since, ready);
    assert!(world.landing(LANDING).landing.members[0].landed.is_none());
    assert!(!world.has_branch(&world.landing(LANDING).landing.round.branch));
    assert_ne!(translate::commit(world.branch(b"main")), prefix);
}
#[test]
fn a_prefix_rebuilt_in_a_new_round_asks_for_its_exact_review_again() {
    let mut world = reviewed(change::GateKind::Agent, change::Freshness::Exact);
    let first = request_judgement(&mut world, 100);
    ask_gate(&mut world, first, 200);
    verdict(&mut world, true);
    change_step(&mut world, 210);
    world.push_file(b"main", b"second", b"new base");
    let second = request_judgement(&mut world, 300);
    assert_ne!(second, first);
    ask_gate(&mut world, second, 400);
    assert_eq!(world.landing(LANDING).landing.preparation, 2);
}

#[test]
fn a_clean_review_carried_through_a_clean_update_counts_at_the_prefix() {
    let mut world = reviewed_history(change::GateKind::Agent, change::Freshness::Clean, true);
    for entry in 100..160 {
        let decision = landings::step_landing(&mut world, LANDING, entry);
        assert_ne!(decision, landing::Decision::Judge { member: 0 }, "clean predecessor carries to the prefix");
        if decision == landing::Decision::Finish { assert!(world.landing(LANDING).landing.members[0].landed.is_some()); return; }
    }
    panic!("clean predecessor never landed");
}
