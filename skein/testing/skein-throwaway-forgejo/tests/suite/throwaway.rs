// No owned Forgejo on macOS: the pinned binary releases are Linux's (runtime/shell.md, 8).
#![cfg(target_os = "linux")]

use skein_forge::forgejo::pin::VERSION;
use skein_throwaway_forgejo::{Forgejo, Init, Outgoing, Settings};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
#[test]
#[ignore = "needs SKEIN_FORGEJO"]
#[expect(clippy::disallowed_methods, reason = "skein's round: reads SKEIN_FORGEJO to locate the pinned test binary")]
fn a_throwaway_forgejo_starts_serves_its_admin_and_is_removed() {
    let forgejo = Forgejo::from_path(Path::new(
        &std::env::var_os("SKEIN_FORGEJO").expect("SKEIN_FORGEJO must name the pinned Forgejo"),
    ))
    .expect("pinned Forgejo");
    let root = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "throwaway-start");
    let scratch = root.path().join("forge");
    let mut instance = forgejo.start(&scratch, &Settings::small()).expect("started");
    let admin = instance.admin().clone();
    let user = instance.exchange(&Outgoing::get("/api/v1/user", Some(&admin.token))).expect("answered");
    assert_eq!(user.status, 200, "the admin's token reads its user");
    let made = instance.create_repository(&admin, "specimen", Init::Readme).expect("made");
    assert_eq!(made.name, "specimen");
    instance.stop().expect("stopped");
    assert!(!scratch.exists(), "the scratch directory is removed");
}

/// A pinned Forgejo version followed by the supplied command body.
fn forgejo_running(root: &Path, body: &str) -> Forgejo {
    fs::create_dir_all(root).expect("root");
    let binary = root.join("forgejo");
    fs::write(
        &binary,
        format!("#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'forgejo version {VERSION}'; exit 0; fi\n{body}\n"),
    )
    .expect("script");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable");
    Forgejo::from_path(&binary).expect("version accepted")
}
#[test]
fn a_work_path_runs_commands_in_the_scratch_with_its_environment() {
    let root = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "work-path-run");
    let forgejo = forgejo_running(root.path(), "pwd; echo \"$HOME $PATH $TZ $TMPDIR\"; echo \"$@\"");
    let work = forgejo.work_path(&root.path().join("work"), &Settings::small()).expect("prepared");
    let output = work.run(&["admin", "user", "list"]).expect("ran");
    let w = work.scratch().display();
    assert_eq!(
        output.stdout,
        format!(
            "{w}\n{w}/home /usr/bin:/bin Asia/Kolkata {w}/tmp\n--work-path {w} --config {w}/app.ini admin user list\n"
        )
    );
    assert!(work.scratch().join("tmp").is_dir());
}
#[test]
fn a_failed_work_path_command_keeps_its_exit_code_and_both_streams() {
    let root = skein_scratch::paths::target_tmp().join("work-path-fail");
    let forgejo = forgejo_running(&root, "echo output; echo diagnostic >&2; exit 7");
    let work = forgejo.work_path(&root.join("work"), &Settings::small()).expect("prepared");
    let output = work.run(&["migrate"]).expect("ran");
    assert_eq!(output.code, Some(7));
    assert_eq!(output.stdout, "output\n");
    assert_eq!(output.stderr, "diagnostic\n");
    fs::remove_dir_all(root).expect("removed");
}

use crate::support;
#[test]
fn attach_reads_the_admin_login_and_id() {
    let fixture = support::attached("kit-owner", 37);
    assert_eq!(fixture.instance.admin().login, "kit-owner");
    assert_eq!(fixture.instance.admin().id, 37);
    fixture.instance.stop().expect("stopped");
    fs::remove_dir_all(fixture.scratch).expect("owner removes scratch");
}
#[test]
fn stopping_an_attached_instance_reaps_the_child_and_keeps_scratch() {
    let fixture = support::attached("kit-owner", 37);
    fixture.instance.stop().expect("stopped");
    assert!(!Path::new(&format!("/proc/{}", fixture.pid)).exists());
    assert_eq!(fs::read(fixture.scratch.join("keep")).expect("owner file"), b"mine");
    fs::remove_dir_all(fixture.scratch).expect("owner removes scratch");
}
