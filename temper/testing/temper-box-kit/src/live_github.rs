//! The owner-run GitHub story (testing/live-tier.md). Never invoked by the gate.
use super::{
    cluster, ctl, github, github_land,
    relay::{Guard, Relay},
};
use serde_json::{Value, json};
use skein_io::kernel::Exit;
use skein_lib::{Duration, Secret, Time, Wall};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    cell::RefCell,
    fmt::Write,
    fs,
    io::Read,
    net::SocketAddr,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    rc::Rc,
};
const WITHIN: Duration = Duration::from_secs(1200);
const POLL: Duration = Duration::from_secs(2);
/// Install links asked while setup is busy, one a poll: the reconcile after a connect is the
/// engine's own, which `forge list --until settled` does not cover (client/ctl/tools.md, 4).
const BUSY: u32 = 30;
const REPOSITORY: &str = "temper-probe";

struct Inputs {
    owner: String,
    installation: u64,
    app: u64,
    slug: String,
    pem: PathBuf,
    git: PathBuf,
    run: String,
    prefix: String,
    scratch: PathBuf,
}
impl Inputs {
    fn read(scratch: &Path) -> Self {
        let owner = temper_test_env::value(&temper_test_env::LIVE_GITHUB_OWNER)
            .expect("owner from proof-live-github.sh")
            .into_string()
            .expect("UTF-8 owner");
        assert_eq!(owner, "fabrig-ai", "probe owner only");
        let fixtures =
            PathBuf::from(temper_test_env::value(&temper_test_env::LIVE_GITHUB_FIXTURES).expect("fixtures path"));
        let app: Value = serde_json::from_slice(&fs::read(fixtures.join("app.json")).expect("public App fixture"))
            .expect("App JSON");
        let installation = app["installations"]
            .as_array()
            .expect("fixture installations")
            .iter()
            .find(|row| row["account"] == owner)
            .expect("owner installation")["id"]
            .as_u64()
            .expect("installation id");
        let git = std::env::split_paths(&temper_test_env::value(&temper_shell::environment::PATH).expect("PATH"))
            .map(|p| p.join("git"))
            .find(|p| p.is_file())
            .expect("real git on PATH")
            .canonicalize()
            .expect("git path");
        let version = std::process::Command::new(&git).arg("--version").env_clear().output().expect("git version");
        let version =
            std::str::from_utf8(&version.stdout).expect("version text").split_whitespace().nth(2).expect("git version");
        let mut parts = version.split('.').map(|part| part.parse::<u32>().expect("git version number"));
        let major = parts.next().expect("major");
        let minor = parts.next().expect("minor");
        assert!(major > 2 || (major == 2 && minor >= 41), "git >= 2.41 required");
        // UTC formatting through date; randomness comes from the OS, neither carries a credential.
        let date = std::process::Command::new("date").args(["-u", "+%Y%m%dT%H%M%S"]).output().expect("UTC date");
        assert!(date.status.success());
        let mut random = [0; 2];
        fs::File::open("/dev/urandom").expect("random source").read_exact(&mut random).expect("run randomness");
        let run =
            format!("{}{:02x}{:02x}", String::from_utf8(date.stdout).expect("UTC text").trim(), random[0], random[1]);
        let prefix = format!("temper/live-{run}-");
        Self {
            owner,
            installation,
            app: app["id"].as_u64().expect("App id"),
            slug: app["slug"].as_str().expect("App slug").into(),
            pem: fixtures.join("app.pem"),
            git,
            run,
            prefix,
            scratch: scratch.into(),
        }
    }
    fn manifest(&self, cleaned: bool) -> Result<(), String> {
        save(
            &self.scratch.join("run.json"),
            &json!({"run":self.run,"owner":self.owner,"repository":REPOSITORY,"prefix":self.prefix,"cleaned":cleaned}),
        )
    }
    fn base(&self) -> String {
        format!("/repos/{}/{}", self.owner, REPOSITORY)
    }
}

pub fn run() {
    let clock = Clock::new();
    let started = clock.now().now;
    let scratch =
        PathBuf::from(temper_test_env::value(&temper_test_env::TMPDIR).expect("run through proof-live-github.sh"));
    let inputs = Inputs::read(&scratch);
    inputs.manifest(false).unwrap_or_else(|why| panic!("{why}"));
    eprintln!("live github run={}", inputs.run);
    let mut relay = Relay::start(Guard {
        owner: inputs.owner.clone(),
        repository: REPOSITORY.into(),
        installation: inputs.installation,
        prefix: inputs.prefix.clone(),
    });
    let result = story(&inputs, &relay);
    relay.quiesce();
    let cleanup = teardown(&inputs, &relay);
    eprintln!(
        "live github run={} requests={} wall={}s state={} (removed on exit unless TEMPER_TEST_LIVE_GITHUB_KEEP=1)",
        inputs.run,
        relay.count(),
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000,
        scratch.display()
    );
    if let Err(failed) = result {
        panic!("{failed}");
    }
    if let Err(failed) = cleanup {
        panic!("{failed}");
    }
    assert!(relay.refused().is_empty(), "relay refusals: {:?}", relay.refused());
}

