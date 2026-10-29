//! The trees' dependencies point one way (AGENTS.md, 1): skein names none
//! of smith, jig or temper, smith names neither jig nor temper, and jig
//! never names temper. The check reads the workspace's manifests: the
//! members the root `Cargo.toml` lists, and each member's package name and
//! every dependency table (normal, target, dev and build), an inherited
//! entry through the root's `[workspace.dependencies]`, a renamed one by its
//! package. A line it does not read is a failure, never a skip.

use std::collections::BTreeMap;
use std::fs;

/// The trees, upstream first: a package may name its own tree's packages
/// and those of the trees before it.
const TREES: [&str; 4] = ["skein", "smith", "jig", "temper"];

/// A member: its directory from the root, its package's name and the
/// packages it depends on, in any table.
#[derive(Debug)]
struct Member {
    directory: String,
    name: String,
    dependencies: Vec<String>,
}

/// A dependency line: its key, the package it names and whether it
/// inherits the workspace's entry.
struct Entry<'a> {
    key: &'a str,
    package: &'a str,
    inherits: bool,
}

/// The tree that holds `directory`, by its place in [`TREES`].
fn tree(directory: &str) -> Option<(usize, &'static str)> {
    let first = directory.split('/').next()?;
    TREES.iter().copied().enumerate().find(|(_, tree)| *tree == first)
}

/// The quoted text at the start of `value`.
fn quoted(value: &str) -> Option<&str> {
    let value = value.trim();
    let quote = value.chars().next().filter(|quote| *quote == '"' || *quote == '\'')?;
    value.strip_prefix(quote)?.split_once(quote).map(|(text, _)| text)
}

/// A manifest's tables, each with its lines, blank lines and comments left
/// out; the lines before the first table are the table named "".
fn tables(source: &str) -> Vec<(&str, Vec<&str>)> {
    let mut tables = vec![("", Vec::new())];
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            tables.push((line.trim_matches(['[', ']']), Vec::new()));
        } else if let Some((_, lines)) = tables.last_mut() {
            lines.push(line);
        }
    }
    tables
}

fn entry(line: &str) -> Option<Entry<'_>> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    let (key, inherits) = match key.strip_suffix(".workspace") {
        Some(key) => (key, value.trim() == "true"),
        None => (key, value.replace(' ', "").contains("workspace=true")),
    };
    let key = quoted(key).unwrap_or(key);
    if key.is_empty() || !key.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
        return None;
    }
    let package = value
        .split_once("package")
        .and_then(|(_, rest)| rest.trim_start().strip_prefix('='))
        .and_then(quoted)
        .unwrap_or(key);
    Some(Entry { key, package, inherits })
}

/// The members the root manifest lists.
fn members(root: &str) -> Result<Vec<String>, String> {
    let (_, lines) =
        tables(root).into_iter().find(|(table, _)| *table == "workspace").ok_or("the root has no [workspace]")?;
    let text = lines.join("\n");
    let list = text
        .split_once("members")
        .and_then(|(_, rest)| rest.split_once('['))
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .ok_or("the root's [workspace] lists no members")?;
    Ok(list.split(',').filter_map(quoted).map(str::to_owned).collect())
}

/// The root's `[workspace.dependencies]`: the package each key names.
fn inherited(root: &str) -> Result<BTreeMap<String, String>, String> {
    let mut packages = BTreeMap::new();
    for (table, lines) in tables(root) {
        if table == "workspace.dependencies" {
            for line in lines {
                let entry =
                    entry(line).ok_or_else(|| format!("Cargo.toml: an entry the check does not read: {line}"))?;
                packages.insert(entry.key.to_owned(), entry.package.to_owned());
            }
        } else if table.contains("dependencies") {
            return Err(format!("Cargo.toml: a table the check does not read: [{table}]"));
        }
    }
    Ok(packages)
}

fn member(directory: &str, source: &str, inherited: &BTreeMap<String, String>) -> Result<Member, String> {
    let mut name = None;
    let mut dependencies = Vec::new();
    for (table, lines) in tables(source) {
        if table == "package" {
            name = lines
                .iter()
                .find_map(|line| line.strip_prefix("name").and_then(|rest| rest.trim_start().strip_prefix('=')))
                .and_then(quoted)
                .or(name);
        } else if table.ends_with("dependencies") {
            for line in lines {
                let entry = entry(line)
                    .ok_or_else(|| format!("{directory}/Cargo.toml: a dependency the check does not read: {line}"))?;
                let package = if entry.inherits {
                    inherited
                        .get(entry.key)
                        .map(String::as_str)
                        .ok_or_else(|| format!("{directory}/Cargo.toml: {} is not a workspace dependency", entry.key))?
                } else {
                    entry.package
                };
                dependencies.push(package.to_owned());
            }
        } else if table.contains("dependencies") {
            return Err(format!("{directory}/Cargo.toml: a table the check does not read: [{table}]"));
        }
    }
    let name = name.ok_or_else(|| format!("{directory}/Cargo.toml: no package name"))?;
    Ok(Member { directory: directory.to_owned(), name: name.to_owned(), dependencies })
}

/// The workspace's members, from the root manifest and `manifest`, which
/// reads a member's own by its directory.
fn read<F>(root: &str, manifest: F) -> Result<Vec<Member>, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    let inherited = inherited(root)?;
    members(root)?.iter().map(|directory| member(directory, &manifest(directory)?, &inherited)).collect()
}

