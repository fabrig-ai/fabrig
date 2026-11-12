//! The TUI on one kernel and clock (temper/docs/design/client/tui/README.md, 4).
#[cfg(any(test, feature = "record"))]
use crate::journey;
use crate::{
    host, limits,
    opener::{self, Opener},
};
use skein_io::{
    self as io,
    kernel::{Complete, Exit, Fd, Op, Submit},
};
use skein_lib::{Duration, Env, Queue, Secret, Time, Token, Wall};
use skein_shell::Host;
use skein_ui_screen::Size;
#[cfg(any(test, feature = "record"))]
use skein_ui_screen_terminal::Key;
use skein_ui_screen_terminal::{Background, Depth, Terminal};
use temper_client_tui::{BoxLine, Note, Scheme};

mod front;

const TAG: u64 = 1 << 61;
const ROOM: u32 = 64;
const OPENER: Token = Token::new(u64::MAX);

pub struct Settings {
    pub engine: io::Addr,
    pub directory: Box<[u8]>,
    pub address: Box<[u8]>,
    pub scheme: Option<Scheme>,
    pub boxed: bool,
    pub link_file: Option<Box<[u8]>>,
    pub note: Option<Note>,
    pub files: Option<Fd>,
    pub depth: Depth,
    pub size: Size,
    pub limits: limits::Limits,
    pub opener: Option<Opener>,
    pub record: Option<Fd>,
    /// How to fetch and start the box.
    pub front: Option<Front>,
}