#[expect(clippy::must_use_candidate, reason = "the fixture callers assert on the returned observations")]
pub fn document(path: &Path) -> Value {
    read_document(path).unwrap_or_else(|why| panic!("{why}"))
}
fn read_document(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|_| "fixture document")?).map_err(|_| "fixture JSON".into())
}
fn save(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "document JSON")?;
    fs::write(path, bytes).map_err(|_| "fixture file")?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|_| "private fixture".into())
}
fn render_owner(value: &mut Value, owner: &str) {
    match value {
        Value::String(s) if s == "acme" => *s = owner.into(),
        Value::Array(rows) => {
            for row in rows {
                render_owner(row, owner);
            }
        }
        Value::Object(fields) => {
            for (_, field) in fields {
                render_owner(field, owner);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
/// Shape the live engine without reading inputs or constructing a relay.
pub fn configure_engine(value: &mut Value, owner: &str, api: SocketAddr, git: SocketAddr, workers: SocketAddr) {
    render_owner(value, owner);
    value["engine"]["hosts"]["listener"][0]["socket"] = workers.to_string().into();
    value["application"]["machine"]["github"] = json!({"api":{"socket":api.to_string()},"transport":"plaintext","remote":format!("http://{git}"),"web":format!("http://{git}")});
    value["application"]["limits"]["overrides"]["forge_component.log_poll_ms"] = 15000.into();
}
fn configure_llm(path: &Path, inputs: &Inputs) -> Result<(), String> {
    github_land::configure_llm(path);
    let mut value = read_document(path)?;
    value["asked"][0]["turns"][0]["lines"][0]["arguments"]["repository"] =
        format!("{}/{}", inputs.owner, REPOSITORY).into();
    value["cued"][0]["turns"] = json!([
        {"lines":[{"call":"write","arguments":{"path":format!("probe/{}.md",inputs.run),"content":format!("temper live run {}\n",inputs.run)}}],"finish":"tool_calls","tokens":20},
        {"lines":[{"call":"finish","arguments":{"form":"change","fields":{"title":format!("Live run {}",inputs.run),"body":format!("Adds probe/{}.md.",inputs.run)}}}],"finish":"tool_calls","tokens":20}
    ]);
    save(path, &value)
}
#[derive(Default)]
struct Seen {
    landed: Option<Value>,
    identity: Option<Value>,
    tasks: Vec<u64>,
    outputs: Vec<Vec<u8>>,
    coder: Option<Value>,
}
#[derive(Debug, PartialEq)]
enum Stage {
    Llm,
    Engine,
    Worker,
    Install,
    Link,
    Accepted,
    Adopt,
    Chat,
    Landed,
    Reply,
    Pages,
    Close,
    Before,
    StopWorker,
    StopEngine,
    RestartEngine,
    RestartWorker,
    Reconnected,
    After,
    Ended,
    Stopping,
}
struct Judge<'a> {
    members: cluster::Members,
    finding: Option<String>,
    inputs: &'a Inputs,
    relay: &'a Relay,
    jwt: &'a Secret,
    clients: SocketAddr,
    workers: SocketAddr,
    stage: Stage,
    deadline: Time,
    poll: Time,
    now: Time,
    seen: Rc<RefCell<Seen>>,
    replied: bool,
    digest: Option<String>,
    busy: u32,
    started: Time,
    landing: Option<Landing>,
}
fn story(inputs: &Inputs, relay: &Relay) -> Result<(), String> {
    let clock = Clock::new();
    // This read precedes product startup, so mint independently rather than wait for its token.
    let (authorization, jwt) = mint_verifier(inputs, relay)?;
    // The first read: it shows the guard the probe's id, the one `/repositories/<id>` it allows.
    let probe = ask(relay, &authorization, &inputs.base())?;
    if probe["full_name"] != format!("{}/{}", inputs.owner, REPOSITORY) {
        return Err("the probe repository".to_owned());
    }
    let [clients, workers] = super::free_addresses(&[relay.api, relay.git]);
    let mut description = cluster::Cluster::github_relay(&inputs.scratch, relay.git);
    description.bind("story.clients", &clients.to_string());
    description.prepare("llm");
    configure_llm(&description.dir("llm").join("llm.json"), inputs)?;
    let (mut members, hosts) = description.members_within(WITHIN);
    members.up(&["llm"]);
    let seen = Rc::new(RefCell::new(Seen::default()));
    let mut world = real::World::new(Judge {
        members,
        finding: None,
        inputs,
        relay,
        jwt: &jwt,
        clients,
        workers,
        stage: Stage::Llm,
        deadline: clock.now().now.saturating_add(WITHIN),
        poll: clock.now().now,
        now: clock.now().now,
        seen: seen.clone(),
        replied: false,
        digest: None,
        busy: 0,
        started: clock.now().now,
        landing: None,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(cluster::Root::client);
    let outcome = world.try_run(&clock, WITHIN)?;
    if !outcome.procs.iter().all(Host::is_empty) {
        return Err("no process outlives the story".to_owned());
    }
    let seen = seen.borrow();
    github_musts(inputs, relay, &authorization, &seen)?;
    scan(inputs, relay, &jwt, &seen.outputs)?;

    Ok(())
}
impl Judge<'_> {
    fn advance(&mut self, now: Time, hosts: &mut [cluster::Root]) -> Result<(), String> {
        self.now = now;
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = Stage::Stopping;
            return Ok(());
        }
        let host = hosts.last_mut().ok_or_else(|| "client".to_owned())?;
        if self.stage == Stage::Llm && self.members.is_up() {
            self.members.prepare("engine");
            let path = self.members.dir("engine").join("config/temper.json");
            let mut value = read_document(&path)?;
            configure_engine(&mut value, &self.inputs.owner, self.relay.api, self.relay.git, self.workers);
            save(&path, &value)?;
            save(
                &self.inputs.scratch.join("landing-settings.json"),
                &json!({"items":github_land::landing_settings(self.relay.git, &self.inputs.owner, REPOSITORY)}),
            )?;
            self.members.up(&["engine"]);
            self.stage = Stage::Engine;
        } else if self.stage == Stage::Engine && self.members.is_up() {
            self.members.prepare("w1");
            let path = self.members.dir("w1").join("config/temper.json");
            let mut value = read_document(&path)?;
            value["application"]["workers"][0]["git"] = json!(self.inputs.git);
            save(&path, &value)?;
            self.members.up(&["w1"]);
            self.stage = Stage::Worker;
        } else if self.stage == Stage::Worker && self.members.is_up() {
            if self.members.linked("w1") != 1 {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    self.members.linked("w1"),
                    1
                ));
            }
            self.begin(host)?;
        } else if self.stage == Stage::StopWorker && self.members.ended("w1") {
            self.members.stop_one("engine");
            self.stage = Stage::StopEngine;
        } else if self.stage == Stage::StopEngine && self.members.ended("engine") {
            self.members.restart("engine");
            self.stage = Stage::RestartEngine;
        } else if self.stage == Stage::RestartEngine && self.members.capture("engine", "link").is_some() {
            self.members.restart("w1");
            self.stage = Stage::RestartWorker;
        } else if self.stage == Stage::RestartWorker && self.members.is_up() {
            if self.members.linked("w1") != 2 {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    self.members.linked("w1"),
                    2
                ));
            }
            self.queue(
                host,
                vec![
                    ctl::Call::new(&["forge", "list", "--until", "connected:1"]),
                    ctl::Call::new(&["forge", "list", "--json"]),
                ],
            )?;
            self.stage = Stage::Reconnected;
        } else if host.is_empty()
            && now >= self.poll
            && !matches!(
                self.stage,
                Stage::Llm
                    | Stage::Engine
                    | Stage::Worker
                    | Stage::StopWorker
                    | Stage::StopEngine
                    | Stage::RestartEngine
                    | Stage::RestartWorker
                    | Stage::Stopping
            )
        {
            let cluster::Root::Client { calls, .. } = host else { return Err("client".to_owned()) };
            let calls = calls.take().ok_or_else(|| "finished batch".to_owned())?;
            self.called(host, &calls)?;
            self.poll = now.saturating_add(POLL);
        }

        Ok(())
    }

    fn queue(&mut self, host: &mut cluster::Root, args: Vec<ctl::Call>) -> Result<(), String> {
        let link =
            format!("http://{}", self.members.capture("engine", "link").ok_or_else(|| "engine link".to_owned())?);
        let mut batch = ctl::Calls::linked(&link, &self.inputs.scratch.join("client"));
        for call in args {
            batch.push(ctl::Step::Run(call));
        }
        let cluster::Root::Client { calls, .. } = host else { return Err("client root".to_owned()) };
        *calls = Some(Box::new(batch));

        Ok(())
    }
    fn install(&mut self, host: &mut cluster::Root) -> Result<(), String> {
        // A key's first answer is replayed (client/ctl/README.md, 5): each ask again draws a fresh key.
        let key = format!("live-install-{}", self.busy);
        self.one(host, &["forge", "install", "1", "--key", &key], Stage::Link)?;

        Ok(())
    }
    fn elapsed(&self) -> u64 {
        self.now.saturating_since(self.started).as_nanos() / 1_000_000_000
    }
    fn one(&mut self, host: &mut cluster::Root, args: &[&str], stage: Stage) -> Result<(), String> {
        self.queue(host, vec![ctl::Call::new(args)])?;
        self.stage = stage;

        Ok(())
    }
    fn read_landing(&mut self, host: &mut cluster::Root, stage: Stage) -> Result<(), String> {
        let mut reads = github_land::landing_reads();
        reads.push(ctl::Call::new(&["task", "show", "3", "--json"]));
        self.queue(host, reads)?;
        self.stage = stage;

        Ok(())
    }
    fn begin(&mut self, host: &mut cluster::Root) -> Result<(), String> {
        self.queue(
            host,
            vec![
                ctl::Call::sign_in(),
                ctl::Call::new(&[
                    "settings",
                    "apply",
                    self.inputs
                        .scratch
                        .join("landing-settings.json")
                        .to_str()
                        .ok_or_else(|| "landing settings FILE".to_owned())?,
                    "--against",
                    "1",
                    "--key",
                    "live-landing-settings",
                ]),
                ctl::Call::new(&[
                    "account",
                    "add-key",
                    "1",
                    self.members.dir("llm").join("llm.key").to_str().ok_or_else(|| "LLM key path".to_owned())?,
                    "--deployment",
                    "--key",
                    "live-llm",
                ]),
                ctl::Call::new(&["account", "list", "--until", "usable"]),
                ctl::Call::new(&[
                    "forge",
                    "connect",
                    "1",
                    "github-app",
                    &self.inputs.app.to_string(),
                    self.inputs.pem.to_str().ok_or_else(|| "App key path".to_owned())?,
                    "--key",
                    "live-app",
                ]),
                ctl::Call::new(&["forge", "list", "--until", "connected:1"]),
                ctl::Call::new(&["forge", "list", "--until", "settled"]),
            ],
        )?;
        self.stage = Stage::Install;

        Ok(())
    }
    fn row(packet: &Value) -> Result<Value, String> {
        Ok(packet["forges"]
            .as_array()
            .ok_or_else(|| "forge rows".to_owned())?
            .iter()
            .find(|row| row["number"] == 1)
            .ok_or_else(|| "forge 1".to_owned())?
            .clone())
    }
    fn accepted(&self, row: &Value) -> bool {
        row["installations"].as_array().is_some_and(|rows| {
            rows.iter().any(|row| {
                row["id"] == self.inputs.installation
                    && row["account"]["login"] == self.inputs.owner
                    && row["stray"] == false
                    && row["suspended"] == false
            })
        })
    }
    #[expect(clippy::too_many_lines, reason = "the ordered client script shares M1's cluster and landing reads")]
    fn called(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        for (index, ended) in calls.ends().iter().enumerate() {
            self.seen.borrow_mut().outputs.extend([ended.stdout.clone(), ended.stderr.clone()]);
            if self.stage == Stage::Link
                && ended.exit == Some(Exit::Code(2))
                && ended.stderr.trim_ascii() == b"busy"
                && self.busy < BUSY
            {
                self.busy += 1;
                eprintln!(
                    "live github: install refused busy, asked again ({}/{BUSY}) at {}s",
                    self.busy,
                    self.elapsed()
                );
                self.install(host)?;
                return Ok(());
            }
            let expected = if index == 6 && matches!(self.stage, Stage::Before | Stage::After) { 2 } else { 0 };
            if ended.exit != Some(Exit::Code(expected)) {
                // The failed call's words are shown once the secrecy scan has passed over them.
                scan(self.inputs, self.relay, self.jwt, &[ended.stdout.clone(), ended.stderr.clone()])?;
                return Err(format!(
                    "live ctl call {index} failed in {:?}: {:?}, stdout: {}, stderr: {}",
                    self.stage,
                    ended.exit,
                    String::from_utf8_lossy(&ended.stdout),
                    String::from_utf8_lossy(&ended.stderr)
                ));
            }
        }
        match self.stage {
            Stage::Install => self.install(host)?,
            Stage::Link => {
                let text = std::str::from_utf8(&calls.ends().last().ok_or_else(|| "install".to_owned())?.stdout)
                    .map_err(|_| "link text".to_owned())?;
                let link = text
                    .lines()
                    .find(|s| s.starts_with("https://"))
                    .ok_or_else(|| "GitHub installation link".to_owned())?;
                let state = link
                    .split('?')
                    .nth(1)
                    .ok_or_else(|| "link query".to_owned())?
                    .split('&')
                    .find_map(|part| part.strip_prefix("state="))
                    .ok_or_else(|| "install state".to_owned())?;
                if !state.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_%.".contains(&b)) {
                    return Err("URL state".to_owned());
                }
                let url = format!(
                    "http://{}/forges/1/github/setup?installation_id={}&setup_action=install&state={state}",
                    self.clients, self.inputs.installation
                );
                let response = github::request_url(self.clients, "GET", &url, "", b"")?;
                if response.status != 200 {
                    return Err("setup accepted".to_owned());
                }
                self.seen.borrow_mut().outputs.push(response.body);
                self.one(host, &["forge", "list", "--json"], Stage::Accepted)?;
            }
            Stage::Accepted => {
                let row = Self::row(&json_call(calls.ends().first().ok_or("completed ctl call")?)?)?;
                if self.accepted(&row) {
                    self.one(
                        host,
                        &[
                            "repository",
                            "adopt",
                            "--project",
                            "1",
                            "--forge",
                            "1",
                            "--owner",
                            &self.inputs.owner,
                            "--name",
                            REPOSITORY,
                            "--landing",
                            "main",
                            "--prefix",
                            &self.inputs.prefix,
                            "--no-ci",
                            "--check",
                            "1",
                            "--key",
                            "live-adopt",
                            "--json",
                        ],
                        Stage::Adopt,
                    )?;
                } else {
                    self.one(host, &["forge", "list", "--json"], Stage::Accepted)?;
                }
            }
            Stage::Adopt => {
                if !json_call(calls.ends().first().ok_or("completed ctl call")?)?["adopted"].is_object() {
                    return Err("repository adopted".to_owned());
                }
                self.one(
                    host,
                    &[
                        "chat",
                        "start",
                        "--project",
                        "1",
                        "--key",
                        "live-chat",
                        "@forge_chat Make the live probe",
                        "--json",
                    ],
                    Stage::Chat,
                )?;
            }
            Stage::Chat => {
                self.queue(
                    host,
                    vec![
                        ctl::Call::new(&["task", "show", "1", "--until", "replied"]),
                        ctl::Call::new(&["task", "show", "2", "--until", "ended", "--json"]),
                    ],
                )?;
                self.stage = Stage::Landed;
            }
            Stage::Landed => {
                let packet = json_call(calls.ends().last().ok_or_else(|| "landed task".to_owned())?)?;
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
                self.seen.borrow_mut().landed = Some(packet["task"]["result"].clone());
                self.one(host, &["task", "show", "1", "--json"], Stage::Reply)?;
            }
            Stage::Reply => {
                self.digest = Some(
                    json_call(calls.ends().last().ok_or_else(|| "reply snapshot".to_owned())?)?["digest"]
                        .as_str()
                        .ok_or_else(|| "task digest".to_owned())?
                        .into(),
                );
                self.one(host, &["chat", "show", "1", "--json"], Stage::Pages)?;
            }
            Stage::Pages => {
                let page = json_call(calls.ends().first().ok_or("completed ctl call")?)?;
                self.replied |= page["activity"]["rows"].to_string().contains("The change landed.");
                if let Some(before) = page["activity"]["before"]["value"].as_str() {
                    self.one(host, &["chat", "show", "1", "--before", before, "--json"], Stage::Pages)?;
                } else if !self.replied {
                    self.one(
                        host,
                        &[
                            "task",
                            "show",
                            "1",
                            "--until",
                            "replied",
                            "--since",
                            &self.digest.clone().ok_or_else(|| "snapshot".to_owned())?,
                            "--json",
                        ],
                        Stage::Reply,
                    )?;
                } else {
                    self.queue(
                        host,
                        vec![
                            ctl::Call::new(&["chat", "close", "1", "--key", "live-close", "--json"]),
                            ctl::Call::new(&["task", "show", "1", "--until", "ended", "--json"]),
                        ],
                    )?;
                    self.stage = Stage::Close;
                }
            }
            Stage::Close => self.read_landing(host, Stage::Before)?,
            Stage::Reconnected => {
                let row = Self::row(&json_call(calls.ends().last().ok_or_else(|| "reconnected forges".to_owned())?)?)?;
                if self.accepted(&row) {
                    self.read_landing(host, Stage::After)?;
                } else {
                    self.one(host, &["forge", "list", "--json"], Stage::Reconnected)?;
                }
            }
            Stage::Before | Stage::After => self.check_landing(host, calls)?,
            Stage::Ended => self.ended_pages(host, calls)?,
            Stage::Llm
            | Stage::Engine
            | Stage::Worker
            | Stage::StopWorker
            | Stage::StopEngine
            | Stage::RestartEngine
            | Stage::RestartWorker
            | Stage::Stopping => return Err("calls only in script stages".to_owned()),
        }

        Ok(())
    }
    fn check_landing(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let task = json_call(calls.ends().first().ok_or("completed ctl call")?)?;
        if task["chip"]["phase"] != "ended" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                task["chip"]["phase"], "ended"
            ));
        }
        let landed = self.seen.borrow().landed.clone().ok_or_else(|| "first landing".to_owned())?;
        if task["result"] != landed {
            return Err("same pull and merge".to_owned());
        }
        if json_call(calls.ends().get(1).ok_or("completed ctl call")?)?["chip"]["phase"] != "ended" {
            return Err("chat closed".to_owned());
        }
        no_live_roots(&json_call(calls.ends().get(5).ok_or("completed ctl call")?)?)?;
        if calls.ends().get(6).ok_or("completed ctl call")?.stderr.trim_ascii_end() != br#"{"error":"unknown"}"# {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                calls.ends().get(6).ok_or("completed ctl call")?.stderr.trim_ascii_end(),
                br#"{"error":"unknown"}"#
            ));
        }
        let row = Self::row(&json_call(calls.ends().get(4).ok_or("completed ctl call")?)?)?;
        if row["state"] != "connected" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                row["state"], "connected"
            ));
        }
        if row["login"] != format!("{}[bot]", self.inputs.slug) {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                row["login"],
                format!("{}[bot]", self.inputs.slug)
            ));
        }
        if row["id"].as_u64().is_none() {
            return Err("assertion failed: row[\"id\"].as_u64().is_some()".to_owned());
        }
        if !self.accepted(&row) {
            return Err("same accepted installation".to_owned());
        }
        let coder = json_call(calls.ends().get(7).ok_or("completed ctl call")?)?;
        if coder["chip"]["phase"] != "ended" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                coder["chip"]["phase"], "ended"
            ));
        }
        let mut landing = Landing {
            restarted: self.stage == Stage::After,
            tasks: vec![1],
            row,
            coder: coder["result"].clone(),
            pages: Vec::new(),
        };
        for index in [2, 3] {
            let packet = json_call(calls.ends().get(index).ok_or("completed ctl call")?)?;
            no_live_tasks(&packet)?;
            landing.page(&packet)?;
        }
        self.landing = Some(landing);
        self.next_page(host)?;

        Ok(())
    }
    fn ended_pages(&mut self, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        let landing = self.landing.as_mut().ok_or_else(|| "landing read".to_owned())?;
        for ended in calls.ends() {
            landing.page(&json_call(ended)?)?;
        }
        self.next_page(host)?;

        Ok(())
    }
    fn next_page(&mut self, host: &mut cluster::Root) -> Result<(), String> {
        let pages = std::mem::take(&mut self.landing.as_mut().ok_or_else(|| "landing read".to_owned())?.pages);
        if !pages.is_empty() {
            let reads = pages
                .iter()
                .map(|next| ctl::Call::new(&next.split_ascii_whitespace().chain(["--json"]).collect::<Vec<_>>()))
                .collect();
            self.queue(host, reads)?;
            self.stage = Stage::Ended;
            return Ok(());
        }
        let Landing { restarted, mut tasks, row, coder, .. } =
            self.landing.take().ok_or_else(|| "landing read".to_owned())?;
        tasks.sort_unstable();
        if tasks != [1, 2, 3, 4] {
            return Err("complete task set".to_owned());
        }
        let mut seen = self.seen.borrow_mut();
        if restarted {
            if row["id"] != seen.identity.as_ref().ok_or_else(|| "prior bot".to_owned())?["id"] {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    row["id"],
                    seen.identity.as_ref().ok_or_else(|| "prior bot".to_owned())?["id"]
                ));
            }
            if tasks != seen.tasks {
                return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", tasks, seen.tasks));
            }
            if Some(&coder) != seen.coder.as_ref() {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    Some(&coder),
                    seen.coder.as_ref()
                ));
            }
            self.members.stop();
            self.stage = Stage::Stopping;
        } else {
            seen.identity = Some(row);
            seen.tasks = tasks;
            seen.coder = Some(coder);
            self.members.stop_one("w1");
            self.stage = Stage::StopWorker;
        }

        Ok(())
    }
}
/// A landing read's checks, held while its ended delegates are read page by page.
struct Landing {
    restarted: bool,
    tasks: Vec<u64>,
    row: Value,
    coder: Value,
    pages: Vec<String>,
}
impl Landing {
    /// One page of a parent's ended delegates. A page may hold fewer rows than match and still name more:
    /// its `read_on` is the next read, followed until a page names none.
    fn page(&mut self, packet: &Value) -> Result<(), String> {
        let rows = &packet["ended"]["rows"];
        if rows["cut"] != false {
            return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", rows["cut"], false));
        }
        for row in rows["rows"].as_array().ok_or("task rows")? {
            self.tasks.push(row["chip"]["task"].as_u64().ok_or("task id")?);
        }
        if !rows["after"].is_null() {
            let next = packet["read_on"].as_str().ok_or_else(|| "a page with more names its next read".to_owned())?;
            if !next.contains("--after") {
                return Err(format!("the next read continues after this page: {next}"));
            }
            self.pages.push(next.to_owned());
        }

        Ok(())
    }
}
impl Referee<cluster::Root> for Judge<'_> {
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
        if !self.relay.refused().is_empty() {
            return Err(format!("relay refusals: {:?}", self.relay.refused()));
        }
        if let Some(stopped) = self.members.stopped() {
            let scan =
                github::Scan::new(&fs::read(&self.inputs.pem).map_err(|_| "PEM read only for scan".to_owned())?)?;
            let secrets = self.relay.tokens();
            let mut diagnostic = format!("live GitHub cluster failure={:?}\n", self.members.failure());
            for process in stopped {
                clean(&scan, self.jwt, &secrets, &process.stderr)?;
                writeln!(
                    diagnostic,
                    "{} life={} stderr:\n{}",
                    process.name,
                    process.life,
                    String::from_utf8_lossy(&process.stderr)
                )
                .map_err(|_| "diagnostic text".to_owned())?;
            }
            clean(&scan, self.jwt, &secrets, diagnostic.as_bytes())?;
            eprint!("{diagnostic}");
            if !hosts.iter().all(Host::is_empty) {
                return Err("assertion failed: hosts.iter().all(Host::is_empty)".to_owned());
            }
            if stopped.iter().map(|p| p.name.as_str()).collect::<Vec<_>>() != ["w1", "engine", "w1", "engine", "llm"] {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    stopped.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
                    ["w1", "engine", "w1", "engine", "llm"]
                ));
            }
            for process in stopped {
                self.seen.borrow_mut().outputs.extend([process.stdout.clone(), process.stderr.clone()]);
                if process.exit != Some(Exit::Code(0)) {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        process.exit,
                        Some(Exit::Code(0))
                    ));
                }
                if process.killed || !process.survivors.is_empty() {
                    return Err("assertion failed: !process.killed && process.survivors.is_empty()".to_owned());
                }
            }
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.members.next_deadline().unwrap_or(self.deadline).min(self.deadline).min(if self.poll > self.now {
            self.poll
        } else {
            self.deadline
        }))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| format!("live GitHub deadline in {:?}", self.stage))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.members.verdict() == Ok(true)
    }
}

