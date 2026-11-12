#[test]
fn documentation_answer_bound_matches_the_engine() {
    assert_eq!(temper_service_core::DOC_ANSWER, temper_engine_smith::DOC_ANSWER);
}
