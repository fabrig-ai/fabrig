//! temper's client crates carry `temper-client-` (jig's README.md, 15).

#[test]
fn client_crates_carry_the_client_prefix() {
    // Tests run from their package's directory, `tests/client`.
    let root = std::env::current_dir().expect("package directory").join("../..");
    let root = root.canonicalize().expect("temper's root");
    // The playground composes the engine, worker and visitor (temper/docs/design/playground.md, 3).
    let breaches = jig_boundary::client_names::breaches(
        &root,
        "temper",
        &[
            // The client's thin binary calls its prefixed shell (temper/docs/design/cli/README.md, 2).
            "temper",
            "temper-box-shell",
            // Both binaries share their start helpers (temper/docs/design/client/README.md, 2).
            "temper-shell",
            // The engine's host composes the service and bundle (temper/docs/design/protocol/components.md).
            "temper-engine-shell",
            // The setup host is shared by the engine's commands (temper/docs/design/protocol/setup/README.md, 7).
            "temper-forge-setup-shell",
            // The engine world and shared fixtures keep their test-kit names
            // (temper/docs/design/testing/README.md, 5).
            "temper-engine-domain-world",
            "temper-fixtures",
            "temper-test-env",
            "temper-one-process-world",
            "temper-real-loop-world",
            "temper-playground",
            "temper-playground-world",
        ],
    );
    assert!(breaches.is_empty(), "client crates without the temper-client- prefix: {}", breaches.join(", "));
}
