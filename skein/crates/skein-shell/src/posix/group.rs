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
        refused_group_signal(errno, || crate::darwin::group::group_has_running_member(group))
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
    use super::{group_live, negative_group, process_live, refused_group_signal, signal_group};
    use crate::system::group::{group_has_other_members, signal_group as system_signal_group};
    use skein_io::kernel::{Error, Signal};
    use std::os::unix::process::CommandExt;

    #[test]
    fn a_group_refusal_is_read_only_when_permission_was_denied() {
        for (case, errno, running, expected, should_read) in [
            ("only zombies", libc::EPERM, Ok(false), Ok(()), true),
            ("empty group", libc::EPERM, Ok(false), Ok(()), true),
            ("running member", libc::EPERM, Ok(true), Err(Error::Permission), true),
            ("failed reading", libc::EPERM, Err(Error::Other(libc::EIO)), Err(Error::Permission), true),
            ("possibly truncated reading", libc::EPERM, Err(Error::NoBufferSpace), Err(Error::Permission), true),
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
    fn a_group_of_one_zombie_accepts_a_kill() {
        #[expect(
            clippy::disallowed_types,
            reason = "the group signal test retains a std child as a zombie independently of shell handles"
        )]
        let mut child = std::process::Command::new("/bin/sh")
            .args(["-c", "exit 0"])
            .process_group(0)
            .spawn()
            .expect("child starts in its own group");
        let pid = child.id();
        // SAFETY: siginfo_t is C output storage; zero is a valid initial representation.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        loop {
            // SAFETY: info is writable for waitid; WNOWAIT retains this owned child.
            let rc =
                unsafe { libc::waitid(libc::P_PID, pid, std::ptr::from_mut(&mut info), libc::WEXITED | libc::WNOWAIT) };
            if rc == 0 {
                break;
            }
            assert_eq!(crate::system::last_errno(), libc::EINTR, "waitid observes the child's exit without reaping");
        }
        let result = system_signal_group(pid, Signal::Kill);
        child.wait().expect("the zombie is reaped after its group signal");
        assert_eq!(result, Ok(()), "a group containing only its zombie leader accepts a kill");
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
    fn the_running_process_is_live() {
        assert_eq!(process_live(std::process::id()), Ok(true), "the caller exists");
    }

    #[test]
    fn a_reaped_child_is_not_live() {
        #[expect(
            clippy::disallowed_types,
            reason = "the liveness test reaps a std child independently of the shell's process handles"
        )]
        let mut child = std::process::Command::new("/bin/sh").args(["-c", "exit 0"]).spawn().expect("child starts");
        let pid = child.id();
        child.wait().expect("child is reaped");
        assert_eq!(process_live(pid), Ok(false), "a reaped child no longer exists");
    }

    #[test]
    fn only_specific_positive_processes_can_be_probed() {
        for pid in [0, 1, u32::MAX] {
            assert_eq!(process_live(pid), Err(Error::InvalidArgument), "PID {pid} is refused");
        }
    }
}
