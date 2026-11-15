//! Staging's turn drills on separate engine and worker processes and a scripted model.
//! temper/docs/design/domain/landing.md, 5, 7, 8 and 15.
mod batch;
mod delivery;
mod departures;
mod go_live;
use super::{
    Process, Scratch,
    batch_landing::call,
    ctl,
    forgejo::{JobBarrier, Throwaway},
    person, turn_restart,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Line, Rule, Rules, Script, Scripts};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drill {
    Phases,
    Delivery,
    BatchFour,
    Breaking,
    Conflict,
    GoLive,
    Moving,
}
impl Drill {
    fn members(self) -> usize {
        if self == Self::Moving {
            3
        } else if self.is_batch() {
            4
        } else {
            3
        }
    }
    fn is_batch(self) -> bool {
        matches!(self, Self::BatchFour | Self::Breaking | Self::Conflict | Self::Moving)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Setup,
    Start,
    Observe,
    Restart,
    Resume,
    Landed,
    Verify,
    WorkerStopping,
    DeliveryRest,
    Batch,
    GoLiveApply,
    GoLiveStays,
    Stop,
}

fn document(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("fixture document")).expect("fixture JSON")
}
fn write(path: &Path, value: &Value) {
    super::write_private(path, &serde_json::to_vec(value).expect("fixture JSON"));
}

