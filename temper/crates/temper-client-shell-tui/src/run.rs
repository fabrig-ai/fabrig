//! The TUI on one kernel and clock (temper/docs/design/client/README.md, 6.16).
use crate::{
    host, limits,
    opener::{self, Opener},
};
use skein_io::{
    self as io,
    kernel::{Complete, Exit, Fd, Op, Submit},
};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_shell::Host;
use skein_ui_screen::Size;
use skein_ui_screen_terminal::{Depth, Terminal};
use temper_client_tui::Scheme;

const TAG: u64 = 1 << 61;
const ROOM: u32 = 64;
const OPENER: Token = Token::new(u64::MAX);

pub struct Settings {
    pub engine: io::Addr,
    pub directory: Box<[u8]>,
    pub address: Box<[u8]>,
    pub scheme: Scheme,
    pub depth: Depth,
    pub size: Size,
    pub limits: limits::Limits,
    pub opener: Option<Opener>,
    pub record: Option<Fd>,
}

pub struct Run {
    host: host::Host,
    shell: jig_client_shell_native::Shell,
    terminal: Terminal,
    io: io::Io,
    env: Env<io::Limits>,
    signals: Option<Token>,
    root: Fd,
    opener: Option<Opener>,
    opens: Queue<Box<[u8]>>,
    opening: bool,
    child: Option<Token>,
    opened: Option<bool>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    raw: Queue<Submit>,
    told: Queue<io::Event>,
    operations: u32,
    start: Option<(Scheme, Box<[u8]>)>,
    size: Option<Size>,
    signed_in: Option<Box<[u8]>>,
    closing: bool,
    refused: bool,
}

impl Run {
    /// # Errors
    /// The descriptor io could not adopt.
    pub fn new(
        settings: Settings,
        root: Fd,
        user: u32,
        seed: u64,
        input: Fd,
        output: Fd,
        signals: Fd,
    ) -> Result<Run, Fd> {
        let host = host::Host::new(&host::Settings { limits: settings.limits.clone() }, seed, Time::ZERO, Wall::EPOCH);
        let terminal_settings = limits::terminal(&settings.limits, settings.depth);
        let mut shell = jig_client_shell_native::Shell::new(jig_client_shell_native::Settings {
            engine: settings.engine,
            directory: settings.directory,
            limits: settings.limits.native.connections,
            io: settings.limits.native.io,
        });
        shell.adopt(root, user, seed);
        let mut terminal = Terminal::new(terminal_settings, settings.size).expect("checked terminal settings and size");
        terminal.adopt(input, output)?;
        let read = temper_client_shell_terminal::limits::terminal().read;
        let limits = io::Limits {
            sockets: 2,
            names: 0,
            refusals: 1,
            intake: read,
            receive: read,
            output: read,
            sends: 1,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(5),
            retry: Duration::from_millis(1),
        };
        let mut io = io::Io::new(&limits);
        let signals = io.adopt_signals(signals)?;
        let operations = shell
            .operations()
            .checked_add(Terminal::operations(&terminal_settings).expect("checked terminal operations"))
            .and_then(|n| n.checked_add(io::operations(&limits).expect("signal operations")))
            .expect("combined operations");
        let capacity = operations.checked_mul(2).expect("completion and submission room");
        Ok(Self {
            host,
            shell,
            terminal,
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            signals: Some(signals),
            root,
            opener: settings.opener,
            opens: Queue::uncapped(),
            opening: false,
            child: None,
            opened: None,
            complete: Queue::with_capacity(capacity),
            submit: Queue::with_capacity(capacity),
            raw: Queue::with_capacity(ROOM),
            told: Queue::uncapped(),
            operations,
            start: Some((settings.scheme, settings.address)),
            size: None,
            signed_in: None,
            closing: false,
            refused: false,
        })
    }

    /// The address the host gave the binary to keep.
    #[must_use]
    pub fn signed_in(&self) -> Option<&[u8]> {
        self.signed_in.as_deref()
    }

    /// The native shell's files refused the directory.
    #[must_use]
    pub const fn refused(&self) -> bool {
        self.refused
    }

