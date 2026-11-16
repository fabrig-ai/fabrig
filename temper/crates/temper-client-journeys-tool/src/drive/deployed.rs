//! Deployment lifecycle over injected lines, exits and time (testing/qa/deployment.md, 4–7).
use super::{Action, Event, Exit, Machine, Output, Phase, Process, STEP, deployment};
use crate::line::Deployment;
use skein_lib::{Duration, Time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Preparing,
    Starting(Process),
    Signing,
    Ready,
    Reading,
    Stopping(bool),
    Writing,
    Classifying,
}
pub(super) struct State {
    stage: Stage,
    live: Vec<Process>,
    pub values: Option<deployment::Values>,
    pub failed: Option<String>,
    clients: bool,
    workers: bool,
    link: bool,
    first: Option<Box<[u8]>>,
    patch: Option<deployment::RenderedFile>,
    pub patch_until: Option<Time>,
}
impl State {
    pub fn new() -> Self {
        Self {
            stage: Stage::Preparing,
            live: vec![],
            values: None,
            failed: None,
            clients: false,
            workers: false,
            link: false,
            first: None,
            patch: None,
            patch_until: None,
        }
    }
}
impl Process {
    pub(super) fn deployment_log(self) -> Option<&'static str> {
        match self {
            Self::Forgejo => Some("forgejo.log"),
            Self::Model => Some("model.log"),
            Self::Engine => Some("engine.log"),
            Self::Worker => Some("worker.log"),
            Self::Box | Self::SignIn | Self::Ctl => None,
        }
    }
}
impl Machine {
    fn deployed(&mut self) -> &mut State {
        self.deployment.as_mut().expect("deployment session")
    }
    pub(super) fn deployment_event(&mut self, event: &Event, now: Time, out: &mut Vec<Action>) -> bool {
        if self.deployment.is_none() {
            return false;
        }
        if self.phase == Phase::Finished {
            return true;
        }
        let stage = self.deployed().stage;
        match event {
            Event::DeploymentPrepared(result) if stage == Stage::Preparing => match result {
                Ok(values) => {
                    self.deployed().values = Some(values.clone());
                    self.deployment_start(Process::Forgejo, now, out);
                }
                Err(why) => self.finish(Exit::own(why.clone()), out),
            },
            Event::ProcessLine { process, output, line } => self.deployment_line(*process, *output, line, now, out),
            Event::Exited { process, code, .. } if process.deployment_log().is_some() => {
                self.deployment_exited(*process, *code, now, out);
            }
            Event::Exited { process: Process::SignIn, code, .. } if stage == Stage::Signing => {
                out.push(Action::RemoveLink);
                if self.until.is_some_and(|at| now >= at) {
                    self.deployment_failed("ctl sign-in deadline passed".into(), now, out);
                } else if *code == 0 {
                    self.deployment_ready(out);
                } else {
                    self.deployment_failed(format!("ctl sign-in exited {code}"), now, out);
                }
            }
            Event::DeploymentPatch(result) if stage == Stage::Reading => match result {
                Ok(file) => {
                    self.deployed().patch = Some(file.clone());
                    self.deployment_stop(true, now, out);
                }
                Err(why) => {
                    self.deployed().patch_until = None;
                    self.answer(Some(why.clone()), out);
                    self.deployed().stage = Stage::Ready;
                }
            },
            Event::DeploymentWritten(result) if stage == Stage::Writing => match result {
                Ok(()) => self.deployment_start(Process::Engine, now, out),
                Err(why) => self.deployment_failed(why.clone(), now, out),
            },
            Event::Deadline if self.phase == Phase::Deployment => {
                if self.until.is_none_or(|at| now < at) && self.deployed().patch_until.is_none_or(|at| now < at) {
                    return true;
                }
                if let Stage::Stopping(patch) = stage {
                    if let Some(process) = self.deployed().live.pop() {
                        out.push(Action::StopDeployment { process, force: true });
                        self.deployed().failed.get_or_insert(format!("{process:?} did not stop within its bound"));
                    }
                    if patch {
                        self.deployment_failed("patch stop deadline passed".into(), now, out);
                    } else {
                        self.deployment_stop(false, now, out);
                    }
                } else {
                    self.deployment_failed(format!("{stage:?}: deadline passed"), now, out);
                }
            }
            Event::DeploymentLogs(logs) if stage == Stage::Classifying => self.deployment_classify(logs, out),
            Event::OwnFailure(_) => {
                for process in self.deployed().live.clone().into_iter().rev() {
                    out.push(Action::StopDeployment { process, force: true });
                }
                return false;
            }
            _ => return false,
        }
        true
    }
    fn deployment_start(&mut self, process: Process, now: Time, out: &mut Vec<Action>) {
        let [forge, model, engine, worker] = self.plan.deployment_programs();
        let (program, seconds) = match process {
            Process::Forgejo => (forge, 60),
            Process::Model => (model, 60),
            Process::Engine => (engine, 120),
            Process::Worker => (worker, 120),
            _ => unreachable!("deployment process"),
        };
        if process == Process::Engine {
            let state = self.deployed();
            state.clients = false;
            state.workers = false;
            state.link = false;
        }
        self.deployed().live.push(process);
        self.deployed().stage = Stage::Starting(process);
        self.wait(Phase::Deployment, now, Duration::from_secs(seconds));
        out.push(Action::Start { process, program });
    }
    fn deployment_line(&mut self, process: Process, output: Output, line: &[u8], now: Time, out: &mut Vec<Action>) {
        if !(process == Process::Engine && output == Output::Stdout) {
            out.push(Action::DeploymentLog { process, line: line.into() });
        }
        if !self.deployed().live.contains(&process) {
            return;
        }
        if self.deployed().stage != Stage::Starting(process) {
            return;
        }
        if self.next_deadline().is_some_and(|at| now >= at) {
            self.deployment_failed(format!("{process:?}: ready deadline passed"), now, out);
            return;
        }
        let ready = match process {
            Process::Forgejo | Process::Model => output == Output::Stdout && line.starts_with(b"listening "),
            Process::Worker => output == Output::Stderr && line.starts_with(b"linked "),
            Process::Engine => {
                let state = self.deployed();
                if output == Output::Stderr {
                    state.clients |= line.starts_with(b"listening clients ");
                    state.workers |= line.starts_with(b"listening workers ");
                }
                if output == Output::Stdout && line.starts_with(b"http://") && !state.link {
                    state.link = true;
                    if state.patch_until.is_none() {
                        state.first = Some(line.into());
                    }
                }
                state.clients && state.workers && state.link
            }
            _ => false,
        };
        if !ready {
            return;
        }
        match process {
            Process::Forgejo => self.deployment_start(Process::Model, now, out),
            Process::Model => self.deployment_start(Process::Engine, now, out),
            Process::Engine => self.deployment_start(Process::Worker, now, out),
            Process::Worker => {
                if self.deployed().patch_until.is_some() {
                    if let Some(crate::line::Record::Deployment(Deployment::Patch { document, .. })) =
                        self.current.as_ref().map(super::Line::record)
                    {
                        self.shown.push(format!("patched {}", document.name()));
                    }
                    self.deployed().patch_until = None;
                    self.answer(None, out);
                    self.deployed().stage = Stage::Ready;
                } else {
                    let first = self.deployed().first.take().expect("engine's first link");
                    self.deployed().stage = Stage::Signing;
                    self.wait(Phase::Deployment, now, STEP);
                    out.push(Action::SignInFile { link: first, program: self.plan.deployment_sign_in() });
                }
            }
            _ => {}
        }
    }
    fn deployment_ready(&mut self, out: &mut Vec<Action>) {
        self.deployed().stage = Stage::Ready;
        self.phase = Phase::Ready;
        self.until = None;
        out.push(Action::Ready);
    }
    pub(super) fn deployment_patch(&mut self, patch: &Deployment, now: Time, out: &mut Vec<Action>) {
        let Deployment::Patch { document, path } = patch;
        self.deployed().stage = Stage::Reading;
        self.deployed().patch_until = Some(now.saturating_add(Duration::from_secs(300)));
        self.wait(Phase::Deployment, now, Duration::from_secs(300));
        out.push(Action::ReadDeploymentPatch { document: *document, path: path.clone() });
    }
    fn deployment_exited(&mut self, process: Process, code: i32, now: Time, out: &mut Vec<Action>) {
        if !self.deployed().live.contains(&process) {
            return;
        }
        let stage = self.deployed().stage;
        let expected = matches!(stage, Stage::Stopping(_)) && self.deployed().live.last() == Some(&process);
        self.deployed().live.retain(|live| *live != process);
        if let Stage::Stopping(patch) = stage
            && expected
        {
            if code != 0 {
                self.deployed().failed.get_or_insert(format!("{process:?} exited {code}"));
            }
            if patch && code != 0 {
                self.deployment_failed(format!("{process:?} exited {code} during patch"), now, out);
            } else {
                self.deployment_stop(patch, now, out);
            }
        } else {
            self.deployment_failed(format!("{process:?} exited {code}"), now, out);
        }
    }
    fn deployment_failed(&mut self, why: String, now: Time, out: &mut Vec<Action>) {
        self.deployed().failed.get_or_insert(why.clone());
        self.deployed().first = None;
        self.deployed().patch_until = None;
        out.extend([Action::StopCtl, Action::RemoveLink]);
        if self.forge.take().is_some() {
            out.push(Action::CancelForge);
        }
        if self.current.is_some() {
            self.answer(Some(why), out);
            self.deployed().stage = Stage::Ready;
        } else if self.phase == Phase::Ready {
            self.deployed().stage = Stage::Ready;
        } else {
            self.deployment_stop(false, now, out);
        }
    }
    pub(super) fn deployment_end(&mut self, now: Time, out: &mut Vec<Action>) {
        if matches!(self.deployed().stage, Stage::Stopping(false) | Stage::Classifying) || self.phase == Phase::Finished
        {
            return;
        }
        self.current = None;
        self.deployed().patch_until = None;
        if self.forge.take().is_some() {
            out.push(Action::CancelForge);
        }
        out.extend([Action::StopCtl, Action::RemoveLink]);
        self.deployment_stop(false, now, out);
    }
    fn deployment_stop(&mut self, patch: bool, now: Time, out: &mut Vec<Action>) {
        if patch && self.deployed().live.last() == Some(&Process::Model) {
            self.deployed().stage = Stage::Writing;
            out.push(Action::WriteDeploymentPatch(self.deployed().patch.take().expect("validated patch")));
            return;
        }
        if let Some(process) = self.deployed().live.last().copied() {
            let seconds = if matches!(process, Process::Worker | Process::Engine) { 60 } else { 30 };
            self.deployed().stage = Stage::Stopping(patch);
            self.wait(Phase::Deployment, now, Duration::from_secs(seconds));
            out.push(Action::StopDeployment { process, force: false });
        } else if self.deployed().failed.is_some() {
            self.deployed().stage = Stage::Classifying;
            self.until = None;
            out.push(Action::DeploymentLogs);
        } else {
            self.finish(Exit { status: 0, shown: vec!["ended".into()], error: None }, out);
        }
    }
    fn deployment_classify(&mut self, logs: &[Vec<Box<[u8]>>], out: &mut Vec<Action>) {
        let scratch = self.plan.options.scratch.as_os_str().as_encoded_bytes();
        let refusal = logs.iter().enumerate().find_map(|(index, lines)| {
            lines.iter().find(|line| {
                let line = super::strip_log_time(line);
                (index == 2 && line.starts_with(b"temper-engine serve: ") && super::names_scratch(line, scratch))
                    || (index == 0 && super::forge_state_refused(line, scratch))
            })
        });
        let exit = if let Some(line) = refusal {
            Exit {
                status: 3,
                shown: vec![format!("refused: {}", String::from_utf8_lossy(super::strip_log_time(line)))],
                error: None,
            }
        } else {
            let mut shown = vec![format!(
                "deployment failed: {}",
                self.deployed().failed.as_deref().unwrap_or("the deployment exited").replace(['\r', '\n'], " ")
            )];
            for lines in logs {
                for line in lines.iter().skip(lines.len().saturating_sub(20)) {
                    shown.push(format!("| {}", String::from_utf8_lossy(line).trim_end_matches('\n')));
                }
            }
            Exit { status: 4, shown, error: None }
        };
        self.finish(exit, out);
    }
}
