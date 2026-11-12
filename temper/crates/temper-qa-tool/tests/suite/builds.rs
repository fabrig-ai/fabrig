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
const REQUIRED: [&str; 6] = [
    "bin/temper-box",
    "bin/fake-claude",
    "bin/temper-client",
    "bin/temper-client-journeys",
    "bundle/index.html",
    "bundle-record/index.html",
];

fn stage() -> (Scratch, Home, PathBuf) {
    let scratch = Scratch::new("qa-drive-build");
    fs::write(
        scratch.path().join("qa.json"),
        br#"{"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low"}}"#,
    )
    .expect("config");
    let home = Home::open(scratch.path()).expect("home");
    let stage = scratch.path().join("stage");
    for directory in ["bin", "bundle", "bundle-record"] {
        fs::create_dir_all(stage.join(directory)).expect("stage directory");
    }
    for binary in ["temper-box", "temper-client", "temper-client-journeys"] {
        script(&stage.join("bin").join(binary), &format!("{binary} 0.1.0 ({SHORT})"));
    }
    script(&stage.join("bin/fake-claude"), "2.1.312 (Claude Code)");
    fs::write(stage.join("bundle/index.html"), b"web\n").expect("bundle");
    fs::write(stage.join("bundle-record/index.html"), b"record\n").expect("record bundle");
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
fn all_six_required_files_are_named_when_missing_before_anything_moves() {
    let (_scratch, home, stage) = stage();
    for path in REQUIRED {
        let full = stage.join(path);
        let contents = fs::read(&full).expect("staged file");
        fs::remove_file(&full).expect("remove required file");
        refusal(&home, &stage, &format!("{} lacks {path}", stage.display()));
        fs::write(&full, contents).expect("restore required file");
        fs::set_permissions(&full, fs::Permissions::from_mode(0o700)).expect("restore executable");
    }
    for path in REQUIRED {
        fs::remove_file(stage.join(path)).expect("remove required file");
    }
    let words = REQUIRED.map(|path| format!("{} lacks {path}", stage.display())).join("; ");
    refusal(&home, &stage, &words);
}

#[test]
fn a_full_stage_keeps_the_coding_agent_version_and_checksums_all_six_files() {
    let (_scratch, home, stage) = stage();
    let mut paths = REQUIRED;
    paths.sort_unstable();
    let mut expected = format!("commit {COMMIT}\n");
    for path in paths {
        let bytes = fs::read(stage.join(path)).expect("staged bytes");
        writeln!(expected, "{:x}  {path}", Sha256::digest(&bytes)).expect("manifest row");
    }
    assert_eq!(keep(&home, COMMIT, &stage, &[]).expect("kept"), format!("build {SHORT} kept: 6 files\n"));
    assert!(!stage.exists());
    let kept = home.builds.join(COMMIT);
    assert_eq!(fs::read_to_string(kept.join("manifest")).expect("manifest"), expected);
    home.read(|ledger| {
        assert_eq!(ledger.builds.len(), 1);
        assert_eq!(ledger.builds[0].commit, COMMIT);
        assert_eq!(ledger.builds[0].state, BuildState::Kept);
        Ok(())
    })
    .expect("ledger");
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
