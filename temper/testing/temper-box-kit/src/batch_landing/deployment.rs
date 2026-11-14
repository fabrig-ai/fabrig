//! Deployment constraints and a departed member's later landing
//! (temper/docs/design/domain/landing.md, 3, 4.2 and 5.1;
//! temper/docs/design/protocol/configuration/settings.md, 1).
use super::{Judge, Phase, Process, Script, Story, call, candidate, hold, members, release};
use crate::{
    ctl,
    forgejo::{JobBarrier, Throwaway},
    turn_restart, write_private,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

// The copied ceiling has these executor kinds and fifteen forge grants.
const COPIED_CHARTERS: &[u64] = &[1, 2, 4, 5, 9, 11, 12, 13, 14, 15, 16];
const COPIED_PROCEDURES: &[u64] = &[2, 3, 4, 6];
const COPIED_JOBS: &[u64] = &[3, 4];
const COPIED_ROLES: &[u64] = &[0, 1, 2];
const COPIED_FORGE_KINDS: &[u64] = &[1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17];

fn copied_executor(value: &Value) -> bool {
    for (field, numbers) in
        [("charter", COPIED_CHARTERS), ("procedure", COPIED_PROCEDURES), ("job", COPIED_JOBS), ("role", COPIED_ROLES)]
    {
        if let Some(number) = value[field].as_u64() {
            return numbers.contains(&number);
        }
    }
    false
}

fn copied_grant(value: &Value, delete_branch: bool) -> bool {
    value["kind"].as_u64().is_some_and(|kind| COPIED_FORGE_KINDS.contains(&kind) && (delete_branch || kind != 15))
}

/// The stock copy retains its grants; a retained project can lack branch deletion.
#[derive(Clone, Copy)]
pub enum Policy {
    StockCopy,
    WithoutDeleteBranch,
}

/// Shipped capacity bounds and the retained project that lacks branch deletion.
pub fn configure(root: &Path, repository: u64) {
    configure_policy(root, repository, 1, Policy::WithoutDeleteBranch);
}

/// The owner adds only the prepare executor to the selected copied policy.
pub fn configure_policy(root: &Path, repository: u64, batch: u16, policy: Policy) {
    configure_standard(root, repository, batch);
    let delete_branch = match policy {
        Policy::StockCopy => true,
        Policy::WithoutDeleteBranch => false,
    };
    write_private(
        &root.join("copied-project.json"),
        &serde_json::to_vec(&json!({"delete_branch":delete_branch})).expect("policy option"),
    );
}

/// Shipped capacities, with the explicit member and branch bounds a larger batch needs.
pub fn configure_standard(root: &Path, repository: u64, batch: u16) {
    turn_restart::configure_batch(root, repository, batch);
    let path = root.join("temper.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    let shipped: Value = serde_json::from_slice(include_bytes!("../../../../crates/temper-init/template/temper.json"))
        .expect("shipped deployment");
    for part in ["engine", "application"] {
        value[part]["limits"] = shipped[part]["limits"].clone();
    }
    if batch > 2 {
        value["application"]["limits"]["overrides"]["forge.landing_members"] = batch.into();
        value["application"]["limits"]["overrides"]["forge.resources_per_task"] = (u32::from(batch) + 1).into();
    }
    // Delivery fallback is pacing: keep a fake-model walk within the process deadline.
    value["application"]["limits"]["overrides"]["forge.client.fallback_ms"] = 5000.into();
    value["workers"][0]["limits"] = shipped["workers"][0]["limits"].clone();
    value["workers"][0]["slots"] = shipped["workers"][0]["slots"].clone();
    let agent_path = root.join("agent.json");
    let mut agent: Value = serde_json::from_slice(&fs::read(&agent_path).expect("agent configuration")).expect("JSON");
    let shipped_agent: Value =
        serde_json::from_slice(include_bytes!("../../../../crates/temper-init/template/agent.json"))
            .expect("shipped agent");
    agent["profile"] = shipped_agent["profile"].clone();
    crate::configuration::turn_bounds::apply(&mut value, crate::configuration::turn_bounds::turn_body(&agent));
    write_private(&agent_path, &serde_json::to_vec(&agent).expect("agent JSON"));
    write_private(&path, &serde_json::to_vec(&value).expect("configuration JSON"));
}

pub(super) fn setup_count(root: &Path) -> usize {
    turn_restart::SETUP_CALLS + if root.join("copied-project.json").exists() { 7 } else { 0 }
}

pub(super) fn setup(root: &Path) -> Vec<ctl::Step> {
    let mut steps = turn_restart::setup(root);
    if !root.join("copied-project.json").exists() {
        return steps;
    }
    let option: Value =
        serde_json::from_slice(&fs::read(root.join("copied-project.json")).expect("policy option")).expect("JSON");
    let delete_branch = option["delete_branch"].as_bool().expect("branch-delete grant option");
    let base = steps.len();
    steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "--json"]).in_directory("private")));
    steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "policy", "1", "--json"]).in_directory("private")));
    let file = root.join("copied-project.json");
    steps.push(ctl::Step::Then(Box::new(move |calls| {
        let mut policy = ctl::json(&calls[base + 1])["value"].clone();
        let ceiling = &mut policy["ceiling"];
        ceiling["delegation"]["kinds"].as_array_mut().expect("executors").retain(copied_executor);
        ceiling["grants"].as_array_mut().expect("grants").retain(|grant| copied_grant(grant, delete_branch));
        policy["projections"].as_array_mut().expect("projections").retain(|grant| copied_grant(grant, delete_branch));
        for role in policy["roles"].as_array_mut().expect("roles") {
            role["authority"]["grants"]
                .as_array_mut()
                .expect("role grants")
                .retain(|grant| copied_grant(grant, delete_branch));
            role["authority"]["delegation"]["kinds"].as_array_mut().expect("role executors").retain(copied_executor);
        }
        write_private(
            &file,
            &serde_json::to_vec(&json!({"items":[{
                "setting":{"kind":"project-policy","project":1},"set":policy
            }]}))
            .expect("copied policy"),
        );
        ctl::settings_change(&calls[base], &file, "copy-project-policy").in_directory("private")
    })));
    steps.push(ctl::Step::Run(ctl::Call::new(&["project", "show", "1", "--json"]).in_directory("private")));
    steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "--json"]).in_directory("private")));
    let copied = root.join("copied-project.json");
    let file = root.join("prepare-executor.json");
    steps.push(ctl::Step::Then(Box::new(move |calls| {
        let copied: Value = serde_json::from_slice(&fs::read(&copied).expect("copied project")).expect("policy JSON");
        let mut policy = copied["items"][0]["set"].clone();
        assert_eq!(
            policy["ceiling"]["grants"].as_array().expect("grants").iter().any(|grant| grant["kind"] == 15),
            delete_branch,
            "adding the executor preserves the copied grants"
        );
        policy["ceiling"]["delegation"]["kinds"].as_array_mut().expect("executors").push(json!({"job":1}));
        write_private(
            &file,
            &serde_json::to_vec(&json!({"items":[{
                "setting":{"kind":"project-policy","project":1},"set":policy
            }]}))
            .expect("updated policy"),
        );
        let revision = ctl::newest(&calls[base + 4]).to_string();
        ctl::Call::new(&[
            "settings",
            "apply",
            file.to_str().expect("settings path"),
            "--against",
            &revision,
            "--running",
            "stays",
            "--key",
            "add-prepare-executor",
        ])
        .in_directory("private")
    })));
    steps.push(ctl::Step::Run(ctl::Call::new(&["project", "show", "1", "--json"]).in_directory("private")));
    steps
}

