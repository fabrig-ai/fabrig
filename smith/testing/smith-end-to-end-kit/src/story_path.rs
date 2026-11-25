//! The shipped-binary stories' PATH, derived from the test process's rg location.
//! Keeps no state and passes on no other directory from the test process's PATH
//! (smith/docs/design/testing-tiers.md, section 3).

use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::Path,
};

const SYSTEM_DIRECTORIES: [&str; 2] = ["/usr/bin", "/bin"];

/// Put rg's directory, if outside the two system directories, before them.
#[must_use]
pub fn stories_path_from_rg(rg: Option<&Path>) -> OsString {
    let directory = rg
        .and_then(Path::parent)
        .filter(|directory| SYSTEM_DIRECTORIES.iter().all(|system| directory.as_os_str() != *system));
    std::env::join_paths(directory.into_iter().chain(SYSTEM_DIRECTORIES.map(Path::new)))
        .expect("story PATH directories contain no path separator")
}

/// Find rg as smith does, then build the stories' PATH from its location.
#[must_use]
pub fn stories_path_from_test_path(test_path: Option<&OsStr>) -> OsString {
    let rg = smith_agent_shell::config::find_rg(test_path.map(OsStrExt::as_bytes));
    stories_path_from_rg(rg.as_deref())
}
