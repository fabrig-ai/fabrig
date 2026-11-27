//! Passing only the system's own parent links (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Fstatat and readlinkat borrow local NUL-terminated paths until return,
//! retaining no pointers or descriptor ownership. Fstatat writes exclusive,
//! initialized stat storage; readlinkat writes only the supplied buffer length.
//! The final no-follow open, not this walk, enforces refusal of remaining links.

use super::{retry, status};
use crate::system::last_errno;
use skein_io::kernel::{LONGEST_NAME, is_name};
use std::{
    ffi::{CStr, CString},
    mem, ptr,
};

const LINKS_PASSED: u8 = 8;

/// Rewrites trusted links among the parents, leaving the first unwalkable
/// component and everything after it as given. `dir` is borrowed, or `AT_FDCWD`.
pub(crate) fn pass_links(dir: i32, path: &[u8], owner: u32) -> Option<Box<[u8]>> {
    walk(path, owner, |name| stat_at(dir, name), |name| read_link(dir, name))
}

/// Resolves at most one link, from one leaf to a regular file beside it.
/// The caller opens the resulting name without following links, including
/// if an entry changes after this inspection.
pub(crate) fn sibling_file(dir: i32, name: &[u8]) -> Result<Option<Box<[u8]>>, i32> {
    let name = CString::new(name).map_err(|_| libc::EINVAL)?;
    let mut target = [0_u8; LONGEST_NAME + 1];
    let length = match read_link_at(dir, &name, &mut target) {
        Ok(length) => skein_lib::usize_of(length),
        Err(libc::EINVAL) => return Ok(None), // The leaf is not a link.
        Err(errno) => return Err(errno),
    };
    let target = target.get(..length).expect("readlinkat stays within its buffer");
    if length > LONGEST_NAME || !is_name(target) {
        return Err(libc::ELOOP);
    }
    if stat_at(dir, target).is_none_or(|entry| entry.mode & libc::S_IFMT != libc::S_IFREG) {
        return Err(libc::ELOOP);
    }
    Ok(Some(target.into()))
}

struct Entry {
    mode: libc::mode_t,
    owner: libc::uid_t,
}

fn walk(
    path: &[u8],
    owner: u32,
    mut lookup: impl FnMut(&[u8]) -> Option<Entry>,
    mut read: impl FnMut(&[u8]) -> Option<Box<[u8]>>,
) -> Option<Box<[u8]>> {
    let mut path: Box<[u8]> = path.into();
    let mut start = 0;
    let mut passed = 0_u8;
    loop {
        let remaining = path.get(start..).expect("walk stays within path");
        let length = remaining.iter().position(|byte| *byte == b'/').unwrap_or(remaining.len());
        let end = start.checked_add(length).expect("component ends within path");
        let rest = path.get(end..).expect("component ends within path");
        if rest.iter().all(|byte| *byte == b'/') {
            break; // The last non-empty component is never passed.
        }
        let component = path.get(start..end).expect("component lies within path");
        if component == b".." {
            break;
        }
        if !component.is_empty() && component != b"." {
            let name = path.get(..end).expect("prefix lies within path");
            let Some(entry) = lookup(name) else { break };
            if entry.mode & libc::S_IFMT == libc::S_IFLNK {
                if passed == LINKS_PASSED || entry.owner != owner {
                    break;
                }
                let prefix = path.get(..start).expect("holding directory lies within path");
                let Some(parent) = lookup(if prefix.is_empty() { b"." } else { prefix }) else { break };
                if parent.owner != owner || parent.mode & 0o022 != 0 {
                    break;
                }
                let Some(target) = read(name) else { break };
                let absolute = target.starts_with(b"/");
                path = [if absolute { b"" } else { prefix }, &target, rest].concat().into_boxed_slice();
                if absolute {
                    start = 0;
                }
                passed = passed.checked_add(1).expect("at most eight links passed");
                continue;
            }
            if entry.mode & libc::S_IFMT != libc::S_IFDIR {
                break;
            }
        }
        start = end.checked_add(1).expect("a slash follows this component");
    }
    if passed == 0 { None } else { Some(path) }
}

fn stat_at(dir: i32, path: &[u8]) -> Option<Entry> {
    let path = CString::new(path).ok()?;
    // SAFETY: all-zero stat storage is valid and remains exclusive until return.
    let mut stat: libc::stat = unsafe { mem::zeroed() };
    retry(|| {
        // SAFETY: path is terminated and stat is live exclusive output storage.
        status(unsafe { libc::fstatat(dir, path.as_ptr(), ptr::from_mut(&mut stat), libc::AT_SYMLINK_NOFOLLOW) })
    })
    .ok()?;
    Some(Entry { mode: stat.st_mode, owner: stat.st_uid })
}

fn read_link(dir: i32, path: &[u8]) -> Option<Box<[u8]>> {
    let mut target =
        skein_lib::bytes::zeroed(skein_lib::usize_of(u32::try_from(libc::PATH_MAX).expect("positive path limit")));
    let length = skein_lib::usize_of(read_target(dir, path, &mut target).ok()?);
    Some(target.get(..length).expect("readlinkat stays within its buffer").into())
}

/// Reads one link without following the last component; refuses a possibly cut target.
/// A kernel `ReadLink` passes a bare name, so it follows no parents either.
pub(crate) fn read_target(dir: i32, name: &[u8], buf: &mut [u8]) -> Result<u32, i32> {
    let name = CString::new(name).map_err(|_| libc::EINVAL)?;
    let count = read_link_at(dir, &name, buf)?;
    if count == 0 || skein_lib::usize_of(count) == buf.len() { Err(libc::EOVERFLOW) } else { Ok(count) }
}

fn read_link_at(dir: i32, name: &CStr, buf: &mut [u8]) -> Result<u32, i32> {
    retry(|| {
        // SAFETY: name is terminated; buf is exclusive for the supplied length.
        let length = unsafe { libc::readlinkat(dir, name.as_ptr(), buf.as_mut_ptr().cast(), buf.len()) };
        if length < 0 {
            return Err(last_errno());
        }
        Ok(u32::try_from(length).expect("readlinkat fills at most the supplied room"))
    })
}

#[cfg(test)]
mod tests;
