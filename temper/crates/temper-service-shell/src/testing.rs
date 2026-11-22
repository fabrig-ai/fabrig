//! Provider setting fixture for the shell's tests (tools/crates/design.md, 3.2).
use jig_protocol_config::{self as config, ProviderSetting};

#[must_use]
pub fn provider_fixture() -> ProviderSetting {
    let document = config::document(br#"{"number":4,"name":"local","endpoint":"local","signed_in_plan":false,"sources":{},"endpoints":[{"name":"local","number":5,"dialect":3,"address":{"socket":"127.0.0.1:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}"#, &config::DocumentLimits { bytes: 4096, tokens: 256, depth: 16 }).expect("parse the fixture provider");
    let mut problems = config::Problems::empty();
    config::llm::provider_setting(&document, 0, &config::Path::root(), &mut problems)
        .expect("build the fixture provider")
}
