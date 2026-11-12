//! The command line's layout, main.rs's size and no features
//! (skein/docs/foundation/cli.md, 2, 3 and 9, checks 1 to 3).
//! Every binary is its own package at <tree>/bin/<name>/main.rs, with no
//! library or other binary; its source is at most 30 lines. No member has
//! a [features] table or dependency feature switches; only external crates
//! in the root's [workspace.dependencies] may choose features.
//! Findings are excepted by name until their tree's round or temper's
//! milestone. Exceptions only shrink: an unmatched exception fails too.
//! Metadata, sources and manifest lines the check cannot read fail it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// What a check excepts, and the round or milestone that removes it
/// (skein/docs/foundation/cli.md, 9).
struct Exception {
    what: &'static str,
    until: &'static str,
}

const LAYOUT_EXCEPTIONS: &[Exception] = &[
    Exception { what: "forgejo-conformance forgejo-conformance", until: "skein's round" },
    Exception { what: "github-conformance github-conformance", until: "skein's round" },
    Exception { what: "hearth hearth", until: "hearth's round" },
    Exception { what: "jig-test jig-test", until: "jig's round" },
    Exception { what: "skein-codegen skein-codegen", until: "skein's round" },
    Exception { what: "skein-conformance-real process_fixture", until: "skein's round" },
    Exception { what: "skein-conformance-real signal_sender_fixture", until: "skein's round" },
    Exception { what: "skein-echo-shell skein-echo", until: "skein's round" },
    Exception { what: "skein-fail-stop-tests fail_stop_fixture", until: "skein's round" },
    Exception { what: "skein-io-world io_process_fixture", until: "skein's round" },
    Exception { what: "skein-throwaway-forgejo attached-peer", until: "skein's round" },
    Exception { what: "skein-ui-bundle skein-ui-bundle", until: "skein's round" },
    Exception { what: "skein-ui-example-server skein-ui-example", until: "skein's round" },
    Exception { what: "skein-ui-web-tests harness_fixture", until: "skein's round" },
    Exception { what: "skein-world-tests world_process_fixture", until: "skein's round" },
    Exception { what: "skein-world-tests world_terminal_fixture", until: "skein's round" },
    Exception { what: "smith smith", until: "smith's round" },
    Exception { what: "smith smith-agent", until: "smith's round" },
    Exception { what: "smith-bench smith-bench", until: "smith's round" },
    Exception { what: "smith-bench test-peer", until: "smith's round" },
    Exception { what: "smith-fake-claude fake-claude", until: "smith's round" },
    Exception { what: "smith-fake-claude fake-claude-server", until: "smith's round" },
];
const MAIN_EXCEPTIONS: &[Exception] = &[
    Exception { what: "forgejo-conformance forgejo-conformance", until: "skein's round" },
    Exception { what: "github-conformance github-conformance", until: "skein's round" },
    Exception { what: "hearth hearth", until: "hearth's round" },
    Exception { what: "skein-codegen skein-codegen", until: "skein's round" },
    Exception { what: "skein-conformance-real process_fixture", until: "skein's round" },
    Exception { what: "skein-conformance-real signal_sender_fixture", until: "skein's round" },
    Exception { what: "skein-echo-shell skein-echo", until: "skein's round" },
    Exception { what: "skein-fail-stop-tests fail_stop_fixture", until: "skein's round" },
    Exception { what: "skein-io-world io_process_fixture", until: "skein's round" },
    Exception { what: "skein-ui-bundle skein-ui-bundle", until: "skein's round" },
    Exception { what: "skein-ui-example-server skein-ui-example", until: "skein's round" },
    Exception { what: "skein-ui-web-tests harness_fixture", until: "skein's round" },
    Exception { what: "skein-world-tests world_process_fixture", until: "skein's round" },
    Exception { what: "skein-world-tests world_terminal_fixture", until: "skein's round" },
    Exception { what: "smith smith-agent", until: "smith's round" },
    Exception { what: "smith-bench smith-bench", until: "smith's round" },
    Exception { what: "smith-bench test-peer", until: "smith's round" },
];
const FEATURE_EXCEPTIONS: &[Exception] = &[
    Exception { what: "Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/crates/temper-fake/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/crates/temper-playground/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: [features]", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: jig-protocol-worker-agents", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: jig-protocol-worker-link", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: smith-channel", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: smith-host-domain", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: smith-host-protocol", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: smith-protocol-channel", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: smith-protocol-program", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: temper-worker-domain", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: temper-worker-domain-checkout", until: "cli C5" },
    Exception { what: "temper/crates/temper-service/Cargo.toml: temper-worker-protocol-git", until: "cli C5" },
    Exception { what: "temper/crates/temper-worker-shell/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/crates/temper-box-shell/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/testing/temper-github-app/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/client-browser/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/client/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/engine/domain/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/one-process/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/playground/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/real-loop/Cargo.toml: temper-service", until: "cli C5" },
    Exception { what: "temper/tests/system/Cargo.toml: temper-service", until: "cli C5" },
];

/// The findings no exception names, and the exceptions that name no
/// finding, as one failure message; `None` when both are empty.
fn judged(check: &str, findings: &[String], exceptions: &[Exception]) -> Option<String> {
    let mut failures = Vec::new();
    for finding in findings {
        if !exceptions.iter().any(|exception| exception.what == finding) {
            failures.push(format!("{check}: {finding}"));
        }
    }
    for exception in exceptions {
        if !findings.iter().any(|finding| finding == exception.what) {
            failures.push(format!(
                "{check}: {} (until {}): remove it: the check's exceptions only shrink",
                exception.what, exception.until
            ));
        }
    }
    (!failures.is_empty()).then(|| failures.join("\n"))
}

/// A bin target, and whether its package has no library and no other bin.
#[derive(Debug)]
struct Binary {
    package: String,
    name: String,
    path: String,
    alone: bool,
}

/// Cargo preserves `..` in target paths; resolve them without leaving the workspace.
fn source_path(root: &Path, source: &str) -> Result<String, String> {
    let relative =
        Path::new(source).strip_prefix(root).map_err(|why| format!("{source}: outside workspace_root: {why}"))?;
    let mut path = PathBuf::new();
    for part in relative.components() {
        match part {
            Component::Normal(part) => path.push(part),
            Component::ParentDir if path.pop() => (),
            Component::CurDir => (),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("{source}: not a source within workspace_root"));
            }
        }
    }
    if path.as_os_str().is_empty() {
        return Err(format!("{source}: not a source within workspace_root"));
    }
    Ok(path.to_string_lossy().into_owned())
}

