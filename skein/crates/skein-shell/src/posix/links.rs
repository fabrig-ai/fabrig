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
use std::{ffi::CString, mem, ptr};

const LINKS_PASSED: u8 = 8;

/// Rewrites trusted links among the parents, leaving the first unwalkable
/// component and everything after it as given. `dir` is borrowed, or `AT_FDCWD`.
pub(crate) fn pass_links(dir: i32, path: &[u8], owner: u32) -> Option<Box<[u8]>> {
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
            let Some(entry) = stat_at(dir, name) else { break };
            if entry.st_mode & libc::S_IFMT == libc::S_IFLNK {
                if passed == LINKS_PASSED || entry.st_uid != owner {
                    break;
                }
                let prefix = path.get(..start).expect("holding directory lies within path");
                let Some(parent) = stat_at(dir, if prefix.is_empty() { b"." } else { prefix }) else { break };
                if parent.st_uid != owner || parent.st_mode & 0o022 != 0 {
                    break;
                }
                let Some(target) = read_link(dir, name) else { break };
                let absolute = target.starts_with(b"/");
                path = [if absolute { b"" } else { prefix }, &target, rest].concat().into_boxed_slice();
                if absolute {
                    start = 0;
                }
                passed = passed.checked_add(1).expect("at most eight links passed");
                continue;
            }
            if entry.st_mode & libc::S_IFMT != libc::S_IFDIR {
                break;
            }
        }
        start = end.checked_add(1).expect("a slash follows this component");
    }
    if passed == 0 { None } else { Some(path) }
}

fn stat_at(dir: i32, path: &[u8]) -> Option<libc::stat> {
    let path = CString::new(path).ok()?;
    // SAFETY: all-zero stat storage is valid and remains exclusive until return.
    let mut stat: libc::stat = unsafe { mem::zeroed() };
    retry(|| {
        // SAFETY: path is terminated and stat is live exclusive output storage.
        status(unsafe { libc::fstatat(dir, path.as_ptr(), ptr::from_mut(&mut stat), libc::AT_SYMLINK_NOFOLLOW) })
    })
    .ok()?;
    Some(stat)
}

fn read_link(dir: i32, path: &[u8]) -> Option<Box<[u8]>> {
    let path = CString::new(path).ok()?;
    let mut target =
        skein_lib::bytes::zeroed(skein_lib::usize_of(u32::try_from(libc::PATH_MAX).expect("positive path limit")));
    let length = retry(|| {
        // SAFETY: path is terminated; target is exclusive for the supplied length.
        let length = unsafe { libc::readlinkat(dir, path.as_ptr(), target.as_mut_ptr().cast(), target.len()) };
        if length < 0 {
            Err(last_errno())
        } else {
            Ok(skein_lib::usize_of(u32::try_from(length).expect("readlinkat fills at most PATH_MAX bytes")))
        }
    })
    .ok()?;
    // A full buffer might be truncated; leave that link for the open to refuse.
    if length == target.len() {
        return None;
    }
    Some(target.get(..length).expect("readlinkat stays within its buffer").into())
}

#[cfg(test)]
mod tests;