/// The captured box start (temper/docs/design/client/tui/box.md).
pub struct Front {
    pub executable: Box<[u8]>,
    pub cache: Box<[u8]>,
    pub arguments: Box<[Box<[u8]>]>,
    pub environment: Box<[Box<[u8]>]>,
    /// An owned descriptor of the captured working directory.
    pub directory: Fd,
    /// DATA/box/logs/box.log, for the exit's report.
    pub log: Box<[u8]>,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "box placement, opening, closing and refusal are independent run facts"
)]
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
    start: Option<(Option<Scheme>, Box<[u8]>)>,
    loaded: Option<(Option<Secret>, Vec<jig_client::Record>)>,
    boxed: bool,
    front: bool,
    box_child: Option<front::Child>,
    spawn_root: Option<LocalFile>,
    front_ready: bool,
    local_wait: bool,
    held: Queue<jig_client::Request>,
    link_file: Option<Box<[u8]>>,
    local: LocalFile,
    note: Option<Note>,
    notes: Option<FileSave>,
    address: Box<[u8]>,
    size: Option<Size>,
    signed_in: Option<Box<[u8]>>,
    closing: bool,
    refused: bool,
    terminated: bool,
    #[cfg(any(test, feature = "record"))]
    recording: Option<Recording>,
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
        let terminal_settings = limits::terminal(&settings.limits, settings.depth, settings.scheme.is_some());
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
            sockets: 12,
            names: 0,
            refusals: 1,
            intake: read.max(u32::try_from(BoxLine::MOST).expect("box line bound")),
            receive: read.max(u32::try_from(BoxLine::MOST).expect("box line bound")),
            output: read,
            sends: 1,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(5),
            retry: Duration::from_millis(1),
        };
        let mut io = io::Io::new(&limits);
        let signals = io.adopt_signals(signals)?;
        let front = settings.front.is_some();
        let spawn_root = settings.front.as_ref().map(|front| {
            let mut root = LocalFile::new(front.directory, user);
            root.owned = Some(root.io.adopt_root(front.directory).expect("spawn directory room"));
            root
        });
        let box_child = settings.front.map(front::Child::new);
        #[cfg(any(test, feature = "record"))]
        let recording = settings.record.map(|root| Recording::new(root, user, seed, settings.files == Some(root)));
        #[cfg(any(test, feature = "record"))]
        let shared_notes = recording.as_ref().is_some_and(|recording| recording.notes);
        #[cfg(not(any(test, feature = "record")))]
        let shared_notes = false;
        let notes = settings.files.filter(|_| !shared_notes).map(|root| FileSave::new(root, user, seed, b"sign-in"));
        let operations = shell
            .operations()
            .checked_add(Terminal::operations(&terminal_settings).expect("checked terminal operations"))
            .and_then(|n| n.checked_add(io::operations(&limits).expect("signal operations")))
            .and_then(|n| n.checked_add(2 + if notes.is_some() { 2 } else { 0 } + if front { 2 } else { 0 }))
            .expect("combined operations");
        #[cfg(any(test, feature = "record"))]
        let operations =
            operations.checked_add(if recording.is_some() { 2 } else { 0 }).expect("file and cancel operations");
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
            start: Some((settings.scheme, settings.address.clone())),
            loaded: None,
            boxed: settings.boxed,
            front,
            box_child,
            spawn_root,
            front_ready: !front,
            local_wait: false,
            held: Queue::uncapped(),
            link_file: settings.link_file,
            local: LocalFile::new(root, user),
            note: settings.note,
            notes,
            address: settings.address,
            size: None,
            signed_in: None,
            closing: false,
            refused: false,
            terminated: false,
            #[cfg(any(test, feature = "record"))]
            recording,
        })
    }

    /// The address the host gave the binary to keep.
    #[must_use]
    pub fn signed_in(&self) -> Option<&[u8]> {
        self.signed_in.as_deref()
    }

    /// Read the filename of `path` beneath its parent descriptor `root`, before
    /// the first iteration. The run owns `root` and closes it on shutdown.
    #[must_use]
    pub fn with_local_link(mut self, root: Fd, path: &[u8]) -> Self {
        assert!(self.start.is_some(), "local link supplied before starting");
        assert!(self.local.owned.is_none(), "one local link root");
        self = self.with_local_path(root, path.rsplit(|byte| *byte == b'/').next().expect("link filename"));
        self
    }

    pub(crate) fn with_local_path(mut self, root: Fd, path: &[u8]) -> Self {
        self.local.owned = Some(self.local.io.adopt_root(root).expect("local link root room"));
        self.local.root = root;
        self.link_file = Some(path.into());
        self
    }

    /// The native shell's files refused the directory.
    #[must_use]
    pub const fn refused(&self) -> bool {
        self.refused
    }

    fn complete(&mut self, mut complete: Complete) {
        let lane = complete.op.tag();
        if lane == 3 || lane == 4 || lane == 5 {
            complete.op = Token::new(complete.op.untagged());
            if let Op::Cancel { target } = &mut complete.kind {
                *target = Token::new(target.untagged());
            }
            if lane == 3 {
                self.notes.as_mut().expect("note completion").complete(self.env.now, complete);
            } else if lane == 4 {
                self.local.complete(self.env.now, complete);
            } else {
                self.spawn_root.as_mut().expect("spawn root completion").complete(self.env.now, complete);
            }
            return;
        }
        if Terminal::owns(complete.op) {
            self.terminal.completions().push(complete);
        } else if complete.op.raw() & TAG == 0 {
            self.shell.completions().push(complete);
        } else {
            complete.op = Token::new(complete.op.raw() & !TAG);
            if let Op::Cancel { target } = &mut complete.kind {
                *target = Token::new(target.raw() & !TAG);
            }
            #[cfg(any(test, feature = "record"))]
            if io::file_layer::owns(complete.op) {
                self.recording.as_mut().expect("recording file completion").save.complete(self.env.now, complete);
                return;
            }
            io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.raw);
        }
    }

    fn event(&mut self, event: &io::Event) {
        if self.box_child.as_mut().is_some_and(|front| front.event(event)) {
            return;
        }
        match event {
            io::Event::Shutdown { .. } => {
                self.terminated = true;
                self.close();
            }
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
            | io::Event::Window { .. } => unreachable!("the run's io owns signals, the box's lines and an opener"),
        }
    }

    fn front_progress(&mut self) {
        if let Some(front) = &mut self.box_child {
            front.progress();
        }
        while self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions {
            let Some(request) = self.box_child.as_mut().and_then(|front| front.requests.pop()) else { break };
            self.down(request);
        }
    }

    /// A termination signal leaves after the TUI's own shutdown, without waiting on the box.
    #[must_use]
    pub fn terminated(&self) -> bool {
        self.terminated && self.terminal.closed()
    }

    /// The box's exit words, printed only after leaving the terminal.
    #[must_use]
    pub fn box_exit(&self) -> Option<(u8, Box<[u8]>)> {
        self.box_child.as_ref().map(front::Child::exit)
    }

    fn close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.local.closing = true;
        if let Some(root) = &mut self.spawn_root {
            root.closing = true;
        }
        #[cfg(any(test, feature = "record"))]
        let saving = self.recording.is_some();
        #[cfg(not(any(test, feature = "record")))]
        let saving = false;
        if !saving && self.notes.is_none() && self.local.closed() {
            self.terminal.close();
        }
        self.shell.close();
        self.held.clear();
        if let Some(front) = &mut self.box_child {
            if self.terminated {
                front.leave();
            } else {
                front.close();
            }
        }
        self.opens.clear();
        if let Some(entity) = self.child {
            self.down(io::Request::Abort { entity });
        }
        if let Some(entity) = self.signals.take() {
            self.down(io::Request::Abort { entity });
        }
    }

    fn report(&mut self, report: host::Report) {
        #[cfg(any(test, feature = "record"))]
        let duties = if let Some(recorder) = self.recording.as_mut().and_then(|recording| recording.recorder.as_mut()) {
            recorder.report(&mut self.host, report)
        } else {
            self.host.report(report)
        };
        #[cfg(not(any(test, feature = "record")))]
        let duties = self.host.report(report);
        for request in duties.asked {
            if self.front_ready {
                self.shell.down(request);
            } else {
                self.held.push(request);
            }
        }
        if let Some(credential) = duties.keep {
            self.shell.keep(credential);
        }
        if let Some(note) = duties.note
            && let Some(notes) = &mut self.notes
        {
            notes.noted = Some(note.print(&self.address));
        }
        #[cfg(any(test, feature = "record"))]
        if let Some(note) = duties.note
            && let Some(recording) = self.recording.as_mut().filter(|recording| recording.notes)
        {
            recording.save.noted = Some(note.print(&self.address));
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
        if let Some(stop) = duties.stop
            && let Some(front) = &mut self.box_child
        {
            front.stop(stop);
        }
        if duties.close {
            self.close();
        }
        if duties.local {
            if self.front && !self.front_ready {
                self.local_wait = true;
            } else if let Some(path) = &self.link_file {
                self.local.pending = Some(path.clone());
            } else {
                self.report(host::Report::Local { link: None });
            }
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

    fn started(&mut self, credential: Option<Secret>, kept: Vec<jig_client::Record>) {
        let (scheme, address) = self.start.take().expect("the shell loads once");
        let background = match self.terminal.background() {
            Background::Answered(rgb) => Some(rgb),
            Background::Waiting | Background::Unanswered => None,
        };
        let scheme = scheme.or_else(|| background.map(Scheme::of_background));
        let size = self.terminal.size();
        self.size = Some(size);
        let start = host::Start {
            size,
            scheme,
            background,
            boxed: self.boxed,
            address,
            local: self.front || self.link_file.is_some(),
            kept,
            credential,
            note: self.note,
            front: self.front,
        };
        #[cfg(any(test, feature = "record"))]
        if let Some(recording) = &mut self.recording {
            let mut recorder = journey::Recorder::new(&journey::Facts {
                seed: recording.seed,
                now: Time::ZERO,
                wall: Wall::EPOCH,
                size: start.size,
                scheme: start.scheme,
                background: start.background,
                boxed: start.boxed,
                address: start.address.clone(),
                local: start.local,
                front: start.front,
                note: start.note,
                kept: start.kept.clone(),
                credential: start.credential.clone(),
            });
            recorder.turn(self.env.now, self.env.wall);
            recording.recorder = Some(recorder);
        }
        self.report(host::Report::Start(start));
    }

    fn reports(&mut self) {
        self.host.clocks(self.env.now, self.env.wall);
        #[cfg(any(test, feature = "record"))]
        if let Some(recorder) = self.recording.as_mut().and_then(|recording| recording.recorder.as_mut()) {
            recorder.turn(self.env.now, self.env.wall);
        }
        let mut reported = false;
        if let Some(loaded) = self.shell.loaded() {
            self.loaded = Some(loaded);
        }
        if self.terminal.background() != Background::Waiting
            && let Some((credential, kept)) = self.loaded.take()
        {
            self.started(credential, kept);
            reported = true;
        }
        if self.size.is_some() {
            if let Some(link) = self.local.result.take() {
                self.report(host::Report::Local { link });
                reported = true;
            }
            if let Some(opened) = self.opened.take() {
                self.report(host::Report::Opened { opened });
                reported = true;
            }
            if let Some(written) = self.shell.kept() {
                self.report(host::Report::Kept { written });
                reported = true;
            }
            while let Some(report) = self.box_child.as_mut().and_then(|front| front.reports.pop()) {
                match report {
                    front::Report::Stage(stage) => self.report(host::Report::Stage(stage)),
                    front::Report::Line(line) => self.report(host::Report::Front(line)),
                    front::Report::Link { engine, address, link } if !self.closing => {
                        self.shell.point(engine);
                        self.address = address;
                        self.front_ready = true;
                        while let Some(request) = self.held.pop() {
                            self.shell.down(request);
                        }
                        if self.local_wait {
                            self.local_wait = false;
                            self.report(host::Report::Local { link: Some(link) });
                        }
                    }
                    front::Report::Link { .. } => {}
                }
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
                #[cfg(any(test, feature = "record"))]
                if self.record_key(&key) {
                    continue;
                }
                self.report(host::Report::Key(key));
                reported = true;
            }
            if !self.closing && self.host.deadline().is_some_and(|at| self.env.now >= at) {
                self.report(host::Report::Due);
                reported = true;
            }
        }
        #[cfg(any(test, feature = "record"))]
        if let Some(recorder) = self.recording.as_mut().and_then(|recording| recording.recorder.as_mut()) {
            recorder.deadline(self.host.deadline());
        }
        if reported && !self.closing {
            self.terminal.draw(self.host.frame());
        }
        if !self.closing
            && self.size.is_none()
            && self.loaded.is_none()
            && !self.shell.work_pending(self.env.now)
            && self.shell.next_deadline().is_none()
            && self.shell.submissions().is_empty()
            && self.shell.completions().is_empty()
        {
            self.refused = true;
            self.close();
        }
    }

    #[cfg(any(test, feature = "record"))]
    fn record_key(&mut self, key: &Key) -> bool {
        if let Some(recording) = &mut self.recording
            && let Some(recorder) = &mut recording.recorder
        {
            match key {
                Key::Alt('M' | 'm') => {
                    recorder.deadline(self.host.deadline());
                    recorder.turn(self.env.now, self.env.wall);
                    recorder.mark(&mut self.host);
                    recorder.deadline(self.host.deadline());
                    recorder.turn(self.env.now, self.env.wall);
                    return true;
                }
                Key::Alt('S' | 's') => {
                    recorder.deadline(self.host.deadline());
                    recording.save.pending = Some(recorder.save(&mut self.host));
                    recorder.turn(self.env.now, self.env.wall);
                    return true;
                }
                Key::Alt(_)
                | Key::Char(_)
                | Key::Control(_)
                | Key::Paste(_)
                | Key::Enter
                | Key::Tab
                | Key::BackTab
                | Key::Backspace
                | Key::Delete
                | Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown
                | Key::Escape => {}
            }
        }
        false
    }

    fn publish(&mut self) {
        if let Some(notes) = &mut self.notes {
            publish_files(&mut notes.submissions, &mut self.submit, 3);
        }
        publish_files(&mut self.local.submissions, &mut self.submit, 4);
        if let Some(root) = &mut self.spawn_root {
            publish_files(&mut root.submissions, &mut self.submit, 5);
        }
        #[cfg(any(test, feature = "record"))]
        if let Some(recording) = &mut self.recording {
            publish_files(&mut recording.save.submissions, &mut self.submit, 2);
        }
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
        #[cfg(any(test, feature = "record"))]
        if let Some(recording) = &mut self.recording {
            if self.closing && self.shell.is_empty() && !recording.closing {
                recording.save.pending = recording.recorder.as_ref().map(|recorder| recorder.save(&mut self.host));
                recording.closing = true;
                recording.save.closing = true;
            }
            recording.save.progress(now);
        }
        self.local.progress(now);
        if let Some(root) = &mut self.spawn_root {
            root.progress(now);
        }
        if let Some(notes) = &mut self.notes {
            notes.closing = self.closing && self.shell.is_empty();
            notes.progress(now);
        }
        #[cfg(any(test, feature = "record"))]
        let recorded = self.recording.as_ref().is_none_or(|recording| recording.save.closed());
        #[cfg(not(any(test, feature = "record")))]
        let recorded = true;
        if self.closing && recorded && self.notes.as_ref().is_none_or(FileSave::closed) && self.local.closed() {
            self.terminal.close();
        }
        self.io.reclaim();
        self.open();
        self.front_progress();
        self.publish();
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, now: Time) -> bool {
        #[cfg(any(test, feature = "record"))]
        if self.recording.as_ref().is_some_and(|recording| recording.save.pending() || recording.save.io.is_due(now)) {
            return true;
        }
        self.box_child
            .as_ref()
            .is_some_and(|front| front.pending() || (!front.reports.is_empty() && self.size.is_some()))
            || self.spawn_root.as_ref().is_some_and(|root| root.pending() || root.io.is_due(now))
            || self.local.pending()
            || self.local.io.is_due(now)
            || self.notes.as_ref().is_some_and(|notes| notes.pending() || notes.io.is_due(now))
            || !self.complete.is_empty()
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
        #[cfg(any(test, feature = "record"))]
        let recording = self.recording.as_ref().and_then(|recording| recording.save.io.next_deadline());
        #[cfg(not(any(test, feature = "record")))]
        let recording = None;
        [
            self.shell.next_deadline(),
            self.terminal.next_deadline(),
            self.io.next_deadline(),
            recording,
            self.local.io.next_deadline(),
            self.spawn_root.as_ref().and_then(|root| root.io.next_deadline()),
            self.notes.as_ref().and_then(|notes| notes.io.next_deadline()),
        ]
        .into_iter()
        .flatten()
        .min()
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
        #[cfg(any(test, feature = "record"))]
        if self.recording.as_ref().is_some_and(|recording| !recording.save.closed()) {
            return false;
        }
        self.closing
            && self.local.closed()
            && self.spawn_root.as_ref().is_none_or(LocalFile::closed)
            && self.notes.as_ref().is_none_or(FileSave::closed)
            && self.terminal.closed()
            && self.shell.is_empty()
            && self.io.is_empty()
            && self.complete.is_empty()
            && self.submit.is_empty()
            && self.raw.is_empty()
            && self.told.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then(|| {
            Exit::Code(if self.refused || self.terminated { 1 } else { self.box_exit().map_or(0, |(code, _)| code) })
        })
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}

#[cfg(any(test, feature = "record"))]
struct Recording {
    notes: bool,
    save: FileSave,
    recorder: Option<journey::Recorder>,
    seed: u64,
    closing: bool,
}

#[cfg(any(test, feature = "record"))]
impl Recording {
    fn new(root: Fd, user: u32, seed: u64, notes: bool) -> Self {
        Self { notes, save: FileSave::new(root, user, seed, b"recording"), recorder: None, seed, closing: false }
    }
}

// Each private writer owns its file lane (skein/docs/design/runtime/io/files.md).
#[derive(Clone, Copy, PartialEq, Eq)]
enum SaveStage {
    Ready,
    Opening,
    Writing,
    ClosingPrivate,
    ClosingRoot,
    Closed,
}

/// A private replace, serialized with later saves and drained before LEAVE.
/// temper/docs/design/client/tui/journeys.md, 3.1; skein/docs/design/runtime/io/files.md, 3.
struct FileSave {
    io: io::file_layer::FileIo,
    env: Env<io::file_layer::Limits>,
    root: Option<Token>,
    private: Option<Token>,
    stage: SaveStage,
    events: Queue<io::file::Event>,
    pending: Option<Box<[u8]>>,
    active: Option<(&'static [u8], Box<[u8]>)>,
    noted: Option<Box<[u8]>>,
    path: &'static [u8],
    submissions: Queue<Submit>,
    closing: bool,
}

impl FileSave {
    fn new(root: Fd, user: u32, seed: u64, path: &'static [u8]) -> Self {
        // Four descriptors cover the parent, private directory, target and temporary.
        let limits =
            io::file_layer::Limits { files: 4, read: 1, entries: 1, whole: u32::MAX, timeout: Duration::from_secs(30) };
        let mut io = io::file_layer::FileIo::with_limits(&limits).with_effective_user(user);
        io.seed_randomness(seed);
        let root = io.adopt_root(root).expect("private save root room");
        Self {
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            root: Some(root),
            private: None,
            stage: SaveStage::Ready,
            events: Queue::uncapped(),
            pending: None,
            active: None,
            noted: None,
            path,
            submissions: Queue::with_capacity(4),
            closing: false,
        }
    }

    fn complete(&mut self, now: Time, complete: Complete) {
        self.env.now = now;
        io::file_layer::step::up(&mut self.io, &self.env, complete, &mut self.events, &mut self.submissions);
    }

    fn request(&mut self, stage: SaveStage, request: io::file::Request) {
        self.stage = stage;
        io::file_layer::step::down(&mut self.io, &self.env, request, &mut self.events, &mut self.submissions);
    }

    fn progress(&mut self, now: Time) {
        use io::file::{Event, Expect, Request};
        self.env.now = now;
        if self.io.is_due(now) {
            io::file_layer::step::expire(&mut self.io, &self.env, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            match self.stage {
                SaveStage::Opening => {
                    if let Event::Opened { file, .. } = event {
                        self.private = Some(file);
                    } else {
                        self.active = None;
                    }
                    self.stage = SaveStage::Ready;
                }
                SaveStage::Writing => self.stage = SaveStage::Ready,
                SaveStage::ClosingPrivate => {
                    assert!(matches!(event, Event::Closed { .. }), "private save directory closed");
                    self.private = None;
                    self.stage = SaveStage::Ready;
                }
                SaveStage::ClosingRoot => {
                    assert!(matches!(event, Event::Closed { .. }), "private save parent closed");
                    self.root = None;
                    self.stage = SaveStage::Closed;
                }
                SaveStage::Ready | SaveStage::Closed => unreachable!("private save request in flight"),
            }
        }
        if self.stage != SaveStage::Ready || !self.io.takes() || self.submissions.room() < 2 {
            return;
        }
        let owner = Token::new(1);
        if self.active.is_some() || self.pending.is_some() || self.noted.is_some() {
            if let Some(root) = self.private {
                let (path, bytes) = self.active.take().or_else(|| self.next()).expect("queued save");
                self.request(
                    SaveStage::Writing,
                    Request::Store { owner, root, path: path.into(), bytes, expected: Expect::Any, no_follow: true },
                );
            } else {
                self.active = self.next();
                self.request(
                    SaveStage::Opening,
                    Request::OpenPrivate {
                        owner,
                        root: self.root.expect("private save parent"),
                        path: b"tui".as_slice().into(),
                    },
                );
            }
        } else if self.closing {
            if let Some(file) = self.private {
                self.request(SaveStage::ClosingPrivate, Request::Close { owner, file });
            } else {
                self.request(
                    SaveStage::ClosingRoot,
                    Request::Close { owner, file: self.root.expect("private save parent") },
                );
            }
        }
    }

    fn next(&mut self) -> Option<(&'static [u8], Box<[u8]>)> {
        if let Some(bytes) = self.noted.take() {
            Some((b"sign-in", bytes))
        } else {
            self.pending.take().map(|bytes| (self.path, bytes))
        }
    }

    fn pending(&self) -> bool {
        !self.submissions.is_empty()
            || !self.events.is_empty()
            || (self.stage == SaveStage::Ready
                && (self.active.is_some() || self.pending.is_some() || self.noted.is_some() || self.closing))
    }

    fn closed(&self) -> bool {
        self.submissions.is_empty() && self.stage == SaveStage::Closed && self.io.open_files() == 0 && self.io.takes()
    }
}

// The namespace includes cancellation targets as well as file operations.
fn publish_files(source: &mut Queue<Submit>, target: &mut Queue<Submit>, lane: u8) {
    while target.room() > 0 {
        let Some(mut submit) = source.pop() else { break };
        submit.op = Token::tagged(lane, submit.op.raw()).expect("file namespace");
        if let Op::Cancel { target } = &mut submit.kind {
            *target = Token::tagged(lane, target.raw()).expect("file cancel namespace");
        }
        target.push(submit);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LocalStage {
    Ready,
    Opening,
    Stating,
    Reading,
    Closing,
    ClosingRoot,
}

/// A bounded private read on the loop, with the checks of skein-shell's
/// `read_private_file` (temper/docs/design/client/tui/README.md, 2).
struct LocalFile {
    io: io::file_layer::FileIo,
    env: Env<io::file_layer::Limits>,
    root: Fd,
    owned: Option<Token>,
    closing: bool,
    user: u32,
    stage: LocalStage,
    file: Option<Token>,
    events: Queue<io::file::Event>,
    submissions: Queue<Submit>,
    pending: Option<Box<[u8]>>,
    link: Option<Secret>,
    #[expect(clippy::option_option, reason = "a completed read can find no private file")]
    result: Option<Option<Secret>>,
}

impl LocalFile {
    fn new(root: Fd, user: u32) -> Self {
        let limits =
            io::file_layer::Limits { files: 2, read: 4097, entries: 1, whole: 4097, timeout: Duration::from_secs(30) };
        Self {
            io: io::file_layer::FileIo::with_limits(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            root,
            owned: None,
            closing: false,
            user,
            stage: LocalStage::Ready,
            file: None,
            events: Queue::uncapped(),
            submissions: Queue::with_capacity(4),
            pending: None,
            link: None,
            result: None,
        }
    }

    fn complete(&mut self, now: Time, complete: Complete) {
        self.env.now = now;
        io::file_layer::step::up(&mut self.io, &self.env, complete, &mut self.events, &mut self.submissions);
    }

    fn request(&mut self, stage: LocalStage, request: io::file::Request) {
        self.stage = stage;
        io::file_layer::step::down(&mut self.io, &self.env, request, &mut self.events, &mut self.submissions);
    }

    fn finish(&mut self) {
        if let Some(file) = self.file {
            self.request(LocalStage::Closing, io::file::Request::Close { owner: Token::new(1), file });
        } else {
            self.stage = LocalStage::Ready;
            self.result = Some(self.link.take());
        }
    }

    fn event(&mut self, event: io::file::Event) {
        use io::file::{Event, Request};
        match self.stage {
            LocalStage::Opening => {
                if let Event::Opened { file, .. } = event {
                    self.file = Some(file);
                    self.request(LocalStage::Stating, Request::Stat { owner: Token::new(1), file });
                } else {
                    self.finish();
                }
            }
            LocalStage::Stating => {
                if let Event::Stated { stat, .. } = event
                    && self.private(&stat)
                {
                    self.request(
                        LocalStage::Reading,
                        Request::ReadAt {
                            owner: Token::new(1),
                            file: self.file.expect("opened link"),
                            offset: 0,
                            max: 4097,
                        },
                    );
                } else {
                    self.finish();
                }
            }
            LocalStage::Reading => {
                if let Event::Read { bytes, .. } = event
                    && bytes.len() <= 4096
                {
                    self.link = Some(Secret::new(bytes.strip_suffix(b"\n").unwrap_or(&bytes).into()));
                }
                self.finish();
            }
            LocalStage::Closing => {
                self.file = None;
                self.stage = LocalStage::Ready;
                self.result = Some(self.link.take());
            }
            LocalStage::ClosingRoot => {
                self.owned = None;
                self.stage = LocalStage::Ready;
            }
            LocalStage::Ready => unreachable!("local request in flight"),
        }
    }

    #[expect(clippy::verbose_bit_mask, reason = "the private-file rule names the group and other permission bits")]
    fn private(&self, stat: &io::kernel::Stat) -> bool {
        stat.kind == io::kernel::Kind::File
            && stat.owner == self.user
            && stat.mode & 0o077 == 0
            && stat.links == 1
            && stat.size <= 4096
    }

    fn progress(&mut self, now: Time) {
        use io::file::Request;
        self.env.now = now;
        if self.io.is_due(now) {
            io::file_layer::step::expire(&mut self.io, &self.env, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            self.event(event);
        }
        if self.stage != LocalStage::Ready || !self.io.takes() || self.submissions.room() < 2 {
            return;
        }
        if let Some(name) = self.pending.take() {
            self.request(
                LocalStage::Opening,
                Request::OpenReadNoFollow { owner: Token::new(1), root: self.root, name },
            );
        } else if self.closing
            && let Some(file) = self.owned
        {
            self.request(LocalStage::ClosingRoot, Request::Close { owner: Token::new(1), file });
        }
    }

    fn pending(&self) -> bool {
        !self.events.is_empty()
            || !self.submissions.is_empty()
            || self.pending.is_some()
            || self.result.is_some()
            || (self.closing && self.owned.is_some() && self.stage == LocalStage::Ready)
    }

    fn closed(&self) -> bool {
        self.stage == LocalStage::Ready
            && self.owned.is_none()
            && self.io.open_files() == 0
            && self.io.takes()
            && self.pending.is_none()
            && self.submissions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings {
            engine: io::Addr::Ip("127.0.0.1:8367".parse().expect("loopback")),
            directory: b"tui".as_slice().into(),
            address: b"127.0.0.1:8367".as_slice().into(),
            scheme: Some(Scheme::Dark),
            boxed: false,
            link_file: None,
            note: None,
            files: None,
            depth: Depth::Indexed,
            size: Size { rows: 24, columns: 80 },
            limits: limits::tui(),
            opener: None,
            record: None,
            front: None,
        }
    }

    #[test]
    fn recording_keys_pass_to_the_host_without_a_root() {
        let mut run = Run::new(settings(), Fd::new(3), 1000, 7, Fd::new(0), Fd::new(1), Fd::new(4))
            .expect("adopt synthetic descriptors without kernel calls");
        for key in [Key::Alt('M'), Key::Alt('S'), Key::Alt('m'), Key::Alt('s')] {
            assert!(!run.record_key(&key), "no interception without a root: {key:?}");
        }
    }

    #[test]
    fn a_signal_drops_a_link_already_waiting_to_be_reported() {
        let mut settings = settings();
        settings.front = Some(Front {
            executable: b"/bin/temper".as_slice().into(),
            cache: b"/cache".as_slice().into(),
            arguments: Box::new([]),
            environment: Box::new([]),
            directory: Fd::new(5),
            log: b"/data/box/logs/box.log".as_slice().into(),
        });
        let mut run =
            Run::new(settings, Fd::new(3), 1000, 7, Fd::new(0), Fd::new(1), Fd::new(4)).expect("synthetic descriptors");
        run.started(None, Vec::new());
        run.box_child.as_mut().expect("front").reports.push(front::Report::Link {
            engine: io::Addr::Ip("[::1]:9400".parse().expect("handed engine")),
            address: b"[::1]:9400".as_slice().into(),
            link: Secret::new(
                b"http://[::1]:9400/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                    .as_slice()
                    .into(),
            ),
        });
        run.event(&io::Event::Shutdown { signal: io::kernel::ServiceSignal::Terminate, from: None });
        run.reports();
        assert!(run.closing);
        assert!(!run.front_ready, "no point or connection while closing");
        assert!(run.held.is_empty());
    }
}