fn binaries(metadata: &str) -> Result<Vec<Binary>, String> {
    let metadata: Value = serde_json::from_str(metadata).map_err(|why| format!("metadata: {why}"))?;
    let root = metadata["workspace_root"].as_str().ok_or("metadata: no workspace_root string")?;
    let root = Path::new(root);
    if !root.is_absolute() {
        return Err("metadata: workspace_root is not absolute".to_owned());
    }
    let packages = metadata["packages"].as_array().ok_or("metadata: no packages array")?;
    let mut binaries = Vec::new();
    for package in packages {
        let package_name = package["name"].as_str().ok_or("metadata: no package name string")?;
        let targets = package["targets"].as_array().ok_or_else(|| format!("{package_name}: no targets array"))?;
        let mut bins = Vec::new();
        let mut library = false;
        for target in targets {
            let name = target["name"].as_str().ok_or_else(|| format!("{package_name}: no target name string"))?;
            let context = format!("{package_name} {name}");
            let kind = target["kind"].as_array().ok_or_else(|| format!("{context}: no kind array"))?;
            let kind = kind
                .iter()
                .map(|kind| kind.as_str().ok_or_else(|| format!("{context}: a kind is not a string")))
                .collect::<Result<Vec<_>, _>>()?;
            let source = target["src_path"].as_str().ok_or_else(|| format!("{context}: no src_path string"))?;
            let path = source_path(root, source).map_err(|why| format!("{context}: {why}"))?;
            library |=
                kind.iter().any(|kind| ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"].contains(kind));
            if kind.contains(&"bin") {
                bins.push(Binary { package: package_name.to_owned(), name: name.to_owned(), path, alone: false });
            }
        }
        let alone = bins.len() == 1 && !library;
        for mut bin in bins {
            bin.alone = alone;
            binaries.push(bin);
        }
    }
    Ok(binaries)
}

fn workspace_binaries() -> Vec<Binary> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let metadata = String::from_utf8(output.stdout).expect("cargo metadata is UTF-8");
    binaries(&metadata).expect("the workspace's metadata reads")
}

