//! Staged builds are checked before publication (qa/builds.md, 3 and 4).
use sha2::{Digest, Sha256};
use skein_scratch::Scratch;
use std::{
    fmt::Write,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use temper_qa_tool::{BuildState, Home, build::keep};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const SHORT: &str = "0123456789ab";
const REQUIRED: [&str; 8] = [
    "bin/temper-box",
    "bin/fake-claude",
    "bin/temper",
    "bin/temper-client-journeys",
    "bin/temper-engine",
    "bin/temper-worker",
    "bin/temper-agent",
    "bundle/index.html",
];

fn stage() -> (Scratch, Home, PathBuf) {
    let scratch = Scratch::new("qa-drive-build");
    fs::write(
        scratch.path().join("qa.json"),
        br#"{"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#,
    )
    .expect("config");
    let home = Home::open(scratch.path()).expect("home");
    let stage = scratch.path().join("stage");
    for directory in ["bin", "bundle"] {
        fs::create_dir_all(stage.join(directory)).expect("stage directory");
    }
    for binary in ["temper-box", "temper", "temper-client-journeys", "temper-engine", "temper-worker", "temper-agent"] {
        script(&stage.join("bin").join(binary), &format!("{binary} 0.1.0 ({SHORT})"));
    }
    script(&stage.join("bin/fake-claude"), "2.1.312 (Claude Code)");
    fs::write(stage.join("bundle/index.html"), b"web\n").expect("bundle");
    (scratch, home, stage)
}

fn script(path: &Path, line: &str) {
    fs::write(path, format!("#!/bin/sh\nprintf '{line}\\n'\n")).expect("script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable");
}

fn refusal(home: &Home, stage: &Path, words: &str) {
    let before = fs::read(&home.ledger).ok();
    assert_eq!(keep(home, COMMIT, stage, &[]).expect_err("stage refused").to_string(), words);
    assert!(stage.is_dir(), "staging was moved");
    assert!(!home.builds.join(COMMIT).exists(), "build was published");
    assert!(!home.builds.join(format!(".{COMMIT}.partial")).exists(), "staging was moved to partial");
    assert_eq!(fs::read(&home.ledger).ok(), before, "ledger was changed");
}

#[test]
fn missing_deployment_binaries_are_named_before_anything_moves() {
    for missing in [vec!["bin/temper-worker"], vec!["bin/temper-engine", "bin/temper-agent"]] {
        let (_scratch, home, stage) = stage();
        for path in &missing {
            fs::remove_file(stage.join(path)).expect("remove required file");
        }
        let words =
            missing.iter().map(|path| format!("{} lacks {path}", stage.display())).collect::<Vec<_>>().join("; ");
        refusal(&home, &stage, &words);
    }
}

#[test]
fn a_deployment_stage_keeps_the_coding_agent_version_and_checksums_all_eight_files() {
    let (_scratch, home, stage) = stage();
    let mut paths = REQUIRED;
    paths.sort_unstable();
    let mut expected = format!("commit {COMMIT}\n");
    for path in paths {
        let bytes = fs::read(stage.join(path)).expect("staged bytes");
        writeln!(expected, "{:x}  {path}", Sha256::digest(&bytes)).expect("manifest row");
    }
    assert_eq!(keep(&home, COMMIT, &stage, &[]).expect("kept"), format!("build {SHORT} kept: 8 files\n"));
    assert!(!stage.exists());
    let kept = home.builds.join(COMMIT);
    assert_eq!(fs::read_to_string(kept.join("manifest")).expect("manifest"), expected);
    assert!(!kept.join("bundle-record").exists());
    home.read(|ledger| {
        assert_eq!(ledger.builds.len(), 1);
        assert_eq!(ledger.builds[0].commit, COMMIT);
        assert_eq!(ledger.builds[0].state, BuildState::Kept);
        Ok(())
    })
    .expect("ledger");
}

#[test]
fn an_older_recording_bundle_is_kept_in_the_manifest() {
    let (_scratch, home, stage) = stage();
    fs::create_dir(stage.join("bundle-record")).expect("recording directory");
    fs::write(stage.join("bundle-record/index.html"), b"record\n").expect("recording bundle");
    assert_eq!(keep(&home, COMMIT, &stage, &[]).expect("kept"), format!("build {SHORT} kept: 9 files\n"));
    let kept = home.builds.join(COMMIT);
    assert_eq!(fs::read(kept.join("bundle-record/index.html")).expect("kept recording"), b"record\n");
    let manifest = fs::read_to_string(kept.join("manifest")).expect("manifest");
    assert!(
        manifest.lines().any(|line| line == format!("{:x}  bundle-record/index.html", Sha256::digest(b"record\n")))
    );
}

#[test]
fn the_engine_must_print_the_kept_commit() {
    let (_scratch, home, stage) = stage();
    script(&stage.join("bin/temper-engine"), "temper-engine 0.1.0 (abcdef012345)");
    refusal(&home, &stage, &format!("bin/temper-engine prints temper-engine 0.1.0 (abcdef012345), not {SHORT}"));
}

#[test]
fn even_the_coding_agent_and_extra_bin_files_must_be_executable() {
    let (_scratch, home, stage) = stage();
    for path in ["bin/fake-claude", "bin/temper-client-journeys", "bin/extra"] {
        let full = stage.join(path);
        if !full.exists() {
            script(&full, &format!("extra ({SHORT})"));
        }
        fs::set_permissions(&full, fs::Permissions::from_mode(0o600)).expect("not executable");
        refusal(&home, &stage, &format!("{path} is not executable"));
        fs::set_permissions(&full, fs::Permissions::from_mode(0o700)).expect("restore executable");
    }
}

#[test]
fn the_drive_tool_and_every_file_but_fake_claude_must_print_the_kept_commit() {
    let (_scratch, home, stage) = stage();
    let path = stage.join("bin/temper-client-journeys");
    script(&path, "temper-client-journeys 0.1.0 (abcdef012345)");
    refusal(
        &home,
        &stage,
        &format!("bin/temper-client-journeys prints temper-client-journeys 0.1.0 (abcdef012345), not {SHORT}"),
    );
    script(&path, &format!("temper-client-journeys 0.1.0 ({SHORT})"));
    script(&stage.join("bin/fake-claude-extra"), "2.1.312 (Claude Code)");
    refusal(&home, &stage, &format!("bin/fake-claude-extra prints 2.1.312 (Claude Code), not {SHORT}"));
}

#[test]
fn a_manifest_uses_one_sha256sum_faces_answer_for_every_sorted_byte_path() {
    manifest_keeps_path(Path::new("bundle/weird\\\n\r"), b"bundle/weird\\\\\\n\\r");
}

// APFS, macOS's file system, holds no file name that is not UTF-8 (temper/docs/design/testing/macos.md, 3).
#[cfg(target_os = "linux")]
#[test]
fn a_manifest_keeps_a_non_utf8_path_as_its_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    manifest_keeps_path(
        &PathBuf::from(OsString::from_vec(b"bundle/weird\\\n\r\xff".to_vec())),
        b"bundle/weird\\\\\\n\\r\xff",
    );
}

