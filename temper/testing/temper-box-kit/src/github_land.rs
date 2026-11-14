//! A chat lands on a connected App and finds the same landing after restart (testing/process-tier.md).
use super::{
    Scratch, WITHIN, cluster, ctl,
    github::{self, App, Scan, make_app, request_url},
};
use serde_json::Value;
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

const INSTALLATION: u64 = 1;
const CHANGE: u64 = 2; // Fresh deployment: chat 1, change 2, coder 3, reviewer 4.
const EDIT: &[u8] = b"Changed by temper.\n";

pub fn restart() {
    let clock = Clock::new();
    let began = clock.now().now;
    let scratch = Scratch::new("github_land");
    let directory = skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/github");
    let mut description = cluster::Cluster::read_from(scratch.path(), &directory);
    let [clients, workers] = super::free_addresses(&[]);
    description.bind("story.clients", &clients.to_string());
    description.prepare("forge");
    let mut fixture: Value = serde_json::from_str(
        &fs::read_to_string(description.dir("forge").join("github.json")).expect("GitHub fixture"),
    )
    .expect("GitHub JSON");
    // The document already admits an account with no installation. Retain
    // acme/app, but make the person's App and its installation through HTTP.
    fixture["github"]["accounts"][0]["installation"] = Value::Null;
    fs::write(description.dir("forge").join("github.json"), serde_json::to_vec(&fixture).expect("fixture"))
        .expect("uninstalled fixture");
    let key = description.dir("llm").join("llm.key");
    let observations = description.dir("forge").join("observations");
    let engine = description.dir("engine").join("config/temper.json");
    std::os::unix::fs::symlink(skein_scratch::paths::binary("temper-box"), description.dir("w1").join("git"))
        .expect("worker's fake git");
    let (mut members, hosts) = description.members();
    members.up(&["forge"]);
    let mut world = real::World::new(Judge {
        members,
        finding: None,
        deadline: clock.now().now.saturating_add(WITHIN),
        stage: Stage::Forge,
        scratch: scratch.path().to_path_buf(),
        clients,
        workers,
        app: None,
        identity: None,
        restored: None,
        outputs: Vec::new(),
        restart_began: None,
        key,
        observations,
        engine,
        client: scratch.path().join("client"),
        began,
        ready: BTreeSet::new(),
        call_marks: 0,
        landed: None,
        panel: None,
        tasks: None,
        life_tasks: Vec::new(),
        reply_snapshot: None,
        replied: false,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(cluster::Root::client);
    let outcome = world.run(&clock, WITHIN);
    assert!(outcome.procs.iter().all(Host::is_empty), "every process and client settled");
    scratch.remove();
    eprintln!("github land: complete {:?}", clock.now().now.saturating_since(began));
}

#[derive(Debug)]
enum Stage {
    Forge,
    Fakes,
    Worker,
    Install,
    Adopt,
    Chat,
    Change,
    ReplySnapshot,
    ReplyPages,
    ChatEnded,
    ChatEvidence,
    BeforeRestart,
    StopWorker,
    StopEngine,
    RestartEngine,
    RestartWorker,
    Restored,
    LandingRestored,
    TasksPage { restored: bool },
    Stopping,
}
struct Judge {
    members: cluster::Members,
    finding: Option<String>,
    deadline: Time,
    stage: Stage,
    scratch: PathBuf,
    clients: SocketAddr,
    workers: SocketAddr,
    app: Option<App>,
    identity: Option<Value>,
    restored: Option<Value>,
    outputs: Vec<Vec<u8>>,
    restart_began: Option<Time>,
    key: PathBuf,
    observations: PathBuf,
    engine: PathBuf,
    client: PathBuf,
    began: Time,
    ready: BTreeSet<&'static str>,
    call_marks: usize,
    landed: Option<Value>,
    panel: Option<Value>,
    tasks: Option<Vec<u64>>,
    life_tasks: Vec<u64>,
    reply_snapshot: Option<String>,
    replied: bool,
}
impl Judge {
    #[expect(clippy::too_many_lines, reason = "the cluster startup and restart share one ordered script")]
    fn advance(&mut self, now: Time, hosts: &mut [cluster::Root]) -> Result<(), String> {
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = Stage::Stopping;
            return Ok(());
        }
        let host = hosts.last_mut().ok_or_else(|| "client root".to_owned())?;
        if matches!(self.stage, Stage::Forge) && self.members.is_up() {
            self.members.prepare("llm");
            let path = self.members.dir("llm").join("llm.json");
            configure_llm(&path);
            let mut value: Value = serde_json::from_slice(&fs::read(&path).map_err(|_| "LLM fixture".to_owned())?)
                .map_err(|_| "LLM JSON".to_owned())?;
            value["asked"][0]["turns"].as_array_mut().ok_or_else(|| "chat turns".to_owned())?.insert(
                2,
                serde_json::json!({"lines":[{"call":"read_pull_for","arguments":{"head":"temper/1/c2","base":"main"}}],"finish":"tool_calls","tokens":20}),
            );
            super::write_private(&path, &serde_json::to_vec(&value).map_err(|_| "LLM JSON".to_owned())?);
            self.members.up(&["llm"]);
            self.stage = Stage::Fakes;
        } else if matches!(self.stage, Stage::Fakes) && self.members.is_up() {
            self.mark("fakes ready");
            let api = self
                .members
                .capture("forge", "address")
                .ok_or_else(|| "forge address".to_owned())?
                .parse()
                .map_err(|_| "API socket".to_owned())?;
            self.app = Some(make_app(api, self.clients, &self.scratch)?);
            self.mark("App made by manifest and conversion");
            if self.members.dir("engine").join("data/state").exists() {
                return Err("Must: no seeded forge records".to_owned());
            }
            self.members.prepare("engine");
            let mut document: Value =
                serde_json::from_slice(&fs::read(&self.engine).map_err(|_| "engine configuration".to_owned())?)
                    .map_err(|_| "engine JSON".to_owned())?;
            // Restart retains the worker's files. Reserve its engine endpoint,
            // as the clients' endpoint is reserved for the App's setup URL.
            document["engine"]["hosts"]["listener"][0]["socket"] = self.workers.to_string().into();
            let address = self
                .members
                .capture("forge", "address")
                .ok_or_else(|| "forge address".to_owned())?
                .parse()
                .map_err(|_| "socket".to_owned())?;
            configure_engine(&mut document);
            fs::create_dir_all(&self.client).map_err(|_| "client directory".to_owned())?;
            super::write_private(
                &self.client.join("landing-settings.json"),
                &serde_json::to_vec(&landing_settings(address, "acme", "app"))
                    .map_err(|_| "landing settings".to_owned())?,
            );
            fs::write(&self.engine, serde_json::to_vec(&document).map_err(|_| "engine JSON".to_owned())?)
                .map_err(|_| "fixed workers listener".to_owned())?;
            let directory = self.members.dir("engine");
            super::forgejo::bootstrap(&directory.join("config"), &directory.join("data"), &directory.join("cache"));
            // temper/docs/design/protocol/setup/github.md, 7; temper/docs/design/protocol/forge/engine.md, 1.
            let mut record = temper_real_loop_world::fixtures::forgejo_record(address);
            record.kind = temper_engine_domain_forge::Kind::GitHub;
            record.place.web = Some(format!("http://{address}").into_bytes().into());
            record.bot.login = b"x-access-token".as_slice().into();
            record.read_only = true;
            temper_real_loop_world::fixtures::seed_record(&directory.join("data/state"), record, &[]);
            self.members.up(&["w1"]);
            self.stage = Stage::Worker;
            self.members.act(now, hosts);
        } else if matches!(self.stage, Stage::Worker) && self.members.is_up() {
            self.mark("engine and worker ready");
            self.begin(host)?;
        } else if matches!(self.stage, Stage::StopWorker) && self.members.ended("w1") {
            self.members.stop_one("engine");
            self.stage = Stage::StopEngine;
        } else if matches!(self.stage, Stage::StopEngine) && self.members.ended("engine") {
            self.members.restart("engine");
            self.stage = Stage::RestartEngine;
        } else if matches!(self.stage, Stage::RestartEngine) && self.members.capture("engine", "link").is_some() {
            self.members.restart("w1");
            self.stage = Stage::RestartWorker;
        } else if matches!(self.stage, Stage::RestartWorker) && self.members.is_up() {
            self.mark("engine and worker restarted on ready lines");
            eprintln!(
                "github land restart: ready {:?}",
                now.saturating_since(self.restart_began.ok_or_else(|| "restart began".to_owned())?)
            );
            if self.members.linked("w1") != 2 {
                return Err("Must: worker linked in both lives".to_owned());
            }
            self.queue(
                host,
                vec![
                    ctl::Call::new(&["forge", "list", "--until", "connected:1"]),
                    ctl::Call::new(&["forge", "list", "--json"]),
                ],
            )?;
            self.stage = Stage::Restored;
        } else if host.is_empty()
            && matches!(
                self.stage,
                Stage::Install
                    | Stage::Adopt
                    | Stage::Chat
                    | Stage::Change
                    | Stage::ReplySnapshot
                    | Stage::ReplyPages
                    | Stage::ChatEnded
                    | Stage::ChatEvidence
                    | Stage::BeforeRestart
                    | Stage::Restored
                    | Stage::LandingRestored
                    | Stage::TasksPage { .. }
            )
        {
            let cluster::Root::Client { calls, .. } = host else { return Err("client root".to_owned()) };
            let calls = calls.take().ok_or_else(|| "completed calls".to_owned())?;
            self.called(now, host, &calls)?;
        }

        Ok(())
    }

    fn mark(&self, step: &str) {
        eprintln!("github land: {step} {:?}", Clock::new().now().now.saturating_since(self.began));
    }
    fn queue(&mut self, host: &mut cluster::Root, calls: Vec<ctl::Call>) -> Result<(), String> {
        self.queue_steps(host, calls.into_iter().map(ctl::Step::Run).collect())?;

        Ok(())
    }
    fn queue_steps(&mut self, host: &mut cluster::Root, steps: Vec<ctl::Step>) -> Result<(), String> {
        self.call_marks = 0;
        let link =
            format!("http://{}", self.members.capture("engine", "link").ok_or_else(|| "engine link".to_owned())?);
        let mut batch = ctl::Calls::linked(&link, &self.client);
        for step in steps {
            batch.push(step);
        }
        let cluster::Root::Client { calls, .. } = host else { return Err("client root".to_owned()) };
        *calls = Some(Box::new(batch));

        Ok(())
    }
    fn begin(&mut self, host: &mut cluster::Root) -> Result<(), String> {
        use ctl::Call;
        let engine: Value = serde_json::from_slice(&fs::read(&self.engine).map_err(|_| "engine document".to_owned())?)
            .map_err(|_| "engine JSON".to_owned())?;
        if engine["workers"] != serde_json::json!([]) {
            return Err("engine alone".to_owned());
        }
        if engine["application"]["workers"] != serde_json::json!([]) {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                engine["application"]["workers"],
                serde_json::json!([])
            ));
        }
        let workers = engine["engine"]["hosts"]["workers"].as_array().ok_or_else(|| "worker positions".to_owned())?;
        if workers.len() != 1 {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", workers.len(), 1));
        }
        if workers[0]["name"] != "w1" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                workers[0]["name"], "w1"
            ));
        }
        if self.members.linked("w1") != 1 {
            return Err("the sole worker linked".to_owned());
        }
        self.queue_steps(
            host,
            super::github::settings_setup(
                &self.client,
                vec![
                    Call::new(&[
                        "account",
                        "add-key",
                        "1",
                        self.key.to_str().ok_or_else(|| "key path".to_owned())?,
                        "--deployment",
                        "--key",
                        "github-llm",
                    ]),
                    Call::new(&["account", "list", "--until", "usable"]),
                    Call::new(&[
                        "forge",
                        "connect",
                        "1",
                        "github-app",
                        &self.app.as_ref().ok_or_else(|| "made App".to_owned())?.id.to_string(),
                        self.scratch.join("github-app.pem").to_str().ok_or_else(|| "PEM path".to_owned())?,
                        "--key",
                        "github-app",
                    ]),
                    Call::new(&["forge", "list", "--until", "connected:1"]),
                    Call::new(&["forge", "install", "1", "--key", "github-install"]),
                ],
            ),
        )?;
        self.stage = Stage::Install;

        Ok(())
    }
    fn adoption() -> ctl::Call {
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
            "app",
            "--landing",
            "main",
            "--key",
            "adopt-app",
            "--json",
        ])
    }
    fn row(packet: &Value) -> Result<Value, String> {
        Ok(packet["forges"]
            .as_array()
            .ok_or_else(|| "forges rows".to_owned())?
            .iter()
            .find(|row| row["number"] == 1)
            .ok_or_else(|| "Must: forge 1 is present".to_owned())?
            .clone())
    }
    fn restored(&mut self, host: &mut cluster::Root, packet: &Value) -> Result<(), String> {
        let row = Self::row(packet)?;
        // The account is learned from the token minted after startup. Wait on
        // the document's digest when T1 exposes a row still learning it.
        if row["installations"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty() && rows.iter().any(|row| row["account"].is_null()))
        {
            let digest = packet["digest"].as_str().ok_or_else(|| "forges digest".to_owned())?;
            self.queue(host, vec![ctl::Call::new(&["forge", "list", "--since", digest, "--json"])])?;
            return Ok(());
        }
        self.restored = Some(row);
        self.queue(host, landing_reads())?;
        self.stage = Stage::LandingRestored;

        Ok(())
    }
    fn install(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let output = std::str::from_utf8(&calls.ends().last().ok_or_else(|| "install link".to_owned())?.stdout)
            .map_err(|_| "install output".to_owned())?;
        let url =
            output.lines().find(|line| line.starts_with("http://")).ok_or_else(|| "printed install link".to_owned())?;
        let address = self
            .members
            .capture("forge", "address")
            .ok_or_else(|| "forge address".to_owned())?
            .parse()
            .map_err(|_| "socket".to_owned())?;
        let page = request_url(address, "GET", url, "", b"")?;
        if page.status != 302 {
            return Err("Must: fake install redirects to setup".to_owned());
        }
        let location = page.location.ok_or_else(|| "setup redirect".to_owned())?;
        if !location.starts_with(&format!("http://{}/forges/1/github/setup?", self.clients)) {
            return Err("setup URL".to_owned());
        }
        if !location.contains(&format!("installation_id={INSTALLATION}")) {
            return Err("Must: accepted visited installation id".to_owned());
        }
        let accepted = request_url(self.clients, "GET", &location, "", b"")?;
        if accepted.status != 200 {
            return Err("Must: setup accepted installation".to_owned());
        }
        if skein_lib::bytes::find(&accepted.body, b"temper has the installation; you may close this tab.").is_none() {
            return Err("the setup page says so".to_owned());
        }
        self.outputs.push(accepted.body);
        self.queue(host, vec![ctl::Call::new(&["forge", "list", "--until", "settled"]), Self::adoption()])?;
        self.stage = Stage::Adopt;

        Ok(())
    }
    #[expect(clippy::too_many_lines, reason = "the client script keeps its returning Musts beside each stage")]
    fn called(&mut self, now: Time, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        for (index, ended) in calls.ends().iter().enumerate() {
            self.outputs.push(ended.stdout.clone());
            self.outputs.push(ended.stderr.clone());
            if ended.exit
                != Some(Exit::Code(
                    if index == 6 && matches!(self.stage, Stage::BeforeRestart | Stage::LandingRestored) {
                        2
                    } else {
                        0
                    },
                ))
            {
                return Err(format!(
                    "ctl: {} {}",
                    String::from_utf8_lossy(&ended.stderr),
                    String::from_utf8_lossy(&ended.stdout)
                ));
            }
        }
        match self.stage {
            Stage::Install => {
                self.install(host, calls)?;
            }
            Stage::Adopt => self.adopted(host, calls)?,
            Stage::Chat => {
                self.mark("signed in, key usable, adopted, chat created");
                let packet = json_call(calls.ends().last().ok_or_else(|| "chat creation".to_owned())?)?;
                if packet["outcome"]["done"]["document"]["created"]["first"] != 1 {
                    return Err(format!("{packet}"));
                }
                // The chat's first reply acknowledges its already committed change task.
                self.queue(
                    host,
                    vec![
                        ctl::Call::new(&["task", "show", "1", "--until", "replied"]),
                        ctl::Call::new(&["change", "show", "2", "--until", "landed", "--json"]),
                        ctl::Call::new(&["change", "show", "2", "--json"]),
                        ctl::Call::new(&["task", "show", "2"]),
                        ctl::Call::new(&["task", "show", "2", "--until", "ended", "--json"]),
                    ],
                )?;
                self.stage = Stage::Change;
            }
            Stage::Change => {
                let waited = json_call(calls.ends().get(1).ok_or("completed ctl call")?)?;
                if waited["met"] != true {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        waited["met"], true
                    ));
                }
                let panel = json_call(calls.ends().get(2).ok_or("completed ctl call")?)?;
                if panel["task"] != CHANGE {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        panel["task"], CHANGE
                    ));
                }
                if panel["step"] != "landed" {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        panel["step"], "landed"
                    ));
                }
                if waited["change"]["pull"] != panel["pull"] {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        waited["change"]["pull"], panel["pull"]
                    ));
                }
                if waited["change"]["merge"] != panel["merge"] {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        waited["change"]["merge"], panel["merge"]
                    ));
                }
                if panel["pull"]["number"] != 1 {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        panel["pull"]["number"], 1
                    ));
                }
                let forge = self.members.capture("forge", "address").ok_or_else(|| "forge address".to_owned())?;
                if panel["pull"]["address"] != format!("http://{forge}/acme/app/pull/1") {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        panel["pull"]["address"],
                        format!("http://{forge}/acme/app/pull/1")
                    ));
                }
                self.panel = Some(panel);
                let words = String::from_utf8_lossy(&calls.ends().get(3).ok_or("completed ctl call")?.stdout);
                if !words.contains("result: Landed · pull request #1 · merge commit ") {
                    return Err(format!("{words}"));
                }
                if !words.contains("next: temper ctl DIR change show 2\n") {
                    return Err(format!("{words}"));
                }
                let packet = json_call(calls.ends().last().ok_or_else(|| "landed change".to_owned())?)?;
                if packet["task"]["chip"]["phase"] != "ended" {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        packet["task"]["chip"]["phase"], "ended"
                    ));
                }
                if packet["task"]["result"]["value"]["change"]["kind"] != 1 {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        packet["task"]["result"]["value"]["change"]["kind"], 1
                    ));
                }
                self.landed = Some(packet["task"]["result"].clone());
                self.mark("change landed");
                self.queue(host, vec![ctl::Call::new(&["task", "show", "1", "--json"])])?;
                self.stage = Stage::ReplySnapshot;
            }
            Stage::ReplySnapshot => self.reply_snapshot(host, calls)?,
            Stage::ReplyPages => self.reply_page(host, calls)?,
            Stage::ChatEnded => {
                if json_call(calls.ends().last().ok_or_else(|| "chat ended".to_owned())?)?["task"]["chip"]["phase"]
                    != "ended"
                {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        json_call(calls.ends().last().ok_or_else(|| "chat ended".to_owned())?)?["task"]["chip"]["phase"],
                        "ended"
                    ));
                }
                self.queue(host, vec![ctl::Call::new(&["task", "activity", "1", "--limit", "32", "--json"])])?;
                self.stage = Stage::ChatEvidence;
            }
            Stage::ChatEvidence => self.chat_evidence(host, calls)?,
            Stage::BeforeRestart => self.before_restart(now, host, calls)?,
            Stage::Restored => {
                self.restored(host, &json_call(calls.ends().last().ok_or_else(|| "restored forges".to_owned())?)?)?;
            }
            Stage::LandingRestored => {
                let cursor = self.check_landing(calls)?;
                self.read_tasks(now, host, true, cursor)?;
            }
            Stage::TasksPage { restored } => {
                let packet = json_call(calls.ends().first().ok_or("completed ctl call")?)?;
                let rows = &packet["ended"]["rows"];
                self.collect_tasks(rows)?;
                let cursor = rows["after"]["value"].as_str().map(str::to_owned);
                self.read_tasks(now, host, restored, cursor)?;
            }
            Stage::Forge
            | Stage::Fakes
            | Stage::Worker
            | Stage::StopWorker
            | Stage::StopEngine
            | Stage::RestartEngine
            | Stage::RestartWorker
            | Stage::Stopping => return Err("active calls".to_owned()),
        }

        Ok(())
    }
    fn reply_snapshot(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let packet = json_call(calls.ends().last().ok_or_else(|| "reply snapshot".to_owned())?)?;
        let families = jig_domain_core::ToolFamilies::JIG;
        if packet["authority"]["authority"]["tools"] != families.delegate.0 | families.message.0 | families.read.0 {
            return Err(format!("the default chat inherits Delegate, Message and Read: {packet}"));
        }
        self.reply_snapshot = Some(packet["digest"].as_str().ok_or_else(|| "task digest".to_owned())?.to_owned());
        self.queue(host, vec![ctl::Call::new(&["chat", "show", "1", "--json"])])?;
        self.stage = Stage::ReplyPages;

        Ok(())
    }
    fn chat_evidence(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let activity = json_call(calls.ends().last().ok_or_else(|| "chat activity".to_owned())?)?;
        // The scripted reply says only “The change landed.” The PR's
        // words therefore come from the result the run was given.
        if !activity["rows"].to_string().contains("pull request #1") {
            return Err(format!("chat heard the landed pull: {activity}"));
        }
        if !activity["rows"].to_string().contains("Pull 1 head ") {
            return Err(format!("chat received its read_pull_for answer: {activity}"));
        }
        self.queue(host, landing_reads())?;
        self.stage = Stage::BeforeRestart;

        Ok(())
    }
    fn adopted(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let packet = json_call(calls.ends().last().ok_or_else(|| "adoption".to_owned())?)?;
        if !packet["adopted"].is_object() {
            return Err(format!("Must: first life adopts acme/app: {packet}"));
        }
        self.queue(
            host,
            vec![ctl::Call::new(&[
                "chat",
                "start",
                "--project",
                "1",
                "--key",
                "change",
                "@forge_chat Update README",
                "--json",
            ])],
        )?;
        self.stage = Stage::Chat;

        Ok(())
    }
    fn reply_page(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let page = json_call(calls.ends().first().ok_or("completed ctl call")?)?;
        self.replied |= page["activity"]["rows"].to_string().contains("The change landed.");
        if let Some(before) = page["activity"]["before"]["value"].as_str() {
            self.queue(host, vec![ctl::Call::new(&["chat", "show", "1", "--before", before, "--json"])])?;
        } else if !self.replied {
            self.queue(
                host,
                vec![
                    ctl::Call::new(&[
                        "task",
                        "show",
                        "1",
                        "--until",
                        "replied",
                        "--since",
                        self.reply_snapshot.as_deref().ok_or_else(|| "reply snapshot".to_owned())?,
                        "--json",
                    ]),
                    ctl::Call::new(&["task", "show", "1", "--json"]),
                ],
            )?;
            self.stage = Stage::ReplySnapshot;
        } else {
            self.queue(
                host,
                vec![
                    ctl::Call::new(&["chat", "close", "1", "--key", "close-land", "--json"]),
                    ctl::Call::new(&["task", "show", "1", "--until", "ended", "--json"]),
                ],
            )?;
            self.stage = Stage::ChatEnded;
        }

        Ok(())
    }
    fn before_restart(&mut self, now: Time, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let cursor = self.check_landing(calls)?;
        let row = Self::row(&json_call(calls.ends().get(4).ok_or("completed ctl call")?)?)?;
        if row["state"] != "connected" {
            return Err("Must: connected before restart".to_owned());
        }
        if row["login"] != "temper-person[bot]" {
            return Err("Must: connected as the person's App".to_owned());
        }
        if row["id"].as_u64().is_none() {
            return Err("Must: connected bot id".to_owned());
        }
        self.identity = Some(row);
        self.forge(true)?;
        self.read_tasks(now, host, false, cursor)?;

        Ok(())
    }
    fn collect_tasks(&mut self, rows: &Value) -> Result<(), String> {
        if rows["cut"] != false {
            return Err("whole task page".to_owned());
        }
        for row in rows["rows"].as_array().ok_or("ended rows")? {
            if row["chip"]["phase"] != "ended" {
                return Err(format!(
                    "assertion `left == right` failed: task phase {:?}, expected ended",
                    row["chip"]["phase"]
                ));
            }
            self.life_tasks.push(row["chip"]["task"].as_u64().ok_or("task id")?);
        }

        Ok(())
    }
    fn read_tasks(
        &mut self,
        now: Time,
        host: &mut cluster::Root,
        restored: bool,
        cursor: Option<String>,
    ) -> Result<(), String> {
        if let Some(cursor) = cursor {
            self.queue(
                host,
                vec![ctl::Call::new(&[
                    "task",
                    "list",
                    "--project",
                    "1",
                    "--parent",
                    "2",
                    "--ended",
                    "--after",
                    &cursor,
                    "--json",
                ])],
            )?;
            self.stage = Stage::TasksPage { restored };
            return Ok(());
        }
        self.life_tasks.sort_unstable();
        if self.life_tasks != [1, 2, 3, 4] {
            return Err("the complete task set".to_owned());
        }
        if restored {
            if Some(&self.life_tasks) != self.tasks.as_ref() {
                return Err("no task made after restart".to_owned());
            }
            self.mark("same landing and task set found after restart");
            eprintln!(
                "github land restart: restored {:?}",
                now.saturating_since(self.restart_began.ok_or_else(|| "restart began".to_owned())?)
            );
            self.members.stop();
            self.stage = Stage::Stopping;
        } else {
            self.tasks = Some(self.life_tasks.clone());
            self.mark("first life landed and replied; restarting");
            self.restart_began = Some(now);
            self.members.stop_one("w1");
            self.stage = Stage::StopWorker;
        }

        Ok(())
    }
    fn check_landing(&mut self, calls: &ctl::Calls) -> Result<Option<String>, String> {
        let task = json_call(calls.ends().first().ok_or("completed ctl call")?)?;
        if task["chip"]["phase"] != "ended" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                task["chip"]["phase"], "ended"
            ));
        }
        if task["result"] != *self.landed.as_ref().ok_or_else(|| "first landed result".to_owned())? {
            return Err("same PR and merge commit".to_owned());
        }
        if json_call(calls.ends().get(1).ok_or("completed ctl call")?)?["chip"]["phase"] != "ended" {
            return Err("the Open chat was closed before restart".to_owned());
        }
        let unknown = &calls.ends().get(6).ok_or("completed ctl call")?;
        if unknown.exit != Some(Exit::Code(2)) {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                unknown.exit,
                Some(Exit::Code(2))
            ));
        }
        if unknown.stderr.trim_ascii_end() != br#"{"error":"unknown"}"# {
            return Err("no task beyond the four first-life tasks".to_owned());
        }
        self.life_tasks = vec![1];
        for index in [2, 3] {
            let packet = json_call(calls.ends().get(index).ok_or("completed ctl call")?)?;
            no_live_tasks(&packet)?;
            let rows = &packet["ended"]["rows"];
            if index == 2 {
                if !rows["after"].is_null() {
                    return Err("complete single change delegate".to_owned());
                }
                if rows["rows"].as_array().ok_or_else(|| "ended rows".to_owned())?.len() != 1 {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        rows["rows"].as_array().ok_or_else(|| "ended rows".to_owned())?.len(),
                        1
                    ));
                }
                if rows["rows"][0]["chip"]["task"] != CHANGE {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        rows["rows"][0]["chip"]["task"], CHANGE
                    ));
                }
            }
            self.collect_tasks(rows)?;
        }
        no_live_roots(&json_call(calls.ends().get(5).ok_or("completed ctl call")?)?)?;
        let packet = json_call(calls.ends().get(3).ok_or("completed ctl call")?)?;
        Ok(packet["ended"]["rows"]["after"]["value"].as_str().map(str::to_owned))
    }
    #[expect(clippy::too_many_lines, reason = "the forge observations keep their returning Musts in order")]
    fn forge(&self, read_objects: bool) -> Result<(), String> {
        let lines = fs::read_to_string(&self.observations).map_err(|_| "forge observations".to_owned())?;
        let rows: Vec<Value> = lines
            .lines()
            .map(|line| serde_json::from_str(line).map_err(|_| "observation JSON".to_owned()))
            .collect::<Result<_, _>>()?;
        if rows.last().ok_or_else(|| "checkpoint".to_owned())?["observations_lost"] != 0 {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                rows.last().ok_or_else(|| "checkpoint".to_owned())?["observations_lost"],
                0
            ));
        }
        let calls: Vec<_> = rows.iter().filter(|row| row["observation"] == "git_call").collect();
        if !calls.iter().any(|row| row["writes"] == false && row["status"] == 200) {
            return Err("clone/read observed".to_owned());
        }
        if calls.iter().filter(|row| row["writes"] == true).count() != 1 {
            return Err("Must: exactly one push across both lives".to_owned());
        }
        for call in calls {
            // The record's closed schema is the token-secrecy proof: no string/value field.
            let decoded = temper_fake::forge::observations::decode(call.to_string().as_bytes())
                .map_err(|_| "closed git record".to_owned())?;
            if !matches!(decoded, Some(temper_fake::forge::observations::Observation::GitCall(_))) {
                return Err("assertion failed: matches!(decoded, Some(temper_fake::forge::observations::Observation::GitCall(_)))".to_owned());
            }
            if call["installation"] != INSTALLATION {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    call["installation"], INSTALLATION
                ));
            }
            if call["mint"].as_u64().is_none_or(|mint| mint == 0) {
                return Err("assertion failed: call[\"mint\"].as_u64().is_some_and(|mint| mint > 0)".to_owned());
            }
            if call["writes"] == true {
                if call["status"] != 200 {
                    return Err("push succeeds".to_owned());
                }
            } else {
                // Before its first push git checks whether the new branch exists.
                if call["status"] != 200 && call["status"] != 404 {
                    return Err(format!("git read: {call}"));
                }
            }
        }
        if rows.iter().any(|row| row["observation"] == "git_calls_full" || row["observation"] == "installations_full") {
            return Err("Must: observation stream complete".to_owned());
        }
        let created: Vec<_> = rows.iter().filter(|row| row["observation"] == "installation_created").collect();
        if created.len() != 1 {
            return Err("Must: one installation creation across both lives, no second acceptance".to_owned());
        }
        if created[0]["installation"] != INSTALLATION {
            return Err("Must: the visited installation was created".to_owned());
        }
        if rows.iter().any(|row| row["observation"] == "installation_deleted") {
            return Err("Must: no uninstall across both lives".to_owned());
        }
        for row in rows
            .iter()
            .filter(|row| matches!(row["observation"].as_str(), Some("installation_created" | "installation_deleted")))
        {
            temper_fake::forge::observations::decode(row.to_string().as_bytes())
                .map_err(|_| "closed installation record".to_owned())?
                .ok_or_else(|| "installation observation".to_owned())?;
        }
        let opened: Vec<_> =
            rows.iter().filter(|row| row["observation"] == "opened" && row["kind"] == "pull").collect();
        let merged: Vec<_> = rows.iter().filter(|row| row["observation"] == "merged").collect();
        if opened.len() != 1 {
            return Err("one pull request across both lives".to_owned());
        }
        if merged.len() != 1 {
            return Err("one merge across both lives".to_owned());
        }
        let merge = merged[0];
        if opened[0]["repository"] != "acme/app" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                opened[0]["repository"], "acme/app"
            ));
        }
        if merge["repository"] != "acme/app" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                merge["repository"], "acme/app"
            ));
        }
        if merge["number"] != opened[0]["number"] {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                merge["number"], opened[0]["number"]
            ));
        }
        if merge["base"] != "main" {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", merge["base"], "main"));
        }
        let bot = &self.identity.as_ref().ok_or_else(|| "first bot".to_owned())?["id"];
        if merge["by"] != *bot {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", merge["by"], *bot));
        }
        if opened[0]["by"] != *bot {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", opened[0]["by"], *bot));
        }
        let pushed: Vec<_> =
            rows.iter().filter(|row| row["observation"] == "moved" && row["branch"] != "main").collect();
        if pushed.len() != 1 {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", pushed.len(), 1));
        }
        if merge["head"] != pushed[0]["to"] {
            return Err("merge at the coder's pushed head".to_owned());
        }
        let main: Vec<_> = rows.iter().filter(|row| row["observation"] == "moved" && row["branch"] == "main").collect();
        if main.len() != 1 {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", main.len(), 1));
        }
        if main[0]["to"] != merge["commit"] {
            return Err("merge commit is main's tip".to_owned());
        }
        let commit = merge["commit"].as_u64().ok_or_else(|| "merge commit".to_owned())?;
        let mut hex = String::new();
        for byte in skein_fake_forge_domain::object_bytes(commit) {
            write!(hex, "{byte:02x}").map_err(|_| "commit hex".to_owned())?;
        }
        let result = &self.landed.as_ref().ok_or_else(|| "landed result".to_owned())?["value"]["change"];
        let words = result["words"]["bytes"].as_str().ok_or_else(|| "landing words".to_owned())?;
        if words.lines().next() != Some(hex.as_str()) {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                words.lines().next(),
                Some(hex.as_str())
            ));
        }
        let forge = self.members.capture("forge", "address").ok_or_else(|| "forge address".to_owned())?;
        if words.lines().nth(1)
            != Some(format!("pull request #1 of acme/app on {forge}: http://{forge}/acme/app/pull/1").as_str())
        {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                words.lines().nth(1),
                Some(format!("pull request #1 of acme/app on {forge}: http://{forge}/acme/app/pull/1").as_str())
            ));
        }
        let panel = self.panel.as_ref().ok_or_else(|| "ctl change panel".to_owned())?;
        if panel["merge"] != hex {
            return Err("ctl names the fake forge's merge".to_owned());
        }
        if panel["pull"]["number"] != merge["number"] {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                panel["pull"]["number"], merge["number"]
            ));
        }
        if result["part"]["key"]["number"] != merge["number"] {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                result["part"]["key"]["number"], merge["number"]
            ));
        }
        if read_objects {
            let address = self
                .members
                .capture("forge", "address")
                .ok_or_else(|| "forge address".to_owned())?
                .parse()
                .map_err(|_| "socket".to_owned())?;
            let objects = github::try_objects(address, commit)?;
            let object = objects["objects"]
                .as_array()
                .ok_or_else(|| "objects".to_owned())?
                .iter()
                .find(|object| object["commit"] == commit)
                .ok_or_else(|| "merge object".to_owned())?;
            let mut file = None;
            for candidate in object["tree"].as_array().ok_or_else(|| "tree".to_owned())? {
                if decode(&candidate["path"])?.as_ref() == b"README.md" {
                    file = Some(candidate);
                    break;
                }
            }
            let file = file.ok_or("README")?;
            if decode(&file["content"])?.as_ref() != [temper_fake::forge::FILES[0].1, EDIT].concat() {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    decode(&file["content"])?.as_ref(),
                    [temper_fake::forge::FILES[0].1, EDIT].concat()
                ));
            }
        }

        Ok(())
    }
}
impl Referee<cluster::Root> for Judge {
    fn act(&mut self, now: Time, hosts: &mut [cluster::Root]) {
        if self.finding.is_some() {
            return;
        }
        if let Err(why) = self.advance(now, hosts) {
            self.finding = Some(why);
        }
    }
    fn watch(&mut self, now: Time, hosts: &[cluster::Root]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        self.members.observe(now, hosts.split_last().ok_or("client root")?.1);
        self.members.verdict()?;
        for (process, capture) in [("forge", "address"), ("llm", "address"), ("engine", "link"), ("w1", "engine")] {
            if self.members.capture(process, capture).is_some() && self.ready.insert(process) {
                self.mark(&format!("{process} ready"));
            }
        }
        if let Some(cluster::Root::Client { calls: Some(calls), .. }) = hosts.last() {
            for index in self.call_marks..calls.ends().len() {
                self.mark(&format!("ctl {:?} call {} completed", self.stage, index + 1));
            }
            self.call_marks = calls.ends().len();
        }
        if let Some(stopped) = self.members.stopped() {
            if !hosts.iter().all(Host::is_empty) {
                return Err("Must: no process or ctl client outlives the story".to_owned());
            }
            for member in stopped {
                if member.exit != Some(Exit::Code(0)) {
                    return Err(format!("{}: {}", member.name, String::from_utf8_lossy(&member.stderr)));
                }
                if member.killed || !member.survivors.is_empty() {
                    return Err("no surviving process".to_owned());
                }
            }
            if stopped.iter().map(|member| member.name.as_str()).collect::<Vec<_>>()
                != ["w1", "engine", "w1", "engine", "llm", "forge"]
            {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    stopped.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
                    ["w1", "engine", "w1", "engine", "llm", "forge"]
                ));
            }
            for (index, member) in stopped.iter().enumerate() {
                if member.ending != if index < 2 { cluster::Ending::Alone } else { cluster::Ending::Cluster } {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        member.ending,
                        if index < 2 { cluster::Ending::Alone } else { cluster::Ending::Cluster }
                    ));
                }
            }
            self.forge(false)?;
            let app = self.app.as_ref().ok_or_else(|| "made App".to_owned())?;
            let scan = Scan::new(app.pem.expose())?;
            scan.files(
                &self.scratch,
                &github::secrets(&self.members.dir("engine"))?,
                &self.scratch.join("github-app.pem"),
            )?;
            for member in stopped {
                scan.clean(&member.stdout)?;
                scan.clean(&member.stderr)?;
            }
            for output in &self.outputs {
                scan.clean(output)?;
            }
            self.mark("Musts: same landing, one PR, one merge, no new tasks, secrecy and teardown passed");
            let row = self.restored.as_ref().ok_or_else(|| "restored forges row".to_owned())?;
            let prior = self.identity.as_ref().ok_or_else(|| "first life identity".to_owned())?;
            if row["state"] != "connected" {
                return Err("Must: forge 1 connected after restart".to_owned());
            }
            if row["id"] != prior["id"] {
                return Err("Must: same bot id after restart".to_owned());
            }
            if row["login"] != prior["login"] {
                return Err("Must: same SLUG[bot] after restart".to_owned());
            }
            let installations = row["installations"]
                .as_array()
                .ok_or_else(|| "Must (T1): restored forge row exposes installations".to_owned())?;
            if installations.len() != 1 {
                return Err("Must (T1): exactly one accepted installation after restart".to_owned());
            }
            let installation = &installations[0];
            if installation["id"] != INSTALLATION {
                return Err("Must (T1): same visited installation after restart".to_owned());
            }
            if installation["account"]["login"] != "acme" {
                return Err("Must (T1): installation account filled after startup mint".to_owned());
            }
            if installation["stray"] != false {
                return Err("Must (T1): accepted installation is not stray".to_owned());
            }
            if installation["suspended"] != false {
                return Err("Must (T1): installation is not suspended".to_owned());
            }
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| format!("GitHub process story waited in {:?}", self.stage))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.members.verdict() == Ok(true)
    }
}

