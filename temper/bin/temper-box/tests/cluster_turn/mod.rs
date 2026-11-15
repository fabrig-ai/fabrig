//! Landing's two process boundaries on the fake forge (testing/process-tier.md).
use serde_json::Value;
use skein_io::kernel::Exit;
use skein_lib::{Duration, Time};
use skein_shell::Host;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use temper_box_kit::{
    self as support,
    cluster::{Cluster, Members, Root},
    cluster_turn as setup,
    ctl::{self, Call, Step},
};

#[test]
fn a_change_lands_by_turn_across_an_engine_killed_mid_proof() {
    story("cluster-turn-engine", Cut::Engine).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn a_worker_restarted_mid_delivery_lands_its_saved_work_by_turn() {
    story("cluster-turn-delivery", Cut::Worker).unwrap_or_else(|why| panic!("{why}"));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cut {
    Engine,
    Worker,
}
#[derive(Debug, PartialEq, Eq)]
enum Stage {
    Start,
    Created,
    Holding,
    Releasing,
    WorkerEnded,
    EngineEnded,
    Restarted,
    List,
    Panel,
    PanelAgain(Time),
    Change,
    Verify,
    Jobs,
    Closed,
    Stopping,
}

fn story(name: &str, cut: Cut) -> Result<(), String> {
    let scratch = support::Scratch::new(name);
    let root = scratch.path();
    let mut cluster = Cluster::read_from(root, &setup::fixture());
    cluster.deployment_binaries();
    setup::prepare(&mut cluster, root, &support::free_addresses(&[]));
    if cut == Cut::Worker {
        let wrapper = root.join("held-git");
        support::write_private(&wrapper, setup::wrapper(root, &root.join("git")).as_bytes());
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).expect("executable wrapper");
        let path = cluster.dir("w1").join("config/temper.json");
        let mut worker = setup::read(&path);
        worker["application"]["workers"][0]["git"] = wrapper.display().to_string().into();
        setup::write(&path, &worker);
        support::write_private(&root.join("delivery.hold"), b"");
    }
    let key = cluster.dir("llm").join("llm.key");
    let admin = cluster.dir("forge").join("setup/admin_token");
    let (mut members, hosts) = cluster.members();
    members.up(&["w1"]);
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(Judge {
        members,
        root: root.into(),
        key,
        admin,
        cut,
        stage: Stage::Start,
        deadline: clock.now().now.saturating_add(support::WITHIN),
        tick: None,
        finding: None,
        chat: String::new(),
        landing: String::new(),
        change: String::new(),
        candidate: Value::Null,
        run: None,
        panel: Value::Null,
        changed: Value::Null,
        last: Value::Null,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(Root::client);
    let outcome = world.try_run(&clock, support::WITHIN)?;
    if !outcome.procs.iter().all(Host::is_empty) {
        return Err("every member and the client settled".into());
    }
    scratch.remove();
    Ok(())
}

struct Judge {
    members: Members,
    root: PathBuf,
    key: PathBuf,
    admin: PathBuf,
    cut: Cut,
    stage: Stage,
    deadline: Time,
    tick: Option<Time>,
    finding: Option<String>,
    chat: String,
    landing: String,
    change: String,
    candidate: Value,
    run: Option<u64>,
    panel: Value,
    changed: Value,
    last: Value,
}

fn json(call: &support::Ended) -> Result<Value, String> {
    if call.exit != Some(Exit::Code(0)) {
        return Err(format!("story call: {:?}; {}", call.exit, support::without_secrets(&call.stderr)));
    }
    serde_json::from_slice(&call.stdout).map_err(|why| format!("story JSON: {why}"))
}
fn one<'a>(value: &'a Value, field: &str) -> Result<&'a Value, String> {
    let [row] = value[field].as_array().ok_or_else(|| format!("{field}: {value}"))?.as_slice() else {
        return Err(format!("exactly one {field}: {value}"));
    };
    Ok(row)
}

impl Judge {
    fn queue(&self, host: &mut Root, steps: Vec<Step>) -> Result<(), String> {
        let link = format!("http://{}", self.members.capture("engine", "link").ok_or("engine link")?);
        let mut next = ctl::Calls::linked(&link, &self.root.join("client"));
        for step in steps {
            next.push(step);
        }
        let Root::Client { calls, .. } = host else {
            return Err("client root".into());
        };
        *calls = Some(Box::new(next));
        Ok(())
    }
    fn calls(&self, host: &mut Root, calls: Vec<Call>) -> Result<(), String> {
        self.queue(host, calls.into_iter().map(Step::Run).collect())
    }
    fn list(&mut self, host: &mut Root) -> Result<(), String> {
        self.stage = Stage::List;
        self.queue(
            host,
            vec![Step::Until(Call::new(&["landing", "list", "--project", "1", "--ended", "--json"]), b"\"task\":")],
        )
    }
    fn panel(&mut self, host: &mut Root) -> Result<(), String> {
        self.stage = Stage::Panel;
        self.calls(host, vec![Call::new(&["landing", "show", &self.landing, "--json"])])
    }
    fn begin(&mut self, host: &mut Root) -> Result<(), String> {
        let address = self.members.capture("forge", "address").ok_or("forge address")?;
        let settings_root = self.root.clone();
        self.queue(
            host,
            vec![
                Step::Run(Call::sign_in()),
                Step::Run(Call::new(&["settings", "show", "--json"])),
                Step::Run(Call::new(&["settings", "show", "policy", "1", "--json"])),
                Step::Run(Call::new(&["settings", "show", "charter", "1", "--json"])),
                Step::Run(Call::new(&["settings", "show", "charter", "2", "--json"])),
                Step::Then(Box::new(move |calls| setup::settings(&settings_root, calls))),
                Step::Run(Call::new(&[
                    "forge",
                    "connect",
                    "forgejo-admin",
                    self.admin.to_str().ok_or("admin path")?,
                    "--api",
                    address,
                    "--git",
                    &format!("http://{address}"),
                    "--key",
                    "turn-forge",
                ])),
                Step::Run(Call::new(&["forge", "list", "--until", "connected:1"])),
                Step::Run(Call::new(&[
                    "repository",
                    "adopt",
                    "--project",
                    "1",
                    "--forge",
                    "1",
                    "--owner",
                    "fake",
                    "--name",
                    "widget",
                    "--landing",
                    "main",
                    "--key",
                    "turn-adopt",
                ])),
                Step::Run(Call::new(&[
                    "account",
                    "add-key",
                    "1",
                    self.key.to_str().ok_or("key path")?,
                    "--deployment",
                    "--key",
                    "turn-llm",
                ])),
                Step::Run(Call::new(&["account", "list", "--until", "usable"])),
                Step::Run(Call::new(&["project", "show", "1", "--json"])),
                Step::Then(Box::new(|calls| {
                    let project = ctl::json(calls.last().expect("project read"));
                    let period = project["policy"]["period_spend"].as_u64().expect("project period");
                    Call::new(&[
                        "project",
                        "fund",
                        "1",
                        &(period / 48 * 41).to_string(),
                        "--role",
                        "0",
                        "--key",
                        "turn-funds",
                    ])
                })),
                Step::Run(Call::new(&[
                    "chat",
                    "start",
                    "--project",
                    "1",
                    "--key",
                    "turn-chat",
                    "Add a file NOTICE to widget",
                    "--json",
                ])),
            ],
        )?;
        self.stage = Stage::Created;
        Ok(())
    }
    fn api(&self, suffix: &str) -> Result<Call, String> {
        let address = self.members.capture("forge", "address").ok_or("forge address")?;
        Ok(Call::command(
            Path::new("/usr/bin/curl"),
            &[
                "--fail",
                "--silent",
                "--show-error",
                "--header",
                &format!("@{}", self.root.join("person.header").display()),
                &format!("http://{address}/api/v1/repos/fake/widget{suffix}"),
            ],
        ))
    }
    fn verify(&mut self, host: &mut Root) -> Result<(), String> {
        self.stage = Stage::Verify;
        self.calls(
            host,
            vec![
                Call::new(&["landing", "list", "--project", "1", "--ended", "--json"]),
                self.api("/branches/main")?,
                self.api("/pulls?state=all&limit=100&page=1")?,
                self.api("/actions/runs?limit=100&page=1")?,
                self.api("/contents/NOTICE?ref=main")?,
            ],
        )
    }

    fn advance(&mut self, now: Time, hosts: &mut [Root]) -> Result<(), String> {
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = Stage::Stopping;
            return Ok(());
        }
        let (host, _) = hosts.split_last_mut().ok_or("client root")?;
        self.tick = None;
        match self.stage {
            Stage::Start if self.members.is_up() => self.begin(host)?,
            Stage::PanelAgain(at) => {
                if now >= at {
                    self.panel(host)?;
                } else {
                    self.tick = Some(at);
                }
            }
            Stage::Holding => {
                if self.root.join("delivery.entered").is_file() {
                    self.members.stop_one("w1");
                    self.stage = Stage::Releasing;
                    self.tick = Some(now);
                } else {
                    self.tick = Some(now.saturating_add(Duration::from_millis(20)));
                }
            }
            Stage::Releasing => {
                // members.act has now requested the worker's orderly stop. Let its
                // already running push finish, so its delivery is part of the drain.
                fs::remove_file(self.root.join("delivery.hold")).map_err(|why| why.to_string())?;
                self.stage = Stage::WorkerEnded;
            }
            Stage::WorkerEnded if self.members.ended("w1") => {
                self.members.restart("w1");
                self.stage = Stage::Restarted;
                self.tick = Some(now);
            }
            Stage::EngineEnded if self.members.ended("engine") => {
                self.members.restart("engine");
                self.stage = Stage::Restarted;
                self.tick = Some(now);
            }
            Stage::Restarted if self.members.is_up() => {
                if self.cut == Cut::Engine {
                    self.panel(host)?;
                } else {
                    self.list(host)?;
                }
            }
            Stage::Created
            | Stage::List
            | Stage::Panel
            | Stage::Change
            | Stage::Verify
            | Stage::Jobs
            | Stage::Closed
                if host.is_empty() =>
            {
                let Root::Client { calls, .. } = host else {
                    return Err("client root".into());
                };
                let calls = calls.take().ok_or("completed calls")?;
                for ended in calls.ends() {
                    if ended.exit != Some(Exit::Code(0)) {
                        return Err(format!(
                            "story call {:?}: {}",
                            ended.exit,
                            support::without_secrets(&ended.stderr)
                        ));
                    }
                }
                self.completed(now, host, calls.ends())?;
            }
            Stage::Start
            | Stage::Created
            | Stage::WorkerEnded
            | Stage::EngineEnded
            | Stage::Restarted
            | Stage::List
            | Stage::Panel
            | Stage::Change
            | Stage::Verify
            | Stage::Jobs
            | Stage::Closed
            | Stage::Stopping => {}
        }
        Ok(())
    }

    fn completed(&mut self, now: Time, host: &mut Root, calls: &[support::Ended]) -> Result<(), String> {
        self.last = json(calls.last().ok_or("last call")?)?;
        match self.stage {
            Stage::Created => {
                self.chat = self.last["outcome"]["done"]["document"]["created"]["first"]
                    .as_u64()
                    .ok_or("chat task")?
                    .to_string();
                if self.cut == Cut::Worker {
                    self.stage = Stage::Holding;
                    self.tick = Some(now);
                } else {
                    self.list(host)?;
                }
            }
            Stage::List => {
                if self.last["landings"].as_array().ok_or("landings")?.is_empty() {
                    self.list(host)?;
                } else {
                    let row = one(&self.last, "landings")?;
                    if self.cut == Cut::Engine && self.run.is_none() {
                        setup::proof_cut(row)?;
                    }
                    self.landing = row["task"].as_u64().ok_or("landing task")?.to_string();
                    self.panel(host)?;
                }
            }
            Stage::Panel => self.landing_read(now, host)?,
            Stage::Change => {
                if !self.last["hold"].is_null() {
                    return Err(format!("change held: {}", self.last));
                }
                if self.last["step"] == "landed" {
                    if self.last["merge"] != self.candidate {
                        return Err(format!("change at candidate: {}", self.last));
                    }
                    self.changed = self.last.clone();
                    self.verify(host)?;
                } else {
                    self.calls(host, vec![Call::new(&["change", "show", &self.change, "--json"])])?;
                }
            }
            Stage::Verify => self.forge_read(host, calls)?,
            Stage::Jobs => {
                let [job] = self.last.as_array().ok_or("gate jobs")?.as_slice() else {
                    return Err("one gate job".into());
                };
                if job["name"] != "gate" || job["status"] != "success" {
                    return Err(format!("gate passed: {job}"));
                }
                self.stage = Stage::Closed;
                self.calls(
                    host,
                    vec![
                        Call::new(&["chat", "close", &self.chat, "--key", "close-turn-chat"]),
                        Call::new(&["task", "show", &self.chat, "--until", "ended", "--json"]),
                    ],
                )?;
            }
            Stage::Closed => {
                self.members.stop();
                self.stage = Stage::Stopping;
                self.tick = Some(now);
            }
            Stage::Start
            | Stage::Holding
            | Stage::Releasing
            | Stage::WorkerEnded
            | Stage::EngineEnded
            | Stage::Restarted
            | Stage::PanelAgain(_)
            | Stage::Stopping => return Err("no calls in this stage".into()),
        }
        Ok(())
    }
    fn panel_later(&mut self, now: Time) {
        let at = now.saturating_add(Duration::from_millis(100));
        self.stage = Stage::PanelAgain(at);
        self.tick = Some(at);
    }
    fn landing_read(&mut self, now: Time, host: &mut Root) -> Result<(), String> {
        let panel = self.last.clone();
        if !panel["hold"].is_null() {
            return Err(format!("landing held: {panel}"));
        }
        let member = one(&panel, "members")?;
        self.change = member["change"].as_u64().ok_or("member change")?.to_string();
        if self.cut == Cut::Engine && self.run.is_none() {
            if let Some(run) = setup::proof_cut(&panel)? {
                self.candidate = member["candidate"].clone();
                self.run = Some(run);
                self.members.kill("engine");
                self.stage = Stage::EngineEnded;
                self.tick = Some(now);
            } else {
                self.panel_later(now);
            }
            return Ok(());
        }
        if panel["phase"] != "landed" {
            self.panel_later(now);
            return Ok(());
        }
        if self.candidate.is_null() {
            self.candidate = member["candidate"].clone();
        }
        if !self.candidate.is_string()
            || member["candidate"] != self.candidate
            || member["landed"] != self.candidate
            || panel["preparation"] != 1
            || panel["stale_proofs"] != 0
        {
            return Err(format!("one preparation lands the same candidate: {panel}"));
        }
        self.panel = panel;
        self.stage = Stage::Change;
        self.calls(host, vec![Call::new(&["change", "show", &self.change, "--json"])])
    }
    fn forge_read(&mut self, host: &mut Root, calls: &[support::Ended]) -> Result<(), String> {
        let [list, main, pulls, runs, content] = calls else {
            return Err("final five reads".into());
        };
        let list = json(list)?;
        let row = one(&list, "landings")?;
        if row["task"].as_u64().map(|n| n.to_string()).as_deref() != Some(&self.landing) || row["phase"] != "landed" {
            return Err(format!("one ended, landed landing: {list}"));
        }
        let main = json(main)?;
        if main["commit"]["id"] != self.candidate {
            return Err(format!("main at candidate: {main}"));
        }
        let pulls = json(pulls)?;
        let pulls = pulls.as_array().ok_or("pulls")?;
        let member = &self.changed["pull"]["number"];
        let round = &self.panel["round"]["pull"]["number"];
        if pulls.len() != 2
            || member == round
            || pulls.iter().filter(|pull| pull["number"] == *member && pull["merged"] == true).count() != 1
            || pulls.iter().filter(|pull| pull["number"] == *round && pull["merged"] == true).count() != 1
        {
            return Err(format!(
                "one change PR and one round PR, both merged: {pulls:?}; member={member}, round={round}"
            ));
        }
        let runs = json(runs)?;
        let run = one(&runs, "workflow_runs")?;
        if run["workflow_id"] != "gate.yml"
            || run["head_sha"] != self.candidate
            || run["status"] != "success"
            || self.run.is_some_and(|id| run["id"] != id)
        {
            return Err(format!("one successful gate.yml run at the candidate: {runs}"));
        }
        let content = json(content)?;
        let encoded: Vec<_> =
            content["content"].as_str().ok_or("main content")?.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
        let bytes = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, &encoded, 1024)
            .map_err(|why| format!("main base64: {why:?}"))?;
        if bytes.as_ref() != setup::CONTENT {
            return Err(format!("both lines at main: {}", String::from_utf8_lossy(&bytes)));
        }
        let run_id = run["id"].as_u64().ok_or("run id")?;
        self.stage = Stage::Jobs;
        self.calls(host, vec![self.api(&format!("/actions/runs/{run_id}/jobs"))?])
    }
}

