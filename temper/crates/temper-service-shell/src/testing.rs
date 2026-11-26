//! Hosted shell seams and fixtures (tools/crates/design.md, 3.2).
use jig_protocol_config::{self as config, ProviderSetting};

/// Redirect served links to the hosting world's output, without replacing the service adapter.
pub fn hosted_output<S>(host: &mut crate::host::Host<S>, store: Box<[u8]>, output: impl std::io::Write + 'static) {
    host.store = store;
    host.output = Box::new(output);
}

/// The same ending the shell reports on exit.
#[must_use]
pub fn hosted_ending<S>(host: &crate::host::Host<S>) -> (String, u8) {
    host.ending()
}

#[must_use]
pub fn provider_fixture() -> ProviderSetting {
    let document = config::document(br#"{"number":4,"name":"local","endpoint":"local","signed_in_plan":false,"sources":{},"endpoints":[{"name":"local","number":5,"dialect":3,"address":{"socket":"127.0.0.1:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}"#, &config::DocumentLimits { bytes: 4096, tokens: 256, depth: 16 }).expect("parse the fixture provider");
    let mut problems = config::Problems::empty();
    config::llm::provider_setting(&document, 0, &config::Path::root(), &mut problems)
        .expect("build the fixture provider")
}
