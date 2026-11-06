use jig_conformance::{Cut, scenarios::Scenario};
use jig_conformance_world::{Testing, replay};

#[test]
fn a_forked_snapshot_replays_to_the_durable_rows() {
    let world = replay::fresh(Scenario::LateCopy, Cut::None, 81, false).expect("fresh story");
    let rows = world.store.rows.clone();
    let forked = jig_conformance::Harness::<Testing>::from_snapshot(world.cold_snapshot()).expect("fork restarts");
    assert_eq!(forked.store.rows, rows);
}

pub(super) fn compare(scenario: Scenario, seed: u64, selected: Option<&[u64]>) {
    let cuts = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let reached = std::rc::Rc::clone(&cuts);
    let visit = move |cut, resumed: jig_conformance_world::actions::World| {
        let actual = replay::outside(&resumed);
        assert!(resumed.outcome().crashes.contains(&cut));
        drop(resumed);
        let fresh = replay::fresh(scenario, Cut::AfterCommit(cut), seed, true).expect("fresh selected cut");
        if let Some(difference) = actual.first_difference(&replay::outside(&fresh)) {
            panic!("{scenario:?}, cut {cut}, seed {seed}: {difference}");
        }
        reached.borrow_mut().push(cut);
    };
    let baseline = match selected {
        Some(cuts) => replay::chosen_cuts_traced(scenario, seed, Some(true), cuts, visit),
        None => replay::every_cut_traced(scenario, seed, Some(true), visit),
    }
    .expect("snapshot cuts");
    assert_eq!(*cuts.borrow(), selected.map_or_else(|| (1..=baseline.commits).collect::<Vec<_>>(), <[u64]>::to_vec));
    assert!(baseline.commits >= if selected.is_some() { 3 } else { 1 });
}
#[test]
fn late_copy_snapshots_match_complete_fresh_state() {
    compare(Scenario::LateCopy, 81, Some(&[1, 2, 3]));
}
#[test]
fn other_hand_snapshots_match_complete_fresh_state() {
    compare(Scenario::OtherHand, 82, Some(&[1, 2, 3]));
}
#[test]
fn uncertain_restarts_snapshots_match_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83, Some(&[1, 2, 3]));
}
#[test]
fn changed_judge_snapshots_match_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84, Some(&[1, 2, 3]));
}
#[test]
fn shrinking_pool_snapshots_match_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85, Some(&[1, 2, 3]));
}
#[test]
fn narrowing_policy_snapshots_match_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86, Some(&[1, 2, 3]));
}
#[test]
fn standing_task_snapshots_match_complete_fresh_state() {
    compare(Scenario::StandingTask, 87, Some(&[1, 2, 3]));
}
#[test]
fn run_budget_snapshots_match_complete_fresh_state() {
    compare(Scenario::RunBudget, 88, Some(&[1, 2, 3]));
}
