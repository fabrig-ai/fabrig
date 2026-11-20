//! Process helpers shared by Linux and Darwin (runtime/readiness.md, 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Signal sets are local initialized C storage, borrowed only for each
//! synchronous call. No borrowed memory is retained.

#![expect(unsafe_code, reason = "signal calls borrow local storage only until return (runtime/readiness.md, 5)")]

use crate::system::process::resources;
use skein_io::kernel::{Error, Signal, Usage};
use std::{ffi::CString, ptr};

/// Negative diagnostics cannot collide with Darwin errno numbers.
#[cfg(any(target_os = "macos", test))]
pub(crate) const WAIT_UNEXPECTED_STATUS: i32 = -10_003;

/// Only an exit consumes the child's identity. Stops and continues leave Wait
/// pending even when the call asked to reap; other codes keep the diagnostic.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn decode_child_wait(
    reap: bool,
    code: i32,
    status: i32,
) -> (Result<Option<skein_io::kernel::Exit>, Error>, bool) {
    use skein_io::kernel::Exit;
    let exit = match code {
        libc::CLD_EXITED => Exit::Code(u8::try_from(status).expect("a child exit code fits in a byte")),
        libc::CLD_KILLED | libc::CLD_DUMPED => Exit::Signal(u32::try_from(status).expect("a signal is positive")),
        libc::CLD_STOPPED | libc::CLD_CONTINUED => return (Ok(None), false),
        _ => return (Err(Error::Other(WAIT_UNEXPECTED_STATUS)), false),
    };
    (Ok(Some(exit)), reap)
}

pub(crate) fn cstring(bytes: &[u8]) -> Result<CString, i32> {
    CString::new(bytes).map_err(|_| libc::EINVAL)
}

pub(crate) fn action(rc: i32) -> Result<(), i32> {
    if rc == 0 { Ok(()) } else { Err(rc) }
}

/// A pipe write after the child closes its read end returns EPIPE instead of
/// terminating the service with SIGPIPE. The kernel is tied to this thread.
pub(crate) fn block_sigpipe() -> Result<(), i32> {
    // SAFETY: sigset_t is a C signal set that sigemptyset initializes.
    let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: `set` is a valid writable signal set for both calls.
    action(unsafe { libc::sigemptyset(ptr::from_mut(&mut set)) })?;
    // SAFETY: `set` remains live and writable.
    action(unsafe { libc::sigaddset(ptr::from_mut(&mut set), libc::SIGPIPE) })?;
    // SAFETY: only this kernel thread's mask is changed; a child gets a
    // cleared mask through its system's spawn attributes.
    action(unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, ptr::from_ref(&set), ptr::null_mut()) })
}

pub(crate) fn map_error(errno: i32) -> Error {
    match errno {
        libc::ENOENT => Error::NotFound,
        libc::EACCES | libc::EPERM => Error::Permission,
        libc::ENOTDIR => Error::NotADirectory,
        libc::ELOOP => Error::TooManyLinks,
        libc::ENAMETOOLONG => Error::NameTooLong,
        libc::EXDEV => Error::Escape,
        libc::EMFILE | libc::ENFILE => Error::TooManyOpenFiles,
        libc::ENOMEM | libc::ENOBUFS => Error::NoBufferSpace,
        libc::EINVAL => Error::InvalidArgument,
        other => Error::Other(other),
    }
}

/// Reads the process and its reaped children at one submission (runtime/kernel.md, section 6.3).
pub(crate) fn usage() -> Result<Usage, Error> {
    Ok(Usage { own: resources(libc::RUSAGE_SELF)?, children: resources(libc::RUSAGE_CHILDREN)? })
}

pub(crate) fn cpu(value: libc::timeval) -> Result<skein_lib::Duration, Error> {
    let seconds = u64::try_from(value.tv_sec).map_err(|_| Error::Other(libc::EOVERFLOW))?;
    let micros = u64::try_from(value.tv_usec).map_err(|_| Error::Other(libc::EOVERFLOW))?;
    let nanos = seconds
        .checked_mul(1_000_000_000)
        .and_then(|base| micros.checked_mul(1000).and_then(|tail| base.checked_add(tail)))
        .ok_or(Error::Other(libc::EOVERFLOW))?;
    Ok(skein_lib::Duration::from_nanos(nanos))
}

pub(crate) fn signal_number(signal: Signal) -> i32 {
    match signal {
        Signal::Interrupt => libc::SIGINT,
        Signal::Terminate => libc::SIGTERM,
        Signal::Kill => libc::SIGKILL,
        Signal::Stop => libc::SIGSTOP,
        Signal::Continue => libc::SIGCONT,
    }
}

#[cfg(test)]
mod tests {
    use super::decode_child_wait;
    use skein_io::kernel::{Error, Exit};

    #[test]
    fn a_child_wait_marks_reaped_only_for_an_exit() {
        for reap in [false, true] {
            for (case, code, status, expected, is_exit) in [
                ("normal exit", libc::CLD_EXITED, 42, Ok(Some(Exit::Code(42))), true),
                ("killed", libc::CLD_KILLED, libc::SIGKILL, Ok(Some(Exit::Signal(9))), true),
                ("dumped", libc::CLD_DUMPED, libc::SIGQUIT, Ok(Some(Exit::Signal(3))), true),
                ("stopped", libc::CLD_STOPPED, libc::SIGSTOP, Ok(None), false),
                ("continued", libc::CLD_CONTINUED, libc::SIGCONT, Ok(None), false),
                ("trapped", libc::CLD_TRAPPED, libc::SIGTRAP, Err(Error::Other(-10_003)), false),
                ("unknown code", 0, 0, Err(Error::Other(-10_003)), false),
            ] {
                let (result, mark_reaped) = decode_child_wait(reap, code, status);
                assert_eq!(result, expected, "{case}, reap={reap}: only an exit completes Wait");
                assert_eq!(mark_reaped, reap && is_exit, "{case}, reap={reap}: only a reaping exit marks the handle");
            }
        }
    }
}
