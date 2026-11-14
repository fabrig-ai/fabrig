//! A person connects their App, pushes once and restarts (testing/process-tier.md).
use super::{
    Scratch, WITHIN, cluster, ctl,
    story_socket::{Scope, Socket},
};
use serde_json::Value;
use skein_io::kernel::Exit;
use skein_lib::{Secret, Time, base64};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    net::SocketAddr,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

const INSTALLATION: u64 = 1;
const CODER: u64 = 3; // Fresh deployment: chat 1, change procedure 2, its coder 3.
const EDIT: &[u8] = b"Changed by temper.\n";

/// Both GitHub stories park chat promptly; a stock run and change fit in 64 units.
/// Read the stock's whole settings after sign-in, then commit both edits together.
pub fn settings_setup(client: &Path, following: Vec<ctl::Call>) -> Vec<ctl::Step> {
    use ctl::{Call, Step};
    let file = client.join("settings.json");
    let mut steps = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&["settings", "show", "--json"])),
        Step::Run(Call::new(&["settings", "show", "charter", "1", "--json"])),
        Step::Run(Call::new(&["settings", "show", "template", "1", "--json"])),
        Step::Then(Box::new(move |calls| {
            let charter = ctl::json(&calls[2]);
            let template = ctl::json(&calls[3]);
            let mut chat = charter["value"].clone();
            chat["waiting"] = 1_000_000.into();
            let mut recipe = template["value"].clone();
            recipe["budget"] = serde_json::json!({"as_allowed":{"cap":64}});
            let mut change = serde_json::json!({"items":[
                {"setting":charter["setting"],"set":chat},
                {"setting":template["setting"],"set":recipe}
            ]});
            let rules = client_landing_items(&file);
            change["items"].as_array_mut().expect("GitHub settings").extend(rules);
            fs::write(&file, serde_json::to_vec(&change).expect("GitHub settings JSON")).expect("settings FILE");
            ctl::settings_change(&calls[1], &file, "github-settings")
        })),
    ];
    steps.extend(following.into_iter().map(Step::Run));
    steps
}

