//! The session's place and its ports by use (testing/qa/kinds.md, 3–5).
use super::{Action, Event, Exit, Output, Phase, Plan, Process, STEP, deployment, values};
use crate::line::{BoxValue, Deployment, Record};
use skein_lib::{Duration, Time};
use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

/// The fixed nine-position list, named by what binds each position (kinds.md, 4).
#[derive(Clone, Debug)]
pub(super) struct Ports {
    pub clients: u16,
    pub owned: u16,
    pub forge: u16,
    pub llm: u16,
    pub webhooks: u16,
    pub github: u16,
    pub oauth: u16,
    pub model: u16,
    pub forgejo: u16,
    given: Option<String>,
}
impl Ports {
    pub fn new(ports: [u16; 9]) -> Self {
        let [clients, owned, forge, llm, webhooks, github, oauth, model, forgejo] = ports;
        Self { clients, owned, forge, llm, webhooks, github, oauth, model, forgejo, given: None }
    }
    pub fn parse(list: &str) -> Result<Self, String> {
        let mut ports = Self::new(super::options::parse_ports(list)?);
        // Keep the accepted spelling of --ports in the box's environment.
        ports.given = Some(list.into());
        Ok(ports)
    }
    pub fn array(&self) -> [u16; 9] {
        [
            self.clients,
            self.owned,
            self.forge,
            self.llm,
            self.webhooks,
            self.github,
            self.oauth,
            self.model,
            self.forgejo,
        ]
    }
    pub fn list(&self) -> String {
        self.given.clone().unwrap_or_else(|| self.array().iter().map(u16::to_string).collect::<Vec<_>>().join(","))
    }
}

pub(super) enum Place {
    Box(BoxState),
    Deployment(DeploymentState),
}
pub(super) struct BoxState {
    stage: BoxStage,
    until: Option<Time>,
    exited: bool,
    failed: Option<String>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum BoxStage {
    Starting,
    Running,
    Stopping,
    Classifying,
}
#[derive(Clone, Copy)]
pub(super) struct Activity {
    pub phase: Phase,
    pub busy: bool,
    pub tui_running: bool,
}
impl Activity {
    fn starting(self) -> bool {
        matches!(self.phase, Phase::SignIn | Phase::Spent | Phase::FreshLink | Phase::Fresh | Phase::Root | Phase::Main)
    }
}
type ForgeHands = (Box<[u8]>, PathBuf);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    Preparing,
    Starting(Process),
    Idle,
    PatchTui,
    Reading,
    Stopping(bool),
    Writing,
    Classifying,
}
pub(super) struct DeploymentState {
    pub stage: Stage,
    pub live: Vec<Process>,
    pub values: Option<deployment::Values>,
    pub failed: Option<String>,
    pub clients: bool,
    pub workers: bool,
    pub link: bool,
    pub first: Option<Box<[u8]>>,
    pub patch: Option<deployment::RenderedFile>,
    pub until: Option<Time>,
    pub patch_until: Option<Time>,
}

/// The sign-in's existing failure words and timing, supplied with the first link.
#[derive(Clone, Copy)]
pub(super) struct SignIn {
    pub strict_deadline: bool,
    pub stderr: bool,
    pub deadline: &'static str,
}
pub(super) struct First {
    pub link: Box<[u8]>,
    pub origin: Box<[u8]>,
    pub sign_in: SignIn,
}
impl First {
    pub(super) fn new(link: Box<[u8]>, validate_origin: bool, sign_in: SignIn) -> Result<Self, String> {
        let origin = if validate_origin { origin(&link).ok_or("invalid first link")? } else { Box::new([]) };
        Ok(Self { link, origin, sign_in })
    }
}

