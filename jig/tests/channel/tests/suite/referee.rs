use jig_channel_world::fixtures;
use skein_world::domain::Verdict;

#[test]
fn an_answer_acknowledged_twice_is_caught() {
    let mut referee = fixtures::referee();
    referee.answered(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    assert!(matches!(referee.verdict(), Verdict::Failed(_)));
}

#[test]
fn a_grant_value_planted_in_a_root_s_state_is_caught() {
    for (label, value) in [("token", b"planted-secret".as_slice()), ("account id", b"acc".as_slice())] {
        let mut referee = fixtures::referee();
        referee.roots(&format!("root {{ credential: {value:?} }}"), &[value]);
        assert!(matches!(referee.verdict(), Verdict::Failed(_)), "{label}");
    }
}

#[test]
fn a_refused_start_acknowledged_is_caught() {
    let mut referee = fixtures::referee();
    referee.refused(fixtures::TASK, fixtures::ATTEMPT);
    referee.answered(fixtures::TASK, fixtures::ATTEMPT);
    referee.acknowledged(fixtures::TASK, fixtures::ATTEMPT);
    assert!(matches!(referee.verdict(), Verdict::Failed(_)));
}

#[test]
fn an_attempt_placed_before_its_predecessors_stop_bound_is_caught() {
    use skein_lib::Time;
    for (label, at, failed) in [("before stop bound", 9, true), ("at stop bound", 10, false)] {
        let mut referee = fixtures::referee();
        referee.placed(fixtures::TASK, fixtures::ATTEMPT, Time::ZERO);
        referee.lost(fixtures::TASK, fixtures::ATTEMPT, Time::from_nanos(10));
        referee.placed(fixtures::TASK, fixtures::ATTEMPT + 1, Time::from_nanos(at));
        assert_eq!(matches!(referee.verdict(), Verdict::Failed(_)), failed, "{label}");
    }
}

#[test]
fn roots_are_due_where_a_grant_arrives_and_at_the_next_root_step() {
    let mut checks = fixtures::root_checks(false);
    assert!(!checks.due(0, 1, 0));
    assert!(!checks.due(0, 1, 1));
    assert!(checks.due(1, 1, 1), "grant arrives without a root step");
    assert!(!checks.due(1, 1, 1), "same step after arrival");
    assert!(checks.due(1, 2, 7), "each host sees the grant");
    assert!(checks.due(1, 1, 2), "receiving root takes up the grant");
    assert!(!checks.due(1, 1, 2));
    assert!(!checks.due(1, 1, 3), "later steps need no check");
    assert!(!checks.due(1, 2, 7), "another host's step does not take up this host's grant");
    assert!(checks.due(1, 2, 8), "the second host still has its next step due");
    assert!(!checks.due(1, 2, 9));
    assert!(checks.due(2, 1, 3), "a replacement grant arrives");
    assert!(!checks.due(2, 1, 3));
    assert!(checks.due(2, 1, 4));
    assert!(!checks.due(2, 1, 5));
    assert!(checks.due(2, 2, 9), "replacement arrives independently on the second host");
    assert!(checks.due(2, 2, 10));
    assert!(!checks.due(2, 2, 11));
}

#[test]
fn every_step_keeps_each_root_step_due() {
    let mut checks = fixtures::root_checks(true);
    assert!(checks.due(1, 1, 4));
    assert!(!checks.due(1, 1, 4));
    assert!(checks.due(1, 1, 5));
    assert!(!checks.due(1, 1, 5));
    assert!(checks.due(1, 1, 6), "steps after the grant's receiving step stay due");
    assert!(!checks.due(1, 1, 6));
    assert!(checks.due(1, 2, 8));
    assert!(!checks.due(1, 2, 8));
    assert!(checks.due(1, 2, 9));
    assert!(checks.due(1, 2, 10));
    assert!(checks.due(1, 1, 7), "hosts keep separate step counts");
}
