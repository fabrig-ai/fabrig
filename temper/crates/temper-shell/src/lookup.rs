//! Executables on a process's absolute PATH entries (protocol/configuration/moved.md, 2).
use crate::directories::EnvironmentEntry;
use std::{
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
    path.is_absolute()
        && std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.mode() & 0o111 != 0)
}
#[must_use]
pub fn find(environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
    let entries = candidates(environment, name)
        .into_iter()
        .map(|path| {
            let executable = is_executable(&path);
            (path, executable)
        })
        .collect::<Vec<_>>();
    lookup(environment, name, &entries)
}
