use jig_protocol_config::{self as config, Defaults, Directories, DocumentLimits};

#[test]
fn small_limits_pass_the_engine_reader_checks() {
    let document = config::document(
        br#"{
        "limits":{},
        "llm":{"endpoints":[],"providers":[]},
        "hosts":{"listener":[]},
        "clients":{"heartbeat_ms":10000,"link":false},
        "deployment":{"directory":"state"},
        "providers":[{"number":1,"kind":"deployment"}],
        "owners":[{"provider":1,"subject":"owner"}],
        "projects":[],"connectors":[],"memory_bytes":34359738368
        }"#,
        &DocumentLimits { bytes: 2048, tokens: 512, depth: 8 },
    )
    .expect("fixture JSON");
    let parsed = config::engine_over(
        &document,
        0,
        &Directories::working(),
        Defaults::Passed,
        &config::Fixed::NONE,
        &crate::small(),
    )
    .expect("small limits pass every reader check");
    let mut expected = crate::small();
    // The reader derives two streams plus four ordinary exchanges.
    expected.clients.exchanges = 6;
    assert_eq!(parsed.limits, expected);
}

#[test]
fn small_holds_composed_settings() {
    let limits = crate::small();
    let tasks = limits.core.tasks;
    assert_eq!(tasks.tasks, 8);
    assert_eq!(tasks.batch, 2);
    assert_eq!(tasks.spec_bytes, 4_096);
    assert_eq!(tasks.result_bytes, 512);
    assert_eq!(limits.core.settings.alternatives, 63);
    assert_eq!(limits.store.store.value, 4_580_852);
}