fn decode(value: &Value) -> Result<Box<[u8]>, String> {
    skein_lib::base64::decode(
        skein_lib::base64::Alphabet::Standard,
        value.as_str().ok_or_else(|| "base64 text".to_owned())?.as_bytes(),
        8192,
    )
    .map_err(|_| "file bytes".to_owned())
}
pub fn configure_llm(path: &Path) {
    use serde_json::json;
    let mut value: Value = serde_json::from_slice(&fs::read(path).expect("LLM fixture")).expect("LLM JSON");
    value["asked"][0]["turns"][0]["lines"] =
        json!([{"call":"change","arguments":{"words":"@coding Fix","repository":"acme/app"}}]);
    // This fixture's chat is Open, as in forge_land: text replies, then Close ends it.
    value["asked"][0]["turns"].as_array_mut().expect("chat turns").extend([
        json!({"lines":[{"text":"The change landed."}],"finish":"stop","tokens":20}),
        json!({"lines":[{"text":"The change chat is closed."}],"finish":"stop","tokens":20}),
    ]);
    value["cued"].as_array_mut().expect("scripts").push(json!({"cue":"Review this change at the named head","turns":[{"lines":[{"call":"finish","arguments":{"form":"verdict","label":"1","text":"Checked","fields":{},"items":[]}}],"finish":"tool_calls","tokens":20}]}));
    super::write_private(path, &serde_json::to_vec(&value).expect("LLM JSON"));
}
fn configure_engine(value: &mut Value) {
    // This fixture seeds its own project on stock policy 1, with no story-specific tool grants.
    let engine = value["engine"].as_object_mut().expect("engine object");
    for member in ["charters", "templates", "policies", "rules"] {
        engine.remove(member);
    }
    engine.insert("projects".into(), serde_json::json!([{"number": 1, "policy": 1}]));
    // Nine slots hold the stock charters.
    value["engine"]["limits"]["overrides"]["tasks.charters"] = 16.into();
    value["engine"]["limits"]["overrides"]["core.templates"] = 6.into();
    // Keep log polling aligned with the live GitHub story.
    let limits = &mut value["application"]["limits"]["overrides"];
    for (name, millis) in [("poll_ms", 10), ("poll_max_ms", 50), ("slow_ms", 100), ("hinted_ms", 10)] {
        limits[format!("forge.client.{name}")] = millis.into();
    }
    limits["forge.client.window_ms"] = 100.into();
    limits["forge.client.pending"] = 8.into();
    limits["forge.client.calls"] = 4.into();
    limits["forge_component.connections"] = 4.into();
    limits["forge_component.log_poll_ms"] = 15000.into();
    value["engine"]["limits"]["overrides"]["tasks.result_bytes"] = 512.into();
}

