//! The terminal's loan (temper/docs/design/client/tui/box.md, Lending its terminal).
use super::{Run, host};
use jig_client::contract::LoginExit;
use skein_io::{self as io, kernel::{Complete, Done, Exit, Fd, Op, Signal, Submit, Target}};
use skein_lib::{Queue, Token};

pub(super) const LOGIN: Token = Token::new(u64::MAX - 3);
pub(super) const LINE: Token = Token::new(6 << 60);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Lending,
    Writing,
    Starting,
    Spawning,
    Running,
    Stopping,
    TakingBack,
}

pub(super) struct Lending {
    phase: Phase,
    lend: Option<host::Lend>,
    child: Option<Token>,
    exit: LoginExit,
    stop: bool,
    input: Fd,
    output: Fd,
    writes: Queue<Submit>,
    requests: Queue<io::Request>,
}

impl Lending {
    pub(super) fn new(input: Fd, output: Fd) -> Self {
        Self {
            phase: Phase::Idle,
            lend: None,
            child: None,
            exit: LoginExit::NotStarted,
            stop: false,
            input,
            output,
            writes: Queue::uncapped(),
            requests: Queue::uncapped(),
        }
    }
    pub(super) fn begin(&mut self, lend: host::Lend) {
        assert!(!self.busy(), "one loan");
        self.lend = Some(lend);
        self.phase = Phase::Lending;
        self.exit = LoginExit::NotStarted;
        self.stop = false;
    }
    pub(super) fn busy(&self) -> bool {
        self.phase != Phase::Idle
    }
    pub(super) fn pending(&self, lent: bool) -> bool {
        !self.writes.is_empty() || !self.requests.is_empty()
            || (self.phase == Phase::Lending && lent)
            || self.phase == Phase::TakingBack
    }
    pub(super) fn stop(&mut self) {
        self.stop = true;
        if self.phase == Phase::Running {
            self.phase = Phase::Stopping;
            self.requests.push(io::Request::Signal {
                child: self.child.expect("running loan"), signal: Signal::Terminate, to: Target::Group,
            });
        }
    }
    pub(super) fn written(&mut self, complete: Complete) {
        let Op::PipeWrite { fd, bytes, from } = complete.kind else { unreachable!("loan's line write") };
        assert!(self.phase == Phase::Writing);
        match complete.result {
            Ok(Done::Count(count)) if count > 0 => {
                let from = from.checked_add(count).expect("line offset");
                if usize::try_from(from).expect("line offset") < bytes.len() {
                    self.writes.push(Submit { op: LINE, kind: Op::PipeWrite { fd, bytes, from } });
                } else {
                    self.phase = Phase::Starting;
                }
            }
            Ok(Done::Count(_)) | Err(_) => self.phase = Phase::TakingBack,
            Ok(Done::Accepted { .. } | Done::Fd(_) | Done::Nothing | Done::Spawned { .. }
                | Done::Exit(_) | Done::Window(_) | Done::Signalled(_)
                | Done::Stat(_) | Done::Bound(_) | Done::Usage(_)) => unreachable!("line write returns a count"),
        }
    }
    pub(super) fn event(&mut self, event: &io::Event) -> bool {
        match event {
            io::Event::Spawned { owner, child, pipes } if *owner == LOGIN => {
                assert!(pipes.is_empty(), "login inherits the terminal");
                self.child = Some(*child);
                self.phase = Phase::Running;
                if self.stop { self.stop(); }
                true
            }
            io::Event::Exited { owner, exit } if *owner == LOGIN => {
                self.exit = match exit {
                    Exit::Code(code) => LoginExit::Exited { code: *code },
                    Exit::Signal(signal) => LoginExit::Killed { signal: u8::try_from(*signal).expect("signal fits u8") },
                };
                self.requests.push(io::Request::Abort { entity: self.child.expect("spawned login") });
                true
            }
            io::Event::Failed { owner, .. } if *owner == LOGIN => true,
            io::Event::Closed { owner } if *owner == LOGIN => {
                self.child = None;
                self.phase = Phase::TakingBack;
                true
            }
            io::Event::Shutdown { .. } | io::Event::Failed { .. } | io::Event::Spawned { .. }
            | io::Event::Exited { .. } | io::Event::Closed { .. } | io::Event::Stream { .. }
            | io::Event::Resized | io::Event::Output { .. } | io::Event::Listening { .. }
            | io::Event::Accepted { .. } | io::Event::Connecting { .. } | io::Event::Connected { .. }
            | io::Event::Usage { .. } | io::Event::Window { .. } => false,
        }
    }
}

impl Run {
    /// Gives the run startup's saved modes, retained until its final restoration.
    #[must_use]
    pub fn with_kept(mut self, kept: skein_shell::Kept) -> Self {
        assert!(self.kept.is_none(), "startup supplies modes once");
        self.kept = Some(kept);
        self
    }

    /// Restores startup's modes after the run leaves its terminal.
    pub fn restore_terminal(&mut self) {
        if let Some(kept) = self.kept.take() {
            kept.restore().expect("restore terminal modes");
        }
    }

    pub(super) fn lending_progress(&mut self) {
        if self.lending.phase == Phase::Lending && self.terminal.lent() {
            if let Some(kept) = &self.kept {
                kept.set_kept().expect("set kept terminal modes");
            }
            let mut line = self.lending.lend.as_ref().expect("loan").line.to_vec();
            line.push(b'\n');
            self.lending.phase = Phase::Writing;
            self.lending.writes.push(Submit {
                op: LINE, kind: Op::PipeWrite { fd: self.lending.output, bytes: line.into(), from: 0 },
            });
        }
        if self.lending.phase == Phase::Starting {
            let name = &self.lending.lend.as_ref().expect("loan").name;
            let valid = (1..=32).contains(&name.len())
                && name.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
            let spawn = valid.then(|| self.box_child.as_ref().and_then(|front| front.login(name))).flatten();
            if let Some(spawn) = spawn.filter(|_| !self.lending.stop) {
                self.lending.phase = Phase::Spawning;
                self.lending.requests.push(io::Request::Spawn { owner: LOGIN, spawn });
            } else {
                self.lending.phase = Phase::TakingBack;
            }
        }
        while self.submit.room() > 0 {
            let Some(write) = self.lending.writes.pop() else { break };
            self.submit.push(write);
        }
        while self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions {
            let Some(request) = self.lending.requests.pop() else { break };
            self.down(request);
        }
        if self.lending.phase == Phase::TakingBack {
            if let Some(kept) = &self.kept {
                skein_shell::take_foreground(self.lending.input).expect("take terminal foreground back");
                kept.set_raw().expect("set raw terminal modes");
            }
            self.terminal.resume();
            self.lending.phase = Phase::Idle;
            let number = self.lending.lend.take().expect("loan").number;
            self.report(host::Report::Lent { number, exit: self.lending.exit });
            self.terminal.draw(self.host.frame());
            if self.close_after_lending {
                self.close_after_lending = false;
                self.close();
            }
        }
    }
}