/// What the place gives the session; client steps are carried out only by Machine.
pub(super) enum Progress {
    None,
    Waiting,
    First(First),
    Link { link: Box<[u8]>, replace: bool },
    Failed(String),
    LostLine(String),
    End,
    Answer(Option<String>),
    PatchTui,
    Patched,
    Finished(Exit),
}
impl Place {
    pub fn start(plan: &Plan, now: Time) -> (Self, Action) {
        if plan.options.deployment.is_some() {
            (Self::Deployment(DeploymentState::new(now)), Action::PrepareDeployment)
        } else {
            (
                Self::Box(BoxState {
                    stage: BoxStage::Starting,
                    until: Some(now.saturating_add(STEP)),
                    exited: false,
                    failed: None,
                }),
                Action::Start { process: Process::Box, program: plan.box_program() },
            )
        }
    }
    pub fn deadline(&self) -> Option<Time> {
        match self {
            Self::Box(state) => state.until,
            Self::Deployment(state) => state.until.into_iter().chain(state.patch_until).min(),
        }
    }
    pub fn patch_deadline(&self) -> Option<Time> {
        match self {
            Self::Box(_) => None,
            Self::Deployment(state) => state.patch_until,
        }
    }
    pub fn line_finished(&mut self) {
        if let Self::Deployment(state) = self {
            state.patch_until = None;
        }
    }
    pub fn exited_line(&self) -> Option<&'static str> {
        match self {
            Self::Box(state) if state.exited => Some("the box exited"),
            Self::Deployment(state) if state.failed.is_some() => Some("the deployment exited"),
            Self::Box(_) | Self::Deployment(_) => None,
        }
    }
    pub fn refuse_line(&self, record: &Record, ctl: bool) -> Option<&'static str> {
        match self {
            Self::Box(_) if matches!(record, Record::Deployment(_)) => Some("a box takes no deployment line"),
            Self::Box(_) if ctl && matches!(record, Record::Web(_) | Record::Tui(_)) => {
                Some("this session takes ctl and forge lines alone")
            }
            Self::Deployment(_) if ctl && matches!(record, Record::Web(_) | Record::Tui(_)) => {
                Some("this session takes ctl, forge and deployment lines alone")
            }
            Self::Box(_) | Self::Deployment(_) => None,
        }
    }
    // A deployment's exited lines precede parsing and face-line admission; a box's follow them.
    pub fn early_exit(&self) -> Option<&'static str> {
        match self {
            Self::Deployment(state) if state.failed.is_some() => self.exited_line(),
            Self::Box(_) | Self::Deployment(_) => None,
        }
    }
    pub fn ctl_values(&self, args: &[Box<[u8]>], log: Option<&[u8]>) -> Result<Option<Vec<Box<[u8]>>>, String> {
        if args.iter().any(|arg| BoxValue::parse(arg).is_some()) && self.needs_log(log) {
            return Ok(None);
        }
        args.iter()
            .map(|arg| match BoxValue::parse(arg) {
                Some(name) => self.value(name, log),
                None => Ok(arg.clone()),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }
    fn needs_log(&self, log: Option<&[u8]>) -> bool {
        matches!(self, Self::Box(_)) && log.is_none()
    }
    fn value(&self, name: BoxValue, log: Option<&[u8]>) -> Result<Box<[u8]>, String> {
        match self {
            Self::Box(_) => values::read(log.expect("box log supplied"), name),
            Self::Deployment(state) => Ok(state.values.as_ref().expect("prepared deployment").value(name).into()),
        }
    }
    pub fn read_values(plan: &Plan) -> Action {
        Action::ReadBoxLog { path: plan.options.scratch.join("session/box.log") }
    }
    pub fn forge_hands(&self, log: Option<&[u8]>) -> Result<Option<ForgeHands>, String> {
        if self.needs_log(log) {
            return Ok(None);
        }
        let git = self.value(BoxValue::Git, log)?;
        let admin = self.value(BoxValue::Admin, log)?;
        Ok(Some(([git.as_ref(), b"/api/v1"].concat().into(), PathBuf::from(std::ffi::OsStr::from_bytes(&admin)))))
    }
    pub fn patch(&mut self, patch: &Deployment, now: Time, out: &mut Vec<Action>) {
        let Self::Deployment(state) = self else { unreachable!("admitted deployment line") };
        let Deployment::Patch { document, path } = patch;
        state.stage = Stage::Reading;
        state.patch_until = Some(now.saturating_add(Duration::from_secs(300)));
        state.until = state.patch_until;
        out.push(Action::ReadDeploymentPatch { document: *document, path: path.clone() });
    }
    pub fn event(
        &mut self,
        event: &Event,
        activity: Activity,
        plan: &Plan,
        now: Time,
        out: &mut Vec<Action>,
    ) -> Option<Progress> {
        match self {
            Self::Box(state) => state.event(event, activity, plan, now, out),
            Self::Deployment(state) => state.event(event, activity, plan, now, out),
        }
    }
    pub fn failed(&mut self, why: String, busy: bool, phase: Phase, out: &mut Vec<Action>) -> Progress {
        match self {
            Self::Box(state) => {
                state.failed.get_or_insert(why);
                Progress::End
            }
            Self::Deployment(state) => {
                state.failed.get_or_insert(why.clone());
                state.first = None;
                state.patch_until = None;
                out.extend([Action::StopCtl, Action::RemoveLink]);
                if busy || phase == Phase::Ready {
                    state.until = None;
                    state.stage = Stage::Idle;
                    if busy { Progress::Answer(Some(why)) } else { Progress::None }
                } else {
                    Progress::End
                }
            }
        }
    }
    pub fn ending(&self) -> bool {
        match self {
            Self::Box(state) => matches!(state.stage, BoxStage::Stopping | BoxStage::Classifying),
            Self::Deployment(state) => matches!(state.stage, Stage::Stopping(false) | Stage::Classifying),
        }
    }
    pub fn begin_end(&mut self, out: &mut Vec<Action>) {
        match self {
            Self::Box(state) => {
                state.stage = BoxStage::Running;
                state.until = None;
            }
            Self::Deployment(state) => {
                state.patch_until = None;
                state.until = None;
                state.stage = Stage::Idle;
                out.extend([Action::StopCtl, Action::RemoveLink]);
            }
        }
    }
    pub fn stop(&mut self, now: Time, out: &mut Vec<Action>) -> Progress {
        match self {
            Self::Box(state) if state.exited => {
                state.classifying(state.failed.clone().unwrap_or_else(|| "the box exited".into()), out);
                Progress::Waiting
            }
            Self::Box(state) => {
                state.stage = BoxStage::Stopping;
                state.until = Some(now.saturating_add(STEP));
                out.push(Action::Stop { process: Process::Box, force: false });
                Progress::Waiting
            }
            Self::Deployment(state) => state.stop(false, now, out),
        }
    }
    pub fn browser_close_expired(&mut self, out: &mut Vec<Action>) {
        let why = "browser did not close within 60 s".into();
        match self {
            Self::Box(state) => {
                state.failed.get_or_insert(why);
                out.push(Action::Stop { process: Process::Box, force: true });
            }
            Self::Deployment(state) => {
                state.failed.get_or_insert(why);
            }
        }
    }
    pub fn own_failure(&self, out: &mut Vec<Action>) -> Action {
        match self {
            Self::Box(_) => Action::Stop { process: Process::Box, force: true },
            Self::Deployment(state) => {
                for process in state.live.iter().rev() {
                    out.push(Action::Stop { process: *process, force: true });
                }
                Action::ForceBrowser
            }
        }
    }
}
impl BoxState {
    fn event(
        &mut self,
        event: &Event,
        activity: Activity,
        plan: &Plan,
        now: Time,
        out: &mut Vec<Action>,
    ) -> Option<Progress> {
        Some(match event {
            Event::BoxLine { output, line } | Event::ProcessLine { process: Process::Box, output, line } => {
                self.line(*output, line, activity, out)
            }
            Event::Exited { process: Process::Box, code, stderr, .. } => {
                self.exited = true;
                if self.stage == BoxStage::Stopping {
                    self.until = None;
                    if *code == 0 && self.failed.is_none() {
                        Progress::Finished(Exit { status: 0, shown: vec!["ended".into()], error: None })
                    } else {
                        self.classifying(format!("box exited {code}"), out);
                        Progress::Waiting
                    }
                } else if activity.phase == Phase::Ready {
                    self.failed.get_or_insert_with(|| format!("box exited {code}"));
                    Progress::None
                } else if !matches!(activity.phase, Phase::Finished | Phase::ClosingBrowser)
                    && self.stage != BoxStage::Classifying
                {
                    if activity.busy {
                        self.failed = Some(format!("box exited {code}"));
                        Progress::LostLine("the box exited".into())
                    } else {
                        Progress::Failed(before_ready(*code, stderr))
                    }
                } else {
                    Progress::None
                }
            }
            Event::Deadline if self.until.is_some_and(|at| now >= at) => {
                if self.stage == BoxStage::Stopping {
                    out.push(Action::Stop { process: Process::Box, force: true });
                    self.classifying("box did not stop within 60 s".into(), out);
                    Progress::Waiting
                } else {
                    Progress::Failed("Link: deadline passed".into())
                }
            }
            Event::Logs { serve, forges, box_tail } if self.stage == BoxStage::Classifying => {
                let scratch = plan.options.scratch.as_os_str().as_encoded_bytes();
                let refusal = serve
                    .iter()
                    .map(|line| strip_log_time(line))
                    .find(|line| line.starts_with(b"temper serve: ") && names_scratch(line, scratch))
                    .or_else(|| {
                        forges.iter().map(|line| strip_log_time(line)).find(|line| forge_state_refused(line, scratch))
                    });
                Progress::Finished(classify(
                    "box",
                    self.failed.as_deref().unwrap_or("the box exited"),
                    refusal,
                    &[box_tail, serve],
                ))
            }
            _ => return None,
        })
    }
    fn line(&mut self, output: Output, line: &[u8], activity: Activity, out: &mut Vec<Action>) -> Progress {
        if output == Output::Stderr {
            out.push(Action::Log { line: line.into() });
            if line.windows(11).any(|s| s == b"; giving up") || line.windows(13).any(|s| s == b"; restarting ") {
                return if self.stage == BoxStage::Starting || activity.starting() {
                    Progress::Failed(String::from_utf8_lossy(line).into_owned())
                } else {
                    Progress::None
                };
            }
        }
        if output != Output::Stdout || !line.starts_with(b"http://") {
            return Progress::None;
        }
        let address = line.strip_suffix(b"\n").unwrap_or(line).into();
        if self.stage == BoxStage::Starting {
            self.stage = BoxStage::Running;
            self.until = None;
            match First::new(
                address,
                true,
                SignIn { strict_deadline: false, stderr: true, deadline: "SignIn: deadline passed" },
            ) {
                Ok(first) => Progress::First(first),
                Err(why) => Progress::Failed(why),
            }
        } else {
            Progress::Link { link: address, replace: true }
        }
    }
    fn classifying(&mut self, why: String, out: &mut Vec<Action>) {
        self.failed.get_or_insert(why);
        self.until = None;
        self.stage = BoxStage::Classifying;
        out.push(Action::Classify);
    }
}

pub(super) fn classify(kind: &str, why: &str, refusal: Option<&[u8]>, logs: &[&[Box<[u8]>]]) -> Exit {
    if let Some(line) = refusal {
        return Exit { status: 3, shown: vec![format!("refused: {}", String::from_utf8_lossy(line))], error: None };
    }
    let mut shown = vec![format!("{kind} failed: {}", why.replace(['\r', '\n'], " "))];
    for lines in logs {
        for line in lines.iter().skip(lines.len().saturating_sub(20)) {
            shown.push(format!("| {}", String::from_utf8_lossy(line).trim_end_matches('\n')));
        }
    }
    Exit { status: 4, shown, error: None }
}

pub(super) fn origin(link: &[u8]) -> Option<Box<[u8]>> {
    let rest = link.strip_prefix(b"http://")?;
    let end = rest.iter().position(|b| b"/?#".contains(b)).unwrap_or(rest.len());
    if end == 0 { None } else { Some(link.get(..end.checked_add(7)?)?.into()) }
}
pub(super) fn strip_log_time(line: &[u8]) -> &[u8] {
    if line.get(10) == Some(&b'T') && line.get(19..21) == Some(b"Z ") { line.get(21..).unwrap_or(line) } else { line }
}
pub(super) fn forge_state_refused(line: &[u8], scratch: &[u8]) -> bool {
    let Some(rest) = line.strip_prefix(b"temper-box fake forge: state ") else { return false };
    let Some(at) = rest.windows(b" refused: ".len()).position(|part| part == b" refused: ") else { return false };
    let path = Path::new(std::ffi::OsStr::from_bytes(&rest[..at]));
    path.starts_with(Path::new(std::ffi::OsStr::from_bytes(scratch)))
        && path != Path::new(std::ffi::OsStr::from_bytes(scratch))
        && !path.components().any(|part| part == std::path::Component::ParentDir)
}
pub(super) fn names_scratch(line: &[u8], scratch: &[u8]) -> bool {
    !scratch.is_empty()
        && line.windows(scratch.len()).enumerate().any(|(at, part)| {
            let before = at.checked_sub(1).and_then(|at| line.get(at));
            let after = line.get(at + scratch.len());
            part == scratch
                && before.is_none_or(|b| b.is_ascii_whitespace() || b"\"'`(".contains(b))
                && after.is_none_or(|b| *b == b'/' || b.is_ascii_whitespace() || b"\"'`,)".contains(b))
        })
}
/// A box's exit before its ready line, with the last of what it said that no
/// log line took: a start that failed says why there (client/drive.md, 2.5).
pub(super) fn before_ready(code: i32, stderr: &[u8]) -> String {
    let words = String::from_utf8_lossy(stderr);
    let words = words.trim();
    if words.is_empty() {
        return format!("box exited {code} before ready");
    }
    let cut = words.char_indices().rev().nth(511).map_or(0, |(at, _)| at);
    format!("box exited {code} before ready: {}", &words[cut..])
}