fn layout_findings(binaries: &[Binary]) -> Vec<String> {
    binaries
        .iter()
        .filter(|binary| {
            !binary.alone
                || binary.package != binary.name
                || !["skein", "smith", "jig", "temper", "hearth"]
                    .iter()
                    .any(|tree| binary.path == format!("{tree}/bin/{}/main.rs", binary.name))
        })
        .map(|binary| format!("{} {}", binary.package, binary.name))
        .collect()
}

fn main_findings<F>(binaries: &[Binary], read: F) -> Result<Vec<String>, String>
where
    F: Fn(&Binary) -> Result<String, String>,
{
    let mut findings = Vec::new();
    for binary in binaries {
        let source =
            read(binary).map_err(|why| format!("{} {}: {}: {why}", binary.package, binary.name, binary.path))?;
        if source.lines().count() > 30 {
            findings.push(format!("{} {}", binary.package, binary.name));
        }
    }
    Ok(findings)
}

/// A manifest's tables with blank lines and comments left out, as in direction.rs.
fn tables(source: &str) -> Result<Vec<(&str, Vec<&str>)>, String> {
    let mut tables = vec![("", Vec::new())];
    for line in source.lines().map(without_comment).map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            let line = parts(line, '\0')?.first().copied().ok_or("no table header")?;
            tables.push((line.trim_matches(['[', ']']), Vec::new()));
        } else if let Some((_, lines)) = tables.last_mut() {
            lines.push(line);
        }
    }
    Ok(tables)
}

fn without_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (at, ch) in line.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if delimiter == '"' && ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch == '#' {
            return &line[..at];
        }
    }
    line
}

/// Split outside strings, arrays and inline tables. Unsupported or incomplete
/// expressions fail, so a dependency line is never silently left unread.
fn parts(text: &str, separator: char) -> Result<Vec<&str>, String> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut brackets = Vec::new();
    for (at, ch) in text.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if delimiter == '"' && ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            }
        } else if ch == '#' {
            if !brackets.is_empty() {
                return Err(format!("an expression the check does not read: {text}"));
            }
            result.push(text[start..at].trim());
            return Ok(result);
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch == '[' || ch == '{' {
            brackets.push(ch);
        } else if ch == ']' || ch == '}' {
            let expected = if ch == ']' { '[' } else { '{' };
            if brackets.pop() != Some(expected) {
                return Err(format!("an expression the check does not read: {text}"));
            }
        } else if ch == separator && brackets.is_empty() {
            result.push(text[start..at].trim());
            start = at + ch.len_utf8();
        }
    }
    if quote.is_some() || !brackets.is_empty() {
        return Err(format!("an expression the check does not read: {text}"));
    }
    result.push(text[start..].trim());
    Ok(result)
}

fn quoted(text: &str) -> Option<&str> {
    let quote = text.chars().next().filter(|quote| *quote == '"' || *quote == '\'')?;
    text.strip_prefix(quote)?.strip_suffix(quote).filter(|text| !text.contains([quote, '\\']))
}

fn key(text: &str) -> Result<&str, String> {
    let text = text.trim();
    let name = quoted(text).unwrap_or(text);
    if name.is_empty() || !name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
        return Err(format!("a key the check does not read: {text}"));
    }
    Ok(name)
}

/// The members the root manifest lists, without silently dropping an unread entry.
fn members(root: &str) -> Result<Vec<String>, String> {
    let (_, lines) =
        tables(root)?.into_iter().find(|(table, _)| *table == "workspace").ok_or("the root has no [workspace]")?;
    let start = lines
        .iter()
        .position(|line| line.split_once('=').is_some_and(|(name, _)| key(name) == Ok("members")))
        .ok_or("the root's [workspace] lists no members")?;
    let text = lines[start..].join("\n");
    let list = text
        .split_once('=')
        .and_then(|(_, rest)| rest.split_once('['))
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .ok_or("the root's [workspace] lists no members")?;
    parts(list, ',')?
        .into_iter()
        .filter(|item| !item.is_empty())
        .map(|item| quoted(item).map(str::to_owned).ok_or_else(|| format!("a member the check does not read: {item}")))
        .collect()
}

#[derive(Default)]
struct Dependency {
    path: bool,
    switches: bool,
}

