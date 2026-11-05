use hearth_config_world::fixtures;

mod limits;
mod measured;

#[test]
fn each_step_holds_no_more_than_its_ceiling_under_tiny_limits() {
    const CEILING: u64 = 53_000;
    let limits = limits::JSON;
    let documents = fixtures::at_the_limits(&limits);
    assert_eq!(documents.len(), 8);
    for (document, what) in documents {
        let peak = measured::metered(&document, &limits, CEILING, what);
        assert!(peak > 0, "{what}");
    }
}

#[test]
fn declarations_hold_no_more_than_their_ceiling_at_the_limits() {
    const CEILING: u64 = 18_000;
    let limits = limits::DECLARATION;
    let documents = fixtures::declarations_at_the_limits(&limits);
    assert_eq!(documents.len(), 4);
    for (document, what) in documents {
        let peak = measured::metered_declaration(&document, &limits, CEILING, what);
        assert!(peak > 0, "{what}");
    }
}
