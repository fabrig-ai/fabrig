//! Executable observations retain Linux process identities
//! (temper/docs/design/testing/process-tier.md, the newer-build story).
#![cfg(target_os = "linux")]

use std::{
    fs,
    os::unix::fs::MetadataExt,
    process::{Command, Stdio},
};
use temper_box_kit::parent::Guard;

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
