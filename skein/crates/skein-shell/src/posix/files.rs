//! Startup file reads and error meanings shared by Linux and Darwin (runtime/shell-startup.md, 1.1, and runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Fstat writes initialized local stat storage; read writes only within the
//! exclusive buffer's supplied length. Directory calls borrow terminated
//! names and keep their parent descriptors open. No call retains a pointer;
//! the caller closes descriptors or hands them up as roots.

#![expect(unsafe_code, reason = "file calls borrow local storage only until return (runtime/readiness.md, 5)")]

use crate::startup::{Unread, private_metadata, read_bounded};
use crate::system::last_errno;
use skein_io::{
    file::Unsafe,
    kernel::{Error, Fd, Kind, Stat},
};
use std::{ffi::CStr, ptr};

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

fn stat_opened(fd: i32) -> Result<libc::stat, Unread> {
    // SAFETY: all-zero stat storage is valid; fstat initializes this live
    // exclusive local during the call and retains no pointer.
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: stat is exclusive initialized storage that lives across fstat.
    super::status(unsafe { libc::fstat(fd, ptr::from_mut(&mut stat)) })
        .map_err(|errno| Unread::Kernel(kernel_error(errno)))?;
    Ok(stat)
}

pub(crate) fn check_private(fd: i32, user: u32) -> Result<(), Unread> {
    let stat = stat_opened(fd)?;
    #[cfg(target_os = "linux")]
    let (mode, links) = (stat.st_mode, u32::try_from(stat.st_nlink).unwrap_or(u32::MAX));
    #[cfg(target_os = "macos")]
    let (mode, links) = (u32::from(stat.st_mode), u32::from(stat.st_nlink));
    let metadata = Stat {
        kind: kind_of(stat.st_mode),
        size: u64::try_from(stat.st_size).unwrap_or(0),
        mode: mode & skein_io::kernel::PERMISSIONS,
        owner: stat.st_uid,
        links,
    };
    private_metadata(metadata, user)
}

pub(crate) fn read_opened(fd: i32, max: u32) -> Result<Box<[u8]>, Unread> {
    let stat = stat_opened(fd)?;
    read_bounded(kind_of(stat.st_mode), stat.st_size, max, |buffer| {
        // SAFETY: the kernel writes only within this live exclusive buffer;
        // the descriptor is ours until the outer function closes it.
        let read = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
        usize::try_from(read).map_err(|_| last_errno())
    })
}

pub(crate) fn make_child(parent: Fd, name: &CStr, mode: u32) -> Result<(), i32> {
    #[cfg(target_os = "macos")]
    let mode = libc::mode_t::try_from(mode).expect("permission bits fit mode_t");
    // SAFETY: the parent is held open, and the leaf name remains live and terminated.
    super::status(unsafe { libc::mkdirat(parent.raw(), name.as_ptr(), mode) })
}

pub(crate) fn open_child(parent: Fd, name: &CStr) -> Result<Fd, i32> {
    // SAFETY: the parent is held open, and the leaf name remains live and terminated.
    let fd = unsafe { libc::openat(parent.raw(), name.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) };
    if fd < 0 { Err(last_errno()) } else { Ok(Fd::new(fd)) }
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
