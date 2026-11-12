//! Terminal syscalls shared by Linux and Darwin (runtime/shell.md, section 6.3, and runtime/readiness.md, section 5).
//!
//! # The `unsafe`, and why it is sound
//!
//! Terminal calls borrow initialized local modes or window sizes until
//! return. Output storage is read only after the call reports success.
//! Fcntl duplicates the live input descriptor; restoration consumes and
//! closes that duplicate once. No call retains a pointer to Rust storage.
//! Writes borrow a live byte slice only until the synchronous call returns.
//! Open borrows a terminated path; signal masks are local initialized C
//! storage. No call retains either pointer, and the prior mask is restored.
#![expect(unsafe_code, reason = "terminal calls borrow local storage only until return (runtime/readiness.md, 5)")]

use crate::terminal::{Input, Kept, Keys, Modes};
use skein_io::kernel::{Error, Fd, Size};
use std::{mem::MaybeUninit, ptr};

pub(crate) fn open_terminal() -> Result<Fd, Error> {
    // SAFETY: the path is terminated and open retains no pointer to it.
    let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC) };
    if fd < 0 { Err(Error::Other(crate::system::last_errno())) } else { Ok(Fd::new(fd)) }
}

pub(crate) fn take_foreground(terminal: Fd) -> Result<(), Error> {
    // SAFETY: sigset_t is C storage, initialized by sigemptyset below.
    let mut blocked: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: sigemptyset initializes this live signal set.
    if unsafe { libc::sigemptyset(ptr::from_mut(&mut blocked)) } != 0 {
        return Err(Error::Other(crate::system::last_errno()));
    }
    // SAFETY: blocked is initialized and SIGTTOU is a valid signal.
    if unsafe { libc::sigaddset(ptr::from_mut(&mut blocked), libc::SIGTTOU) } != 0 {
        return Err(Error::Other(crate::system::last_errno()));
    }
    let mut previous = MaybeUninit::uninit();
    // SAFETY: blocked is initialized; the prior thread mask is written to live storage.
    let error = unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, ptr::from_ref(&blocked), previous.as_mut_ptr()) };
    if error != 0 {
        return Err(Error::Other(error));
    }
    // SAFETY: successful pthread_sigmask initialized the prior mask.
    let previous = unsafe { previous.assume_init() };
    // SAFETY: getpgrp reads the calling process's group and borrows no storage.
    let group = unsafe { libc::getpgrp() };
    // SAFETY: tcsetpgrp borrows no storage, and SIGTTOU is blocked in this thread.
    let result = if unsafe { libc::tcsetpgrp(terminal.raw(), group) } == 0 {
        Ok(())
    } else {
        Err(Error::Other(crate::system::last_errno()))
    };
    // SAFETY: previous is initialized and borrowed only until the thread mask is restored.
    let error = unsafe { libc::pthread_sigmask(libc::SIG_SETMASK, ptr::from_ref(&previous), ptr::null_mut()) };
    result?;
    if error == 0 { Ok(()) } else { Err(Error::Other(error)) }
}

#[cfg(test)]
pub(crate) fn signal_mask() -> Vec<i32> {
    // SAFETY: sigset_t is C storage; zeroing also initializes any unused storage.
    let mut mask: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: a null input only reads this thread's mask into live storage.
    assert_eq!(unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, ptr::null(), ptr::from_mut(&mut mask)) }, 0);
    let bits = size_of_val(&mask).checked_mul(8).expect("signal set bits fit");
    let mut members = Vec::new();
    // The set's storage bounds all signal numbers; unsupported numbers return -1.
    for signal in 1..=i32::try_from(bits).expect("signal numbers fit") {
        // SAFETY: mask is initialized; sigismember only reads it during this call.
        if unsafe { libc::sigismember(ptr::from_ref(&mask), signal) } == 1 {
            members.push(signal);
        }
    }
    members
}

