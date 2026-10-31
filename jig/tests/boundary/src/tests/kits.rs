//! Kit independence (jig/docs/design/README.md, 6.6).

use super::kit_root;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    process::Command,
};

// Each remaining world edge has a reason; removing an edge also removes its entry.
const PENDING: &[(&str, &str, &str)] = &[
    ("jig-accounts-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-brief-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-channel-world", "jig-client-protocol-world", "The fixture uses the protocol client driver."),
    ("jig-channel-world", "skein-llm-world", "The protocol fixture uses the simulated LLM peer."),
    ("jig-channel-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-channel-world", "smith-agent-process-world", "The protocol fixture drives the agent process peer."),
    ("jig-client-browser-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-client-protocol-world", "jig-client-world", "The fixture reuses the shared client harness."),
    ("jig-client-protocol-world", "jig-core-world", "The fixture uses the core world driver and its peers."),
    ("jig-client-protocol-world", "jig-engine-clients-world", "The client fixture reuses engine protocol scenarios."),
    ("jig-client-protocol-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-client-shell-native", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-client-test-kit", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-client-views-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-client-wire-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-client-world", "jig-core-world", "The fixture uses the core world driver and its peers."),
    ("jig-client-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-conformance", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-conformance-world", "jig-client-world", "The fixture reuses the shared client harness."),
    ("jig-conformance-world", "jig-core-world", "The fixture uses the core world driver and its peers."),
    ("jig-conformance-world", "jig-engine-clients-world", "The client fixture reuses engine protocol scenarios."),
    ("jig-conformance-world", "jig-test-connector-world", "The fixture uses the connector simulator."),
    ("jig-conformance-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-core-world", "jig-test-connector-world", "The fixture uses the connector simulator."),
    ("jig-core-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-engine-clients-world", "jig-client-world", "The fixture reuses the shared client harness."),
    ("jig-engine-clients-world", "jig-core-world", "The fixture uses the core world driver and its peers."),
    ("jig-engine-clients-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-engine-workers-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-fleet-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-host-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-notes-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-one-process-world", "jig-channel-world", "The process fixture composes the channel world."),
    ("jig-one-process-world", "jig-client-protocol-world", "The fixture uses the protocol client driver."),
    ("jig-one-process-world", "jig-protocol-accounts-world", "The fixture uses the accounts protocol peer."),
    ("jig-one-process-world", "skein-llm-world", "The protocol fixture uses the simulated LLM peer."),
    ("jig-one-process-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-one-process-world", "smith-agent-process-world", "The protocol fixture drives the agent process peer."),
    ("jig-ops-world", "jig-client-protocol-world", "The fixture uses the protocol client driver."),
    ("jig-ops-world", "jig-client-world", "The fixture reuses the shared client harness."),
    ("jig-ops-world", "jig-core-world", "The fixture uses the core world driver and its peers."),
    ("jig-ops-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-people-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-protocol-accounts-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-store-world", "jig-core-world", "The fixture uses the core world driver and its peers."),
    ("jig-tasks-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-test", "jig-channel-world", "The process fixture composes the channel world."),
    ("jig-test", "jig-client-protocol-world", "The fixture uses the protocol client driver."),
    ("jig-test", "jig-protocol-accounts-world", "The fixture uses the accounts protocol peer."),
    ("jig-test", "skein-tls-world", "The process tests use the TLS fixture."),
    ("jig-test", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-test-domain", "jig-test-connector-world", "The fixture uses the connector simulator."),
    ("jig-test-service", "jig-protocol-accounts-world", "The fixture uses the accounts protocol peer."),
    ("jig-test-service", "skein-world", "The simulator supplies the clock, verdict and world driver."),
    ("jig-test-store", "jig-test-connector-world", "The fixture uses the connector simulator."),
    ("jig-test-store-files", "jig-test-connector-world", "The fixture uses the connector simulator."),
    ("jig-worker-link-world", "skein-world", "The simulator supplies the clock, verdict and world driver."),
];

pub(super) fn pending_world_dependency(package: &str, dependency: &str) -> bool {
    PENDING.iter().any(|(from, to, _)| *from == package && *to == dependency)
}

#[derive(Debug)]
struct Package {
    name: String,
    dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
}

fn packages() -> Vec<Package> {
    let root = kit_root();
    let output = Command::new("cargo")
        .args(["metadata", "--offline", "--no-deps", "--format-version", "1"])
        .current_dir(&root)
        .output()
        .expect("Cargo metadata runs without building");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Cargo metadata is JSON");
    metadata["packages"]
        .as_array()
        .expect("metadata packages")
        .iter()
        .filter_map(|package| {
            let path = std::path::Path::new(package["manifest_path"].as_str().expect("manifest path"));
            if !path.starts_with(&root) {
                return None;
            }
            let mut dependencies = Vec::new();
            let mut dev_dependencies = Vec::new();
            for dependency in package["dependencies"].as_array().expect("package dependencies") {
                let name = dependency["name"].as_str().expect("dependency package name");
                match dependency["kind"].as_str() {
                    None => dependencies.push(name.to_owned()),
                    Some("dev") => {
                        dependencies.push(name.to_owned());
                        dev_dependencies.push(name.to_owned());
                    }
                    Some("build") => {}
                    Some(kind) => panic!("unknown dependency kind: {kind}"),
                }
            }
            Some(Package {
                name: package["name"].as_str().expect("package name").to_owned(),
                dependencies,
                dev_dependencies,
            })
        })
        .collect()
}

fn world_breaches(packages: &[Package], pending: &[(&str, &str, &str)]) -> Vec<String> {
    let edges: BTreeSet<_> = packages
        .iter()
        .flat_map(|package| {
            package
                .dependencies
                .iter()
                .filter(|name| name.ends_with("-world"))
                .map(|name| (package.name.as_str(), name.as_str()))
        })
        .collect();
    let mut failures = Vec::new();
    let mut listed = BTreeSet::new();
    for &(package, dependency, reason) in pending {
        if reason.trim().is_empty() {
            failures.push(format!("{package} -> {dependency}: pending reason is empty"));
        }
        if !listed.insert((package, dependency)) {
            failures.push(format!("{package} -> {dependency}: duplicate pending edge"));
        }
        if !edges.contains(&(package, dependency)) {
            failures.push(format!("{package} -> {dependency}: pending edge no longer exists"));
        }
    }
    for (package, dependency) in edges.difference(&listed) {
        failures.push(format!("{package} -> {dependency}"));
    }
    failures
}

fn return_path(packages: &[Package], start: &str, target: &str) -> Option<Vec<String>> {
    let by_name: BTreeMap<_, _> = packages.iter().map(|package| (package.name.as_str(), package)).collect();
    let mut queue = VecDeque::from([start]);
    let mut parents: BTreeMap<&str, Option<&str>> = BTreeMap::from([(start, None)]);
    while let Some(name) = queue.pop_front() {
        if name == target {
            let mut path = vec![name.to_owned()];
            let mut previous = parents[name];
            while let Some(parent) = previous {
                path.push(parent.to_owned());
                previous = parents[parent];
            }
            path.reverse();
            return Some(path);
        }
        if let Some(package) = by_name.get(name) {
            for dependency in &package.dependencies {
                if !parents.contains_key(dependency.as_str()) {
                    parents.insert(dependency.as_str(), Some(name));
                    queue.push_back(dependency.as_str());
                }
            }
        }
    }
    None
}

fn dev_cycles(packages: &[Package]) -> Vec<String> {
    let mut cycles = BTreeSet::new();
    for package in packages {
        for dependency in &package.dev_dependencies {
            if let Some(path) = return_path(packages, dependency, &package.name) {
                cycles.insert(format!("{} -> dev -> {}", package.name, path.join(" -> ")));
            }
        }
    }
    cycles.into_iter().collect()
}

const DROP_IN_KITS: [&str; 3] = ["jig-conformance", "jig-fake-parties", "jig-fake-workers"];

fn drop_in_breaches(packages: &[Package]) -> Vec<String> {
    packages
        .iter()
        .filter(|package| DROP_IN_KITS.contains(&package.name.as_str()))
        .flat_map(|package| {
            package
                .dependencies
                .iter()
                .filter(|name| name.starts_with("jig-test-"))
                .map(|dependency| format!("{} -> {dependency}", package.name))
        })
        .collect()
}

#[test]
fn kit_packages_depend_on_no_world_except_exact_pending_edges() {
    let packages = packages();
    assert!(!packages.is_empty(), "metadata includes jig packages");
    let failures = world_breaches(&packages, PENDING);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn kit_packages_have_no_dev_dependency_cycles() {
    let cycles = dev_cycles(&packages());
    assert!(cycles.is_empty(), "{}", cycles.join("\n"));
}

#[test]
fn drop_in_kits_link_no_test_application_package() {
    let packages = packages();
    for name in DROP_IN_KITS {
        assert_eq!(packages.iter().filter(|package| package.name == name).count(), 1, "one {name} package");
    }
    let failures = drop_in_breaches(&packages);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn fixture(name: &str, normal: &[&str], dev: &[&str]) -> Package {
    Package {
        name: name.to_owned(),
        dependencies: normal.iter().chain(dev).map(|name| (*name).to_owned()).collect(),
        dev_dependencies: dev.iter().map(|name| (*name).to_owned()).collect(),
    }
}

#[test]
fn world_edges_name_normal_and_dev_dependencies_and_require_exact_reasons() {
    let packages = [fixture("jig-sample", &["normal-world"], &["dev-world"])];
    assert_eq!(world_breaches(&packages, &[]), ["jig-sample -> dev-world", "jig-sample -> normal-world"]);
    let pending =
        [("jig-sample", "normal-world", "Uses its driver."), ("jig-sample", "dev-world", "Uses its fixture.")];
    assert!(world_breaches(&packages, &pending).is_empty());
    assert_eq!(
        world_breaches(&[], &pending),
        [
            "jig-sample -> normal-world: pending edge no longer exists",
            "jig-sample -> dev-world: pending edge no longer exists",
        ]
    );
    let failures = world_breaches(&packages, &[("jig-sample", "normal-world", "")]);
    assert_eq!(failures, ["jig-sample -> normal-world: pending reason is empty", "jig-sample -> dev-world"]);
}

#[test]
fn cycles_name_the_dev_edge_and_each_package_on_the_return_path() {
    let packages = [fixture("a", &[], &["b"]), fixture("b", &["c"], &[]), fixture("c", &["a"], &[])];
    assert_eq!(dev_cycles(&packages), ["a -> dev -> b -> c -> a"]);
    let packages = [fixture("a", &[], &["b"]), fixture("b", &[], &["a"])];
    assert_eq!(dev_cycles(&packages), ["a -> dev -> b -> a", "b -> dev -> a -> b"]);
    assert!(dev_cycles(&[fixture("a", &[], &["b"]), fixture("b", &[], &[])]).is_empty());
    assert_eq!(dev_cycles(&[fixture("a", &[], &["a"])]), ["a -> dev -> a"]);
}

#[test]
fn drop_in_edges_name_each_test_application_dependency() {
    let packages = [
        fixture("jig-conformance", &["jig-test-domain"], &["jig-test-profile"]),
        fixture("jig-fake-parties", &["skein-lib"], &[]),
        fixture("jig-fake-workers", &["jig-test-worker"], &[]),
        fixture("jig-sample", &["jig-test-domain"], &[]),
    ];
    assert_eq!(
        drop_in_breaches(&packages),
        [
            "jig-conformance -> jig-test-domain",
            "jig-conformance -> jig-test-profile",
            "jig-fake-workers -> jig-test-worker",
        ]
    );
}
