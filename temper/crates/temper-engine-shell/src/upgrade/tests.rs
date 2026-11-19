//! The upgrade command on a deployment copy (protocol/binaries.md, 1, 2).
#![expect(clippy::disallowed_types, reason = "scratch fixtures set filesystem modification times without waiting")]
use super::{Request, run};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::{Duration, SystemTime};
use temper_deployment_fixtures::deployment;

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
fn deployment_rehearsal_with_accounts_workers_and_forge_reaches_its_report() {
    let (_scratch, directories) = deployment();
    let cache = crate::path(&directories.cache);
    let stale = cache.join("upgrade-00000000000000aa");
    std::fs::create_dir_all(stale.join("nested")).expect("stale scratch");
    std::fs::write(stale.join("file"), b"stale").expect("stale file");
    let outside = crate::path(&directories.data).join("outside");
    std::fs::write(&outside, b"checkpoint bytes").expect("outside file");
    std::fs::hard_link(&outside, stale.join("nested/linked")).expect("scratch link");
    assert_eq!(std::fs::metadata(&outside).expect("linked file").nlink(), 2);
    modified(&stale, SystemTime::now() - Duration::from_secs(7200));
    let busy = cache.join("upgrade-00000000000000bb");
    std::fs::create_dir_all(&busy).expect("in-use scratch");
    let held = std::fs::File::open(&busy).expect("in-use directory opens");
    held.try_lock().expect("exclusive scratch lock");
    std::fs::write(busy.join("file"), b"in use").expect("in-use file");
    modified(&busy, SystemTime::now() - Duration::from_secs(7200));
    for name in ["upgrade-x", "other", "upgrade-00000000000000aA", "upgrade-00000000000000aaa"] {
        std::fs::create_dir(cache.join(name)).expect("unrelated directory");
    }
    let regular = cache.join("upgrade-00000000000000cc");
    std::fs::write(&regular, b"regular file").expect("not a directory");
    let link = cache.join("upgrade-00000000000000dd");
    std::os::unix::fs::symlink(crate::path(&directories.data), &link).expect("not a directory entry");
    let before = std::fs::read_dir(cache).expect("cache").count();
    let active_path = Path::new(std::ffi::OsStr::from_bytes(&directories.data)).join("state/active");
    let active = std::fs::read(&active_path).expect("floor active generation");
    let code = run(
        "temper-engine",
        &directories,
        &Request::Check([directories.data.as_ref(), b"/state"].concat().into()),
        "test-build",
    )
    .expect("upgrade reaches its report");
    assert_eq!(code, 0, "the floor rehearses through restores");
    assert_eq!(std::fs::read(active_path).expect("active after rehearsal"), active);
    assert!(!stale.exists(), "the stale scratch is removed whole");
    assert_eq!(std::fs::metadata(&outside).expect("outside file remains").nlink(), 1);
    assert_eq!(std::fs::read(busy.join("file")).expect("busy contents remain"), b"in use");
    drop(held);
    for name in ["upgrade-x", "other", "upgrade-00000000000000aA", "upgrade-00000000000000aaa"] {
        assert!(cache.join(name).is_dir(), "other name stays: {name}");
    }
    assert_eq!(std::fs::read(regular).expect("regular file stays"), b"regular file");
    assert!(link.is_symlink(), "the sweep does not follow symbolic links");
    assert_eq!(std::fs::read_dir(cache).expect("cache after rehearsal").count(), before - 1, "own scratch gone");
}

#[test]
fn a_rehearsals_own_scratch_is_swept_only_after_its_lock_is_released() {
    let cache = skein_scratch::Scratch::new("rehearsal-scratch");
    let (path, held) = super::scratch::make(cache.path()).expect("rehearsal makes and locks its scratch");
    let name = path.file_name().expect("scratch name");
    let seed = name.as_bytes().strip_prefix(b"upgrade-").expect("scratch prefix");
    assert_eq!(seed.len(), 16);
    assert!(seed.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)));
    let entries: Vec<_> = std::fs::read_dir(cache.path())
        .expect("cache entries")
        .map(|entry| entry.expect("cache entry").file_name())
        .collect();
    assert_eq!(entries, [name], "only the renamed scratch is visible");
    let second = std::fs::File::open(&path).expect("second scratch open");
    let Err(std::fs::TryLockError::WouldBlock) = second.try_lock() else { panic!("scratch must already be locked") };
    std::fs::write(path.join("file"), b"in use").expect("own scratch file");
    modified(&path, SystemTime::now() - Duration::from_secs(7200));
    assert_eq!(super::scratch::sweep("temper-engine", cache.path()), Vec::<String>::new());
    assert_eq!(std::fs::read(path.join("file")).expect("held scratch remains"), b"in use");
    drop(held);
    assert_eq!(super::scratch::sweep("temper-engine", cache.path()), Vec::<String>::new());
    assert!(!path.exists(), "the released scratch is stale");
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

fn modified(path: &Path, time: SystemTime) {
    std::fs::File::open(path).expect("directory opens for time").set_modified(time).expect("directory time");
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
fn own_scratch_removal_releases_all_contents_and_outside_links() {
    let cache = skein_scratch::Scratch::new("rehearsal-remove");
    let outside = cache.path().join("outside");
    std::fs::write(&outside, b"checkpoint bytes").expect("outside file");
    let (path, held) = super::scratch::make(cache.path()).expect("own scratch");
    std::fs::write(path.join("file"), b"left behind").expect("own file");
    std::fs::create_dir(path.join("nested")).expect("own directory");
    std::fs::hard_link(&outside, path.join("nested/linked")).expect("scratch link");
    assert_eq!(super::scratch::remove("temper-engine", &path, held), None);
    assert!(!path.exists(), "own scratch is removed whole");
    assert_eq!(std::fs::read(&outside).expect("outside file remains"), b"checkpoint bytes");
    assert_eq!(std::fs::metadata(&outside).expect("outside file").nlink(), 1);
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
fn a_refused_rehearsal_leaves_no_own_scratch_in_the_cache() {
    let (_scratch, directories) = deployment();
    let source = crate::path(&directories.data).join("state");
    let held = std::fs::File::open(&source).expect("source directory");
    held.try_lock().expect("source held by another run");
    let code = run("temper-engine", &directories, &Request::Check(source.as_os_str().as_bytes().into()), "test-build")
        .expect("rehearsal reaches its refusal");
    assert_eq!(code, 1, "the source lock refuses the rehearsal after scratch creation");
    let leftovers: Vec<_> = std::fs::read_dir(crate::path(&directories.cache))
        .expect("cache entries")
        .map(|entry| entry.expect("cache entry").file_name())
        .filter(|name| name.as_bytes().starts_with(b"upgrade-"))
        .collect();
    assert_eq!(leftovers, Vec::<std::ffi::OsString>::new());
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

#[test]
fn deployment_back_with_accounts_workers_and_forge_reports_no_previous_generation() {
    let (_scratch, directories) = deployment();
    let code = run("temper-engine", &directories, &Request::Back, "test-build").expect("back reaches its report");
    assert_eq!(code, 1, "floor has no previous generation");
}
