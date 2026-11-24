//! Staged builds are checked before publication (qa/builds.md, 3 and 4).
use sha2::{Digest, Sha256};
use skein_scratch::Scratch;
use std::{
    fmt::Write,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{
    BuildState, Home,
    build::keep_with,
    faces::{RealClock, RealProcesses},
};

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
    let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
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
    assert_eq!(
        keep_with(&home, COMMIT, &stage, &[], &mut RealFiles, &mut RealProcesses::default(), &RealClock::default())
            .expect("kept"),
        format!("build {SHORT} kept: 8 files\n")
    );
    assert!(!stage.exists());
    let kept = home.builds.join(COMMIT);
    assert_eq!(fs::read_to_string(kept.join("manifest")).expect("manifest"), expected);
    assert!(!kept.join("bundle-record").exists());
    home.read(&mut RealFiles, |ledger| {
        assert_eq!(ledger.builds.len(), 1);
        assert_eq!(ledger.builds[0].commit, COMMIT);
        assert_eq!(ledger.builds[0].state, BuildState::Kept);
        Ok(())
    })
    .expect("ledger");
}
