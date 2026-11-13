//! External entries carry their resolved feature union (tools/crates/design.md, 3.3).
//! Each of the three fixed contexts includes dev-dependencies and every occurrence,
//! including the host's build scripts and procedural macros. Members inherit one
//! request per entry; rustls's two entries are named only on their platforms.
//! Cargo's root settings unify the workspace, including crates below ours.
//! There are no exceptions. Unreadable metadata, tree output, packages or config fail.

use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use serde_json::Value;

type Features = BTreeSet<String>;
type FeatureTable = BTreeMap<String, Features>;

const CONTEXTS: &[&str] = &["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "wasm32-unknown-unknown"];
const PLATFORM_ENTRIES: &[(&str, &[&str])] =
    &[("rustls", &["wasm32-unknown-unknown"]), ("rustls-std", &["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"])];

#[derive(Debug, PartialEq, Eq)]
struct Request {
    features: Features,
    defaults: bool,
}

struct Package {
    name: String,
    version: String,
    features: FeatureTable,
}

struct Entry {
    member: String,
    name: String,
    package: String,
    request: Request,
}

struct Metadata {
    packages: BTreeMap<String, Package>,
    entries: Vec<Entry>,
}

struct Occurrence {
    package: String,
    features: Features,
}

fn strings(value: &Value) -> Result<Features, String> {
    value
        .as_array()
        .ok_or("expected a string array")?
        .iter()
        .map(|item| item.as_str().map(str::to_owned).ok_or_else(|| "expected a string".to_owned()))
        .collect()
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key].as_str().ok_or_else(|| format!("metadata: missing or invalid {key}"))
}

fn packages(metadata: &Value) -> Result<BTreeMap<String, Package>, String> {
    let mut result = BTreeMap::new();
    for package in metadata["packages"].as_array().ok_or("metadata: no packages array")? {
        let id = field(package, "id")?;
        let features = package["features"]
            .as_object()
            .ok_or_else(|| format!("{id}: no feature table"))?
            .iter()
            .map(|(name, values)| Ok((name.clone(), strings(values)?)))
            .collect::<Result<_, String>>()?;
        let package = Package {
            name: field(package, "name")?.to_owned(),
            version: field(package, "version")?.to_owned(),
            features,
        };
        if result.insert(id.to_owned(), package).is_some() {
            return Err(format!("metadata: duplicate package {id}"));
        }
    }
    Ok(result)
}

fn package<'a>(packages: &'a BTreeMap<String, Package>, id: &str) -> Result<&'a Package, String> {
    packages.get(id).ok_or_else(|| format!("metadata does not hold package {id}"))
}

fn record_entries(
    member: &Value,
    node: &Value,
    packages: &BTreeMap<String, Package>,
    members: &Features,
) -> Result<Vec<Entry>, String> {
    let mut result = Vec::new();
    for dependency in node["deps"].as_array().ok_or("metadata: no node deps array")? {
        let id = field(dependency, "pkg")?;
        let resolved = package(packages, id)?;
        if members.contains(id) {
            continue;
        }
        let code_name = field(dependency, "name")?;
        let kinds = dependency["dep_kinds"].as_array().ok_or("metadata: no dep_kinds array")?;
        let mut found = false;
        for record in member["dependencies"].as_array().ok_or("metadata: no dependencies array")? {
            let name = match record.get("rename") {
                Some(Value::String(name)) => name.as_str(),
                Some(Value::Null) => field(record, "name")?,
                Some(_) | None => return Err("metadata: invalid dependency rename".to_owned()),
            };
            if name.replace('-', "_") != code_name || field(record, "name")? != resolved.name {
                continue;
            }
            if !kinds
                .iter()
                .any(|kind| kind.get("kind") == record.get("kind") && kind.get("target") == record.get("target"))
            {
                continue;
            }
            result.push(Entry {
                member: field(member, "name")?.to_owned(),
                name: name.to_owned(),
                package: id.to_owned(),
                request: Request {
                    features: strings(&record["features"])?,
                    defaults: record["uses_default_features"]
                        .as_bool()
                        .ok_or("metadata: invalid uses_default_features")?,
                },
            });
            found = true;
        }
        if !found {
            return Err(format!("{}: no dependency record for {code_name} ({id})", field(member, "name")?));
        }
    }
    Ok(result)
}

