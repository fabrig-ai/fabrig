//! The upgrade command on a deployment copy (protocol/binaries.md, 1, 2).
#![expect(clippy::disallowed_types, reason = "scratch fixtures set filesystem modification times without waiting")]
use super::{Request, run};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, SystemTime};
use temper_deployment_fixtures::deployment;

use crate::testing::modified;

mod opening;

#[test]
fn workers_arguments_are_distinct_and_modes_exclude_each_other() {
    let environment = [(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))];
    let words = |words: &[&[u8]]| words.iter().map(|word| Box::from(*word)).collect::<Vec<_>>();
    for mode in ["--check", "--workers"] {
        let (directories, request) =
            super::configure(mode, &words(&[b"--data", b"state", b"backup"]), &environment, b"/work")
                .expect("one directory");
        assert_eq!(directories.data.as_ref(), b"/work/state");
        assert_eq!(
            request,
            if mode == "--check" {
                Request::Check(b"/work/backup".as_slice().into())
            } else {
                Request::Workers(b"/work/backup".as_slice().into())
            }
        );
        for invalid in [
            words(&[]),
            words(&[b""]),
            words(&[b"one", b"two"]),
            words(&[b"--check", b"one"]),
            words(&[b"--workers", b"one"]),
            words(&[b"--back"]),
            words(&[b"one", b"--back"]),
        ] {
            assert!(super::configure(mode, &invalid, &environment, b"/work").is_err(), "{mode}: {invalid:?}");
        }
    }
    assert_eq!(super::configure("--back", &[], &environment, b"/work").unwrap().1, Request::Back);
    for invalid in [words(&[b"one"]), words(&[b"--workers", b"one"]), words(&[b"--check", b"one"])] {
        assert!(super::configure("--back", &invalid, &environment, b"/work").is_err());
    }
    let arguments = words(&[b"upgrade", b"--workers", b"backup"]);
    let skein_shell::Reached::Command(row, rest) = crate::ENGINE.reach(b"temper-engine", &arguments) else {
        panic!("workers row")
    };
    assert_eq!(row, &crate::commands::UPGRADE_WORKERS);
    assert_eq!(
        super::configure("--workers", rest, &environment, b"/work").unwrap().1,
        Request::Workers(b"/work/backup".as_slice().into())
    );
}

#[test]
fn agent_fixture_round_trip_keeps_the_engine_file_and_installs_figures() {
    use temper_config_shell::{FILE, FORMER, Shape, parse, render};
    let (_scratch, directories) = deployment();
    let config = crate::path(&directories.config);
    let path = config.join(FILE);
    let facts = temper_config_shell::facts(&directories).expect("fixture facts");
    let read = parse(&std::fs::read(&path).expect("engine file"), Shape::EngineAlone, &facts).expect("engine file");
    let canonical = render(&read.file, Shape::EngineAlone);
    let mut formatted = canonical.to_vec();
    formatted.extend_from_slice(b"\n# fixture formatting\n");
    std::fs::write(&path, formatted).expect("formatted engine file");
    let mut agent = temper_agent_fixtures::read(config);
    agent["profile"]["policy"]["max_turns"] = 17.into();
    temper_agent_fixtures::install(config, &agent);
    assert_eq!(std::fs::read(&path).expect("rewritten file"), canonical.as_ref());
    assert!(!config.join(FORMER).exists());
    let installed = temper_agent_fixtures::read(config);
    assert_eq!(installed["profile"]["policy"]["max_turns"], 17);
    assert_eq!(installed["endpoints"], agent["endpoints"]);
    let start = temper_agent_fixtures::start(config);
    assert_eq!(start["workers"], serde_json::json!([]));
    assert_eq!(start["engine"]["hosts"]["listener"][0]["socket"], "127.0.0.1:7103");
}

