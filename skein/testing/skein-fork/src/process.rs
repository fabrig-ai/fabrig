//! The process boundary (`skein/docs/design/testing/README.md`, 6).
//!
//! # The `unsafe`, and why it is sound
//!
//! The test world runs on one thread, with libtest's main thread waiting.
//! Fork retains that world's stack and owned memory. The child never returns
//! to libtest: success and the first panic use `_exit`, without unwinding.
//! Waitpid borrows local status storage only until return and reaps only our child.
#![expect(
    unsafe_code,
    reason = "fork, waitpid and _exit form the test's process boundary (skein/docs/design/testing/README.md, 6)"
)]

use crate::{Exit, Split};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

static CHILD: AtomicBool = AtomicBool::new(false);

fn end(code: i32) -> ! {
    // SAFETY: _exit takes only an exit code and never returns into libtest.
    unsafe { libc::_exit(code) }
}

fn child() -> Split {
    CHILD.store(true, Ordering::Relaxed);
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        hook(info);
        end(101);
    }));
    Split::Child
}

/// Fork the process and wait in the parent. A child cannot split again.
#[must_use]
pub fn split() -> Split {
    assert!(!CHILD.load(Ordering::Relaxed), "a forked child cannot split again");
    std::io::stdout().flush().expect("flush stdout before fork");
    std::io::stderr().flush().expect("flush stderr before fork");
    // SAFETY: the world owns its state on this thread; the child cannot return
    // to libtest and cannot fork again. Neither side shares writes after fork.
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
    if pid == 0 {
        return child();
    }
    let mut status = 0;
    loop {
        // SAFETY: pid names our child; status is writable local storage which
        // waitpid does not retain. No other code waits for this child.
        let waited = unsafe { libc::waitpid(pid, &raw mut status, 0) };
        if waited == pid {
            break;
        }
        let error = std::io::Error::last_os_error();
        assert_eq!(error.raw_os_error(), Some(libc::EINTR), "waitpid failed: {error}");
    }
    if libc::WIFEXITED(status) {
        Split::Parent(Exit::Code(libc::WEXITSTATUS(status)))
    } else {
        assert!(libc::WIFSIGNALED(status), "waitpid returned no terminal status: {status}");
        Split::Parent(Exit::Signal(libc::WTERMSIG(status)))
    }
}

/// End a child successfully; return in a process which is not a child.
pub fn passed() {
    if CHILD.load(Ordering::Relaxed) {
        end(0);
    }
}
