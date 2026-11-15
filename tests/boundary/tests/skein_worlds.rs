//! A world's library is its own (skein/docs/design/testing/README.md, 6): no package
//! depends on a package under skein/tests/, and no #[path] leaves its package.
//! One tracked-source walk feeds both source rules (tools/crates/design.md, 3).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use syn::visit::Visit;

struct Dependency {
    name: String,
    directory: PathBuf,
    dev: bool,
}

struct Package {
    name: String,
    directory: PathBuf,
    dependencies: Vec<Dependency>,
}

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => assert!(result.pop(), "{} leaves its root", path.display()),
            Component::CurDir => (),
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => result.push(part),
        }
    }
    result
}

fn packages(metadata: &str) -> Vec<Package> {
    let metadata: Value = serde_json::from_str(metadata).expect("cargo metadata is JSON");
    let root = Path::new(metadata["workspace_root"].as_str().expect("metadata workspace_root"));
    metadata["packages"]
        .as_array()
        .expect("metadata packages")
        .iter()
        .map(|package| Package {
            name: package["name"].as_str().expect("package name").to_owned(),
            directory: normalized(
                Path::new(package["manifest_path"].as_str().expect("manifest path"))
                    .parent()
                    .expect("manifest directory")
                    .strip_prefix(root)
                    .expect("workspace package"),
            ),
            dependencies: package["dependencies"]
                .as_array()
                .expect("package dependencies")
                .iter()
                .filter_map(|dependency| {
                    let path = Path::new(dependency["path"].as_str()?);
                    Some(Dependency {
                        name: dependency["name"].as_str().expect("dependency name").to_owned(),
                        directory: normalized(path.strip_prefix(root).unwrap_or(path)),
                        dev: dependency["kind"].as_str() == Some("dev"),
                    })
                })
                .collect(),
        })
        .collect()
}

fn world_edges(packages: &[Package]) -> Vec<String> {
    let mut failures = Vec::new();
    for package in packages {
        for dependency in &package.dependencies {
            if dependency.directory.starts_with("skein/tests")
                && !(dependency.dev && dependency.directory == package.directory)
            {
                failures.push(format!("{} -> {}: {}", package.name, dependency.name, dependency.directory.display()));
            }
        }
    }
    failures.sort();
    failures.dedup();
    failures
}

#[derive(Default)]
struct SeamPaths(bool);

impl<'ast> Visit<'ast> for SeamPaths {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let mut segments = path.segments.iter();
        self.0 |= segments.next().is_some_and(|segment| segment.ident == "skein_forge_connection")
            && segments.next().is_some_and(|segment| segment.ident == "seam");
        syn::visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.0 |= seam_use(&item.tree, "");
    }
}

fn seam_use(tree: &syn::UseTree, prefix: &str) -> bool {
    match tree {
        syn::UseTree::Path(path) => {
            let prefix = format!("{prefix}{}::", path.ident);
            prefix == "skein_forge_connection::seam::" || seam_use(&path.tree, &prefix)
        }
        syn::UseTree::Name(name) => format!("{prefix}{}", name.ident) == "skein_forge_connection::seam",
        syn::UseTree::Rename(rename) => format!("{prefix}{}", rename.ident) == "skein_forge_connection::seam",
        syn::UseTree::Group(group) => group.items.iter().any(|item| seam_use(item, prefix)),
        syn::UseTree::Glob(_) => false,
    }
}

#[derive(Default)]
struct Paths(Vec<String>);

impl<'ast> Visit<'ast> for Paths {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if attribute.path().is_ident("path") {
            let value = &attribute.meta.require_name_value().expect("path attribute has a value").value;
            if let syn::Expr::Lit(value) = value
                && let syn::Lit::Str(path) = &value.lit
            {
                self.0.push(path.value());
            } else {
                panic!("path attribute has a string value");
            }
        }
    }
}

#[derive(Default)]
struct Walk {
    failures: Vec<String>,
    read: usize,
    parsed: usize,
}

