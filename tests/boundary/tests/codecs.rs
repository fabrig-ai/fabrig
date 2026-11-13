//! Codec crate dependencies (tools/crates/design.md, 2.2 and 3.2).
//! Cargo metadata selects workspace packages and their normal dependencies under
//! every target. Packages holding generated records depend on no workspace crate
//! but skein-lib, skein-codec and the allowances below. Glue alone is free.
//! Allowances and exceptions only shrink: unmatched entries fail the check too.
//! Metadata or a generated directory the check cannot read always fails it.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use skein_scratch::Scratch;

struct Allowance {
    package: &'static str,
    dependency: &'static str,
    reason: &'static str,
}

const ALLOWED: &[Allowance] = &[
    Allowance {
        package: "jig-protocol-worker-wire",
        dependency: "skein-channel",
        reason: "it declares its channel's kinds and schema in skein-channel's vocabulary beside its records, and skein-channel depends on skein-lib alone",
    },
    Allowance { package: "smith-channel", dependency: "skein-channel", reason: "the same, for smith's channel" },
    Allowance {
        package: "jig-protocol-worker-wire",
        dependency: "smith-channel",
        reason: "protocol/channel/wires.md, 2: spoken() states smith's channel identity",
    },
    Allowance {
        package: "jig-protocol-worker-wire",
        dependency: "smith-charter",
        reason: "protocol/channel/wires.md, 2: spoken() states smith's charter identity",
    },
    Allowance {
        package: "jig-protocol-worker-wire",
        dependency: "smith-transcript",
        reason: "protocol/channel/wires.md, 2: spoken() states smith's transcript identity",
    },
];

struct Exception {
    what: &'static str,
    until: &'static str,
}

const EXCEPTIONS: &[Exception] = &[Exception { what: "smith-local-protocol", until: "G3" }];

#[derive(Debug)]
struct Package {
    name: String,
    records: bool,
    dependencies: BTreeSet<String>,
    failures: Vec<String>,
}

fn packages(metadata: &str) -> Result<Vec<Package>, String> {
    let metadata: Value = serde_json::from_str(metadata).map_err(|why| format!("metadata: {why}"))?;
    let members = metadata["workspace_members"].as_array().ok_or("metadata: no workspace_members array")?;
    let members = members
        .iter()
        .map(|member| member.as_str().ok_or("metadata: invalid workspace member"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for package in metadata["packages"].as_array().ok_or("metadata: no packages array")? {
        let id = package["id"].as_str().ok_or("metadata: no package id")?;
        if !members.contains(id) {
            continue;
        }
        seen.insert(id);
        let name = package["name"].as_str().ok_or("metadata: no package name")?;
        let manifest = package["manifest_path"].as_str().ok_or_else(|| format!("{name}: no manifest_path"))?;
        let directory = Path::new(manifest).parent().ok_or_else(|| format!("{name}: no manifest directory"))?;
        let mut dependencies = BTreeSet::new();
        for dependency in package["dependencies"].as_array().ok_or_else(|| format!("{name}: no dependencies array"))? {
            let dependency_name = dependency["name"].as_str().ok_or_else(|| format!("{name}: no dependency name"))?;
            match dependency.get("kind") {
                Some(Value::Null) => {
                    dependencies.insert(dependency_name.to_owned());
                }
                Some(Value::String(kind)) if kind == "dev" || kind == "build" => (),
                Some(_) | None => return Err(format!("{name}: invalid dependency kind for {dependency_name}")),
            }
        }
        let mut package = Package { name: name.to_owned(), records: false, dependencies, failures: Vec::new() };
        generated(&directory.join("src/generated"), &mut package);
        result.push(package);
    }
    if seen != members {
        return Err("metadata: workspace member missing from packages".to_owned());
    }
    Ok(result)
}

fn generated(directory: &Path, package: &mut Package) {
    match fs::symlink_metadata(directory) {
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return,
        Err(why) => {
            package.failures.push(format!("{}: {why}", directory.display()));
            return;
        }
        Ok(_) => (),
    }
    match fs::read_dir(directory) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if record_family(&path).is_some() {
                            match fs::metadata(&path) {
                                Ok(metadata) => package.records |= metadata.is_file(),
                                Err(why) => package.failures.push(format!("{}: {why}", path.display())),
                            }
                        }
                    }
                    Err(why) => package.failures.push(format!("{}: {why}", directory.display())),
                }
            }
        }
        Err(why) => package.failures.push(format!("{}: {why}", directory.display())),
    }
}

