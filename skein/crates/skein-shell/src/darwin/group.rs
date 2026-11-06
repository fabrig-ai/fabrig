//! Darwin's tree keeper calls (shell.md, 7.5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Libproc fills a local PID array bounded by its supplied size and retains no pointer.

#![expect(unsafe_code, reason = "kill takes checked integers; libproc borrows a bounded PID array (shell.md, 7.5)")]

use crate::posix::group::negative_group;
pub(crate) use crate::posix::group::{group_live, process_live, signal_group};
use skein_io::kernel::{Error, Fd};

use super::{last_errno, process::map_error};

#[link(name = "proc")]
unsafe extern "C" {
    fn proc_listpids(kind: u32, info: u32, buffer: *mut libc::c_void, size: libc::c_int) -> libc::c_int;
}

pub(crate) fn child_id(handle: Fd) -> Result<u32, Error> {
    super::process::child_id(handle)
}

pub(crate) fn group_has_other_members(group: u32) -> Result<bool, Error> {
    negative_group(group)?;
    // libproc's PROC_PGRP_ONLY is 2. Two slots suffice to distinguish the
    // retained leader alone from a group with any other member.
    let mut members = [0_i32; 2];
    // SAFETY: members is writable for the supplied byte count; no pointer is retained.
    let count = unsafe { proc_listpids(2, group, members.as_mut_ptr().cast(), 8) };
    if count < 0 {
        return Err(map_error(last_errno()));
    }
    Ok(members.into_iter().filter_map(|pid| u32::try_from(pid).ok()).any(|pid| pid > 1 && pid != group))
}
