//! Signals to process groups and process liveness (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Kill takes only a checked process or negative process group ID and a signal number;
//! it borrows no memory and retains no pointer. Tests observe an unreaped child
//! with waitid into local initialized storage, borrowed only until return.

#![expect(unsafe_code, reason = "kill takes checked integers only (runtime/readiness.md, 5)")]

use super::process::map_error;
use crate::system::last_errno;
use skein_io::kernel::{Error, Signal};

pub(crate) fn negative_group(group: u32) -> Result<libc::pid_t, Error> {
    let positive = libc::pid_t::try_from(group).map_err(|_| Error::InvalidArgument)?;
    // Negating 0 or 1 would select the caller's group or every permitted process.
    if positive <= 1 {
        return Err(Error::InvalidArgument);
    }
    positive.checked_neg().ok_or(Error::InvalidArgument)
}

pub(crate) fn signal_group(group: u32, signal: Signal) -> Result<(), Error> {
    let negative = negative_group(group)?;
    let number = super::process::signal_number(signal);
    // SAFETY: negative is a checked negative ID; kill takes only integers.
    let sent = unsafe { libc::kill(negative, number) };
    if sent == 0 {
        return Ok(());
    }
    let errno = last_errno();
    #[cfg(target_os = "macos")]
    {
        refused_group_signal(errno, || crate::darwin::group::group_has_running_member(group, number))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(map_error(errno))
    }
}

/// Only a complete reading with no running member can settle Darwin's EPERM.
/// Other errors retain the kernel's answer without reading the group's members.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn refused_group_signal(errno: i32, running: impl FnOnce() -> Result<bool, Error>) -> Result<(), Error> {
    if errno == libc::EPERM {
        match running() {
            Ok(false) => Ok(()),
            Ok(true) | Err(_) => Err(Error::Permission),
        }
    } else {
        Err(map_error(errno))
    }
}

/// The raw libproc listing, including errno captured immediately after the call.
#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy, Debug)]
pub(crate) struct GroupListing {
    pub bytes: i32,
    pub capacity: i32,
    pub errno: i32,
}

#[cfg(any(target_os = "macos", test))]
impl GroupListing {
    fn member_count(self) -> Result<usize, GroupRefusal> {
        if self.bytes < 0 || (self.bytes == 0 && self.errno != 0) {
            return Err(GroupRefusal::ListingFailed { bytes: self.bytes, errno: self.errno });
        }
        if self.bytes == 0 {
            return Ok(0);
        }
        let bytes = skein_lib::usize_of(self.bytes.unsigned_abs());
        if !bytes.is_multiple_of(size_of::<libc::pid_t>()) {
            return Err(GroupRefusal::ListingMalformed { bytes: self.bytes });
        }
        if self.bytes >= self.capacity {
            return Err(GroupRefusal::ListingTruncated { bytes: self.bytes, capacity: self.capacity });
        }
        Ok(bytes.checked_div(size_of::<libc::pid_t>()).expect("pid_t has nonzero size"))
    }
}

#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroupMember {
    Status { pid: i32, status: u32, flags: u32 },
    Failed { pid: i32, bytes: i32, errno: i32 },
}

#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroupRefusal {
    ListingFailed { bytes: i32, errno: i32 },
    ListingMalformed { bytes: i32 },
    ListingTruncated { bytes: i32, capacity: i32 },
    InvalidMember { pid: i32 },
    Member(GroupMember),
    MissingMember,
}

/// A complete census with only ending or vanished members settles a refusal.
/// The iterator is read only up to the listing's byte count, and stops at the
/// first member that keeps the refusal. Darwin supplies status reads lazily.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn group_refusal(
    listing: GroupListing,
    members: impl IntoIterator<Item = GroupMember>,
) -> Option<GroupRefusal> {
    let count = match listing.member_count() {
        Ok(count) => count,
        Err(reason) => return Some(reason),
    };
    let mut members = members.into_iter();
    for _ in 0..count {
        let Some(member) = members.next() else { return Some(GroupRefusal::MissingMember) };
        let pid = match member {
            GroupMember::Status { pid, .. } | GroupMember::Failed { pid, .. } => pid,
        };
        if pid <= 0 {
            return Some(GroupRefusal::InvalidMember { pid });
        }
        match member {
            GroupMember::Status { status, flags, .. } => {
                // XNU sys/proc.h's SZOMB is 5; sys/proc_info.h's
                // PROC_FLAG_INEXIT is 4. Both mean this member is ending.
                const PROC_FLAG_INEXIT: u32 = 4;
                if status != 5 && flags & PROC_FLAG_INEXIT == 0 {
                    return Some(GroupRefusal::Member(member));
                }
            }
            GroupMember::Failed { bytes, errno: libc::ESRCH, .. } if bytes <= 0 => {}
            GroupMember::Failed { .. } => return Some(GroupRefusal::Member(member)),
        }
    }
    None
}