impl Walk {
    fn source(&mut self, root: &Path, path: &Path, source: &str, manifests: &BTreeSet<PathBuf>) {
        let seam = !path.starts_with("skein/crates/skein-forge-connection")
            && !path.starts_with("skein/tests/forge/machine")
            && source.contains("skein_forge_connection");
        let paths = path.starts_with("skein") && source.contains("path");
        if !seam && !paths {
            return;
        }
        let parsed = syn::parse_file(source).unwrap_or_else(|why| panic!("{}: {why}", path.display()));
        self.parsed += 1;
        if seam {
            let mut seam = SeamPaths::default();
            seam.visit_file(&parsed);
            if seam.0 {
                self.failures.push(format!("{}: seam is private to skein-forge-machine-world", path.display()));
            }
        }
        if paths {
            let mut paths = Paths::default();
            paths.visit_file(&parsed);
            let absolute = root.join(path);
            let parent = absolute.parent().expect("source directory");
            let package = parent
                .ancestors()
                .find(|directory| manifests.contains(&directory.join("Cargo.toml")))
                .expect("Rust source has a package");
            for target in paths.0 {
                let resolved = normalized(&parent.join(&target));
                if !resolved.starts_with(package) {
                    self.failures.push(format!(
                        "{}: #[path = {target:?}] leaves {}",
                        path.display(),
                        package.strip_prefix(root).expect("package under root").display(),
                    ));
                }
            }
        }
    }
}

fn walk(root: &Path, paths: &[&str], mut read: impl FnMut(&str) -> String) -> Walk {
    let manifests = paths
        .iter()
        .filter(|path| Path::new(path).file_name().is_some_and(|name| name == "Cargo.toml"))
        .map(|path| root.join(path))
        .collect();
    let mut walk = Walk::default();
    for path in paths.iter().map(Path::new).filter(|path| path.extension().is_some_and(|extension| extension == "rs")) {
        let source = read(path.to_str().expect("tracked path is UTF-8"));
        walk.read += 1;
        walk.source(root, path, &source, &manifests);
    }
    walk.failures.sort();
    walk
}