pub(super) fn repair_script(scripts: &mut Vec<Script>) {
    let mut read = call(b"read", &json!({"path":"batch-1.txt"}));
    read.lines = [vec![hold(b"deployment-repair")], read.lines.into_vec()].concat().into_boxed_slice();
    scripts.push(Script {
        cue: b"Repair the failed landing gate".as_slice().into(),
        turns: vec![
            read,
            call(b"write", &json!({"path":"batch-1.txt","content":"Repaired\n"})),
            call(b"finish", &json!({"form":"change","fields":{"title":"Repaired change","body":"Gate repaired."}})),
        ]
        .into_boxed_slice(),
    });
}

impl Judge<'_> {
    fn goal_starting(&mut self, procs: &mut [Process]) -> Result<bool, String> {
        if let Phase::StartGoal = self.phase {
            self.amends = super::calls(procs)?.ends().len();
            let now = skein_shell::Clock::new().now().wall;
            let due = skein_lib::Wall::from_nanos(
                now.as_nanos().saturating_add(skein_lib::Duration::from_secs(3600).as_nanos()),
            );
            let date = due.civil();
            let minute = due.as_secs() / 60;
            let deadline = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}Z",
                date.year,
                date.month,
                date.day,
                (minute / 60) % 24,
                minute % 60
            );
            super::calls(procs)?.push(ctl::Step::Run(
                ctl::Call::new(&[
                    "goal",
                    "start",
                    "--project",
                    "1",
                    "--template",
                    "4",
                    "@moving-goal Write the design",
                    "--budget",
                    "1000000",
                    "--deadline",
                    &deadline,
                    "--key",
                    "moving-goal",
                ])
                .in_directory("private"),
            ));
            self.phase = Phase::GoalGates;
            return Ok(true);
        }
        if let Phase::GoalGates = self.phase {
            let calls = super::calls(procs)?;
            let Some(ended) = calls.ends().get(self.amends) else { return Ok(true) };
            if ended.exit != Some(skein_io::kernel::Exit::Code(0)) {
                return Err(format!("goal start: {}", String::from_utf8_lossy(&ended.stderr)));
            }
            let change = match self.change(4) {
                Ok(change) => change,
                Err(why) if why.contains("\"error\":\"unknown\"") => return Ok(true),
                Err(why) => return Err(why),
            };
            release(procs, b"batch-reviews")?;
            if change["gates"][0]["verdict"] == "approved" {
                self.phase = Phase::Batch;
            }
            let goal = self.read(&["task", "show", "3"])?;
            if !goal["hold"].is_null() {
                return Err(format!("moving goal held: {}", goal["hold"]));
            }
            return Ok(true);
        }
        Ok(false)
    }
    pub(super) fn deployment_step(&mut self, procs: &mut [Process]) -> Result<(), String> {
        if self.goal_starting(procs)? {
            return Ok(());
        }
        let first = self.first.as_ref().ok_or("first proof")?;
        let task = first.panel["task"].as_u64().ok_or("landing task")?;
        let panel = self.landing(task)?;
        let owner = self.read(&["task", "show", &task.to_string()])?;
        if !owner["hold"].is_null() {
            return Err(format!("landing {task} held: {}", owner["hold"]));
        }
        let change_task = if matches!(self.story, Story::Conflict | Story::MidProof) { 4 } else { 3 };
        let change = self.change(change_task)?;
        self.observe(0, json!({"landing":panel,"change":change}));
        if let Phase::SingleLanded = self.phase {
            if self.landed(first)? {
                self.phase = Phase::Ending;
            }
            return Ok(());
        }
        if let Phase::Moved = self.phase {
            release(procs, b"goal-midproof")?;
            if change["head"] == super::sha(&members(&first.panel)?[0]["head"])? {
                return Ok(());
            }
            self.barrier.release(first.run);
            if owner["chip"]["phase"] != "ended" {
                return Ok(());
            }
            let left = &members(&panel)?[0]["left"];
            if left["head_moved"].is_null() || !members(&panel)?[0]["landed"].is_null() {
                return Err("the superseded member left because its head moved".into());
            }
            if super::sha(&left["head_moved"])? != change["head"].as_str().ok_or("new head")? {
                return Err("the leave records the delivered head".into());
            }
            if super::main(self.forge)? != self.base {
                return Err("superseded prefix never lands".into());
            }
            self.phase = Phase::Repaired;
        }
        if let Phase::Conflict = self.phase {
            release(procs, b"batch-reviews")?;
            if !self.landed(first)? || change["step"] != "resolving" {
                return Ok(());
            }
            if super::main(self.forge)? != candidate(&first.panel)? {
                return Err("only the first conflicting change landed".into());
            }
            release(procs, b"deployment-resolution")?;
            self.phase = Phase::Resolved;
        }
        if let Phase::Repair = self.phase {
            if members(&panel)?[0]["left"].is_null() {
                return Ok(());
            }
            release(procs, b"deployment-repair")?;
            if owner["chip"]["phase"] != "ended" {
                return Ok(());
            }
            if members(&panel)?[0]["left"].is_null() || !members(&panel)?[0]["landed"].is_null() {
                return Err("the failed member left without landing".into());
            }
            if super::main(self.forge)? != self.base {
                return Err("red never reached main".into());
            }
            release(procs, b"deployment-repair")?;
            self.phase = Phase::Repaired;
        }
        if matches!(self.phase, Phase::Repaired | Phase::Resolved) {
            release(procs, b"deployment-resolution")?;
            release(procs, b"deployment-repair")?;
            release(procs, b"batch-reviews")?;
            if self.second.is_none() {
                if let Some(proof) = self.proof(Some(first.run), &[change_task])? {
                    if matches!(self.story, Story::Repaired)
                        && members(&proof.panel)?[0]["head"] == members(&first.panel)?[0]["head"]
                    {
                        return Err("repair delivers a new head".into());
                    }
                    self.barrier.release(proof.run);
                    self.second = Some(proof);
                }
            } else if self.landed(self.second.as_ref().ok_or("repaired proof")?)? {
                if super::main(self.forge)? != candidate(&self.second.as_ref().ok_or("repaired proof")?.panel)? {
                    return Err("repaired prefix reached main".into());
                }
                if matches!(self.story, Story::MidProof)
                    && super::get(self.forge, &format!("actions/runs/{}", first.run))["status"] != "success"
                {
                    return Ok(());
                }
                self.phase = Phase::Ending;
            }
        }
        Ok(())
    }
}

