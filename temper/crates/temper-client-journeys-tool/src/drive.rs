//! The complete event/action session policy, without io (client/drive.md, 2–6).
use crate::line::{Line, Record, Web};
use skein_fake_person::{Doing, Find};
use skein_lib::{Duration, Time};
use skein_ui::{Act, NodeId, Role};
use std::path::PathBuf;
mod io;
mod options;
mod tui;
pub use io::Session;
pub use options::{Input, Options, Plan, Program};
const STEP: Duration = Duration::from_secs(60);
const LOOK: Duration = Duration::from_secs(10);
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
    Start { process: Process, program: Program },
    StartTui { program: Program, rows: u16, columns: u16 },
    WriteTui(Vec<u8>),
    SaveTui,
    StopTui,
    BrowserAddress,
    Open(Box<[u8]>),
    Find(Find),
    Perform(Doing<Box<[u8]>>),
    Snapshot,
    Screenshot { width: u32, path: PathBuf },
    Describe { role: Option<Role> },
    CancelFace,
    StopCtl,
    Log { output: Output, line: Box<[u8]> },
    Keep(Box<[u8]>),
    Answer(Answer),
    Ready,
    CloseBrowser,
    TerminateBox,
    KillBox,
    Classify,
    Finished(Exit),
}
#[derive(Debug)]
pub enum Event {
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
    Snapshot(Box<[u8]>),
    Description(Box<[u8]>),
    Logs { serve: Vec<Box<[u8]>>, box_tail: Vec<Box<[u8]>> },
    Deadline,
    OwnFailure(String),
}
#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Link,
    SignIn,
    Spent,
    FreshLink,
    Fresh,
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
    Ctl,
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
    Stopping,
    Classifying,
    Finished,
}
pub struct Machine {
    plan: Plan,
    phase: Phase,
    until: Option<Time>,
    poll: Option<Time>,
    number: u64,
    current: Option<Line>,
    first: Option<Box<[u8]>>,
    fresh: Option<Box<[u8]>>,
    origin: Box<[u8]>,
    box_exited: bool,
    box_failed: Option<String>,
    diagnostic: String,
    shown: Vec<String>,
    tui_running: bool,
    save_until: Option<Time>,
    frame: Box<[u8]>,
    return_address: Option<Box<[u8]>>,
    tui_browse: Option<Box<[u8]>>,
}
impl Machine {
    #[must_use]
    pub fn start(plan: Plan, now: Time) -> (Self, Vec<Action>) {
        let action = Action::Start { process: Process::Box, program: plan.box_program() };
        (
            Self {
                plan,
                phase: Phase::Link,
                until: Some(now.saturating_add(STEP)),
                poll: None,
                number: 0,
                current: None,
                first: None,
                fresh: None,
                origin: Box::new([]),
                box_exited: false,
                box_failed: None,
                diagnostic: String::new(),
                shown: vec![],
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
        self.until.into_iter().chain(self.poll).chain(self.save_until).min()
    }
    pub fn step(&mut self, event: Event, now: Time) -> Vec<Action> {
        let mut out = vec![];
        match event {
            Event::TuiReady | Event::TuiSaved(_) | Event::TuiWritten | Event::TuiStopped
            | Event::TuiFailure(_) | Event::BrowserAddress(_) => self.tui_event(event, now, &mut out),
            Event::Line(bytes) => self.line(&bytes, now, &mut out),
            Event::End => self.end(now, &mut out),
            Event::BoxLine { output, line } => self.box_line(output, &line, now, &mut out),
            Event::Exited { process, code, stdout, stderr } => {
                self.exited(process, code, &stdout, &stderr, now, &mut out);
            }
            Event::Found(node) => self.found(node, now, &mut out),
            Event::FaceDone(result) => self.face_done(result, now, &mut out),
            Event::Snapshot(text) if self.phase == Phase::Snapshot => {
                self.shown.extend(shown_lines(&text));
                self.answer(None, &mut out);
            }
            Event::Description(text) if self.phase == Phase::Diagnosing => {
                let text = String::from_utf8_lossy(text.get(..4096).unwrap_or(&text));
                self.answer(Some(format!("{}; shown instead: {text}", self.diagnostic)), &mut out);
            }
            Event::Logs { serve, box_tail } if self.phase == Phase::Classifying => {
                self.classify(&serve, &box_tail, &mut out);
            }
            Event::Deadline => self.deadline(now, &mut out),
            Event::OwnFailure(why) => {
                self.phase = Phase::Finished;
                self.until = None;
                self.poll = None;
                self.save_until = None;
                out.extend([Action::StopTui, Action::CancelFace, Action::StopCtl, Action::KillBox, Action::Finished(Exit::own(why))]);
            }
            Event::Snapshot(_) | Event::Description(_) | Event::Logs { .. } => {}
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
                    failed: Some(format!("line {}: {why}", self.number)),
                }));
                return;
            }
        };
        let record = self.current.as_ref().expect("current line").record();
        if matches!(record, Record::End) {
            self.end(now, out);
            return;
        }
        if self.box_exited {
            self.answer(Some("the box exited".into()), out);
            return;
        }
        match record {
            Record::Empty => self.answer(None, out),
            Record::End => unreachable!("end dispatched"),
            Record::Tui(_) => self.tui(now, out),
            Record::Ctl(args) => {
                if let Err(why) = self.plan.check_ctl_writes(args) {
                    self.answer(Some(why), out);
                    return;
                }
                let program = self.plan.ctl(args, Input::Null);
                let bound = ctl_bound(args);
                self.wait(Phase::Ctl, now, bound);
                out.push(Action::Start { process: Process::Ctl, program });
            }
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
            Record::Web(Web::Look(None)) => {
                self.wait(Phase::Snapshot, now, LOOK);
                out.push(Action::Snapshot);
            }
        }
    }
    fn answer(&mut self, why: Option<String>, out: &mut Vec<Action>) {
        let failed = why.map(|why| format!("line {}: {}", self.number, why.replace(['\n', '\r'], " ")));
        if let Some(line) = self.current.take().filter(Line::kept) {
            out.push(Action::Keep(line.print().into()));
        }
        out.push(Action::Answer(Answer { shown: std::mem::take(&mut self.shown), failed }));
        self.phase = Phase::Ready;
        self.until = None;
        self.poll = None;
        self.save_until = None;
    }
    fn box_line(&mut self, output: Output, line: &[u8], now: Time, out: &mut Vec<Action>) {
        out.push(Action::Log { output, line: line.into() });
        if output == Output::Stderr
            && (line.windows(11).any(|s| s == b"; giving up") || line.windows(13).any(|s| s == b"; restarting "))
        {
            if matches!(
                self.phase,
                Phase::Link
                    | Phase::SignIn
                    | Phase::Spent
                    | Phase::FreshLink
                    | Phase::Fresh
                    | Phase::Root
                    | Phase::Main
            ) {
                self.fail_box(String::from_utf8_lossy(line).into_owned(), now, out);
            }
            return;
        }
        if output != Output::Stdout || !line.starts_with(b"http://") {
            return;
        }
        let line: Box<[u8]> = line.strip_suffix(b"\n").unwrap_or(line).into();
        match self.phase {
            Phase::Link => {
                let Some(origin) = origin(&line) else {
                    self.fail_box("invalid first link".into(), now, out);
                    return;
                };
                self.origin = origin;
                self.first = Some(line.clone());
                let mut input = line.to_vec();
                input.push(b'\n');
                let program =
                    self.plan.ctl(&[b"sign-in".as_slice().into(), b"-".as_slice().into()], Input::Bytes(input.into()));
                self.wait(Phase::SignIn, now, STEP);
                out.push(Action::Start { process: Process::SignIn, program });
            }
            Phase::Spent => self.fresh = Some(line),
            Phase::FreshLink => {
                self.wait(Phase::Fresh, now, STEP);
                out.push(Action::Open(line));
            }
            _ => {}
        }
    }
    fn exited(&mut self, process: Process, code: i32, stdout: &[u8], stderr: &[u8], now: Time, out: &mut Vec<Action>) {
        match process {
            Process::Box => {
                self.box_exited = true;
                if self.phase == Phase::Stopping {
                    if code == 0 && self.box_failed.is_none() {
                        self.finish(Exit { status: 0, shown: vec!["ended".into()], error: None }, out);
                    } else {
                        self.classifying(format!("box exited {code}"), out);
                    }
                } else if self.phase == Phase::Ready {
                    self.box_failed.get_or_insert_with(|| format!("box exited {code}"));
                } else if !matches!(self.phase, Phase::Finished | Phase::Classifying | Phase::ClosingBrowser) {
                    if self.current.is_some() {
                        out.extend([Action::CancelFace, Action::StopCtl]);
                        self.answer(Some("the box exited".into()), out);
                        self.box_failed = Some(format!("box exited {code}"));
                    } else {
                        self.fail_box(format!("box exited {code} before ready"), now, out);
                    }
                }
            }
            Process::SignIn if self.phase == Phase::SignIn => {
                if code != 0 {
                    self.fail_box(format!("ctl sign-in exited {code}: {}", String::from_utf8_lossy(stderr)), now, out);
                } else {
                    self.wait(Phase::Spent, now, Duration::from_secs(30));
                    out.push(Action::Open(self.first.clone().expect("first link")));
                }
            }
            Process::Ctl if self.phase == Phase::Ctl => {
                self.shown.extend(shown_lines(stderr));
                self.shown.extend(shown_lines(stdout));
                self.answer(if code == 0 { None } else { Some(format!("exit {code}")) }, out);
            }
            Process::SignIn | Process::Ctl => {}
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
        if !matches!(self.phase, Phase::Main | Phase::MainLine | Phase::Seeing | Phase::Finding) {
            return;
        }
        if node.is_none() {
            self.poll = Some(now.saturating_add(POLL));
            return;
        }
        match self.phase {
            Phase::Main => {
                self.phase = Phase::Ready;
                self.until = None;
                self.poll = None;
                out.push(Action::Ready);
            }
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
    fn face_done(&mut self, result: Result<(), String>, now: Time, out: &mut Vec<Action>) {
        if self.tui_face_done(&result, now, out) { return; }
        if let Err(why) = result {
            if self.phase == Phase::ClosingBrowser {
                self.stop_box(now, out);
            } else if self.current.is_some() {
                out.push(Action::CancelFace);
                self.answer(Some(format!("{}: {why}", self.sought())), out);
            } else if self.phase != Phase::Ready {
                self.fail_box(why, now, out);
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
                self.phase = Phase::Root;
                out.push(Action::Open([self.origin.as_ref(), b"/"].concat().into()));
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
            Phase::ClosingBrowser => self.stop_box(now, out),
            _ => {}
        }
    }
    fn sought(&self) -> String {
        self.current
            .as_ref()
            .map_or_else(|| format!("{:?}", self.phase), |line| String::from_utf8_lossy(line.print()).into_owned())
    }
    fn deadline(&mut self, now: Time, out: &mut Vec<Action>) {
        if self.tui_deadline(now, out) { return; }
        if self.until.is_some_and(|until| now >= until) {
            if self.current.is_some() && !matches!(self.phase, Phase::ClosingBrowser | Phase::Stopping) {
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
            } else if self.phase == Phase::Stopping {
                out.push(Action::KillBox);
                self.classifying("box did not stop within 60 s".into(), out);
            } else if self.phase == Phase::ClosingBrowser {
                self.box_failed.get_or_insert("browser did not close within 60 s".into());
                out.push(Action::KillBox);
                self.stop_box(now, out);
            } else if !matches!(self.phase, Phase::Finished | Phase::Classifying) {
                self.fail_box(format!("{:?}: deadline passed", self.phase), now, out);
            }
            return;
        }
        if self.poll.is_some_and(|poll| now >= poll) {
            self.poll = None;
            let find = match self.phase {
                Phase::Main | Phase::MainLine => Some(Find::named(Role::Main, b"")),
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
        if matches!(self.phase, Phase::Finished | Phase::Classifying | Phase::ClosingTui | Phase::ClosingBrowser | Phase::Stopping) {
            return;
        }
        self.current = None;
        self.save_until = None;
        if self.tui_running {
            self.wait(Phase::ClosingTui, now, LOOK);
            out.extend([Action::CancelFace, Action::StopCtl, Action::StopTui]);
        } else {
            self.wait(Phase::ClosingBrowser, now, STEP);
            out.extend([Action::CancelFace, Action::StopCtl, Action::CloseBrowser]);
        }
    }
    fn fail_box(&mut self, why: String, now: Time, out: &mut Vec<Action>) {
        self.box_failed.get_or_insert(why);
        self.end(now, out);
    }
    fn stop_box(&mut self, now: Time, out: &mut Vec<Action>) {
        if self.box_exited {
            self.classifying(self.box_failed.clone().unwrap_or_else(|| "the box exited".into()), out);
        } else {
            self.wait(Phase::Stopping, now, STEP);
            out.push(Action::TerminateBox);
        }
    }
    fn classifying(&mut self, why: String, out: &mut Vec<Action>) {
        self.box_failed.get_or_insert(why);
        self.phase = Phase::Classifying;
        self.until = None;
        self.poll = None;
        out.push(Action::Classify);
    }
    fn classify(&mut self, serve: &[Box<[u8]>], box_tail: &[Box<[u8]>], out: &mut Vec<Action>) {
        let scratch = self.plan.options.scratch.as_os_str().as_encoded_bytes();
        let refusal = serve.iter().find_map(|line| {
            let line = strip_log_time(line);
            if line.starts_with(b"temper serve: ") && names_scratch(line, scratch) {
                Some(String::from_utf8_lossy(line).into_owned())
            } else {
                None
            }
        });
        let (status, shown) = if let Some(line) = refusal {
            (3, vec![format!("refused: {line}")])
        } else {
            let mut shown = vec![format!(
                "box failed: {}",
                self.box_failed.as_deref().unwrap_or("the box exited").replace(['\r', '\n'], " ")
            )];
            for lines in [box_tail, serve] {
                for line in lines.iter().skip(lines.len().saturating_sub(20)) {
                    shown.push(format!("| {}", String::from_utf8_lossy(line).trim_end_matches('\n')));
                }
            }
            (4, shown)
        };
        self.finish(Exit { status, shown, error: None }, out);
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
fn shown_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes).lines().map(str::to_owned).collect()
}
fn origin(link: &[u8]) -> Option<Box<[u8]>> {
    let rest = link.strip_prefix(b"http://")?;
    let end = rest.iter().position(|b| b"/?#".contains(b)).unwrap_or(rest.len());
    if end == 0 { None } else { Some(link.get(..end.checked_add(7)?)?.into()) }
}
fn strip_log_time(line: &[u8]) -> &[u8] {
    if line.get(10) == Some(&b'T') && line.get(19..21) == Some(b"Z ") { line.get(21..).unwrap_or(line) } else { line }
}
fn names_scratch(line: &[u8], scratch: &[u8]) -> bool {
    !scratch.is_empty()
        && line.windows(scratch.len()).enumerate().any(|(at, part)| {
            let before = at.checked_sub(1).and_then(|at| line.get(at));
            let after = line.get(at + scratch.len());
            part == scratch
                && before.is_none_or(|b| b.is_ascii_whitespace() || b"\"'`(".contains(b))
                && after.is_none_or(|b| *b == b'/' || b.is_ascii_whitespace() || b"\"'`,)".contains(b))
        })
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
mod tests;
