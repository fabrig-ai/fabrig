#![expect(clippy::disallowed_types, reason = "scratch fixtures set filesystem modification times without waiting")]
use std::{
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
    time::{Duration, SystemTime},
};
use temper_deployment_fixtures::deployment;
use temper_engine_shell::{
    testing::modified,
    upgrade::{Request, run},
};

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
fn deployment_back_with_accounts_workers_and_forge_reports_no_previous_generation() {
    let (_scratch, directories) = deployment();
    let code = run("temper-engine", &directories, &Request::Back, "test-build").expect("back reaches its report");
    assert_eq!(code, 1, "floor has no previous generation");
}
