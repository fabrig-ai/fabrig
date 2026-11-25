//! Pin the narrow PATH for shipped-binary stories (smith/docs/design/testing-tiers.md, section 3).

use crate::support::{stories_path_from_rg, stories_path_from_test_path};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[test]
fn stories_prepend_only_a_nonsystem_rg_directory() {
    for (rg, expected) in [
        (Some("/usr/bin/rg"), "/usr/bin:/bin"),
        (Some("/bin/rg"), "/usr/bin:/bin"),
        (Some("/opt/homebrew/bin/rg"), "/opt/homebrew/bin:/usr/bin:/bin"),
        (None, "/usr/bin:/bin"),
    ] {
        assert_eq!(stories_path_from_rg(rg.map(Path::new)), expected, "story PATH for {rg:?}");
    }
}

#[test]
fn stories_find_rg_on_the_supplied_test_path() {
    let scratch = skein_scratch::Scratch::new("stories-path");
    let directory = scratch.path().as_os_str();
    let rg = scratch.path().join("rg");
    fs::write(&rg, b"#!/bin/sh\nexit 0\n").expect("write rg stand-in");
    fs::set_permissions(&rg, fs::Permissions::from_mode(0o755)).expect("make rg stand-in executable");
    let mut expected = directory.to_os_string();
    expected.push(":/usr/bin:/bin");
    assert_eq!(stories_path_from_test_path(Some(directory)), expected, "use the rg on the supplied PATH");
}