/// Generated records, not glue (tools/crates/design.md, 2.2).
fn record_family(path: &Path) -> Option<&str> {
    let stem = path.file_name()?.to_str()?.strip_suffix(".rs")?;
    let (family, version) = if let Some(version) = stem
        .strip_prefix('v')
        .filter(|version| !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit()))
    {
        ("(own)", version)
    } else {
        let (family, version) = stem.rsplit_once("_v")?;
        if family.is_empty()
            || !family.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return None;
        }
        (family, version)
    };
    (!version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())).then_some(family)
}

fn findings(packages: &[Package], allowed: &[Allowance], exceptions: &[Exception]) -> Vec<String> {
    let members: BTreeSet<_> = packages.iter().map(|package| package.name.as_str()).collect();
    let mut edges = BTreeSet::new();
    let mut failures = Vec::new();
    for package in packages {
        failures.extend(package.failures.iter().map(|why| format!("{}: {why}", package.name)));
        if package.records {
            for dependency in &package.dependencies {
                if members.contains(dependency.as_str()) && !["skein-lib", "skein-codec"].contains(&dependency.as_str())
                {
                    edges.insert((package.name.as_str(), dependency.as_str()));
                }
            }
        }
    }
    let mut excepted = BTreeSet::new();
    for (package, dependency) in &edges {
        if allowed.iter().any(|allowance| allowance.package == *package && allowance.dependency == *dependency) {
            continue;
        }
        excepted.insert(*package);
        if !exceptions.iter().any(|exception| exception.what == *package) {
            failures.push(format!("{package}: normal dependency on workspace member {dependency}"));
        }
    }
    for allowance in allowed {
        if !edges.contains(&(allowance.package, allowance.dependency)) {
            failures.push(format!(
                "{} -> {} ({}): remove it: the check's allowances only shrink",
                allowance.package, allowance.dependency, allowance.reason
            ));
        }
    }
    for exception in exceptions {
        if !excepted.contains(exception.what) {
            failures.push(format!(
                "{} (until {}): remove it: the check's exceptions only shrink",
                exception.what, exception.until
            ));
        }
    }
    failures
}

#[test]
fn workspace_generated_records_have_only_codec_dependencies() {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let packages = packages(&String::from_utf8(output.stdout).expect("cargo metadata is UTF-8"))
        .expect("workspace metadata reads");
    for finding in findings(&packages, ALLOWED, &[]) {
        eprintln!("{finding}");
    }
    let failures = findings(&packages, ALLOWED, EXCEPTIONS);
    assert!(failures.is_empty(), "Codec findings:\n{}", failures.join("\n"));
}

fn scratch(files: &[&str]) -> Scratch {
    let scratch = Scratch::new("boundary-codecs");
    for name in files {
        let path = scratch.path().join(name);
        fs::create_dir_all(path.parent().expect("fixture directory")).expect("create fixture directory");
        fs::write(path, "").expect("write fixture source");
    }
    scratch
}

fn fixture_metadata(scratch: &Scratch, dependencies: &Value) -> Value {
    let packages: Vec<_> = ["a", "b", "c", "skein-lib", "skein-codec"]
        .into_iter()
        .map(|name| {
            serde_json::json!({
                "id": name, "name": name,
                "manifest_path": scratch.path().join(name).join("Cargo.toml"),
                "dependencies": if name == "a" { dependencies.clone() } else { serde_json::json!([]) }
            })
        })
        .collect();
    serde_json::json!({"workspace_members": ["a", "b", "c", "skein-lib", "skein-codec"], "packages": packages})
}

