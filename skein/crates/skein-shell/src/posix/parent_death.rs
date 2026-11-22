//! The parent-death watcher (runtime/readiness.md, 5; runtime/shell.md, 3).
//!
//! # The `unsafe`, and why it is sound
//!
//! Before fork the caller prepares both pids, the signal number, the close
//! bound, the handshake pipe and the signal set. The watcher uses only local
//! C storage and integer descriptors; no pointer survives a synchronous call.
//! It allocates nothing, takes no locks, runs no destructors and ends with
//! `_exit`. Its calls are async-signal-safe, plus each system's exit wait.
//! Closing inherited descriptors changes only the watcher's descriptor table.
//! It signals only while getppid still names the program that forked it.

use crate::system::{
    last_errno,
    process::{open_exit_watch, wait_exit_watch},
};
use std::ptr;

pub(crate) fn close_bound() -> Result<i32, i32> {
    // SAFETY: rlimit is plain C output storage initialized by getrlimit.
    let mut limit: libc::rlimit = unsafe { std::mem::zeroed() };
    // SAFETY: limit is exclusive writable output storage until return.
    super::status(unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, ptr::from_mut(&mut limit)) })?;
    i32::try_from(limit.rlim_cur.min(65_536)).map_err(|_| libc::EINVAL)
}

/// Called by the program to be, including in a pre-exec hook. Only integers
/// are captured by that hook; the close bound is obtained before spawn.
pub(crate) fn watch_parent(parent: libc::pid_t, signal: i32, until: i32) -> Result<(), i32> {
    if parent <= 1 {
        return Err(libc::ESRCH);
    }
    // SAFETY: getpid takes no arguments and borrows no memory.
    let program = unsafe { libc::getpid() };
    // SAFETY: sigset_t is local C storage initialized by sigfillset.
    let mut blocked: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: blocked is exclusive writable storage for a signal set.
    super::status(unsafe { libc::sigfillset(ptr::from_mut(&mut blocked)) })?;
    let mut pair = [0; 2];
    // SAFETY: pair holds writable storage for two new descriptors.
    super::status(unsafe { libc::pipe(pair.as_mut_ptr()) })?;
    let [reader, writer] = pair;
    let prepared = (|| {
        for descriptor in pair {
            // SAFETY: these are newly owned pipe ends; fcntl takes integers.
            super::status(unsafe { libc::fcntl(descriptor, libc::F_SETFD, libc::FD_CLOEXEC) })?;
        }
        Ok(())
    })();
    if let Err(errno) = prepared {
        close(reader);
        close(writer);
        return Err(errno);
    }
    // SAFETY: the child uses only the prepared integers and signal set,
    // async-signal-safe calls and the exit wait, and never returns.
    let watcher = unsafe { libc::fork() };
    if watcher == 0 {
        run(parent, program, signal, until, writer, &blocked);
    }
    let fork_errno = last_errno();
    close(writer);
    if watcher < 0 {
        close(reader);
        return Err(fork_errno);
    }
    let result = handshake(reader);
    close(reader);
    if result.is_err() {
        // SAFETY: watcher is our newly forked, unreaped child. End it even
        // if a read error prevented us from receiving its refusal.
        let _killed = unsafe { libc::kill(watcher, libc::SIGKILL) };
        loop {
            // SAFETY: waitpid targets only our own watcher; no status requested.
            if unsafe { libc::waitpid(watcher, ptr::null_mut(), 0) } >= 0 || last_errno() != libc::EINTR {
                break;
            }
        }
    }
    result
}

fn close(descriptor: i32) {
    // SAFETY: this path relinquishes its own descriptor once; never retry close.
    let _closed = unsafe { libc::close(descriptor) };
}

fn handshake(reader: i32) -> Result<(), i32> {
    let mut bytes = [0; 4];
    let mut used = 0;
    loop {
        let room = bytes.get_mut(used..).ok_or(libc::EIO)?;
        // SAFETY: room is exclusive writable storage with the supplied length.
        let count = unsafe { libc::read(reader, room.as_mut_ptr().cast(), room.len()) };
        if count == 0 {
            return Err(libc::EIO);
        }
        if count < 0 {
            let errno = last_errno();
            if errno != libc::EINTR {
                return Err(errno);
            }
        } else {
            used = used.checked_add(usize::try_from(count).map_err(|_| libc::EIO)?).ok_or(libc::EIO)?;
            if used == bytes.len() {
                let errno = i32::from_ne_bytes(bytes);
                return if errno == 0 { Ok(()) } else { Err(errno) };
            }
        }
    }
}

fn run(parent: i32, program: i32, signal: i32, until: i32, writer: i32, blocked: &libc::sigset_t) -> ! {
    // SAFETY: blocked is initialized, borrowed only until return; only this
    // forked process's mask changes. SIGKILL and SIGSTOP remain unblockable.
    let mask = unsafe { libc::sigprocmask(libc::SIG_SETMASK, ptr::from_ref(blocked), ptr::null_mut()) };
    let mask_errno = last_errno();
    for descriptor in 0..until {
        if descriptor != writer {
            close(descriptor);
        }
    }
    let watch = if mask < 0 { Err(mask_errno) } else { open_exit_watch(parent, program) };
    let errno = match watch {
        Ok(_) => 0,
        Err(errno) => errno,
    };
    let bytes = errno.to_ne_bytes();
    let written = loop {
        // SAFETY: writer is the sole retained pipe end; four initialized bytes
        // fit PIPE_BUF. Signals are blocked, including SIGPIPE.
        let count = unsafe { libc::write(writer, bytes.as_ptr().cast(), bytes.len()) };
        if count < 0 && last_errno() == libc::EINTR {
            continue;
        }
        break count == 4;
    };
    close(writer);
    if written && let Ok((first, second)) = watch {
        // Bit 2 is the program's exit: it wins even when both exits arrive
        // in this wait. No signal is sent after an error in the wait either.
        if let Ok(1) = wait_exit_watch(first, second, parent, program) {
            // SAFETY: getppid takes no arguments. Reparenting after the
            // program's death prevents signalling a recycled numeric pid.
            if unsafe { libc::getppid() } == program {
                // SAFETY: program is still our parent; kill takes integers only.
                let _sent = unsafe { libc::kill(program, signal) };
            }
        }
    }
    // SAFETY: end the watcher without any Rust destructors or libc cleanup;
    // the kernel closes its wait descriptors.
    unsafe { libc::_exit(0) }
}

/// Spawn transports only errno; supply the diagnostic outside the hook.
pub(crate) fn parent_error(errno: i32) -> std::io::Error {
    if errno == libc::ESRCH {
        std::io::Error::new(std::io::ErrorKind::NotFound, "parent changed before exec")
    } else {
        std::io::Error::from_raw_os_error(errno)
    }
}