pub fn restart() {
    let clock = Clock::new();
    let began = clock.now().now;
    let scratch = Scratch::new("github_git");
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
    members.up(&["forge", "llm"]);
    let mut world = real::World::new(Judge {
        members,
        finding: None,
        deadline: clock.now().now.saturating_add(WITHIN),
        stage: Stage::Fakes,
        scratch: scratch.path().to_path_buf(),
        clients,
        workers,
        app: None,
        identity: None,
        adoption: None,
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
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(cluster::Root::client);
    let outcome = world.run(&clock, WITHIN);
    assert!(outcome.procs.iter().all(Host::is_empty), "every process and client settled");
    scratch.remove();
    eprintln!("github git: complete {:?}", clock.now().now.saturating_since(began));
}

#[derive(Debug)]
enum Stage {
    Fakes,
    Worker,
    Install,
    Adopt,
    Chat,
    Tree,
    Coder,
    BeforeRestart,
    StopWorker,
    StopEngine,
    RestartEngine,
    RestartWorker,
    Restored,
    AdoptionRestored,
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
    adoption: Option<Value>,
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
}
impl Judge {
    fn advance(&mut self, now: Time, hosts: &mut [cluster::Root]) -> Result<(), String> {
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = Stage::Stopping;
            return Ok(());
        }
        let host = hosts.last_mut().ok_or_else(|| "client root".to_owned())?;
        if matches!(self.stage, Stage::Fakes) && self.members.is_up() {
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
            fs::write(&self.engine, serde_json::to_vec(&document).map_err(|_| "engine JSON".to_owned())?)
                .map_err(|_| "fixed workers listener".to_owned())?;
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
                "github restart: ready {:?}",
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
                    | Stage::Tree
                    | Stage::Coder
                    | Stage::BeforeRestart
                    | Stage::Restored
                    | Stage::AdoptionRestored
            )
        {
            let cluster::Root::Client { calls, .. } = host else { return Err("client root".to_owned()) };
            let calls = calls.take().ok_or_else(|| "completed calls".to_owned())?;
            self.called(now, host, &calls)?;
        }

        Ok(())
    }

    fn mark(&self, step: &str) {
        eprintln!("github git: {step} {:?}", Clock::new().now().now.saturating_since(self.began));
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
        self.queue(host, vec![Self::adoption()])?;
        self.stage = Stage::AdoptionRestored;

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
    fn called(&mut self, now: Time, host: &mut cluster::Root, calls: &ctl::Calls) -> Result<(), String> {
        for ended in calls.ends() {
            self.outputs.push(ended.stdout.clone());
            self.outputs.push(ended.stderr.clone());
            if ended.exit != Some(Exit::Code(0)) {
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
            Stage::Adopt => {
                let packet = json_call(calls.ends().last().ok_or_else(|| "adoption".to_owned())?)?;
                if !packet["adopted"].is_object() {
                    return Err(format!("Must: first life adopts acme/app: {packet}"));
                }
                self.adoption = Some(packet["adopted"].clone());
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
            }
            Stage::Chat => {
                self.mark("signed in, key usable, adopted, chat created");
                let packet = json_call(calls.ends().last().ok_or_else(|| "chat creation".to_owned())?)?;
                if packet["outcome"]["done"]["document"]["created"]["first"] != 1 {
                    return Err(format!("{packet}"));
                }
                self.queue(host, vec![ctl::Call::new(&["task", "show", "1", "--tree", "--json"])])?;
                self.stage = Stage::Tree;
            }
            Stage::Tree => self.tree(host, &json_call(calls.ends().first().ok_or("completed ctl call")?)?)?,
            Stage::Coder => {
                Self::coder(&json_call(calls.ends().first().ok_or("completed ctl call")?)?)?;
                self.mark("coder delivered");
                self.queue(host, vec![ctl::Call::new(&["forge", "list", "--json"])])?;
                self.stage = Stage::BeforeRestart;
            }
            Stage::BeforeRestart => {
                let row = Self::row(&json_call(calls.ends().first().ok_or("completed ctl call")?)?)?;
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
                self.mark("first life delivered; restarting");
                self.restart_began = Some(now);
                self.members.stop_one("w1");
                self.stage = Stage::StopWorker;
            }
            Stage::Restored => {
                self.restored(host, &json_call(calls.ends().last().ok_or_else(|| "restored forges".to_owned())?)?)?;
            }
            Stage::AdoptionRestored => {
                let packet = json_call(calls.ends().first().ok_or("completed ctl call")?)?;
                if packet["adopted"] != *self.adoption.as_ref().ok_or_else(|| "first adoption".to_owned())? {
                    return Err("Must: same adoption key answers its kept adopted outcome after restart".to_owned());
                }
                self.mark("adoption found after restart");
                eprintln!(
                    "github restart: restored {:?}",
                    now.saturating_since(self.restart_began.ok_or_else(|| "restart began".to_owned())?)
                );
                self.members.stop();
                self.stage = Stage::Stopping;
            }
            Stage::Fakes
            | Stage::Worker
            | Stage::StopWorker
            | Stage::StopEngine
            | Stage::RestartEngine
            | Stage::RestartWorker
            | Stage::Stopping => return Err("active calls".to_owned()),
        }

        Ok(())
    }
    fn tree(&mut self, host: &mut cluster::Root, packet: &Value) -> Result<(), String> {
        let tree = &packet["tree"];
        // Watches carry their last digest forward, so discovering a new child never polls.
        if tree["rows"].as_array().ok_or_else(|| "tree rows".to_owned())?.iter().any(|row| row["chip"]["task"] == CODER)
        {
            self.mark("coder created");
            self.queue(
                host,
                vec![ctl::Call::new(&["task", "show", &CODER.to_string(), "--until", "ended", "--json"])],
            )?;
            self.stage = Stage::Coder;
        } else {
            let digest = tree["digest"].as_str().ok_or_else(|| format!("tree digest: {packet}"))?;
            self.queue(host, vec![ctl::Call::new(&["task", "tree", "1", "--since", digest, "--json"])])?;
        }

        Ok(())
    }
    fn coder(task: &Value) -> Result<(), String> {
        let task = &task["task"];
        if task["chip"]["task"] != CODER {
            return Err(format!("{task}"));
        }
        let owner = &task["chip"]["details"]["owner"];
        if !owner["details"]["ending"]["done"].is_object() {
            return Err(format!("coder delivered: {task}"));
        }
        if !owner["details"]["ending"]["done"]["change"].is_object() {
            return Err(format!("change delivered: {task}"));
        }
        // begin() checks that this engine alone has exactly one worker position,
        // w1, and its linked line. Only that worker can have executed this coder.

        Ok(())
    }
    #[expect(clippy::too_many_lines, reason = "the forge observations keep their returning Musts in order")]
    fn forge(&self, read_objects: bool) -> Result<(), String> {
        let bytes = fs::read(&self.observations).map_err(|_| "forge observations".to_owned())?;
        let bytes = if read_objects { certified_prefix(&bytes)? } else { &bytes };
        let lines = std::str::from_utf8(bytes).map_err(|_| "observation UTF-8".to_owned())?;
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
        // Only the push's branch: once CI passes, the change may merge and move
        // main in either life; the landing's Musts are not this story's.
        let moved: Vec<_> =
            rows.iter().filter(|row| row["observation"] == "moved" && row["branch"] != "main").collect();
        if moved.len() != 1 {
            return Err("Must: one pushed branch across both lives".to_owned());
        }
        let identity = &self.identity.as_ref().ok_or_else(|| "first identity".to_owned())?["id"];
        for row in rows.iter().filter(|row| row["observation"] == "moved" && row["branch"] == "main") {
            if &row["by"] != identity {
                return Err("Must: the base branch moves only by temper's own App".to_owned());
            }
        }
        if moved[0]["repository"] != "acme/app" {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                moved[0]["repository"], "acme/app"
            ));
        }
        if moved[0]["branch"] == "main" {
            return Err(format!(
                "assertion `left != right` failed\n  left: {:?}\n right: {:?}",
                moved[0]["branch"], "main"
            ));
        }
        if !moved[0]["from"].is_null() {
            return Err("push creates its branch".to_owned());
        }
        if moved[0]["by"] != self.identity.as_ref().ok_or_else(|| "first identity".to_owned())?["id"] {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                moved[0]["by"],
                self.identity.as_ref().ok_or_else(|| "first identity".to_owned())?["id"]
            ));
        }
        let commit = moved[0]["to"].as_u64().ok_or_else(|| "pushed head".to_owned())?;
        if !read_objects {
            return Ok(());
        }
        let address = self
            .members
            .capture("forge", "address")
            .ok_or_else(|| "forge address".to_owned())?
            .parse()
            .map_err(|_| "socket".to_owned())?;
        let objects = try_objects(address, commit)?;
        let object = objects["objects"]
            .as_array()
            .ok_or_else(|| "objects".to_owned())?
            .iter()
            .find(|object| object["commit"] == commit)
            .ok_or_else(|| "pushed commit".to_owned())?;
        let tree = object["tree"].as_array().ok_or_else(|| "pushed tree".to_owned())?;
        let mut file = None;
        for candidate in tree {
            if decode(&candidate["path"])?.as_ref() == b"README.md" {
                file = Some(candidate);
                break;
            }
        }
        let file = file.ok_or("README")?;
        let expected = [temper_fake::forge::FILES[0].1, EDIT].concat();
        if decode(&file["content"])?.as_ref() != expected {
            return Err("the branch holds the coder's edit".to_owned());
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
            if stopped.iter().map(|member| member.name.as_str()).collect::<Vec<_>>()
                != ["w1", "engine", "w1", "engine", "llm", "forge"]
            {
                return Err(stopped_members_details(stopped)?);
            }
            for (index, member) in stopped.iter().enumerate() {
                if member.exit != Some(Exit::Code(0)) {
                    return Err(format!("{}: {}", member.name, String::from_utf8_lossy(&member.stderr)));
                }
                if member.ending != if index < 2 { cluster::Ending::Alone } else { cluster::Ending::Cluster } {
                    return Err(format!(
                        "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                        member.ending,
                        if index < 2 { cluster::Ending::Alone } else { cluster::Ending::Cluster }
                    ));
                }
                if member.killed || !member.survivors.is_empty() {
                    return Err("no surviving process".to_owned());
                }
            }
            self.forge(false)?;
            let app = self.app.as_ref().ok_or_else(|| "made App".to_owned())?;
            let scan = Scan::new(app.pem.expose())?;
            scan.files(&self.scratch, &secrets(&self.members.dir("engine"))?, &self.scratch.join("github-app.pem"))?;
            for member in stopped {
                scan.clean(&member.stdout)?;
                scan.clean(&member.stderr)?;
            }
            for output in &self.outputs {
                scan.clean(output)?;
            }
            self.mark("Musts: adoption, one creation, no uninstall, one push, secrecy and teardown passed");
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
fn stopped_members_details(stopped: &[cluster::Stopped]) -> Result<String, String> {
    use std::fmt::Write as _;
    const STDERR_TAIL_BYTES: usize = 2048;
    let mut details = String::new();
    for member in stopped {
        let stderr = String::from_utf8_lossy(&member.stderr);
        let mut start = stderr.len().saturating_sub(STDERR_TAIL_BYTES);
        while !stderr.is_char_boundary(start) {
            start += 1;
        }
        writeln!(details, "{} exit={:?}\nstderr tail:\n{}", member.name, member.exit, &stderr[start..])
            .map_err(|_| "member diagnostic".to_owned())?;
    }
    Ok(details)
}

pub fn stopped_members_details_include_exits_and_bounded_stderr() {
    let stderr = [b"discard old output\n".as_slice(), "é".repeat(2048).as_bytes(), b"listener bind failed\n"].concat();
    let stopped = [
        cluster::Stopped {
            name: "engine".into(),
            life: 0,
            ending: cluster::Ending::Cluster,
            exit: Some(Exit::Code(1)),
            killed: false,
            survivors: Vec::new(),
            stdout: Vec::new(),
            stderr,
        },
        cluster::Stopped {
            name: "forge".into(),
            life: 0,
            ending: cluster::Ending::Cluster,
            exit: Some(Exit::Code(0)),
            killed: false,
            survivors: Vec::new(),
            stdout: Vec::new(),
            stderr: vec![0xff],
        },
    ];
    let details = stopped_members_details(&stopped).expect("member diagnostic");
    assert!(details.contains("engine exit=Some(Code(1))"));
    assert!(details.contains("forge exit=Some(Code(0))"));
    assert!(!details.contains("discard old output"));
    let tail = details
        .split_once("stderr tail:\n")
        .expect("engine stderr")
        .1
        .split_once("\nforge exit=")
        .expect("next member")
        .0;
    assert!(tail.len() <= 2048, "stderr tail is bounded in UTF-8 bytes");
    assert!(tail.ends_with("listener bind failed\n"));
    assert!(details.ends_with("stderr tail:\n�\n"), "invalid stderr bytes remain printable");
}

/// The live observations prefix through its last complete checkpoint (protocol/setup/playground.md).
fn certified_prefix(bytes: &[u8]) -> Result<&[u8], String> {
    let mut end = bytes.len();
    for line in bytes.split_inclusive(|byte| *byte == b'\n').rev() {
        if line.ends_with(b"\n")
            && serde_json::from_slice::<Value>(line).is_ok_and(|row| row["observations_lost"].is_u64())
        {
            return Ok(&bytes[..end]);
        }
        end -= line.len();
    }
    Err("forge observations have a complete checkpoint".into())
}
fn decode(value: &Value) -> Result<Box<[u8]>, String> {
    base64::decode(base64::Alphabet::Standard, value.as_str().ok_or_else(|| "base64 text".to_owned())?.as_bytes(), 8192)
        .map_err(|_| "file bytes".to_owned())
}
// The fake's object route is credential-free and does not add a repository git call.
// Like webhooks.rs, the referee makes one bounded request to a separate process.
pub(super) fn objects(address: SocketAddr, commit: u64) -> Value {
    try_objects(address, commit).unwrap_or_else(|why| panic!("{why}"))
}
pub(super) fn try_objects(address: SocketAddr, commit: u64) -> Result<Value, String> {
    let within = skein_lib::Duration::from_secs(10);
    let mut socket = Socket::connect("the GitHub object route", within, address, Scope::World)
        .map_err(|_| "object route".to_owned())?;
    let body = format!("{{\"commits\":[{commit}]}}");
    let request = format!(
        "POST /.fake/git/objects HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket
        .write_all("the GitHub objects request", within, request.as_bytes())
        .map_err(|_| "object request".to_owned())?;
    let response =
        socket.read_to_end("the GitHub objects reply", within, 65_536).map_err(|_| "object response".to_owned())?;
    if !response.starts_with(b"HTTP/1.1 200") {
        return Err(format!("{}", String::from_utf8_lossy(&response)));
    }
    let at = response.windows(4).position(|bytes| bytes == b"\r\n\r\n").ok_or("HTTP head")?;
    serde_json::from_slice(&response[at + 4..]).map_err(|_| "objects JSON".to_owned())
}

pub(crate) struct App {
    pub(crate) id: u64,
    pub(crate) pem: Secret,
}
// These requests go to other processes' real listeners; unlike the one-ring
// tier, a bounded synchronous exchange cannot hold the server's loop up.
pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) location: Option<String>,
    pub(crate) body: Vec<u8>,
}
pub(crate) fn request_url(
    address: SocketAddr,
    method: &str,
    url: &str,
    media: &str,
    body: &[u8],
) -> Result<Response, String> {
    let origin = format!("http://{address}");
    let target = url.strip_prefix(&origin).ok_or_else(|| "only the expected fake or engine origin".to_owned())?;
    if !target.starts_with('/') {
        return Err("local HTTP target".to_owned());
    }
    request(address, method, target, media, body)
}
fn request(address: SocketAddr, method: &str, target: &str, media: &str, body: &[u8]) -> Result<Response, String> {
    let within = skein_lib::Duration::from_secs(10);
    let mut socket = Socket::connect("the GitHub local HTTP listener", within, address, Scope::World)
        .map_err(|_| "local HTTP listener".to_owned())?;
    let version = skein_forge::github::pin::API_VERSION;
    let request = format!(
        "{method} {target} HTTP/1.1\r\nHost: {address}\r\nUser-Agent: temper-test\r\nX-GitHub-Api-Version: {version}\r\nContent-Type: {media}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    socket.write_all("the GitHub HTTP request", within, request.as_bytes()).map_err(|_| "HTTP head".to_owned())?;
    socket.write_all("the GitHub HTTP body", within, body).map_err(|_| "HTTP body".to_owned())?;
    let bytes =
        socket.read_to_end("the GitHub HTTP reply", within, 65_537).map_err(|_| "bounded HTTP response".to_owned())?;
    if bytes.len() > 65_536 {
        return Err("HTTP response bound".to_owned());
    }
    let at = bytes.windows(4).position(|bytes| bytes == b"\r\n\r\n").ok_or("HTTP head")?;
    let head = std::str::from_utf8(&bytes[..at]).map_err(|_| "HTTP header text".to_owned())?;
    let status = head
        .lines()
        .next()
        .ok_or_else(|| "status line".to_owned())?
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "status".to_owned())?
        .parse()
        .map_err(|_| "HTTP status".to_owned())?;
    let location = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .map(|(_, value)| value.trim().to_owned());
    Ok(Response { status, location, body: bytes[at + 4..].to_vec() })
}
pub(crate) fn make_app(address: SocketAddr, clients: SocketAddr, scratch: &Path) -> Result<App, String> {
    use skein_forge::github::setup;
    use std::fmt::Write as _;
    let origin = format!("http://{clients}");
    let setup_url = format!("{origin}/forges/1/github/setup");
    let redirect = format!("{origin}/forges/1/github/redirect");
    let manifest = setup::manifest(
        &setup::Values {
            name: b"temper-person",
            url: origin.as_bytes(),
            hook_url: b"https://temper-test.invalid/",
            redirect_url: redirect.as_bytes(),
            setup_url: setup_url.as_bytes(),
        },
        &temper_fake::forge::git_limits().documents.writer,
    )
    .map_err(|_| "temper manifest".to_owned())?;
    let mut form = String::from("manifest=");
    for byte in &manifest {
        write!(form, "%{byte:02X}").map_err(|_| "form byte".to_owned())?;
    }
    let page = request(
        address,
        "POST",
        "/settings/apps/new?state=person",
        "application/x-www-form-urlencoded",
        form.as_bytes(),
    )?;
    if page.status != 200 {
        return Err("Must: manifest form makes the person's App".to_owned());
    }
    let html = std::str::from_utf8(&page.body).map_err(|_| "App creation page".to_owned())?;
    let code = html
        .split_once("?code=")
        .ok_or_else(|| "conversion link".to_owned())?
        .1
        .split('&')
        .next()
        .ok_or_else(|| "code".to_owned())?;
    let converted = request(address, "POST", &format!("/app-manifests/{code}/conversions"), "application/json", b"")?;
    if converted.status != 201 {
        return Err("Must: manifest conversion over HTTP".to_owned());
    }
    let value: Value = serde_json::from_slice(&converted.body).map_err(|_| "conversion JSON".to_owned())?;
    let app = App {
        id: value["id"].as_u64().ok_or_else(|| "converted App id".to_owned())?,
        pem: Secret::new(value["pem"].as_str().ok_or_else(|| "converted PEM".to_owned())?.as_bytes().into()),
    };
    let path = scratch.join("github-app.pem");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .map_err(|_| "private App key".to_owned())?;
    file.write_all(app.pem.expose()).map_err(|_| "downloaded PEM".to_owned())?;
    if fs::metadata(&path).map_err(|_| "PEM metadata".to_owned())?.permissions().mode() & 0o777 != 0o600 {
        return Err(format!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            fs::metadata(&path).map_err(|_| "PEM metadata".to_owned())?.permissions().mode() & 0o777,
            0o600
        ));
    }
    Ok(app)
}

