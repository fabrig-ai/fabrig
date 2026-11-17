#[test]
fn settings_proposal_profiles_keep_the_advertised_bounds() {
    assert_eq!(jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.settings.proposals, 4);
    assert_eq!(jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core.settings.proposals, 8);
}
