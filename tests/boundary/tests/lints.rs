//! Every `clippy.toml` bans catching a panic (tools/lints/design.md, 4).
//! Every package and `clippy.toml` of the four trees is a definition's copy (section 5).
//! The per-file count of catches only falls (section 7).
//! These checks and the count's regression tests run on every change.

use std::{fs, path::Path, process::Command};

#[test]
fn every_package_takes_a_lint_profile() {
    let output = Command::new("python3")
        .args(["-I", "-B", "tools/lints/lints.py"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("the lint profile check starts");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn every_clippy_configuration_bans_catching_panics() {
    let entries = [
        concat!(
            "    { path = \"std::panic::catch",
            "_unwind\", reason = \"",
            "nothing catches a panic, tests included (skein/docs/foundation/programming-model.md, 10.4)\" },"
        ),
        concat!(
            "    { path = \"std::panic::resume",
            "_unwind\", reason = \"",
            "nothing catches a panic, tests included (skein/docs/foundation/programming-model.md, 10.4)\" },"
        ),
    ];
    let root = skein_scratch::paths::workspace();
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(&root)
        .output()
        .expect("list tracked lint configurations");
    assert!(output.status.success(), "git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let mut offenders = Vec::new();
    for raw in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(raw).expect("tracked path is UTF-8");
        if Path::new(path).file_name().is_some_and(|name| name == "clippy.toml") {
            let text = fs::read_to_string(root.join(path)).unwrap_or_else(|why| panic!("{path}: {why}"));
            if !entries.iter().all(|entry| text.lines().any(|line| line == *entry)) {
                offenders.push(path);
            }
        }
    }
    assert!(offenders.is_empty(), "Lint configurations lacking the panic-catching bans:\n{}", offenders.join("\n"));
}

#[test]
fn no_file_catches_more_panics_than_its_list() {
    let output = Command::new("python3")
        .args(["-I", "-B", "tools/lints/panics.py"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("the count of catches starts");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn the_count_of_catches_has_regression_tests() {
    let output = Command::new("python3")
        .args(["-I", "-B", "tools/lints/test_panics.py"])
        .current_dir(skein_scratch::paths::workspace())
        .output()
        .expect("the count of catches' regression tests start");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
