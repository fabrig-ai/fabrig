//! The mandatory `me` receives every membership, before and after reopening the
//! same disk (temper/docs/design/client/tools.md, 12; domain/root.md, 9.1).
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
