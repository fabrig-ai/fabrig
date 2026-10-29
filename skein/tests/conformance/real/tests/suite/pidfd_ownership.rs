//! Admission checks use fake candidates and send no signal.
#![cfg(target_os = "linux")]

use skein_io::kernel::Error;
use std::process::{Child, Command, Stdio};

struct Waiting(Child);
impl Waiting {
    fn start() -> Self {
        Self(Command::new("/bin/sh").args(["-c", "read line"]).stdin(Stdio::piped()).spawn().expect("waiting child"))
    }
}
impl Drop for Waiting {
    fn drop(&mut self) {
        // EOF ends the fixture naturally, including on an assertion failure.
        drop(self.0.stdin.take());
        let _reaped = self.0.wait();
    }
}

#[test]
fn a_late_open_rejects_an_unrelated_replacement_candidate() {
    let parent = Waiting::start();
    let replacement = Waiting::start();
    let result = skein_shell::open_child_pidfd(replacement.0.id(), parent.0.id());
    if let Ok(descriptor) = result {
        skein_shell::close_keeper_fd(descriptor);
    }
    assert_eq!(result, Err(Error::NotFound), "the opened candidate must prove the enumerated parent relationship");
}

#[test]
fn a_live_child_is_admitted_and_reserved_ids_are_refused() {
    let child = Waiting::start();
    let descriptor =
        skein_shell::open_child_pidfd(child.0.id(), std::process::id()).expect("our live child's identity");
    assert_eq!(skein_shell::child_id(descriptor), Ok(child.0.id()));
    skein_shell::close_keeper_fd(descriptor);
    for pid in [0, 1, u32::MAX] {
        assert_eq!(skein_shell::open_pidfd(pid), Err(Error::InvalidArgument));
        assert_eq!(skein_shell::open_child_pidfd(child.0.id(), pid), Err(Error::InvalidArgument));
    }
}
