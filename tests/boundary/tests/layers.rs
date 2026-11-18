//! Every test binary's layer, by rule or by table (tools/tiers/layers.md, 7).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use fabrig_boundary::Target;

const TREES: [&str; 5] = ["skein", "smith", "jig", "temper", "hearth"];

fn binary_id(package: &str, target: &Target) -> Result<(String, bool), String> {
    if target
        .kinds
        .iter()
        .any(|kind| ["lib", "rlib", "cdylib", "dylib", "staticlib", "proc-macro"].contains(&kind.as_str()))
    {
        return Ok((package.to_owned(), true));
    }
    for (kind, prefix) in [("bin", "bin/"), ("test", ""), ("example", "example/"), ("bench", "bench/")] {
        if target.kinds.iter().any(|value| value == kind) {
            let by_rule = kind == "bin";
            return Ok((format!("{package}::{prefix}{}", target.name), by_rule));
        }
    }
    Err(format!("{package} {}: unknown test target kind {:?}", target.name, target.kinds))
}

fn binaries(metadata: &str) -> Result<BTreeMap<String, (Option<String>, bool)>, String> {
    let value: serde_json::Value = serde_json::from_str(metadata).map_err(|why| format!("metadata: {why}"))?;
    let root = value["workspace_root"].as_str().ok_or("metadata: no workspace_root string")?;
    let root = Path::new(root);
    if !root.is_absolute() {
        return Err("metadata: workspace_root is not absolute".to_owned());
    }
    let mut binaries = BTreeMap::new();
    for package in fabrig_boundary::packages(metadata)? {
        let relative = package.directory.strip_prefix(root).map_err(|why| format!("{}: {why}", package.name))?;
        let tree = relative.components().next().and_then(|part| part.as_os_str().to_str());
        let tree = tree.filter(|tree| TREES.contains(tree));
        for target in package.targets.iter().filter(|target| target.test) {
            let (id, by_rule) = binary_id(&package.name, target)?;
            binaries.insert(id, (tree.map(str::to_owned), by_rule || tree.is_none()));
        }
    }
    Ok(binaries)
}

fn layer_names<'a>(table: &'a toml::Table, findings: &mut Vec<String>) -> BTreeSet<&'a str> {
    let Some(values) = table.get("layers").and_then(toml::Value::as_array) else {
        findings.push("layers: expected a list of layer names".to_owned());
        return BTreeSet::new();
    };
    let mut names = BTreeSet::new();
    for value in values {
        if let Some(name) = value.as_str() {
            if !names.insert(name) {
                findings.push(format!("layers: repeated name {name}"));
            }
        } else {
            findings.push("layers: every name must be a string".to_owned());
        }
    }
    if !names.contains("step") {
        findings.push("layers: missing step".to_owned());
    }
    names
}

fn findings(metadata: &str, table: &str) -> Result<Vec<String>, String> {
    let binaries = binaries(metadata)?;
    let table: toml::Table = match toml::from_str(table) {
        Ok(table) => table,
        Err(why) => {
            let line = why.span().map_or(1, |span| {
                1 + table.get(..span.start).expect("TOML error span is within its source").matches('\n').count()
            });
            return Ok(vec![format!("table line {line}: {}", why.message())]);
        }
    };
    let mut findings = Vec::new();
    let layers = layer_names(&table, &mut findings);
    let mut seen = BTreeSet::new();
    for (tree, value) in &table {
        if tree == "layers" {
            continue;
        }
        let Some(lines) = value.as_table() else {
            findings.push(format!("{tree}: expected a tree table; only layers and tree tables belong at the top"));
            continue;
        };
        let known_tree = TREES.contains(&tree.as_str());
        if !known_tree {
            findings.push(format!("[{tree}]: not a tree table"));
        }
        for (id, layer) in lines {
            seen.insert(id.as_str());
            let line = format!("[{tree}] {id}");
            match binaries.get(id) {
                None => findings.push(format!("{line}: no test binary in the workspace")),
                Some((_, true)) => findings.push(format!("{line}: placed by rule; remove the line")),
                Some((owner, false)) => {
                    if known_tree && owner.as_deref() != Some(tree) {
                        findings.push(format!("{line}: belongs under [{}]", owner.as_deref().expect("a tree binary")));
                    }
                }
            }
            if let Some(layer) = layer.as_str() {
                if !layers.contains(layer) {
                    findings.push(format!("{line}: unknown layer {layer}"));
                }
            } else {
                findings.push(format!("{line}: layer must be a string"));
            }
        }
    }
    for (id, (tree, by_rule)) in &binaries {
        if !by_rule && !seen.contains(id.as_str()) {
            findings.push(format!("{id}: missing line under [{}]", tree.as_deref().expect("a tree binary")));
        }
    }
    findings.sort();
    Ok(findings)
}

fn fixture(manifest: &str, name: &str, kinds: &[&str], test: bool) -> String {
    serde_json::json!({
        "workspace_root": "/workspace", "workspace_members": ["demo"],
        "packages": [{
            "id": "demo", "name": "demo", "manifest_path": format!("/workspace/{manifest}/Cargo.toml"),
            "targets": [{"name": name, "kind": kinds, "test": test, "src_path": "/workspace/source.rs"}]
        }]
    })
    .to_string()
}

