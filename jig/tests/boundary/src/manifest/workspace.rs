//! The workspace graph fixture for kit independence (jig/docs/design/domain/testing.md, 7).

use std::{fs, path::Path};
use toml::{Table, Value};

#[derive(Debug)]
pub(crate) struct Package {
    pub name: String,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
}

fn read_manifest(path: &Path) -> Table {
    let source = fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    source.parse().unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn dependency_name<'a>(key: &'a str, value: &'a Value, inherited: &'a Table) -> &'a str {
    let declaration = if value.get("workspace").and_then(Value::as_bool) == Some(true) {
        inherited.get(key).unwrap_or_else(|| panic!("workspace dependency {key} is declared"))
    } else {
        value
    };
    declaration.get("package").and_then(Value::as_str).unwrap_or(key)
}

fn package(manifest: &Table, inherited: &Table) -> Package {
    let name = manifest["package"]["name"].as_str().expect("package name");
    let targets = manifest.get("target").and_then(Value::as_table).into_iter().flat_map(Table::values);
    let mut dependencies = Vec::new();
    let mut dev_dependencies = Vec::new();
    for tables in std::iter::once(manifest).chain(targets.map(|target| target.as_table().expect("target table"))) {
        for (kind, edges) in [("dependencies", &mut dependencies), ("dev-dependencies", &mut dev_dependencies)] {
            if let Some(entries) = tables.get(kind).and_then(Value::as_table) {
                edges.extend(entries.iter().map(|(key, value)| dependency_name(key, value, inherited).to_owned()));
            }
        }
    }
    dependencies.extend(dev_dependencies.iter().cloned());
    Package { name: name.to_owned(), dependencies, dev_dependencies }
}

/// Read the jig members explicitly listed in the workspace manifest.
pub(crate) fn jig_packages(kit: &Path) -> Vec<Package> {
    let root = kit.parent().expect("kit is a tree of the workspace");
    let workspace = read_manifest(&root.join("Cargo.toml"));
    let workspace = workspace["workspace"].as_table().expect("workspace table");
    let inherited = workspace["dependencies"].as_table().expect("workspace dependencies");
    let kit = kit.strip_prefix(root).expect("kit is under the workspace");
    workspace["members"]
        .as_array()
        .expect("workspace members")
        .iter()
        .map(|member| Path::new(member.as_str().expect("workspace member path")))
        .filter(|member| member.starts_with(kit))
        .map(|member| package(&read_manifest(&root.join(member).join("Cargo.toml")), inherited))
        .collect()
}