fn manifest_keeps_path(name: &Path, escaped: &[u8]) {
    let (_scratch, home, stage) = stage();
    fs::write(stage.join(name), b"asset").expect("byte path");
    let mut paths: Vec<_> = REQUIRED.iter().map(PathBuf::from).collect();
    paths.push(name.into());
    paths.sort();
    let mut answer = Vec::new();
    for path in &paths {
        // The face's hashes deliberately differ from the files' bytes.
        let line = if path == name {
            let mut line = format!("\\{}  ", "a".repeat(64)).into_bytes();
            line.extend_from_slice(escaped);
            line.push(b'\n');
            line
        } else {
            format!("{}  {}\n", "b".repeat(64), path.display()).into_bytes()
        };
        answer.extend_from_slice(&line);
    }
    let mut face = super::build_files::Files { answer: answer.clone(), ..Default::default() };
    temper_qa_tool::build::keep_with(&home, COMMIT, &stage, &[], &mut face).expect("kept");
    assert_eq!(face.hashes, [(stage, paths)]);
    let mut expected = format!("commit {COMMIT}\n").into_bytes();
    expected.extend_from_slice(&answer);
    assert_eq!(fs::read(home.builds.join(COMMIT).join("manifest")).expect("manifest"), expected);
}

#[test]
fn a_sha256sum_answer_missing_a_file_or_not_parsing_never_moves_or_records() {
    for (answer, expected) in [
        (format!("{}  bin/fake-claude\n", "0".repeat(64)).into_bytes(), "absent from sha256sum answer"),
        (b"not a checksum\n".to_vec(), "invalid checksum line"),
    ] {
        let (_scratch, home, stage) = stage();
        let mut face = super::build_files::Files { answer, ..Default::default() };
        let why = temper_qa_tool::build::keep_with(&home, COMMIT, &stage, &[], &mut face).expect_err("refused");
        assert!(why.0.contains(expected), "{why}");
        assert!(stage.is_dir());
        assert!(!home.builds.join(COMMIT).exists());
        assert!(!home.builds.join(format!(".{COMMIT}.partial")).exists());
        assert!(!home.ledger.exists());
    }
}