fn metadata(text: &str) -> Result<Metadata, String> {
    let metadata: Value = serde_json::from_str(text).map_err(|why| format!("metadata: {why}"))?;
    let packages = packages(&metadata)?;
    let members = strings(&metadata["workspace_members"])?;
    let nodes = metadata["resolve"]["nodes"].as_array().ok_or("metadata: no resolve nodes")?;
    for node in nodes {
        package(&packages, field(node, "id")?)?;
        for dependency in node["deps"].as_array().ok_or("metadata: no node deps array")? {
            package(&packages, field(dependency, "pkg")?)?;
        }
    }
    let mut entries = Vec::new();
    for id in &members {
        package(&packages, id)?;
        let member = metadata["packages"]
            .as_array()
            .ok_or("metadata: no packages array")?
            .iter()
            .find(|member| member["id"].as_str() == Some(id))
            .ok_or_else(|| format!("metadata: no member {id}"))?;
        let node = nodes
            .iter()
            .find(|node| node["id"].as_str() == Some(id))
            .ok_or_else(|| format!("metadata: no resolve node for {id}"))?;
        entries.extend(record_entries(member, node, &packages, &members)?);
    }
    Ok(Metadata { packages, entries })
}

fn asked(request: &Request, table: &FeatureTable) -> Features {
    let mut pending: Vec<_> = request.features.iter().cloned().collect();
    if request.defaults {
        pending.push("default".to_owned());
    }
    let mut result = Features::new();
    while let Some(feature) = pending.pop() {
        if feature.starts_with("dep:") || feature.contains("?/") {
            continue;
        }
        let feature = if let Some((dependency, _)) = feature.split_once('/') {
            if !table.contains_key(dependency) {
                continue;
            }
            dependency.to_owned()
        } else {
            feature
        };
        if !result.insert(feature.clone()) {
            continue;
        }
        if let Some(values) = table.get(&feature) {
            pending.extend(values.iter().cloned());
        }
    }
    result
}

fn occurrences(text: &str, metadata: &Metadata) -> Result<Vec<Occurrence>, String> {
    let mut result = Vec::new();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let line = line.strip_suffix(" (*)").unwrap_or(line);
        let (printed, features) = line.split_once('|').ok_or_else(|| format!("tree: unreadable line {line:?}"))?;
        let (name, version) =
            printed.split_once(" v").ok_or_else(|| format!("tree: unreadable package {printed:?}"))?;
        let version = if let Some((version, source)) = version.split_once(" (") {
            if !source.ends_with(')') || source == ")" {
                return Err(format!("tree: unreadable source {printed:?}"));
            }
            version
        } else {
            version
        };
        let candidates: Vec<_> = metadata
            .packages
            .iter()
            .filter(|(_, package)| package.name == name && package.version == version)
            .collect();
        let [(id, package)] = candidates.as_slice() else {
            return Err(format!("tree: metadata does not hold a unique package {printed}"));
        };
        let features: Features =
            if features.is_empty() { Features::new() } else { features.split(',').map(str::to_owned).collect() };
        if !features.iter().all(|feature| package.features.contains_key(feature)) {
            return Err(format!("tree: unreadable features for {printed}: {features:?}"));
        }
        result.push(Occurrence { package: (*id).clone(), features });
    }
    if result.is_empty() {
        return Err("tree: no occurrences".to_owned());
    }
    Ok(result)
}

fn findings(context: &str, metadata: &Metadata, occurrences: &[Occurrence]) -> Vec<String> {
    let mut result = BTreeSet::new();
    let mut requests: BTreeMap<&str, (&str, &Request)> = BTreeMap::new();
    for entry in &metadata.entries {
        if let Some((member, request)) = requests.get(entry.name.as_str()) {
            if *request != &entry.request {
                result.insert(format!(
                    "{context}: entry {}: {member} and {} ask differently (features or defaults)",
                    entry.name, entry.member
                ));
            }
        } else {
            requests.insert(&entry.name, (&entry.member, &entry.request));
        }
        if PLATFORM_ENTRIES.iter().any(|(name, platforms)| *name == entry.name && !platforms.contains(&context)) {
            result.insert(format!("{context}: {} names entry {} outside its platforms", entry.member, entry.name));
        }
        let package = metadata.packages.get(&entry.package).expect("entry package was read");
        let asked = asked(&entry.request, &package.features);
        let missing: Features = occurrences
            .iter()
            .filter(|occurrence| occurrence.package == entry.package)
            .flat_map(|occurrence| occurrence.features.difference(&asked).cloned())
            .collect();
        if !missing.is_empty() {
            result.insert(format!(
                "{context}: entry {}: {} v{} does not ask for {}",
                entry.name,
                package.name,
                package.version,
                missing.into_iter().collect::<Vec<_>>().join(",")
            ));
        }
    }
    result.into_iter().collect()
}

