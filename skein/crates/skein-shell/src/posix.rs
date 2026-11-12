//! Calls shared by Linux and Darwin (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Shutdown, close and fcntl take only integers; close relinquishes descriptor
//! ownership once. Geteuid takes no arguments and has no preconditions.
//! Poll borrows one initialized, exclusive pollfd until return, retains no
//! pointer and takes no descriptor ownership.

#![expect(
    unsafe_code,
    reason = "POSIX calls take integers or borrow storage only until return (runtime/readiness.md, 5)"
)]

pub(crate) mod files;
pub(crate) mod group;
pub(crate) mod process;
pub(crate) mod terminal;

use crate::system::last_errno;
use skein_io::kernel::Fd;
use std::ptr;

/// Interrupted calls have made no progress; retry without changing counts.
pub(crate) fn retry<T>(mut call: impl FnMut() -> Result<T, i32>) -> Result<T, i32> {
    loop {
        match call() {
            Err(libc::EINTR) => {}
            result => return result,
        }
    }
}

pub(crate) fn status(result: i32) -> Result<(), i32> {
    if result < 0 { Err(last_errno()) } else { Ok(()) }
}

/// The status flags of what is open on `fd` (`F_GETFL`), or the error
/// number.
pub(crate) fn status_flags(fd: i32) -> Result<i32, i32> {
    // SAFETY: integers only.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 { Err(last_errno()) } else { Ok(flags) }
}

/// Whether a startup descriptor is open for writing. Reads its flags without
/// taking ownership or changing them (runtime/shell.md, 6).
pub fn descriptor_writes(fd: Fd) -> Result<bool, i32> {
    retry(|| status_flags(fd.raw())).map(|flags| flags & libc::O_ACCMODE != libc::O_RDONLY)
}

pub(crate) fn nonblocking(fd: Fd) -> Result<(), i32> {
    let flags = retry(|| status_flags(fd.raw()))?;
    retry(|| {
        // SAFETY: fcntl takes integers and fd is the backend's own description.
        status(unsafe { libc::fcntl(fd.raw(), libc::F_SETFL, flags | libc::O_NONBLOCK) })
    })
}

pub(crate) fn close(fd: Fd) -> Result<(), i32> {
    // Never retry: a retry could close a reused descriptor.
    // SAFETY: close takes only the descriptor, whose ownership io relinquishes.
    status(unsafe { libc::close(fd.raw()) })
}

/// The process effective user, read once by startup for io's private files (runtime/shell.md, section 6).
#[must_use]
pub fn effective_user() -> u32 {
    // SAFETY: geteuid takes no pointers and has no preconditions.
    unsafe { libc::geteuid() }
}

/// An `EINTR` is returned, not retried (runtime/readiness.md, 4).
pub(crate) fn shutdown(fd: Fd) -> Result<(), i32> {
    // SAFETY: shutdown takes only integers.
    status(unsafe { libc::shutdown(fd.raw(), libc::SHUT_WR) })
}

/// Waits up to `bound` for `POLLIN`, without consuming input or driving a
/// `Kernel`. Returns false on timeout and an errno on descriptor or poll
/// errors. Interrupted waits keep the same monotonic deadline.
pub fn wait_readable(fd: Fd, bound: skein_lib::Duration) -> Result<bool, i32> {
    if fd.raw() < 0 {
        return Err(libc::EBADF);
    }
    let clock = crate::Clock::new();
    let deadline = clock.now().now.saturating_add(bound);
    loop {
        let left = deadline.saturating_since(clock.now().now);
        let millis = i32::try_from(left.as_nanos().div_ceil(1_000_000)).unwrap_or(i32::MAX);
        let mut watched = libc::pollfd { fd: fd.raw(), events: libc::POLLIN, revents: 0 };
        // SAFETY: watched is one initialized, exclusive pollfd, writable until
        // poll returns. The call retains no pointer and takes no ownership of fd.
        let result = unsafe { libc::poll(ptr::from_mut(&mut watched), 1, millis) };
        if result < 0 {
            let errno = last_errno();
            if errno == libc::EINTR && clock.now().now < deadline {
                continue;
            }
            return Err(errno);
        }
        if watched.revents & libc::POLLNVAL != 0 {
            return Err(libc::EBADF);
        }
        if watched.revents & (libc::POLLERR | libc::POLLHUP) != 0 {
            return Err(libc::EIO);
        }
        if watched.revents & libc::POLLIN != 0 {
            return Ok(true);
        }
        if clock.now().now >= deadline {
            return Ok(false);
        }
    }
}

#[cfg(test)]
mod descriptor_tests {
    use super::descriptor_writes;
    use skein_io::kernel::Fd;
    use std::os::fd::AsRawFd;

    #[test]
    fn descriptor_write_access_keeps_the_open_mode() {
        for (read, write, expected) in [(true, false, false), (false, true, true), (true, true, true)] {
            let file = std::fs::OpenOptions::new().read(read).write(write).open("/dev/null").expect("null");
            assert_eq!(descriptor_writes(Fd::new(file.as_raw_fd())), Ok(expected));
            assert_eq!(descriptor_writes(Fd::new(file.as_raw_fd())), Ok(expected), "flags unchanged");
        }
        assert!(descriptor_writes(Fd::new(-1)).is_err());
    }
}
