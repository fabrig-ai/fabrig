//! Rehearsal scratch lifetime and stale removal (protocol/binaries.md, section 2).
use std::fs::{DirBuilder, File, TryLockError};
use std::os::unix::{ffi::OsStrExt, fs::DirBuilderExt};
use std::path::{Path, PathBuf};

/// Removal faults precede the rehearsal's report; an unreadable cache is silent.
pub(super) fn sweep(binary: &str, cache: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let Ok(entries) = std::fs::read_dir(cache) else { return lines };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(seed) = name.as_bytes().strip_prefix(b"upgrade-") else { continue };
        if seed.len() != 16 || !seed.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
            continue;
        }
        let Ok(kind) = entry.file_type() else { continue };
        if !kind.is_dir() {
            continue;
        }
        let path = entry.path();
        let Ok(file) = File::open(&path) else { continue };
        match file.try_lock() {
            Ok(()) => {
                if let Err(why) = std::fs::remove_dir_all(&path) {
                    let errno = why.raw_os_error().expect("directory removal fails with a kernel errno");
                    lines.push(format!(
                        "{binary} upgrade: scratch left behind not removed: {}: errno {errno}",
                        name.to_string_lossy()
                    ));
                }
            }
            Err(TryLockError::WouldBlock | TryLockError::Error(_)) => {}
        }
    }
    lines
}

pub(super) fn make(cache: &Path) -> Result<(PathBuf, File), String> {
    let seed = skein_shell::seed().map_err(|errno| format!("scratch seed: errno {errno}"))?;
    let path = cache.join(format!("upgrade-{seed:016x}"));
    DirBuilder::new().recursive(true).mode(0o700).create(&path).map_err(|why| failure(&why))?;
    let file = File::open(&path).map_err(|why| failure(&why))?;
    match file.try_lock() {
        Ok(()) => Ok((path, file)),
        Err(TryLockError::WouldBlock) => Err("scratch: busy".into()),
        Err(TryLockError::Error(why)) => Err(failure(&why)),
    }
}

pub(super) fn failure(why: &std::io::Error) -> String {
    let errno = why.raw_os_error().expect("scratch filesystem calls fail with a kernel errno");
    format!("scratch: errno {errno}")
}
