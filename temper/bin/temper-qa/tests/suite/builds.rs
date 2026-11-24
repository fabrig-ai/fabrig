//! Staged scripts through the shipped command (qa/builds.md, 3 and 4).
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{BuildState, Home};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const SHORT: &str = "0123456789ab";

fn home() -> (Scratch, Home) {
    let scratch = Scratch::new("qa-builds");
    fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low","pass":"high"},"request":{"command":["file"]},"builds":"published"}"#).expect("config");
    let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
    (scratch, home)
}
fn stage(home: &Home, name: &str) -> std::path::PathBuf {
    let path = home.directory.join(name);
    for directory in ["bin", "bundle"] {
        fs::create_dir_all(path.join(directory)).expect("stage directory");
    }
    for binary in ["temper-box", "temper", "temper-client-journeys", "temper-engine", "temper-worker", "temper-agent"] {
        script(&path.join("bin").join(binary), &format!("printf '{binary} 0.1.0 (0123456789ab)\\n'"));
    }
    script(&path.join("bin/fake-claude"), "printf '2.1.312 (Claude Code)\\n'");
    fs::write(path.join("bundle/index.html"), b"a\n").expect("bundle");
    path
}
fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable");
}
fn run(home: &Home, arguments: &[&str]) -> Output {
    Command::new(skein_scratch::paths::binary("temper-qa"))
        .args(arguments)
        .env_clear()
        .env("TEMPER_INTERNAL_QA_HOME", &home.directory)
        .env("QA_VERSION_WORD", "captured")
        .current_dir(&home.directory)
        .stdin(Stdio::null())
        .output()
        .expect("run tool")
}
fn keep(home: &Home, commit: &str, stage: &Path) -> Output {
    run(home, &["build", "keep", commit, stage.to_str().expect("scratch path")])
}
fn success(output: &Output, files: usize) {
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, format!("build {SHORT} kept: {files} files\n").as_bytes());
    assert_eq!(output.stderr.len(), 0);
}
#[test]
fn kept_build_moves_files_writes_sorted_sha256_manifest_and_records_time_without_holds() {
    let (_scratch, home) = home();
    let stage = stage(&home, "stage");
    // Both scripts must receive --version, the captured environment and closed stdin.
    let body = "[ \"$1\" = --version ] && [ \"$QA_VERSION_WORD\" = captured ] && ! read -r input && printf 'temper-box 0.1.0 (0123456789ab)\\nsecond line\\n'";
    script(&stage.join("bin/temper-box"), body);
    script(&stage.join("bin/another"), body);
    fs::create_dir(stage.join("bin/directory")).expect("bin subdirectory");
    fs::write(stage.join("bin/directory/data"), b"a\n").expect("nonexecutable nested data");
    fs::write(stage.join("manifest"), b"stale manifest").expect("old manifest");
    fs::write(stage.join("bundle/a\\b\nc\r"), b"a\n").expect("escaped filename");
    let original = fs::read(stage.join("bin/temper-box")).expect("original script");
    success(&keep(&home, COMMIT, &stage), 11);
    assert!(!stage.exists());
    let kept = home.builds.join(COMMIT);
    assert_eq!(fs::read(kept.join("bin/temper-box")).expect("kept script"), original);
    let manifest = fs::read_to_string(kept.join("manifest")).expect("manifest");
    let mut lines = manifest.lines();
    assert_eq!(lines.next(), Some(format!("commit {COMMIT}").as_str()));
    let rows: Vec<_> = lines.collect();
    assert_eq!(rows.len(), 11);
    let paths: Vec<_> = rows.iter().map(|line| line.split_once("  ").expect("checksum row").1).collect();
    assert_eq!(
        paths,
        [
            "bin/another",
            "bin/directory/data",
            "bin/fake-claude",
            "bin/temper",
            "bin/temper-agent",
            "bin/temper-box",
            "bin/temper-client-journeys",
            "bin/temper-engine",
            "bin/temper-worker",
            "bundle/a\\\\b\\nc\\r",
            "bundle/index.html"
        ]
    );
    assert!(rows.contains(&"87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  bundle/index.html"));
    assert!(rows.contains(&"\\87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  bundle/a\\\\b\\nc\\r"));
    home.read(&mut RealFiles, |ledger| {
        assert_eq!(ledger.builds.len(), 1);
        let build = &ledger.builds[0];
        assert_eq!(build.commit, COMMIT);
        assert_eq!(build.state, BuildState::Kept);
        assert!(build.kept > 0);
        assert_eq!(build.holds, None);
        assert_eq!(build.order, None);
        assert_eq!(build.deleted, None);
        assert!(ledger.document()["builds"][0].get("holds").is_none());
        Ok(())
    })
    .expect("ledger");
}
