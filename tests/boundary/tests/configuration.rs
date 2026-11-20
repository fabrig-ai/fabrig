//! The configuration file has one reader and vocabulary
//! (temper/docs/design/protocol/configuration/file.md, 8).
use std::{fs, path::Path, process::Command};

use aho_corasick::AhoCorasick;
use serde_json::Value;

#[test]
fn temper_configuration_has_one_boundary() {
    let root = skein_scratch::paths::workspace();
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(&root)
        .output()
        .expect("cargo metadata starts");
    assert!(output.status.success(), "cargo metadata: {}", String::from_utf8_lossy(&output.stderr));
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("cargo metadata is JSON");
    let mut findings = Vec::new();
    for package in metadata["packages"].as_array().expect("metadata packages") {
        let name = package["name"].as_str().expect("package name");
        let manifest = Path::new(package["manifest_path"].as_str().expect("manifest path"));
        if !manifest.starts_with(root.join("temper")) || name == "temper-config-shell" {
            continue;
        }
        // Metadata's dependency array includes normal, build, dev and target dependencies,
        // with the original package name even when the dependency is renamed.
        for dependency in package["dependencies"].as_array().expect("package dependencies") {
            if dependency["name"].as_str().expect("dependency name") == "toml" {
                findings.push(format!("{name}: depends on toml ({})", dependency["kind"]));
            }
        }
    }

    let output = Command::new("git")
        .args(["ls-files", "-z", "--", "temper/**/*.rs"])
        .current_dir(&root)
        .output()
        .expect("git ls-files starts");
    assert!(output.status.success(), "git ls-files: {}", String::from_utf8_lossy(&output.stderr));
    let names = String::from_utf8(output.stdout).expect("tracked paths are UTF-8");
    let vocabulary = AhoCorasick::new(["temper.toml", "temper.json"]).expect("file-name matcher");
    for path in names.split('\0').filter(|path| !path.is_empty()) {
        if path.starts_with("temper/crates/temper-config-shell/") {
            continue;
        }
        let text = fs::read_to_string(root.join(path)).unwrap_or_else(|why| panic!("{path}: {why}"));
        for (line, text) in text.lines().enumerate() {
            for found in vocabulary.find_iter(text) {
                findings.push(format!("{path}:{}: {}", line + 1, &text[found.range()]));
            }
        }
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}
