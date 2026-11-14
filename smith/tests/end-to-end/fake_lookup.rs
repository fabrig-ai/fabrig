//! The fake program's sibling lookup (smith/docs/design/testing.md, 2.3).

use super::support::fake_claude_beside;
use skein_scratch::Scratch;
use std::{fs, os::unix::fs::PermissionsExt as _};

#[test]
fn an_executable_fake_is_found_beside_smith() {
    let scratch = Scratch::new("smith-fake-sibling");
    let smith = scratch.path().join("smith");
    let fake = scratch.path().join("fake-claude");
    fs::write(&smith, b"").expect("smith file");
    fs::write(&fake, b"").expect("fake file");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).expect("executable fake");
    assert_eq!(fake_claude_beside(&smith), Ok(fake), "the fake is smith's sibling");
}

#[test]
fn a_missing_fake_names_its_build_command() {
    let scratch = Scratch::new("smith-fake-missing");
    let refusal = fake_claude_beside(&scratch.path().join("smith")).expect_err("fake lookup is refused");
    assert!(refusal.contains("cargo build -p smith-fake-claude --bin fake-claude"), "{refusal}");
}

#[test]
fn a_nonexecutable_fake_names_its_build_command() {
    let scratch = Scratch::new("smith-fake-nonexecutable");
    let fake = scratch.path().join("fake-claude");
    fs::write(&fake, b"").expect("fake file");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o600)).expect("nonexecutable fake");
    let refusal = fake_claude_beside(&scratch.path().join("smith")).expect_err("fake lookup is refused");
    assert!(refusal.contains("cargo build -p smith-fake-claude --bin fake-claude"), "{refusal}");
}