fn settings_findings(text: &str) -> Vec<String> {
    let mut table = "";
    let mut settings = BTreeSet::new();
    for line in text.lines() {
        let line = line.split('#').next().expect("a split has at least one part").trim();
        if line.starts_with('[') {
            table = line;
        } else if let Some((key, value)) = line.split_once('=')
            && key.trim() == "feature-unification"
        {
            settings.insert((table, value.trim()));
        }
    }
    [("[unstable]", "true"), ("[resolver]", "\"workspace\"")]
        .into_iter()
        .filter(|setting| !settings.contains(setting))
        .map(|(table, value)| format!(".cargo/config.toml: missing {table} feature-unification = {value}"))
        .collect()
}

#[test]
fn settings_require_both_values_in_their_own_tables() {
    let unstable = ".cargo/config.toml: missing [unstable] feature-unification = true";
    let resolver = ".cargo/config.toml: missing [resolver] feature-unification = \"workspace\"";
    for (text, expected) in [
        (
            "# workspace unification\n\n [unstable] # nightly\n feature-unification = true # enabled\n[resolver]\nfeature-unification = \"workspace\"\n",
            vec![],
        ),
        ("[unstable]\n[resolver]\nfeature-unification = \"workspace\"\n", vec![unstable]),
        ("[unstable]\nfeature-unification = true\n[resolver]\nfeature-unification = \"selected\"\n", vec![resolver]),
        ("[other]\nfeature-unification = true\n[resolver]\nfeature-unification = \"workspace\"\n", vec![unstable]),
        ("[unstable]\nfeature-unification = true\n[other]\nfeature-unification = \"workspace\"\n", vec![resolver]),
        ("[unstable]\n# feature-unification = true\n[resolver]\nfeature-unification = \"workspace\"\n", vec![unstable]),
        ("[unstable]\nfeature-unification = true\n[resolver]\n# feature-unification = \"workspace\"\n", vec![resolver]),
    ] {
        assert_eq!(settings_findings(text), expected, "{text}");
    }
}

#[test]
fn workspace_settings_unify_every_member() {
    let path = skein_scratch::paths::workspace().join(".cargo/config.toml");
    let text = std::fs::read_to_string(&path).expect("read workspace .cargo/config.toml");
    let failures = settings_findings(&text);
    assert!(failures.is_empty(), "Feature settings findings:\n{}", failures.join("\n"));
}

fn cargo(args: &[&str]) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(skein_scratch::paths::workspace())
        .stdin(Stdio::null())
        .output()
        .map_err(|why| format!("cargo {args:?}: {why}"))?;
    if !output.status.success() {
        return Err(format!("cargo {args:?}: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr)));
    }
    String::from_utf8(output.stdout).map_err(|why| format!("cargo {args:?}: output is not UTF-8: {why}"))
}

struct Reading {
    context: &'static str,
    metadata: Metadata,
    occurrences: Vec<Occurrence>,
}

fn readings() -> &'static Result<Vec<Reading>, String> {
    static READINGS: OnceLock<Result<Vec<Reading>, String>> = OnceLock::new();
    READINGS.get_or_init(|| {
        CONTEXTS
            .iter()
            .map(|context| {
                let metadata = metadata(&cargo(&[
                    "metadata",
                    "--format-version",
                    "1",
                    "--offline",
                    "--locked",
                    "--filter-platform",
                    context,
                ])?)?;
                let tree = cargo(&[
                    "tree",
                    "--workspace",
                    "--offline",
                    "--locked",
                    "--target",
                    context,
                    "-e",
                    "normal,build,dev",
                    "--prefix",
                    "none",
                    "--format",
                    "{p}|{f}",
                ])?;
                let occurrences = occurrences(&tree, &metadata)?;
                Ok(Reading { context, metadata, occurrences })
            })
            .collect()
    })
}

