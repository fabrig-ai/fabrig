//! A world's library is its own (skein/docs/design/testing.md, 6): no package
//! depends on a package under skein/tests/, and no #[path] leaves its package.

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

fn path_crossings(root: &Path) -> Vec<String> {
    let mut directories = vec![root.join("skein")];
    let mut failures = Vec::new();
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).expect("skein directory is readable") {
            let entry = entry.expect("skein directory entry is readable");
            let path = entry.path();
            let kind = entry.file_type().expect("skein file type is readable");
            if kind.is_dir() {
                directories.push(path);
            } else if kind.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("Rust source is readable");
                if !source.contains("path") {
                    continue;
                }
                let mut paths = Paths::default();
                paths.visit_file(&syn::parse_file(&source).unwrap_or_else(|why| panic!("{}: {why}", path.display())));
                let parent = path.parent().expect("source directory");
                let package = parent
                    .ancestors()
                    .find(|directory| directory.join("Cargo.toml").is_file())
                    .expect("Rust source has a package");
                for target in paths.0 {
                    let resolved = normalized(&parent.join(&target));
                    if !resolved.starts_with(package) {
                        failures.push(format!(
                            "{}: #[path = {target:?}] leaves {}",
                            path.strip_prefix(root).expect("source under root").display(),
                            package.strip_prefix(root).expect("package under root").display(),
                        ));
                    }
                }
            }
        }
    }
    failures.sort();
    failures
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
fn attributes_stay_within_the_nearest_package() {
    for (source, expected) in [
        ("#[path = \"local.rs\"] mod local;", false),
        ("#[path = \"../shared.rs\"] mod shared;", false),
        ("#[path = \"../../other/src/lib.rs\"] mod other;", true),
        ("#[cfg(any())]\n# [ path = r\"../../other/src/lib.rs\" ] mod other;", true),
        ("// #[path = \"../../other.rs\"]\nconst TEXT: &str = \"path = outside\";", false),
    ] {
        let scratch = skein_scratch::Scratch::new("world-boundary");
        let root = scratch.path();
        fs::create_dir_all(root.join("skein/outer/nested/src")).expect("fixture source directory");
        for file in ["skein/outer/Cargo.toml", "skein/outer/nested/Cargo.toml"] {
            fs::write(root.join(file), "[package]\npath = \"../../outside\"\n").expect("fixture manifest");
        }
        fs::write(root.join("skein/outer/nested/src/lib.rs"), source).expect("fixture source");
        let expected = if expected {
            vec!["skein/outer/nested/src/lib.rs: #[path = \"../../other/src/lib.rs\"] leaves skein/outer/nested"]
        } else {
            vec![]
        };
        assert_eq!(path_crossings(root), expected, "{source}");
    }
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
    failures.extend(path_crossings(&root));
    assert!(
        failures.is_empty(),
        "A world's library is its own (skein/docs/design/testing.md, 6):\n{}",
        failures.join("\n")
    );
}