/// A real root needs more entries than the tiny key fixture.
pub fn add_root(forge: &Throwaway) {
    for number in 0..17 {
        forge.add_file(&format!("root-{number}.txt"), b"Repository root entry.\n");
    }
}

pub fn run_repaired(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    super::run_story(root, forge, barrier, Story::Repaired);
}
pub fn run_green(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    super::run_story(root, forge, barrier, Story::DeploymentGreen);
}

pub(super) fn resolution_script(scripts: &mut Vec<Script>) {
    let mut read = call(b"read", &json!({"path":"shared.txt"}));
    read.lines = [vec![hold(b"deployment-resolution")], read.lines.into_vec()].concat().into_boxed_slice();
    scripts.push(Script {
        cue: b"Resolve the conflict from change head".as_slice().into(),
        turns: vec![
            read,
            call(b"write", &json!({"path":"shared.txt","content":"Change 1 and change 2 resolved\n"})),
            call(
                b"finish",
                &json!({"form":"change","fields":{"title":"Resolve both changes","body":"Preserve both goals."}}),
            ),
        ]
        .into_boxed_slice(),
    });
}

pub fn run_conflict(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    super::run_story(root, forge, barrier, Story::Conflict);
}

pub(super) fn goal_scripts(forge: &Throwaway) -> Box<[Script]> {
    use skein_fake_llm_domain::api::{Finish, Turn};
    let mut scripts = crate::landing_scripts(crate::ANSWER, forge).into_vec();
    let owner = scripts.iter_mut().find(|script| script.cue.as_ref() == b"@forge_chat").expect("owner");
    owner.turns = Box::new([call(
        b"finish",
        &json!({"form":"report","text":temper_real_loop_world::landing::REQUESTED,"fields":{}}),
    )]);
    let review = scripts
        .iter_mut()
        .find(|script| script.cue.as_ref() == b"Review this change at the named head")
        .expect("review");
    review.turns[0].lines = [vec![hold(b"batch-reviews")], review.turns[0].lines.to_vec()].concat().into_boxed_slice();
    let mut moved = call(b"write", &json!({"path":"design.md","content":"# Design\nSecond delivery\n"}));
    moved.lines = [vec![hold(b"goal-midproof")], moved.lines.into_vec()].concat().into_boxed_slice();
    let finish = call(b"finish", &json!({"form":"report","text":"Design landed.","fields":{}}));
    let held_finish = Turn {
        lines: [vec![hold(b"goal-finish")], finish.lines.into_vec()].concat().into_boxed_slice(),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    scripts.push(Script {
        cue: b"## Task\n\n@moving-goal".as_slice().into(),
        turns: vec![
            call(b"read", &json!({"path":"design.md"})),
            call(b"write", &json!({"path":"design.md","content":"# Design\nFirst delivery\n"})),
            call(b"deliver", &json!({"fields":{"title":"First design","body":"Initial design."}})),
            call(b"change", &json!({"words":"Land the moving design","branch":"mine"})),
            moved,
            call(b"deliver", &json!({"fields":{"title":"Second design","body":"Updated while proving."}})),
            held_finish,
        ]
        .into_boxed_slice(),
    });
    scripts.into_boxed_slice()
}

pub fn run_midproof(root: &Path, forge: &Throwaway, barrier: &JobBarrier) {
    super::run_story(root, forge, barrier, Story::MidProof);
}