#[test]
fn an_unremovable_scratch_is_named_and_the_rehearsal_goes_on() {
    let (_scratch, directories) = deployment();
    let stale = crate::path(&directories.cache).join("upgrade-00000000000000aa");
    std::fs::create_dir_all(&stale).expect("stale scratch");
    std::fs::write(stale.join("file"), b"left behind").expect("stale file");
    modified(&stale, SystemTime::now() - Duration::from_secs(7200));
    std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o500)).expect("removal denied");
    let lines = super::scratch::sweep("temper-engine", crate::path(&directories.cache));
    let code = run(
        "temper-engine",
        &directories,
        &Request::Workers([directories.data.as_ref(), b"/state"].concat().into()),
        "test-build",
    );
    std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o700)).expect("restore cleanup permission");
    assert_eq!(lines, ["temper-engine upgrade: scratch left behind not removed: upgrade-00000000000000aa: errno 13"]);
    assert_eq!(code.expect("removal fault does not stop rehearsal"), 0);
    assert_eq!(std::fs::read(stale.join("file")).expect("unremoved file remains"), b"left behind");
}

fn leftover(name: &str) -> (skein_scratch::Scratch, std::path::PathBuf) {
    let cache = skein_scratch::Scratch::new("rehearsal-leftover");
    let path = cache.path().join(name);
    std::fs::create_dir(&path).expect("leftover directory");
    (cache, path)
}

#[test]
fn an_unlocked_staging_directory_is_never_swept() {
    let (cache, staging) = leftover("upgrade-00000000000000aa.new");
    modified(&staging, SystemTime::now() - Duration::from_secs(7200));
    assert_eq!(super::scratch::sweep("temper-engine", cache.path()), Vec::<String>::new());
    assert!(staging.is_dir(), "the staging name is never swept");
}

#[test]
fn recent_and_future_unlocked_leftovers_are_not_swept() {
    let (cache, leftover) = leftover("upgrade-00000000000000aa");
    std::fs::write(leftover.join("file"), b"copying").expect("leftover file");
    let now = SystemTime::now();
    for time in [now, now + Duration::from_secs(7200)] {
        modified(&leftover, time);
        assert_eq!(super::scratch::sweep("temper-engine", cache.path()), Vec::<String>::new());
        assert_eq!(std::fs::read(leftover.join("file")).expect("recent leftover remains"), b"copying");
    }
    modified(&leftover, now - Duration::from_secs(7200));
    assert_eq!(super::scratch::sweep("temper-engine", cache.path()), Vec::<String>::new());
    assert!(!leftover.exists(), "the same leftover is removed whole once old");
}

#[test]
fn an_unopenable_leftover_is_named_instead_of_silently_skipped() {
    let (cache, leftover) = leftover("upgrade-00000000000000aa");
    modified(&leftover, SystemTime::now() - Duration::from_secs(7200));
    std::fs::set_permissions(&leftover, std::fs::Permissions::from_mode(0o000)).expect("open denied");
    let lines = super::scratch::sweep("temper-engine", cache.path());
    std::fs::set_permissions(&leftover, std::fs::Permissions::from_mode(0o700)).expect("restore cleanup permission");
    assert_eq!(lines, ["temper-engine upgrade: scratch left behind not removed: upgrade-00000000000000aa: errno 13"]);
    assert!(leftover.is_dir(), "unopenable leftover remains");
}

#[test]
fn own_scratch_removal_failure_names_the_path_and_cause() {
    let (_cache, path) = leftover("upgrade-00000000000000aa");
    std::fs::write(path.join("file"), b"left behind").expect("own file");
    let held = std::fs::File::open(&path).expect("own directory");
    held.try_lock().expect("own lock");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o500)).expect("removal denied");
    let line = super::scratch::remove("temper-engine", &path, held);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).expect("restore cleanup permission");
    assert_eq!(line.as_deref(), Some("temper-engine upgrade: scratch not removed: upgrade-00000000000000aa: errno 13"));
}

#[test]
fn scratch_failures_name_errno_or_error_kind_without_panicking() {
    for (why, expected) in [
        (std::io::Error::from_raw_os_error(13), "scratch: errno 13"),
        (std::io::Error::from(std::io::ErrorKind::Other), "scratch: kind Other"),
    ] {
        assert_eq!(super::scratch::failure(&why), expected);
    }
}
