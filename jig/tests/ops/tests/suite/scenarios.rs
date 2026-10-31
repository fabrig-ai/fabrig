//! Every shared scenario is run or reported by name (domain/testing.md, 5).
use jig_conformance::{
    Cut,
    scenarios::{self, Scenario},
};
use jig_ops_world::conformance::Ops;
fn run(scenario: Scenario) {
    let outcome = scenarios::run::<Ops>(scenario, Cut::None, 17)
        .unwrap_or_else(|error| panic!("{scenario:?}: {error:?}"))
        .unwrap_or_else(|| panic!("{scenario:?}: ops must bind this scenario"));
    assert!(!outcome.stopped);
}
#[test]
fn scenario_late_copy() {
    run(Scenario::LateCopy);
}
#[test]
fn scenario_other_hand() {
    run(Scenario::OtherHand);
}
#[test]
fn scenario_uncertain_restarts() {
    run(Scenario::UncertainRestarts);
}
#[test]
fn scenario_changed_judge() {
    run(Scenario::ChangedJudge);
}
#[test]
fn scenario_shrinking_pool() {
    run(Scenario::ShrinkingPool);
}
#[test]
fn scenario_narrowing_policy() {
    run(Scenario::NarrowingPolicy);
}
#[test]
fn scenario_standing_task() {
    run(Scenario::StandingTask);
}
#[test]
fn scenario_run_budget() {
    run(Scenario::RunBudget);
}
#[test]
fn scenario_racing_decisions() {
    run(Scenario::RacingDecisions);
}
#[test]
fn scenario_conversation_ends_while_read() {
    run(Scenario::ConversationEndsWhileRead);
}
#[test]
fn scenario_query_watched_while_tasks_end() {
    run(Scenario::QueryWatchedWhileTasksEnd);
}
#[test]
fn scenario_compound() {
    run(Scenario::Compound);
}
#[test]
fn scenario_grace() {
    run(Scenario::Grace);
}
