//! Calls shared by Linux and Darwin (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Shutdown, close and fcntl take only integers; close relinquishes descriptor
//! ownership once. Geteuid takes no arguments and has no preconditions.
//! Poll borrows one initialized, exclusive pollfd until return, retains no
//! pointer and takes no descriptor ownership. The parent-death watcher prepares
//! its pipe, signal set and integer plan before fork, then makes only
//! async-signal-safe calls and the system's exit wait until _exit; its module
//! documents that process boundary. Uname writes one exclusive local utsname;
//! on success the system NUL-terminates nodename, which is borrowed only while
//! that local lives.

#![expect(
    unsafe_code,
    reason = "POSIX calls take integers or borrow storage only until return (runtime/readiness.md, 5)"
)]

pub(crate) mod files;
pub(crate) mod group;
#[cfg(any(target_os = "macos", test))]
pub(crate) mod parent_death;
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

/// The host's name, as the kernel gives it (runtime/shell.md, 6.1).
pub fn host_name() -> Result<Box<[u8]>, i32> {
    // SAFETY: utsname is a plain C structure of byte arrays, valid when zeroed.
    let mut name: libc::utsname = unsafe { std::mem::zeroed() };
    // SAFETY: name is exclusive and writable for one utsname throughout the call.
    status(unsafe { libc::uname(ptr::from_mut(&mut name)) })?;
    // SAFETY: successful uname NUL-terminated nodename, which lives across this borrow.
    let hostname = unsafe { std::ffi::CStr::from_ptr(name.nodename.as_ptr()) };
    Ok(Box::from(hostname.to_bytes()))
}

/// A user's login at startup (runtime/shell.md, 6). macOS's directory
/// service need not list the effective user in `/etc/passwd`.
pub fn login(user: u32) -> Result<Box<[u8]>, String> {
    if cfg!(target_os = "macos") { crate::login_id() } else { login_passwd(user) }
}

/// A user's login from `/etc/passwd`, also used to check the command lookup.
pub fn login_passwd(user: u32) -> Result<Box<[u8]>, String> {
    let passwd = crate::read_file(std::path::Path::new("/etc/passwd"), 1 << 20).map_err(login_read_error)?;
    passwd_login(&passwd, user)
}

fn passwd_login(passwd: &[u8], user: u32) -> Result<Box<[u8]>, String> {
    for line in passwd.split(|byte| *byte == b'\n') {
        let mut fields = line.split(|byte| *byte == b':');
        let name = fields.next();
        let uid = fields.nth(1).and_then(|id| std::str::from_utf8(id).ok()).and_then(|id| id.parse::<u32>().ok());
        if uid == Some(user) {
            return Ok(name.expect("a split has a first field").into());
        }
    }
    Err(format!("user {user}: no login in /etc/passwd"))
}

fn login_read_error(why: crate::Unread) -> String {
    use crate::Unread;
    use skein_io::kernel::Error;
    match why {
        Unread::Absent => "user: absent".into(),
        Unread::NotAFile => "user: not a file".into(),
        Unread::TooLarge { max } => format!("user: past {max} bytes"),
        Unread::Unsafe(_) => "user: readable by others or reached through a link".into(),
        Unread::Kernel(error) => {
            let errno = match error {
                Error::Reach(why) => return format!("user: {why}"),
                Error::Refused => 111,
                Error::Busy => 11,
                Error::Reset => 104,
                Error::BrokenPipe => 32,
                Error::NotConnected => 107,
                Error::AddressInUse => 98,
                Error::AddressNotAvailable => 99,
                Error::Unreachable => 101,
                Error::TimedOut => 110,
                Error::TooManyOpenFiles => 24,
                Error::NoBufferSpace => 12,
                Error::Cancelled => 125,
                Error::TooLate | Error::NotFound => 2,
                Error::Exists => 17,
                Error::NotADirectory => 20,
                Error::IsADirectory => 21,
                Error::NotEmpty => 39,
                Error::Permission => 13,
                Error::NoSpace => 28,
                Error::ReadOnly => 30,
                Error::TooManyLinks => 40,
                Error::NameTooLong => 36,
                Error::Escape => 18,
                Error::NotAFile => 6,
                Error::InvalidArgument => 22,
                Error::Other(number) => number,
            };
            format!("user: errno {errno}")
        }
    }
}

#[cfg(test)]
mod login_tests {
    #[test]
    fn passwd_login_matches_numeric_uid_and_refuses_missing_user() {
        let passwd = b"root:x:0:0:root:/root:/bin/sh\nmalformed\nbad:x:not-a-uid:42\nalice:x:1042:42::/home/alice:/bin/sh\nbob:x:2042:42::/home/bob:/bin/sh\n";
        assert_eq!(super::passwd_login(passwd, 1042), Ok(Box::from(b"alice".as_slice())));
        assert_eq!(super::passwd_login(passwd, 42), Err("user 42: no login in /etc/passwd".into()));
    }
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