impl skein_world::Referee<Root> for Judge {
    fn act(&mut self, now: Time, hosts: &mut [Root]) {
        if self.finding.is_none() {
            self.finding = self.advance(now, hosts).err();
        }
    }
    fn watch(&mut self, now: Time, hosts: &[Root]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        let (_, members) = hosts.split_last().ok_or("client root")?;
        self.members.observe(now, members);
        self.members.verdict()?;
        if let Some(stopped) = self.members.stopped() {
            let restarted = if self.cut == Cut::Engine { "engine" } else { "w1" };
            if stopped.iter().filter(|life| life.name == restarted).count() != 2 {
                return Err(format!("two lives of {restarted}"));
            }
            for life in stopped {
                let expected = if life.name == "engine" && life.life == 0 && self.cut == Cut::Engine {
                    Exit::Signal(9)
                } else {
                    Exit::Code(0)
                };
                if life.exit != Some(expected) || life.killed || !life.survivors.is_empty() {
                    return Err(format!(
                        "{} life {}: {:?}; {}",
                        life.name,
                        life.life,
                        life.exit,
                        support::without_secrets(&life.stderr)
                    ));
                }
            }
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.members.next_deadline().into_iter().chain(self.tick).fold(self.deadline, Time::min))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline && !self.passed())
            .then(|| format!("cluster turn {:?}; last read: {}", self.stage, self.last))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.stage == Stage::Stopping && self.members.verdict() == Ok(true)
    }
}