/// Each dependency of a tree's package on a package of a tree after it.
fn breaches(members: &[Member]) -> Vec<String> {
    let trees: BTreeMap<&str, (usize, &str)> =
        members.iter().filter_map(|member| Some((member.name.as_str(), tree(&member.directory)?))).collect();
    let mut breaches = Vec::new();
    for member in members {
        let Some((own, _)) = tree(&member.directory) else { continue };
        for dependency in &member.dependencies {
            if let Some(&(theirs, name)) = trees.get(dependency.as_str())
                && theirs > own
            {
                breaches
                    .push(format!("{} ({}) depends on {dependency}, a {name} package", member.name, member.directory));
            }
        }
    }
    breaches
}

fn workspace() -> Vec<Member> {
    let root = skein_scratch::paths::workspace();
    let source = fs::read_to_string(root.join("Cargo.toml")).expect("the root manifest is readable");
    let manifest = |directory: &str| {
        fs::read_to_string(root.join(directory).join("Cargo.toml"))
            .map_err(|why| format!("{directory}/Cargo.toml: {why}"))
    };
    match read(&source, manifest) {
        Ok(members) => members,
        Err(why) => panic!("the workspace's manifests: {why}"),
    }
}

#[test]
fn the_trees_depend_upstream_only() {
    let members = workspace();
    for (place, name) in TREES.iter().copied().enumerate() {
        let upstream = members
            .iter()
            .filter(|member| tree(&member.directory).is_some_and(|(own, _)| own == place))
            .flat_map(|member| &member.dependencies)
            .filter(|dependency| {
                members.iter().any(|other| {
                    other.name == **dependency && tree(&other.directory).is_some_and(|(theirs, _)| theirs < place)
                })
            })
            .count();
        assert!(place == 0 || upstream > 0, "the check reads {name}'s dependencies on the trees before it");
        assert!(
            members.iter().any(|member| tree(&member.directory).is_some_and(|(own, _)| own == place)),
            "the check reads {name}'s members"
        );
    }
    let breaches = breaches(&members);
    assert!(breaches.is_empty(), "dependencies against the trees' order:\n{}", breaches.join("\n"));
}

const ROOT: &str = r#"
[workspace]
members = [
    "skein/a",
    "smith/b",
    "jig/c",
    "temper/d",
    "tests/e",
]

[workspace.dependencies]
skein-a = { path = "skein/a" }
smith-b = { path = "smith/b" }
jig-c = { path = "jig/c" }
renamed = { package = "temper-d", path = "temper/d" }
"#;

fn fixture(manifests: &[(&str, &str)]) -> Result<Vec<Member>, String> {
    read(ROOT, |directory| {
        manifests
            .iter()
            .find(|(at, _)| *at == directory)
            .map(|(_, source)| (*source).to_owned())
            .ok_or_else(|| format!("{directory}: no manifest"))
    })
}

#[test]
fn a_dependency_on_a_later_tree_is_a_breach_in_any_table() {
    let members = fixture(&[
        ("skein/a", "[package]\nname = \"skein-a\"\n[dev-dependencies]\nsmith-b.workspace = true\n"),
        (
            "smith/b",
            "[package]\nname = \"smith-b\"\n[dependencies]\nskein-a.workspace = true\n\
             [target.'cfg(unix)'.dependencies]\njig-c = { workspace = true, features = [\"x\"] }\n",
        ),
        ("jig/c", "[package]\nname = \"jig-c\"\n[build-dependencies]\nrenamed.workspace = true\n"),
        (
            "temper/d",
            "[package]\nname = \"temper-d\"\n[dependencies]\njig-c.workspace = true\n\
             own = { path = \"../../skein/a\", package = \"skein-a\" }\n",
        ),
        ("tests/e", "[package]\nname = \"e\"\n[dependencies]\nrenamed.workspace = true\n"),
    ])
    .expect("the fixture reads");
    assert_eq!(
        breaches(&members),
        [
            "skein-a (skein/a) depends on smith-b, a smith package",
            "smith-b (smith/b) depends on jig-c, a jig package",
            "jig-c (jig/c) depends on temper-d, a temper package",
        ]
    );
}

#[test]
fn a_line_the_check_does_not_read_fails_it() {
    for (source, why) in [
        ("[package]\nname = \"skein-a\"\n[dependencies]\nsmith-b.version = \"1\"\n", "does not read"),
        ("[package]\nname = \"skein-a\"\n[dependencies]\nfeatures = [\n\"x\",\n]\n", "does not read"),
        ("[package]\nname = \"skein-a\"\n[dependencies.smith-b]\nworkspace = true\n", "does not read"),
        ("[package]\nname = \"skein-a\"\n[dependencies]\nunknown.workspace = true\n", "not a workspace dependency"),
        ("[dependencies]\nskein-a.workspace = true\n", "no package name"),
    ] {
        let members = [
            ("skein/a", source),
            ("smith/b", "[package]\nname = \"smith-b\"\n"),
            ("jig/c", "[package]\nname = \"jig-c\"\n"),
            ("temper/d", "[package]\nname = \"temper-d\"\n"),
            ("tests/e", "[package]\nname = \"e\"\n"),
        ];
        let failure = fixture(&members).expect_err("the fixture fails");
        assert!(failure.contains(why), "{source:?}: {failure}");
    }
}

// Operator precheck validation: harmless candidate comment.
