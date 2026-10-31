#[test]
fn invalid_limits_are_refused() {
    let mut limits = smith_agent_world::LIMITS.session;
    assert!(super::valid(&limits));
    limits.completion_blocks = u32::MAX;
    assert!(!super::valid(&limits));
}
