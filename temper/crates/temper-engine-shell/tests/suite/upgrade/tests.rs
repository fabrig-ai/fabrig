#![expect(clippy::disallowed_types, reason = "scratch fixtures set filesystem modification times without waiting")]
use std::{
    os::unix::ffi::OsStrExt,
    time::{Duration, SystemTime},
};
use temper_engine_shell::testing::modified;

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