fn attribute(dependency: &mut Dependency, line: &str) -> Result<(), String> {
    let (name, value) = line.split_once('=').ok_or_else(|| format!("an attribute the check does not read: {line}"))?;
    let name = key(name)?;
    if parts(value, ',')?.len() != 1 || value.trim().is_empty() {
        return Err(format!("an attribute the check does not read: {line}"));
    }
    dependency.path |= name == "path";
    dependency.switches |= ["optional", "default-features", "features"].contains(&name);
    Ok(())
}

fn dependency_line(dependencies: &mut BTreeMap<String, Dependency>, line: &str) -> Result<(), String> {
    let (name, value) = line.split_once('=').ok_or_else(|| format!("a dependency the check does not read: {line}"))?;
    let names = parts(name, '.')?;
    let name = key(names.first().ok_or("no dependency key")?)?;
    let dependency = dependencies.entry(name.to_owned()).or_default();
    if let [_, field] = names.as_slice() {
        return attribute(dependency, &format!("{field}={value}"));
    }
    if names.len() != 1 {
        return Err(format!("a dependency the check does not read: {line}"));
    }
    let values = parts(value, ',')?;
    let value = values.first().copied().ok_or("no dependency value")?;
    if values.len() != 1 {
        return Err(format!("a dependency the check does not read: {line}"));
    }
    if let Some(fields) = value.strip_prefix('{').and_then(|value| value.strip_suffix('}')) {
        for field in parts(fields, ',')?.into_iter().filter(|field| !field.is_empty()) {
            attribute(dependency, field)?;
        }
    } else if quoted(value).is_none() {
        return Err(format!("a dependency the check does not read: {line}"));
    }
    Ok(())
}

fn manifest_findings(manifest: &str, source: &str, root: bool) -> Result<Vec<String>, String> {
    let mut findings = BTreeSet::new();
    for (table, lines) in tables(source).map_err(|why| format!("{manifest}: {why}"))? {
        let names = parts(table, '.').map_err(|why| format!("{manifest}: {why}"))?;
        let names: Vec<_> = names.into_iter().map(|name| quoted(name).unwrap_or(name)).collect();
        if !root && names == ["features"] {
            findings.insert(format!("{manifest}: [features]"));
        }
        let dependency_at =
            names.iter().position(|name| ["dependencies", "dev-dependencies", "build-dependencies"].contains(name));
        let Some(at) = dependency_at else { continue };
        let prefix = &names[..at];
        let permitted = if root {
            prefix == ["workspace"]
        } else {
            prefix.is_empty() || (prefix.len() == 2 && prefix.first() == Some(&"target"))
        };
        if !permitted {
            return Err(format!("{manifest}: a table the check does not read: [{table}]"));
        }
        let mut dependencies = BTreeMap::new();
        let tail = &names[at + 1..];
        let read = match tail {
            [] => lines.into_iter().try_for_each(|line| dependency_line(&mut dependencies, line)),
            [name] => {
                let name = key(name).map_err(|why| format!("{manifest}: {why}"))?;
                let dependency = dependencies.entry(name.to_owned()).or_default();
                lines.into_iter().try_for_each(|line| attribute(dependency, line))
            }
            _ => Err(format!("a table the check does not read: [{table}]")),
        };
        read.map_err(|why| format!("{manifest}: {why}"))?;
        for (name, dependency) in dependencies {
            if dependency.switches && (!root || dependency.path) {
                findings.insert(format!("{manifest}: {name}"));
            }
        }
    }
    Ok(findings.into_iter().collect())
}

#[test]
fn every_binary_is_a_package_of_its_own() {
    let findings = layout_findings(&workspace_binaries());
    if let Some(failure) = judged("layout", &findings, LAYOUT_EXCEPTIONS) {
        panic!("{failure}");
    }
}

#[test]
fn every_main_is_plumbing() {
    let root = skein_scratch::paths::workspace();
    let findings = main_findings(&workspace_binaries(), |binary| {
        fs::read_to_string(root.join(&binary.path)).map_err(|why| why.to_string())
    })
    .expect("every binary's source is readable");
    if let Some(failure) = judged("main.rs size", &findings, MAIN_EXCEPTIONS) {
        panic!("{failure}");
    }
}

