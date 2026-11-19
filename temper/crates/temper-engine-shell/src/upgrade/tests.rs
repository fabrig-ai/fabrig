//! The upgrade command on a deployment copy (protocol/binaries.md, 1).
use super::{Request, run};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
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
    let busy = cache.join("upgrade-00000000000000bb");
    std::fs::create_dir_all(&busy).expect("in-use scratch");
    let held = std::fs::File::open(&busy).expect("in-use directory opens");
    held.try_lock().expect("exclusive scratch lock");
    std::fs::write(busy.join("file"), b"in use").expect("in-use file");
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
    std::fs::write(path.join("file"), b"in use").expect("own scratch file");
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

#[test]
fn deployment_back_with_accounts_workers_and_forge_reports_no_previous_generation() {
    let (_scratch, directories) = deployment();
    let code = run("temper-engine", &directories, &Request::Back, "test-build").expect("back reaches its report");
    assert_eq!(code, 1, "floor has no previous generation");
}
