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

/// The binaries the slow tiers own by name, which a real pair names but its clause leaves out (section 9.2).
fn another_tiers(id: &str) -> bool {
    let name = id.split_once("::").map_or("", |(_, name)| name);
    ["browser_", "conformance_", "recorded_"].iter().any(|prefix| name.starts_with(prefix))
        || ["live", "live_github"].contains(&name)
}

fn real_packages(name: &str, lines: &BTreeSet<&str>, packages: &toml::Table, findings: &mut Vec<String>) {
    let Some(listed) = packages.get(name).and_then(toml::Value::as_array) else {
        findings.push(format!("tools/tiers/packages.toml: no {name} list"));
        return;
    };
    let listed: BTreeSet<&str> = listed.iter().filter_map(toml::Value::as_str).collect();
    let mut built = BTreeSet::new();
    for id in lines.iter().filter(|id| !another_tiers(id)) {
        let package = id.split_once("::").map_or(*id, |(package, _)| package);
        built.insert(package);
        if !listed.contains(package) {
            findings.push(format!("tools/tiers/packages.toml: {name} lacks {package}, which builds {id}"));
        }
    }
    for package in listed.difference(&built) {
        findings.push(format!("tools/tiers/packages.toml: {name} lists {package}, which builds no binary it runs"));
    }
}

/// Each pair's prerequisite binaries in the order the tier runner requires, sorted and unique by package then
/// binary (tools/tiers/design.md, 2.1), so a list the runner would refuse reddens the gate, not after landing.
fn binary_lists(packages: &toml::Table, findings: &mut Vec<String>) {
    let Some(lists) = packages.get("binaries").and_then(toml::Value::as_table) else { return };
    for (pair, list) in lists {
        let records = list.as_array().map(|list| {
            list.iter()
                .map(|record| {
                    let field = |name| record.get(name).and_then(toml::Value::as_str).unwrap_or_default();
                    (field("package"), field("bin"))
                })
                .collect::<Vec<_>>()
        });
        let Some(records) = records else {
            findings.push(format!("tools/tiers/packages.toml: [binaries] {pair} is not a list"));
            continue;
        };
        if !records.windows(2).all(|pair| pair[0] < pair[1]) {
            findings.push(format!("tools/tiers/packages.toml: [binaries] {pair} is not sorted by package then binary"));
        }
    }
}

fn real_profiles(table: &toml::Table, config: &str, packages: &str, findings: &mut Vec<String>) -> Result<(), String> {
    let config: toml::Table = toml::from_str(config).map_err(|why| format!("nextest configuration: {why}"))?;
    let packages: toml::Table = toml::from_str(packages).map_err(|why| format!("package map: {why}"))?;
    binary_lists(&packages, findings);
    let profiles = config.get("profile").and_then(toml::Value::as_table);
    let mut trees: BTreeSet<&str> = TREES.into_iter().collect();
    if let Some(profiles) = profiles {
        trees.extend(profiles.keys().filter_map(|name| name.strip_prefix("real-")));
    }
    for tree in trees {
        let expected: BTreeSet<&str> = table
            .get(tree)
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|lines| lines.iter())
            .filter(|(_, layer)| layer.as_str().is_some_and(|layer| ["real-loop", "end-to-end"].contains(&layer)))
            .map(|(id, _)| id.as_str())
            .collect();
        let name = format!("real-{tree}");
        let profile = profiles.and_then(|profiles| profiles.get(&name));
        if expected.is_empty() {
            if profile.is_some() {
                findings.push(format!("{tree}: [profile.{name}] has no real-loop or end-to-end lines"));
            }
            continue;
        }
        let Some(profile) = profile else {
            findings.push(format!("{tree}: missing [profile.{name}]"));
            continue;
        };
        let filter = profile.get("default-filter").and_then(toml::Value::as_str).unwrap_or_default();
        let named: BTreeSet<&str> = filter
            .split("binary_id(")
            .skip(1)
            .filter_map(|part| part.split_once(')'))
            .filter_map(|(id, _)| id.trim().strip_prefix('=').map(str::trim))
            .collect();
        for id in expected.difference(&named) {
            findings.push(format!("{tree}: [profile.{name}] missing binary_id(={id})"));
        }
        for id in named.difference(&expected) {
            findings.push(format!("{tree}: [profile.{name}] names {id}, not a real-loop or end-to-end line"));
        }
        real_packages(&name, &expected, &packages, findings);
    }
    Ok(())
}