fn configure(root: &Path, repository: u64, drill: Drill) {
    turn_restart::configure(root, repository);
    let path = root.join("temper.json");
    let mut value = document(&path);
    value["workers"][0]["slots"] = 8.into();
    value["workers"][0]["graces"] =
        json!({"grace_ms":1000,"drain_ms":1000,"agent_stop_ms":100,"delivery_ms":100,"save_ms":100});
    value["application"]["limits"]["overrides"]["checkout.workspaces"] = 8.into();
    value["application"]["limits"]["overrides"]["git_component.operations"] = 8.into();
    value["workers"][0]["limits"]["overrides"]["hub.slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["fleet.slots"] = 8.into();
    // A restart heals while the worker retains its prepare job and agent workspace.
    value["engine"]["limits"]["overrides"]["fleet.grace_ms"] = 30_000.into();
    value["application"]["limits"]["overrides"]["forge.client.fallback_ms"] = 1000.into();
    if drill == Drill::GoLive {
        value["application"]["landing"]["projects"][0]["by"] = "queue".into();
    }
    write(&path, &value);
}

/// Render one engine and one external worker from the owned story's documents.
fn split(root: &Path) -> PathBuf {
    let path = root.join("temper.json");
    let mut engine = document(&path);
    let address = super::free_addresses::<1>(&[])[0];
    engine["engine"]["hosts"]["listener"] = json!([{"socket":address.to_string()}]);
    let worker_dir = root.join("worker");
    fs::create_dir(&worker_dir).expect("worker directory");
    fs::set_permissions(&worker_dir, fs::Permissions::from_mode(0o700)).expect("private worker directory");
    let mut worker = json!({"workers":engine["workers"].take(),"application":{
        "workers":engine["application"]["workers"].take(),"limits":engine["application"]["limits"]
    }});
    engine["workers"] = json!([]);
    engine["application"]["workers"] = json!([]);
    worker["workers"][0]["engine"] = json!({"socket":address.to_string()});
    worker["workers"][0]["secret"] = root.join("worker.secret").display().to_string().into();
    worker["application"]["workers"][0]["root"] = root.join("worker-root").display().to_string().into();
    // The wrapper freezes only a prepare merge, before any branch is pushed.
    let git = root.join("held-git");
    let script = format!(
        r#"#!/bin/sh
for arg do
  if [ "$arg" = merge ] && [ -f '{root}/prepare.hold' ]; then
    : > '{root}/prepare.entered'
    while [ -f '{root}/prepare.hold' ]; do sleep 0.02; done
  fi
  if [ "$arg" = push ] && [ -f '{root}/delivery.hold' ]; then
    : > '{root}/delivery.entered'
    while [ -f '{root}/delivery.hold' ]; do sleep 0.02; done
  fi
done
exec /usr/bin/git "$@"
"#,
        root = root.display()
    );
    super::write_private(&git, script.as_bytes());
    fs::set_permissions(&git, fs::Permissions::from_mode(0o700)).expect("executable git wrapper");
    worker["application"]["workers"][0]["git"] = git.display().to_string().into();
    write(&worker_dir.join("temper.json"), &worker);
    write(&path, &engine);
    worker_dir
}

fn peer(root: &Path, forge: &Throwaway, drill: Drill) -> super::llm::Listening {
    let mut shared = super::landing_scripts(super::ANSWER, forge).into_vec();
    let owner = shared.remove(0);
    let Line::Call { arguments, .. } = &owner.turns[0].lines[0] else { panic!("delegate") };
    let delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    shared.retain(|script| !script.cue.starts_with(b"## Task\n\n@coding"));
    let mut scripts = shared;
    for number in 1..=drill.members() {
        let words = format!("@drill-code-{number} Edit line {number}");
        let mut child = delegate["batch"][0].clone();
        child["spec"]["words"] = words.clone().into();
        if drill.is_batch() {
            child["tracked"] = json!({"priority":number});
        }
        let mut goal = owner.clone();
        goal.cue = format!("## Task\n\n@drill-goal-{number}").into_bytes().into();
        goal.turns[0] = call(b"delegate", &json!({"batch":[child]}));
        let mut turns = goal.turns.into_vec();
        turns.push(skein_fake_llm_domain::api::Turn {
            lines: Box::new([Line::Text { text: b"Read the arrived results before waiting again.".as_slice().into() }]),
            finish: skein_fake_llm_domain::api::Finish::Stop,
            tokens: 20,
        });
        goal.turns = turns.into();
        scripts.push(goal);
        scripts.push(Script { cue: format!("## Task\n\n{words}").into_bytes().into(), turns: Box::new([
            call(b"read", &json!({"path":format!("walk/line-{number}")})),
            call(b"write", &json!({"path":format!("walk/line-{number}"),"content":format!("Changed {number}\n")})),
            call(b"finish", &json!({"form":"change","fields":{"title":format!("Drill change {number}"),"body":"Edit one line."}})),
        ]) });
    }
    if drill.is_batch() {
        batch::scripts(&mut scripts);
        if matches!(drill, Drill::Breaking | Drill::Conflict) {
            departures::scripts(&mut scripts, drill);
        }
    }
    if drill == Drill::GoLive {
        go_live::scripts(&mut scripts);
    }
    if drill == Drill::Delivery {
        delivery::scripts(&mut scripts);
    }
    let scripts = temper_real_loop_world::llm::cued(scripts.into_boxed_slice());
    let rules = Rules {
        cued: scripts
            .cued
            .iter()
            .map(|script| -> Box<[Rule]> {
                if script.cue.starts_with(b"## Task\n\n@drill-goal-") {
                    Box::new([
                        Rule { cue: b"Messages arrived. Read them before continuing.".as_slice().into(), turn: 3 },
                        Rule { cue: b"Nothing more will come:".as_slice().into(), turn: 2 },
                        Rule { cue: b"Begin the work your brief describes.".as_slice().into(), turn: 0 },
                        Rule { cue: Box::new([]), turn: 1 },
                    ])
                } else {
                    Box::new([])
                }
            })
            .collect(),
        asked: Box::new([]),
    };
    let peer = super::llm_recipe(recipe(scripts, rules));
    peer.configure(root);
    peer
}

fn recipe(scripts: Scripts, rules: Rules) -> temper_real_loop_world::llm::Recipe {
    let mut recipe = temper_real_loop_world::llm::scripted(Box::new([]));
    recipe.limits.connections = 16;
    recipe.limits.io.sockets = 32;
    recipe.domain_limits.calls = 16;
    // Four held reviews and the readiness latch share the provider's call room.
    recipe.protocol_limits.calls = recipe.domain_limits.calls;
    recipe.domain_limits.strict = true;
    recipe.limits.observations *= 8;
    recipe.limits.observation_bytes *= 8;
    recipe.domain = skein_fake_llm_domain::Domain::played_by_input(
        &recipe.domain_limits,
        17,
        scripts,
        skein_fake_llm_domain::api::Menu { arguments: Box::new([]), invalid: Box::new([]) },
        rules,
    )
    .expect("drill scripts");
    recipe
}

/// The drill's concurrent held model calls fit its byte provider.
pub mod tests {
    use skein_fake_llm_protocol::provider;
    use skein_lib::{
        Env, Queue, Time, Token, Wall,
        stream::{Down, Slice, Up},
    };

    pub fn four_reviews_and_the_latch_can_wait_on_the_model_together() {
        let recipe = super::recipe(
            super::Scripts { cued: Box::new([]), asked: Box::new([]), otherwise: Box::new([]) },
            super::Rules { cued: Box::new([]), asked: Box::new([]) },
        );
        let credentials = [skein_llm::Credential::key(b"fixture".as_slice().into())];
        let env = Env { now: Time::ZERO, wall: Wall::from_nanos(0), limits: recipe.protocol_limits };
        let mut service = provider::Service::new(recipe.config, &env.limits).expect("drill provider");
        let body = br#"{"model":"test","instructions":"work","input":[{"role":"user","content":[{"type":"input_text","text":"work"}]}],"stream":true,"store":false}"#;
        assert!(
            skein_fake_llm_protocol::documents::request_on(skein_llm::Provider::OpenAi, body, &env.limits.documents)
                .is_ok()
        );
        let request = format!(
            "POST /v1/responses HTTP/1.1\r\nHost: fixture\r\nAuthorization: Bearer fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).expect("request text")
        );
        let mut servers = Vec::new();
        for number in 0..5 {
            let mut server = provider::Server::new(Token::new(number), &env.limits).expect("model connection");
            let mut above = Queue::uncapped();
            let mut below = Queue::uncapped();
            let mut input = Slice::new(request.as_bytes());
            let mut admitted = false;
            provider::start(&mut server, &mut service, &credentials, &env, &mut above, &mut below);
            for _ in 0..1000 {
                while let Some(event) = above.pop() {
                    if let provider::Event::Domain(skein_fake_llm_domain::Event::Call { .. }) = event {
                        admitted = true;
                    }
                }
                if admitted {
                    break;
                }
                if let Some(Down::Demand { read, room }) = below.pop() {
                    if let Some(event) = input.answer(read).or_else(|| (room > 0).then_some(Up::Room)) {
                        provider::up(&mut server, &mut service, &credentials, &env, event, &mut above, &mut below);
                    }
                } else {
                    provider::resume(&mut server, &mut service, &credentials, &env, &mut above, &mut below);
                }
            }
            assert!(admitted, "held model call {} must reach the script domain", number + 1);
            servers.push(server);
        }
        assert_eq!(service.calls(), 5, "four reviews and the latch remain pending together");
    }
}

fn setup(root: &Path, drill: Drill) -> Vec<ctl::Step> {
    let mut steps = turn_restart::setup(root);
    steps
        .push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "template", "1", "--json"]).in_directory("private")));
    steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "--json"]).in_directory("private")));
    let file = root.join("drill-template.json");
    steps.push(ctl::Step::Then(Box::new(move |calls| {
        ctl::setting_change_file(&file, &calls[calls.len() - 2], |value| {
            value["number"] = 2.into();
            value["name"] = json!({"bytes":"turn drill goal","cut":false});
            value["flags"]["conversation"] = false.into();
            value["flags"]["listed"] = true.into();
            value["flags"]["ended_by"] = "result".into();
            value["tracked"] = 1.into();
            value["deadline"] = json!({"span":3_600_000_000_000_u64});
            value["label"] = 2.into();
        });
        let mut change = document(&file);
        change["items"][0]["setting"]["number"] = 2.into();
        write(&file, &change);
        ctl::settings_change(calls.last().expect("settings revision"), &file, "drill-template").in_directory("private")
    })));
    for charter in [1, 2, 4] {
        steps.push(ctl::Step::Run(
            ctl::Call::new(&["settings", "show", "charter", &charter.to_string(), "--json"]).in_directory("private"),
        ));
    }
    steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "--json"]).in_directory("private")));
    let file = root.join("drill-charters.json");
    steps.push(ctl::Step::Then(Box::new(move |calls| {
        let mut items = Vec::new();
        for ended in &calls[calls.len() - 4..calls.len() - 1] {
            let packet = ctl::json(ended);
            let mut value = packet["value"].clone();
            assert!(value["time"].is_number(), "charter time");
            value["time"] = 270_000_000_000_u64.into();
            items.push(json!({"setting":packet["setting"],"set":value}));
        }
        write(&file, &json!({"items":items}));
        ctl::settings_change(calls.last().expect("settings revision"), &file, "drill-agent-deadlines")
            .in_directory("private")
    })));
    if drill == Drill::GoLive {
        go_live::setup(root, &mut steps);
    }
    steps
}

