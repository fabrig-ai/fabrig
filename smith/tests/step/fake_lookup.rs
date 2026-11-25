//! Decide the sibling program's lookup from file facts (smith/docs/design/testing-tiers.md, 3).

use crate::support::{FakeFile, FakeLookup};
use std::{path::Path, path::PathBuf};

fn beside_smith() -> FakeLookup {
    FakeLookup::beside(Path::new("profile/smith"))
}

fn assert_needs_build(file: FakeFile) {
    let refusal = beside_smith().decide(file).expect_err("fake lookup is refused");
    assert!(refusal.contains("cargo build -p smith-fake-claude --bin fake-claude"), "{refusal}");
}

#[test]
fn an_executable_fake_is_found_beside_smith() {
    assert_eq!(beside_smith().decide(FakeFile::Regular { mode: 0o700 }), Ok(PathBuf::from("profile/fake-claude")));
}

#[test]
fn a_missing_fake_names_its_build_command() {
    assert_needs_build(FakeFile::Absent);
}

#[test]
fn a_nonexecutable_fake_names_its_build_command() {
    assert_needs_build(FakeFile::Regular { mode: 0o600 });
}
