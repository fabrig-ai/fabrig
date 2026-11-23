//! Fixture writers find the checkout when they run (skein's testing-strategy.md, 8).
//! The text check includes comments and strings, and ignores whitespace around
//! the manifest macro. Its own matcher examples are excluded, as in outward.rs.
//! Codec golden bytes stay with every and patches (skein's data/codec/README.md, 5, and data/codec/golden.md, 1).

use std::{fs, path::Path, process::Command};

fn compiled_fixture_path(source: &str) -> bool {
    let rewrites = ["SKEIN_CODEGEN_WRITE", "SKEIN_DIFF_WRITE", "SKEIN_PATCH_WRITE", "temper_test_env::rewrite"];
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
        let absolute = root.join(&path);
        let metadata = fs::symlink_metadata(&absolute).unwrap_or_else(|why| panic!("{path}: {why}"));
        let bytes = if metadata.is_symlink() {
            fs::read_link(&absolute).unwrap_or_else(|why| panic!("{path}: {why}")).into_os_string().into_encoded_bytes()
        } else {
            fs::read(&absolute).unwrap_or_else(|why| panic!("{path}: {why}"))
        };
        let source = std::str::from_utf8(&bytes).unwrap_or_else(|why| panic!("{path}: {why}"));
        if compiled_fixture_path(source) {
            offenders.push(path);
        }
    }
    assert!(
        offenders.is_empty(),
        "Fixture writers must use skein_scratch::paths at run time, not a compiled manifest path:\n{}",
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
    let every = directories.starts_with(&["skein", "tests", "codec", "golden"]);
    let patch_goldens = directories.windows(2).any(|pair| pair == ["patches", "golden"]);
    let patch_story =
        directories.starts_with(&["skein", "tests", "codec-patches", "story"]) && directories.get(5) == Some(&"golden");
    !(every || patch_goldens || patch_story)
}

#[test]
fn only_kept_codec_goldens_are_tracked() {
    let offenders: Vec<_> = tracked_files().into_iter().filter(|path| codec_golden_outside_kept_places(path)).collect();
    assert!(offenders.is_empty(), "Codec golden files belong only to every and patches:\n{}", offenders.join("\n"));
}

#[test]
fn codec_golden_paths_keep_only_pins_and_patches() {
    let paths = [
        "smith/crates/smith-channel/golden/v11/record_frame_full.bin",
        "other/golden/record_row_smallest.bin",
        "other/golden/enum_kind_full.bin",
        "other/golden/enum_kind_smallest.bin",
        "skein/tests/codec/golden/record_every_full.bin",
        "jig/crates/jig-protocol-store/patches/golden/0003/record_row_smallest.bin",
        "skein/tests/codec-patches/story/rename/golden/enum_kind_full.bin",
        "jig/crates/jig-client-view/golden/rendered/record_row_full.txt",
        "other/record_row_full.bin",
        "other/goldenish/record_row_full.bin",
        "skein/tests/codec/goldenish/golden/record_row_full.bin",
        "other/patches/goldenish/golden/record_row_full.bin",
        "skein/tests/codec-patches/story/rename/deeper/golden/record_row_full.bin",
    ];
    let offenders: Vec<_> = paths.into_iter().filter(|path| codec_golden_outside_kept_places(path)).collect();
    assert_eq!(offenders, [paths[0], paths[1], paths[2], paths[3], paths[10], paths[11], paths[12],]);
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
