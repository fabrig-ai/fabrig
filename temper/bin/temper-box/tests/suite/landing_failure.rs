//! Failure evidence for the owned-forge story (testing/live-tier.md).
use crate::support;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};
use support::forge_land::{keep_box_data, run_failure_lines};

#[test]
fn run_failure_lines_keep_only_run_records_in_rotation_order() {
    let scratch = support::Scratch::new("landing-failure-lines");
    let logs = scratch.path().join("box/logs");
    fs::create_dir_all(&logs).expect("log fixture");
    let oldest = "temper worker: run 2 attempt 1 failed Unprepared(Transient): host local: git failed";
    let refused = "temper serve: task 7 run 2 attempt 1 refused Invalid(Start(Bounds)): oversized charter";
    let failed = "temper serve: task 7 run 2 attempt 1 failed Run(Model): anthropic 400 invalid_request_error";
    let version = "temper-box serve: task 8 run 3 attempt 2 refused: worker version mismatch";
    fs::write(logs.join("serve.log.2"), format!("ordinary step\n{oldest}\n")).expect("oldest log");
    fs::write(logs.join("serve.log.1"), format!("{refused}\n")).expect("rotated log");
    fs::write(logs.join("serve.log.3.gz"), format!("{failed}\n")).expect("unread compressed sibling");
    fs::write(logs.join("worker.log"), format!("{oldest}\n")).expect("other log");
    fs::write(
        logs.join("serve.log"),
        format!(
            "temper serve: listening on 127.0.0.1:8370\n\
         temper serve: task 7 run 2 attempt 1 started\n\
         temper serve: task seven run 2 attempt 1 failed Run(Model): not a record\n\
         ordinary step quotes {failed}\n\
         temper worker: run 2 attempt 1 refused Busy: not a worker failure\n\
         {failed}\r\n{version}"
        ),
    )
    .expect("current log");
    assert_eq!(run_failure_lines(scratch.path()), [oldest, refused, failed, version]);
    scratch.remove();
}

