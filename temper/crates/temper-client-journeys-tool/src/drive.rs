//! The complete event/action session policy, without io (client/drive.md, 2–6).
use crate::line::{Line, Record, Web};
use skein_browser::boundary::Trouble;
use skein_fake_person::{Doing, Find};
use skein_lib::{Duration, Time};
use skein_ui::{Act, NodeId, Role};
use std::{fmt::Write, path::PathBuf};
mod deployed;
pub mod deployment;
mod forge;
mod io;
mod options;
mod place;
mod requests;
use place::{Place, Progress};
mod tui;
mod values;
pub use io::Session;
pub use options::{Input, Options, Plan, Program};
const STEP: Duration = Duration::from_secs(60);
const LOOK: Duration = Duration::from_secs(10);
const REQUESTS: Duration = Duration::from_secs(30);
const QUIET: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(50);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub shown: Vec<String>,
    pub failed: Option<String>,
}
impl Answer {
    #[must_use]
    pub fn print(&self) -> String {
        let mut text = String::new();
        for line in &self.shown {
            text.push_str("| ");
            text.push_str(line);
            text.push('\n');
        }
        if let Some(why) = &self.failed {
            text.push_str("failed: ");
            text.push_str(why);
            text.push('\n');
        } else {
            text.push_str("ok\n");
        }
        text
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exit {
    pub status: u8,
    pub shown: Vec<String>,
    pub error: Option<String>,
}
impl Exit {
    #[must_use]
    pub fn own(mut why: String) -> Self {
        why.insert_str(0, "failed: ");
        Self { status: 1, shown: vec![], error: Some(why) }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Process {
    Box,
    Forgejo,
    Model,
    Engine,
    Worker,
    SignIn,
    Ctl,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Stdout,
    Stderr,
}
#[derive(Debug)]
pub enum Action {
    PrepareDeployment,
    ReadDeploymentWorker,
    ReadSettingsFiles {
        directory: Option<PathBuf>,
        args: Vec<Box<[u8]>>,
    },
    ReadDeploymentPatch {
        document: crate::line::Document,
        path: Box<[u8]>,
    },
    WriteDeploymentPatch(deployment::RenderedFile),
    SignInFile {
        link: Box<[u8]>,
        program: Program,
    },
    RemoveLink,
    DeploymentLog {
        process: Process,
        line: Box<[u8]>,
    },
    Stop {
        process: Process,
        force: bool,
    },
    DeploymentLogs,
    ReadBoxLog {
        path: PathBuf,
    },
    ForgeRequest {
        api: Box<[u8]>,
        token_file: PathBuf,
        method: skein_http::Method,
        path: Box<[u8]>,
        body: Box<[u8]>,
        until: Time,
    },
    ReadForgeFiles {
        directory: PathBuf,
        paths: Vec<Box<[u8]>>,
    },
    CancelForge,
    Start {
        process: Process,
        program: Program,
    },
    StartTui {
        program: Program,
        rows: u16,
        columns: u16,
    },
    WriteTui(Vec<u8>),
    SaveTui,
    StopTui,
    BrowserAddress,
    Open(Box<[u8]>),
    Find(Find),
    Perform(Doing<Box<[u8]>>),
    Snapshot,
    Screenshot {
        width: u32,
        path: PathBuf,
    },
    Describe {
        role: Option<Role>,
    },
    CancelFace,
    StopCtl,
    Log {
        line: Box<[u8]>,
    },
    Keep(Box<[u8]>),
    Answer(Answer),
    Ready,
    CloseBrowser,
    ForceBrowser,
    Classify,
    Finished(Exit),
}
#[derive(Debug)]
pub enum Event {
    DeploymentPrepared(Result<deployment::Values, String>),
    DeploymentWorker(Result<deployment::Worker, String>),
    SettingsFiles(Result<Vec<Box<[u8]>>, String>),
    DeploymentPatch(Result<deployment::RenderedFile, String>),
    DeploymentWritten(Result<(), String>),
    ProcessLine { process: Process, output: Output, line: Box<[u8]> },
    DeploymentLogs(Vec<Vec<Box<[u8]>>>),
    BoxLog(Result<Box<[u8]>, String>),
    ForgeFiles(Result<Vec<Box<[u8]>>, String>),
    ForgeResponse { status: u16, body: Box<[u8]> },
    ForgeFailure(String),
    TuiReady,
    TuiSaved(Box<[u8]>),
    TuiWritten,
    TuiStopped,
    TuiFailure(String),
    BrowserAddress(Box<[u8]>),
    Line(Box<[u8]>),
    End,
    BoxLine { output: Output, line: Box<[u8]> },
    Exited { process: Process, code: i32, stdout: Box<[u8]>, stderr: Box<[u8]> },
    Found(Option<NodeId>),
    FaceDone(Result<(), String>),
    PageTrouble { trouble: Trouble, text: Box<[u8]>, url: Box<[u8]> },
    PageSent { id: Box<[u8]>, load: Box<[u8]>, document: bool, method: Box<[u8]>, url: Box<[u8]> },
    PageAnswered { id: Box<[u8]> },
    Snapshot(Box<[u8]>),
    Description(Box<[u8]>),
    Logs { serve: Vec<Box<[u8]>>, forges: Vec<Box<[u8]>>, box_tail: Vec<Box<[u8]>> },
    Deadline,
    OwnFailure(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Place,
    SignIn,
    Spent,
    FreshLink,
    Fresh,
    Signing,
    Root,
    Main,
    Ready,
    Finding,
    Acting,
    Seeing,
    Reloading,
    MainLine,
    Screenshot,
    Snapshot,
    Diagnosing,
    Requests,
    Ctl,
    CtlValues,
    CtlFiles,
    Forge,
    TuiStarting,
    TuiBrowse,
    TuiAddress,
    TuiOpening,
    TuiApprove,
    TuiPressed,
    TuiReturning,
    TuiRecover,
    TuiSeeing,
    TuiLook,
    TuiKey,
    TuiStopping,
    ClosingTui,
    ClosingBrowser,
    Finished,
}
pub struct Machine {
    plan: Plan,
    place: Place,
    phase: Phase,
    until: Option<Time>,
    poll: Option<Time>,
    number: u64,
    current: Option<Line>,
    first: Option<Box<[u8]>>,
    fresh: Option<Box<[u8]>>,
    origin: Box<[u8]>,
    sign_in: Option<place::SignIn>,
    diagnostic: String,
    shown: Vec<String>,
    requests: requests::Log,
    forge: Option<forge::Pending>,
    tui_running: bool,
    save_until: Option<Time>,
    frame: Box<[u8]>,
    return_address: Option<Box<[u8]>>,
    tui_browse: Option<Box<[u8]>>,
}
impl Machine {
    #[must_use]
    pub fn start(plan: Plan, now: Time) -> (Self, Vec<Action>) {
        let (place, action) = Place::start(&plan, now);
        (
            Self {
                plan,
                place,
                phase: Phase::Place,
                until: None,
                poll: None,
                number: 0,
                current: None,
                first: None,
                fresh: None,
                origin: Box::new([]),
                sign_in: None,
                diagnostic: String::new(),
                shown: vec![],
                requests: requests::Log::new(now),
                forge: None,
                tui_running: false,
                save_until: None,
                frame: Box::new([]),
                return_address: None,
                tui_browse: None,
            },
            vec![action],
        )
    }
    #[must_use]
    pub fn ready(&self) -> bool {
        self.phase == Phase::Ready
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.phase == Phase::Finished {
            return None;
        }
        self.until.into_iter().chain(self.poll).chain(self.save_until).chain(self.place.deadline()).min()
    }
    pub fn step(&mut self, event: Event, now: Time) -> Vec<Action> {
        let mut out = vec![];
        if let Some(progress) = self.place.event(
            &event,
            place::Activity { phase: self.phase, busy: self.current.is_some(), tui_running: self.tui_running },
            &self.plan,
            now,
            &mut out,
        ) {
            if matches!(event, Event::TuiStopped | Event::TuiFailure(_))
                || matches!(progress, Progress::Waiting) && self.phase == Phase::TuiStopping
            {
                self.tui_running = false;
            }
            self.progress(progress, now, &mut out);
            return out;
        }
        match event {
            Event::SettingsFiles(result) if self.phase == Phase::CtlFiles => match result {
                Ok(args) => self.ctl_run(&args, now, &mut out),
                Err(why) => self.answer(Some(why), &mut out),
            },
            Event::BoxLog(result) if self.phase == Phase::CtlValues => self.ctl_values(result, now, &mut out),
            Event::BoxLog(_) | Event::ForgeFiles(_) | Event::ForgeResponse { .. } | Event::ForgeFailure(_) => {
                self.forge_event(event, now, &mut out);
            }
            Event::TuiReady
            | Event::TuiSaved(_)
            | Event::TuiWritten
            | Event::TuiStopped
            | Event::TuiFailure(_)
            | Event::BrowserAddress(_) => self.tui_event(event, now, &mut out),
            Event::Line(bytes) => self.line(&bytes, now, &mut out),
            Event::End => self.end(now, &mut out),

            Event::Exited { process, code, stdout, stderr } => {
                self.exited(process, code, &stdout, &stderr, now, &mut out);
            }
            Event::Found(node) => self.found(node, now, &mut out),
            Event::FaceDone(result) => self.face_done(result, now, &mut out),
            Event::PageTrouble { trouble, text, url } => self.page_trouble(trouble, &text, &url, now, &mut out),
            Event::PageSent { id, load, document, method, url } => {
                self.requests.sent(id, &load, document, method, &url, now);
                self.requests_check(now, &mut out);
            }
            Event::PageAnswered { id } => {
                self.requests.answered(&id, now);
                self.requests_check(now, &mut out);
            }
            Event::Snapshot(text) if self.phase == Phase::Snapshot => {
                self.shown.extend(shown_lines(&text));
                self.answer(None, &mut out);
            }
            Event::Description(text) if self.phase == Phase::Diagnosing => {
                let text = String::from_utf8_lossy(text.get(..4096).unwrap_or(&text));
                self.answer(Some(format!("{}; shown instead: {text}", self.diagnostic)), &mut out);
            }
            Event::Deadline => self.deadline(now, &mut out),
            Event::OwnFailure(why) => {
                let force = self.place.own_failure(&mut out);
                self.phase = Phase::Finished;
                self.until = None;
                self.poll = None;
                self.save_until = None;
                if self.forge.take().is_some() {
                    out.push(Action::CancelForge);
                }
                out.extend([
                    Action::StopTui,
                    Action::CancelFace,
                    Action::StopCtl,
                    force,
                    Action::Finished(Exit::own(why)),
                ]);
            }
            Event::SettingsFiles(_)
            | Event::DeploymentPrepared(_)
            | Event::DeploymentWorker(_)
            | Event::DeploymentPatch(_)
            | Event::DeploymentWritten(_)
            | Event::BoxLine { .. }
            | Event::ProcessLine { .. }
            | Event::DeploymentLogs(_)
            | Event::Snapshot(_)
            | Event::Description(_)
            | Event::Logs { .. } => {}
        }
        out
    }
    fn wait(&mut self, phase: Phase, now: Time, bound: Duration) {
        self.phase = phase;
        self.until = Some(now.saturating_add(bound));
        self.poll = None;
    }
    fn line(&mut self, source: &[u8], now: Time, out: &mut Vec<Action>) {
        if !self.ready() {
            return;
        }
        self.number = self.number.saturating_add(1);
        self.current = match Line::parse(source) {
            Ok(line) => Some(line),
            Err(why) => {
                out.push(Action::Answer(Answer {
                    shown: vec![],
                    failed: Some(
                        self.place.early_exit().map_or_else(|| format!("line {}: {why}", self.number), str::to_owned),
                    ),
                }));
                return;
            }
        };
        let record = self.current.as_ref().expect("current line").record();
        if let Some(why) = self.place.early_exit().filter(|_| !matches!(record, Record::End)) {
            if let Some(line) = self
                .current
                .take()
                .filter(|line| line.kept() && !matches!(line.record(), Record::Web(_) | Record::Tui(_)))
            {
                out.push(Action::Keep(line.print().into()));
            }
            out.push(Action::Answer(Answer { shown: vec![], failed: Some(why.into()) }));
            return;
        }
        let refused = match record {
            Record::Web(Web::Mark) => Some("web mark is a journey's line"),
            _ => self.place.refuse_line(record, self.plan.options.ctl),
        };
        if let Some(why) = refused {
            self.current = None;
            self.answer(Some(why.into()), out);
            return;
        }
        if matches!(record, Record::End) {
            self.end(now, out);
            return;
        }
        if let Some(why) = self.place.exited_line() {
            self.answer(Some(why.into()), out);
            return;
        }
        match record {
            Record::Empty => self.answer(None, out),
            Record::End => unreachable!("end dispatched"),
            Record::Deployment(patch) => {
                let progress = self.place.patch(patch, self.tui_running, &self.plan, now, out);
                self.progress(progress, now, out);
            }
            Record::Forge(_) => self.forge_start(now, out),
            Record::Tui(_) => self.tui(now, out),
            Record::Ctl(args) => match self.place.ctl_values(args, None) {
                Ok(Some(args)) => self.ctl_start(&args, now, out),
                Ok(None) => {
                    self.wait(Phase::CtlValues, now, STEP);
                    out.push(Place::read_values(&self.plan));
                }
                Err(why) => self.answer(Some(why), out),
            },
            Record::Web(Web::Mark) => unreachable!("journey line refused"),
            Record::Web(Web::Act { role, name, order, .. }) => {
                let find = Find::exactly_named(*role, name).nth(*order);
                self.wait(Phase::Finding, now, STEP);
                out.push(Action::Find(find));
            }
            Record::Web(Web::See(text)) => {
                let find = Find::text(text);
                self.wait(Phase::Seeing, now, STEP);
                out.push(Action::Find(find));
            }
            Record::Web(Web::Reload(path)) => {
                let url = [self.origin.as_ref(), path].concat().into();
                self.wait(Phase::Reloading, now, STEP);
                out.push(Action::Open(url));
            }
            Record::Web(Web::Look(Some(width))) => {
                let width = *width;
                let path = self.plan.look(self.number, width);
                self.wait(Phase::Screenshot, now, LOOK);
                out.push(Action::Screenshot { width, path });
            }
            Record::Web(Web::Requests) => {
                self.wait(Phase::Requests, now, REQUESTS);
                self.requests_check(now, out);
            }
            Record::Web(Web::Look(None)) => {
                self.wait(Phase::Snapshot, now, LOOK);
                out.push(Action::Snapshot);
            }
        }
    }
    fn ctl_values(&mut self, result: Result<Box<[u8]>, String>, now: Time, out: &mut Vec<Action>) {
        let Some(Record::Ctl(args)) = self.current.as_ref().map(Line::record) else { return };
        let resolved = result.and_then(|log| self.place.ctl_values(args, Some(&log)));
        match resolved {
            Ok(Some(args)) => self.ctl_start(&args, now, out),
            Ok(None) => unreachable!("values supplied"),
            Err(why) => self.answer(Some(why), out),
        }
    }
    fn ctl_start(&mut self, args: &[Box<[u8]>], now: Time, out: &mut Vec<Action>) {
        if let Err(why) = self.plan.check_ctl_writes(args) {
            self.answer(Some(why), out);
            return;
        }
        if crate::line::settings_paths(args).is_empty() {
            self.ctl_run(args, now, out);
        } else {
            self.wait(Phase::CtlFiles, now, STEP);
            out.push(Action::ReadSettingsFiles { directory: self.plan.options.files.clone(), args: args.to_vec() });
        }
    }
    fn ctl_run(&mut self, args: &[Box<[u8]>], now: Time, out: &mut Vec<Action>) {
        let program = self.plan.ctl(args, Input::Null);
        self.wait(Phase::Ctl, now, ctl_bound(args));
        out.push(Action::Start { process: Process::Ctl, program });
    }
    fn answer(&mut self, why: Option<String>, out: &mut Vec<Action>) {
        let failed = why.map(|why| format!("line {}: {}", self.number, why.replace(['\n', '\r'], " ")));
        if let Some(line) = self.current.take().filter(Line::kept) {
            out.push(Action::Keep(line.print().into()));
        }
        out.push(Action::Answer(Answer { shown: std::mem::take(&mut self.shown), failed }));
        self.place.line_finished();
        self.phase = Phase::Ready;
        self.until = None;
        self.poll = None;
        self.save_until = None;
        self.forge = None;
    }
    fn exited(&mut self, process: Process, code: i32, stdout: &[u8], stderr: &[u8], now: Time, out: &mut Vec<Action>) {
        match process {
            Process::SignIn if self.phase == Phase::SignIn => {
                out.push(Action::RemoveLink);
                let rules = self.sign_in.expect("sign-in rules");
                if rules.strict_deadline && self.until.is_some_and(|at| now >= at) {
                    self.fail_place("ctl sign-in deadline passed".into(), now, out);
                } else if code != 0 {
                    let why = if rules.stderr {
                        format!("ctl sign-in exited {code}: {}", String::from_utf8_lossy(stderr))
                    } else {
                        format!("ctl sign-in exited {code}")
                    };
                    self.fail_place(why, now, out);
                } else if self.place.signed_in(now, out) {
                    self.wait_for_place();
                } else {
                    self.after_sign_in(now, out);
                }
            }
            Process::Ctl if self.phase == Phase::Ctl => {
                self.shown.extend(shown_lines(stderr));
                self.shown.extend(shown_lines(stdout));
                self.answer(if code == 0 { None } else { Some(format!("exit {code}")) }, out);
            }
            Process::Box
            | Process::Forgejo
            | Process::Model
            | Process::Engine
            | Process::Worker
            | Process::SignIn
            | Process::Ctl => {}
        }
    }
    fn after_sign_in(&mut self, now: Time, out: &mut Vec<Action>) {
        if self.plan.options.ctl {
            self.first = None;
            self.session_ready(out);
        } else {
            self.wait(Phase::Spent, now, Duration::from_secs(30));
            out.push(Action::Open(self.first.take().expect("first link")));
        }
    }
    fn found(&mut self, node: Option<NodeId>, now: Time, out: &mut Vec<Action>) {
        if self.phase == Phase::TuiApprove {
            if let Some(node) = node {
                self.phase = Phase::TuiPressed;
                self.poll = None;
                out.push(Action::Perform(Doing::Act { node, act: Act::Press }));
            } else {
                self.poll = Some(now.saturating_add(POLL));
            }
            return;
        }
        if !matches!(self.phase, Phase::Signing | Phase::Main | Phase::MainLine | Phase::Seeing | Phase::Finding) {
            return;
        }
        if node.is_none() {
            self.poll = Some(now.saturating_add(POLL));
            return;
        }
        match self.phase {
            Phase::Signing => {
                self.phase = Phase::Root;
                self.poll = None;
                out.push(Action::Open([self.origin.as_ref(), b"/"].concat().into()));
            }
            Phase::MainLine if self.patch_line() => self.patched(out),
            Phase::Main => self.session_ready(out),
            Phase::MainLine | Phase::Seeing => self.answer(None, out),
            Phase::Finding => {
                let Some(Record::Web(Web::Act { act, .. })) = self.current.as_ref().map(Line::record) else {
                    unreachable!("control act")
                };
                let act = copy_act(act);
                self.phase = Phase::Acting;
                self.poll = None;
                out.push(Action::Perform(Doing::Act { node: node.expect("found node"), act }));
            }
            _ => {}
        }
    }
    fn page_trouble(&mut self, trouble: Trouble, text: &[u8], url: &[u8], now: Time, out: &mut Vec<Action>) {
        if self.patch_line() {
            return;
        }
        if matches!(
            self.phase,
            Phase::Spent | Phase::FreshLink | Phase::Fresh | Phase::Signing | Phase::Root | Phase::Main
        ) && trouble == Trouble::Log
            && text.windows(b"status of 401".len()).any(|part| part == b"status of 401")
            && resource(url).strip_prefix(self.origin.as_ref()) == Some(b"/jig/v1/sign-in/link".as_slice())
        {
            return;
        }
        self.face_done(Err(trouble_words(trouble, text, url)), now, out);
    }
    fn face_done(&mut self, result: Result<(), String>, now: Time, out: &mut Vec<Action>) {
        if self.tui_face_done(&result, now, out) {
            return;
        }
        if let Err(why) = result {
            if self.phase == Phase::ClosingBrowser {
                self.stop_place(now, out);
            } else if self.current.is_some() {
                out.push(Action::CancelFace);
                self.answer(Some(format!("{}: {why}", self.sought())), out);
            } else if self.phase != Phase::Ready {
                self.fail_place(why, now, out);
            }
            return;
        }
        match self.phase {
            Phase::Spent => {
                if let Some(link) = self.fresh.take() {
                    self.wait(Phase::Fresh, now, STEP);
                    out.push(Action::Open(link));
                } else {
                    self.phase = Phase::FreshLink;
                }
            }
            Phase::Fresh => {
                self.phase = Phase::Signing;
                out.push(Action::Find(Find::named(Role::Main, b"")));
            }
            Phase::Root => {
                self.phase = Phase::Main;
                out.push(Action::Find(Find::named(Role::Main, b"")));
            }
            Phase::Reloading => {
                self.phase = Phase::MainLine;
                out.push(Action::Find(Find::named(Role::Main, b"")));
            }
            Phase::Acting => self.answer(None, out),
            Phase::Screenshot => {
                if let Some(Record::Web(Web::Look(Some(width)))) = self.current.as_ref().map(Line::record) {
                    self.shown.push(self.plan.look(self.number, *width).display().to_string());
                }
                self.phase = Phase::Snapshot;
                out.push(Action::Snapshot);
            }
            Phase::ClosingBrowser => self.stop_place(now, out),
            _ => {}
        }
    }
    fn sought(&self) -> String {
        self.current
            .as_ref()
            .map_or_else(|| format!("{:?}", self.phase), |line| String::from_utf8_lossy(line.print()).into_owned())
    }
    fn deadline(&mut self, now: Time, out: &mut Vec<Action>) {
        if self.phase == Phase::Requests {
            self.requests_check(now, out);
            return;
        }
        if self.forge_deadline(now, out) {
            return;
        }
        if self.tui_deadline(now, out) {
            return;
        }
        if self.until.is_some_and(|until| now >= until) {
            if self.current.is_some() && !matches!(self.phase, Phase::ClosingBrowser) {
                out.push(Action::CancelFace);
                if self.phase == Phase::Ctl {
                    out.push(Action::StopCtl);
                }
                if matches!(self.phase, Phase::Finding | Phase::Seeing | Phase::MainLine) {
                    let role = match self.current.as_ref().map(Line::record) {
                        Some(Record::Web(Web::Act { role, .. })) => Some(*role),
                        _ => None,
                    };
                    self.diagnostic = format!("{} not found within 60 s", self.sought());
                    self.wait(Phase::Diagnosing, now, LOOK);
                    out.push(Action::Describe { role });
                } else {
                    self.answer(Some(format!("{}: deadline passed", self.sought())), out);
                }
            } else if self.phase == Phase::ClosingBrowser {
                self.place.browser_close_expired(out);
                self.stop_place(now, out);
            } else if self.phase != Phase::Finished {
                let why = if self.phase == Phase::SignIn {
                    self.sign_in.expect("sign-in rules").deadline.into()
                } else {
                    format!("{:?}: deadline passed", self.phase)
                };
                self.fail_place(why, now, out);
            }
            return;
        }
        if self.poll.is_some_and(|poll| now >= poll) {
            self.poll = None;
            let find = match self.phase {
                Phase::Signing | Phase::Main | Phase::MainLine => Some(Find::named(Role::Main, b"")),
                Phase::Finding => match self.current.as_ref().map(Line::record) {
                    Some(Record::Web(Web::Act { role, name, order, .. })) => {
                        Some(Find::exactly_named(*role, name).nth(*order))
                    }
                    _ => None,
                },
                Phase::Seeing => match self.current.as_ref().map(Line::record) {
                    Some(Record::Web(Web::See(text))) => Some(Find::text(text)),
                    _ => None,
                },
                _ => None,
            };
            if let Some(find) = find {
                out.push(Action::Find(find));
            }
        }
    }
    fn end(&mut self, now: Time, out: &mut Vec<Action>) {
        if matches!(self.phase, Phase::Finished | Phase::ClosingTui | Phase::ClosingBrowser) || self.place.ending() {
            return;
        }
        self.current = None;
        if self.forge.take().is_some() {
            out.push(Action::CancelForge);
        }
        self.place.begin_end(out);
        self.close_clients(now, out);
    }
    fn close_clients(&mut self, now: Time, out: &mut Vec<Action>) {
        self.save_until = None;
        if self.tui_running {
            self.wait(Phase::ClosingTui, now, LOOK);
            out.extend([Action::CancelFace, Action::StopCtl, Action::StopTui]);
        } else if self.plan.options.ctl {
            out.push(Action::StopCtl);
            self.stop_place(now, out);
        } else {
            self.wait(Phase::ClosingBrowser, now, STEP);
            out.extend([Action::CancelFace, Action::StopCtl, Action::CloseBrowser]);
        }
    }
    fn fail_place(&mut self, why: String, now: Time, out: &mut Vec<Action>) {
        let progress = self.place.failed(why, self.current.is_some(), self.phase, out);
        if self.forge.take().is_some() {
            out.push(Action::CancelForge);
        }
        self.progress(progress, now, out);
    }
    fn stop_place(&mut self, now: Time, out: &mut Vec<Action>) {
        let progress = self.place.stop(&self.plan, now, out);
        self.progress(progress, now, out);
    }
    fn wait_for_place(&mut self) {
        self.phase = Phase::Place;
        self.until = None;
        self.poll = None;
    }
    fn session_ready(&mut self, out: &mut Vec<Action>) {
        self.phase = Phase::Ready;
        self.until = None;
        self.poll = None;
        out.push(Action::Ready);
    }
    fn patch_line(&self) -> bool {
        self.place.patch_deadline().is_some()
    }
    fn patched(&mut self, out: &mut Vec<Action>) {
        if let Some(Record::Deployment(crate::line::Deployment::Patch { document, .. })) =
            self.current.as_ref().map(Line::record)
        {
            self.shown.push(format!("patched {}", document.name()));
        }
        self.answer(None, out);
    }
    fn progress(&mut self, progress: Progress, now: Time, out: &mut Vec<Action>) {
        match progress {
            Progress::None => {}
            Progress::Waiting => self.wait_for_place(),
            Progress::Started => self.after_sign_in(now, out),
            Progress::First(first) => {
                self.first = Some(first.link.strip_suffix(b"\n").unwrap_or(&first.link).into());
                self.origin = first.origin;
                self.sign_in = Some(first.sign_in);
                self.wait(Phase::SignIn, now, STEP);
                out.push(Action::SignInFile { link: first.link, program: self.plan.deployment_sign_in() });
            }
            Progress::Link { link, replace } => match self.phase {
                Phase::Spent if replace || self.fresh.is_none() => self.fresh = Some(link),
                Phase::FreshLink => {
                    self.wait(Phase::Fresh, now, STEP);
                    out.push(Action::Open(link));
                }
                _ => {}
            },
            Progress::Failed(why) => self.fail_place(why, now, out),
            Progress::LostLine(why) => {
                if self.forge.take().is_some() {
                    out.push(Action::CancelForge);
                }
                out.extend([Action::CancelFace, Action::StopCtl]);
                self.answer(Some(why), out);
            }
            Progress::End => self.end(now, out),
            Progress::Answer(why) => self.answer(why, out),
            Progress::PatchTui => {
                self.wait(Phase::TuiStopping, now, LOOK);
                out.push(Action::StopTui);
            }
            Progress::Patched => {
                if self.plan.options.ctl {
                    self.patched(out);
                } else {
                    self.wait(Phase::Reloading, now, STEP);
                    self.until = self.until.into_iter().chain(self.place.patch_deadline()).min();
                    out.push(Action::Open([self.origin.as_ref(), b"/"].concat().into()));
                }
            }
            Progress::Finished(exit) => self.finish(exit, out),
        }
    }
    fn finish(&mut self, exit: Exit, out: &mut Vec<Action>) {
        self.phase = Phase::Finished;
        self.until = None;
        self.poll = None;
        out.push(Action::Finished(exit));
    }
}
fn copy_act(act: &Act) -> Act {
    match act {
        Act::Press => Act::Press,
        Act::Submit => Act::Submit,
        Act::Toggle => Act::Toggle,
        Act::Input { value } => Act::Input { value: value.clone() },
        Act::Near { .. } | Act::Drawing { .. } => unreachable!("no drive line gives an internal act"),
    }
}
fn resource(url: &[u8]) -> &[u8] {
    url.split(|byte| b"?#".contains(byte)).next().unwrap_or_default()
}
fn trouble_words(trouble: Trouble, text: &[u8], url: &[u8]) -> String {
    let mut words = format!("{trouble:?}: {}", String::from_utf8_lossy(text));
    let url = resource(url);
    if !url.is_empty() {
        write!(words, " ({})", String::from_utf8_lossy(url)).expect("string write");
    }
    words
}
fn shown_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes).lines().map(str::to_owned).collect()
}
fn ctl_bound(args: &[Box<[u8]>]) -> Duration {
    if !args.iter().any(|arg| [b"--until".as_slice(), b"--since"].contains(&arg.as_ref())) {
        return STEP;
    }
    let seconds = args.windows(2).find(|p| p[0].as_ref() == b"--timeout").and_then(|p| duration(&p[1])).unwrap_or(300);
    Duration::from_secs(seconds.saturating_add(10))
}
fn duration(bytes: &[u8]) -> Option<u64> {
    let text = std::str::from_utf8(bytes).ok()?;
    let (n, scale) = if let Some(n) = text.strip_suffix('s') {
        (n, 1)
    } else if let Some(n) = text.strip_suffix('m') {
        (n, 60)
    } else if let Some(n) = text.strip_suffix('h') {
        (n, 3600)
    } else {
        (text, 1)
    };
    n.parse::<u64>().ok()?.checked_mul(scale).filter(|n| *n <= 3600)
}
#[cfg(test)]
pub(crate) mod tests;
