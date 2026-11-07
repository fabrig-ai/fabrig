use jig_conformance::{
    Application, Cut,
    scenarios::{self, Scenario},
};
use temper_conformance_world::{Config, Temper};

/// Crash one shared scenario at every live durable commit after the fixture receipts.
fn crashed_at_every_live_commit(scenario: Scenario) {
    // Setup's real first-start cut is exercised by the engine-domain world.
    let first_live = Temper::store(&Config::new(101, Some(scenario))).applied + 1;
    let outcome = scenarios::run::<Temper>(scenario, Cut::EachCommitFrom(first_live), 101)
        .unwrap_or_else(|error| panic!("{scenario:?}: {error:?}"))
        .expect("applicable");
    jig_conformance::cut_passed();
    assert_eq!(
        outcome.forked,
        (first_live..=outcome.commits).collect::<Vec<_>>(),
        "{scenario:?}: every live commit, once"
    );
}

#[test]
fn a_late_copy_is_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::LateCopy);
}

#[test]
fn another_hand_is_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::OtherHand);
}

#[test]
fn uncertain_restarts_are_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::UncertainRestarts);
}

#[test]
fn a_changed_judge_is_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::ChangedJudge);
}

#[test]
fn a_narrowing_policy_is_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::NarrowingPolicy);
}

#[test]
fn a_standing_task_is_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::StandingTask);
}

#[test]
fn a_run_budget_is_crashed_at_every_live_commit() {
    crashed_at_every_live_commit(Scenario::RunBudget);
}

#[test]
fn a_shrinking_pool_is_not_applicable() {
    let outcome = scenarios::run::<Temper>(Scenario::ShrinkingPool, Cut::None, 101).expect("not applicable");
    assert!(outcome.is_none(), "temper's forge has no pooled holds");
}
