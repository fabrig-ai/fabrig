//! The whole incident, before and after a restart of the same file store
//! (temper/docs/design/domain/root.md, 5.7 and 9.1).
#[test]
fn a_core_answers_after_a_project_is_made() {
    for outcome in temper_one_process_world::fixtures::core_after_project::whole_sequence() {
        assert_eq!(outcome.calls, 4);
        assert_eq!(outcome.fresh, 1);
        assert!(outcome.native);
    }
}