    fn complete(&mut self, mut complete: Complete) {
        if Terminal::owns(complete.op) {
            self.terminal.completions().push(complete);
        } else if complete.op.raw() & TAG == 0 {
            self.shell.completions().push(complete);
        } else {
            complete.op = Token::new(complete.op.raw() & !TAG);
            if let Op::Cancel { target } = &mut complete.kind {
                *target = Token::new(target.raw() & !TAG);
            }
            io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.raw);
        }
    }

    fn event(&mut self, event: &io::Event) {
        match event {
            io::Event::Shutdown { .. } => self.close(),
            io::Event::Failed { owner, .. } => {
                if *owner == OPENER {
                    self.opened = Some(false);
                    if let Some(entity) = self.child {
                        self.down(io::Request::Abort { entity });
                    }
                } else {
                    self.close();
                }
            }
            io::Event::Spawned { owner, child, pipes } => {
                assert_eq!(*owner, OPENER);
                assert!(pipes.is_empty(), "the opener has null descriptors");
                self.child = Some(*child);
                if self.closing || !self.opens.is_empty() {
                    self.down(io::Request::Abort { entity: *child });
                }
            }
            io::Event::Exited { owner, exit } => {
                assert_eq!(*owner, OPENER);
                self.opened = Some(*exit == Exit::Code(0));
                self.down(io::Request::Abort { entity: self.child.expect("spawned opener") });
            }
            io::Event::Closed { owner } => {
                if self.signals == Some(*owner) {
                    self.signals = None;
                } else if self.child == Some(*owner) || *owner == OPENER {
                    self.child = None;
                    self.opening = false;
                }
            }
            io::Event::Resized => self.terminal.resized(),
            io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. } => unreachable!("the run's io owns signals and an opener"),
        }
    }

    fn close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.terminal.close();
        self.shell.close();
        self.opens.clear();
        if let Some(entity) = self.child {
            self.down(io::Request::Abort { entity });
        }
        if let Some(entity) = self.signals.take() {
            self.down(io::Request::Abort { entity });
        }
    }

    fn report(&mut self, report: host::Report) {
        let duties = self.host.report(report);
        for request in duties.asked {
            self.shell.down(request);
        }
        if let Some(credential) = duties.keep {
            self.shell.keep(credential);
        }
        if let Some(address) = duties.address {
            self.signed_in = Some(address);
        }
        if let Some(address) = duties.open.filter(|_| !self.closing) {
            if self.opener.is_some() {
                self.opens.push(address);
                if let Some(entity) = self.child {
                    self.down(io::Request::Abort { entity });
                }
            } else {
                self.report(host::Report::Opened { opened: false });
            }
        }
        if duties.close {
            self.close();
        }
    }

    fn down(&mut self, request: io::Request) {
        assert!(self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions, "run io request room");
        io::down(&mut self.io, &self.env, request, &mut self.raw);
    }

    fn open(&mut self) {
        if self.closing || self.opening || !self.io.takes() || self.raw.room() < io::MAX_OUT_DOWN.submissions {
            return;
        }
        if let Some(address) = self.opens.pop() {
            let spawn = opener::spawn(self.opener.as_ref().expect("configured opener"), &address, self.root);
            self.opening = true;
            self.down(io::Request::SpawnAlone { owner: OPENER, spawn });
        }
    }

    fn reports(&mut self) {
        self.host.clocks(self.env.now, self.env.wall);
        let mut reported = false;
        if let Some((credential, kept)) = self.shell.loaded() {
            let (scheme, address) = self.start.take().expect("the shell loads once");
            let size = self.terminal.size();
            self.size = Some(size);
            self.report(host::Report::Start(host::Start { size, scheme, address, kept, credential }));
            reported = true;
        }
        if self.size.is_some() {
            if let Some(opened) = self.opened.take() {
                self.report(host::Report::Opened { opened });
                reported = true;
            }
            if let Some(written) = self.shell.kept() {
                self.report(host::Report::Kept { written });
                reported = true;
            }
            while let Some(event) = self.shell.events().pop() {
                self.report(host::Report::Heard(event));
                reported = true;
            }
            if !self.closing && self.size != Some(self.terminal.size()) {
                let size = self.terminal.size();
                self.size = Some(size);
                self.report(host::Report::Resize(size));
                reported = true;
            }
            while !self.closing {
                let Some(key) = self.terminal.keys().pop() else { break };
                self.report(host::Report::Key(key));
                reported = true;
            }
            if !self.closing && self.host.deadline().is_some_and(|at| self.env.now >= at) {
                self.report(host::Report::Due);
                reported = true;
            }
        }
        if reported && !self.closing {
            self.terminal.draw(self.host.frame());
        }
        if !self.closing
            && self.size.is_none()
            && !self.shell.work_pending(self.env.now)
            && self.shell.next_deadline().is_none()
            && self.shell.submissions().is_empty()
            && self.shell.completions().is_empty()
        {
            self.refused = true;
            self.close();
        }
    }

    fn publish(&mut self) {
        while self.submit.room() > 0 {
            let Some(mut submit) = self.raw.pop() else { break };
            assert!(submit.op.raw() & TAG == 0, "signal token namespace");
            submit.op = Token::new(submit.op.raw() | TAG);
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::new(target.raw() | TAG);
            }
            self.submit.push(submit);
        }
        while self.submit.room() > 0 {
            let Some(submit) = self.terminal.submissions().pop() else { break };
            self.submit.push(submit);
        }
        while self.submit.room() > 0 {
            let Some(submit) = self.shell.submissions().pop() else { break };
            self.submit.push(submit);
        }
    }
}

impl Host for Run {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        while self.io.is_ready() && self.raw.room() >= io::MAX_OUT_RESUME.submissions {
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.raw);
        }
        while !self.io.is_ready() && self.raw.room() >= io::MAX_OUT_UP.submissions {
            let Some(complete) = self.complete.pop() else { break };
            self.complete(complete);
        }
        while self.io.is_due(now) && self.raw.room() >= io::MAX_OUT_FIRE.submissions {
            io::fire(&mut self.io, &self.env, &mut self.told, &mut self.raw);
        }
        while let Some(event) = self.told.pop() {
            self.event(&event);
        }
        self.shell.iterate(now, wall);
        self.terminal.iterate(now);
        if self.terminal.ended() {
            self.close();
        }
        self.reports();
        self.io.reclaim();
        self.open();
        self.publish();
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.complete.is_empty()
            || !self.raw.is_empty()
            || !self.submit.is_empty()
            || !self.told.is_empty()
            || (!self.opens.is_empty() && !self.opening)
            || self.io.is_ready()
            || self.io.is_due(now)
            || self.shell.work_pending(now)
            || self.terminal.work_pending(now)
            || self.next_policy_deadline().is_some_and(|at| now >= at)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        [self.shell.next_deadline(), self.terminal.next_deadline(), self.io.next_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [
            self.shell.next_policy_deadline(),
            if self.closing || self.size.is_none() { None } else { self.host.deadline() },
        ]
        .into_iter()
        .flatten()
        .min()
    }
    fn is_empty(&self) -> bool {
        self.closing
            && self.terminal.closed()
            && self.shell.is_empty()
            && self.io.is_empty()
            && self.complete.is_empty()
            && self.submit.is_empty()
            && self.raw.is_empty()
            && self.told.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(u8::from(self.refused)))
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}