pub(crate) fn secrets(engine: &Path) -> Result<PathBuf, String> {
    let path = engine.join("data/state/generations/1/secrets");
    if !path.is_dir() {
        return Err("excluded directory is jig's secrets store".into());
    }
    Ok(path)
}

pub(crate) struct Scan {
    pem: Vec<u8>,
    der: Box<[u8]>,
    encoded: Vec<u8>,
}

impl Scan {
    pub(crate) fn new(pem: &[u8]) -> Result<Self, String> {
        let compact: String = std::str::from_utf8(pem)
            .map_err(|_| "PEM text")?
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        let der = base64::decode(base64::Alphabet::Standard, compact.as_bytes(), 4096).map_err(|_| "App DER")?;
        Ok(Self { pem: pem.to_vec(), der, encoded: compact.into_bytes() })
    }
    pub(crate) fn clean(&self, bytes: &[u8]) -> Result<(), String> {
        if bytes.windows(40).any(|part| part.starts_with(b"ghs_") && part[4..].iter().all(u8::is_ascii_alphanumeric)) {
            return Err("Must: no installation token outside jig's secrets store".into());
        }
        for (name, key) in
            [("PEM", self.pem.as_slice()), ("DER", self.der.as_ref()), ("base64 DER", self.encoded.as_slice())]
        {
            if skein_lib::bytes::find(bytes, key).is_some() {
                return Err(format!("Must: no App {name} outside jig's secrets store and downloaded PEM"));
            }
        }
        Ok(())
    }
    pub(crate) fn files(&self, root: &Path, secrets: &Path, downloaded: &Path) -> Result<(), String> {
        if !secrets.is_dir() {
            return Err("excluded directory is jig's secrets store".into());
        }
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            if path == secrets || path == downloaded {
                continue;
            }
            let metadata = fs::symlink_metadata(&path).map_err(|_| format!("scratch metadata: {}", path.display()))?;
            if metadata.is_dir() {
                for entry in fs::read_dir(&path).map_err(|_| format!("scratch directory: {}", path.display()))? {
                    pending.push(entry.map_err(|_| "scratch entry")?.path());
                }
            } else if metadata.is_file() {
                let bytes = fs::read(&path).map_err(|_| format!("scratch file: {}", path.display()))?;
                self.clean(&bytes).map_err(|why| format!("{}: {why}", path.display()))?;
            }
        }
        Ok(())
    }
}

