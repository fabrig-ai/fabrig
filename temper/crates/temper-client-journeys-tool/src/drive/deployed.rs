//! Deployment-only preparation, processes and patch (testing/qa/deployment.md, 4–7).
use super::place::{Activity, DeploymentState, First, Progress, SignIn, Stage, classify};
use super::{Action, Event, Exit, Output, Phase, Plan, Process, STEP};
use skein_lib::{Duration, Time};

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
impl DeploymentState {
    pub fn new(now: Time) -> Self {
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
            worker: None,
            until: Some(now.saturating_add(STEP)),
            patch_until: None,
        }
    }
    pub fn event(
        &mut self,
        event: &Event,
        activity: Activity,
        plan: &Plan,
        now: Time,
        out: &mut Vec<Action>,
    ) -> Option<Progress> {
        if activity.phase == Phase::Finished {
            return Some(Progress::None);
        }
        Some(match event {
            Event::DeploymentPrepared(result) if self.stage == Stage::Preparing => match result {
                Ok(values) => {
                    self.values = Some(values.clone());
                    self.start(Process::Forgejo, plan, now, out)
                }
                Err(why) => Progress::Finished(Exit::own(why.clone())),
            },
            Event::DeploymentWorker(result) if self.stage == Stage::WorkerFile => match result {
                Ok(worker) => {
                    self.worker = Some(worker.clone());
                    self.worker_command(
                        Stage::WorkerList,
                        plan.ctl(
                            &[b"worker".as_slice().into(), b"list".as_slice().into(), b"--json".as_slice().into()],
                            super::Input::Null,
                        ),
                        now,
                        out,
                    )
                }
                Err(why) => Progress::Failed(why.clone()),
            },
            Event::Exited { process: Process::Ctl, code, stdout, stderr }
                if matches!(self.stage, Stage::WorkerList | Stage::WorkerAdd) =>
            {
                self.worker_exited(*code, (stdout, stderr), plan, now, out)
            }
            Event::ProcessLine { process, output, line } => {
                self.line((*process, *output, line), activity.phase, plan, now, out)
            }
            Event::Exited { process, code, .. } if process.deployment_log().is_some() => {
                self.exited(*process, *code, plan, now, out)
            }
            Event::DeploymentPatch(result) if self.stage == Stage::Reading => match result {
                Ok(file) => {
                    self.patch = Some(file.clone());
                    self.restart(activity.tui_running, plan, now, out)
                }
                Err(why) => {
                    self.patch_until = None;
                    self.until = None;
                    self.stage = Stage::Idle;
                    Progress::Answer(Some(why.clone()))
                }
            },
            Event::TuiStopped | Event::TuiFailure(_) if self.stage == Stage::PatchTui => {
                self.stop(true, plan, now, out)
            }
            Event::Deadline if self.stage == Stage::PatchTui && self.until.is_some_and(|at| now >= at) => {
                out.push(Action::StopTui);
                self.stop(true, plan, now, out)
            }
            Event::DeploymentWritten(result) if self.stage == Stage::Writing => match result {
                Ok(()) => self.start(Process::Engine, plan, now, out),
                Err(why) => Progress::Failed(why.clone()),
            },
            Event::Deadline if activity.phase == Phase::Place => {
                if self.until.is_none_or(|at| now < at) && self.patch_until.is_none_or(|at| now < at) {
                    return Some(Progress::None);
                }
                if let Stage::Stopping(patch) = self.stage {
                    if let Some(process) = self.live.pop() {
                        out.push(Action::Stop { process, force: true });
                        self.failed.get_or_insert(format!("{process:?} did not stop within its bound"));
                    }
                    if patch {
                        Progress::Failed("patch stop deadline passed".into())
                    } else {
                        self.stop(false, plan, now, out)
                    }
                } else {
                    Progress::Failed(format!("{:?}: deadline passed", self.stage))
                }
            }
            Event::DeploymentLogs(logs) if self.stage == Stage::Classifying => {
                let scratch = plan.options.scratch.as_os_str().as_encoded_bytes();
                let refusal = logs.iter().enumerate().find_map(|(index, lines)| {
                    lines.iter().map(|line| super::place::strip_log_time(line)).find(|line| {
                        (index == 2
                            && line.starts_with(b"temper-engine serve: ")
                            && super::place::names_scratch(line, scratch))
                            || (index == 0 && super::place::forge_state_refused(line, scratch))
                    })
                });
                Progress::Finished(classify(
                    "deployment",
                    self.failed.as_deref().unwrap_or("the deployment exited"),
                    refusal,
                    &logs.iter().map(Vec::as_slice).collect::<Vec<_>>(),
                ))
            }
            _ => return None,
        })
    }
    fn worker_exited(
        &mut self,
        code: i32,
        output: (&[u8], &[u8]),
        plan: &Plan,
        now: Time,
        out: &mut Vec<Action>,
    ) -> Progress {
        if self.until.is_some_and(|at| now >= at) {
            return Progress::Failed("worker admission deadline passed".into());
        }
        if code != 0 {
            return Progress::Failed(format!(
                "worker admission {:?} exited {code}: {}",
                self.stage,
                String::from_utf8_lossy(output.1.get(..4096).unwrap_or(output.1))
            ));
        }
        let worker = self.worker.as_ref().expect("read worker");
        if self.stage == Stage::WorkerList {
            match worker.listed(output.0) {
                Ok(true) => {}
                Ok(false) => return self.worker_command(Stage::WorkerAdd, worker.add(plan), now, out),
                Err(why) => return Progress::Failed(why),
            }
        }
        self.worker = None;
        self.start(Process::Worker, plan, now, out)
    }
    fn worker_command(&mut self, stage: Stage, program: super::Program, now: Time, out: &mut Vec<Action>) -> Progress {
        self.stage = stage;
        self.until = Some(now.saturating_add(super::STEP));
        out.push(Action::Start { process: Process::Ctl, program });
        Progress::Waiting
    }
    fn start(&mut self, process: Process, plan: &Plan, now: Time, out: &mut Vec<Action>) -> Progress {
        let [forge, model, engine, worker] = plan.deployment_programs();
        let (program, seconds) = match process {
            Process::Forgejo => (forge, 60),
            Process::Model => (model, 60),
            Process::Engine => (engine, 120),
            Process::Worker => (worker, 120),
            _ => unreachable!("deployment process"),
        };
        if process == Process::Engine {
            self.clients = false;
            self.workers = false;
            self.link = false;
        }
        self.live.push(process);
        self.stage = Stage::Starting(process);
        self.until = Some(now.saturating_add(Duration::from_secs(seconds)));
        out.push(Action::Start { process, program });
        Progress::Waiting
    }
    fn line(
        &mut self,
        line: (Process, Output, &[u8]),
        phase: Phase,
        plan: &Plan,
        now: Time,
        out: &mut Vec<Action>,
    ) -> Progress {
        let (process, output, line) = line;
        if !(process == Process::Engine && output == Output::Stdout) {
            out.push(Action::DeploymentLog { process, line: line.into() });
        }
        if !self.live.contains(&process) {
            return Progress::None;
        }
        if process == Process::Engine && output == Output::Stdout && matches!(phase, Phase::Spent | Phase::FreshLink) {
            return if line.starts_with(b"http://") {
                Progress::Link { link: line.strip_suffix(b"\n").unwrap_or(line).into(), replace: false }
            } else {
                Progress::None
            };
        }
        if self.stage != Stage::Starting(process) {
            return Progress::None;
        }
        if self.until.into_iter().chain(self.patch_until).min().is_some_and(|at| now >= at) {
            return Progress::Failed(format!("{process:?}: ready deadline passed"));
        }
        let ready = match process {
            Process::Forgejo | Process::Model => output == Output::Stdout && line.starts_with(b"listening "),
            Process::Worker => output == Output::Stderr && line.starts_with(b"linked "),
            Process::Engine => {
                if output == Output::Stderr {
                    self.clients |= line.starts_with(b"listening clients ");
                    self.workers |= line.starts_with(b"listening workers ");
                }
                if output == Output::Stdout && line.starts_with(b"http://") && !self.link {
                    self.link = true;
                    if self.patch_until.is_none() {
                        self.first = Some(line.into());
                    }
                }
                self.clients && self.workers && self.link
            }
            _ => false,
        };
        if !ready {
            return Progress::None;
        }
        match process {
            Process::Forgejo => self.start(Process::Model, plan, now, out),
            Process::Model => self.start(Process::Engine, plan, now, out),
            Process::Engine if self.patch_until.is_some() => self.start(Process::Worker, plan, now, out),
            Process::Engine => {
                self.stage = Stage::Idle;
                self.until = None;
                let first = self.first.take().expect("engine's first link");
                match First::new(
                    first,
                    !plan.options.ctl,
                    SignIn { strict_deadline: true, stderr: false, deadline: "Signing: deadline passed" },
                ) {
                    Ok(first) => Progress::First(first),
                    Err(why) => Progress::Failed(why),
                }
            }
            Process::Worker => {
                self.stage = Stage::Idle;
                self.until = None;
                if self.patch_until.is_some() { Progress::Patched } else { Progress::Started }
            }
            _ => Progress::None,
        }
    }
    fn exited(&mut self, process: Process, code: i32, plan: &Plan, now: Time, out: &mut Vec<Action>) -> Progress {
        if !self.live.contains(&process) {
            return Progress::None;
        }
        let expected = matches!(self.stage, Stage::Stopping(_)) && self.live.last() == Some(&process);
        self.live.retain(|live| *live != process);
        if let Stage::Stopping(patch) = self.stage
            && expected
        {
            if code != 0 {
                self.failed.get_or_insert(format!("{process:?} exited {code}"));
            }
            if patch && code != 0 {
                Progress::Failed(format!("{process:?} exited {code} during patch"))
            } else {
                self.stop(patch, plan, now, out)
            }
        } else {
            Progress::Failed(format!("{process:?} exited {code}"))
        }
    }
    pub fn restart(&mut self, tui: bool, plan: &Plan, now: Time, out: &mut Vec<Action>) -> Progress {
        if tui {
            self.stage = Stage::PatchTui;
            self.until = Some(now.saturating_add(super::LOOK));
            Progress::PatchTui
        } else {
            self.stop(true, plan, now, out)
        }
    }
    pub fn stop(&mut self, patch: bool, plan: &Plan, now: Time, out: &mut Vec<Action>) -> Progress {
        if patch && self.live.last() == Some(&Process::Model) {
            if let Some(file) = self.patch.take() {
                self.stage = Stage::Writing;
                out.push(Action::WriteDeploymentPatch(file));
                return Progress::Waiting;
            }
            return self.start(Process::Engine, plan, now, out);
        }
        if let Some(process) = self.live.last().copied() {
            let seconds = if matches!(process, Process::Worker | Process::Engine) { 60 } else { 30 };
            self.stage = Stage::Stopping(patch);
            self.until = Some(now.saturating_add(Duration::from_secs(seconds)));
            out.push(Action::Stop { process, force: false });
            Progress::Waiting
        } else if self.failed.is_some() {
            self.stage = Stage::Classifying;
            self.until = None;
            out.push(Action::DeploymentLogs);
            Progress::Waiting
        } else {
            Progress::Finished(Exit { status: 0, shown: vec!["ended".into()], error: None })
        }
    }
}
