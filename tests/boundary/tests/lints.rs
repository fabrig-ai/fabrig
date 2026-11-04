//! Every package and `clippy.toml` of the four trees is a definition's copy
//! (tools/lints/design.md, 5). The test runs the check, so that it runs on every change.

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
