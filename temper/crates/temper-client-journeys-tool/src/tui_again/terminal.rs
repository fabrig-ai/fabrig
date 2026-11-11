//! The TUI's terminal, owned by the walker
//! (temper/docs/design/client/tui/journeys.md, 8.2).
//! A shell waits for one line before exec: the master is resized before the
//! TUI can read its screen size. All output is drained on this same thread.
use super::Paths;
use skein_io::kernel::{Complete, Done, Fd, Op, Pipe, Signal, Size, Spawn, Submit, Target, Way};
use skein_lib::{Queue, Token};
use skein_shell::{Config, Kernel, Wait, keeper};
use std::{collections::VecDeque, ffi::OsString, os::unix::ffi::OsStrExt};

pub(super) struct Terminal {
    kernel: Kernel,
    pidfd: Fd,
    master: Fd,
    reading: Option<Token>,
    writing: Option<Token>,
    writes: VecDeque<Box<[u8]>>,
    submits: Queue<Submit>,
    complete: Queue<Complete>,
    serial: u64,
}
impl Terminal {
    pub(super) fn start(
        paths: &Paths,
        address: &[u8],
        environment: &[(OsString, OsString)],
    ) -> Result<Self, &'static str> {
        let kernel = Kernel::open(Config { operations: 8 }).map_err(|_| "cannot open TUI terminal kernel")?;
        let root = skein_shell::open_root(&paths.scratch).map_err(|_| "cannot open TUI scratch directory")?;
        let mut spawn = Spawn {
            program: b"/bin/sh".as_slice().into(),
            args: vec![
                b"-c".as_slice().into(),
                b"read -r ready; exec \"$@\"".as_slice().into(),
                b"tui-again".as_slice().into(),
                paths.temper.as_os_str().as_bytes().into(),
                address.into(),
            ]
            .into_boxed_slice(),
            env: environment
                .iter()
                .map(|(name, value)| {
                    let mut bytes = name.as_bytes().to_vec();
                    bytes.push(b'=');
                    bytes.extend_from_slice(value.as_bytes());
                    bytes.into_boxed_slice()
                })
                .collect(),
            root,
            dir: b".".as_slice().into(),
            pipes: vec![Pipe { child: 2, way: Way::Null, parent: None }].into_boxed_slice(),
        };
        let child = keeper::start_binary(&mut spawn, true);
        skein_shell::close_startup_fd(root);
        let (pidfd, master) = child.map_err(|_| "cannot start TUI on a pseudo-terminal")?;
        let mut terminal = Self {
            kernel,
            pidfd,
            master: master.expect("terminal startup returns its master"),
            reading: None,
            writing: None,
            writes: VecDeque::new(),
            submits: Queue::with_capacity(8),
            complete: Queue::with_capacity(8),
            serial: 0,
        };
        skein_shell::resize_terminal(terminal.master, Size { rows: 24, columns: 80 })
            .map_err(|_| "cannot size TUI terminal to 80 by 24")?;
        terminal.write(b"\n".to_vec());
        terminal.tick()?;
        Ok(terminal)
    }
    fn submit(&mut self, kind: Op) -> Token {
        self.serial += 1;
        let op = Token::new(self.serial);
        self.submits.push(Submit { op, kind });
        op
    }
    pub(super) fn ready(&self) -> bool {
        skein_shell::terminal_modes(self.master).is_ok_and(|modes| !modes.lines && !modes.echo && !modes.signals)
    }
    pub(super) fn write(&mut self, bytes: Vec<u8>) {
        self.writes.push_back(bytes.into());
    }
    pub(super) fn tick(&mut self) -> Result<(), &'static str> {
        self.kernel.submit(&mut self.submits, Wait::No);
        self.kernel.reap(&mut self.complete);
        while let Some(complete) = self.complete.pop() {
            if self.reading == Some(complete.op) {
                self.reading = None;
                if !matches!(complete.result, Ok(Done::Count(count)) if count > 0) {
                    return Err("TUI terminal closed before the walk ended");
                }
            } else if self.writing == Some(complete.op) {
                self.writing = None;
                let Op::PipeWrite { bytes, from, .. } = complete.kind else { unreachable!("terminal write") };
                let Ok(Done::Count(count)) = complete.result else { return Err("cannot write a TUI key") };
                if count == 0 {
                    return Err("TUI key write made no progress");
                }
                let from = from.checked_add(count).ok_or("TUI key write overflow")?;
                if usize::try_from(from).expect("write offset fits usize") < bytes.len() {
                    self.writing = Some(self.submit(Op::PipeWrite { fd: self.master, bytes, from }));
                }
            }
        }
        if keeper::poll_child(self.pidfd, false).map_err(|_| "cannot observe TUI exit")?.is_some() {
            return Err("TUI exited before the walk ended");
        }
        if self.reading.is_none() {
            self.reading = Some(self.submit(Op::PipeRead { fd: self.master, buf: vec![0; 32768].into() }));
        }
        if self.writing.is_none()
            && let Some(bytes) = self.writes.pop_front()
        {
            self.writing = Some(self.submit(Op::PipeWrite { fd: self.master, bytes, from: 0 }));
        }
        self.kernel.submit(&mut self.submits, Wait::No);
        Ok(())
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _signalled = keeper::signal_kept_child(self.pidfd, Signal::Kill, Target::Group);
        for target in [self.reading.take(), self.writing.take()].into_iter().flatten() {
            self.submit(Op::Cancel { target });
        }
        // Pending reads and writes release their buffers before any descriptor
        // is closed; the keeper then reaps only this owned child and its group.
        self.kernel.submit(&mut self.submits, Wait::No);
        while self.kernel.in_flight() != 0 {
            self.kernel.reap(&mut self.complete);
            self.complete.clear();
            if self.kernel.in_flight() != 0 {
                self.kernel.submit(&mut self.submits, Wait::Forever);
            }
        }
        keeper::abandon_binary(Some(self.pidfd), &[self.pidfd, self.master]);
    }
}
