//! Endpoint capacity at temper's start (protocol/configuration/figures.md, 2).
use super::fixtures;
use skein_lib::{Decimal, List};
fn given(count: u32) -> crate::stock::Given {
    let mut providers = temper_worker_service_fixtures::providers();
    let mut endpoints = List::with_capacity(count);
    for number in 1..=count {
        let mut endpoint = providers.first().unwrap().endpoints.first().unwrap().clone();
        endpoint.number = number;
        if number != 1 {
            let number = Decimal::of(u64::from(number));
            let mut name = List::uncapped();
            for byte in b"fake-".iter().chain(number.as_bytes()) {
                name.push(*byte).unwrap();
            }
            endpoint.name = name.into_boxed();
        }
        endpoints.push(endpoint).unwrap();
    }
    providers.first_mut().unwrap().endpoints = endpoints.into_boxed();
    let mut jig = crate::stock::checked_with(Box::new([fixtures::small_model()]), 32).unwrap();
    jig.providers = providers;
    crate::stock::Given::Own { jig, forge: Box::new([]), ways: Box::new([]) }
}
#[test]
fn startup_accepts_64_endpoints_and_names_65() {
    let directories = jig_protocol_config::Directories::working();
    let document = fixtures::engine_part(&fixtures::document());
    let accepted = crate::startup::named_with(&document, &directories, &given(64)).unwrap();
    assert_eq!(accepted.llm.endpoints.len(), 64);
    let found = crate::startup::named_with(&document, &directories, &given(65)).unwrap_err();
    assert_eq!(
        crate::startup::lines(&found).as_ref(),
        b"providers.setting: 65 endpoints, at most 64, the workers' figures carry no more\n"
    );
}
