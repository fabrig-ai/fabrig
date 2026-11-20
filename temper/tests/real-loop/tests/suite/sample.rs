//! The sample uses the real shell, the lowest tier admitting child programs
//! (skein/docs/foundation/testing-strategy.md, 2.8; protocol/setup/owned.md, 7).
use std::{fs, process::Command};

#[test]
fn sample_checks_pass_for_the_greetings_and_fail_for_a_broken_hello() {
    let scratch = skein_scratch::Scratch::new("sample-greetings");
    fs::write(scratch.path().join("hello"), temper_forge_setup::SAMPLE_HELLO).unwrap();
    fs::write(scratch.path().join("test.sh"), temper_forge_setup::SAMPLE_TEST).unwrap();
    let checked = Command::new("/bin/sh").arg("test.sh").current_dir(scratch.path()).output().unwrap();
    assert_eq!(checked.status.code(), Some(0));
    assert!(checked.stderr.is_empty(), "{:?}", checked.stderr);
    let stdout = std::str::from_utf8(&checked.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 2);
    for line in stdout.lines() {
        assert!(line.starts_with("ok:"), "{line}");
    }

    fs::write(scratch.path().join("hello"), b"#!/bin/sh\nprintf 'Hi, world!\\n'\n").unwrap();
    let broken = Command::new("/bin/sh").arg("test.sh").current_dir(scratch.path()).output().unwrap();
    assert_eq!(broken.status.code(), Some(1));
    assert!(broken.stderr.is_empty(), "{:?}", broken.stderr);
    assert!(std::str::from_utf8(&broken.stdout).unwrap().lines().any(|line| line.starts_with("failed:")));
}
