//! Fixture writers find the checkout when they run (skein's testing-strategy.md, 8).
//! This text check includes comments and strings, and ignores whitespace around
//! the manifest macro. Its own matcher examples are excluded, as in outward.rs.

use std::{fs, process::Command};

fn compiled_fixture_path(source: &str) -> bool {
    let rewrites = ["SKEIN_CODEGEN_WRITE", "SKEIN_DIFF_WRITE", "SKEIN_PATCH_WRITE", "temper_test_env::rewrite"];
    if !rewrites.iter().any(|switch| source.contains(switch)) {
        return false;
    }
    let compact: String = source.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    compact.contains("env!(\"CARGO_MANIFEST_DIR\")")
}

#[test]
fn fixture_writers_find_paths_at_run_time() {
    let root = skein_scratch::paths::workspace();
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "-z", "--", "*.rs"])
        .output()
        .expect("list tracked Rust sources");
    assert!(output.status.success(), "git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let mut offenders = Vec::new();
    for raw in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(raw).expect("tracked path is UTF-8");
        if path == "tests/boundary/tests/fixture_paths.rs" {
            continue;
        }
        let absolute = root.join(path);
        let metadata = fs::symlink_metadata(&absolute).unwrap_or_else(|why| panic!("{path}: {why}"));
        let bytes = if metadata.is_symlink() {
            fs::read_link(&absolute).unwrap_or_else(|why| panic!("{path}: {why}")).into_os_string().into_encoded_bytes()
        } else {
            fs::read(&absolute).unwrap_or_else(|why| panic!("{path}: {why}"))
        };
        let source = std::str::from_utf8(&bytes).unwrap_or_else(|why| panic!("{path}: {why}"));
        if compiled_fixture_path(source) {
            offenders.push(path.to_owned());
        }
    }
    assert!(
        offenders.is_empty(),
        "Fixture writers must use skein_scratch::paths at run time, not a compiled manifest path:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn rewrite_sources_need_runtime_paths() {
    for source in [
        r#"SKEIN_CODEGEN_WRITE env!("CARGO_MANIFEST_DIR")"#,
        r#"SKEIN_DIFF_WRITE concat!(env!("CARGO_MANIFEST_DIR"), "/golden")"#,
        "SKEIN_PATCH_WRITE env! ( \n \"CARGO_MANIFEST_DIR\" )",
        r#"temper_test_env::rewrite() env!("CARGO_MANIFEST_DIR")"#,
    ] {
        assert!(compiled_fixture_path(source), "{source}");
    }
    for source in
        ["SKEIN_CODEGEN_WRITE skein_scratch::paths::package()", r#"std::fs::read(env!("CARGO_MANIFEST_DIR"))"#]
    {
        assert!(!compiled_fixture_path(source), "{source}");
    }
}
