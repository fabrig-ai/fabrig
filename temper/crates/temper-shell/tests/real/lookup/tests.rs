use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
use temper_shell::lookup::{candidates, find};

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