#[test]
fn workspace_entries_carry_every_contexts_union() {
    let failures: Vec<_> = readings()
        .as_ref()
        .expect("workspace feature readings")
        .iter()
        .flat_map(|reading| findings(reading.context, &reading.metadata, &reading.occurrences))
        .collect();
    assert!(failures.is_empty(), "Feature findings:\n{}", failures.join("\n"));
}

fn serde_table() -> Value {
    serde_json::json!({
        "alloc": ["serde_core/alloc"], "default": ["std"], "derive": ["serde_derive"],
        "rc": ["serde_core/rc"], "serde_derive": ["dep:serde_derive"], "std": ["serde_core/std"],
        "unstable": ["serde_core/unstable"]
    })
}

fn sample(name: &str, rename: Option<&str>, defaults: bool, features: &[&str], table: &Value) -> Value {
    serde_json::json!({
        "workspace_members": ["member"],
        "packages": [
            {"id": "member", "name": "consumer", "version": "0.0.0", "features": {}, "dependencies": [
                {"name": name, "rename": rename, "features": features, "uses_default_features": defaults,
                 "kind": null, "target": "cfg(unix)"}
            ]},
            {"id": "external", "name": name, "version": "1.0.229", "features": table}
        ],
        "resolve": {"nodes": [
            {"id": "member", "deps": [{"name": rename.unwrap_or(name).replace('-', "_"), "pkg": "external",
                "dep_kinds": [{"kind": null, "target": "cfg(unix)"}]}]},
            {"id": "external", "deps": []}
        ]}
    })
}

fn sample_findings(context: &str, sample: &Value, tree: &str) -> Vec<String> {
    let metadata = metadata(&sample.to_string()).expect("sample metadata");
    let occurrences = occurrences(tree, &metadata).expect("sample tree");
    findings(context, &metadata, &occurrences)
}

#[test]
fn serde_request_closes_defaults_and_derive_but_not_alloc() {
    let sample = sample("serde", None, true, &["derive"], &serde_table());
    let metadata = metadata(&sample.to_string()).expect("serde metadata");
    let request = &metadata.entries.first().expect("serde entry").request;
    assert_eq!(
        asked(request, &metadata.packages["external"].features),
        Features::from_iter(["default", "std", "derive", "serde_derive"].map(str::to_owned))
    );
    assert_eq!(
        sample_findings(CONTEXTS[0], &sample, "serde v1.0.229|alloc,default,derive,serde_derive,std"),
        ["x86_64-unknown-linux-gnu: entry serde: serde v1.0.229 does not ask for alloc"]
    );
}

#[test]
fn no_defaults_and_dependency_feature_syntax_close_only_local_features() {
    let table = serde_json::json!({"default": ["std"], "std": [], "start": ["dep:hidden", "weak?/remote", "strong/remote", "absent/remote"], "strong": ["start"], "weak": [], "hidden": []});
    let sample = sample("syntax", None, false, &["start"], &table);
    let metadata = metadata(&sample.to_string()).expect("syntax metadata");
    assert_eq!(
        asked(&metadata.entries[0].request, &metadata.packages["external"].features),
        Features::from_iter(["start", "strong"].map(str::to_owned))
    );
}

#[test]
fn every_occurrence_counts_including_hosts_and_extra_requests_are_free() {
    let mut sample = sample("serde", None, true, &["derive"], &serde_table());
    let tree = "consumer v0.0.0 (/workspace/consumer)|\nserde v1.0.229|default,derive,serde_derive,std\nserde v1.0.229|alloc,default,derive,serde_derive,std\nserde v1.0.229|alloc,default,derive,serde_derive,std (*)\n";
    assert_eq!(
        sample_findings(CONTEXTS[0], &sample, tree),
        ["x86_64-unknown-linux-gnu: entry serde: serde v1.0.229 does not ask for alloc"]
    );
    sample["packages"][0]["dependencies"][0]["features"] = serde_json::json!(["derive", "alloc", "rc"]);
    assert_eq!(sample_findings(CONTEXTS[0], &sample, tree), Vec::<String>::new());
}

