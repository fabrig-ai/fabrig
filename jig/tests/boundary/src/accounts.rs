//! The accounts component links its child, store, configuration and skein's drivers.
const MANIFEST: &str = include_str!("../../../crates/jig-protocol-accounts/Cargo.toml");

fn breaches(source: &str) -> Vec<String> {
    let manifest = crate::manifest::parse_manifest("jig-protocol-accounts/Cargo.toml".into(), source);
    manifest
        .dependencies
        .into_iter()
        .filter(|name| name.starts_with("jig-") || name.starts_with("skein-") || name.starts_with("smith-"))
        .filter(|name| {
            ![
                "skein-lib",
                "skein-codec",
                "skein-oauth",
                "skein-oauth-accounts",
                "skein-io",
                "jig-protocol-accounts-codec",
                "jig-domain-accounts",
                "jig-domain-link",
                "jig-protocol-store",
                "jig-protocol-config",
                "skein-kv",
            ]
            .contains(&name.as_str())
        })
        .collect()
}

#[test]
fn the_accounts_part_links_only_its_child_store_configuration_and_skein() {
    let failures = breaches(MANIFEST);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn accounts_fixture_checks_each_normal_dependency_form() {
    for dependency in [
        "jig-domain-core = { path = \"x\" }",
        "jig-domain-core = { workspace = true }",
        "jig-domain-core.workspace = true",
    ] {
        let source = format!("[package]\nname = \"jig-protocol-accounts\"\n[dependencies]\n{dependency}\n");
        assert_eq!(breaches(&source), ["jig-domain-core"], "{dependency}");
    }
    let source = "[package]\nname = \"jig-protocol-accounts\"\n[dependencies]\n\
                  jig-domain-accounts = { path = \"x\" }\n[dev-dependencies]\n\
                  jig-domain-core = { path = \"x\" }\n";
    assert_eq!(breaches(source).len(), 0);
}
