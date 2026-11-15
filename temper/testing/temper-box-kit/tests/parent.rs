//! Executable observations retain Linux process identities
//! (temper/docs/design/testing/process-tier.md, the newer-build story).
#![cfg(target_os = "linux")]

use std::{
    fs,
    os::unix::fs::MetadataExt,
    process::{Command, Stdio},
};
use temper_box_kit::parent::Guard;

#[path = "../src/parent/executable_read.rs"]
mod executable_read;

#[test]
fn a_reaped_child_has_no_executable() {
    let child = Command::new("/bin/cat").stdin(Stdio::piped()).stdout(Stdio::null()).spawn().expect("owned child");
    let pid = child.id();
    let mut guard = Guard::new(child);
    guard.record(pid, std::process::id());
    let executable = guard.executable(pid).expect("live executable read").expect("cat is alive");
    let expected = fs::metadata("/bin/cat").expect("installed cat");
    let observed = executable.file.metadata().expect("observed executable");
    assert_eq!((observed.dev(), observed.ino()), (expected.dev(), expected.ino()));
    assert_eq!(fs::metadata(executable.path).expect("executable link").ino(), expected.ino());
    assert!(executable.arguments.split(|byte| *byte == 0).any(|argument| argument == b"/bin/cat"));
    let child = guard.child.as_mut().expect("owned child");
    child.kill().expect("kill child");
    child.wait().expect("reap child");
    assert!(guard.executable(pid).expect("exited is not unreadable").is_none());
}

#[test]
fn an_unreadable_executable_waits_for_exit_before_it_is_judged() {
    use executable_read::{Read, observe};

    // The task has dropped its memory map, but its pidfd is not readable yet.
    let missing = || std::io::Error::from(std::io::ErrorKind::NotFound);
    let Read::Wait(error) = observe::<()>(Err(missing()), false, false) else {
        panic!("an unreadable executable with no exit notification must wait");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    let Read::Complete(exited) = observe::<()>(Err(error), true, true) else {
        panic!("a confirmed exit must finish the read");
    };
    assert!(exited.expect("an exiting process is not unreadable").is_none());

    let Read::Complete(alive) = observe::<()>(Err(missing()), false, true) else {
        panic!("a bounded wait must finish even without an exit");
    };
    assert_eq!(alive.expect_err("a still-live process stays unreadable").kind(), std::io::ErrorKind::NotFound);

    let Read::Complete(live) = observe(Ok(7), false, false) else {
        panic!("a readable process must not wait");
    };
    assert_eq!(live.expect("readable executable"), Some(7));
    let Read::Complete(exited) = observe(Ok(7), true, false) else {
        panic!("an observed exit must not wait");
    };
    assert_eq!(exited.expect("exit after a successful read"), None);
}