pub fn file_scan_excludes_only_jigs_generation_secrets_store() {
    let scratch = Scratch::new(&format!("github_file_scan-{}", std::process::id()));
    let engine = scratch.path().join("engine");
    let store = engine.join("data/state/generations/1/secrets");
    assert_eq!(secrets(&engine), Err("excluded directory is jig's secrets store".into()));
    fs::create_dir_all(&store).expect("generation secrets store");
    let pem = b"-----BEGIN PRIVATE KEY-----\nZmFrZS1hcHAta2V5LWJ5dGVz\n-----END PRIVATE KEY-----\n";
    let scan = Scan::new(pem).expect("synthetic PEM");
    fs::write(store.join("log-1"), &scan.der).expect("key in secrets store");
    let downloaded = scratch.path().join("github-app.pem");
    fs::write(&downloaded, pem).expect("downloaded PEM");
    let excluded = secrets(&engine).expect("generation secrets store");
    assert_eq!(scan.files(scratch.path(), &excluded, &downloaded), Ok(()));

    fs::write(engine.join("ordinary"), &scan.der).expect("key outside secrets store");
    let finding = scan.files(scratch.path(), &excluded, &downloaded).expect_err("ordinary file leaks DER");
    assert_eq!(
        finding,
        format!(
            "{}: Must: no App DER outside jig's secrets store and downloaded PEM",
            engine.join("ordinary").display()
        )
    );
    assert!(!finding.contains("fake-app-key-bytes"), "finding never includes matched bytes");
    scratch.remove();
}

fn client_landing_items(file: &Path) -> Vec<Value> {
    let path = file.parent().expect("client directory").join("landing-settings.json");
    if !path.exists() {
        return Vec::new();
    }
    let items: Value =
        serde_json::from_slice(&fs::read(path).expect("landing settings")).expect("landing settings JSON");
    items.as_array().expect("landing items").clone()
}

fn json_call(ended: &super::Ended) -> Result<Value, String> {
    serde_json::from_slice(&ended.stdout).map_err(|error| {
        format!("ctl JSON: {error}; exit {:?}; stderr: {}", ended.exit, String::from_utf8_lossy(&ended.stderr))
    })
}
