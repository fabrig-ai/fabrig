//! Fixture writers find the checkout when they run (skein's testing-strategy.md, 8).
//! The text check includes comments and strings, and ignores whitespace around
//! the manifest macro. Its own matcher examples are excluded, as in outward.rs.
//! Kits find binaries without the caller's Cargo variables (tools/crates/design.md, 3.2).
//! Codec golden bytes stay with every (skein's data/codec/golden.md, 1).

use std::{fs, path::Path, process::Command};

fn compiled_fixture_path(source: &str) -> bool {
    let rewrites = ["SKEIN_CODEGEN_WRITE", "temper_test_env::rewrite"];
    if !rewrites.iter().any(|switch| source.contains(switch)) {
        return false;
    }
    let compact: String = source.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    compact.contains("env!(\"CARGO_MANIFEST_DIR\")")
}

fn tracked_files() -> Vec<String> {
    let root = skein_scratch::paths::workspace();
    let output =
        Command::new("git").arg("-C").arg(&root).args(["ls-files", "-z"]).output().expect("list tracked files");
    assert!(output.status.success(), "git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|raw| std::str::from_utf8(raw).expect("tracked path is UTF-8").to_owned())
        .collect()
}

fn tracked_source(root: &Path, path: &str) -> String {
    let absolute = root.join(path);
    let metadata = fs::symlink_metadata(&absolute).unwrap_or_else(|why| panic!("{path}: {why}"));
    let bytes = if metadata.is_symlink() {
        fs::read_link(&absolute).unwrap_or_else(|why| panic!("{path}: {why}")).into_os_string().into_encoded_bytes()
    } else {
        fs::read(&absolute).unwrap_or_else(|why| panic!("{path}: {why}"))
    };
    String::from_utf8(bytes).unwrap_or_else(|why| panic!("{path}: {why}"))
}

#[test]
fn fixture_writers_find_paths_at_run_time() {
    let root = skein_scratch::paths::workspace();
    let mut offenders = Vec::new();
    for path in tracked_files() {
        if Path::new(&path).extension().is_none_or(|extension| extension != "rs")
            || path == "tests/boundary/tests/fixture_paths.rs"
        {
            continue;
        }
        if compiled_fixture_path(&tracked_source(&root, &path)) {
            offenders.push(path);
        }
    }
    assert!(
        offenders.is_empty(),
        "Fixture writers must use skein_scratch::paths at run time, not a compiled manifest path:\n{}",
        offenders.join("\n")
    );
}

fn identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[test]
fn kits_find_built_binaries() {
    let root = skein_scratch::paths::workspace();
    let mut offenders = Vec::new();
    for path in tracked_files() {
        if Path::new(&path).extension().is_none_or(|extension| extension != "rs")
            || path == "skein/testing/skein-scratch/src/paths.rs"
        {
            continue;
        }
        let mut parts = path.split('/');
        let _tree = parts.next();
        if parts.next() != Some("testing") || parts.nth(1) != Some("src") {
            continue;
        }
        let source = tracked_source(&root, &path);
        for (offset, name) in source.match_indices("paths") {
            let before = &source[..offset];
            if before.ends_with(identifier_char) {
                continue;
            }
            let suffix = source[offset + name.len()..].trim_start();
            if suffix
                .strip_prefix("::")
                .and_then(|rest| rest.trim_start().strip_prefix("binary"))
                .is_some_and(|rest| !rest.starts_with(identifier_char))
            {
                let line = before.matches('\n').count() + 1;
                offenders.push(format!("{path}:{line}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "Kits must use paths::built(package, name), not paths::binary:\n{}",
        offenders.join("\n")
    );
}

fn codec_golden_outside_kept_places(path: &str) -> bool {
    let parts: Vec<_> = path.split('/').collect();
    let Some((name, directories)) = parts.split_last() else {
        return false;
    };
    let codec_name = (name.starts_with("record_") || name.starts_with("enum_"))
        && (name.ends_with("_smallest.bin") || name.ends_with("_full.bin"));
    if !codec_name || !directories.contains(&"golden") {
        return false;
    }
    !directories.starts_with(&["skein", "tests", "codec", "golden"])
}

#[test]
fn only_kept_codec_goldens_are_tracked() {
    let offenders: Vec<_> = tracked_files().into_iter().filter(|path| codec_golden_outside_kept_places(path)).collect();
    assert!(
        offenders.is_empty(),
        "Codec golden files belong only in skein/tests/codec/golden:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn codec_golden_paths_keep_only_pins() {
    let paths = [
        "smith/crates/smith-channel/golden/v11/record_frame_full.bin",
        "other/golden/record_row_smallest.bin",
        "other/golden/enum_kind_full.bin",
        "other/golden/enum_kind_smallest.bin",
        "skein/tests/codec/golden/record_every_full.bin",
        "jig/crates/jig-client-view/golden/rendered/record_row_full.txt",
        "other/record_row_full.bin",
        "other/goldenish/record_row_full.bin",
        "skein/tests/codec/goldenish/golden/record_row_full.bin",
    ];
    let offenders: Vec<_> = paths.into_iter().filter(|path| codec_golden_outside_kept_places(path)).collect();
    assert_eq!(offenders, [paths[0], paths[1], paths[2], paths[3], paths[8]]);
}

#[test]
fn rewrite_sources_need_runtime_paths() {
    for source in [
        r#"SKEIN_CODEGEN_WRITE env!("CARGO_MANIFEST_DIR")"#,
        r#"SKEIN_CODEGEN_WRITE concat!(env!("CARGO_MANIFEST_DIR"), "/golden")"#,
        "SKEIN_CODEGEN_WRITE env! ( \n \"CARGO_MANIFEST_DIR\" )",
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
