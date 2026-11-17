//! Named limits round trips (temper/docs/design/protocol/configuration/figures.md, 9).
#![cfg(test)]
use skein_lib::Duration;
use temper_engine_service_vocabulary::limits;

#[test]
fn temper_limits_by_name_round_trip_both_stocks() {
    let small = temper_limits_fixtures::small();
    let stock = limits::stock();
    for (mut source, mut restored) in [(small, stock), (stock, small)] {
        source.checkout.workspaces = 7;
        source.checkout.remote_timeout = Duration::from_millis(321);
        let values = limits::list(&source);
        let mut unique = skein_lib::Set::with_capacity(values.len());
        for &(name, _) in &values {
            assert!(unique.insert(name).unwrap());
        }
        for &(name, value) in &values {
            limits::apply(&mut restored, name.as_bytes(), value).unwrap();
        }
        assert_eq!(restored, source);
        let mut shared = temper_service_core::limits::stock();
        for &(name, value) in &values {
            temper_service_core::limits::apply(&mut shared, name.as_bytes(), value).unwrap();
        }
        assert_eq!(shared.checkout, source.checkout);
        assert_eq!(shared.git_component, source.git_component);
        assert_eq!(shared.peer.forges, source.forge.forges);
        assert_eq!(shared.peer.installations, source.forges.installations);
        assert_eq!(shared.peer.token_bytes, source.forge_component.token_bytes);
    }
}