/// The one stderr line for a refusal that a census could not settle.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn group_refusal_line(group: u32, signal: i32, reason: GroupRefusal) -> String {
    let reading = match reason {
        GroupRefusal::ListingFailed { bytes, errno } => format!("listing failed: bytes {bytes} errno {errno}"),
        GroupRefusal::ListingMalformed { bytes } => format!("malformed listing: bytes {bytes}"),
        GroupRefusal::ListingTruncated { bytes, capacity } => {
            format!("possibly truncated listing: bytes {bytes} capacity {capacity}")
        }
        GroupRefusal::InvalidMember { pid } => format!("invalid member pid {pid}"),
        GroupRefusal::Member(GroupMember::Status { pid, status, flags }) => {
            format!("member {pid} status {status} flags {flags:#x}")
        }
        GroupRefusal::Member(GroupMember::Failed { pid, bytes, errno }) => {
            format!("member {pid} status read failed: bytes {bytes} errno {errno}")
        }
        GroupRefusal::MissingMember => String::from("listing has no status for a member"),
    };
    format!("skein-shell: group {group} refused signal {signal}: {reading}")
}

pub(crate) fn group_live(group: u32) -> Result<bool, Error> {
    let group = negative_group(group)?;
    // SAFETY: group is a checked negative ID; signal zero probes membership.
    if unsafe { libc::kill(group, 0) } == 0 {
        return Ok(true);
    }
    match last_errno() {
        libc::ESRCH => Ok(false),
        libc::EPERM => Ok(true),
        other => Err(map_error(other)),
    }
}

