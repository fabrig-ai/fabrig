//! Every binary's `main` installs `skein_fail_stop`'s hook before anything else
//! (skein/docs/foundation/programming-model.md, 2): the dev profile unwinds, so a panic aborts
//! a process only once the hook is in. The check asks `cargo metadata` for the workspace's
//! bin targets, wherever cargo finds them, and reads each one's source: in each `fn main()`,
//! only white space, `//` comments and `use` and `const` items may come before
//! `skein_fail_stop::install();`. A source with no `fn main()` breaches it. A source it cannot
//! read and metadata it does not read are failures, never skips.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// A bin target: its package, its name and its source's path from the workspace's root.
#[derive(Debug, PartialEq, Eq)]
struct Binary {
    package: String,
    name: String,
    path: String,
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

/// The bin targets in `cargo metadata`'s output, in its order: a `path` from its
/// `workspace_root`; a package or target of another shape is an error.
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
            if kind.contains(&"bin") {
                binaries.push(Binary { package: package_name.to_owned(), name: name.to_owned(), path });
            }
        }
    }
    Ok(binaries)
}

/// Python's `\s`, including its four additional ASCII control separators.
fn whitespace(ch: char) -> bool {
    ch.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&ch)
}

/// The script's `(?:\s+|//[^\n]*|(?:use|const)\s[^;]*;)*` followed by the call.
/// The states coexist: a comment may end before its newline, as regex backtracking permits.
fn hook_first(mut source: &str) -> bool {
    let mut prefix = true;
    let mut comment = false;
    let mut item = false;
    while let Some(ch) = source.chars().next() {
        prefix |= comment;
        if prefix && source.starts_with("skein_fail_stop::install();") {
            return true;
        }
        if prefix && let Some(rest) = source.strip_prefix("//") {
            comment = true;
            prefix = false;
            source = rest;
            continue;
        }
        if prefix
            && ["use", "const"].iter().any(|keyword| {
                source.strip_prefix(keyword).and_then(|rest| rest.chars().next()).is_some_and(whitespace)
            })
        {
            item = true;
        }
        prefix = (prefix && whitespace(ch)) || (item && ch == ';');
        item &= ch != ';';
        comment &= ch != '\n';
        source = &source[ch.len_utf8()..];
    }
    false
}

/// Whether `source` has a `fn main()` and each one installs the hook first.
fn installs_first(mut source: &str) -> bool {
    let mut found = false;
    while let Some((before, after)) = source.split_once("fn main()") {
        source = after;
        if before.chars().next_back().is_some_and(|ch| ch.is_alphanumeric() || ch == '_') {
            continue;
        }
        let Some((_, body)) = after.split_once('{') else { break };
        found = true;
        if !hook_first(body) {
            return false;
        }
        source = body;
    }
    found
}

/// Each binary that breaches the rule or cannot be read: `PACKAGE BINARY: PATH: why`.
fn breaches<F>(binaries: &[Binary], read: F) -> Vec<String>
where
    F: Fn(&Binary) -> Result<String, String>,
{
    binaries
        .iter()
        .filter_map(|binary| {
            let why = match read(binary) {
                Ok(source) if installs_first(&source) => return None,
                Ok(_) => "main does not install skein_fail_stop's hook first".to_owned(),
                Err(why) => format!("cannot read source: {why}"),
            };
            Some(format!("{} {}: {}: {why}", binary.package, binary.name, binary.path))
        })
        .collect()
}

#[test]
fn every_binary_installs_the_hook_first() {
    let root = skein_scratch::paths::workspace();
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(&root)
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let metadata = String::from_utf8(output.stdout).expect("cargo metadata is UTF-8");
    let binaries = binaries(&metadata).expect("the workspace's metadata reads");
    for tree in ["skein", "smith", "jig", "temper", "hearth"] {
        assert!(
            binaries.iter().any(|binary| binary.path.starts_with(&format!("{tree}/"))),
            "the check reads {tree}'s binaries"
        );
    }
    let breaches =
        breaches(&binaries, |binary| fs::read_to_string(root.join(&binary.path)).map_err(|why| why.to_string()));
    assert!(
        breaches.is_empty(),
        "Each binary's main installs skein_fail_stop's hook first (skein/docs/foundation/programming-model.md, 2):\n{}",
        breaches.join("\n")
    );
}

#[test]
fn a_main_that_does_anything_first_is_a_breach() {
    for source in [
        "fn other() {}",
        "fn main() {}",
        "fn main() { let x = 1; skein_fail_stop::install(); }",
        "fn other() { skein_fail_stop::install(); } fn main() {}",
        "fn main() { skein_fail_stop::install(); } fn main() {}",
        "fn main() { /* comment */ skein_fail_stop::install(); }",
        "fn main() { useful(); skein_fail_stop::install(); }",
        "fn main() { constant(); skein_fail_stop::install(); }",
        "fn main() { use x }",
        "fn main () { skein_fail_stop::install(); }",
        "xfn main() { skein_fail_stop::install(); }",
        "_fn main() { skein_fail_stop::install(); }",
        "éfn main() { skein_fail_stop::install(); }",
        "fn main()",
        "// fn main() {}\nfn main() { skein_fail_stop::install(); }",
    ] {
        assert!(!installs_first(source), "{source}");
    }
}

