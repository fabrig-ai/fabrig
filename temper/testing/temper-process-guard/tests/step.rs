//! Executable observations over scripted read and exit results.
#![cfg(target_os = "linux")]
use temper_process_guard::executable_read;

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
