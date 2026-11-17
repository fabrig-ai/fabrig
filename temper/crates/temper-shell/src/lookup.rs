//! Executables on a process's absolute PATH entries (protocol/configuration/moved.md, 2).
use crate::directories::EnvironmentEntry;
use std::{
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::PathBuf,
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
#[must_use]
pub fn find(environment: &[EnvironmentEntry], name: &[u8]) -> Option<PathBuf> {
    let entries = candidates(environment, name)
        .into_iter()
        .map(|path| {
            let executable =
                std::fs::metadata(&path).is_ok_and(|metadata| metadata.is_file() && metadata.mode() & 0o111 != 0);
            (path, executable)
        })
        .collect::<Vec<_>>();
    lookup(environment, name, &entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};
    #[test]
    fn git_lookup_skips_missing_nonexecutable_and_nonabsolute_entries() {
        let scratch = skein_scratch::Scratch::new("git-lookup");
        let directory = scratch.path();
        let git = directory.join("git");
        fs::write(&git, b"#!/bin/sh\nexit 0\n").expect("scratch git");
        let value = format!("/nonexistent:{}", directory.display());
        let env = vec![(Box::from(b"PATH".as_slice()), Box::from(value.as_bytes()))];
        fs::set_permissions(&git, fs::Permissions::from_mode(0o700)).expect("executable");
        assert_eq!(find(&env, b"git"), Some(git.clone()));
        fs::set_permissions(&git, fs::Permissions::from_mode(0o600)).expect("nonexecutable");
        assert_eq!(find(&env, b"git"), None);
        fs::set_permissions(&git, fs::Permissions::from_mode(0o700)).expect("executable again");
        let relative = directory.strip_prefix("/").expect("absolute scratch");
        let value = format!(":{}", relative.display());
        let env = vec![(Box::from(b"PATH".as_slice()), Box::from(value.as_bytes()))];
        assert_eq!(candidates(&env, b"git"), Vec::<PathBuf>::new());
        assert_eq!(find(&env, b"git"), None);
    }
}
