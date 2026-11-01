//! Profile codec and settings-record edges remain representable.
use jig_protocol_config::{Profile, profile};
#[test]
fn settings_records_fit_the_supplied_profiles() {
    for selected in [Profile::Small, Profile::Standard] {
        let limits = profile(selected).core;
        assert!(jig_domain_core::valid(&limits));
        let store = jig_protocol_store::codec::limits(&limits).expect("profile codec limits");
        assert!(jig_protocol_store::core_v7::SettledCall::encoded_bound(&store.core).is_some());
        assert!(jig_protocol_store::core_v7::SettledCall::decoded_bound(&store.core).is_some());
        assert!(jig_protocol_store::core_v7::CallRecord::encoded_bound(&store.core).is_some());
        assert!(jig_protocol_store::core_v7::CallRecord::decoded_bound(&store.core).is_some());
        assert!(jig_protocol_store::core_v7::Row::encoded_bound(&store.core).is_some());
        assert!(jig_protocol_store::core_v7::Row::decoded_bound(&store.core).is_some());
        assert!(jig_domain_core::settings::set_bound(&limits).is_some());
    }
}