fn findings(metadata: &str, table: &str, config: &str, packages: &str) -> Result<Vec<String>, String> {
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
    real_profiles(&table, config, packages, &mut findings)?;
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
        assert_eq!(
            findings(metadata, &table, "", "").expect("fixture metadata reads"),
            expected,
            "{metadata}\n{table}"
        );
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
        assert_eq!(findings(metadata, table, "", "").expect("fixture metadata reads"), expected, "{table}");
    }
}

#[test]
fn real_pair_rules_on_fixtures() {
    let world = fixture("skein/crates/demo", "world", &["test"], true);
    let real = "layers = ['step', 'real-loop', 'end-to-end']\n[skein]\n\"demo::world\" = 'real-loop'";
    let end_to_end = "layers = ['step', 'real-loop', 'end-to-end']\n[skein]\n\"demo::world\" = 'end-to-end'";
    let profile = "[profile.real-skein]\ndefault-filter = 'binary_id(=demo::world)'";
    let packages = "real-skein = ['demo']";
    for (table, config, expected) in [
        (real, profile, vec![]),
        (end_to_end, profile, vec![]),
        (end_to_end, "", vec!["skein: missing [profile.real-skein]"]),
        (
            real,
            "[profile.real-skein]\ndefault-filter = 'all()'",
            vec!["skein: [profile.real-skein] missing binary_id(=demo::world)"],
        ),
        (
            real,
            "[profile.real-skein]\ndefault-filter = 'binary_id(=demo::world) | binary_id(=extra::world)'",
            vec!["skein: [profile.real-skein] names extra::world, not a real-loop or end-to-end line"],
        ),
        (real, "", vec!["skein: missing [profile.real-skein]"]),
        (
            "layers = ['step', 'domain']\n[skein]\n\"demo::world\" = 'domain'",
            profile,
            vec!["skein: [profile.real-skein] has no real-loop or end-to-end lines"],
        ),
    ] {
        assert_eq!(
            findings(&world, table, config, packages).expect("fixture texts read"),
            expected,
            "{table}\n{config}"
        );
    }
    let browser = "layers = ['step', 'end-to-end']\n[skein]\n\"demo::world\" = 'end-to-end'\n\
                   \"view::browser_pages\" = 'end-to-end'";
    let both = "[profile.real-skein]\ndefault-filter = 'binary_id(=demo::world) | binary_id(=view::browser_pages)'";
    for (packages, expected) in [
        (packages, vec![]),
        (
            "real-skein = ['demo', 'view']",
            vec!["tools/tiers/packages.toml: real-skein lists view, which builds no binary it runs"],
        ),
        ("real-skein = []", vec!["tools/tiers/packages.toml: real-skein lacks demo, which builds demo::world"]),
        ("", vec!["tools/tiers/packages.toml: no real-skein list"]),
    ] {
        let found = findings(&world, browser, both, packages).expect("fixture texts read");
        let found: Vec<_> = found.into_iter().filter(|finding| finding.starts_with("tools/")).collect();
        assert_eq!(found, expected, "{packages}");
    }
    let sorted = "real-skein = ['demo', 'view']\n[binaries]\nreal-skein = [{ package = 'a', bin = 'x' }, { package = 'b', bin = 'y' }]";
    let unsorted = "real-skein = ['demo', 'view']\n[binaries]\nreal-skein = [{ package = 'b', bin = 'y' }, { package = 'a', bin = 'x' }]";
    for (packages, expected) in [
        (sorted, vec![]),
        (unsorted, vec!["tools/tiers/packages.toml: [binaries] real-skein is not sorted by package then binary"]),
    ] {
        let found = findings(&world, browser, both, packages).expect("fixture texts read");
        let found: Vec<_> = found.into_iter().filter(|finding| finding.contains("[binaries]")).collect();
        assert_eq!(found, expected, "{packages}");
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
    let config = fs::read_to_string(root.join(".config/nextest.toml")).expect("the nextest configuration reads");
    let packages = fs::read_to_string(root.join("tools/tiers/packages.toml")).expect("the package map reads");
    let findings = findings(&metadata, &table, &config, &packages).expect("workspace texts read");
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}
