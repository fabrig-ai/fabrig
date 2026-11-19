//! The walk binary starts no browser (temper/docs/design/client/journey-walks.md, 7).
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use toml::Value;

fn normal(manifest: &Value, inherited: &Value) -> Vec<String> {
    let mut tables = vec![];
    if let Some(table) = manifest.get("dependencies") {
        tables.push(table);
    }
    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            if let Some(table) = target.get("dependencies") {
                tables.push(table);
            }
        }
    }
    let mut names = vec![];
    for table in tables {
        for (name, given) in table.as_table().expect("dependency table") {
            let given = if given.get("workspace").and_then(Value::as_bool) == Some(true) {
                inherited.get(name).expect("workspace dependency")
            } else {
                given
            };
            names.push(given.get("package").and_then(Value::as_str).unwrap_or(name).to_owned());
        }
    }
    names
}

#[test]
fn the_walk_binarys_normal_closure_has_no_browser_or_journeys_tool() {
    let root = skein_scratch::paths::workspace();
    let workspace: Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml")).expect("workspace manifest"))
        .expect("workspace TOML");
    let inherited = &workspace["workspace"]["dependencies"];
    let mut dependencies = BTreeMap::new();
    for directory in workspace["workspace"]["members"].as_array().expect("workspace members") {
        let manifest: Value = toml::from_str(
            &fs::read_to_string(root.join(directory.as_str().expect("member path")).join("Cargo.toml"))
                .expect("member manifest"),
        )
        .expect("member TOML");
        let name = manifest["package"]["name"].as_str().expect("package name").to_owned();
        dependencies.insert(name, normal(&manifest, inherited));
    }
    assert!(dependencies.contains_key("temper-client-walk"), "the binary is a workspace member");
    let mut seen = BTreeSet::new();
    let mut pending = vec!["temper-client-walk".to_owned()];
    while let Some(name) = pending.pop() {
        if seen.insert(name.clone())
            && let Some(next) = dependencies.get(&name)
        {
            pending.extend(next.iter().cloned());
        }
    }
    for forbidden in ["skein-browser", "skein-fake-person-browser", "temper-client-journeys-tool"] {
        assert!(!seen.contains(forbidden), "temper-client-walk's normal closure names {forbidden}");
    }
    assert!(seen.contains("temper-client-walk-world"), "the check traverses the world's composition");
}