#[test]
fn run_failure_lines_redact_credentials_but_keep_run_context() {
    use std::fmt::Write as _;
    let scratch = support::Scratch::new("landing-failure-redaction");
    let logs = scratch.path().join("box/logs");
    fs::create_dir_all(&logs).expect("log fixture");
    let context = "temper serve: task 7 run 2 attempt 1 failed Run(Model): ";
    let details = [
        ("secret=abc123 task-12", "secret=… task-12"),
        (r#"{"api_key":"k1"} task-12"#, r#"{"api_key":"…"} task-12"#),
        ("x-api-key: sk-ant-api03-xyz task-12", "x-api-key: … task-12"),
        ("sk-ant-api03-xyz task-12", "… task-12"),
        ("X-Api-Key:\theader-value task-12", "X-Api-Key:\t… task-12"),
        ("(sk-ant-api03-xyz), task-12", "(…), task-12"),
        ("task-12 wordsk-ant-not-a-key _sk-ant-not-a-key", "task-12 wordsk-ant-not-a-key _sk-ant-not-a-key"),
    ];
    let mut text = String::new();
    for (detail, _) in details {
        writeln!(text, "{context}{detail}").expect("fixture line");
    }
    fs::write(logs.join("serve.log"), text).expect("credential-shaped fixture lines");
    let lines = run_failure_lines(scratch.path());
    assert_eq!(lines, details.map(|(_, safe)| format!("{context}{safe}")));
    assert!(lines.iter().all(|line| !line.contains("abc123") && !line.contains("k1") && !line.contains("xyz")));
    scratch.remove();
}

#[test]
fn run_failure_lines_allow_a_box_that_never_started() {
    let scratch = support::Scratch::new("landing-no-serve-log");
    assert_eq!(run_failure_lines(&scratch.path().join("absent")), Vec::<String>::new());
    assert_eq!(run_failure_lines(scratch.path()), Vec::<String>::new());
    fs::create_dir_all(scratch.path().join("box/logs")).expect("empty log directory");
    assert_eq!(run_failure_lines(scratch.path()), Vec::<String>::new());
    scratch.remove();
}

fn data_fixture(data: &Path) {
    for directory in [
        "box/logs",
        "deployment/generations/1/store/logs",
        "deployment/generations/1/secrets",
        "deployment/checkpoints/1/secrets",
        "programs/live",
        "forge-1",
        "box/nested/secrets",
        "box/nested/checkpoints",
    ] {
        fs::create_dir_all(data.join(directory)).expect("data fixture directory");
    }
    for excluded in [
        "deployment/generations/1/secrets/s",
        "deployment/checkpoints/1/secrets/s",
        "worker.secret",
        "provider.key",
        "programs/live/x",
        "forge-1/admin_token",
        "forge-1/setup",
    ] {
        // Empty stand-ins: the keeper must not open these credential-bearing names.
        fs::File::create(data.join(excluded)).expect("excluded fixture entry");
    }
    fs::write(data.join("box/logs/serve.log"), b"ordinary log\n\0\xff").expect("serve log bytes");
    fs::write(data.join("deployment/generations/1/store/logs/2"), b"store log\0\xff").expect("store log bytes");
    symlink("../worker.secret", data.join("box/link")).expect("unfollowed link");
    symlink("../deployment", data.join("box/directory-link")).expect("directory link");
}

fn assert_private_tree(directory: &Path) {
    assert_eq!(fs::symlink_metadata(directory).expect("directory metadata").permissions().mode() & 0o777, 0o700);
    for entry in fs::read_dir(directory).expect("kept entries") {
        let entry = entry.expect("kept entry");
        let name = entry.file_name();
        assert_ne!(name, "secrets");
        assert_ne!(name, "checkpoints");
        if entry.file_type().expect("kept entry type").is_dir() {
            assert_private_tree(&entry.path());
        }
    }
}

#[test]
fn kept_box_data_is_private_credential_free_and_does_not_follow_links() {
    let scratch = support::Scratch::new("landing-kept-data");
    let source = scratch.path().join("source");
    data_fixture(&source);
    let kept = keep_box_data(&source, scratch.path()).expect("kept data");
    assert_eq!(kept, scratch.path().join("data"));
    for log in ["box/logs/serve.log", "deployment/generations/1/store/logs/2"] {
        assert_eq!(fs::read(kept.join(log)).expect("kept log"), fs::read(source.join(log)).expect("source log"));
    }
    for link in ["box/link", "box/directory-link"] {
        assert!(fs::symlink_metadata(kept.join(link)).expect("kept link metadata").is_symlink());
        assert_eq!(
            fs::read_link(kept.join(link)).expect("kept link"),
            fs::read_link(source.join(link)).expect("source link")
        );
    }
    for excluded in ["worker.secret", "provider.key", "programs", "forge-1"] {
        assert!(fs::symlink_metadata(kept.join(excluded)).is_err(), "excluded {excluded}");
        assert!(fs::symlink_metadata(source.join(excluded)).is_ok(), "source stays: {excluded}");
    }
    assert_private_tree(&kept);
    scratch.remove();
}

#[test]
fn kept_box_data_never_overwrites_an_existing_destination() {
    let scratch = support::Scratch::new("landing-existing-copy");
    let source = scratch.path().join("source");
    data_fixture(&source);
    let destination = scratch.path().join("data");
    fs::create_dir(&destination).expect("earlier copy");
    fs::write(destination.join("evidence"), b"keep me").expect("earlier evidence");
    let why = keep_box_data(&source, scratch.path()).expect_err("existing destination refused");
    assert!(why.contains(&destination.display().to_string()), "{why}");
    assert_eq!(fs::read(destination.join("evidence")).expect("earlier evidence untouched"), b"keep me");
    assert_eq!(fs::read_dir(&destination).expect("earlier copy untouched").count(), 1);
    scratch.remove();
}
