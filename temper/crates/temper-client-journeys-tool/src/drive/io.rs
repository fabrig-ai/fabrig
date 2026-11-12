//! Thin session io over the browser harness (client/drive.md, 5).
use super::{Action, Answer, Event, Exit, Machine, Options, Process};
use crate::EnvironmentEntry;
use skein_fake_person::{Face, Find, Found};
use skein_fake_person_browser::{
    BrowserFace,
    harness::session::{Pump, Report},
};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{
    Token,
    stream::{self, Down, Up},
};
use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};
mod process;
mod tui;
use process::Child;
type LogLines = Vec<Box<[u8]>>;
enum FaceJob {
    Opening,
    Address,
    Find(Find),
    Operation,
    Snapshot,
    Screenshot(PathBuf),
    Controls(skein_ui::Role),
    Description,
    Cancel,
}
/// An explicitly ended live session. Start returns at ready, line returns one
/// answer, and end stops and drains every child. No build or implicit scratch.
pub struct Session {
    machine: Machine,
    pump: Pump,
    face: BrowserFace<'static>,
    face_job: Option<FaceJob>,
    browser_started: bool,
    browser_closing: bool,
    children: Vec<Child>,
    tui: tui::Tui,
    serial: u64,
    actions: VecDeque<Action>,
    answers: VecDeque<Answer>,
    ready: bool,
    exit: Option<Exit>,
    scratch: PathBuf,
    box_log: File,
    kept: File,
    input: Option<Token>,
    input_bytes: Vec<u8>,
    input_lines: VecDeque<Option<Box<[u8]>>>,
}
impl Session {
    pub fn start(options: Options, environment: &[EnvironmentEntry]) -> Result<Self, Exit> {
        let plan = super::options::prepare(options, environment).map_err(Exit::own)?;
        let scratch = plan.options.scratch.clone();
        let box_log = OpenOptions::new()
            .append(true)
            .open(scratch.join("session/box.log"))
            .map_err(|error| Exit::own(format!("box.log: {error}")))?;
        let kept = OpenOptions::new()
            .append(true)
            .open(scratch.join("session/lines"))
            .map_err(|error| Exit::own(format!("session/lines: {error}")))?;
        let pump = Pump::new(plan.chromium.clone(), scratch.join("session/browser"), plan.environment.clone())
            .map_err(Exit::own)?;
        let (machine, actions) = Machine::start(plan, pump.now());
        let mut session = Self {
            machine,
            pump,
            face: BrowserFace::new(Token::new(100), ""),
            face_job: None,
            browser_started: false,
            browser_closing: false,
            children: vec![],
            tui: tui::Tui::new(scratch.join("session")),
            serial: 1024,
            actions: actions.into(),
            answers: VecDeque::new(),
            ready: false,
            exit: None,
            scratch,
            box_log,
            kept,
            input: None,
            input_bytes: vec![],
            input_lines: VecDeque::new(),
        };
        while !session.ready && session.exit.is_none() {
            session.tick();
        }
        if let Some(exit) = session.exit.take() {
            session.drain();
            Err(exit)
        } else {
            Ok(session)
        }
    }
    pub fn line(&mut self, source: &[u8]) -> Result<Answer, Exit> {
        self.event(Event::Line(source.into()));
        loop {
            self.apply();
            if let Some(answer) = self.answers.pop_front() {
                return Ok(answer);
            }
            if let Some(exit) = self.exit.clone() {
                self.drain();
                return Err(exit);
            }
            self.tick();
        }
    }
    #[must_use]
    pub fn end(mut self) -> Exit {
        self.event(Event::End);
        while self.exit.is_none() {
            self.tick();
        }
        self.drain();
        self.exit.expect("session ended")
    }
    /// Only the binary adopts stdin. An idle person never blocks child pipes.
    pub(crate) fn stdio(mut self) -> Exit {
        self.input = if let Ok(input) = self.pump.input(Fd::new(0)) {
            self.read_input(input);
            Some(input)
        } else {
            self.event(Event::OwnFailure("cannot adopt stdin".into()));
            None
        };
        println!("ready");
        self.flush_stdout();
        while self.exit.is_none() {
            if self.machine.ready()
                && let Some(line) = self.input_lines.pop_front()
            {
                self.event(line.map_or(Event::End, Event::Line));
            }
            self.tick();
            while let Some(answer) = self.answers.pop_front() {
                print!("{}", answer.print());
                self.flush_stdout();
            }
        }
        self.drain();
        self.exit.expect("stdio ended")
    }
    fn flush_stdout(&mut self) {
        if let Err(error) = std::io::stdout().flush() {
            self.event(Event::OwnFailure(format!("stdout: {error}")));
        }
    }
    fn event(&mut self, event: Event) {
        self.actions.extend(self.machine.step(event, self.pump.now()));
    }
    fn read_input(&mut self, stream: Token) {
        self.pump.request(io::Request::Stream {
            stream,
            down: Down::Demand { read: stream::Read::Arrived { max: 32768 }, room: 0 },
        });
    }
    fn input_event(&mut self, up: Up) {
        match up {
            Up::Bytes(bytes) => {
                self.input_bytes.extend_from_slice(&bytes);
                while let Some(end) = self.input_bytes.iter().position(|b| *b == b'\n') {
                    let mut line: Vec<_> = self.input_bytes.drain(..=end).collect();
                    line.pop();
                    self.input_lines.push_back(Some(line.into()));
                }
                if let Some(input) = self.input {
                    self.read_input(input);
                }
            }
            Up::End => {
                if !self.input_bytes.is_empty() {
                    self.input_lines.push_back(Some(std::mem::take(&mut self.input_bytes).into()));
                }
                self.input_lines.push_back(None);
                if let Some(entity) = self.input.take() {
                    self.pump.request(io::Request::Close { entity });
                }
            }
            Up::Failed(error) => self.event(Event::OwnFailure(format!("stdin: {error:?}"))),
            Up::Room => {}
        }
    }
    fn tick(&mut self) {
        self.apply();
        let deadline = self.tui.deadline(self.pump.now(), self.machine.next_deadline());
        for report in self.pump.poll(&mut self.face, deadline) {
            self.report(report);
        }
        for event in self.tui.tick() {
            self.event(event);
        }
        self.face_progress();
        let now = self.pump.now();
        if self.machine.next_deadline().is_some_and(|at| now >= at) {
            self.event(Event::Deadline);
        }
        self.apply();
        self.children.retain(|child| !child.settled);
    }
    fn report(&mut self, report: Report) {
        match report {
            Report::Io(io::Event::Stream { owner, up }) if self.input == Some(owner) => self.input_event(up),
            Report::Io(event) => {
                if let Some(index) =
                    event.owner().and_then(|owner| self.children.iter().position(|child| child.owns(owner)))
                {
                    let events = self.children[index].event(event, &mut self.pump);
                    for event in events {
                        self.event(event);
                    }
                }
            }
            Report::Opened => {
                if matches!(self.face_job, Some(FaceJob::Opening)) {
                    self.face_job = None;
                    self.event(Event::FaceDone(Ok(())));
                }
            }
            Report::BrowserClosed => {
                if self.browser_closing {
                    self.browser_closing = false;
                    self.event(Event::FaceDone(Ok(())));
                }
            }
            Report::Trouble(why) => {
                self.face_job = None;
                self.event(Event::FaceDone(Err(why)));
            }
        }
    }
    fn face_progress(&mut self) {
        if let Some(why) = self.face.failure() {
            if !matches!(self.face_job, Some(FaceJob::Cancel)) {
                self.event(Event::FaceDone(Err(why)));
            }
            self.face_job = None;
            return;
        }
        if self.face.pending() {
            return;
        }
        let Some(job) = self.face_job.take() else {
            return;
        };
        match job {
            FaceJob::Address => {
                if let Some(address) = self.face.take_address() {
                    self.event(Event::BrowserAddress(address));
                } else {
                    self.event(Event::FaceDone(Err("browser supplied no address".into())));
                }
            }
            FaceJob::Opening => self.face_job = Some(FaceJob::Opening),
            FaceJob::Find(find) => match self.face.find(&find) {
                Found::Pending => self.face_job = Some(FaceJob::Find(find)),
                Found::Absent => self.event(Event::Found(None)),
                Found::Node(node) => self.event(Event::Found(Some(node))),
            },
            FaceJob::Operation => self.event(Event::FaceDone(Ok(()))),
            FaceJob::Snapshot => {
                if let Some(text) = self.face.take_snapshot() {
                    self.event(Event::Snapshot(text));
                } else {
                    self.event(Event::FaceDone(Err("snapshot returned no text".into())));
                }
            }
            FaceJob::Screenshot(path) => {
                let result =
                    self.face.take_screenshot().ok_or_else(|| "screenshot returned no PNG".to_owned()).and_then(
                        |png| {
                            let mut file = OpenOptions::new()
                                .create_new(true)
                                .write(true)
                                .mode(0o600)
                                .open(&path)
                                .map_err(|error| format!("{}: {error}", path.display()))?;
                            file.write_all(&png).map_err(|error| format!("{}: {error}", path.display()))
                        },
                    );
                self.event(Event::FaceDone(result));
            }
            FaceJob::Controls(role) => match self.face.find_boxed(&Find::named(role, b"")) {
                Found::Pending => self.face_job = Some(FaceJob::Controls(role)),
                Found::Node(_) | Found::Absent => {
                    let (controls, more) = self.face.boxed_matches();
                    let mut lines: Vec<_> = controls
                        .iter()
                        .map(|seen| {
                            format!("{} {:?}", String::from_utf8_lossy(&seen.role), String::from_utf8_lossy(&seen.name))
                        })
                        .collect();
                    if more > 0 {
                        lines.push(format!("{more} more controls"));
                    }
                    if lines.is_empty() {
                        lines.push(format!("no {role:?} controls"));
                    }
                    self.event(Event::Description(lines.join("\n").into_bytes().into()));
                }
            },
            FaceJob::Description => {
                let text = self.face.take_snapshot().unwrap_or_default();
                self.event(Event::Description(text));
            }
            FaceJob::Cancel => {}
        }
    }
    #[expect(clippy::too_many_lines, reason = "one dispatch translates the session machine's actions without policy")]
    fn apply(&mut self) {
        while let Some(action) = self.actions.pop_front() {
            if matches!(self.face_job, Some(FaceJob::Cancel))
                && self.face.pending()
                && matches!(
                    action,
                    Action::BrowserAddress
                        | Action::Open(_)
                        | Action::Find(_)
                        | Action::Perform(_)
                        | Action::Snapshot
                        | Action::Screenshot { .. }
                        | Action::Describe { .. }
                )
            {
                self.actions.push_front(action);
                break;
            }
            match action {
                Action::StartTui { program, rows, columns } => {
                    if let Err(why) = self.tui.start(&program, rows, columns) {
                        self.event(Event::TuiFailure(why));
                    }
                }
                Action::WriteTui(bytes) => self.tui.write(bytes),
                Action::SaveTui => {
                    if let Err(why) = self.tui.save() {
                        self.event(Event::TuiFailure(why));
                    }
                }
                Action::StopTui => {
                    self.tui.stop();
                    self.event(Event::TuiStopped);
                }
                Action::BrowserAddress => {
                    self.face.address();
                    self.face_job = Some(FaceJob::Address);
                }
                Action::Start { process, program } => {
                    if process == Process::Ctl
                        && let Err(why) = super::options::check_destinations(&program, &self.scratch)
                    {
                        self.event(Event::FaceDone(Err(why)));
                        continue;
                    }
                    self.serial = self.serial.checked_add(1).expect("process identities fit");
                    self.children.push(Child::start(process, Token::new(self.serial), program, &mut self.pump));
                }
                Action::Open(url) => {
                    self.face_job = Some(if self.browser_started { FaceJob::Operation } else { FaceJob::Opening });
                    self.browser_started = true;
                    self.pump.open(url, &mut self.face);
                }
                Action::Find(find) => self.face_job = Some(FaceJob::Find(find)),
                Action::Perform(doing) => {
                    self.face_job = Some(FaceJob::Operation);
                    if let Err(why) = self.face.perform(doing) {
                        self.face_job = None;
                        self.event(Event::FaceDone(Err(why)));
                    }
                }
                Action::Snapshot => {
                    self.face.snapshot();
                    self.face_job = Some(FaceJob::Snapshot);
                }
                Action::Screenshot { width, path } => {
                    self.face.screenshot_at(width);
                    self.face_job = Some(FaceJob::Screenshot(path));
                }
                Action::Describe { role: Some(role) } => self.face_job = Some(FaceJob::Controls(role)),
                Action::Describe { role: None } => {
                    self.face.snapshot();
                    self.face_job = Some(FaceJob::Description);
                }
                Action::CancelFace => {
                    self.face.cancel();
                    self.face_job = if self.face.pending() { Some(FaceJob::Cancel) } else { None };
                }
                Action::StopCtl => {
                    for child in self.children.iter_mut().filter(|c| c.process != Process::Box) {
                        child.stop(&mut self.pump, true);
                    }
                }
                Action::Log { output: _, line } => {
                    if let Err(error) = self.box_log.write_all(&line) {
                        self.event(Event::OwnFailure(format!("session/box.log: {error}")));
                    }
                }
                Action::Keep(line) => {
                    if let Err(error) = self.kept.write_all(&line).and_then(|()| self.kept.write_all(b"\n")) {
                        self.event(Event::OwnFailure(format!("session/lines: {error}")));
                    }
                }
                Action::Answer(answer) => self.answers.push_back(answer),
                Action::Ready => self.ready = true,
                Action::CloseBrowser => {
                    self.browser_closing = true;
                    self.face_job = None;
                    self.pump.close_browser();
                }
                Action::TerminateBox | Action::KillBox => {
                    let force = matches!(action, Action::KillBox);
                    if force {
                        self.pump.force_browser();
                    }
                    for child in self.children.iter_mut().filter(|c| c.process == Process::Box) {
                        child.stop(&mut self.pump, force);
                    }
                }
                Action::Classify => match self.logs() {
                    Ok((serve, forges, box_tail)) => self.event(Event::Logs { serve, forges, box_tail }),
                    Err(why) => self.event(Event::OwnFailure(why)),
                },
                Action::Finished(exit) => self.exit = Some(exit),
            }
        }
    }
    fn logs(&self) -> Result<(LogLines, LogLines, LogLines), String> {
        logs(&self.scratch)
    }
    fn drain(&mut self) {
        self.tui.stop();
        if let Some(entity) = self.input.take() {
            self.pump.request(io::Request::Close { entity });
        }
        self.pump.force_browser();
        for child in &mut self.children {
            child.stop(&mut self.pump, true);
        }
        self.pump.close_root();
        while !self.pump.is_empty() {
            for report in self.pump.poll(&mut self.face, None) {
                if let Report::Io(event) = report
                    && let Some(index) =
                        event.owner().and_then(|owner| self.children.iter().position(|c| c.owns(owner)))
                {
                    let _events = self.children[index].event(event, &mut self.pump);
                }
            }
        }
    }
}

pub(super) fn logs(scratch: &std::path::Path) -> Result<(LogLines, LogLines, LogLines), String> {
    let read = |path: PathBuf| -> Result<LogLines, String> {
        match fs::read(&path) {
            Ok(bytes) => Ok(bytes
                .split_inclusive(|b| *b == b'\n')
                .map(|line| line.strip_suffix(b"\n").unwrap_or(line).into())
                .collect()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    };
    let serve = read(scratch.join("data/box/logs/serve.log"))?;
    let mut forges = Vec::new();
    for name in ["fake-github.log", "fake-forgejo.log"] {
        forges.extend(read(scratch.join("data/box/logs").join(name))?);
    }
    Ok((serve, forges, read(scratch.join("session/box.log"))?))
}
