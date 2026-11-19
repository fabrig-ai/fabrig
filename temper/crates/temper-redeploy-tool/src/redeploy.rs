//! The redeploy's steps (temper/docs/design/redeploy.md, 4, 7).
use crate::{
    description::Description,
    hosts::{self, Hosts, Output, Part, fill, words},
    openings::{self, Event, Wait},
    record::{Check, Grace, Journal, Outage, Previous, seconds},
    stage::{binaries, part_name, valid_commit, version},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Engine,
    Workers,
    Together,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Done,
    Refused,
    Failed,
}

pub fn redeploy(
    description: &Description,
    commit: &str,
    mode: Mode,
    journal: &mut Journal,
    hosts: &mut dyn Hosts,
) -> Outcome {
    let mut run = Run {
        description,
        commit,
        journal,
        hosts,
        stopped: Vec::new(),
        swapped: Vec::new(),
        window: None,
        outage_from: None,
    };
    let result = run.perform(mode);
    let (outcome, line) = match result {
        Ok(()) => (Outcome::Done, "done".into()),
        Err(mut failure) => {
            if let Err(journal) = run.note(&format!("{} failed: {}", failure.step, failure.why)) {
                failure.why = format!("{}; journal: {}", failure.why, journal.why);
            }
            let mut outcome = Outcome::Failed;
            if run.swapped.is_empty() && !failure.recovery {
                match run.restore() {
                    Ok(()) => outcome = Outcome::Refused,
                    Err(recovery) => {
                        failure.why = format!("{}; recovery at {}: {}", failure.why, recovery.step, recovery.why);
                    }
                }
            }
            let line = match outcome {
                Outcome::Refused => format!("refused: {}: {}", failure.step, failure.why),
                Outcome::Failed => format!(
                    "failed at {}: {}; swapped: {}; backup: {}; previous: {}",
                    failure.step,
                    failure.why,
                    run.swapped.iter().map(|part| part_name(description, *part)).collect::<Vec<_>>().join(", "),
                    run.journal.record.backup.as_deref().unwrap_or("none"),
                    run.journal
                        .record
                        .previous
                        .iter()
                        .map(|part| format!("{}={}", part.part, part.commit))
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                Outcome::Done => unreachable!("a failed step cannot be done"),
            };
            (outcome, line)
        }
    };
    let line = line.lines().collect::<Vec<_>>().join(" / ");
    if run.note(&line).is_err() { Outcome::Failed } else { outcome }
}

struct Failure {
    step: String,
    why: String,
    recovery: bool,
}
impl Failure {
    fn at(step: &str, why: impl Into<String>) -> Self {
        Self { step: step.into(), why: why.into(), recovery: false }
    }
}
// The tool's verdict notes (temper/docs/design/redeploy.md, 6.3).
const SERVING_VERDICT_UNSUPPORTED: &str =
    "serving verdict: a build without upgrade --workers is redeployed with --together";
const SERVING_VERDICT_REFUSED: &str = "serving verdict: refused by the staged build; its lines above say why";
const REHEARSAL_LINKED: &str = "rehearsal linked ";
const REHEARSAL_PREFIX: &str = "temper-engine upgrade: rehearsal linked ";
const REHEARSAL_UNKNOWN: &str = "rehearsal linked: unknown; the staged build may copy the whole checkpoint at the stop";

#[derive(Clone, Copy)]
enum CheckStep {
    Check,
    ServingVerdict,
    Verdict,
}

struct Run<'a> {
    description: &'a Description,
    commit: &'a str,
    journal: &'a mut Journal,
    hosts: &'a mut dyn Hosts,
    stopped: Vec<Part>,
    /// A part enters this list immediately after its first successful binary swap.
    swapped: Vec<Part>,
    window: Option<u64>,
    outage_from: Option<u64>,
}
impl Run<'_> {
    fn note(&mut self, text: &str) -> Result<(), Failure> {
        for line in text.lines() {
            self.journal
                .line(self.hosts.now(), &format!("temper-redeploy run: {line}"))
                .map_err(|why| Failure::at("journal", why))?;
        }
        Ok(())
    }
    fn command(&mut self, part: Part, step: &str, line: &str) -> Result<(), Failure> {
        let output = self.hosts.run(part, line);
        success(step, &output)
    }
    fn workers(&self) -> Vec<Part> {
        (0..self.description.workers.len()).map(Part::Worker).collect()
    }
    fn perform(&mut self, mode: Mode) -> Result<(), Failure> {
        self.before()?;
        self.checkpoint_for_check()?;
        let may_stay = self.check(CheckStep::Check)?;
        if !may_stay || mode == Mode::Together {
            return self.together();
        }
        match mode {
            Mode::Together => unreachable!("together has its own path"),
            Mode::Engine | Mode::Workers => {
                if !self.check(CheckStep::ServingVerdict)? {
                    return self.together();
                }
                self.stop(Part::Engine)?;
                if !self.check(CheckStep::Verdict)? {
                    self.restore()?;
                    return self.together();
                }
                self.swap(Part::Engine)?;
                self.restart(&[Part::Engine], &self.workers(), self.window)?;
                match mode {
                    Mode::Workers => self.in_turn(),
                    Mode::Engine => Ok(()),
                    Mode::Together => unreachable!("together has its own path"),
                }
            }
        }
    }
    fn before(&mut self) -> Result<(), Failure> {
        for part in std::iter::once(Part::Engine).chain(self.workers()) {
            let (directory, _, names) = binaries(self.description, part);
            let mut previous = None;
            for name in names {
                let step = format!("{} version {name}", part_name(self.description, part));
                let output = self.hosts.run(part, &words(&[&format!("{directory}/{name}"), "--version"]));
                success(&step, &output)?;
                let commit = installed(&output.stdout, name)
                    .ok_or_else(|| Failure::at(&step, format!("invalid version: {:?}", output.stdout)))?;
                if previous.as_ref().is_some_and(|previous| previous != &commit) {
                    return Err(Failure::at(&step, "installed binaries disagree"));
                }
                previous = Some(commit);
            }
            let commit = previous.expect("each part has binaries");
            let name = part_name(self.description, part);
            self.journal.record.previous.push(Previous { part: name.into(), commit: commit.clone() });
            self.note(&format!("{name} before {commit}"))?;
        }
        let grace =
            temper_service_core::startup::fixed_worker().graces.expect("temper fixes worker graces").grace.as_millis();
        for worker in &self.description.workers {
            self.journal.record.graces.push(Grace { worker: worker.name.clone(), grace_ms: grace });
            self.note(&format!("worker {} grace {grace} ms", worker.name))?;
        }
        Ok(())
    }
    fn remember_stop(&mut self, part: Part, at: u64) {
        if !self.stopped.contains(&part) {
            self.stopped.push(part);
        }
        if part == Part::Engine {
            self.window = Some(at);
        }
    }
    fn stop(&mut self, part: Part) -> Result<(), Failure> {
        let at = self.hosts.now();
        // A failed stop can have stopped the service; starting it again is safe before a swap.
        self.remember_stop(part, at);
        let line = match part {
            Part::Engine => &self.description.engine.stop,
            Part::Worker(index) => &self.description.workers[index].stop,
        };
        let name = part_name(self.description, part);
        self.command(part, &format!("{name} stop"), line)?;
        self.note(&format!("{name} stopped at {at}"))?;
        if part == Part::Engine {
            if self.journal.record.told.is_none() {
                self.read_told(at)?;
            }
            self.backup()?;
        }
        Ok(())
    }
    fn read_told(&mut self, stop: u64) -> Result<(), Failure> {
        let told =
            openings::read_told(self.description, stop, self.hosts).map_err(|why| Failure::at("who was told", why))?;
        self.journal.record.told = Some(told.clone());
        self.journal.save().map_err(|why| Failure::at("journal", why))?;
        for told in told {
            let line = match told.wait_ms {
                Some(wait) => {
                    format!("worker {} told: restart wait {} s", told.worker, seconds(u128::from(wait) * 1_000_000))
                }
                None => format!("worker {} keeps its grace: not told", told.worker),
            };
            self.note(&line)?;
        }
        Ok(())
    }
    fn checkpoint_for_check(&mut self) -> Result<(), Failure> {
        let text = hosts::ctl(self.description, &["deployment", "checkpoint"], self.hosts)
            .map_err(|why| Failure::at("check checkpoint", why))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|why| Failure::at("check checkpoint", why.to_string()))?;
        let name = value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .filter(|name| checkpoint_name(name))
            .ok_or_else(|| Failure::at("check checkpoint", "invalid checkpoint name"))?;
        self.journal.record.checkpoint = Some(name.into());
        self.note(&format!("check checkpoint {name}"))
    }
    fn check(&mut self, kind: CheckStep) -> Result<bool, Failure> {
        let binary = format!("{}/{}/temper-engine", self.description.engine.builds, self.commit);
        let (step, template, checkpoint) = match kind {
            CheckStep::Check => ("check", &self.description.engine.check, &self.journal.record.checkpoint),
            CheckStep::ServingVerdict => {
                ("serving verdict", &self.description.engine.verdict, &self.journal.record.checkpoint)
            }
            CheckStep::Verdict => ("verdict", &self.description.engine.verdict, &self.journal.record.backup),
        };
        let checkpoint = checkpoint.as_deref().expect("checkpoint taken before its check or verdict");
        let output = self.hosts.run(Part::Engine, &fill(template, &[("binary", &binary), ("checkpoint", checkpoint)]));
        let lines: Vec<String> = output.stdout.lines().chain(output.stderr.lines()).map(str::to_owned).collect();
        let check = Some(Check { code: output.code, lines: lines.clone() });
        match kind {
            CheckStep::Check => self.journal.record.check = check,
            CheckStep::ServingVerdict => {}
            CheckStep::Verdict => self.journal.record.verdict = check,
        }
        for line in &lines {
            self.note(line)?;
        }
        // Save even a check or verdict with no output, including a command which could not run.
        self.note(&format!("{step} exit {:?}", output.code))?;
        if let Err(failure) = success(step, &output) {
            if let CheckStep::ServingVerdict = kind {
                self.note(if lines.iter().any(|line| line.contains("no command upgrade --workers")) {
                    SERVING_VERDICT_UNSUPPORTED
                } else {
                    SERVING_VERDICT_REFUSED
                })?;
            }
            return Err(failure);
        }
        match kind {
            CheckStep::Check => {}
            CheckStep::ServingVerdict | CheckStep::Verdict => {
                let note = match rehearsal_counts(&lines) {
                    Some((linked, copied)) => format!("{step} {REHEARSAL_LINKED}{linked} files, copied {copied}"),
                    None => format!("{step} {REHEARSAL_UNKNOWN}"),
                };
                self.note(&note)?;
            }
        }
        let may_stay =
            lines.iter().rev().find_map(|line| line.rsplit_once("workers may stay: ").map(|(_, verdict)| verdict))
                == Some("yes");
        self.note(&format!(
            "{step} workers may stay: {}",
            if may_stay { "yes" } else { "no or unknown; every part together" }
        ))?;
        Ok(may_stay)
    }
    fn backup(&mut self) -> Result<(), Failure> {
        let binary = format!("{}/temper-engine", self.description.engine.binaries);
        let output = self.hosts.run(Part::Engine, &fill(&self.description.engine.checkpoint, &[("binary", &binary)]));
        for line in output.stdout.lines().chain(output.stderr.lines()) {
            self.note(line)?;
        }
        success("stop checkpoint", &output)?;
        let name = output
            .stdout
            .lines()
            .chain(output.stderr.lines())
            .find_map(|line| {
                let name = line.strip_prefix("temper-engine checkpoint: ")?.split_whitespace().next()?;
                checkpoint_name(name).then_some(name)
            })
            .ok_or_else(|| Failure::at("stop checkpoint", "invalid checkpoint name"))?;
        self.journal.record.backup = Some(name.into());
        self.note(&format!("stop checkpoint {name}; backup"))
    }
    fn swap(&mut self, part: Part) -> Result<(), Failure> {
        let name = part_name(self.description, part);
        let previous = self
            .journal
            .record
            .previous
            .iter()
            .find(|previous| previous.part == name)
            .expect("before read every part")
            .commit
            .clone();
        let (installed, builds, names) = binaries(self.description, part);
        let template = match part {
            Part::Engine => &self.description.engine.swap,
            Part::Worker(index) => &self.description.workers[index].swap,
        };
        for binary in names {
            let step = format!("{name} swap {binary}");
            let to = format!("{installed}/{binary}");
            let saved = format!("{builds}/{previous}/{binary}");
            let answer = self.hosts.run(part, &words(&[&saved, "--version"]));
            if answer.code != Some(0) || !version(&answer.stdout, binary, &previous) {
                let copy = format!(
                    "{} && {}",
                    words(&["mkdir", "-p", &format!("{builds}/{previous}")]),
                    words(&["cp", &to, &saved])
                );
                self.command(part, &format!("{name} preserve {binary}"), &copy)?;
                self.note(&format!("{name} preserved {binary} at {previous}"))?;
            }
            let from = format!("{builds}/{}/{binary}", self.commit);
            self.command(part, &step, &fill(template, &[("from", &from), ("to", &to)]))?;
            if !self.swapped.contains(&part) {
                self.swapped.push(part);
            }
            self.note(&format!("{name} swapped {binary}: {previous} -> {}", self.commit))?;
        }
        Ok(())
    }
    fn start(&mut self, part: Part) -> Result<(), Failure> {
        let line = match part {
            Part::Engine => &self.description.engine.start,
            Part::Worker(index) => &self.description.workers[index].start,
        };
        let name = part_name(self.description, part);
        self.command(part, &format!("{name} start"), line)?;
        self.stopped.retain(|stopped| *stopped != part);
        let at = self.hosts.now();
        self.note(&format!("{name} started at {at}"))
    }
    fn wait(&mut self, wait: &mut Wait) -> Result<(), Failure> {
        loop {
            let at = self.hosts.now();
            if at > wait.deadline {
                return Err(Failure::at("openings", wait.expired(at).expect("the bound has passed")));
            }
            let events = wait.poll(self.journal, self.hosts).map_err(|why| Failure::at("openings", why))?;
            for (at, line, event) in events {
                match &event {
                    Event::Opened { worker, .. } => self.note(&format!("worker {worker} opened at {at}: {line}"))?,
                    Event::Refused { worker, .. } => self.note(&format!("worker {worker} refused at {at}: {line}"))?,
                }
            }
            if let Some(why) = wait.refusal() {
                return Err(Failure::at("openings", why));
            }
            if wait.complete() {
                return Ok(());
            }
            if let Some(why) = wait.expired(self.hosts.now()) {
                return Err(Failure::at("openings", why));
            }
            self.hosts.sleep(openings::SECOND);
        }
    }
    fn prepare(&mut self, awaited: &[Part], window: Option<u64>) -> Result<Wait, Failure> {
        Wait::new(self.description, awaited, window, self.journal, self.hosts)
            .map_err(|why| Failure::at("openings", why))
    }
    fn restart(&mut self, parts: &[Part], awaited: &[Part], window: Option<u64>) -> Result<Wait, Failure> {
        let mut wait = self.prepare(awaited, window)?;
        for part in parts {
            self.start(*part)?;
        }
        self.wait(&mut wait)?;
        Ok(wait)
    }
    fn restore(&mut self) -> Result<(), Failure> {
        self.restore_stopped().map_err(|mut failure| {
            failure.recovery = true;
            failure
        })
    }
    fn restore_stopped(&mut self) -> Result<(), Failure> {
        if self.stopped.is_empty() {
            return Ok(());
        }
        let mut parts = self.stopped.clone();
        parts.sort(); // Engine first, then workers in the description's order.
        let awaited = if parts.contains(&Part::Engine) { self.workers() } else { parts.clone() };
        let window = if parts.contains(&Part::Engine) && parts.len() == 1 { self.window } else { None };
        let wait = self.prepare(&awaited, window);
        let mut failure = None;
        for part in parts {
            if let Err(why) = self.start(part) {
                failure = Some(why);
            }
        }
        if let Some(why) = failure {
            return Err(why);
        }
        let mut wait = wait?;
        self.wait(&mut wait)?;
        self.finish_outage(wait.last_opening())
    }
    fn finish_outage(&mut self, to: u64) -> Result<(), Failure> {
        if let Some(from) = self.outage_from.take() {
            self.journal.record.outages.push(Outage { what: "every part".into(), from, to });
            self.note(&format!("every part restored at {to}"))?;
        }
        Ok(())
    }
    fn together(&mut self) -> Result<(), Failure> {
        let from = self.hosts.now();
        self.outage_from = Some(from);
        let lines: Vec<_> = self
            .description
            .workers
            .iter()
            .enumerate()
            .map(|(index, worker)| (Part::Worker(index), worker.stop.clone()))
            .collect();
        let outputs = self.hosts.run_each(&lines);
        // All commands have ended, even if an earlier answer or journal write failed.
        for (part, _) in &lines {
            self.remember_stop(*part, from);
        }
        let mut failure = None;
        for ((part, _), output) in lines.iter().zip(&outputs) {
            let name = part_name(self.description, *part);
            match success(&format!("{name} stop"), output) {
                Ok(()) => self.note(&format!("{name} stopped at {from}"))?,
                Err(why) => failure = Some(why),
            }
        }
        if let Some(why) = failure {
            return Err(why);
        }
        if outputs.len() != lines.len() {
            return Err(Failure::at("workers stop", "missing command answers"));
        }
        self.stop(Part::Engine)?;
        for part in std::iter::once(Part::Engine).chain(self.workers()) {
            self.swap(part)?;
        }
        let parts: Vec<_> = std::iter::once(Part::Engine).chain(self.workers()).collect();
        let wait = self.restart(&parts, &self.workers(), None)?;
        self.finish_outage(wait.last_opening())
    }
    fn in_turn(&mut self) -> Result<(), Failure> {
        for part in self.workers() {
            let from = self.hosts.now();
            self.stop(part)?;
            self.swap(part)?;
            let wait = self.restart(&[part], &[part], None)?;
            let name = part_name(self.description, part);
            loop {
                if self.hosts.now() > wait.deadline {
                    return Err(Failure::at(
                        &format!("{name} logins"),
                        format!("{name} did not log in within its wait"),
                    ));
                }
                let text = hosts::ctl(self.description, &["account", "list"], self.hosts)
                    .map_err(|why| Failure::at(&format!("{name} logins"), why))?;
                if self.hosts.now() > wait.deadline {
                    return Err(Failure::at(
                        &format!("{name} logins"),
                        format!("{name} did not log in within its wait"),
                    ));
                }
                let value: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|why| Failure::at(&format!("{name} logins"), why.to_string()))?;
                let ready = value
                    .pointer("/accounts/deployment_facts/program_hosts")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|hosts| {
                        hosts.iter().any(|host| {
                            host.get("name").and_then(serde_json::Value::as_str) == Some(name)
                                && host.get("connected").and_then(serde_json::Value::as_bool) == Some(true)
                                && host.get("awaiting_logins").and_then(serde_json::Value::as_bool) == Some(false)
                        })
                    });
                if ready {
                    let to = self.hosts.now();
                    self.journal.record.outages.push(Outage { what: name.into(), from, to });
                    self.note(&format!("worker {name} logins at {to}"))?;
                    break;
                }
                if self.hosts.now() >= wait.deadline {
                    return Err(Failure::at(
                        &format!("{name} logins"),
                        format!("{name} did not log in within its wait"),
                    ));
                }
                self.hosts.sleep(openings::SECOND);
            }
        }
        Ok(())
    }
}
fn rehearsal_counts(lines: &[String]) -> Option<(&str, &str)> {
    let counts = lines.iter().find_map(|line| line.strip_prefix(REHEARSAL_PREFIX))?;
    let (linked, copied) = counts.split_once(" files, copied ")?;
    if linked.is_empty()
        || copied.is_empty()
        || !linked.bytes().all(|byte| byte.is_ascii_digit())
        || !copied.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    Some((linked, copied))
}

fn success(step: &str, output: &Output) -> Result<(), Failure> {
    if output.code == Some(0) {
        Ok(())
    } else {
        Err(Failure::at(step, format!("exit {:?}: {} {}", output.code, output.stderr.trim(), output.stdout.trim())))
    }
}
fn checkpoint_name(name: &str) -> bool {
    jig_protocol_store::CheckpointName::parse(name.as_bytes()).is_some()
}
fn installed(text: &str, name: &str) -> Option<String> {
    let commit = text.split_whitespace().nth(2)?.strip_prefix('(')?.strip_suffix(')')?;
    (valid_commit(commit) && version(text, name, commit)).then(|| commit.into())
}