#[test]
fn normal_workspace_dependencies_use_package_names() {
    let scratch = scratch(&["a/src/generated/people_v1.rs"]);
    let mut metadata = fixture_metadata(
        &scratch,
        &serde_json::json!([
            {"name": "skein-lib", "kind": null},
            {"name": "skein-codec", "kind": null},
            {"name": "b", "kind": null, "rename": "serde"},
            {"name": "serde", "kind": null, "rename": "c"}
        ]),
    );
    metadata["packages"].as_array_mut().expect("fixture packages").push(serde_json::json!({"id": "external"}));
    let packages = packages(&metadata.to_string()).expect("fixture metadata");
    assert_eq!(findings(&packages, &[], &[]), ["a: normal dependency on workspace member b"]);
}

#[test]
fn only_normal_dependencies_count_under_any_target() {
    let scratch = scratch(&["a/src/generated/people_v1.rs"]);
    for (kind, target, expected) in [
        (serde_json::json!("dev"), Value::Null, vec![]),
        (serde_json::json!("build"), Value::Null, vec![]),
        (Value::Null, serde_json::json!("cfg(unix)"), vec!["a: normal dependency on workspace member b"]),
    ] {
        let metadata = fixture_metadata(&scratch, &serde_json::json!([{"name": "b", "kind": kind, "target": target}]));
        let packages = packages(&metadata.to_string()).expect("fixture metadata");
        assert_eq!(findings(&packages, &[], &[]), expected, "kind {kind}, target {target}");
    }
}

#[test]
fn records_not_glue_make_a_codec() {
    for (file, record) in [
        ("people_v1.rs", true),
        ("v12.rs", true),
        ("team_2_v03.rs", true),
        ("layouts.rs", false),
        ("People_v1.rs", false),
        ("people_v.rs", false),
        ("people_v1.txt", false),
        ("v.rs", false),
        ("_v1.rs", false),
    ] {
        let scratch = scratch(&[&format!("a/src/generated/{file}")]);
        let metadata = fixture_metadata(&scratch, &serde_json::json!([{"name": "b", "kind": null}]));
        let packages = packages(&metadata.to_string()).expect("fixture metadata");
        let expected = if record { vec!["a: normal dependency on workspace member b"] } else { vec![] };
        assert_eq!(findings(&packages, &[], &[]), expected, "{file}");
    }
    let scratch = scratch(&[]);
    fs::create_dir_all(scratch.path().join("a/src/generated/people_v1.rs")).expect("record-shaped directory");
    let metadata = fixture_metadata(&scratch, &serde_json::json!([{"name": "b", "kind": null}]));
    assert_eq!(findings(&packages(&metadata.to_string()).expect("fixture metadata"), &[], &[]), Vec::<String>::new());
}

#[test]
fn allowances_match_dependencies_and_only_shrink() {
    let scratch = scratch(&["a/src/generated/people_v1.rs"]);
    let allowed = [Allowance { package: "a", dependency: "b", reason: "fixture channel vocabulary" }];
    let metadata = fixture_metadata(&scratch, &serde_json::json!([{"name": "b", "kind": null}]));
    let packages = packages(&metadata.to_string()).expect("fixture metadata");
    assert_eq!(findings(&packages, &allowed, &[]), Vec::<String>::new());
    assert_eq!(
        findings(&packages, &[Allowance { package: "a", dependency: "c", reason: "different dependency" }], &[]),
        [
            "a: normal dependency on workspace member b",
            "a -> c (different dependency): remove it: the check's allowances only shrink"
        ]
    );
    assert_eq!(
        findings(&packages, &allowed, &[Exception { what: "a", until: "split" }]),
        ["a (until split): remove it: the check's exceptions only shrink"]
    );
    for dependencies in [serde_json::json!([]), serde_json::json!([{"name": "b", "kind": "dev"}])] {
        let metadata = fixture_metadata(&scratch, &dependencies);
        let packages = self::packages(&metadata.to_string()).expect("fixture metadata");
        assert_eq!(
            findings(&packages, &allowed, &[]),
            ["a -> b (fixture channel vocabulary): remove it: the check's allowances only shrink"]
        );
    }
}