#[test]
fn members_must_ask_the_same_features_and_defaults() {
    let mut sample = sample("serde", None, true, &["derive"], &serde_table());
    let mut other = sample["packages"][0].clone();
    other["id"] = serde_json::json!("other");
    other["name"] = serde_json::json!("other-consumer");
    sample["packages"].as_array_mut().expect("packages").push(other);
    sample["workspace_members"].as_array_mut().expect("members").push(serde_json::json!("other"));
    let mut node = sample["resolve"]["nodes"][0].clone();
    node["id"] = serde_json::json!("other");
    sample["resolve"]["nodes"].as_array_mut().expect("nodes").push(node);
    for (features, defaults) in [(serde_json::json!(["derive", "alloc"]), true), (serde_json::json!(["derive"]), false)]
    {
        sample["packages"][2]["dependencies"][0]["features"] = features;
        sample["packages"][2]["dependencies"][0]["uses_default_features"] = serde_json::json!(defaults);
        assert_eq!(
            sample_findings(CONTEXTS[0], &sample, "serde v1.0.229|derive,serde_derive"),
            [
                "x86_64-unknown-linux-gnu: entry serde: consumer and other-consumer ask differently (features or defaults)"
            ]
        );
    }
}

#[test]
fn platform_entries_use_resolved_names_not_condition_text() {
    let native = sample("rustls", Some("rustls-std"), false, &["std"], &serde_json::json!({"std": []}));
    assert_eq!(
        sample_findings(CONTEXTS[2], &native, "rustls v1.0.229|std"),
        ["wasm32-unknown-unknown: consumer names entry rustls-std outside its platforms"]
    );
    let wasm = sample("rustls", None, false, &[], &serde_json::json!({}));
    for context in [CONTEXTS[0], CONTEXTS[1]] {
        assert_eq!(sample_findings(context, &native, "rustls v1.0.229|std"), Vec::<String>::new());
        assert_eq!(
            sample_findings(context, &wasm, "rustls v1.0.229|"),
            [format!("{context}: consumer names entry rustls outside its platforms")]
        );
    }
    assert_eq!(sample_findings(CONTEXTS[2], &wasm, "rustls v1.0.229|"), Vec::<String>::new());
}

#[test]
fn inactive_records_and_workspace_edges_carry_no_entry() {
    let mut sample = sample("serde", None, true, &["derive"], &serde_table());
    let mut inactive = sample["packages"][0]["dependencies"][0].clone();
    inactive["target"] = serde_json::json!("cfg(windows)");
    inactive["features"] = serde_json::json!(["alloc"]);
    sample["packages"][0]["dependencies"].as_array_mut().expect("records").push(inactive);
    sample["resolve"]["nodes"][0]["deps"]
        .as_array_mut()
        .expect("deps")
        .push(serde_json::json!({"name": "consumer", "pkg": "member", "dep_kinds": [{"kind": "dev", "target": null}]}));
    assert_eq!(
        sample_findings(CONTEXTS[0], &sample, "serde v1.0.229|default,derive,serde_derive,std"),
        Vec::<String>::new()
    );
    sample["resolve"]["nodes"][0]["deps"] = serde_json::json!([]);
    assert_eq!(sample_findings(CONTEXTS[0], &sample, "serde v1.0.229|alloc"), Vec::<String>::new());
}

#[test]
fn unreadable_sources_fail_instead_of_skipping_packages_or_lines() {
    assert!(metadata("{broken").err().expect("bad JSON").contains("metadata:"));
    let sample = sample("serde", None, true, &["derive"], &serde_table());
    let parsed = metadata(&sample.to_string()).expect("sample metadata");
    for tree in [
        "",
        "not a tree line",
        "serde v1.0.229|std\nnot a tree line",
        "serde v1.0.229|std garbage",
        "serde v1.0.229 (/unfinished|std",
    ] {
        assert!(occurrences(tree, &parsed).err().expect("bad tree").contains("tree:"));
    }
    assert!(occurrences("absent v1.0.229|", &parsed).err().expect("absent package").contains("absent"));
    let mut missing = sample.clone();
    missing["packages"].as_array_mut().expect("packages").pop();
    assert!(metadata(&missing.to_string()).err().expect("missing package").contains("external"));
    let mut missing = sample;
    missing["packages"][0]["dependencies"] = serde_json::json!([]);
    assert!(metadata(&missing.to_string()).err().expect("missing dependency record").contains("no dependency record"));
}
