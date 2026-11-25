//! Executables on a process's absolute PATH entries (protocol/configuration/moved.md, 2).
use crate::directories::EnvironmentEntry;
use skein_io::kernel::Kind;
use std::{
    io,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
};
/// Candidate names only: relative PATH entries are never searched.
#[must_use]
pub fn candidates(environment: &[EnvironmentEntry], name: &[u8]) -> Vec<PathBuf> {
    let Some(value) = crate::environment::PATH.value(environment) else { return Vec::new() };
    value
        .split(|byte| *byte == b':')
        .filter(|entry| entry.starts_with(b"/"))
        .map(|entry| crate::path(entry).join(std::ffi::OsStr::from_bytes(name)))
        .collect()
}
/// Pure choice over supplied facts; the shell alone inspects candidates.
#[must_use]
pub fn lookup(environment: &[EnvironmentEntry], name: &[u8], facts: &[(PathBuf, bool)]) -> Option<PathBuf> {
    candidates(environment, name)
        .into_iter()
        .find(|candidate| facts.iter().any(|(path, executable)| path == candidate && *executable))
}
/// The same executable-file check for named paths and PATH candidates.
#[must_use]
pub fn is_executable(path: &Path) -> bool {
    is_executable_with(path, metadata)
}

fn metadata(path: &Path) -> io::Result<(Kind, u32)> {
    let facts = std::fs::metadata(path)?;
    let kind = if facts.is_file() {
        Kind::File
    } else if facts.is_dir() {
        Kind::Directory
    } else {
        Kind::Other
    };
    Ok((kind, facts.mode()))
}

fn is_executable_with(path: &Path, metadata: impl FnOnce(&Path) -> io::Result<(Kind, u32)>) -> bool {
    path.is_absolute() && metadata(path).is_ok_and(|(kind, mode)| kind == Kind::File && mode & 0o111 != 0)
}

#[must_use]
pub fn find(environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
    find_with(environment, name, metadata)
}

fn find_with(
    environment: &[EnvironmentEntry],
    name: &[u8],
    mut metadata: impl FnMut(&Path) -> io::Result<(Kind, u32)>,
) -> Option<PathBuf> {
    let entries = candidates(environment, name)
        .into_iter()
        .map(|path| {
            let executable = is_executable_with(&path, &mut metadata);
            (path, executable)
        })
        .collect::<Vec<_>>();
    lookup(environment, name, &entries)
}

#[cfg(test)]
mod tests;