struct Judge<'a> {
    root: &'a Path,
    worker: PathBuf,
    drill: Drill,
    delivery_armed: bool,
    go_live_revision: Option<u64>,
    go_live_call: Option<usize>,
    go_live_applied: bool,
    go_live_seed_checked: bool,
    batch: batch::Batch,
    forge: &'a Throwaway,
    barrier: &'a JobBarrier,
    stage: Stage,
    number: u64,
    panel: Option<Value>,
    run: Option<u64>,
    poll: Time,
    deadline: super::patience::Patience,
    failure: Option<String>,
    failed_at: Option<Time>,
    pending: Option<(Vec<String>, usize)>,
    transient: Vec<usize>,
    reads: BTreeMap<Vec<String>, Value>,
    setup_count: usize,
    checked: usize,
    models_checked: usize,
    restarts: u32,
}
fn calls(procs: &mut [Process]) -> &mut ctl::Calls {
    let Process::Calls(calls) = &mut procs[2] else { panic!("ctl actor") };
    calls
}
fn binary(procs: &mut [Process], index: usize) -> &mut skein_world::end_to_end::Binary {
    match &mut procs[index] {
        Process::Binary(binary) => binary,
        Process::Member(member) => member.binary_mut(),
        _ => panic!("binary actor"),
    }
}
impl Judge<'_> {
    fn read(&mut self, procs: &mut [Process], args: &[&str]) -> Option<Value> {
        let key: Vec<String> = args.iter().map(|s| (*s).into()).collect();
        if let Some(value) = self.reads.remove(&key) {
            return Some(value);
        }
        if self.pending.is_none() && calls(procs).foreground_idle() {
            let index = calls(procs).ends().len();
            calls(procs).push(ctl::Step::Run(ctl::Call::new(args).in_directory("private").json()));
            self.pending = Some((key, index));
        }
        None
    }
    fn drive(&mut self, procs: &mut [Process]) -> Result<(), String> {
        let Some((key, index)) = self.pending.as_ref() else { return Ok(()) };
        let Some(ended) = calls(procs).ends().get(*index) else { return Ok(()) };
        if ended.exit != Some(Exit::Code(0)) {
            if (matches!(self.stage, Stage::Resume | Stage::GoLiveApply)
                || (self.stage == Stage::Batch && self.batch.starting()))
                && String::from_utf8_lossy(&ended.stderr).contains("nothing listens")
            {
                self.transient.push(*index);
                self.pending = None;
                return Ok(());
            }
            return Err(format!(
                "ctl {key:?}: {} {}",
                String::from_utf8_lossy(&ended.stdout),
                String::from_utf8_lossy(&ended.stderr)
            ));
        }
        self.reads.insert(key.clone(), super::check_ctl_json(ended)?);
        self.pending = None;
        Ok(())
    }
    fn start_goal(&mut self, procs: &mut [Process]) {
        if self.drill == Drill::Phases && self.number == 1 {
            super::write_private(&self.root.join("prepare.hold"), b"");
        }
        calls(procs).push(ctl::Step::Run(
            ctl::Call::new(&[
                "goal",
                "start",
                "--project",
                "1",
                "--template",
                "2",
                &format!("@drill-goal-{} Edit one line", self.number),
                "--key",
                &format!("drill-goal-{}", self.number),
            ])
            .in_directory("private"),
        ));
        self.panel = None;
        self.run = None;
        eprintln!("drill {:?}: started goal {}", self.drill, self.number);
        self.stage = Stage::Observe;
    }
    fn observe(&mut self, procs: &mut [Process]) -> Result<(), String> {
        if self.drill == Drill::GoLive {
            Self::release_go_live(procs)?;
        }
        let task = if let Some(panel) = &self.panel {
            panel["task"].as_u64().ok_or("landing task")?
        } else {
            let Some(list) = self.read(procs, &["landing", "list", "--project", "1", "--ended"]) else { return Ok(()) };
            let Some(row) =
                list["landings"].as_array().ok_or("landing list")?.iter().find(|row| row["phase"] != "landed")
            else {
                return Ok(());
            };
            self.panel = Some(row.clone());
            row["task"].as_u64().ok_or("landing task")?
        };
        let Some(panel) = self.read(procs, &["landing", "show", &task.to_string()]) else { return Ok(()) };
        if self.drill == Drill::Delivery {
            self.panel = Some(panel.clone());
            return self.delivery(procs, &panel);
        }
        if self.drill == Drill::GoLive {
            self.panel = Some(panel.clone());
            if panel["phase"] == "proving"
                && let Some(run) = self.barrier.waiting(None)
            {
                self.run = Some(run);
                self.barrier.release(run);
                self.stage = Stage::Resume;
            }
            return Ok(());
        }
        let phase = ["preparing", "proving", "turn"][usize::try_from(self.number - 1).expect("three phases")];
        if self.number == 3
            && panel["phase"] == "proving"
            && let Some(run) = self.barrier.waiting(None)
        {
            self.barrier.release(run);
            self.run = Some(run);
        }
        if panel["phase"] == "landed" {
            return Err(format!("missed restart phase {phase}: {panel}"));
        }
        self.panel = Some(panel.clone());
        if panel["phase"] != phase {
            return Ok(());
        }
        if self.number == 1 && !self.root.join("prepare.entered").exists() {
            return Ok(());
        }
        if self.number == 2 {
            let Some(run) = self.barrier.waiting(None) else { return Ok(()) };
            self.run = Some(run);
        }
        eprintln!("drill 5: restart engine in {phase}: {panel}");
        binary(procs, 0).signal(Signal::Kill);
        self.stage = Stage::Restart;
        Ok(())
    }
    fn stop(&mut self, procs: &mut [Process]) {
        binary(procs, 0).signal(Signal::Terminate);
        binary(procs, 3).signal(Signal::Terminate);
        calls(procs).release();
        for proc in procs {
            if let Process::Peer(peer) = proc {
                peer.shutdown();
            }
        }
        self.stage = Stage::Stop;
    }
    fn check(&mut self, procs: &mut [Process]) -> Result<(), String> {
        if self.stage != Stage::Restart && self.stage != Stage::Stop && binary(procs, 0).exit_status().is_some() {
            return Err(format!("engine exited: {}", super::without_secrets(binary(procs, 0).stderr())));
        }
        if !matches!(self.stage, Stage::Stop | Stage::WorkerStopping | Stage::Restart)
            && binary(procs, 3).exit_status().is_some()
        {
            return Err(format!("worker exited: {}", super::without_secrets(binary(procs, 3).stderr())));
        }
        for proc in procs.iter() {
            if let Process::Peer(peer) = proc {
                for event in peer.observations().iter().skip(self.models_checked) {
                    if let skein_fake_peers::llm::Observation::Answered { result: Err(why), .. } = event {
                        for event in peer.observations().iter().rev().take(12) {
                            if let skein_fake_peers::llm::Observation::Query { query, .. } = event {
                                let cues: Vec<_> = [
                                    "Resolve the merge conflict",
                                    "Repair the failed landing gate",
                                    "Review this change at the named head",
                                    "@drill-goal-1",
                                    "@drill-goal-2",
                                    "@drill-code-2",
                                    "@drill-latch",
                                ]
                                .iter()
                                .filter_map(|cue| {
                                    skein_lib::bytes::find(&query.system, cue.as_bytes()).map(|at| (*cue, at))
                                })
                                .collect();
                                eprintln!("model query: cues {cues:?}, {} messages", query.messages.len());
                            }
                        }
                        return Err(format!("fake model refused a scripted query: {why:?}"));
                    }
                    if let skein_fake_peers::llm::Observation::Query { query, .. } = event {
                        for part in query.messages.iter().flat_map(|message| message.parts.iter()) {
                            if let skein_fake_llm_domain::api::Part::ToolOutput { output, is_error: true, .. } = part {
                                return Err(format!("scripted tool refused: {}", String::from_utf8_lossy(output)));
                            }
                        }
                    }
                }
                self.models_checked = peer.observations().len();
            }
        }
        self.drive(procs)?;
        for (index, ended) in calls(procs).ends().iter().enumerate() {
            if ended.exit.is_some_and(|exit| exit != Exit::Code(0)) {
                if self.transient.contains(&index) {
                    continue;
                }
                return Err(format!(
                    "ctl failed: {} {}",
                    String::from_utf8_lossy(&ended.stdout),
                    String::from_utf8_lossy(&ended.stderr)
                ));
            }
        }
        Ok(())
    }
    #[expect(clippy::too_many_lines, reason = "the process phases retain their observations across each restart")]
    fn sample(&mut self, procs: &mut [Process]) -> Result<(), String> {
        self.check(procs)?;
        match self.stage {
            Stage::Setup => {
                if calls(procs).ends().len() < self.setup_count {
                    return Ok(());
                }
                for ended in calls(procs).ends() {
                    if ended.exit != Some(Exit::Code(0)) {
                        return Err(format!(
                            "setup: {} {}",
                            String::from_utf8_lossy(&ended.stdout),
                            String::from_utf8_lossy(&ended.stderr)
                        ));
                    }
                }
                let link = calls(procs).link().to_vec();
                let Process::Person(person) = &mut procs[1] else { return Err("person".into()) };
                person.ctl_ready(&link);
                self.stage = Stage::Start;
            }
            Stage::Start => {
                let Process::Person(person) = &procs[1] else { return Err("person".into()) };
                if !person.finished {
                    return Ok(());
                }
                if self.checked == 0 {
                    calls(procs).push(ctl::Step::Run(
                        ctl::Call::new(&[
                            "repository",
                            "adopt",
                            "--project",
                            "1",
                            "--forge",
                            "1",
                            "--owner",
                            "acme",
                            "--name",
                            "widget",
                            "--landing",
                            "main",
                            "--key",
                            "drill-adopt",
                        ])
                        .in_directory("private"),
                    ));
                    self.checked = 1;
                }
                if calls(procs).foreground_idle() {
                    if self.drill.is_batch() {
                        self.begin_batch(procs);
                    } else {
                        self.start_goal(procs);
                    }
                }
            }
            Stage::Observe => {
                if self.drill == Drill::GoLive && !self.go_live_applied {
                    self.go_live_cut(procs)?;
                } else {
                    self.observe(procs)?;
                }
            }
            Stage::Restart => {
                if !binary(procs, 0).is_empty() {
                    return Ok(());
                }
                if binary(procs, 0).exit_status() != Some(Exit::Signal(9)) {
                    return Err(format!("engine killed: {:?}", binary(procs, 0).exit_status()));
                }
                procs[0] = Process::Member(Box::new(super::cluster::Member::observing(super::serve(self.root))));
                self.restarts += 1;
                if self.drill == Drill::Phases && self.number == 1 {
                    fs::remove_file(self.root.join("prepare.hold")).map_err(|e| e.to_string())?;
                }
                if self.drill == Drill::Delivery {
                    procs[3] = Process::Member(Box::new(super::cluster::Member::observing(super::serve(&self.worker))));
                    self.barrier.release(self.run.ok_or("held gate")?);
                }
                if self.drill == Drill::Phases && self.number == 2 {
                    self.barrier.release(self.run.ok_or("held gate")?);
                }
                self.stage = if self.drill.is_batch() {
                    Stage::Batch
                } else if self.drill == Drill::GoLive {
                    Stage::GoLiveApply
                } else {
                    Stage::Resume
                };
            }
            Stage::Resume => {
                let task = self.panel.as_ref().ok_or("retained landing")?["task"].as_u64().ok_or("task")?;
                let Some(panel) = self.read(procs, &["landing", "show", &task.to_string()]) else { return Ok(()) };
                if let Some(run) = self.barrier.waiting(None) {
                    self.barrier.release(run);
                }
                if panel["phase"] == "landed" {
                    self.panel = Some(panel);
                    self.stage = Stage::Landed;
                }
            }
            Stage::Landed => {
                let panel = self.panel.as_ref().ok_or("landed panel")?.clone();
                let change = panel["members"][0]["change"].as_u64().ok_or("member")?;
                let Some(change) = self.read(procs, &["change", "show", &change.to_string()]) else { return Ok(()) };
                if change["step"] != "landed" {
                    return Ok(());
                }
                let pulls = self.forge.repository_observation("acme", "widget", "pulls?state=all&limit=100");
                let pulls: Vec<_> = pulls
                    .as_array()
                    .ok_or("pulls")?
                    .iter()
                    .filter(|pull| pull["title"] == format!("Drill change {}", self.number))
                    .collect();
                if pulls.len() != 1 {
                    return Err(format!("one member pull for goal {}: {pulls:?}", self.number));
                }
                if pulls[0]["merged"] != true {
                    return Ok(());
                }
                if change["repairs"]["used"] != 0
                    || change["resolutions"]["used"] != 0
                    || !change["hold"].is_null()
                    || !panel["hold"].is_null()
                    || panel["preparation"] != 1
                {
                    return Err(format!("one preparation and no restart tries or hold: {panel}; {change}"));
                }
                eprintln!("drill {:?}: resumed and landed goal {}, landing {}", self.drill, self.number, panel["task"]);
                if self.drill == Drill::GoLive {
                    self.stage = Stage::Verify;
                } else if self.drill == Drill::Delivery {
                    if self.number == 1 {
                        self.number = 2;
                        self.stage = Stage::DeliveryRest;
                    } else {
                        self.stage = Stage::Verify;
                    }
                } else if self.number < 3 {
                    self.number += 1;
                    self.stage = Stage::Start;
                } else {
                    self.stage = Stage::Verify;
                }
            }
            Stage::Verify => {
                let Some(list) = self.read(procs, &["landing", "list", "--project", "1", "--ended"]) else {
                    return Ok(());
                };
                let rows = list["landings"].as_array().ok_or("landings")?;
                let expected = if self.drill == Drill::Phases {
                    3
                } else if self.drill == Drill::GoLive {
                    1
                } else {
                    2
                };
                if rows.len() != expected || rows.iter().any(|row| row["phase"] != "landed") {
                    return Err(format!("exactly {expected} landings: {list}"));
                }
                let runs = self.forge.repository_observation("acme", "widget", "actions/runs?limit=100");
                if runs["workflow_runs"]
                    .as_array()
                    .ok_or("gate runs")?
                    .iter()
                    .filter(|run| run["workflow_id"] == "gate.yaml")
                    .count()
                    != expected
                {
                    return Err(format!("one dispatch per landing: {runs}"));
                }
                if self.drill == Drill::Delivery {
                    self.file_result("walk/line-2", b"Saved work\nDelivered work\n")?;
                }
                self.stop(procs);
            }
            Stage::GoLiveApply => self.go_live_apply(procs)?,
            Stage::GoLiveStays => self.go_live_stays(procs)?,
            Stage::Batch => self.batch(procs)?,
            Stage::WorkerStopping => self.stopped_worker(procs)?,
            Stage::DeliveryRest => self.remaining_delivery(procs)?,
            Stage::Stop => {}
        }
        Ok(())
    }
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if self.failure.is_some() {
            if self.failed_at.is_none() {
                self.failed_at = Some(now);
                for index in [0, 3] {
                    if binary(procs, index).exit_status().is_none() {
                        binary(procs, index).signal(Signal::Terminate);
                    }
                }
            }
            calls(procs).cancel();
            if let Some(run) = self.barrier.waiting(None) {
                self.barrier.release(run);
            }
            for proc in procs {
                if let Process::Peer(peer) = proc {
                    for name in [
                        b"drill-repair".as_slice(),
                        b"drill-batch-reviews",
                        b"drill-batch-ready",
                        b"drill-saved",
                        b"drill-second",
                        b"drill-go-live",
                    ] {
                        let _released = peer.release(name);
                    }
                    peer.shutdown();
                }
            }
            self.poll = now.saturating_add(Duration::from_millis(10));
            return;
        }
        if now < self.poll {
            return;
        }
        // The turn cut needs a short observation interval; stable waits do not.
        let millis =
            if self.drill == Drill::Phases && self.number == 3 && self.stage == Stage::Observe { 100 } else { 1000 };
        self.poll = now.saturating_add(Duration::from_millis(millis));
        if let Err(why) = self.sample(procs) {
            self.failure = Some(why);
        }
    }
    fn watch(&mut self, now: Time, procs: &[Process]) -> Result<(), String> {
        if self.failure.is_some()
            && !procs.iter().all(Host::is_empty)
            && self.failed_at.is_none_or(|at| now.saturating_since(at) < Duration::from_secs(20))
        {
            return Ok(());
        }
        if let Some(why) = self.failure.take() {
            eprintln!("drill {:?}: stage {:?}; batch {:?}", self.drill, self.stage, self.batch);
            for (index, proc) in procs.iter().enumerate() {
                let binary = match proc {
                    Process::Binary(binary) => Some(binary.as_ref()),
                    Process::Member(member) => Some(member.binary()),
                    _ => None,
                };
                if let Some(binary) = binary {
                    eprintln!(
                        "process {index} {:?}: {}",
                        binary.exit_status(),
                        super::without_secrets(binary.stderr())
                    );
                }
            }
            return Err(why);
        }
        if self.stage == Stage::Stop && procs.iter().all(Host::is_empty) {
            for index in [0, 3] {
                let Process::Member(member) = &procs[index] else {
                    return Err("a restarted engine and external worker".into());
                };
                if member.binary().exit_status() != Some(Exit::Code(0)) {
                    return Err(format!(
                        "process {index} did not drain: {:?}: {}",
                        member.binary().exit_status(),
                        super::without_secrets(member.binary().stderr())
                    ));
                }
            }
        }
        if let Process::Person(person) = &procs[1]
            && let Some(why) = &person.failed
        {
            return Err(why.clone());
        }
        if let Some(figures) = self.deadline.failed() {
            for (i, proc) in procs.iter().enumerate() {
                let binary = match proc {
                    Process::Binary(binary) => Some(binary.as_ref()),
                    Process::Member(member) => Some(member.binary()),
                    _ => None,
                };
                if let Some(binary) = binary {
                    eprintln!("binary {i}: {}", super::without_secrets(binary.stderr()));
                }
            }
            return Err(format!(
                "drill {:?} goal {} stalled: {figures}; panel {:?}",
                self.stage, self.number, self.panel
            ));
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.poll)
    }
    fn passed(&self) -> bool {
        self.stage == Stage::Stop
            && self.failure.is_none()
            && self.restarts == if self.drill == Drill::Phases { 3 } else { 1 }
    }
}