fn read(fd: Fd) -> Result<libc::termios, Error> {
    let mut modes = MaybeUninit::uninit();
    // SAFETY: tcgetattr writes a complete termios into this live allocation.
    if unsafe { libc::tcgetattr(fd.raw(), modes.as_mut_ptr()) } != 0 {
        return Err(Error::Other(crate::system::last_errno()));
    }
    // SAFETY: successful tcgetattr initialized every termios field.
    Ok(unsafe { modes.assume_init() })
}
pub(crate) fn set(fd: Fd, modes: &libc::termios) -> Result<(), Error> {
    // SAFETY: modes is an initialized termios, borrowed for this synchronous call.
    if unsafe { libc::tcsetattr(fd.raw(), libc::TCSANOW, ptr::from_ref(modes)) } == 0 {
        Ok(())
    } else {
        Err(Error::Other(crate::system::last_errno()))
    }
}
pub(crate) fn size(fd: Fd) -> Result<Size, Error> {
    let mut size = MaybeUninit::<libc::winsize>::uninit();
    // SAFETY: TIOCGWINSZ writes one winsize into this live allocation.
    if unsafe { libc::ioctl(fd.raw(), libc::TIOCGWINSZ, size.as_mut_ptr()) } != 0 {
        return Err(Error::Other(crate::system::last_errno()));
    }
    // SAFETY: successful ioctl initialized the winsize.
    let size = unsafe { size.assume_init() };
    Ok(Size { rows: size.ws_row, columns: size.ws_col })
}
pub(crate) fn raw_input(fd: Fd, keys: Keys) -> Result<Input, Error> {
    let modes = match read(fd) {
        Ok(modes) => modes,
        Err(Error::Other(libc::ENOTTY)) => return Ok(Input::NotATerminal),
        Err(error) => return Err(error),
    };
    let size = size(fd)?;
    // SAFETY: fcntl duplicates the live input descriptor with close-on-exec.
    let saved = unsafe { libc::fcntl(fd.raw(), libc::F_DUPFD_CLOEXEC, 3) };
    if saved < 0 {
        return Err(Error::Other(crate::system::last_errno()));
    }
    let kept = Kept { fd: Fd::new(saved), modes, keys };
    if let Err(error) = set_raw(fd, &modes, keys) {
        // SAFETY: saved is the descriptor just opened above, uniquely owned here.
        unsafe { libc::close(saved) };
        return Err(error);
    }
    Ok(Input::Terminal { kept, size })
}
pub(crate) fn set_raw(fd: Fd, modes: &libc::termios, keys: Keys) -> Result<(), Error> {
    let mut raw = *modes;
    raw.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON);
    match keys {
        Keys::Signal => raw.c_lflag |= libc::ISIG,
        Keys::Pass => raw.c_lflag &= !libc::ISIG,
    }
    *raw.c_cc.get_mut(libc::VMIN).expect("termios has VMIN") = 1;
    *raw.c_cc.get_mut(libc::VTIME).expect("termios has VTIME") = 0;
    set(fd, &raw)
}
pub(crate) fn restore(fd: Fd, modes: &libc::termios) -> Result<(), Error> {
    let result = set(fd, modes);
    // SAFETY: this is the duplicated descriptor consumed by Kept::restore.
    unsafe { libc::close(fd.raw()) };
    result
}
pub(crate) fn on_abort(fd: Fd, modes: &libc::termios, output: Fd, mut leave: &[u8]) {
    while !leave.is_empty() {
        // SAFETY: write reads only the live slice during this synchronous call.
        let written = unsafe { libc::write(output.raw(), leave.as_ptr().cast(), leave.len()) };
        if written < 0 {
            if crate::system::last_errno() == libc::EINTR {
                continue;
            }
            break;
        }
        if written == 0 {
            break;
        }
        leave = leave.get(written.unsigned_abs()..).expect("the kernel wrote within its slice");
    }
    let _restore_result = set(fd, modes);
}
pub(crate) fn modes(fd: Fd) -> Result<Modes, Error> {
    let modes = read(fd)?;
    Ok(Modes {
        echo: modes.c_lflag & libc::ECHO != 0,
        lines: modes.c_lflag & libc::ICANON != 0,
        signals: modes.c_lflag & libc::ISIG != 0,
    })
}
pub(crate) fn resize(fd: Fd, size: Size) -> Result<(), Error> {
    let size = libc::winsize { ws_row: size.rows, ws_col: size.columns, ws_xpixel: 0, ws_ypixel: 0 };
    // SAFETY: TIOCSWINSZ reads the initialized winsize during this synchronous call.
    if unsafe { libc::ioctl(fd.raw(), libc::TIOCSWINSZ, ptr::from_ref(&size)) } == 0 {
        Ok(())
    } else {
        Err(Error::Other(crate::system::last_errno()))
    }
}
