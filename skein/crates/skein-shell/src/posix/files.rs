//! Startup file reads and error meanings shared by Linux and Darwin (shell.md, 6.1 and 7.5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Fstat writes initialized local stat storage; read writes only within the
//! exclusive buffer's supplied length. No call retains a pointer. The caller
//! owns the descriptor until it closes it after this read returns.

#![expect(unsafe_code, reason = "file calls borrow local storage only until return (shell.md, 7.5)")]

use crate::startup::Unread;
use skein_io::{
    file::Unsafe,
    kernel::{Error, Kind},
};
use std::ptr;

pub(crate) fn private_error(errno: i32) -> Unread {
    match errno {
        libc::ELOOP => Unread::Unsafe(Unsafe::Link),
        libc::ENOENT => Unread::Absent,
        other => Unread::Kernel(kernel_error(other)),
    }
}

pub(crate) fn kernel_error(errno: i32) -> Error {
    match errno {
        libc::EACCES | libc::EPERM => Error::Permission,
        libc::ENOTDIR => Error::NotADirectory,
        libc::ENAMETOOLONG => Error::NameTooLong,
        libc::EMFILE | libc::ENFILE => Error::TooManyOpenFiles,
        libc::EINVAL => Error::InvalidArgument,
        libc::ENOBUFS | libc::ENOMEM => Error::NoBufferSpace,
        other => Error::Other(other),
    }
}

pub(crate) fn read_opened(fd: i32, max: u32) -> Result<Box<[u8]>, Unread> {
    // SAFETY: all-zero stat storage is valid; fstat initializes this live
    // exclusive local during the call and retains no pointer.
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: stat is exclusive initialized storage that lives across fstat.
    if unsafe { libc::fstat(fd, ptr::from_mut(&mut stat)) } < 0 {
        return Err(Unread::Kernel(kernel_error(crate::system::last_errno())));
    }
    if stat.st_mode & libc::S_IFMT != libc::S_IFREG {
        return Err(Unread::NotAFile);
    }
    if stat.st_size > i64::from(max) {
        return Err(Unread::TooLarge { max });
    }
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8 * 1_024];
    loop {
        // SAFETY: the kernel writes only within the live exclusive buffer;
        // the descriptor is ours until the outer function closes it.
        let read = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
        if read < 0 {
            let errno = crate::system::last_errno();
            if errno == libc::EINTR {
                continue;
            }
            return Err(Unread::Kernel(kernel_error(errno)));
        }
        if read == 0 {
            return Ok(bytes.into_boxed_slice());
        }
        let read = usize::try_from(read).expect("a positive read fits usize");
        let length = bytes.len().checked_add(read).ok_or(Unread::TooLarge { max })?;
        if u64::try_from(length).expect("a length fits u64") > u64::from(max) {
            return Err(Unread::TooLarge { max });
        }
        bytes.extend_from_slice(buffer.get(..read).expect("the kernel read within its buffer"));
    }
}

/// The kind of a file whose mode is `mode`.
pub(crate) fn kind_of(mode: libc::mode_t) -> Kind {
    match mode & libc::S_IFMT {
        libc::S_IFREG => Kind::File,
        libc::S_IFDIR => Kind::Directory,
        libc::S_IFLNK => Kind::Symlink,
        _ => Kind::Other,
    }
}
