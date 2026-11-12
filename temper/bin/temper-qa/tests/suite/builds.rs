//! Staged scripts through the shipped command (qa/builds.md, 3 and 4).
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    process::{Command, Output, Stdio},
};
use temper_qa_tool::{BuildState, Home};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const SHORT: &str = "0123456789ab";

fn home() -> (Scratch, Home) {
    let scratch = Scratch::new("qa-builds");
    fs::write(scratch.path().join("qa.json"), br#"{"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low"},"builds":"published"}"#).expect("config");
    let home = Home::open(scratch.path()).expect("home");
    (scratch, home)
}
fn stage(home: &Home, name: &str) -> std::path::PathBuf {
    let path = home.directory.join(name);
    for directory in ["bin", "bundle", "bundle-record"] {
        fs::create_dir_all(path.join(directory)).expect("stage directory");
    }
    script(&path.join("bin/temper-box"), "printf 'temper-box 0.1.0 (0123456789ab)\\n'");
    fs::write(path.join("bundle/index.html"), b"a\n").expect("bundle");
    fs::write(path.join("bundle-record/index.html"), b"record\n").expect("record bundle");
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
    assert!(output.stderr.is_empty());
}
fn refusal(home: &Home, commit: &str, stage: &Path, words: &str) {
    let before = fs::read(&home.ledger).ok();
    let output = keep(home, commit, stage);
    assert_eq!(output.status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stderr), format!("temper-qa: {words}\n"));
    assert!(output.stdout.is_empty());
    assert!(stage.is_dir(), "staging was moved");
    assert!(!home.builds.join(commit).exists(), "build was published");
    assert!(!home.builds.join(format!(".{commit}.partial")).exists(), "staging was moved to partial");
    assert_eq!(fs::read(&home.ledger).ok(), before, "ledger was changed");
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
    success(&keep(&home, COMMIT, &stage), 6);
    assert!(!stage.exists());
    let kept = home.builds.join(COMMIT);
    assert_eq!(fs::read(kept.join("bin/temper-box")).expect("kept script"), original);
    let manifest = fs::read_to_string(kept.join("manifest")).expect("manifest");
    let mut lines = manifest.lines();
    assert_eq!(lines.next(), Some(format!("commit {COMMIT}").as_str()));
    let rows: Vec<_> = lines.collect();
    assert_eq!(rows.len(), 6);
    let paths: Vec<_> = rows.iter().map(|line| line.split_once("  ").expect("checksum row").1).collect();
    assert_eq!(
        paths,
        [
            "bin/another",
            "bin/directory/data",
            "bin/temper-box",
            "bundle-record/index.html",
            "bundle/a\\\\b\\nc\\r",
            "bundle/index.html"
        ]
    );
    assert!(rows.contains(&"87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  bundle/index.html"));
    assert!(rows.contains(&"\\87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7  bundle/a\\\\b\\nc\\r"));
    home.read(|ledger| {
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

#[test]
fn commit_form_refuses_before_build_checks_and_accepts_sha256_ids() {
    let (_scratch, home) = home();
    let stage = stage(&home, "stage");
    for commit in [
        "",
        "0123456789ab",
        "0123456789abcdef0123456789abcdef0123456G",
        "0123456789ABCDEF0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef012345678",
    ] {
        refusal(&home, commit, &stage, &format!("{commit} is no commit id"));
    }
    let long = format!("{COMMIT}0123456789abcdef01234567");
    success(&keep(&home, &long, &stage), 3);
    assert!(home.builds.join(long).join("manifest").exists());
}

#[test]
fn a_kept_commit_refuses_before_inspecting_the_stage() {
    let (_scratch, home) = home();
    home.transact(|ledger| ledger.keep_build(COMMIT, 1)).expect("kept record");
    let stage = home.directory.join("empty");
    fs::create_dir(&stage).expect("empty stage");
    refusal(&home, COMMIT, &stage, &format!("build {SHORT} is kept already"));
}

#[test]
fn every_missing_required_path_is_named_before_versions_are_run() {
    let (_scratch, home) = home();
    for missing in [
        vec!["bin/temper-box"],
        vec!["bundle/index.html"],
        vec!["bundle-record/index.html"],
        vec!["bin/temper-box", "bundle/index.html", "bundle-record/index.html"],
    ] {
        let stage = stage(&home, "stage");
        for path in &missing {
            fs::remove_file(stage.join(path)).expect("remove required file");
        }
        let words =
            missing.iter().map(|path| format!("{} lacks {path}", stage.display())).collect::<Vec<_>>().join("; ");
        refusal(&home, COMMIT, &stage, &words);
        fs::remove_dir_all(stage).expect("reset stage");
    }
}

#[test]
fn symlinks_under_the_stage_are_refused_without_following_them() {
    let (_scratch, home) = home();
    let stage = stage(&home, "stage");
    symlink("missing", stage.join("bundle/link")).expect("link");
    refusal(&home, COMMIT, &stage, &format!("{} holds bundle/link, neither a file nor a directory", stage.display()));
}

#[test]
fn each_regular_file_directly_in_bin_must_be_executable() {
    let (_scratch, home) = home();
    let stage = stage(&home, "stage");
    fs::write(stage.join("bin/data"), b"data").expect("nonexecutable");
    refusal(&home, COMMIT, &stage, "bin/data is not executable");
}

#[test]
fn every_executable_must_print_the_clean_commit_on_its_first_line() {
    let (_scratch, home) = home();
    let stage = stage(&home, "stage");
    for line in [
        "other 0.1.0 (abcdef012345)",
        "other 0.1.0 (0123456789ab-dirty)",
        "wrong first line\\nother 0.1.0 (0123456789ab)",
    ] {
        script(&stage.join("bin/other"), &format!("printf '{line}\\n'"));
        let first = line.split("\\n").next().expect("line");
        refusal(&home, COMMIT, &stage, &format!("bin/other prints {first}, not {SHORT}"));
    }
}

#[test]
fn a_deleted_build_is_kept_anew_and_an_abandoned_partial_is_removed() {
    let (_scratch, home) = home();
    home.transact(|ledger| {
        ledger.keep_build(COMMIT, 1)?;
        ledger.set_build_holds(COMMIT, vec![], 1)?;
        ledger.mark_build_deleted(COMMIT, 2)?;
        Ok(())
    })
    .expect("deleted through operations");
    let partial = home.builds.join(format!(".{COMMIT}.partial"));
    fs::create_dir_all(&partial).expect("abandoned partial");
    fs::write(partial.join("abandoned"), b"old").expect("abandoned file");
    let stage = stage(&home, "stage");
    success(&keep(&home, COMMIT, &stage), 3);
    assert!(!partial.exists());
    assert!(!home.builds.join(COMMIT).join("abandoned").exists());
    assert!(!stage.exists());
    home.read(|ledger| {
        assert_eq!(ledger.builds.len(), 1);
        let build = &ledger.builds[0];
        assert_eq!(build.state, BuildState::Kept);
        assert!(build.kept > 2);
        assert_eq!(build.holds, None);
        assert_eq!(build.order, None);
        assert_eq!(build.deleted, None);
        Ok(())
    })
    .expect("kept anew");
}

#[test]
fn keep_arguments_are_refused_through_the_table_before_home_is_read() {
    let scratch = Scratch::new("qa-build-arguments");
    for (rest, why) in [
        (vec![], "COMMIT and DIR are not given"),
        (vec![COMMIT], "DIR is not given"),
        (vec![COMMIT, "stage", "extra"], "expected COMMIT and DIR"),
        (vec!["--unknown", "stage"], "expected COMMIT and DIR"),
    ] {
        let mut arguments = vec!["build", "keep"];
        arguments.extend(rest);
        let output = Command::new(skein_scratch::paths::binary("temper-qa"))
            .args(&arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("arguments");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let command = temper_qa_tool::TABLE
            .commands
            .iter()
            .find(|command| command.forms == ["COMMIT DIR"])
            .expect("keep command");
        assert_eq!(output.stderr, temper_qa_tool::TABLE.refusal(command, why).as_bytes());
    }
}