#[test]
fn no_workspace_crate_has_features() {
    let root = skein_scratch::paths::workspace();
    let source = fs::read_to_string(root.join("Cargo.toml")).expect("the root manifest is readable");
    let mut findings = manifest_findings("Cargo.toml", &source, true).expect("the root's dependencies read");
    for member in members(&source).expect("the root's members read") {
        let manifest = format!("{member}/Cargo.toml");
        let source = fs::read_to_string(root.join(&manifest)).unwrap_or_else(|why| panic!("{manifest}: {why}"));
        findings.extend(manifest_findings(&manifest, &source, false).unwrap_or_else(|why| panic!("{why}")));
    }
    findings.sort();
    if let Some(failure) = judged("no features", &findings, FEATURE_EXCEPTIONS) {
        panic!("{failure}");
    }
}

#[test]
fn exceptions_only_shrink() {
    let exception = || Exception { what: "demo first", until: "demo's round" };
    for (findings, exceptions, expected) in [
        (vec![], vec![], None),
        (vec!["demo first"], vec![exception()], None),
        (vec!["demo first", "demo second"], vec![], Some("demo: demo first\ndemo: demo second")),
        (
            vec![],
            vec![exception()],
            Some("demo: demo first (until demo's round): remove it: the check's exceptions only shrink"),
        ),
        (
            vec!["demo second"],
            vec![exception()],
            Some(
                "demo: demo second\ndemo: demo first (until demo's round): remove it: the check's exceptions only shrink",
            ),
        ),
    ] {
        let findings: Vec<_> = findings.into_iter().map(str::to_owned).collect();
        assert_eq!(judged("demo", &findings, &exceptions).as_deref(), expected);
    }
}

#[test]
fn layout_requires_the_named_package_path_and_no_library_or_other_binary() {
    for (package, path, extra, expected) in [
        ("demo", "/workspace/skein/bin/demo/main.rs", None, vec![]),
        ("demo", "/workspace/skein/bin/demo/../demo/main.rs", Some("test"), vec![]),
        ("other", "/workspace/skein/bin/demo/main.rs", None, vec!["other demo"]),
        ("demo", "/workspace/skein/bin/demo/src/main.rs", None, vec!["demo demo"]),
        ("demo", "/workspace/other/bin/demo/main.rs", None, vec!["demo demo"]),
        ("demo", "/workspace/skein/bin/demo/main.rs", Some("lib"), vec!["demo demo"]),
        ("demo", "/workspace/skein/bin/demo/main.rs", Some("cdylib"), vec!["demo demo"]),
        ("demo", "/workspace/skein/bin/demo/main.rs", Some("bin"), vec!["demo demo", "demo helper"]),
    ] {
        let mut targets = vec![serde_json::json!({ "name": "demo", "kind": ["bin"], "src_path": path })];
        if let Some(kind) = extra {
            targets.push(serde_json::json!({
                "name": "helper", "kind": [kind], "src_path": "/workspace/skein/bin/helper/main.rs"
            }));
        }
        let metadata = serde_json::json!({
            "workspace_root": "/workspace", "packages": [{ "name": package, "targets": targets }]
        });
        let binaries = binaries(&metadata.to_string()).expect("the fixture's metadata reads");
        assert_eq!(layout_findings(&binaries), expected, "{metadata}");
    }
}

#[test]
fn main_size_counts_all_lines_and_requires_readable_sources() {
    let binaries =
        [Binary { package: "demo".to_owned(), name: "demo".to_owned(), path: "demo/main.rs".to_owned(), alone: true }];
    for (source, expected) in [
        ("// comment\n".repeat(30), vec![]),
        ("\n".repeat(31), vec!["demo demo"]),
        (format!("{}last line", "line\n".repeat(30)), vec!["demo demo"]),
    ] {
        assert_eq!(main_findings(&binaries, |_| Ok(source.clone())).expect("readable"), expected);
    }
    assert_eq!(
        main_findings(&binaries, |_| Err("unreadable".to_owned())),
        Err("demo demo: demo/main.rs: unreadable".to_owned())
    );
}

