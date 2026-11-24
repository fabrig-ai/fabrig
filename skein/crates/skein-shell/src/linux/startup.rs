//! Startup's synchronous bounded regular-file reads (runtime/shell-startup.md, 1.1).
//! Every descriptor stays here and is closed before returning bytes or a
//! refusal. No service or ring state is kept.
//!
//! # The `unsafe`, and why it is sound
//!
//! Open calls read live NUL-terminated paths and initialized `open_how`
//! fields with their exact size. No call retains a pointer. Every descriptor
//! opened here is closed before return.

use crate::posix::files::{check_private, kernel_error, private_error, read_opened};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use skein_io::kernel::Error;
use skein_lib::Secret;

use crate::startup::Unread;

use super::{close_unchecked as close, last_errno};

/// Leaves Linux's path as given for a caller's own link checks (runtime/shell-startup.md, 1.1).
#[must_use]
pub fn resolve_system_links(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub(crate) fn read_private_file(path: &Path, max: u32, user: u32) -> Result<Secret, Unread> {
    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| Unread::Kernel(Error::InvalidArgument))?;
    let flags = libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOCTTY | libc::O_CLOEXEC;
    let fd = match open_private(&path, flags) {
        Ok(fd) => fd,
        Err(libc::EACCES) => return Err(denied_private(&path, user)),
        Err(errno) => return Err(private_error(errno)),
    };
    let result = check_private(fd, user).and_then(|()| read_opened(fd, max)).map(Secret::new);
    close(fd);
    result
}

fn open_private(path: &CString, flags: i32) -> Result<i32, i32> {
    let how = [u64::try_from(flags).expect("positive flags"), 0, libc::RESOLVE_NO_SYMLINKS];
    // SAFETY: the NUL-terminated path and all three initialized open_how
    // fields remain live across the syscall, which retains no pointers.
    let fd =
        unsafe { libc::syscall(libc::SYS_openat2, libc::AT_FDCWD, path.as_ptr(), how.as_ptr(), size_of_val(&how)) };
    if fd < 0 { Err(last_errno()) } else { Ok(i32::try_from(fd).expect("a descriptor fits i32")) }
}

fn denied_private(path: &CString, user: u32) -> Unread {
    let Ok(fd) = open_private(path, libc::O_PATH | libc::O_NOFOLLOW | libc::O_CLOEXEC) else {
        return Unread::Kernel(Error::Permission);
    };
    let checked = check_private(fd, user);
    close(fd);
    match checked {
        Err(Unread::Unsafe(found)) => Unread::Unsafe(found),
        Ok(()) | Err(Unread::Absent | Unread::NotAFile | Unread::TooLarge { .. } | Unread::Kernel(_)) => {
            Unread::Kernel(Error::Permission)
        }
    }
}

pub(crate) fn read_file(path: &Path, max: u32) -> Result<Box<[u8]>, Unread> {
    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| Unread::Kernel(Error::InvalidArgument))?;
    // SAFETY: the NUL-terminated path remains live across open.
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOCTTY | libc::O_CLOEXEC) };
    if fd < 0 {
        let errno = last_errno();
        return if errno == libc::ENOENT { Err(Unread::Absent) } else { Err(Unread::Kernel(kernel_error(errno))) };
    }
    let result = read_opened(fd, max);
    close(fd);
    result
}
