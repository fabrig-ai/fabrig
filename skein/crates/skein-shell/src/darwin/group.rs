//! Darwin's tree keeper calls (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Libproc's null-buffer call borrows no memory. Its listing fills an initialized
//! PID array bounded by its supplied byte size. `proc_pidinfo` fills local initialized
//! `proc_bsdinfo` storage of exactly the supplied size. Neither retains a pointer.

#![expect(
    unsafe_code,
    reason = "libproc borrows bounded PID arrays and local status storage (runtime/readiness.md, 5)"
)]

use crate::posix::group::negative_group;
pub(crate) use crate::posix::group::{group_live, process_live, signal_group};
use skein_io::kernel::{Error, Fd};

use super::{last_errno, process::map_error};

pub(crate) fn child_id(handle: Fd) -> Result<u32, Error> {
    super::process::child_id(handle)
}

/// Conservatively reads the members of a group whose signal was refused.
/// A failed or possibly truncated reading cannot establish that only zombies remain.
pub(crate) fn group_has_running_member(group: u32) -> Result<bool, Error> {
    negative_group(group)?;
    // libproc.h's PROC_PGRP_ONLY is 2. Both the null-buffer estimate and the
    // populated listing count bytes, not entries. The estimate can cover all
    // processes, not just this group; a full listing is still possibly truncated.
    // SAFETY: a null buffer with size zero asks only for the byte estimate.
    let estimate = unsafe { libc::proc_listpids(2, group, std::ptr::null_mut(), 0) };
    if estimate <= 0 {
        return Ok(true);
    }
    let bytes = usize::try_from(estimate).map_err(|_| Error::NoBufferSpace)?;
    let pid_size = size_of::<libc::pid_t>();
    if !bytes.is_multiple_of(pid_size) {
        return Ok(true);
    }
    let mut members = vec![0_i32; bytes.checked_div(pid_size).ok_or(Error::NoBufferSpace)?];
    // SAFETY: members is initialized and writable for exactly estimate bytes.
    let listed = unsafe { libc::proc_listpids(2, group, members.as_mut_ptr().cast(), estimate) };
    // Zero can mean failure, with errno set; never infer an empty group from it.
    if listed <= 0 || listed >= estimate {
        return Ok(true);
    }
    let listed = usize::try_from(listed).map_err(|_| Error::NoBufferSpace)?;
    if !listed.is_multiple_of(pid_size) {
        return Ok(true);
    }
    for pid in members.into_iter().take(listed.checked_div(pid_size).ok_or(Error::NoBufferSpace)?) {
        if pid <= 0 {
            return Ok(true);
        }
        // SAFETY: proc_bsdinfo is C output storage; zero initializes every field.
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        let size = libc::c_int::try_from(size_of::<libc::proc_bsdinfo>()).map_err(|_| Error::NoBufferSpace)?;
        // SAFETY: info is writable for exactly size bytes; libproc retains no pointer.
        // PROC_PIDTBSDINFO with arg 1 also asks for a zombie's status.
        let read =
            unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 1, std::ptr::from_mut(&mut info).cast(), size) };
        if read != size {
            if read == 0 && last_errno() == libc::ESRCH {
                continue;
            }
            return Ok(true);
        }
        // XNU's sys/proc.h: SZOMB is 5. Every other status counts as running.
        if info.pbi_status != 5 {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn group_has_other_members(group: u32) -> Result<bool, Error> {
    negative_group(group)?;
    // libproc's PROC_PGRP_ONLY is 2. Two slots suffice to distinguish the
    // retained leader alone from a group with any other member.
    let mut members = [0_i32; 2];
    // SAFETY: members is writable for the supplied byte count; no pointer is retained.
    let count = unsafe { libc::proc_listpids(2, group, members.as_mut_ptr().cast(), 8) };
    if count < 0 {
        return Err(map_error(last_errno()));
    }
    Ok(members.into_iter().filter_map(|pid| u32::try_from(pid).ok()).any(|pid| pid > 1 && pid != group))
}
