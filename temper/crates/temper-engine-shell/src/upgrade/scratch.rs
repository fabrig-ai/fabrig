//! Rehearsal scratch lifetime and stale removal (protocol/binaries.md, section 2).
use std::ffi::OsString;
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
    let Ok(entries) = std::fs::read_dir(cache) else { return Vec::new() };
    let now = SystemTime::now();
    sweep_with(
        binary,
        entries.flatten().map(|entry| (entry.file_name(), entry)),
        now,
        &mut (),
        |(), entry| modified_directory(entry),
        |(), entry| try_hold(&entry.path()),
        |(), entry, held| {
            let removed = std::fs::remove_dir_all(entry.path());
            drop(held);
            removed
        },
    )
}

#[expect(clippy::disallowed_types, reason = "filesystem modification times use SystemTime (binaries.md, 2)")]
pub(super) fn sweep_with<T, S, Held>(
    binary: &str,
    entries: impl IntoIterator<Item = (OsString, T)>,
    now: SystemTime,
    state: &mut S,
    mut modified_directory: impl FnMut(&mut S, &T) -> std::io::Result<Option<SystemTime>>,
    mut try_hold: impl FnMut(&mut S, &T) -> std::io::Result<Option<Held>>,
    mut remove_leftover: impl FnMut(&mut S, &T, Held) -> std::io::Result<()>,
) -> Vec<String> {
    let mut lines = Vec::new();
    for (name, entry) in entries {
        let Some(seed) = name.as_bytes().strip_prefix(b"upgrade-") else { continue };
        if seed.len() != 16 || !seed.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
            continue;
        }
        let result = (|| {
            let old = modified_directory(state, &entry)?
                .and_then(|modified| now.duration_since(modified).ok())
                .is_some_and(|age| age >= Duration::from_secs(3600));
            if old && let Some(held) = try_hold(state, &entry)? {
                remove_leftover(state, &entry, held)?;
            }
            Ok::<(), std::io::Error>(())
        })();
        if let Err(why) = result
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
fn modified_directory(entry: &DirEntry) -> std::io::Result<Option<SystemTime>> {
    let metadata = entry.metadata()?;
    if !metadata.is_dir() {
        return Ok(None);
    }
    metadata.modified().map(Some)
}

fn try_hold(path: &Path) -> std::io::Result<Option<File>> {
    let file = File::open(path)?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
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
    remove_with(binary, path, held, &mut (), |(), path| std::fs::remove_dir_all(path), |(), held| drop(held))
}

pub(super) fn remove_with<S, Held>(
    binary: &str,
    path: &Path,
    held: Held,
    state: &mut S,
    remove: impl FnOnce(&mut S, &Path) -> std::io::Result<()>,
    release: impl FnOnce(&mut S, Held),
) -> Option<String> {
    let result = remove(state, path);
    release(state, held);
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
