//! Startup file decisions over a path or the facts of an open file (runtime/shell-startup.md, 1.1).

use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use skein_io::{
    file::Unsafe,
    kernel::{Kind, Stat},
};

use super::{DirectoryAccess, Unread};
use crate::posix::files::kernel_error;

/// Checks whether the effective user can write and search the directory at `path`
/// without changing it (runtime/shell-startup.md, 1.1).
pub fn directory_access(path: &Path) -> DirectoryAccess {
    directory_access_with(path, crate::system::directory_access)
}

fn directory_access_with(path: &Path, check: impl FnOnce(&CStr) -> DirectoryAccess) -> DirectoryAccess {
    match CString::new(path.as_os_str().as_bytes()) {
        Ok(path) => check(&path),
        Err(_) => DirectoryAccess::Denied(libc::EINVAL),
    }
}

pub(crate) fn private_metadata(stat: Stat, user: u32) -> Result<(), Unread> {
    match Unsafe::private_file(stat, user) {
        Some(found) => Err(Unread::Unsafe(found)),
        None => Ok(()),
    }
}

pub(crate) fn read_bounded(
    kind: Kind,
    size: i64,
    max: u32,
    mut read: impl FnMut(&mut [u8]) -> Result<usize, i32>,
) -> Result<Box<[u8]>, Unread> {
    if kind != Kind::File {
        return Err(Unread::NotAFile);
    }
    if size > i64::from(max) {
        return Err(Unread::TooLarge { max });
    }
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8 * 1_024];
    loop {
        let count = match read(&mut buffer) {
            Ok(0) => return Ok(bytes.into_boxed_slice()),
            Ok(count) => count,
            Err(libc::EINTR) => continue,
            Err(errno) => return Err(Unread::Kernel(kernel_error(errno))),
        };
        let length = bytes.len().checked_add(count).ok_or(Unread::TooLarge { max })?;
        if skein_lib::len_u64(length) > u64::from(max) {
            return Err(Unread::TooLarge { max });
        }
        bytes.extend_from_slice(buffer.get(..count).expect("the read filled bytes within its buffer"));
    }
}

#[cfg(test)]
mod tests;