pub(crate) fn process_live(pid: u32) -> Result<bool, Error> {
    let pid = libc::pid_t::try_from(pid).map_err(|_| Error::InvalidArgument)?;
    if pid <= 1 {
        return Err(Error::InvalidArgument);
    }
    // SAFETY: pid is a checked positive ID; signal zero probes existence.
    if unsafe { libc::kill(pid, 0) } == 0 {
        return Ok(true);
    }
    match last_errno() {
        libc::ESRCH => Ok(false),
        libc::EPERM => Ok(true),
        other => Err(map_error(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GroupListing, GroupMember, group_live, group_refusal, group_refusal_line, negative_group, process_live,
        refused_group_signal, signal_group,
    };
    use crate::system::group::group_has_other_members;
    use skein_io::kernel::{Error, Signal};

    #[test]
    fn a_group_refusal_is_read_only_when_permission_was_denied() {
        for (case, errno, running, expected, should_read) in [
            ("no running member", libc::EPERM, Ok(false), Ok(()), true),
            ("running member", libc::EPERM, Ok(true), Err(Error::Permission), true),
            ("failed reading", libc::EPERM, Err(Error::Other(libc::EIO)), Err(Error::Permission), true),
            ("missing group", libc::ESRCH, Ok(false), Err(Error::Other(libc::ESRCH)), false),
            ("invalid signal", libc::EINVAL, Ok(false), Err(Error::InvalidArgument), false),
        ] {
            let mut read = false;
            let result = refused_group_signal(errno, || {
                read = true;
                running
            });
            assert_eq!(result, expected, "{case}: refusal follows the members only for EPERM");
            assert_eq!(read, should_read, "{case}: other errors never read members");
        }
    }

    #[test]
    fn a_group_census_settles_only_ending_members_and_explains_a_kept_refusal() {
        use GroupMember::{Failed, Status};
        let zombie = Status { pid: 4250, status: 5, flags: 0 };
        let running = Status { pid: 4251, status: 2, flags: 0 };
        let listing = |bytes, capacity, errno| GroupListing { bytes, capacity, errno };
        for (case, listing, members, expected) in [
            ("empty", listing(0, 16, 0), &[][..], None),
            ("empty failed", listing(0, 16, libc::EPERM), &[][..], Some("listing failed: bytes 0 errno 1")),
            ("failed", listing(-1, 16, libc::EIO), &[][..], Some("listing failed: bytes -1 errno 5")),
            ("zombies", listing(8, 16, 0), &[zombie, zombie][..], None),
            ("exiting", listing(4, 16, 0), &[Status { pid: 4250, status: 2, flags: 4 }][..], None),
            ("exit flag among others", listing(4, 16, 0), &[Status { pid: 4250, status: 2, flags: 0x104 }][..], None),
            (
                "stopped",
                listing(4, 16, 0),
                &[Status { pid: 4250, status: 4, flags: 0 }][..],
                Some("member 4250 status 4 flags 0x0"),
            ),
            ("running", listing(4, 16, 0), &[running][..], Some("member 4251 status 2 flags 0x0")),
            (
                "non-exit flags",
                listing(4, 16, 0),
                &[Status { pid: 4250, status: 2, flags: 0x100 }][..],
                Some("member 4250 status 2 flags 0x100"),
            ),
            ("running after ending", listing(8, 16, 0), &[zombie, running][..], Some("member 4251 status 2 flags 0x0")),
            ("vanished", listing(4, 16, 0), &[Failed { pid: 4250, bytes: 0, errno: libc::ESRCH }][..], None),
            (
                "status failed",
                listing(4, 16, 0),
                &[Failed { pid: 4250, bytes: 0, errno: libc::EIO }][..],
                Some("member 4250 status read failed: bytes 0 errno 5"),
            ),
            (
                "short status",
                listing(4, 16, 0),
                &[Failed { pid: 4250, bytes: 1, errno: 0 }][..],
                Some("member 4250 status read failed: bytes 1 errno 0"),
            ),
            ("malformed", listing(3, 16, 0), &[][..], Some("malformed listing: bytes 3")),
            ("full", listing(8, 8, 0), &[][..], Some("possibly truncated listing: bytes 8 capacity 8")),
            ("oversized", listing(12, 8, 0), &[][..], Some("possibly truncated listing: bytes 12 capacity 8")),
            (
                "invalid pid",
                listing(4, 16, 0),
                &[Status { pid: 0, status: 5, flags: 0 }][..],
                Some("invalid member pid 0"),
            ),
            ("missing status", listing(4, 16, 0), &[][..], Some("listing has no status for a member")),
            ("unlisted storage", listing(4, 16, 0), &[zombie, running][..], None),
        ] {
            let reason = group_refusal(listing, members.iter().copied());
            let result = refused_group_signal(libc::EPERM, || Ok(reason.is_some()));
            assert_eq!(result, if expected.is_some() { Err(Error::Permission) } else { Ok(()) }, "{case}");
            let line = reason.map(|reason| group_refusal_line(4242, libc::SIGKILL, reason));
            let expected = expected.map(|reading| format!("skein-shell: group 4242 refused signal 9: {reading}"));
            assert_eq!(line, expected, "{case}: a kept refusal names what kept it");
        }
    }

    #[test]
    fn only_specific_positive_groups_can_be_addressed() {
        for group in [0, 1, u32::MAX] {
            assert_eq!(negative_group(group), Err(Error::InvalidArgument));
            assert_eq!(group_live(group), Err(Error::InvalidArgument));
            assert_eq!(group_has_other_members(group), Err(Error::InvalidArgument), "reserved groups are refused");
            assert_eq!(signal_group(group, Signal::Kill), Err(Error::InvalidArgument));
        }
        assert_eq!(negative_group(2), Ok(-2));
        assert_eq!(negative_group(i32::MAX.unsigned_abs()), Ok(-i32::MAX));
    }

    #[test]
    fn only_specific_positive_processes_can_be_probed() {
        for pid in [0, 1, u32::MAX] {
            assert_eq!(process_live(pid), Err(Error::InvalidArgument), "PID {pid} is refused");
        }
    }
}
