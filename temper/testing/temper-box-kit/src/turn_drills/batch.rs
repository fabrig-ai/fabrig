//! Template goals made ready together (domain/landing.md, 5 and 9).
use super::{Drill, Judge, Line, Process, Script, Signal, Stage, Value, binary, calls, document, json, write};
use skein_fake_llm_domain::api::{Finish, Turn};

#[derive(Debug, Default)]
pub(super) struct Batch {
    phase: Phase,
    latch_index: usize,
    latch: u64,
    members: Vec<u64>,
    next: usize,
    task_read: bool,
    panel: Option<Value>,
    run: Option<u64>,
    pub(super) departed: bool,
    pub(super) green_prefix: bool,
}
#[derive(Debug, Default)]
enum Phase {
    #[default]
    Start,
    Reviews,
    Amended,
    Gates,
    Proof,
    Running,
    Moving,
    Inspect,
    Recover,
    FinalPanel,
}

impl Batch {
    pub(super) fn starting(&self) -> bool {
        matches!(self.phase, Phase::Start)
    }
}
pub(super) fn scripts(scripts: &mut Vec<Script>) {
    let review = scripts
        .iter_mut()
        .find(|script| script.cue.as_ref() == b"Review this change at the named head")
        .expect("review");
    review.turns[0].lines =
        [vec![super::super::batch_landing::hold(b"drill-batch-reviews")], review.turns[0].lines.to_vec()]
            .concat()
            .into();
    scripts.push(Script {
        cue: b"@drill-latch".as_slice().into(),
        turns: Box::new([Turn {
            lines: Box::new([
                super::super::batch_landing::hold(b"drill-batch-ready"),
                Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"form":"report","text":"Ready together","fields":{}}"#.as_slice().into(),
                },
            ]),
            finish: Finish::ToolCalls,
            tokens: 20,
        }]),
    });
}
fn created(ended: &super::super::Ended) -> Result<u64, String> {
    let text = String::from_utf8_lossy(&ended.stdout);
    text.strip_prefix("done: task ")
        .and_then(|text| text.split(',').next())
        .and_then(|number| number.trim().parse().ok())
        .ok_or_else(|| format!("created task: {text}"))
}
fn release(procs: &mut [Process], name: &[u8]) -> Result<(), String> {
    for proc in procs {
        if let Process::Peer(peer) = proc {
            peer.release(name).map_err(|why| format!("batch release: {why:?}"))?;
        }
    }
    Ok(())
}
impl Judge<'_> {
    pub(super) fn begin_batch(&mut self, procs: &mut [Process]) {
        let path = self.root.join("temper.json");
        let mut value = document(&path);
        value["application"]["landing"]["projects"][0]["batch"] = self.drill.members().into();
        value["application"]["limits"]["overrides"]["forge.landing_members"] = self.drill.members().into();
        value["application"]["limits"]["overrides"]["forge.resources_per_task"] = (self.drill.members() + 1).into();
        write(&path, &value);
        binary(procs, 0).signal(Signal::Kill);
        self.stage = Stage::Restart;
    }
    pub(super) fn batch(&mut self, procs: &mut [Process]) -> Result<(), String> {
        match self.batch.phase {
            Phase::Start => self.batch_start(procs),
            Phase::Reviews => self.batch_reviews(procs)?,
            Phase::Amended => {
                if calls(procs).foreground_idle() {
                    self.batch.phase = Phase::Gates;
                }
            }
            Phase::Gates => self.batch_gates(procs)?,
            Phase::Proof => self.batch_proof(procs)?,
            Phase::Moving => {
                release(procs, b"drill-moving")?;
                let panel = self.batch.panel.as_ref().ok_or("panel")?;
                let task = panel["members"][1]["change"].as_u64().ok_or("moving change")?;
                let original = panel["members"][1]["head"].clone();
                if let Some(change) = self.read(procs, &["change", "show", &task.to_string()])
                    && change["head"] != original
                {
                    self.barrier.release(self.batch.run.ok_or("held proof")?);
                    self.batch.phase = Phase::Running;
                }
            }
            Phase::Running => {
                release(procs, b"drill-batch-reviews")?;
                if matches!(self.drill, Drill::Breaking | Drill::Conflict | Drill::Moving)
                    && let Some(run) = self.barrier.waiting(None)
                {
                    eprintln!("batch {:?}: release gate {run}", self.drill);
                    self.barrier.release(run);
                }
                let task = self.batch.panel.as_ref().ok_or("panel")?["task"].as_u64().ok_or("landing")?;
                let Some(panel) = self.read(procs, &["landing", "show", &task.to_string()]) else { return Ok(()) };
                if matches!(self.drill, Drill::Breaking | Drill::Conflict | Drill::Moving) {
                    self.departure(procs, &panel)?;
                    if let Some(run) = self.barrier.waiting(None) {
                        self.barrier.release(run);
                    }
                }
                if !panel["hold"].is_null() {
                    return Err(format!("batch held: {panel}"));
                }
                if panel["phase"] == "landed" {
                    self.batch.panel = Some(panel);
                    self.batch.phase = if self.drill == Drill::BatchFour { Phase::Inspect } else { Phase::Recover };
                    self.batch.next = 0;
                }
            }
            Phase::Inspect => self.batch_inspect(procs)?,
            Phase::Recover => self.batch_recover(procs)?,
            Phase::FinalPanel => {
                let task = self.batch.panel.as_ref().ok_or("panel")?["task"].as_u64().ok_or("task")?;
                let Some(panel) = self.read(procs, &["landing", "show", &task.to_string()]) else { return Ok(()) };
                self.batch.panel = Some(panel);
                self.batch.phase = Phase::Inspect;
                self.batch.next = 0;
            }
        }
        Ok(())
    }
    fn batch_start(&mut self, procs: &mut [Process]) {
        if self.read(procs, &["settings", "show"]).is_none() {
            return;
        }
        self.batch.latch_index = calls(procs).ends().len() + self.drill.members();
        for number in 1..=self.drill.members() {
            calls(procs).push(super::super::ctl::Step::Run(
                super::super::ctl::Call::new(&[
                    "goal",
                    "start",
                    "--project",
                    "1",
                    "--template",
                    "2",
                    &format!("@drill-goal-{number} Edit one line"),
                    "--key",
                    &format!("drill-goal-{number}"),
                ])
                .in_directory("private"),
            ));
        }
        calls(procs).push(super::super::ctl::Step::Run(
            super::super::ctl::Call::new(&[
                "goal",
                "start",
                "--project",
                "1",
                "--template",
                "2",
                "@drill-latch Hold readiness until all four reviews agree",
                "--key",
                "drill-latch",
            ])
            .in_directory("private"),
        ));
        self.batch.phase = Phase::Reviews;
    }
    fn batch_reviews(&mut self, procs: &mut [Process]) -> Result<(), String> {
        if !calls(procs).foreground_idle() {
            return Ok(());
        }
        self.batch.latch = created(calls(procs).ends().get(self.batch.latch_index).ok_or("latch call")?)?;
        let pulls = self.forge.repository_observation("acme", "widget", "pulls?state=open&limit=100");
        let mut members = Vec::new();
        for number in 1..=self.drill.members() {
            let Some(pull) =
                pulls.as_array().ok_or("pulls")?.iter().find(|pull| pull["title"] == format!("Drill change {number}"))
            else {
                return Ok(());
            };
            let branch = pull["head"]["ref"].as_str().ok_or("member branch")?;
            members.push(
                branch
                    .rsplit('/')
                    .next()
                    .and_then(|name| name.strip_prefix('c'))
                    .and_then(|number| number.parse().ok())
                    .ok_or("change task")?,
            );
        }
        self.batch.members = members;
        for task in &self.batch.members {
            calls(procs).push(super::super::ctl::Step::Run(
                super::super::ctl::Call::new(&[
                    "task",
                    "amend",
                    &task.to_string(),
                    "--rev",
                    "0",
                    "--dependency",
                    &self.batch.latch.to_string(),
                    "Become ready with the batch",
                    "--key",
                    &format!("drill-keep-{task}"),
                ])
                .in_directory("private"),
            ));
        }
        eprintln!("batch {:?}: held members {:?}, latch {}", self.drill, self.batch.members, self.batch.latch);
        self.batch.phase = Phase::Amended;
        Ok(())
    }
    fn batch_gates(&mut self, procs: &mut [Process]) -> Result<(), String> {
        release(procs, b"drill-batch-reviews")?;
        if let Some(task) = self.batch.members.get(self.batch.next).copied() {
            if self.batch.task_read {
                let Some(task) = self.read(procs, &["task", "show", &task.to_string()]) else { return Ok(()) };
                if task["chip"]["phase"] == "idle" {
                    self.batch.next += 1;
                    self.batch.task_read = false;
                }
            } else {
                let Some(change) = self.read(procs, &["change", "show", &task.to_string()]) else { return Ok(()) };
                if change["gates"][0]["verdict"] == "approved" {
                    self.batch.task_read = true;
                }
            }
        } else {
            release(procs, b"drill-batch-ready")?;
            self.batch.phase = Phase::Proof;
        }
        Ok(())
    }
    fn batch_proof(&mut self, procs: &mut [Process]) -> Result<(), String> {
        release(procs, b"drill-batch-ready")?;
        let task = if let Some(panel) = &self.batch.panel {
            panel["task"].as_u64().ok_or("landing")?
        } else {
            let Some(list) = self.read(procs, &["landing", "list", "--project", "1", "--ended"]) else { return Ok(()) };
            let rows = list["landings"].as_array().ok_or("landings")?;
            if rows.is_empty() {
                return Ok(());
            }
            if rows.len() != 1 || rows[0]["changes"] != json!(self.batch.members) {
                return Err(format!("four ready together in queue order: {list}; expected {:?}", self.batch.members));
            }
            self.batch.panel = Some(rows[0].clone());
            rows[0]["task"].as_u64().ok_or("landing")?
        };
        let Some(panel) = self.read(procs, &["landing", "show", &task.to_string()]) else { return Ok(()) };
        if panel["phase"] != "proving" {
            return Ok(());
        }
        let Some(run) = self.barrier.waiting(None) else { return Ok(()) };
        if panel["members"].as_array().ok_or("members")?.len() != self.drill.members()
            || panel["preparation"] != 1
            || panel["round"]["number"] != 1
        {
            return Err(format!("one four-member candidate: {panel}"));
        }
        eprintln!("batch {:?}: task {task}, {} members, proof run {run}", self.drill, self.drill.members());
        self.batch.panel = Some(panel);
        self.batch.run = Some(run);
        if self.drill == Drill::Moving {
            self.batch.phase = Phase::Moving;
        } else {
            self.barrier.release(run);
            self.batch.phase = Phase::Running;
        }
        Ok(())
    }
    fn batch_recover(&mut self, procs: &mut [Process]) -> Result<(), String> {
        release(procs, b"drill-batch-reviews")?;
        release(procs, b"drill-repair")?;
        if let Some(run) = self.barrier.waiting(None) {
            self.barrier.release(run);
        }
        let Some(list) = self.read(procs, &["landing", "list", "--project", "1", "--ended"]) else { return Ok(()) };
        let rows = list["landings"].as_array().ok_or("landings")?;
        if rows.len() > 2 {
            return Err(format!("one departing member rejoins once: {list}"));
        }
        if rows.len() != 2 || rows.iter().any(|row| row["phase"] != "landed") {
            return Ok(());
        }
        if !self.batch.departed || !self.batch.green_prefix {
            return Err("a named departure after a green prefix".into());
        }
        let next = rows
            .iter()
            .find(|row| row["task"] != self.batch.panel.as_ref().expect("first panel")["task"])
            .ok_or("next landing")?
            .clone();
        self.batch.panel = Some(next);
        self.batch.phase = Phase::FinalPanel;
        Ok(())
    }
    fn batch_inspect(&mut self, procs: &mut [Process]) -> Result<(), String> {
        let panel = self.batch.panel.as_ref().ok_or("landed panel")?.clone();
        if let Some(task) = self.batch.members.get(self.batch.next).copied() {
            let Some(change) = self.read(procs, &["change", "show", &task.to_string()]) else { return Ok(()) };
            if change["step"] != "landed" {
                return Ok(());
            }
            let repaired = self.drill == Drill::Breaking && self.batch.next == 2;
            let resolved = self.drill == Drill::Conflict && self.batch.next == 1;
            if change["repairs"]["used"] != u64::from(repaired)
                || change["resolutions"]["used"] != u64::from(resolved)
                || !change["hold"].is_null()
            {
                return Err(format!("batch change: {change}"));
            }
            let number = change["pull"]["number"].as_u64().ok_or("pull")?;
            let pull = self.forge.repository_observation("acme", "widget", &format!("pulls/{number}"));
            if pull["merged"] != true {
                return Err(format!("member pull not merged: {number}"));
            }
            self.batch.next += 1;
        } else {
            let runs = self.forge.repository_observation("acme", "widget", "actions/runs?limit=100");
            let runs: Vec<_> = runs["workflow_runs"]
                .as_array()
                .ok_or("runs")?
                .iter()
                .filter(|run| run["workflow_id"] == "gate.yaml")
                .collect();
            let expected = if self.drill == Drill::Breaking {
                5
            } else if self.drill == Drill::Conflict {
                2
            } else if self.drill == Drill::Moving {
                3
            } else {
                1
            };
            if runs.len() != expected
                || (self.drill == Drill::BatchFour
                    && (runs[0]["id"] != self.batch.run.ok_or("gate")? || runs[0]["status"] != "success"))
            {
                return Err(format!("one successful gate: {runs:?}"));
            }
            let candidate = panel["members"].as_array().ok_or("members")?.last().ok_or("top")?["candidate"]
                .as_str()
                .ok_or("candidate")?;
            if self.forge.branch_head(b"main").as_deref() != Some(candidate.get(..40).ok_or("SHA")?.as_bytes()) {
                return Err("main at candidate".into());
            }
            eprintln!("batch {:?}: all {} member pulls merged, main at candidate", self.drill, self.drill.members());
            self.stop(procs);
        }
        Ok(())
    }
}
