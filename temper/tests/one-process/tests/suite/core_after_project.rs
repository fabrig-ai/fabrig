//! No core wait: the gate is open and the root ready; a two-membership client
//! rejects the mandatory `me` after project three (temper/docs/design/client/tools.md, 12;
//! jig/docs/design/client/domain/contract/documents.md, 1), and meets those memberships again at restart.
fn answers(outcomes: [temper_one_process_world::fixtures::core_after_project::Outcome; 2]) {
    for outcome in outcomes {
        assert_eq!(outcome.calls, 4);
        assert_eq!(outcome.fresh, 1);
        assert!(outcome.native, "{}", outcome.gate);
    }
}

#[test]
fn a_core_answers_after_a_project_is_made() {
    answers(temper_one_process_world::fixtures::core_after_project::sequence(false));
}

#[test]
fn the_incident_s_whole_sequence_answers() {
    answers(temper_one_process_world::fixtures::core_after_project::sequence(true));
}

#[test]
fn a_store_made_with_the_two_membership_client_answers_after_restart() {
    let [before, after] = temper_one_process_world::fixtures::core_after_project::legacy_store();
    assert_eq!(before.calls, 4);
    assert_eq!(before.fresh, 1);
    assert!(before.legacy_rejected, "{}", before.gate);
    assert!(!before.native);
    assert_eq!(after.calls, 4);
    assert_eq!(after.fresh, 1);
    assert!(after.native, "{}", after.gate);
}
