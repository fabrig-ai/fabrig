//! Startup terminal modes and window sizes (runtime/shell.md, section 6.3).
use skein_io::kernel::{Error, Fd, Size};

/// Whether interrupt keys keep their terminal signals once raw.
#[derive(Clone, Copy, Debug)]
pub enum Keys {
    /// Keep the terminal making interrupt signals.
    Signal,
    /// Pass interrupt keys through as input bytes.
    Pass,
}
/// Which signals startup reads from its one signalfd.
#[derive(Clone, Copy, Debug)]
pub enum Watch {
    /// Read SIGINT and SIGTERM.
    Termination,
    /// Read SIGINT, SIGTERM and SIGWINCH.
    TerminationAndWindow,
    /// Read SIGINT, SIGTERM and SIGHUP, leaving a SIGHUP ignored at startup ignored.
    TerminationAndHangup,
}
/// What startup found on its input descriptor.
pub enum Input {
    /// Raw input was set, with its original modes and startup size.
    Terminal { kept: Kept, size: Size },
    /// Input was left as it was.
    NotATerminal,
}
/// The saved modes and a duplicate input descriptor, kept until drive returns.
pub struct Kept {
    pub(crate) fd: Fd,
    pub(crate) modes: libc::termios,
    pub(crate) keys: Keys,
}
/// The modes visible from a terminal's other end.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Modes {
    pub echo: bool,
    pub lines: bool,
    pub signals: bool,
}
/// Sets input to key-at-a-time reading, retaining the modes for restoration.
pub fn raw_input(input: Fd, keys: Keys) -> Result<Input, Error> {
    crate::system::terminal::raw_input(input, keys)
}
/// Opens the process's controlling terminal, `/dev/tty`, for reading and
/// writing, close-on-exec, never making it a controlling terminal
/// (runtime/shell.md, 6.3). The caller owns the descriptor.
pub fn open_terminal() -> Result<Fd, Error> {
    crate::system::terminal::open_terminal()
}
/// Makes the caller's process group the foreground group of `terminal`,
/// with SIGTTOU blocked for the call, so that a background group may take
/// it (runtime/shell.md, 6.3).
pub fn take_foreground(terminal: Fd) -> Result<(), Error> {
    crate::system::terminal::take_foreground(terminal)
}
/// Blocks the selected signals and opens the signalfd io adopts.
pub fn open_signals(watch: Watch) -> Result<Fd, i32> {
    crate::system::process::open_signals(watch)
}
impl Kept {
    /// Sets the saved modes back without giving up the saved descriptor or modes.
    pub fn set_kept(&self) -> Result<(), Error> {
        crate::system::terminal::set(self.fd, &self.modes)
    }

    /// Sets raw modes again with the interrupt-key policy requested at startup.
    pub fn set_raw(&self) -> Result<(), Error> {
        crate::system::terminal::set_raw(self.fd, &self.modes, self.keys)
    }

    /// Writes `leave` to `output` with blocking writes, then restores the kept modes.
    /// Errors are ignored: this runs inside a panic (runtime/shell.md, 6.3).
    /// Take the function back from the hook before calling [`Self::restore`].
    pub fn on_abort(&self, output: Fd, leave: &'static [u8]) -> Box<dyn Fn() + Send + Sync> {
        let fd = self.fd;
        let modes = self.modes;
        Box::new(move || crate::system::terminal::on_abort(fd, &modes, output, leave))
    }

    /// Restores startup's modes after drive returns, before the last stderr line.
    pub fn restore(self) -> Result<(), Error> {
        crate::system::terminal::restore(self.fd, &self.modes)
    }
}
/// Sets a pseudo-terminal size from its master, for a scripted person.
pub fn resize_terminal(master: Fd, size: Size) -> Result<(), Error> {
    crate::system::terminal::resize(master, size)
}
/// Reads modes from the other end of a terminal, for a scripted person.
pub fn terminal_modes(fd: Fd) -> Result<Modes, Error> {
    crate::system::terminal::modes(fd)
}

#[cfg(test)]
mod tests {
    use super::{Input, Kept, Keys, raw_input};
    use skein_io::kernel::{Error, Fd};
    use std::os::fd::AsRawFd;

    #[test]
    fn kept_mode_changes_refuse_a_pipe() {
        let (reader, _writer) = std::io::pipe().expect("a pipe is not a terminal");
        let fd = Fd::new(reader.as_raw_fd());
        assert!(matches!(raw_input(fd, Keys::Pass), Ok(Input::NotATerminal)));
        // tcsetattr must refuse the descriptor, whatever modes were kept.
        let kept = Kept {
            fd,
            modes: libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                #[cfg(target_os = "linux")]
                c_line: 0,
                c_cc: [0; libc::NCCS],
                c_ispeed: 0,
                c_ospeed: 0,
            },
            keys: Keys::Pass,
        };
        assert_eq!(kept.set_kept(), Err(Error::Other(libc::ENOTTY)));
        assert_eq!(kept.set_raw(), Err(Error::Other(libc::ENOTTY)));
    }
}