#[test]
fn rules_on_fixtures() {
    let world = fixture("skein/crates/demo", "world", &["test"], true);
    let library = fixture("skein/crates/demo", "demo", &["rlib", "cdylib"], true);
    let binary = fixture("skein/crates/demo", "x", &["bin"], true);
    let repository = fixture("tests/demo", "world", &["test"], true);
    let example = fixture("skein/crates/demo", "x", &["example"], true);
    let bench = fixture("skein/crates/demo", "x", &["bench"], true);
    let disabled = fixture("skein/crates/demo", "world", &["test"], false);
    let two = serde_json::json!({
        "workspace_root": "/workspace", "workspace_members": ["demo", "another"],
        "packages": [
            {"id": "demo", "name": "demo", "manifest_path": "/workspace/skein/crates/demo/Cargo.toml",
             "targets": [{"name": "world", "kind": ["test"], "test": true, "src_path": "/workspace/source.rs"}]},
            {"id": "another", "name": "another", "manifest_path": "/workspace/jig/crates/another/Cargo.toml",
             "targets": [{"name": "world", "kind": ["test"], "test": true, "src_path": "/workspace/other.rs"}]}
        ]
    })
    .to_string();
    for (metadata, table, expected) in [
        (&two, "", vec!["another::world: missing line under [jig]", "demo::world: missing line under [skein]"]),
        (&world, "[skein]\n", vec!["demo::world: missing line under [skein]"]),
        (&world, "", vec!["demo::world: missing line under [skein]"]),
        (&world, "[skein]\n\"demo::world\" = 'domain'", vec![]),
        (&world, "[skein]\n\"demo::world\" = 'worlds'", vec!["[skein] demo::world: unknown layer worlds"]),
        (&world, "[jig]\n\"demo::world\" = 'domain'", vec!["[jig] demo::world: belongs under [skein]"]),
        (
            &world,
            "[skein]\n\"demo::gone\" = 'domain'",
            vec!["[skein] demo::gone: no test binary in the workspace", "demo::world: missing line under [skein]"],
        ),
        (&library, "", vec![]),
        (&library, "[skein]\ndemo = 'step'", vec!["[skein] demo: placed by rule; remove the line"]),
        (&binary, "", vec![]),
        (&binary, "[skein]\n\"demo::bin/x\" = 'step'", vec!["[skein] demo::bin/x: placed by rule; remove the line"]),
        (&repository, "", vec![]),
        (
            &repository,
            "[skein]\n\"demo::world\" = 'step'",
            vec!["[skein] demo::world: placed by rule; remove the line"],
        ),
        (&disabled, "", vec![]),
        (
            &disabled,
            "[skein]\n\"demo::world\" = 'domain'",
            vec!["[skein] demo::world: no test binary in the workspace"],
        ),
        (&example, "[skein]\n\"demo::example/x\" = 'domain'", vec![]),
        (&example, "", vec!["demo::example/x: missing line under [skein]"]),
        (&bench, "[skein]\n\"demo::bench/x\" = 'domain'", vec![]),
        (&fixture("skein/tests/demo", "world", &["test"], true), "", vec!["demo::world: missing line under [skein]"]),
        (&library, "[other]", vec!["[other]: not a tree table"]),
        (&world, "[other]\n\"demo::world\" = 'domain'", vec!["[other]: not a tree table"]),
        (&library, "extra = 7", vec!["extra: expected a tree table; only layers and tree tables belong at the top"]),
        (&library, "skein = 7", vec!["skein: expected a tree table; only layers and tree tables belong at the top"]),
        (&world, "[skein]\n\"demo::world\" = 7", vec!["[skein] demo::world: layer must be a string"]),
    ] {
        let table = format!("layers = ['step', 'domain']\n{table}");
        assert_eq!(findings(metadata, &table).expect("fixture metadata reads"), expected, "{metadata}\n{table}");
    }
    for (metadata, table, expected) in [
        (&library, "", vec!["layers: expected a list of layer names"]),
        (&library, "layers = 'step'", vec!["layers: expected a list of layer names"]),
        (&library, "layers = ['step', 'step']", vec!["layers: repeated name step"]),
        (&library, "layers = ['domain']", vec!["layers: missing step"]),
        (&library, "layers = []", vec!["layers: missing step"]),
        (&library, "layers = ['step', 7]", vec!["layers: every name must be a string"]),
        (&world, "layers = ['step', 'custom']\n[skein]\n\"demo::world\" = 'custom'", vec![]),
        (
            &world,
            "layers = ['domain']\n[jig]\n\"demo::world\" = 'worlds'",
            vec![
                "[jig] demo::world: belongs under [skein]",
                "[jig] demo::world: unknown layer worlds",
                "layers: missing step",
            ],
        ),
    ] {
        assert_eq!(findings(metadata, table).expect("fixture metadata reads"), expected, "{table}");
    }
}

#[test]
fn workspace_binaries_have_layers() {
    let root = skein_scratch::paths::workspace();
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(&root)
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let metadata = String::from_utf8(output.stdout).expect("cargo metadata is UTF-8");
    let table = fs::read_to_string(root.join("tools/tiers/layers.toml")).expect("the layer table reads");
    let findings = findings(&metadata, &table).expect("workspace metadata reads");
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}
