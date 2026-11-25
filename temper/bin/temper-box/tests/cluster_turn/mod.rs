//! Cluster process boundaries on the fake forge (testing/process-tier.md).
mod unit_stop;
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
fn an_engine_killed_is_started_again_on_its_directories_and_its_worker_links_again() {
    story("cluster-turn-engine", Cut::Engine).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn a_worker_restarted_mid_delivery_links_again_on_its_directories() {
    story("cluster-turn-delivery", Cut::Worker).unwrap_or_else(|why| panic!("{why}"));
}

/// draft/archive/worker-restarts.md, 9, test 8: unit stop and a stray agent signal while an answer is held.
#[test]
fn a_unit_stop_settles_a_workers_held_turn() {
    story("cluster-unit-stop", Cut::Unit).unwrap_or_else(|why| panic!("{why}"));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cut {
    Engine,
    Worker,
    Unit,
}
impl Cut {
    fn within(self) -> Duration {
        // The user's process-temper wall backstop (tools/tiers/budgets.toml).
        // The unit variant's stop deadline is the printed bound plus the help's margin;
        // setup and ctl reads still have WITHIN, and its CPU budget remains 60 s.
        if self == Self::Unit { Duration::from_secs(1080) } else { support::WITHIN }
    }
}
#[derive(Debug, PartialEq, Eq)]
enum Stage {
    Start,
    Configured,
    Created,
    ChatReplied,
    ChatClosed,
    BeforeKill,
    Holding,
    Releasing,
    WorkerEnded,
    EngineEnded,
    Restarted,
    WorkerRead,
    PullReady,
    Pull,
    Head,
    AfterRestart,
    Closed,
    Stopping,
    Unit,
}

const CHAT: &str = "Answer before the engine restarts";
const REPLY: &str = "The worker answered before the restart.";
const DELIVERY: &str = "Add a file NOTICE to widget";

fn story(name: &str, cut: Cut) -> Result<(), String> {
    let scratch = support::Scratch::new(name);
    let root = scratch.path();
    let mut cluster = Cluster::read_from(root, &setup::fixture());
    cluster.deployment_binaries();
    setup::prepare(&mut cluster, root, &support::free_addresses(&[]));
    if cut == Cut::Worker {
        let directory = root.join("held-git");
        fs::create_dir(&directory).expect("held git directory");
        let wrapper = directory.join("git");
        support::write_private(&wrapper, setup::wrapper(root, &root.join("git")).as_bytes());
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).expect("executable wrapper");
        cluster.git_on_path("w1", &directory);
        support::write_private(&root.join("delivery.hold"), b"");
    }
    if cut == Cut::Unit {
        unit_stop::prepare(&mut cluster);
    }
    let within = cut.within();
    let key = cluster.dir("llm").join("llm.key");
    let admin = cluster.dir("forge").join("setup/admin_token");
    let (mut members, hosts) = cluster.members_within(within);
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
        linked_before: 0,
        turn: String::new(),
        committed: Value::Null,
        unit: unit_stop::Stop::new(),
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(Root::client);
    let outcome = world.try_run(&clock, within)?;
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
    linked_before: usize,
    turn: String,
    committed: Value,
    unit: unit_stop::Stop,
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
    fn begin(&mut self, host: &mut Root) -> Result<(), String> {
        let address = self.members.capture("forge", "address").ok_or("forge address")?;
        let settings_root = self.root.clone();
        let workers = support::cluster::worker_adds(&self.members.dir("engine").join("settings.json"));
        let mut steps = vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["project", "make", "Cluster turn", "--key", "turn-project"])),
            Step::Run(Call::new(&["settings", "show", "--json"])),
            Step::Run(Call::new(&["settings", "show", "charter", "1", "--json"])),
            Step::Run(Call::new(&["settings", "show", "charter", "2", "--json"])),
            Step::Then(Box::new(move |calls| setup::settings(&settings_root, &calls[1..]))),
        ];
        steps.extend(workers.into_iter().map(Step::Run));
        if self.cut != Cut::Engine {
            self.adopt(address, &mut steps)?;
        }
        steps.extend([
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
                let spend = period / 48 * 41;
                let amount = format!("{}.{:02}", spend / 100, spend % 100);
                Call::new(&["project", "fund", "1", &amount, "--role", "0", "--key", "turn-funds"])
            })),
        ]);
        self.queue(host, steps)?;
        self.stage = Stage::Configured;
        Ok(())
    }
    fn adopt(&self, address: &str, steps: &mut Vec<Step>) -> Result<(), String> {
        steps.extend([
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
            Step::Run(self.api_request(
                "/collaborators/temper-admin",
                &[
                    "--request",
                    "PUT",
                    "--header",
                    "Content-Type: application/json",
                    "--data",
                    r#"{"permission":"admin"}"#,
                ],
            )?),
        ]);
        Ok(())
    }
    fn api(&self, suffix: &str) -> Result<Call, String> {
        self.api_request(suffix, &[])
    }
    fn api_request(&self, suffix: &str, extra: &[&str]) -> Result<Call, String> {
        let address = self.members.capture("forge", "address").ok_or("forge address")?;
        let header = format!("@{}", self.root.join("person.header").display());
        let url = format!("http://{address}/api/v1/repos/fake/widget{suffix}");
        let mut arguments = vec![
            "--fail",
            "--silent",
            "--show-error",
            "--write-out",
            "%{stderr}%{onerror}%{url_effective}\\n",
            "--header",
            &header,
            &url,
        ];
        arguments.extend_from_slice(extra);
        Ok(Call::command(Path::new("/usr/bin/curl"), &arguments))
    }
    fn advance(&mut self, now: Time, hosts: &mut [Root]) -> Result<(), String> {
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = Stage::Stopping;
            return Ok(());
        }
        if self.stage == Stage::Unit {
            return self.advance_unit(now, hosts);
        }
        let (host, _) = hosts.split_last_mut().ok_or("client root")?;
        self.tick = None;
        match self.stage {
            Stage::Start if self.members.capture("engine", "link").is_some() => self.begin(host)?,
            Stage::Holding => {
                if self.members.is_up() && self.root.join("delivery.entered").is_file() {
                    self.members.stop_one("w1");
                    self.linked_before = self.members.linked("w1");
                    self.stage = Stage::Releasing;
                    self.tick = Some(now);
                } else {
                    self.tick = Some(now.saturating_add(Duration::from_millis(20)));
                }
            }
            Stage::Releasing => {
                // members.act has signalled the worker; the in-flight push is part of its drain.
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
            Stage::Restarted if self.members.is_up() && self.members.linked("w1") > self.linked_before => {
                if self.cut == Cut::Engine {
                    self.read_turn(host, Stage::AfterRestart)?;
                } else {
                    self.read_worker(host)?;
                }
            }
            Stage::PullReady => {
                if self.pull_opened()? {
                    self.calls(host, vec![self.api("/pulls?state=all&limit=32&page=1")?])?;
                    self.stage = Stage::Pull;
                } else {
                    self.tick = Some(now.saturating_add(Duration::from_millis(20)));
                }
            }
            Stage::Configured
            | Stage::Created
            | Stage::ChatReplied
            | Stage::ChatClosed
            | Stage::BeforeKill
            | Stage::WorkerRead
            | Stage::Pull
            | Stage::Head
            | Stage::AfterRestart
            | Stage::Closed
                if host.is_empty() =>
            {
                let calls = check_calls(host)?;
                self.completed(now, host, calls.ends())?;
            }
            Stage::Start
            | Stage::Configured
            | Stage::Created
            | Stage::ChatReplied
            | Stage::ChatClosed
            | Stage::BeforeKill
            | Stage::WorkerEnded
            | Stage::EngineEnded
            | Stage::Restarted
            | Stage::WorkerRead
            | Stage::Pull
            | Stage::Head
            | Stage::AfterRestart
            | Stage::Closed
            | Stage::Stopping
            | Stage::Unit => {}
        }
        Ok(())
    }
    // The link can reopen before the engine has opened the pull. Use the fake's
    // event only as readiness; the oracle reads the pull and its head through HTTP.
    fn pull_opened(&self) -> Result<bool, String> {
        let path = self.members.dir("forge").join("observations.jsonl");
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(why) => return Err(format!("forge observations: {why}")),
        };
        for line in bytes.split_inclusive(|byte| *byte == b'\n').filter(|line| line.ends_with(b"\n")) {
            let entry: Value = serde_json::from_slice(line).map_err(|why| format!("forge observation: {why}"))?;
            if entry["observation"] == "opened" {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn close_chat(&self) -> Vec<Call> {
        vec![
            Call::new(&["chat", "close", &self.chat, "--key", "close-cluster-chat"]),
            Call::new(&["task", "show", &self.chat, "--until", "ended", "--json"]),
        ]
    }
    fn read_turn(&mut self, host: &mut Root, stage: Stage) -> Result<(), String> {
        self.calls(host, vec![Call::new(&["task", "turn", &self.chat, &self.turn, "--json"])])?;
        self.stage = stage;
        Ok(())
    }
    fn read_worker(&mut self, host: &mut Root) -> Result<(), String> {
        self.calls(host, vec![Call::new(&["account", "list", "--json"])])?;
        self.stage = Stage::WorkerRead;
        Ok(())
    }
    fn completed(&mut self, now: Time, host: &mut Root, calls: &[support::Ended]) -> Result<(), String> {
        let call = calls.last().ok_or("completed story call")?;
        match self.stage {
            Stage::Configured => {
                let cue = if self.cut == Cut::Engine { CHAT } else { DELIVERY };
                self.calls(
                    host,
                    vec![Call::new(&["chat", "start", "--project", "1", "--key", "cluster-chat", cue, "--json"])],
                )?;
                self.stage = Stage::Created;
            }
            Stage::Created => {
                self.chat = json(call)?["outcome"]["done"]["document"]["created"]["first"]
                    .as_u64()
                    .ok_or("chat task")?
                    .to_string();
                match self.cut {
                    Cut::Engine => {
                        self.calls(
                            host,
                            vec![
                                Call::new(&["task", "show", &self.chat, "--until", "replied", "--json"]),
                                Call::new(&["account", "list", "--json"]),
                            ],
                        )?;
                        self.stage = Stage::ChatReplied;
                    }
                    Cut::Worker => self.stage = Stage::Holding,
                    Cut::Unit => self.stage = Stage::Unit,
                }
                self.tick = Some(now);
            }
            Stage::ChatReplied => {
                let task = json(calls.first().ok_or("live chat")?)?;
                let details = &task["task"]["chip"]["details"]["owner"]["details"];
                let hosts = json(call)?;
                if worker_channel(&hosts)?
                    != details["observation"]["channel"].as_u64().ok_or("answered chat's channel")?
                {
                    return Err("the chat was answered on w1".into());
                }
                let attempt = details["attempt"].as_u64().ok_or("answered chat's attempt")?;
                self.turn = format!("{attempt}.1");
                self.calls(host, self.close_chat())?;
                self.stage = Stage::ChatClosed;
            }
            Stage::ChatClosed => self.read_turn(host, Stage::BeforeKill)?,
            Stage::BeforeKill => {
                self.committed = committed_reply(call, REPLY)?;
                self.linked_before = self.members.linked("w1");
                self.members.kill("engine");
                self.stage = Stage::EngineEnded;
                self.tick = Some(now);
            }
            Stage::AfterRestart => {
                if committed_reply(call, REPLY)? != self.committed {
                    return Err("the committed turn changed after restarting the engine".into());
                }
                self.stop(now);
            }
            Stage::WorkerRead => {
                worker_channel(&json(call)?)?;
                self.stage = Stage::PullReady;
                self.tick = Some(now);
            }
            Stage::Pull => {
                let pulls = json(call)?;
                let [pull] = pulls.as_array().ok_or("forge pulls")?.as_slice() else {
                    return Err(format!("one change pull request: {pulls}"));
                };
                let head = pull["head"]["sha"].as_str().ok_or("pull head")?;
                self.calls(host, vec![self.api(&format!("/raw/NOTICE?ref={head}"))?])?;
                self.stage = Stage::Head;
            }
            Stage::Head => {
                if call.stdout != setup::CONTENT {
                    return Err(format!("both lines at the pull head: {}", String::from_utf8_lossy(&call.stdout)));
                }
                self.calls(host, self.close_chat())?;
                self.stage = Stage::Closed;
            }
            Stage::Closed => self.stop(now),
            Stage::Start
            | Stage::Holding
            | Stage::Releasing
            | Stage::WorkerEnded
            | Stage::EngineEnded
            | Stage::Restarted
            | Stage::PullReady
            | Stage::Stopping
            | Stage::Unit => return Err("no calls in this stage".into()),
        }
        Ok(())
    }
    fn stop(&mut self, now: Time) {
        self.members.stop();
        self.tick = Some(now);
        self.stage = Stage::Stopping;
    }
}

fn worker_channel(accounts: &Value) -> Result<u64, String> {
    accounts["accounts"]["deployment_facts"]["program_hosts"]
        .as_array()
        .ok_or("deployment hosts")?
        .iter()
        .find(|host| host["name"] == "w1" && host["connected"] == true)
        .and_then(|host| host["channel"].as_u64())
        .ok_or_else(|| "connected w1 in the engine".into())
}

fn committed_reply(call: &support::Ended, reply: &str) -> Result<Value, String> {
    let turn = json(call)?;
    crate::complete_cluster_json(&turn)?;
    let decoded = &turn["turn"]["decoded"];
    let messages = decoded["messages"].as_array().ok_or("committed turn's messages")?;
    if !messages
        .iter()
        .filter(|message| message["speaker"] == "assistant")
        .filter_map(|message| message["blocks"].as_array())
        .flatten()
        .any(|block| block["text"]["text"].as_str().is_some_and(|text| text.contains(reply)))
    {
        return Err(format!("the committed turn holds {reply:?}: {turn}"));
    }
    Ok(decoded.clone())
}

fn check_calls(host: &mut Root) -> Result<Box<ctl::Calls>, String> {
    let Root::Client { calls, .. } = host else { return Err("client root".into()) };
    let calls = calls.take().ok_or("completed story calls")?;
    for ended in calls.ends() {
        if ended.exit != Some(Exit::Code(0)) {
            return Err(format!("story call {:?}: {}", ended.exit, support::without_secrets(&ended.stderr)));
        }
    }
    Ok(calls)
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
            use support::cluster::Ending::{Alone, Cluster, Kill};
            let expected: &[(&str, usize, support::cluster::Ending)] = match self.cut {
                Cut::Engine => &[
                    ("engine", 0, Kill),
                    ("engine", 1, Cluster),
                    ("w1", 0, Cluster),
                    ("llm", 0, Cluster),
                    ("forge", 0, Cluster),
                ],
                Cut::Worker => &[
                    ("w1", 0, Alone),
                    ("w1", 1, Cluster),
                    ("engine", 0, Cluster),
                    ("llm", 0, Cluster),
                    ("forge", 0, Cluster),
                ],
                Cut::Unit => &[("w1", 0, Alone), ("engine", 0, Cluster), ("llm", 0, Cluster), ("forge", 0, Cluster)],
            };
            let order: Vec<_> = stopped.iter().map(|life| (life.name.as_str(), life.life, life.ending)).collect();
            if order != expected {
                return Err(format!("intentional endings then reverse stop: {order:?}"));
            }
            for life in stopped {
                let expected = if life.ending == Kill { Exit::Signal(9) } else { Exit::Code(0) };
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
        (now >= self.deadline && !self.passed()).then(|| {
            if self.stage == Stage::Unit {
                format!("cluster unit stop {:?}", self.unit)
            } else {
                format!("cluster restart {:?}", self.stage)
            }
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.stage == Stage::Stopping && self.members.verdict() == Ok(true)
    }
}
