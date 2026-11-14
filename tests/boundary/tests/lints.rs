//! Every package and `clippy.toml` of the four trees is a definition's copy
//! (tools/lints/design.md, 5). The per-file count of catches only falls (section 7).
//! These checks and the count's regression tests run on every change.

use std::process::Command;

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
