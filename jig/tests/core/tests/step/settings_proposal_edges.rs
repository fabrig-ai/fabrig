#[test]
fn settings_proposal_profiles_keep_the_advertised_bounds() {
    assert_eq!(jig_protocol_config_fixtures::small().core.settings.proposals, 4);
    assert_eq!(jig_protocol_config::stock().core.settings.proposals, 8);
}
