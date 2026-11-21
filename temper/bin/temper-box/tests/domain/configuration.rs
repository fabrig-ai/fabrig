//! Configuration values and committed overrides.
use crate::support;
use support::configuration_fixture::DOCUMENT;
mod turn_landing;

#[test]
fn the_browsers_client_bounds_are_the_terminals() {
    assert_eq!(
        temper_client_shell_web::limits::client(),
        temper_client_shell_terminal::limits::native().client,
        "the browser receives what the terminal does (client/README.md, 2)"
    );
}

#[test]
fn the_owned_example_fits_the_native_client() {
    let value: serde_json::Value = serde_json::from_slice(support::owned::EXAMPLE).expect("owned example JSON");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    // Parsing a fixture without a deployment; only its limits are used.
    let mut figures = temper_playground::documents::Configuration::stock(value.clone());
    figures.jig_over("documents.text_bytes", 512);
    figures.jig_over("documents.rows", 2);
    let engine = temper_service::startup::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &mut figures.jig,
    )
    .expect("engine configuration");
    let native = temper_client_shell_terminal::limits::native().client.client.domain;
    assert!(engine.limits.core.documents.rows <= native.rows);
    assert!(engine.limits.core.documents.lines <= native.rows);
    let paths = engine
        .limits
        .core
        .authority
        .segments
        .max(engine.limits.core.tasks.hold_segments)
        .max(engine.limits.core.tasks.authority_segments);
    assert!(paths <= native.segments);
    assert!(engine.limits.core.documents.text_bytes <= native.text);
    assert!(engine.limits.core.authority.grants <= native.authority);
    assert!(engine.limits.core.tasks.authority_grants <= native.authority);
    assert_owned_limits(&value["engine"]);
}

fn assert_owned_limits(engine: &serde_json::Value) {
    assert!(engine.get("limits").is_none());
    assert!(engine.get("memory_bytes").is_none());
}

#[test]
fn the_retained_cluster_overrides_name_no_charter_documents() {
    // Two charters, the chat template, the deployment landing rule, two workers and the agents' policy.
    const ITEMS: usize = 7;
    let bytes = std::fs::read(skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/cluster/settings.json"))
        .expect("retained cluster overrides");
    // The process story fills whole settings from the owner's reads before decoding.
    let overrides: serde_json::Value = serde_json::from_slice(&bytes).expect("cluster overrides JSON");
    let items = overrides["items"].as_array().expect("cluster setting overrides");
    assert_eq!(items.len(), ITEMS);
    for (index, number) in [(0, 1), (1, 2)] {
        let item = &items[index];
        assert_eq!(item["setting"], serde_json::json!({"kind":"charter","number":number}));
        assert_eq!(item["set"]["documents"], serde_json::json!([]));
    }
}
