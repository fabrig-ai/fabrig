//! Rehearsal scratch lifetime and stale removal (protocol/binaries.md, section 2).
use std::fs::{DirBuilder, DirEntry, File, TryLockError};
use std::os::unix::{ffi::OsStrExt, fs::DirBuilderExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Removal faults precede the rehearsal's report; an unreadable cache is silent.
#[expect(
    clippy::disallowed_types,
    reason = "the scratch sweep compares filesystem times to the wall clock (binaries.md, 2)"
)]
pub(super) fn sweep(binary: &str, cache: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let Ok(entries) = std::fs::read_dir(cache) else { return lines };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(seed) = name.as_bytes().strip_prefix(b"upgrade-") else { continue };
        if seed.len() != 16 || !seed.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
            continue;
        }
        if let Err(why) = sweep_entry(&entry, now)
            && why.kind() != std::io::ErrorKind::NotFound
        {
            lines.push(format!(
                "{binary} upgrade: scratch left behind not removed: {}: {}",
                name.to_string_lossy(),
                cause(&why)
            ));
        }
    }
    lines
}

#[expect(clippy::disallowed_types, reason = "filesystem modification times use SystemTime (binaries.md, 2)")]
fn sweep_entry(entry: &DirEntry, now: SystemTime) -> std::io::Result<()> {
    let metadata = entry.metadata()?;
    if !metadata.is_dir() {
        return Ok(());
    }
    let Ok(age) = now.duration_since(metadata.modified()?) else { return Ok(()) };
    if age < Duration::from_secs(3600) {
        return Ok(());
    }
    let path = entry.path();
    let file = File::open(&path)?;
    match file.try_lock() {
        Ok(()) => std::fs::remove_dir_all(&path),
        Err(TryLockError::WouldBlock) => Ok(()),
        Err(TryLockError::Error(why)) => Err(why),
    }
}

pub(super) fn make(cache: &Path) -> Result<(PathBuf, File), String> {
    let seed = skein_shell::seed().map_err(|errno| format!("scratch seed: errno {errno}"))?;
    let path = cache.join(format!("upgrade-{seed:016x}"));
    let staging = cache.join(format!("upgrade-{seed:016x}.new"));
    DirBuilder::new().recursive(true).mode(0o700).create(&staging).map_err(|why| failure(&why))?;
    match lock_and_rename(&staging, &path) {
        Ok(file) => Ok((path, file)),
        Err(why) => {
            let _removed = std::fs::remove_dir_all(staging);
            Err(why)
        }
    }
}

fn lock_and_rename(staging: &Path, path: &Path) -> Result<File, String> {
    let file = File::open(staging).map_err(|why| failure(&why))?;
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Err("scratch: busy".into()),
        Err(TryLockError::Error(why)) => return Err(failure(&why)),
    }
    std::fs::rename(staging, path).map_err(|why| failure(&why))?;
    Ok(file)
}

/// Hold the rehearsal's lock until all its scratch contents have been removed.
pub(super) fn remove(binary: &str, path: &Path, held: File) -> Option<String> {
    let result = std::fs::remove_dir_all(path);
    drop(held);
    result.err().map(|why| {
        format!(
            "{binary} upgrade: scratch not removed: {}: {}",
            path.file_name().expect("scratch has a name").to_string_lossy(),
            cause(&why)
        )
    })
}

fn cause(why: &std::io::Error) -> String {
    match why.raw_os_error() {
        Some(errno) => format!("errno {errno}"),
        None => format!("kind {:?}", why.kind()),
    }
}

pub(super) fn failure(why: &std::io::Error) -> String {
    format!("scratch: {}", cause(why))
}