pub(crate) fn landing_reads() -> Vec<ctl::Call> {
    [
        &["task", "show", "2", "--json"][..],
        &["task", "show", "1", "--json"],
        &["task", "list", "--project", "1", "--parent", "1", "--ended", "--json"],
        &["task", "list", "--project", "1", "--parent", "2", "--ended", "--json"],
        &["forge", "list", "--json"],
        &["task", "list", "--project", "1", "--json"],
        &["task", "show", "5", "--json"],
    ]
    .into_iter()
    .map(ctl::Call::new)
    .collect()
}

pub(crate) fn landing_settings(forge: SocketAddr, owner: &str, repository: &str) -> Vec<Value> {
    use serde_json::json;
    let value = json!({"world_settings":{"landing":{"deployment":[{"connector":1,"kind":4,"pattern":{"segments":["forge",forge.to_string(),owner,repository,"branch"],"last":"exact","value":"main"},"enforce":true,"ci":true,"up_to_date":true,"gates":[{"number":1,"blocking":true,"freshness":"exact"}],"approvals":[]}]}}});
    temper_real_loop_world::settings_items::forge_items(&temper_real_loop_world::settings::forge(&value))
        .into_iter()
        .filter(|item| item["setting"]["kind"] == "landing-rule")
        .collect()
}

fn json_call(ended: &super::Ended) -> Result<Value, String> {
    serde_json::from_slice(&ended.stdout).map_err(|error| {
        format!("ctl JSON: {error}; exit {:?}; stderr: {}", ended.exit, String::from_utf8_lossy(&ended.stderr))
    })
}

fn no_live_tasks(value: &Value) -> Result<(), String> {
    if value["live"] != serde_json::json!([]) {
        return Err(format!("no live tasks in this query: {value}"));
    }
    if !value["live_after"].is_null() {
        return Err("all live tasks read".into());
    }
    Ok(())
}
fn no_live_roots(value: &Value) -> Result<(), String> {
    if value["tasks"]["rows"] != serde_json::json!([]) {
        return Err(format!("no live root tasks: {value}"));
    }
    if value["tasks"]["more"] != false {
        return Err(format!("all live root tasks read: {value}"));
    }
    Ok(())
}
