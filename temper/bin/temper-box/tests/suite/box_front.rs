//! The binary hands its terminal to its signed-in front (temper/docs/design/protocol/setup/box.md, 6).
#[expect(dead_code, reason = "the shared box fixture also supports pipe and sign-in stories")]
#[expect(clippy::duplicate_mod, reason = "the front and listener stories include the shared box fixture")]
#[path = "../fake_backends/box_link.rs"]
mod box_link;
use crate::support::{self, patience::Patience};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Signal, Size, Spawn, Submit, Target, Way};
use skein_lib::{Queue, Token};
use skein_shell::{Config, Kernel, Wait, keeper};
use std::{collections::BTreeMap, fmt::Write, fs, os::unix::ffi::OsStrExt, path::Path};

// One read, one write and their cancellation records fit the terminal's ring.
const OPERATIONS: u32 = 8;
// As the journey terminal, drain a screen stream in 32 KiB reads.
const READ_BYTES: usize = 32768;
// A fresh welcome and its terminal controls fit within the box-link fixture's output bound.
const OUTPUT_BYTES: usize = 262_144;

struct Terminal {
    kernel: Kernel,
    pidfd: Fd,
    master: Fd,
    reading: Option<Token>,
    writing: Option<Token>,
    ended: bool,
    complete: Queue<Complete>,
    submits: Queue<Submit>,
    serial: u64,
    bytes: Vec<u8>,
}
impl Terminal {
    fn start(root: &Path) -> Self {
        let mut command = box_link::command(root, "");
        command.env("XDG_STATE_HOME", root.join("state"));
        if temper_test_env::value(&temper_shell::environment::TERM).is_none() {
            command.env("TERM", "xterm-256color");
        }
        let mut environment: BTreeMap<_, _> = temper_test_env::environment()
            .into_vec()
            .into_iter()
            .map(|(name, value)| {
                (std::ffi::OsStr::from_bytes(&name).to_os_string(), std::ffi::OsStr::from_bytes(&value).to_os_string())
            })
            .collect();
        for (name, value) in command.get_envs() {
            if let Some(value) = value {
                environment.insert(name.to_owned(), value.to_owned());
            } else {
                environment.remove(name);
            }
        }
        // The shell waits until the master has its size before starting the box.
        let mut arguments = vec![
            b"-c".as_slice().into(),
            b"read -r ready; exec \"$@\"".as_slice().into(),
            b"box-front".as_slice().into(),
            command.get_program().as_bytes().into(),
        ];
        arguments.extend(command.get_args().map(|word| Box::from(word.as_bytes())));
        let kernel = Kernel::open(Config { operations: OPERATIONS }).expect("terminal kernel");
        let directory = skein_shell::open_root(root).expect("scratch root");
        let mut spawn = Spawn {
            program: b"/bin/sh".as_slice().into(),
            args: arguments.into_boxed_slice(),
            env: environment
                .into_iter()
                .map(|(name, value)| {
                    let mut bytes = name.as_bytes().to_vec();
                    bytes.push(b'=');
                    bytes.extend_from_slice(value.as_bytes());
                    bytes.into_boxed_slice()
                })
                .collect(),
            root: directory,
            dir: b".".as_slice().into(),
            pipes: Box::new([Pipe { child: 2, way: Way::Null, parent: None }]),
        };
        let started = keeper::start_binary(&mut spawn, true);
        skein_shell::close_startup_fd(directory);
        let (pidfd, master) = started.expect("start box on a pseudo-terminal in its own session");
        let mut terminal = Self {
            kernel,
            pidfd,
            master: master.expect("pseudo-terminal master"),
            reading: None,
            writing: None,
            ended: false,
            complete: Queue::with_capacity(OPERATIONS),
            submits: Queue::with_capacity(OPERATIONS),
            serial: 0,
            bytes: Vec::new(),
        };
        skein_shell::resize_terminal(terminal.master, Size { columns: 80, rows: 24 }).expect("80 by 24 terminal");
        terminal.write(b'\n');
        terminal
    }
    fn submit(&mut self, kind: Op) -> Token {
        self.serial += 1;
        let op = Token::new(self.serial);
        self.submits.push(Submit { op, kind });
        op
    }
    fn write(&mut self, byte: u8) {
        assert!(self.writing.is_none(), "previous terminal byte completed");
        self.writing = Some(self.submit(Op::PipeWrite { fd: self.master, bytes: vec![byte].into(), from: 0 }));
    }
    fn tick(&mut self) -> Option<Exit> {
        self.kernel.submit(&mut self.submits, Wait::No);
        self.kernel.reap(&mut self.complete);
        while let Some(done) = self.complete.pop() {
            if self.reading == Some(done.op) {
                self.reading = None;
                let Op::PipeRead { buf, .. } = done.kind else { panic!("terminal read") };
                match done.result {
                    Ok(Done::Count(count)) if count > 0 => {
                        self.bytes.extend_from_slice(&buf[..usize::try_from(count).expect("read count")]);
                        assert!(self.bytes.len() <= OUTPUT_BYTES, "bounded fresh-box terminal output");
                    }
                    Ok(Done::Count(_)) | Err(_) => self.ended = true,
                    other => panic!("terminal read count: {other:?}"),
                }
            } else if self.writing == Some(done.op) {
                self.writing = None;
                assert!(matches!(done.result, Ok(Done::Count(1))), "one terminal byte written");
            } else {
                panic!("unknown terminal completion");
            }
        }
        if !self.ended && self.reading.is_none() {
            self.reading = Some(self.submit(Op::PipeRead { fd: self.master, buf: vec![0; READ_BYTES].into() }));
        }
        self.kernel.submit(&mut self.submits, Wait::No);
        keeper::poll_child(self.pidfd, false).expect("box exit observation")
    }
    fn wait(&mut self, root: &Path, what: &'static str, ready: impl Fn(&Self) -> bool) {
        let mut patience = Patience::arm(what, support::WITHIN);
        patience.watch_process(keeper::child_id(self.pidfd).expect("box PID"));
        loop {
            let exit = self.tick();
            assert!(exit.is_none(), "box exited before {what}: {exit:?}\n{}", log(root));
            if ready(self) {
                return;
            }
            if let Some(figures) = patience.failed() {
                let progress = self.progress();
                panic!("waiting for {what}: {figures}; {progress}\n{}", log(root));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    fn progress(&self) -> String {
        let raw =
            skein_shell::terminal_modes(self.master).is_ok_and(|modes| !modes.lines && !modes.echo && !modes.signals);
        let connecting_seen = skein_lib::bytes::find(&self.bytes, b"Connecting").is_some();
        let live_seen = skein_lib::bytes::find(&self.bytes, b"Live").is_some();
        let mut progress = String::new();
        write!(
            &mut progress,
            "raw={raw}, connecting_seen={connecting_seen}, live_seen={live_seen}, terminal bytes={}",
            self.bytes.len()
        )
        .expect("progress text");
        #[cfg(target_os = "linux")]
        for pid in support::parent::Guard::observing_test().pids() {
            let status = fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
            let state = status.lines().find(|line| line.starts_with("State:")).unwrap_or("state unavailable");
            let wait = fs::read_to_string(format!("/proc/{pid}/wchan")).unwrap_or_default();
            write!(&mut progress, "; process {pid}: {state}, wait={wait}").expect("progress text");
        }
        progress
    }
    fn stopped(&mut self, root: &Path) -> Exit {
        let mut patience = Patience::arm("box stops with its front", support::WITHIN);
        patience.watch_process(keeper::child_id(self.pidfd).expect("box PID"));
        loop {
            let exit = self.tick();
            if let Some(exit) = exit
                && self.ended
            {
                return exit;
            }
            if let Some(figures) = patience.failed() {
                panic!("box did not stop: {figures}\n{}", log(root));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _signalled = keeper::signal_kept_child(self.pidfd, Signal::Kill, Target::Group);
        for target in [self.reading.take(), self.writing.take()].into_iter().flatten() {
            self.submit(Op::Cancel { target });
        }
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

fn log(root: &Path) -> String {
    let mut text = String::new();
    for name in ["box", "serve", "tui"] {
        if let Ok(log) = fs::read_to_string(root.join(format!("data/box/logs/{name}.log"))) {
            write!(&mut text, "{name}:\n{log}").expect("log text");
        }
    }
    text
}

#[test]
fn a_box_in_a_terminal_runs_its_tui_in_front_and_stops_with_it() {
    let scratch = support::Scratch::startup("box-front");
    let root = scratch.path();
    let mut terminal = Terminal::start(root);
    let path = box_link::link_path(root);
    // Serve writes a fresh local link only after the handed link signed the front in.
    terminal.wait(root, "front signs in", |_| fs::read(&path).is_ok_and(|bytes| bytes.ends_with(b"\n")));
    terminal.wait(root, "front takes raw input", |terminal| {
        terminal.writing.is_none()
            && skein_shell::terminal_modes(terminal.master)
                .is_ok_and(|modes| !modes.lines && !modes.echo && !modes.signals)
    });
    #[cfg(target_os = "linux")]
    let observed = support::parent::Guard::observing_test();
    terminal.write(0x03);
    let exit = terminal.stopped(root);
    assert_eq!(exit, Exit::Code(0), "box drains after its TUI quits\n{}", log(root));
    let log = log(root);
    let mut remaining = log.as_str();
    for fact in ["tui exited 0", "Stopping", "Stopped"] {
        remaining = remaining.split_once(fact).expect("ordered front exit and box stop").1;
    }
    for entry in fs::read_dir(root.join("data/box/logs")).expect("box logs") {
        let bytes = fs::read(entry.expect("log entry").path()).expect("log bytes");
        assert!(skein_lib::bytes::find(&bytes, b"secret=").is_none(), "no link secret in a log");
    }
    assert!(
        skein_lib::bytes::find(&terminal.bytes, b"Open this link in a browser").is_none(),
        "the box's printed block stays off the front's terminal"
    );
    #[cfg(target_os = "linux")]
    assert!(observed.all_exited(), "no process outlived the fixture box");
    drop(terminal);
    scratch.remove();
}