#[test]
fn features_are_found_in_every_dependency_form_and_only_external_root_entries_are_exempt() {
    for (root, source, expected) in [
        (false, "[features]\n", vec!["[features]"]),
        (false, "['features'] # a table comment\n", vec!["[features]"]),
        (false, "[dependencies]\ndemo = { version = '1', optional = false }\n", vec!["demo"]),
        (false, "[dev-dependencies]\ndemo = { workspace = true, default-features = true }\n", vec!["demo"]),
        (false, "[build-dependencies]\n'demo' = { version = '1', features = [] }\n", vec!["demo"]),
        (false, "[target.'cfg(unix)'.dependencies]\ndemo = { features = ['one', 'two'] }\n", vec!["demo"]),
        (false, "[target.'cfg(target_os = \"a.b\")'.dev-dependencies.demo]\nfeatures = []\n", vec!["demo"]),
        (false, "[target.'cfg(unix)'.build-dependencies]\ndemo.features = []\n", vec!["demo"]),
        (false, "[dependencies.demo]\nworkspace = true\noptional = true\nfeatures = []\n", vec!["demo"]),
        (false, "[\"dependencies\"]\ndemo = { features = [] }\n", vec!["demo"]),
        (false, "[dependencies]\ndemo = '1'\nother.workspace = true\n", vec![]),
        (
            false,
            "[dependencies]\ndemo = { version = 'features # optional', package = 'features' } # features = []\n",
            vec![],
        ),
        (false, "[package.metadata]\nfeatures = []\noptional = true\n", vec![]),
        (
            true,
            "[workspace.dependencies]\ndemo = { version = '1', features = ['one', 'two'], optional = true }\n",
            vec![],
        ),
        (true, "[workspace.dependencies]\ndemo = { path = 'demo', features = [] }\n", vec!["demo"]),
        (true, "[workspace.dependencies]\ndemo = { path = 'demo', optional = false }\n", vec!["demo"]),
        (true, "[workspace.dependencies]\ndemo = { path = 'demo', default-features = true }\n", vec!["demo"]),
        (true, "[workspace.dependencies.demo]\nfeatures = []\npath = 'demo'\n", vec!["demo"]),
        (true, "[workspace.dependencies]\ndemo.features = []\ndemo.path = 'demo'\n", vec!["demo"]),
        (true, "[workspace.dependencies]\ndemo = { path = 'demo' }\n", vec![]),
    ] {
        let expected: Vec<_> = expected.into_iter().map(|finding| format!("Cargo.toml: {finding}")).collect();
        assert_eq!(manifest_findings("Cargo.toml", source, root).expect("the fixture reads"), expected, "{source}");
    }
}

#[test]
fn unreadable_dependency_lines_members_and_metadata_fail() {
    for source in [
        "[dependencies]\nnot a dependency\n",
        "[dependencies]\ndemo = false\n",
        "[dependencies]\ndemo = { features = [\n'one',\n] }\n",
        "[dependencies]\ndemo = { optional = }\n",
        "[dependencies]\ndemo = { features = ['one' }\n",
        "[dependencies]\ndemo = { features = ['one] }\n",
        "[dependencies]\n'demo.other' = { optional = true }\n",
        "[dependencies.demo.extra]\nfeatures = []\n",
        "[other.dependencies]\ndemo = { optional = true }\n",
    ] {
        let failure = manifest_findings("demo/Cargo.toml", source, false).expect_err("an unread line fails");
        assert!(failure.starts_with("demo/Cargo.toml:"), "{failure}");
        assert!(failure.contains("does not read"), "{failure}");
    }
    assert_eq!(
        members("[workspace] # comment\ndefault-members = ['other']\nmembers = [\n'demo', # comment\n'other',\n]\n")
            .expect("members read"),
        ["demo", "other"]
    );
    for source in ["", "[workspace]\n", "[workspace]\nmembers = ['demo', unread]\n"] {
        assert!(members(source).is_err(), "{source}");
    }
    for metadata in [
        "not JSON".to_owned(),
        serde_json::json!({ "workspace_root": "relative", "packages": [] }).to_string(),
        serde_json::json!({ "workspace_root": "/workspace", "packages": [{ "name": "demo" }] }).to_string(),
        serde_json::json!({
            "workspace_root": "/workspace", "packages": [{ "name": "demo", "targets": [
                { "name": "demo", "kind": [null], "src_path": "/workspace/skein/bin/demo/main.rs" }
            ] }]
        })
        .to_string(),
    ] {
        assert!(binaries(&metadata).is_err(), "{metadata}");
    }
    for source in ["/outside/main.rs", "/workspace/../main.rs", "/workspace"] {
        assert!(source_path(Path::new("/workspace"), source).is_err(), "{source}");
    }
}