#[test]
fn exceptions_excuse_every_package_finding_and_only_shrink() {
    let scratch = scratch(&["a/src/generated/people_v1.rs"]);
    let exceptions = [Exception { what: "a", until: "G3" }];
    let metadata = fixture_metadata(
        &scratch,
        &serde_json::json!([
            {"name": "b", "kind": null}, {"name": "c", "kind": null},
            {"name": "b", "kind": null, "target": "cfg(unix)"}
        ]),
    );
    let packages = packages(&metadata.to_string()).expect("fixture metadata");
    assert_eq!(
        findings(&packages, &[], &[]),
        ["a: normal dependency on workspace member b", "a: normal dependency on workspace member c"]
    );
    assert_eq!(findings(&packages, &[], &exceptions), Vec::<String>::new());
    let metadata = fixture_metadata(&scratch, &serde_json::json!([]));
    let packages = self::packages(&metadata.to_string()).expect("fixture metadata");
    assert_eq!(findings(&packages, &[], &exceptions), ["a (until G3): remove it: the check's exceptions only shrink"]);
    assert_eq!(
        findings(&packages, &[], &[Exception { what: "gone", until: "split" }]),
        ["gone (until split): remove it: the check's exceptions only shrink"]
    );
}

#[test]
fn metadata_requires_readable_fields() {
    let scratch = scratch(&[]);
    assert!(packages("not JSON").expect_err("invalid JSON").contains("metadata"));
    for (pointer, value, why) in [
        ("/workspace_members", Value::Null, "workspace_members"),
        ("/workspace_members/0", Value::Null, "workspace member"),
        ("/packages", Value::Null, "packages"),
        ("/packages/0/id", Value::Null, "package id"),
        ("/packages/0/id", serde_json::json!("not-a-member"), "workspace member missing"),
        ("/packages/0/name", Value::Null, "package name"),
        ("/packages/0/manifest_path", Value::Null, "manifest_path"),
        ("/packages/0/dependencies", Value::Null, "dependencies"),
        ("/packages/0/dependencies/0/name", Value::Null, "dependency name"),
        ("/packages/0/dependencies/0/kind", serde_json::json!(false), "dependency kind"),
    ] {
        let mut metadata = fixture_metadata(&scratch, &serde_json::json!([{"name": "b", "kind": null}]));
        *metadata.pointer_mut(pointer).expect("fixture field") = value;
        let failure = packages(&metadata.to_string()).expect_err("unreadable metadata");
        assert!(failure.contains(why), "{pointer}: {failure}");
    }
}

#[test]
fn unreadable_generated_directories_are_never_excused() {
    let scratch = scratch(&["a/src/generated", "b/src/generated/people_v1.rs"]);
    let mut metadata = fixture_metadata(&scratch, &serde_json::json!([]));
    metadata["packages"][1]["dependencies"] = serde_json::json!([{"name": "a", "kind": null}]);
    let packages = packages(&metadata.to_string()).expect("fixture metadata");
    let failures = findings(&packages, &[], &[Exception { what: "b", until: "G3" }]);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].starts_with("a: ") && failures[0].contains("src/generated"), "{failures:?}");
    let failures = findings(&packages, &[], &[Exception { what: "a", until: "G3" }]);
    assert_eq!(failures.len(), 3, "{failures:?}");
    assert!(failures[0].starts_with("a: ") && failures[0].contains("src/generated"), "{failures:?}");
    assert_eq!(failures[1], "b: normal dependency on workspace member a");
    assert_eq!(failures[2], "a (until G3): remove it: the check's exceptions only shrink");
}