fn mint_verifier(inputs: &Inputs, relay: &Relay) -> Result<(Vec<u8>, Secret), String> {
    use skein_forge::github::jwt;
    use skein_lib::base64::{self, Alphabet};
    let pem = fs::read(&inputs.pem).map_err(|_| "App PEM".to_owned())?;
    let compact: String = std::str::from_utf8(&pem)
        .map_err(|_| "PEM text".to_owned())?
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    let key = base64::decode(Alphabet::Standard, compact.as_bytes(), 4096).map_err(|_| "App DER".to_owned())?;
    let wall = Clock::new().now().wall.as_nanos();
    let issued = wall.saturating_sub(Duration::from_secs(60).as_nanos());
    let expires = issued.checked_add(jwt::LIFE.as_nanos()).ok_or_else(|| "JWT expiry".to_owned())?;
    let input =
        jwt::signing_input(inputs.app, Wall::from_nanos(issued), Wall::from_nanos(expires)).ok_or("App JWT claims")?;
    let signature = skein_tls::sign_rs256(&key, &input).ok_or("App key signs RS256")?;
    let jwt = jwt::token(&input, &signature).ok_or("App JWT")?;
    let auth = [b"Bearer ".as_slice(), jwt.expose()].concat();
    let path = format!("/app/installations/{}/access_tokens", inputs.installation);
    let (status, body) = relay
        .ask("POST", &path, &auth, br#"{"repositories":["temper-probe"]}"#)
        .map_err(|_| "verifier mint through relay".to_owned())?;
    if status != 201 {
        return Err("verifier installation token minted".to_owned());
    }
    let value: Value = serde_json::from_slice(&body).map_err(|_| "installation token JSON".to_owned())?;
    let token = value["token"].as_str().ok_or_else(|| "installation token".to_owned())?.as_bytes();
    if token.is_empty() || token.iter().any(u8::is_ascii_whitespace) {
        return Err("one installation token".to_owned());
    }
    // Relay records this mint before returning, for the scan and teardown even if the story panics.
    Ok(([b"Bearer ".as_slice(), token].concat(), jwt))
}
fn ask(relay: &Relay, authorization: &[u8], path: &str) -> Result<Value, String> {
    let (status, body) =
        relay.ask("GET", path, authorization, b"").map_err(|_| "verifier read through relay".to_owned())?;
    if status != 200 {
        return Err(format!("verifier read {path}"));
    }
    serde_json::from_slice(&body).map_err(|_| "GitHub JSON".to_owned())
}
fn github_musts(inputs: &Inputs, relay: &Relay, authorization: &[u8], seen: &Seen) -> Result<(), String> {
    let base = inputs.base();
    let result = &seen.landed.as_ref().ok_or_else(|| "landed result".to_owned())?["value"]["change"];
    let number = result["part"]["key"]["number"].as_u64().ok_or_else(|| "pull number".to_owned())?;
    let pull = ask(relay, authorization, &format!("{base}/pulls/{number}"))?;
    if pull["merged"] != true {
        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", pull["merged"], true));
    }
    if pull["merge_commit_sha"] != result["words"]["bytes"] {
        return Err(format!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            pull["merge_commit_sha"], result["words"]["bytes"]
        ));
    }
    let head = pull["head"]["sha"].as_str().ok_or_else(|| "pull head".to_owned())?;
    // GitHub must agree with the exact head temper submitted for its change's merge.
    if relay.merge_heads() != [head] {
        return Err("pull head is the change head submitted by temper".to_owned());
    }
    let file = ask(relay, authorization, &format!("{base}/contents/probe/{}.md?ref=main", inputs.run))?;
    let compact: Vec<u8> = file["content"]
        .as_str()
        .ok_or_else(|| "file base64".to_owned())?
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    let bytes = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, &compact, 8192)
        .map_err(|_| "file bytes".to_owned())?;
    if bytes.as_ref() != format!("temper live run {}\n", inputs.run).as_bytes() {
        return Err(format!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            bytes.as_ref(),
            format!("temper live run {}\n", inputs.run).as_bytes()
        ));
    }
    let checks = ask(relay, authorization, &format!("{base}/commits/{head}/check-runs?per_page=100"))?;
    let ci = checks["check_runs"]
        .as_array()
        .ok_or_else(|| "check runs".to_owned())?
        .iter()
        .find(|run| run["name"] == "ci" && run["app"]["slug"] == "github-actions")
        .ok_or_else(|| "real Actions ci".to_owned())?;
    if ci["head_sha"] != head {
        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", ci["head_sha"], head));
    }
    if ci["status"] != "completed" {
        return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", ci["status"], "completed"));
    }
    if ci["conclusion"] != "success" {
        return Err(format!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            ci["conclusion"], "success"
        ));
    }
    if ci["completed_at"].as_str().ok_or_else(|| "completion UTC".to_owned())?
        > pull["merged_at"].as_str().ok_or_else(|| "merge UTC".to_owned())?
    {
        return Err("CI completed before merge".to_owned());
    }
    let journal = relay.journal();
    for (method, path) in [("POST", format!("{base}/pulls")), ("PUT", format!("{base}/pulls/{number}/merge"))] {
        if journal.iter().filter(|row| row.0 == "api" && row.1 == method && row.2 == path).count() != 1 {
            return Err(format!("one {method} {path}"));
        }
    }
    if !relay.refused().is_empty() {
        return Err(format!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            (relay.refused()).len(),
            0
        ));
    }

    Ok(())
}
fn clean(scan: &github::Scan, jwt: &Secret, secrets: &[Vec<u8>], bytes: &[u8]) -> Result<(), String> {
    scan.clean(bytes)?;
    if skein_lib::bytes::find(bytes, jwt.expose()).is_some() {
        return Err("no verifier JWT outside memory".to_owned());
    }
    for secret in secrets {
        if skein_lib::bytes::find(bytes, secret).is_some() {
            return Err("no minted token outside jig secrets".to_owned());
        }
    }

    Ok(())
}
fn scan(inputs: &Inputs, relay: &Relay, jwt: &Secret, outputs: &[Vec<u8>]) -> Result<(), String> {
    let scan = github::Scan::new(&fs::read(&inputs.pem).map_err(|_| "PEM read only for scan".to_owned())?)?;
    let secrets = relay.tokens();
    let clean = |bytes: &[u8]| clean(&scan, jwt, &secrets, bytes);
    for bytes in outputs {
        clean(bytes)?;
    }
    let mut pending = ["engine", "w1", "llm"].map(|name| inputs.scratch.join(name)).to_vec();
    let excluded = github::secrets(&inputs.scratch.join("engine"))?;
    while let Some(path) = pending.pop() {
        if path == excluded {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|_| "scan metadata".to_owned())?;
        if metadata.is_dir() {
            for entry in fs::read_dir(&path).map_err(|_| "scan directory")? {
                pending.push(entry.map_err(|_| "entry")?.path());
            }
        } else if metadata.is_file() {
            clean(&fs::read(&path).map_err(|_| "scan file".to_owned())?)
                .map_err(|why| format!("{}: {why}", path.display()))?;
        }
    }

    Ok(())
}
fn cleanup_refs(inputs: &Inputs, relay: &Relay, auth: &[u8]) -> Result<(), String> {
    let base = inputs.base();
    let mut findings = Vec::new();
    let list = relay.teardown("GET", &format!("{base}/git/matching-refs/heads/{}", inputs.prefix), auth, b"");
    match list {
        Ok((200, body)) => {
            if let Ok(Value::Array(refs)) = serde_json::from_slice::<Value>(&body) {
                for row in refs {
                    let Some(name) = row["ref"].as_str().and_then(|r| r.strip_prefix("refs/heads/")) else {
                        findings.push("branch ref".to_owned());
                        continue;
                    };
                    if !name.starts_with(&inputs.prefix) {
                        findings.push("teardown confined to run".to_owned());
                        continue;
                    }
                    if !relay
                        .teardown("DELETE", &format!("{base}/git/refs/heads/{name}"), auth, b"")
                        .is_ok_and(|(status, _)| status == 204 || status == 404)
                    {
                        findings.push(format!("live github branch left={name}"));
                    }
                }
            } else {
                findings.push("teardown branch list JSON".to_owned());
            }
        }
        Ok((status, _)) => findings.push(format!("teardown branch list status={status}")),
        Err(_) => findings.push("teardown branch list request".to_owned()),
    }
    // Query by the run's exact head prefix, then verify it before closing any pull.
    match relay.teardown(
        "GET",
        &format!("{base}/pulls?state=open&head={}:{}1/c2&per_page=100", inputs.owner, inputs.prefix),
        auth,
        b"",
    ) {
        Ok((200, body)) => {
            if let Ok(Value::Array(pulls)) = serde_json::from_slice::<Value>(&body) {
                for pull in pulls {
                    let Some(head) = pull["head"]["ref"].as_str() else {
                        findings.push("pull branch".to_owned());
                        continue;
                    };
                    if !head.starts_with(&inputs.prefix)
                        || pull["head"]["repo"]["full_name"] != format!("{}/{}", inputs.owner, REPOSITORY)
                    {
                        findings.push("run's pull only".to_owned());
                        continue;
                    }
                    let Some(number) = pull["number"].as_u64().filter(|number| *number > 0) else {
                        findings.push("pull number".to_owned());
                        continue;
                    };
                    if !relay
                        .teardown("PATCH", &format!("{base}/pulls/{number}"), auth, br#"{"state":"closed"}"#)
                        .is_ok_and(|(status, _)| status == 200)
                    {
                        findings.push(format!("live github pull left={number}"));
                    }
                }
            } else {
                findings.push("teardown pull list JSON".to_owned());
            }
        }
        Ok((status, _)) => findings.push(format!("teardown pull list status={status}")),
        Err(_) => findings.push("teardown pull list request".to_owned()),
    }
    if findings.is_empty() { Ok(()) } else { Err(findings.join("\n")) }
}
fn teardown(inputs: &Inputs, relay: &Relay) -> Result<(), String> {
    let tokens = relay.tokens();
    let mut findings = Vec::new();
    // The verifier is minted first, before any product process can write or mint a token.
    if let Some(token) = tokens.first() {
        let auth = [b"Bearer ".as_slice(), token].concat();
        if let Err(why) = cleanup_refs(inputs, relay, &auth) {
            findings.push(why);
        }
    }
    for (index, token) in tokens.iter().enumerate() {
        let auth = [b"Bearer ".as_slice(), token].concat();
        if !relay
            .teardown("DELETE", "/installation/token", &auth, b"")
            .is_ok_and(|(status, _)| status == 204 || status == 401)
        {
            findings.push(format!("installation token {index} revocation failed"));
        }
    }
    if !findings.is_empty() {
        return Err(format!("GitHub teardown incomplete; script backstop will retry\n{}", findings.join("\n")));
    }
    inputs.manifest(true)
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