#[test]
fn only_use_const_and_comments_may_come_first() {
    for source in [
        "fn main() { skein_fail_stop::install(); }",
        "fn main() -> ExitCode { skein_fail_stop::install(); }",
        "fn main() { use\n crate::{\n A, B\n }; skein_fail_stop::install(); }",
        "fn main() { const X: u8 = 1; skein_fail_stop::install(); }",
        "fn main() { // comment\n use x; // another\n const X: u8 = 1;\n skein_fail_stop::install(); }",
        "fn main() { skein_fail_stop::install(); } fn main() { skein_fail_stop::install(); }",
        "fn main() {\u{1c}\u{85}skein_fail_stop::install(); }",
        "fn main() { // skein_fail_stop::install();\n}",
        "fn main() { // use x\nanything; skein_fail_stop::install(); }",
        "fn main() // fn main()\n { skein_fail_stop::install(); }",
    ] {
        assert!(installs_first(source), "{source}");
    }
}

fn fixture() -> Vec<Binary> {
    vec![
        Binary { package: "fixture-a".to_owned(), name: "first".to_owned(), path: "skein/a/other.rs".to_owned() },
        Binary { package: "fixture-a".to_owned(), name: "second".to_owned(), path: "shared/main.rs".to_owned() },
        Binary { package: "fixture-b".to_owned(), name: "third".to_owned(), path: "shared/main.rs".to_owned() },
    ]
}

#[test]
fn a_breach_names_the_package_the_binary_and_the_path() {
    assert_eq!(
        breaches(&fixture(), |_| Ok("fn main() {}".to_owned())),
        [
            "fixture-a first: skein/a/other.rs: main does not install skein_fail_stop's hook first",
            "fixture-a second: shared/main.rs: main does not install skein_fail_stop's hook first",
            "fixture-b third: shared/main.rs: main does not install skein_fail_stop's hook first",
        ]
    );
}

#[test]
fn a_source_the_check_cannot_read_fails_it() {
    assert_eq!(
        breaches(&fixture(), |_| Err("unreadable".to_owned())),
        [
            "fixture-a first: skein/a/other.rs: cannot read source: unreadable",
            "fixture-a second: shared/main.rs: cannot read source: unreadable",
            "fixture-b third: shared/main.rs: cannot read source: unreadable",
        ]
    );
}

fn metadata_fixture() -> Value {
    serde_json::json!({
        "workspace_root": "/workspace",
        "packages": [
            { "name": "fixture-a", "targets": [
                { "name": "library", "kind": ["lib"], "src_path": "/workspace/skein/a/src/lib.rs" },
                { "name": "first", "kind": ["bin"], "src_path": "/workspace/skein/a/../a/other.rs" },
                { "name": "second", "kind": ["example", "bin"], "src_path": "/workspace/shared/main.rs" }
            ] },
            { "name": "fixture-b", "targets": [
                { "name": "third", "kind": ["bin"], "src_path": "/workspace/shared/main.rs" }
            ] }
        ]
    })
}

#[test]
fn metadata_lists_binaries_in_order_including_shared_and_nonstandard_paths() {
    assert_eq!(binaries(&metadata_fixture().to_string()).expect("the fixture reads"), fixture());
}

#[test]
fn metadata_the_check_does_not_read_fails_it() {
    let mut cases = vec![("not JSON".to_owned(), "metadata:")];
    for (pointer, replacement, why) in [
        ("/workspace_root", Value::Null, "no workspace_root string"),
        ("/workspace_root", Value::from("relative"), "workspace_root is not absolute"),
        ("", serde_json::json!({ "workspace_root": "/workspace" }), "no packages array"),
        ("/packages", Value::Null, "no packages array"),
        ("/packages/0", Value::Null, "no package name string"),
        ("/packages/0", serde_json::json!({ "name": "fixture-a" }), "fixture-a: no targets array"),
        ("/packages/0/targets", Value::Null, "fixture-a: no targets array"),
        ("/packages/0/targets/0", Value::Null, "fixture-a: no target name string"),
        ("/packages/0/targets/0", serde_json::json!({ "name": "library" }), "fixture-a library: no kind array"),
        ("/packages/0/targets/0/kind", Value::Null, "fixture-a library: no kind array"),
        ("/packages/0/targets/0/kind/0", Value::Null, "fixture-a library: a kind is not a string"),
        ("/packages/0/targets/0/src_path", Value::Null, "fixture-a library: no src_path string"),
        ("/packages/0/targets/1/src_path", Value::from("/elsewhere/main.rs"), "outside workspace_root"),
        ("/packages/0/targets/1/src_path", Value::from("/workspace/../main.rs"), "not a source within workspace_root"),
    ] {
        let mut metadata = metadata_fixture();
        *metadata.pointer_mut(pointer).expect("the fixture has the field") = replacement;
        cases.push((metadata.to_string(), why));
    }
    for (source, why) in cases {
        let failure = binaries(&source).expect_err("the metadata fails");
        assert!(failure.contains(why), "{source}: {failure}");
    }
}
