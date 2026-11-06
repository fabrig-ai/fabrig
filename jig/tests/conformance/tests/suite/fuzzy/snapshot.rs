use super::super::snapshot::compare;
use jig_conformance::scenarios::Scenario;

#[test]
fn late_copy_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::LateCopy, 81, None);
}
#[test]
fn other_hand_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::OtherHand, 82, None);
}
#[test]
fn uncertain_restarts_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83, None);
}
#[test]
fn changed_judge_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84, None);
}
#[test]
fn shrinking_pool_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85, None);
}
#[test]
fn narrowing_policy_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86, None);
}
#[test]
fn standing_task_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::StandingTask, 87, None);
}
#[test]
fn run_budget_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::RunBudget, 88, None);
}
