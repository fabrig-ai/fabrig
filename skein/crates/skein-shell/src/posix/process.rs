//! Process helpers shared by Linux and Darwin (shell.md, 7.5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Signal sets are local initialized C storage, borrowed only for each
//! synchronous call. No borrowed memory is retained.

#![expect(unsafe_code, reason = "signal calls borrow local storage only until return (shell.md, 7.5)")]

use skein_io::kernel::{Error, Signal};
use std::{ffi::CString, ptr};

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