/// Run a staging drill on an owned Forgejo, with separate engine and worker processes.
pub fn run(drill: Drill) {
    // A cut after an attempt's save can owe its whole finding lifetime
    // (skein/docs/design/forge/client.md, 7.5), beside the other two landings.
    let within = if drill == Drill::Phases {
        super::LANDING.saturating_add(turn_restart::STAGING_LIFETIME)
    } else {
        super::LANDING
    };
    let scratch = Scratch::new(&format!("turn-drill-{drill:?}"));
    let root = scratch.path();
    let barrier = JobBarrier::new(root);
    let forge = turn_restart::start(&scratch);
    let workflow =
        barrier.gate_workflow(&super::landing_by_turn::WORKFLOW.replace("&& sh check.sh", "&& test ! -e walk/red"));
    forge.add_file(".forgejo/workflows/gate.yaml", workflow.as_bytes());
    for number in 1..=4 {
        forge.add_file(&format!("walk/line-{number}"), b"Before\n");
    }
    super::configuration(root, &forge.forge());
    configure(root, forge.repository(), drill);
    let peer = peer(root, &forge, drill);
    let worker = split(root);
    let mut binary = super::serve(root);
    let streams = binary.take_streams().expect("engine pipes");
    let steps = setup(root, drill);
    let setup_count = steps.len();
    let clock = Clock::new();
    let mut world = real::World::new(Judge {
        root,
        worker: worker.clone(),
        drill,
        delivery_armed: false,
        go_live_revision: None,
        go_live_call: None,
        go_live_applied: false,
        go_live_seed_checked: false,
        batch: batch::Batch::default(),
        forge: &forge,
        barrier: &barrier,
        stage: Stage::Setup,
        number: 1,
        panel: None,
        run: None,
        poll: clock.now().now,
        deadline: super::patience::Patience::watching("staging restart phases", within, &binary),
        failure: None,
        failed_at: None,
        pending: None,
        transient: Vec::new(),
        reads: BTreeMap::new(),
        setup_count,
        checked: 0,
        models_checked: 0,
        restarts: 0,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    let person_root = skein_shell::open_root(root).expect("person root");
    world.spawn_with_fds(vec![person_root], || {
        Process::Person(Box::new(person::Person::waiting_for_ctl_with_key(
            person_root,
            person::answered_chat(b"Hello", super::ANSWER),
        )))
    });
    let ctl_root = skein_shell::open_root(root).expect("ctl root");
    let mut fds = streams.descriptors();
    fds.push(ctl_root);
    world.spawn_with_fds(fds, || {
        let mut calls = ctl::Calls::new(ctl_root, steps, Some(&streams), None);
        calls.hold();
        Process::Calls(Box::new(calls))
    });
    let worker_binary = super::cluster::Member::observing(super::serve(&worker));
    world.spawn_with_fds(worker_binary.descriptors(), || Process::Member(Box::new(worker_binary)));
    peer.spawn(&mut world);
    let outcome = world.run(&clock, within.saturating_add(Duration::from_secs(10)));
    assert!(outcome.procs.iter().all(Host::is_empty), "all drill actors drained");
    forge.stop();
    scratch.remove();
}
