use hearth_config_world::fixtures;
use hearth_protocol_config::json;
#[path = "../../src/fixtures/metered.rs"]
mod measured;
#[expect(clippy::module_inception, reason = "The memory suite module keeps the fixture module's name.")]
#[path = "../../src/fixtures/memory.rs"]
mod memory;
#[test]
fn each_step_holds_no_more_than_the_worst_case_under_tiny_limits() {
    let limits = memory::JSON;
    let bound = json::worst_case(&limits).expect("valid limits");
    let documents = fixtures::at_the_limits(&limits);
    assert_eq!(documents.len(), 8);
    for (document, what) in documents {
        assert!(measured::metered(&document, &limits, bound, what) > 0, "{what}");
    }
}

#[test]
fn declarations_hold_no_more_than_their_worst_case_at_the_limits() {
    let limits = memory::DECLARATION;
    let bound = hearth_protocol_config::declaration::worst_case(&limits).expect("valid limits");
    let documents = fixtures::declarations_at_the_limits(&limits);
    assert_eq!(documents.len(), 4);
    for (document, what) in documents {
        assert!(measured::metered_declaration(&document, &limits, bound, what) > 0, "{what}");
    }
}