fn tracked_walk(root: &Path) -> Walk {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .expect("list tracked sources and manifests");
    assert!(output.status.success(), "git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let paths: Vec<_> = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|raw| std::str::from_utf8(raw).expect("tracked path is UTF-8"))
        .collect();
    walk(root, &paths, |path| fs::read_to_string(root.join(path)).unwrap_or_else(|why| panic!("{path}: {why}")))
}

fn fixture(files: &[(&str, &str)]) -> Walk {
    let paths: Vec<_> = files.iter().map(|(path, _)| *path).collect();
    walk(Path::new("/workspace"), &paths, |path| {
        files.iter().find(|(name, _)| *name == path).expect("fixture source").1.to_owned()
    })
}

#[test]
fn dependency_paths_hold_for_every_kind_and_tree() {
    for (directory, dependency, kind, expected) in [
        ("skein/crates/step", "skein/tests/renamed", Value::Null, true),
        ("smith/tests/world", "skein/tests/json", Value::from("dev"), true),
        ("jig/testing/kit", "skein/tests/json", Value::from("build"), true),
        ("temper/crates/app", "skein/tests/json", Value::Null, true),
        ("hearth/crates/app", "skein/tests/json", Value::from("dev"), true),
        ("skein/tests/json", "skein/tests/http", Value::from("dev"), true),
        ("skein/tests/json", "skein/tests/json", Value::from("dev"), false),
        ("skein/tests/json", "skein/testing/json-kit", Value::Null, false),
        ("skein/tests/json", "skein/tests-nearby/kit", Value::Null, false),
    ] {
        let metadata = serde_json::json!({
            "workspace_root": "/workspace",
            "packages": [{
                "name": "consumer", "manifest_path": format!("/workspace/{directory}/Cargo.toml"),
                "dependencies": [{
                    "name": "provider", "rename": "alias", "kind": kind, "target": "cfg(unix)",
                    "path": format!("/workspace/{dependency}/../{}/", Path::new(dependency).file_name().expect("directory").to_string_lossy()),
                }, { "name": "registry", "kind": null }],
            }],
        });
        let failures = world_edges(&packages(&metadata.to_string()));
        let expected = if expected { vec![format!("consumer -> provider: {dependency}")] } else { vec![] };
        assert_eq!(failures, expected, "{directory} -> {dependency}, {kind}");
    }
}

#[test]
fn seam_is_private_to_the_machine_world() {
    for (path, source, expected) in [
        ("skein/tests/forge/machine/src/hand.rs", "use skein_forge_connection::seam::GitHub;", false),
        ("temper/crates/x/src/lib.rs", "use skein_forge_connection::seam::GitHub;", true),
        ("skein/crates/skein-forge-connection/src/lib.rs", "use skein_forge_connection::seam::GitHub;", false),
        ("temper/crates/x/src/lib.rs", "use skein_forge_connection::{Component, seam::{GitHub, Access}};", true),
        ("temper/crates/x/src/lib.rs", "type State = skein_forge_connection::seam::GitHub;", true),
        ("temper/crates/x/src/lib.rs", "use skein_forge_connection::Component;", false),
        ("temper/crates/x/src/lib.rs", r#"const TEXT: &str = "skein_forge_connection::seam::GitHub";"#, false),
    ] {
        let failures = fixture(&[(path, source)]).failures;
        let expected =
            if expected { vec![format!("{path}: seam is private to skein-forge-machine-world")] } else { vec![] };
        assert_eq!(failures, expected, "{path}: {source}");
    }
}

#[test]
fn attributes_stay_within_the_nearest_package() {
    for (source, expected) in [
        ("#[path = \"local.rs\"] mod local;", false),
        ("#[path = \"../shared.rs\"] mod shared;", false),
        ("#[path = \"../../other/src/lib.rs\"] mod other;", true),
        ("#[cfg(any())]\n# [ path = r\"../../other/src/lib.rs\" ] mod other;", true),
        ("// #[path = \"../../other.rs\"]\nconst TEXT: &str = \"path = outside\";", false),
    ] {
        let walk = fixture(&[
            ("skein/outer/Cargo.toml", "[package]\npath = \"../../outside\"\n"),
            ("skein/outer/nested/Cargo.toml", "[package]\npath = \"../../outside\"\n"),
            ("skein/outer/nested/src/lib.rs", source),
        ]);
        let expected = if expected {
            vec!["skein/outer/nested/src/lib.rs: #[path = \"../../other/src/lib.rs\"] leaves skein/outer/nested"]
        } else {
            vec![]
        };
        assert_eq!(walk.failures, expected, "{source}");
    }
}

#[test]
fn one_walk_reads_each_source_and_parses_for_both_rules_once() {
    let source = "use skein_forge_connection::seam::GitHub; #[path = \"../../other/src/lib.rs\"] mod other;";
    let walk = fixture(&[
        ("skein/outer/nested/Cargo.toml", ""),
        ("skein/outer/nested/src/lib.rs", source),
        ("temper/crates/x/src/lib.rs", source),
        ("skein/outer/nested/src/main.rs", "fn main() {}"),
    ]);
    assert_eq!(walk.read, 3);
    assert_eq!(walk.parsed, 2);
    assert_eq!(
        walk.failures,
        [
            "skein/outer/nested/src/lib.rs: #[path = \"../../other/src/lib.rs\"] leaves skein/outer/nested",
            "skein/outer/nested/src/lib.rs: seam is private to skein-forge-machine-world",
            "temper/crates/x/src/lib.rs: seam is private to skein-forge-machine-world",
        ]
    );
}

#[test]
fn skein_worlds_are_private_to_their_packages() {
    let root = skein_scratch::paths::workspace();
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked", "--offline"])
        .current_dir(&root)
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
    let packages = packages(&String::from_utf8(output.stdout).expect("metadata is UTF-8"));
    for tree in ["skein", "smith", "jig", "temper", "hearth"] {
        assert!(packages.iter().any(|package| package.directory.starts_with(tree)), "metadata reads {tree}");
    }
    let mut failures = world_edges(&packages);
    let walk = tracked_walk(&root);
    eprintln!("Read {} Rust source files; parsed {} files.", walk.read, walk.parsed);
    failures.extend(walk.failures);
    assert!(
        failures.is_empty(),
        "A world's library is its own (skein/docs/design/testing/README.md, 6):\n{}",
        failures.join("\n")
    );
}
