//! Darwin's process observations and tree keeper calls (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Libproc's null-buffer call borrows no memory. Its listing fills an initialized
//! PID array bounded by its supplied byte size. `proc_pidinfo` fills local initialized
//! `proc_bsdinfo` storage of exactly the supplied size. Neither retains a pointer.
//! __error returns the calling thread's writable errno slot, used only locally.

#![expect(
    unsafe_code,
    reason = "libproc borrows bounded PID arrays and status storage; errno is thread-local (runtime/readiness.md, 5)"
)]

use crate::posix::group::{GroupListing, GroupMember, GroupRefusal, group_refusal, group_refusal_line, negative_group};
pub(crate) use crate::posix::group::{group_live, process_live, signal_group};
use skein_io::kernel::{Error, Fd};

use super::{last_errno, process::map_error};

pub(crate) fn unreaped_children() -> Result<Box<[u32]>, i32> {
    // libproc.h's PROC_PPID_ONLY is 6. The estimate and listing count bytes,
    // not entries; the walk includes zombies after the living processes.
    let parent = super::process_id();
    // SAFETY: a null buffer with size zero asks only for the byte estimate.
    let estimate = unsafe { libc::proc_listpids(6, parent, std::ptr::null_mut(), 0) };
    if estimate <= 0 {
        let errno = last_errno();
        return Err(if errno == 0 { libc::ENOBUFS } else { errno });
    }
    let bytes = usize::try_from(estimate).map_err(|_| libc::ENOBUFS)?;
    let pid_size = size_of::<libc::pid_t>();
    if !bytes.is_multiple_of(pid_size) {
        return Err(libc::ENOBUFS);
    }
    let mut children = vec![0_i32; bytes.checked_div(pid_size).ok_or(libc::ENOBUFS)?];
    // SAFETY: __error returns this thread's writable errno slot.
    let errno_slot = unsafe { libc::__error() };
    // SAFETY: the returned thread-local errno slot is writable.
    unsafe { errno_slot.write(0) };
    // SAFETY: children is initialized and writable for exactly estimate bytes.
    let listed = unsafe { libc::proc_listpids(6, parent, children.as_mut_ptr().cast(), estimate) };
    if listed == 0 {
        let errno = last_errno();
        return if errno == 0 { Ok(Box::new([])) } else { Err(errno) };
    }
    if listed < 0 {
        return Err(last_errno());
    }
    if listed >= estimate {
        return Err(libc::ENOBUFS);
    }
    let listed = usize::try_from(listed).map_err(|_| libc::ENOBUFS)?;
    if !listed.is_multiple_of(pid_size) {
        return Err(libc::ENOBUFS);
    }
    let mut pids = Vec::new();
    for pid in children.into_iter().take(listed.checked_div(pid_size).ok_or(libc::ENOBUFS)?) {
        if pid <= 0 {
            return Err(libc::ENOBUFS);
        }
        pids.push(u32::try_from(pid).map_err(|_| libc::ENOBUFS)?);
    }
    Ok(pids.into_boxed_slice())
}

pub(crate) fn child_id(handle: Fd) -> Result<u32, Error> {
    super::process::child_id(handle)
}

/// Conservatively reads the members of a group whose signal was refused.
/// A failed or possibly truncated reading cannot establish that only zombies remain.
pub(crate) fn group_has_running_member(group: u32, signal: i32) -> Result<bool, Error> {
    negative_group(group)?;
    match read_group(group) {
        None => Ok(false),
        Some(reason) => {
            eprintln!("{}", group_refusal_line(group, signal, reason));
            Ok(true)
        }
    }
}

fn read_group(group: u32) -> Option<GroupRefusal> {
    // libproc.h's PROC_PGRP_ONLY is 2. Both the null-buffer estimate and the
    // populated listing count bytes, not entries. The estimate can cover all
    // processes, not just this group; a full listing is still possibly truncated.
    clear_errno();
    // SAFETY: a null buffer with size zero asks only for the byte estimate.
    let estimate = unsafe { libc::proc_listpids(2, group, std::ptr::null_mut(), 0) };
    let errno = last_errno();
    if estimate <= 0 {
        return group_refusal(GroupListing { bytes: estimate, capacity: 0, errno }, []);
    }
    let bytes = skein_lib::usize_of(estimate.unsigned_abs());
    let pid_size = size_of::<libc::pid_t>();
    if !bytes.is_multiple_of(pid_size) {
        return group_refusal(GroupListing { bytes: estimate, capacity: estimate, errno }, []);
    }
    let mut members = vec![0_i32; bytes.checked_div(pid_size).expect("pid_t has nonzero size")];
    clear_errno();
    // SAFETY: members is initialized and writable for exactly estimate bytes.
    let listed = unsafe { libc::proc_listpids(2, group, members.as_mut_ptr().cast(), estimate) };
    let errno = last_errno();
    let listing = GroupListing { bytes: listed, capacity: estimate, errno };
    group_refusal(
        listing,
        members.into_iter().map(|pid| {
            if pid <= 0 {
                return GroupMember::Failed { pid, bytes: 0, errno: libc::EINVAL };
            }
            // SAFETY: proc_bsdinfo is C output storage; zero initializes every field.
            let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
            let size = libc::c_int::try_from(size_of::<libc::proc_bsdinfo>()).expect("proc_bsdinfo size fits c_int");
            clear_errno();
            // SAFETY: info is writable for exactly size bytes; libproc retains no pointer.
            // PROC_PIDTBSDINFO with arg 1 also asks for a zombie's status.
            let read = unsafe {
                libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 1, std::ptr::from_mut(&mut info).cast(), size)
            };
            let errno = last_errno();
            if read == size {
                GroupMember::Status { pid, status: info.pbi_status, flags: info.pbi_flags }
            } else {
                GroupMember::Failed { pid, bytes: read, errno }
            }
        }),
    )
}

fn clear_errno() {
    // SAFETY: __error returns this thread's writable errno slot; no pointer is retained.
    unsafe { *libc::__error() = 0 };
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
