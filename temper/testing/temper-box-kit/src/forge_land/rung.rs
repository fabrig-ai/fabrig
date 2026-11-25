//! The ladder's real-model walks and their process proof (testing/live-tier.md).
//!
//! Like `forge_land`'s oracle, this driver uses bounded child commands. Its box
//! outlives the live test in its own session so proof-live can return the held
//! account once the test ends. Offline, with no launcher, it dies with the test.
//! Credentials go to ctl and curl by path alone. No bot credential or store is read
//! by the driver.
use super::super::{Process, configuration::turn_bounds, held_login::LOCK_FD_9};
use serde_json::{Value, json};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Deadline, Expired, Referee, Starved,
    end_to_end::{Binary, Expectation, Mode},
    real,
};

use std::{
    fs,
    io::Write,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
};

const REQUEST: &str =
    "Add CSV export to widget. Before anything is changed, have the work tell me its plan and wait for my go-ahead.";
const ANSWER: &str = "Go ahead. The CSV's first line must be the header fruit,count.";
const WHOLE: u64 = 374_930;

fn now() -> Time {
    skein_shell::Clock::new().now().now
}

fn require(yes: bool, message: &str) -> Result<(), String> {
    if yes { Ok(()) } else { Err(message.to_owned()) }
}

fn write_json(path: &Path, value: &Value) {
    super::super::write_private(path, &serde_json::to_vec(value).expect("configuration JSON"));
}

/// Setup may print progress JSON before its final answer. Validate every
/// document and return the last one, retaining all of them in the transcript.
pub fn ctl_json(bytes: &[u8]) -> Result<Value, String> {
    let mut last = None;
    for document in serde_json::Deserializer::from_slice(bytes).into_iter::<Value>() {
        last = Some(document.map_err(|error| format!("ctl JSON: {error}"))?);
    }
    last.ok_or_else(|| "ctl returned no JSON document".into())
}

/// `render/outcome.rs` and `document/act_answers.rs`: the top-level `task`
/// names an operation target, not the task created by the owner's chat act.
pub fn created_chat(value: &Value) -> Result<u64, String> {
    let created = &value["outcome"]["done"]["document"]["created"];
    let first = created["first"].as_u64().ok_or("chat created.first missing")?;
    require(created["last"] == first && first != 0, "one chat created")?;
    Ok(first)
}

/// `change.rs::json` wraps waited panels; accounts keep their specialized
/// flat envelope, while Jig's other waits retain their named subject.
pub fn waited(args: &[&str], value: Value) -> Result<Value, String> {
    require(value["met"] == true, "wait condition unmet")?;
    if args.starts_with(&["change", "show"]) {
        let panel = value.get("change").filter(|v| v.is_object()).ok_or("waited change panel missing")?;
        Ok(panel.clone())
    } else {
        Ok(value)
    }
}

/// `read_sets.rs::encode_tasks` uses the tasks subject for its first live
/// page; the ended read uses `read_queries.rs::encode_task_page` instead.
pub fn child_rows(live: &Value, ended: &Value) -> Result<Vec<Value>, String> {
    require(live["tasks"]["more"] == false && live["read_on"].is_null(), "whole live child set")?;
    let mut rows = live["tasks"]["rows"].as_array().ok_or("live children missing")?.clone();
    match &ended["ended"] {
        Value::String(none) if none == "none" => {}
        Value::Object(group) => {
            let page = group.get("rows").ok_or("ended page missing")?;
            require(page["after"].is_null() && page["cut"] == false, "whole ended child set")?;
            rows.extend(page["rows"].as_array().ok_or("ended children missing")?.iter().cloned());
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Array(_) => {
            return Err("ended children group missing".into());
        }
    }
    Ok(rows)
}

/// Explicit messages are `Activity::Words`, with their author in `by`;
/// other input, such as results and forge news, uses the separate `Heard` form.
pub fn plan_question(rows: &[Value], goal: u64, question: u64) -> Result<&str, String> {
    rows.iter()
        .find_map(|row| {
            let words = &row["event"]["owner"]["source"]["line"]["words"];
            (words["kind"] == "question" && words["by"]["task"] == goal && words["message"] == question)
                .then(|| words["words"].as_str())
                .flatten()
        })
        .ok_or_else(|| "the inbox question was not asked by the planning work".into())
}

/// The delivered approval's activity timestamp, not a call argument or a
/// later run's time. `read_owner_activity.rs` encodes explicit answer words.
#[must_use]
pub fn approval_at(rows: &[Value], chat: u64) -> Option<u64> {
    rows.iter().find_map(|row| {
        let words = &row["event"]["owner"]["source"]["line"]["words"];
        (words["kind"]["answer"]["question"].is_u64() && words["by"]["task"] == chat && words["words"] == ANSWER)
            .then(|| row["at"].as_u64())
            .flatten()
    })
}

/// A plain task read is its subject; adding `--tree` makes a compound read.
pub fn task_digest(value: &Value) -> Result<&str, String> {
    value["digest"].as_str().or_else(|| value["task"]["digest"].as_str()).ok_or_else(|| "task digest missing".into())
}

/// The guide's init and the existing owned-forge fixture, with QA's model/room
/// recipe (temper/docs/development/qa.md, 3; protocol/configuration/settings-output.md, 1),
/// before the first start.
#[expect(
    clippy::must_use_candidate,
    reason = "subprocess worlds use the files; focused checks retain the typed figures"
)]
pub fn deployment(paths: &[PathBuf; 3]) -> temper_playground::documents::Configuration {
    super::live_deployment(paths, super::LiveModel::ChatGpt { account: None });
    let config = paths[0].join(temper_config_shell::FILE);
    let value = crate::limit_settings::configuration(&paths[0]);
    let worker = temper_agent_fixtures::read(&paths[0]);
    let (value, setting) = compose_deployment(value, worker);
    setting.lay(&paths[1]);
    crate::limit_settings::remember(&paths[0], &value);
    temper_box_shell::operations_in_with_limits(
        "temper",
        &[],
        b"/",
        &deployment_directories(paths),
        &crate::read_start(&config, temper_config_shell::Shape::Standalone).document,
        &temper_service::stock::Given::Stock,
        &value.jig,
        &value.limits,
    )
    .unwrap_or_else(|message| panic!("B0 setup refused: {message}"));
    value
}

/// B0's sizing inputs at first start, before any account, store or backend.
pub struct DeploymentProfile {
    pub directories: temper_box_shell::directories::Directories,
    pub document: Box<[u8]>,
    pub configuration: temper_playground::documents::Configuration,
    pub agents: Box<[u8]>,
}

/// Compose the same init and B0 edits with fixed listener addresses instead of drawing ports.
#[must_use]
pub fn deployment_profile(paths: &[PathBuf; 3]) -> DeploymentProfile {
    // Native box init can create the local provider key; sizing needs only its presence.
    let document = temper_config_shell::init_document(true);
    let start = serde_json::from_slice(&document).expect("init start");
    let providers = temper_service::stock::providers();
    let worker = temper_agent_fixtures::with_figures(&start, &temper_service::stock::agents(), &paths[1], &providers);
    let mut value = temper_playground::documents::Configuration::from_jig(start);
    value["world_providers"] = temper_agent_fixtures::providers::flat(&providers);
    super::live_limits(&mut value);
    let (configuration, setting) = compose_deployment(value, worker);
    DeploymentProfile { directories: deployment_directories(paths), document, configuration, agents: setting.bytes() }
}

fn deployment_directories(paths: &[PathBuf; 3]) -> temper_box_shell::directories::Directories {
    let mut directories = super::super::configuration::directories(paths[0].parent().expect("deployment root"));
    directories.config = paths[0].as_os_str().as_bytes().into();
    directories.data = paths[1].as_os_str().as_bytes().into();
    directories.cache = paths[2].as_os_str().as_bytes().into();
    directories
}

fn compose_deployment(
    mut value: temper_playground::documents::Configuration,
    mut worker: Value,
) -> (temper_playground::documents::Configuration, temper_agent_fixtures::Setting) {
    for table in ["providers", "endpoints"] {
        temper_agent_fixtures::providers::input(&mut value)[table]
            .as_array_mut()
            .expect("LLM table")
            .retain(|row| row["name"] != "anthropic");
    }
    worker["endpoints"].as_array_mut().expect("endpoints").retain(|endpoint| endpoint["name"] != "anthropic");
    // Keep the stock endpoint ceiling: its response holds the escaped stock tool payload.
    // B0's model setting requests 32,000 tokens within that ceiling.
    worker["endpoints"][0]["limits"]["reasoning_item"] = 32_768.into();
    // B0 uses one call per response.
    worker["profile"]["declared"]["calls_per_response"] = 1.into();
    // The stock carriers also hold the stock's 82-message inbox; keep their
    // floor when the one-call recipe makes the agent's turn smaller.
    let stock_turn = value.jig.hub.turn_bytes;
    let turn = turn_bounds::turn_body(&worker).max(u32::try_from(stock_turn).expect("stock turn fits"));
    turn_limits(&mut value, turn);
    // Walk 1's admitted declarations, not a phase-bound increase.
    value.limits.root.transcript_memory = 817_372_775_016_049_u64;
    assert_eq!(value.jig.core.people.requests, 256);
    assert_eq!(value.jig.hub.turn_bytes, u64::from(turn));
    assert_eq!(value.limits.root.loads.loads, 32);
    assert_eq!(worker["endpoints"][0]["limits"]["output"], 48_000);
    assert_eq!(worker["endpoints"][0]["limits"]["reasoning_item"], 32_768);
    let providers = temper_agent_fixtures::providers::take(&mut value).expect("deployment providers");
    let setting = temper_agent_fixtures::compose_setting(&mut value, &worker, &providers);
    value.jig_over("hub.run_calls", 1);
    (value, setting)
}

fn turn_limits(value: &mut temper_playground::documents::Configuration, turn: u32) {
    turn_bounds::apply(value, turn);
    value.jig_over("people.requests", 256);
    value.temper_over("root.loads.loads", 32);
}

/// A private command transcript, with a hard wall bound and no pipe backpressure.
struct Driver {
    binary: PathBuf,
    store: PathBuf,
    client: PathBuf,
    sequence: usize,
    until: Time,
    deadline: Deadline,
    phases: Vec<(&'static str, Deadline)>,
    active: Option<usize>,
    starved: Option<Starved>,
    phase: &'static str,
    requested: Option<Time>,
    api: String,
    header: PathBuf,
    project: String,
    forge: u64,
}

struct Transcript<F> {
    stdout: (PathBuf, F),
    stderr: (PathBuf, F),
}

impl Driver {
    fn owned(store: &Path, client: &Path) -> Self {
        Self::new_at(
            store,
            client,
            skein_scratch::paths::built("temper-box", "temper-box"),
            Deadline::arm("the rung's setup", Duration::from_secs(300)),
        )
    }

    fn new_at(store: &Path, client: &Path, binary: PathBuf, deadline: Deadline) -> Self {
        Self {
            binary,
            store: store.into(),
            client: client.into(),
            sequence: 0,
            until: deadline.until(),
            deadline,
            phases: Vec::new(),
            active: None,
            starved: None,
            phase: "the rung's setup",
            requested: None,
            api: String::new(),
            header: PathBuf::new(),
            project: String::new(),
            forge: 0,
        }
    }

    fn arm(&mut self, phase: &'static str, within: Duration) {
        self.arm_deadline(phase, Deadline::arm(phase, within));
    }

    fn arm_deadline(&mut self, phase: &'static str, deadline: Deadline) {
        self.phase = phase;
        self.until = deadline.until();
        self.deadline = deadline;
        self.active = None;
    }

    fn deadline(&mut self) -> &mut Deadline {
        match self.active {
            Some(index) => &mut self.phases[index].1,
            None => &mut self.deadline,
        }
    }

    fn check_wait(&mut self) -> Result<(), String> {
        self.check_wait_at(now(), Deadline::expired)
    }

    fn check_wait_at(&mut self, now: Time, expire: impl FnOnce(&Deadline) -> Expired) -> Result<(), String> {
        if now < self.until {
            return Ok(());
        }
        let message = format!("phase {} exceeded its bound", self.phase);
        match expire(self.deadline()) {
            Expired::Starved(starved) => {
                let message = format!("{message}: {}", starved.figures());
                self.starved = Some(starved);
                Err(message)
            }
            Expired::Failed(figures) => Err(format!("{message}: {figures}")),
        }
    }

    fn request(&mut self) {
        self.request_at(now(), Deadline::arm);
    }

    fn request_at(&mut self, now: Time, mut arm: impl FnMut(&'static str, Duration) -> Deadline) {
        self.requested = Some(now);
        self.phases = [
            ("inbox question", 40_480),
            ("person's answer sent", 186_660),
            ("answer reached goal", 190_910),
            ("change delegated", 214_930),
            ("first code write", 257_930),
            ("pull request and head CI", 286_410),
            ("agent review approved", 363_470),
            ("landed", 364_410),
            ("main CI passed", 367_410),
            ("chat's last word and whole", WHOLE),
        ]
        .into_iter()
        .map(|(phase, milliseconds)| (phase, arm(phase, Duration::from_millis(milliseconds))))
        .collect();
    }

    fn execute(&mut self, command: Command, input: Option<&str>, keep: bool) -> Result<Vec<u8>, String> {
        self.execute_within(command, input, keep, None)
    }

    fn execute_within(
        &mut self,
        command: Command,
        input: Option<&str>,
        keep: bool,
        within: Option<Duration>,
    ) -> Result<Vec<u8>, String> {
        self.execute_with(
            now(),
            Deadline::expired,
            |path| fs::File::create(path).map_err(|e| e.to_string()),
            |driver, number, transcript| driver.run_call(number, transcript, command, input, keep, within),
        )
    }

    fn execute_with<F>(
        &mut self,
        now: Time,
        expire: impl FnOnce(&Deadline) -> Expired,
        mut open: impl FnMut(&Path) -> Result<F, String>,
        run: impl FnOnce(&mut Self, usize, Transcript<F>) -> Result<Vec<u8>, String>,
    ) -> Result<Vec<u8>, String> {
        self.check_wait_at(now, expire)?;
        let number = self.sequence;
        self.sequence += 1;
        let stdout = self.store.join(format!("call-{number}.out"));
        let stderr = self.store.join(format!("call-{number}.err"));
        let out = open(&stdout)?;
        let err = open(&stderr)?;
        run(self, number, Transcript { stdout: (stdout, out), stderr: (stderr, err) })
    }

    fn run_call(
        &mut self,
        number: usize,
        transcript: Transcript<fs::File>,
        mut command: Command,
        input: Option<&str>,
        keep: bool,
        within: Option<Duration>,
    ) -> Result<Vec<u8>, String> {
        let Transcript { stdout: (stdout, out), stderr: (stderr, err) } = transcript;
        command
            .current_dir(&self.store)
            .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(out)
            .stderr(err);
        let mut request_wait = within.map(|within| Deadline::arm("the rung's forge request", within));
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        self.deadline().watch_process(child.id());
        if let Some(deadline) = &mut request_wait {
            deadline.watch_process(child.id());
        }
        if let Some(input) = input {
            child
                .stdin
                .take()
                .ok_or("command stdin missing")?
                .write_all(input.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if let Err(message) = self.check_wait() {
                super::super::parent::end_child(&mut child);
                return Err(format!("{message}; call {number}"));
            }
            if let Some(deadline) = &request_wait
                && deadline.passed()
            {
                match deadline.expired() {
                    Expired::Starved(starved) => {
                        let message = format!("forge request exceeded its bound: {}", starved.figures());
                        self.starved = Some(starved);
                        super::super::parent::end_child(&mut child);
                        return Err(message);
                    }
                    Expired::Failed(figures) => {
                        super::super::parent::end_child(&mut child);
                        return Err(format!("forge request exceeded its bound: {figures}"));
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        require(
            status.success(),
            &format!("phase {}: call {number} failed ({status}); see {}", self.phase, stderr.display()),
        )?;
        let bytes = fs::read(&stdout).map_err(|e| e.to_string())?;
        if !keep {
            fs::remove_file(stdout).map_err(|e| e.to_string())?;
        }
        Ok(bytes)
    }

    fn ctl(&mut self, args: &[&str], input: Option<&str>) -> Result<Value, String> {
        let mut command = if args.starts_with(&["account", "import"]) {
            let mut locked = Command::new("/bin/sh");
            let import = format!(
                "exec 9>\"$1\" || exit 1\n{LOCK_FD_9} || exit 1\ntest ! -e \"$2\" || exit 1\nshift 2\nexec \"$@\""
            );
            locked
                .args(["-c", &import, "rung-import"])
                .arg(self.store.join("login.lock"))
                .arg(self.store.join("returning"))
                .arg(&self.binary);
            locked
        } else {
            Command::new(&self.binary)
        };
        command.arg("ctl").arg(&self.client).args(args).arg("--json");
        command.env_clear().env("PATH", "/usr/bin:/bin");
        let bytes = self.execute(command, input, true)?;
        ctl_json(&bytes).map_err(|error| format!("phase {}: {error}", self.phase))
    }

    fn forge(&mut self, route: &str) -> Result<Vec<u8>, String> {
        let mut command = Command::new("curl");
        command.args(["-fsS", "-H"]).arg(format!("@{}", self.header.display()));
        command.arg(format!("{}/repos/playground/widget/{route}", self.api));
        self.execute_within(command, None, true, Some(Duration::from_secs(10)))
    }

    fn forge_json(&mut self, route: &str) -> Result<Value, String> {
        serde_json::from_slice(&self.forge(route)?).map_err(|e| format!("forge JSON: {e}"))
    }

    fn phase(&mut self, phase: &'static str) {
        self.phase = phase;
        let index = self.phases.iter().position(|(name, _)| *name == phase).expect("a named rung phase");
        self.until = self.phases[index].1.until();
        self.active = Some(index);
    }

    fn met(&self, bound: u64) {
        super::super::live::notice(&format!(
            "PHASE {} {}ms <= {}ms from request",
            self.phase,
            now().saturating_since(self.requested.expect("request clock")).as_nanos() / 1_000_000,
            bound
        ));
    }

    fn wait(&mut self, args: &[&str], condition: &str) -> Result<Value, String> {
        let left = self.until.saturating_since(now()).as_nanos() / 1_000_000;
        self.check_wait()?;
        // ctl accepts whole seconds. The child is still killed at this
        // driver's exact millisecond deadline, including its final fraction.
        let timeout = left.div_ceil(1_000).to_string();
        // Since already names the observed change. Combining it with
        // "until changed" requires another change after the first delivery,
        // which can miss the chat's last reply before the watch opens.
        let since = args.contains(&"--since");
        let mut args = args.to_vec();
        if condition != "changed" || !since {
            args.extend(["--until", condition]);
        }
        args.extend(["--timeout", timeout.as_str()]);
        let value = self.ctl(&args, None)?;
        waited(&args, value).map_err(|error| format!("phase {}: {condition}: {error}", self.phase))
    }

    /// Activity is a paged public read. Do not mistake a cut first page for the
    /// entire conversation, or a call argument for a message received by work.
    fn activity(&mut self, task: u64) -> Result<Vec<Value>, String> {
        let task = task.to_string();
        let mut cursor = None;
        let mut rows = Vec::new();
        for _ in 0..256 {
            let mut args = vec!["task", "activity", &task, "--limit", "32"];
            if let Some(before) = cursor.as_deref() {
                args.extend(["--before", before]);
            }
            let page = self.ctl(&args, None)?;
            for row in page["rows"].as_array().ok_or("activity rows missing")? {
                if let Some(turn) = row.pointer("/event/owner/decoded/turn/decoded") {
                    require(turn["cut"] == false, "whole committed activity turn")?;
                }
            }
            rows.extend(page["rows"].as_array().ok_or("activity rows missing")?.iter().cloned());
            cursor = page["before"]["value"].as_str().map(str::to_owned);
            if cursor.is_none() {
                require(page["cut"] == false, "activity cut without a cursor")?;
                rows.sort_by_key(|row| row["position"].as_u64());
                rows.dedup_by_key(|row| row["position"].as_u64());
                return Ok(rows);
            }
        }
        Err("activity exceeded 256 pages".into())
    }

    fn children(&mut self, parent: &str) -> Result<Vec<Value>, String> {
        let project = self.project.clone();
        let live = self.ctl(&["task", "list", "--project", &project, "--parent", parent], None)?;
        let ended = self.ctl(&["task", "list", "--project", &project, "--parent", parent, "--ended"], None)?;
        child_rows(&live, &ended)
    }

    fn conversation(&mut self, chat: &str) -> Result<String, String> {
        let mut cursor = None;
        let mut words = String::new();
        for _ in 0..256 {
            let mut args = vec!["chat", "show", chat];
            if let Some(before) = cursor.as_deref() {
                args.extend(["--before", before]);
            }
            let page = self.ctl(&args, None)?;
            words.push_str(&page.to_string());
            cursor = page["activity"]["before"]["value"].as_str().map(str::to_owned);
            if cursor.is_none() {
                return Ok(words);
            }
        }
        Err("chat conversation exceeded 256 pages".into())
    }

    fn unchanged(&mut self, chat: &str, baseline: &Value) -> Result<u64, String> {
        let tree = self.ctl(&["task", "show", chat, "--tree"], None)?;
        let tree = &tree["tree"];
        require(tree["more"] == false && tree["folded"] == json!([]), "whole waiting tree")?;
        let rows = tree["rows"].as_array().ok_or("tree rows missing")?;
        require(rows.len() == 1, "only the planning work, no change before the answer")?;
        require(rows[0]["chip"]["executor"].get("agent").is_some(), "planning delegate is an agent")?;
        let goal = rows[0]["chip"]["task"].as_u64().ok_or("planning task number missing")?;
        for child in self.children(&goal.to_string())? {
            let task = child["chip"]["task"].as_u64().ok_or("planning child task number missing")?.to_string();
            let read = self.ctl(&["task", "show", &task], None)?;
            let read = read.get("task").unwrap_or(&read);
            let authority = &read["authority"];
            // The task page may cut the brief before the advisor's role line.
            require(
                child["chip"]["executor"].get("agent").is_some()
                    && read["priority"].is_null()
                    && authority["cut"] == false
                    && authority["authority"]["tools"] == jig_domain_core::ToolFamilies::JIG.read.0
                    && authority["authority"]["delegation"] == json!({"kinds": [], "tasks": 0, "depth": 0}),
                "planning work has made no change, ended or live; only advisors before the answer",
            )?;
        }
        require(self.forge_json("branches")? == *baseline, "no pushed branch or changed main before the answer")?;
        require(self.forge_json("pulls?state=all")? == json!([]), "no pull request before the answer")?;
        let activity = json!({"rows": self.activity(goal)?});
        let mut tools = super::successful_tools(&activity);
        let chat_activity = json!({"rows": self.activity(chat.parse().map_err(|_| "chat number")?)?});
        tools.extend(super::successful_tools(&chat_activity));
        require(
            !tools.iter().any(|tool| ["change", "write", "edit"].contains(&tool.as_str())),
            "no edit or change before approval",
        )?;
        Ok(goal)
    }
}

fn give_back(driver: &mut Driver) -> Result<(), String> {
    driver.arm("the held record's return", Duration::from_secs(60));
    let output = driver.execute(Command::new(driver.store.join("give-back")), None, true)?;
    super::super::live::notice(&String::from_utf8_lossy(&output));
    Ok(())
}

/// The launcher's return with no box up: a box over the store for this alone, a new client signed
/// in by the box's link, then give-back. The box dies with this process and stops on
/// every ordinary return; the store stays.
pub fn return_on_a_box_of_its_own(
    binary: &Path,
    store: &Path,
    paths: &[PathBuf; 3],
    account: &Path,
) -> Result<(), String> {
    let client = store.join("return-client");
    fs::create_dir_all(&client).map_err(|error| format!("return client: {error}"))?;
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("private return client: {error}"))?;
    let mut driver = Driver::owned(store, &client);
    driver.binary = binary.into();
    let started = now();
    driver.arm("the held record's new box", Duration::from_secs(240));
    let mut guard = super::super::parent::Guard::new(start_box(binary, store, paths, true));
    let boxed = guard.child.as_mut().expect("return box");
    driver.deadline().watch_process(boxed.id());
    let returned = (|| {
        let link = return_link(&mut driver, boxed)?;
        super::super::live::notice(&format!(
            "RETURN box answered in {}ms",
            now().saturating_since(started).as_nanos() / 1_000_000
        ));
        driver.ctl(&["sign-in", "-"], Some(&link))?;
        super::super::held_login::recovery_hook(binary, store, &client, account)?;
        give_back(&mut driver)
    })();
    stop_return_box(boxed);
    returned
}

fn return_link(driver: &mut Driver, boxed: &mut Child) -> Result<String, String> {
    loop {
        if let Some(status) = boxed.try_wait().map_err(|error| error.to_string())? {
            return Err(format!("return box exited ({status}); see {}", driver.store.join("box.err").display()));
        }
        driver.check_wait()?;
        let output = fs::read_to_string(driver.store.join("box.out")).map_err(|error| error.to_string())?;
        if let Some(link) = output.lines().find(|line| line.starts_with("http://") && line.contains("secret=")) {
            return Ok(link.to_owned());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn stop_return_box(boxed: &mut Child) {
    super::super::parent::signal(boxed, "TERM");
    let mut deadline = Deadline::arm("the held record's box stop", Duration::from_secs(30));
    deadline.watch_process(boxed.id());
    let stopped = loop {
        match boxed.try_wait() {
            Ok(Some(status)) => break require(status.success(), &format!("return box stopped with {status}")),
            Ok(None) => {}
            Err(error) => break Err(format!("return box stop: {error}")),
        }
        if deadline.passed() {
            break Err("return box needed killing after its stop bound".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    super::super::parent::end_child(boxed);
    if let Err(message) = stopped {
        super::super::live::notice(&message);
    }
}

fn stop_box(boxed: &mut Child) -> Result<(), String> {
    super::super::parent::signal(boxed, "TERM");
    let status = boxed.wait().map_err(|e| e.to_string())?;
    require(status.success(), "box drained successfully")
}

/// Run only by proof-live with a held record and a fresh private deployment.
pub fn green_light() {
    let account = super::live_account().expect("rung-b0 requires TEMPER_TEST_LIVE_ACCOUNT");
    let (store, kept) = super::live_store(true);
    assert!(!kept, "a rung starts with a fresh box");
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    deployment(&paths);
    super::super::live::notice(&format!("TAG rung-b0-{}", super::live_tag()));
    run_walk(&store, &paths, &account, None).unwrap_or_else(|message| panic!("{message}"));
}

// The no-task sitting commits the prospective copies before the next start
// reads them. A restore on its own would replace them with the store's defaults.
fn bootstrap_settings(paths: &[PathBuf; 3]) {
    let steps = crate::provider_setup::steps(paths[0].join("rung-providers.json"), &paths[1]);
    crate::limit_settings::install_paths_with(paths, steps.into());
}

fn box_command(binary: &Path, store: &Path, paths: &[PathBuf; 3]) -> skein_world::end_to_end::Command {
    skein_world::end_to_end::Command {
        program: binary.into(),
        arguments: super::deployment_flags(paths)
            .into_iter()
            .chain(["--echo".into(), "steps".into()])
            .map(Into::into)
            .collect(),
        directory: store.into(),
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
    }
}

fn start_box(binary: &Path, store: &Path, paths: &[PathBuf; 3], dies_with_test: bool) -> Child {
    let command = box_command(binary, store, paths);
    let command = if dies_with_test { super::super::boxed::dying_with_test(command) } else { command };
    let mut spawn = if dies_with_test {
        Command::new(&command.program)
    } else {
        let mut spawn = Command::new("setsid");
        spawn.arg(&command.program);
        spawn
    };
    spawn
        .args(command.arguments)
        .env_clear()
        .envs(command.environment)
        .current_dir(command.directory)
        .stdin(Stdio::null())
        .stdout(fs::File::create(store.join("box.out")).expect("box output"))
        .stderr(fs::File::create(store.join("box.err")).expect("box errors"))
        .spawn()
        .expect("rung box")
}

/// Both faces import, walk, return the account, then stop through these acts.
fn run_walk(store: &Path, paths: &[PathBuf; 3], account: &Path, probe: Option<&ImportProbe>) -> Result<(), String> {
    let _ = super::cache_forgejo_binaries(&paths[2]);
    let client = store.join("client");
    fs::create_dir(&client).expect("fresh client");
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client");
    let mut driver = Driver::owned(store, &client);
    super::super::held_login::return_hook(&driver.binary, store, &driver.client, account)?;
    bootstrap_settings(paths);
    let boxed = match probe {
        Some(_) => WalkBox::Twin(Box::new(start_twin_box(&driver.binary, store, paths))),
        None => WalkBox::Live(start_box(&driver.binary, store, paths, false)),
    };
    let result = walk(&mut driver, paths, account, probe);
    // Ordinary endings return before stopping. The launcher covers a test that ends first.
    let starved = driver.starved.take();
    driver.arm("the rung's held login return", Duration::from_secs(60));
    let returned = driver.execute(Command::new(store.join("give-back")), None, true);
    if let Err(message) = returned {
        super::super::live::notice(&format!("LOGIN NOT BACK: {message}; box retained for recovery"));
        panic!("LOGIN NOT BACK: {message}; box retained for recovery");
    }
    super::super::live::notice("MUST login at its original path; stat regular, nonempty, 0600");
    match boxed {
        WalkBox::Twin(binary) => stop_twin_box(binary).expect("twin box drained after login return"),
        WalkBox::Live(mut child) => {
            stop_box(&mut child).expect("box stopped after login return");
            if starved.is_some() {
                super::super::parent::end_child(&mut child);
            }
        }
    }
    if let Some(starved) = starved {
        starved.pass();
    }
    result
}

enum WalkBox {
    Twin(Box<Binary>),
    Live(Child),
}

fn start_twin_box(program: &Path, store: &Path, paths: &[PathBuf; 3]) -> Binary {
    let command = box_command(program, store, paths);
    fs::File::create(store.join("box.out")).expect("box output before the driver's first read");
    // Keep the driver's file transcript while the tree observer is not being
    // driven. The shell execs the box in place, so it is the observed leader.
    let command = skein_world::end_to_end::Command {
        program: "/bin/sh".into(),
        arguments: ["-c".into(), "exec \"$@\" </dev/null >box.out 2>box.err".into(), "rung-box".into()]
            .into_iter()
            .chain([command.program.into()])
            .chain(command.arguments)
            .collect(),
        ..command
    };
    Binary::start_with_tree(
        super::super::boxed::dying_with_test(command),
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("twin box starts")
}

#[derive(Default)]
struct TwinStop {
    killing: bool,
    settled: bool,
}

impl Referee<Process> for TwinStop {
    fn act(&mut self, _at: Time, procs: &mut [Process]) {
        if let [Process::Binary(binary)] = procs
            && !self.killing
            && !binary.is_empty()
            && binary.tree_status().live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty())
        {
            binary.signal_tree(Signal::Kill);
            self.killing = true;
        }
    }

    fn watch(&mut self, _at: Time, procs: &[Process]) -> Result<(), String> {
        let [Process::Binary(binary)] = procs else { return Err("twin box observer missing".into()) };
        self.settled = binary.is_empty();
        Ok(())
    }

    fn passed(&self) -> bool {
        self.settled
    }
}

fn stop_twin_box(mut binary: Box<Binary>) -> Result<(), String> {
    // In the subreaper fallback the keeper surveys this process's children.
    // Drive it only now, with every synchronous ctl/curl child already reaped.
    binary.signal(Signal::Terminate);
    let mut world = real::World::new(TwinStop::default());
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(binary));
    let outcome = world.try_run(&Clock::new(), Duration::from_secs(30))?;
    let [Process::Binary(binary)] = outcome.procs.as_slice() else { return Err("twin box observer missing".into()) };
    let exit = binary.exit_status();
    let left = binary.tree_status().live_at_leader_exit.as_deref();
    require(
        exit == Some(Exit::Code(0)) && left == Some(&[]),
        &format!("twin box exit {exit:?}, live at exit: {left:?}; see box.err"),
    )
}

fn twin_peer() -> super::super::llm::Listening {
    super::super::sign_in::plan_document(&serde_json::to_vec(&twin_document()).expect("adapted twin JSON"))
}

fn twin_document() -> Value {
    let mut document: Value =
        serde_json::from_slice(include_bytes!("../../../../tests/real-loop/tests/fixtures/b0/llm.json"))
            .expect("checked-in B0 twin");
    let scripts = document["cued"].as_array_mut().expect("twin scripts");
    let mut review = None;
    for (index, script) in scripts.iter().enumerate() {
        if script["cue"] == "Review this change at the named head" {
            assert!(review.is_none(), "B0 twin has only one review script");
            review = Some(index);
        }
    }
    let review = review.expect("B0 twin has a review script");
    scripts[review]["turns"] = json!([{
        "lines":[{"call":"finish","arguments":{
            "form":"verdict","label":"1","text":"Approved for the driver replay.","fields":{},"items":[]
        }}],
        "finish":"tool_calls","tokens":20
    }]);
    for script in scripts.iter_mut() {
        if script["cue"] == "You are a goal: the work your person asked for" {
            goal_inputs(script);
        }
    }
    // All other call arguments stay recorded. Identity-bearing prose in
    // words/text/content is deliberately untouched and is not an oracle.
    for (task, script) in scripts.iter().enumerate() {
        for (turn, value) in script["turns"].as_array().expect("turns").iter().enumerate() {
            for line in value["lines"].as_array().expect("lines") {
                if let Some(call) = line["call"].as_str() {
                    let args = line["arguments"].as_object().expect("call arguments");
                    let identity = (args.contains_key("head") || args.contains_key("commit"))
                        || args.contains_key("merge")
                        || (call == "read_issues" && args.contains_key("number"));
                    assert!(!identity, "unbound recorded identity: task {task}, turn {turn}, call {call}");
                }
            }
        }
    }
    document
}

// A forge notification may wake the goal before its child returns. Select
// the recorded turns from fresh inputs so the child's result releases the report.
fn goal_inputs(script: &mut Value) {
    script["inputs"] = json!([
        {"cue":"Landed: pull request #", "turn":7},
        {"cue":ANSWER, "turn":5},
        {"cue":"The repository contains a minimal Rust binary:", "turn":3},
        {"cue":"this work is to be agreed", "turn":1},
        {"cue":"Delegated tasks:", "turn":6},
        {"cue":"Message ", "turn":6},
        {"cue":"asks (question", "turn":6},
        {"cue":"forge landing", "turn":6},
        {"cue":"A new activation starts", "turn":6},
        {"cue":"", "turn":0}
    ]);
}

/// Create only a synthetic account, using native connect and export. Its
/// provider registration follows the file to the walk's fresh deployment.
fn fake_account(root: &Path, paths: &[PathBuf; 3], file: &Path) {
    use super::super::{
        boxed,
        ctl::{Call, Step},
        sign_in,
    };
    let donor = root.join("donor");
    fs::create_dir(&donor).expect("fake account donor");
    let donor_paths = ["config", "data", "cache"].map(|name| donor.join(name));
    deployment(&donor_paths);
    // The donor's B0 agent figures need the same limits as the walk's box.
    bootstrap_settings(&donor_paths);
    let issuer = sign_in::patch_configuration(&donor_paths[0]);
    let _ = super::cache_forgejo_binaries(&donor_paths[2]);
    let outcome = boxed::run_ctl_with_account(
        &donor,
        boxed::echo_steps(super::deployment_flags(&donor_paths)),
        &donor,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["account", "connect", "1", "--key", "donor-account"])),
            Step::Run(Call::new(&["account", "list", "--until", "usable", "--timeout", "10s"])),
            Step::Run(Call::new(&["account", "export", "1", file.to_str().expect("fake account path")])),
        ],
        boxed::BOX_DEADLINE,
        || (),
        super::fake_face(issuer, temper_real_loop_world::llm::cued(Box::new([]))),
    );
    for call in &outcome.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "fake account setup: {}", String::from_utf8_lossy(&call.stderr));
    }
    assert_eq!(outcome.calls.len(), 4, "native account exported");
    assert_eq!(outcome.exit, Some(Exit::Code(0)));
    assert!(outcome.checked.is_some() && outcome.left.as_ref().is_none_or(Vec::is_empty), "donor drained");
    let registration =
        temper_agent_fixtures::providers::read(&donor_paths[1])["providers"][0]["sources"]["sign_in"].clone();
    let mut destination = temper_agent_fixtures::providers::read(&paths[1]);
    destination["providers"][0]["sources"]["sign_in"] = registration;
    temper_agent_fixtures::providers::write(&paths[1], &destination);
}

/// One read at the import boundary. Dropping the sender ends the peers.
type ImportProbe = mpsc::Sender<mpsc::Sender<usize>>;

struct TwinPeers {
    finding: Option<String>,
    probe: mpsc::Receiver<mpsc::Sender<usize>>,
    ended: bool,
    deadline: Time,
    wake: Time,
}

impl Referee<Process> for TwinPeers {
    fn act(&mut self, at: Time, procs: &mut [Process]) {
        self.wake = at.saturating_add(Duration::from_millis(10));
        if self.ended {
            return;
        }
        let [Process::Issuer(issuer), Process::Peer(peer)] = procs else {
            self.finding = Some("twin peers".into());
            return;
        };
        match self.probe.try_recv() {
            Ok(reply) => {
                let queries = peer
                    .observations()
                    .iter()
                    .filter(|event| matches!(event, skein_fake_peers::llm::Observation::Query { .. }))
                    .count();
                let _ = reply.send(queries);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                issuer.shutdown();
                peer.shutdown();
                self.ended = true;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
    fn watch(&mut self, _at: Time, _procs: &[Process]) -> Result<(), String> {
        match &self.finding {
            Some(why) => Err(why.clone()),
            None => Ok(()),
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.deadline.min(self.wake))
    }
    fn overdue(&self, at: Time) -> Option<String> {
        (at >= self.deadline).then(|| "B0 twin peers did not settle".into())
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.ended
    }
}

/// Real Forgejo, runner, ctl and clock, with only the account/model replaced.
#[expect(
    clippy::disallowed_methods,
    reason = "the fake peers' world serves the synchronous ctl driver on a scoped thread"
)]
pub fn offline_walk(root: &Path) {
    use super::super::sign_in;
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
    deployment(&paths);
    drop(sign_in::patch_configuration(&paths[0]));
    let account = root.join("fake-account.move");
    fake_account(root, &paths, &account);
    let issuer = sign_in::restart_issued_issuer(&paths[0]);
    let peer = twin_peer();
    peer.configure(&paths[0]);
    // As in account_move: one independent world must serve the fake while
    // the synchronous bounded ctl/curl driver waits for its child commands.
    std::thread::scope(|scope| {
        let (probe, receiver) = mpsc::channel();
        let (ready, listening) = mpsc::channel();
        let peers = scope.spawn(move || {
            let clock = Clock::new();
            // Setup plus the unchanged whole-walk bound; not a phase increase.
            let within = Duration::from_secs(700);
            let mut world = skein_world::real::World::new(TwinPeers {
                finding: None,
                probe: receiver,
                ended: false,
                deadline: clock.now().now.saturating_add(within),
                wake: clock.now().now,
            });
            world.spawn(|| Process::Issuer(Box::new(issuer)));
            let index = peer.spawn(&mut world);
            ready.send(()).expect("twin peer world ready");
            let outcome = world.run(&clock, within);
            assert!(outcome.procs.iter().all(Host::is_empty), "twin peer io drained");
            super::peer_seen(&outcome.procs, index)
        });
        listening.recv().expect("twin peers constructed");
        let walked = run_walk(root, &paths, &account, Some(&probe));
        drop(probe);
        let seen = peers.join().expect("independent twin world");
        walked.unwrap_or_else(|message| panic!("{message}"));
        assert!(!seen.requests.is_empty(), "twin served the driver");
        assert!(seen.answers.iter().all(Result::is_ok), "every twin answer served");
        assert!(seen.schemes.iter().all(|scheme| *scheme == Some(skein_llm::Scheme::Bearer)), "plan bearer transport");
    });
}

fn setup(
    driver: &mut Driver,
    paths: &[PathBuf; 3],
    account: &Path,
    probe: Option<&ImportProbe>,
) -> Result<Value, String> {
    let link = loop {
        let output = fs::read_to_string(driver.store.join("box.out")).map_err(|e| e.to_string())?;
        if let Some(link) = output.lines().find(|line| line.starts_with("http://") && line.contains("secret=")) {
            break link.to_owned();
        }
        driver.check_wait().map_err(|message| format!("setup: box sign-in link timeout; see box.err; {message}"))?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    driver.ctl(&["sign-in", "-"], Some(&link))?;
    setup_tools(driver)?;
    let baseline = setup_observer(driver, paths)?;
    if let Some(probe) = probe {
        check_before_import(driver, probe)?;
    }
    driver.ctl(&["account", "import", account.to_str().ok_or("held path")?, "--key", "connect-plan"], None)?;
    let accounts = driver.wait(&["account", "list"], "usable")?;
    require(
        accounts["accounts"]["party"]
            .as_array()
            .is_some_and(|rows| rows.len() == 1 && rows[0]["provider"] == 1 && rows[0]["plan"] == true),
        "fresh box holds one ChatGPT plan for the return hook",
    )?;
    Ok(baseline)
}

/// All ordinary ctl setup before importing the held account, shared by both faces.
fn setup_tools(driver: &mut Driver) -> Result<(), String> {
    configure_model(driver)?;
    driver.ctl(&["forge", "setup", "--playground", "--key", "rung-forge"], None)?;
    driver.wait(&["forge", "list"], "settled")?;
    let forges = driver.ctl(&["forge", "list"], None)?;
    let first = super::super::first_repository::owned_row(&forges);
    driver.forge = first["number"].as_u64().ok_or("forge number")?;
    require(
        first["first"]["org"] == "playground" && first["first"]["repository"] == "widget",
        "playground/widget adopted",
    )?;
    let me = driver.ctl(&["me"], None)?;
    let project = super::super::first_repository::project_number(&me, "widget")?.to_string();
    driver.project.clone_from(&project);
    fund_project(driver, &project)?;
    let charter = driver.ctl(&["settings", "show", "charter", "1"], None)?;
    require(charter["value"]["model"]["name"] == "gpt-6.1-sol", "QA charter uses sol")?;
    Ok(())
}

fn fund_project(driver: &mut Driver, project: &str) -> Result<(), String> {
    let current = driver.ctl(&["project", "show", project], None)?;
    let period = current["policy"]["period_spend"].as_u64().ok_or("project period spend")?.max(1_005_888);
    let owner = owner_role(&current)?["period_spend"].as_u64().ok_or("owner period spend")?.max(928_512);
    // Funding raises run project first, then pool (client/ctl/tools-acts.md, 1).
    // Keep the current policy when it already funds the rung's work.
    driver.ctl(
        &["project", "fund", project, &format!("{}.{:02}", period / 100, period % 100), "--key", "rung-project-funds"],
        None,
    )?;
    driver.ctl(
        &[
            "project",
            "fund",
            project,
            &format!("{}.{:02}", owner / 100, owner % 100),
            "--role",
            "0",
            "--key",
            "rung-role-funds",
        ],
        None,
    )?;
    Ok(())
}

fn owner_role(project: &Value) -> Result<&Value, String> {
    require(project["cut"] == false && project["policy"]["cut"] == false, "whole project policy read")?;
    project["policy"]["roles"]
        .as_array()
        .and_then(|roles| roles.iter().find(|role| role["number"] == 0))
        .ok_or_else(|| "project owner role missing".into())
}

/// Change the stock row after startup, through the signed-in owner's settings act.
fn configure_model(driver: &mut Driver) -> Result<(), String> {
    let inventory = driver.ctl(&["settings", "show"], None)?;
    let revision = inventory["newest"].as_u64().ok_or("settings revision")?.to_string();
    let setting = driver.ctl(&["settings", "show", "model", "4"], None)?;
    let mut value = setting["value"].clone();
    require(value["model"]["name"] == "gpt-6.1-sol", "stock row 4 is sol")?;
    value["model"]["window"] = 1_048_576.into();
    value["model"]["output"] = 32_000.into();
    let mut items = vec![json!({"setting":setting["setting"], "set":value})];
    for number in ["1", "2"] {
        let charter = driver.ctl(&["settings", "show", "charter", number], None)?;
        let mut charter_value = charter["value"].clone();
        super::super::provider_setup::use_provider(&mut charter_value, 1);
        charter_value["model"] = value["model"].clone();
        items.push(json!({"setting":charter["setting"], "set":charter_value}));
    }
    let providers = temper_agent_fixtures::providers::read(&driver.store.join("data"));
    items.extend(super::super::provider_setup::items(&providers));
    let file = driver.store.join("model-settings.json");
    write_json(&file, &json!({"items":items}));
    driver.ctl(
        &[
            "settings",
            "apply",
            file.to_str().ok_or("settings FILE path")?,
            "--against",
            &revision,
            "--key",
            "rung-model",
        ],
        None,
    )?;
    Ok(())
}

fn setup_observer(driver: &mut Driver, paths: &[PathBuf; 3]) -> Result<Value, String> {
    let log = super::super::boxed::box_log(&paths[1]);
    require(
        log.lines().any(|line| line.ends_with(&format!("Forge {} (Forgejo): started", driver.forge))),
        "box reported the forge started",
    )?;
    let log = std::fs::read_to_string(paths[1].join(format!("box/logs/forgejo-{}.log", driver.forge)))
        .map_err(|e| e.to_string())?;
    driver.api = log
        .lines()
        .find_map(|line| line.split_once("Forgejo: api ").map(|(_, api)| api.trim().to_owned()))
        .ok_or("Forgejo step log has no person's forge API")?;
    require(log.lines().any(|line| line.contains("Forgejo: person ")), "Forgejo step logged the person's login")?;
    driver.header = paths[1].join(format!("forge-{}/person_read.header", driver.forge));
    super::super::live::notice(
        "SETUP sol high output=32000; engine memory=derived-before-start; project and owner funding keep the current policy above the rung's minimum",
    );
    let memory = serde_json::Value::from(temper_service::limits::stock().root.transcript_memory);
    super::super::live::notice(&format!("SETUP engine memory={memory}"));
    let baseline = driver.forge_json("branches")?;
    require(baseline.as_array().is_some_and(|b| b.len() == 1 && b[0]["name"] == "main"), "fresh main alone")?;
    driver.forge("raw/src/main.rs?ref=main")?;
    Ok(baseline)
}

fn check_before_import(driver: &mut Driver, probe: &ImportProbe) -> Result<(), String> {
    let accounts = driver.ctl(&["account", "list"], None)?;
    require(accounts["accounts"]["party"] == json!([]), "no party account before the twin's import")?;
    let project = driver.project.clone();
    check_funding(&driver.ctl(&["project", "show", &project], None)?)?;

    driver.check_wait()?;
    let (reply, count) = mpsc::channel();
    probe.send(reply).map_err(|_| "twin peers stopped before the import check")?;
    let remaining = std::time::Duration::from_nanos(driver.until.saturating_since(now()).as_nanos());
    let queries = match count.recv_timeout(remaining) {
        Ok(queries) => queries,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err("twin peers did not acknowledge the import check".into());
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            driver.check_wait()?;
            return Err("twin import check timed out before the peer acknowledged it".into());
        }
    };
    require(queries == 0, &format!("the twin's model saw {queries} queries before import"))
}

fn check_funding(project: &Value) -> Result<(), String> {
    require(
        project["policy"]["period_spend"].as_u64().is_some_and(|spend| spend >= 1_005_888)
            && project["current_period"]["numbers"]["budget"] == project["policy"]["period_spend"],
        "project policy and open period funded",
    )?;
    let owner = owner_role(project)?;
    require(
        owner["period_spend"].as_u64().is_some_and(|spend| spend >= 928_512)
            && owner["authority"]["authority"]["spend"].as_u64().is_some_and(|spend| spend >= 232_128)
            && owner["authority"]["cut"] == false
            && project["pools"].as_array().is_some_and(|pools| {
                pools.iter().all(|pool| {
                    pool["source"]["pool"]["person"] != 1 || pool["numbers"]["budget"] == owner["period_spend"]
                })
            }),
        "role 0 authority and pool funded",
    )?;
    require(project["cut"] == false && project["configured_pools_cut"] == false, "whole project funding read")?;
    Ok(())
}

fn walk(driver: &mut Driver, paths: &[PathBuf; 3], account: &Path, probe: Option<&ImportProbe>) -> Result<(), String> {
    let baseline = setup(driver, paths, account, probe)?;
    if probe.is_some() {
        // Parking is required, but its stock ten-second threshold is not B0's
        // subject. The full two-minute hold below remains the same on both faces.
        let inventory = driver.ctl(&["settings", "show"], None)?;
        let revision = inventory["newest"].as_u64().ok_or("settings revision")?.to_string();
        let mut items = Vec::new();
        for number in ["1", "2"] {
            let charter = driver.ctl(&["settings", "show", "charter", number], None)?;
            let mut value = charter["value"].clone();
            value["waiting"] = 1_000_000.into();
            items.push(json!({"setting":charter["setting"],"set":value}));
        }
        let file = driver.store.join("offline-parking.json");
        write_json(&file, &json!({"items":items}));
        driver.ctl(
            &[
                "settings",
                "apply",
                file.to_str().ok_or("settings FILE path")?,
                "--against",
                &revision,
                "--key",
                "offline-parking",
            ],
            None,
        )?;
    }
    let project = driver.project.clone();
    driver.request();
    driver.phase("inbox question");
    let created = driver.ctl(
        &[
            "chat",
            "start",
            "--project",
            &project,
            "--model",
            "gpt-6.1-sol",
            "--effort",
            "high",
            "--key",
            "rung-b0",
            REQUEST,
        ],
        None,
    )?;
    let chat = created_chat(&created)?.to_string();
    driver.wait(&["inbox"], "waiting")?;
    driver.met(40_480);
    driver.phase("person's answer sent");
    let inbox = driver.ctl(&["inbox", "--kind", "questions"], None)?;
    let rows = inbox["needs"]["rows"].as_array().ok_or("inbox questions missing")?;
    require(rows.len() == 1, "one question in the person's inbox")?;
    require(
        rows[0]["task"]["live"]["task"].as_u64().map(|n| n.to_string()) == Some(chat.clone()),
        "question is on the person's chat",
    )?;
    let question = rows[0]["object"]["question"]["message"].as_u64().ok_or("inbox question number")?.to_string();
    driver.phase("person's answer sent");
    let goal = driver.unchanged(&chat, &baseline)?;
    let question_rows = driver.activity(chat.parse().map_err(|_| "chat number")?)?;
    let asked = plan_question(&question_rows, goal, question.parse().map_err(|_| "question number")?)?;
    require(
        asked.to_lowercase().contains("plan") && asked.contains('?'),
        "the question tells its plan and asks for go-ahead",
    )?;
    super::super::live::notice(&format!("QUESTION {asked}"));
    super::super::live::notice("MUST work's plan question reached the person's inbox; no change, branch or pull");
    std::thread::sleep(std::time::Duration::from_mins(2));
    // Advisors may end during the hold; unchanged rejects every non-advisor child.
    require(driver.unchanged(&chat, &baseline)? == goal, "same waiting work after two minutes")?;
    super::super::live::notice("MUST after the two-minute hold: no change, no branch, no pull, main unchanged");
    driver.ctl(&["task", "answer", &chat, &question, ANSWER, "--key", "rung-answer"], None)?;
    driver.met(186_660);
    follow(driver, &chat, goal)
}

fn follow(driver: &mut Driver, chat: &str, goal: u64) -> Result<(), String> {
    let goal_text = goal.to_string();
    driver.phase("answer reached goal");
    let answer_at = loop {
        let read = driver.ctl(&["task", "show", &goal_text], None)?;
        let rows = driver.activity(goal)?;
        let answered = approval_at(&rows, chat.parse().map_err(|_| "chat number")?);
        if let Some(at) = answered {
            break at;
        }
        let digest = task_digest(&read)?;
        driver.wait(&["task", "show", &goal_text, "--since", digest], "changed")?;
    };
    driver.met(190_910);
    driver.phase("change delegated");
    let change = loop {
        let read = driver.ctl(&["task", "show", &goal_text, "--tree"], None)?;
        let children = driver.children(&goal_text)?;
        if let Some(row) = children.iter().find(|r| r["chip"]["executor"].get("procedure").is_some()) {
            break row["chip"]["task"].as_u64().ok_or("change task number")?.to_string();
        }
        let digest = task_digest(&read)?;
        driver.wait(&["task", "show", &goal_text, "--since", digest], "changed")?;
    };
    driver.met(214_930);
    let activity = driver.activity(goal)?;
    let tools = super::successful_tools(&json!({"rows":activity}));
    require(
        !tools.contains("write") && !tools.contains("edit"),
        "the planning work never edited before making its change",
    )?;
    let delegated_at = activity
        .iter()
        .filter(|row| super::successful_tools(&json!({"rows":[row]})).contains("change"))
        .filter_map(|row| row["at"].as_u64())
        .min()
        .ok_or("work's committed change call missing")?;
    require(answer_at <= delegated_at, "work received the answer before calling change")?;
    super::super::live::notice("MUST exact answer reached the planning work before change delegation");
    first_edit(driver, &change, answer_at)?;
    landing(driver, chat, &goal_text, &change)
}

fn first_edit(driver: &mut Driver, change: &str, answer_at: u64) -> Result<(), String> {
    driver.phase("first code write");
    loop {
        let tree = driver.ctl(&["task", "show", change, "--tree"], None)?;
        for row in driver.children(change)? {
            let task = row["chip"]["task"].as_u64().ok_or("producer task number")?;
            for event in driver.activity(task)? {
                let tools = super::successful_tools(&json!({"rows":[event]}));
                if tools.contains("write") || tools.contains("edit") {
                    require(
                        event["at"].as_u64().is_some_and(|at| answer_at <= at),
                        "answer before the first code write",
                    )?;
                    driver.met(257_930);
                    return Ok(());
                }
            }
        }
        let digest = tree["tree"]["digest"].as_str().ok_or("change tree digest")?;
        driver.wait(&["task", "tree", change, "--since", digest], "changed")?;
    }
}

fn landing(driver: &mut Driver, chat: &str, goal_text: &str, change: &str) -> Result<(), String> {
    driver.phase("pull request and head CI");
    driver.wait(&["change", "show", change], "pull")?;
    let panel = driver.wait(&["change", "show", change], "ci:passed")?;
    let number = panel["pull"]["number"].as_u64().ok_or("pull number")?;
    let pull = driver.forge_json(&format!("pulls/{number}"))?;
    require(panel["head"] == pull["head"]["sha"], "ctl and forge agree on PR head")?;
    let head = pull["head"]["sha"].as_str().ok_or("forge pull head")?;
    let status = driver.forge_json(&format!("commits/{head}/status"))?;
    require(status["state"] == "success", "forge CI green at the exact PR head")?;
    let runs = driver.forge_json(&format!("actions/runs?head_sha={head}"))?;
    let run = runs["workflow_runs"]
        .as_array()
        .ok_or("Forgejo workflow runs missing")?
        .iter()
        .find(|r| r["commit_sha"] == head && r["status"] == "success")
        .and_then(|r| r["id"].as_u64())
        .ok_or("box runner has not completed CI at the pull head")?;
    let jobs = driver.forge_json(&format!("actions/runs/{run}/jobs"))?;
    let job = jobs
        .as_array()
        .ok_or("Forgejo jobs missing")?
        .iter()
        .find(|job| job["status"] == "success")
        .and_then(|job| job["id"].as_u64())
        .ok_or("no successful runner job at pull head")?;
    let log = driver.forge(&format!("actions/jobs/{job}/logs"))?;
    require(String::from_utf8_lossy(&log).contains("Job succeeded"), "the box runner's job log agrees with green CI")?;
    driver.met(286_410);
    driver.phase("agent review approved");
    let mut panel = panel;
    while !panel["gates"]
        .as_array()
        .is_some_and(|gates| gates.iter().any(|g| g["kind"] == "review" && g["verdict"] == "approved"))
    {
        let digest = panel["digest"].as_str().ok_or("change digest")?;
        panel = driver.wait(&["change", "show", change, "--since", digest], "changed")?;
    }
    driver.met(363_470);
    driver.phase("landed");
    let panel = driver.wait(&["change", "show", change], "landed")?;
    let pull = driver.forge_json(&format!("pulls/{number}"))?;
    let main = driver.forge_json("branches/main")?;
    require(
        pull["merged"] == true
            && panel["merge"] == pull["merge_commit_sha"]
            && panel["merge"] == main["commit"]["id"]
            && pull["head"]["sha"] == head
            && panel["head"] == head,
        "ctl landing, forge merge and main agree",
    )?;
    driver.forge_json(&format!("pulls/{number}/reviews"))?;
    driver.met(364_410);
    driver.phase("main CI passed");
    let merge = pull["merge_commit_sha"].as_str().ok_or("merge hash")?;
    loop {
        if driver.forge_json(&format!("commits/{merge}/status"))?["state"] == "success" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    driver.met(367_410);
    driver.phase("chat's last word and whole");
    let source = driver.forge(&format!("raw/src/main.rs?ref={merge}"))?;
    require(
        source.windows(b"fruit,count".len()).any(|s| s == b"fruit,count"),
        "src/main.rs at landed main contains fruit,count",
    )?;
    super::super::live::notice(
        "MUST landed at forge main; CI green at PR head and main; agent review approved; src/main.rs holds fruit,count",
    );
    let work = driver.wait(&["task", "show", goal_text], "ended")?;
    let ended_at =
        work["task"]["chip"]["details"]["owner"]["details"]["ended_at"].as_u64().ok_or("work's report timestamp")?;
    loop {
        let snapshot = driver.ctl(&["task", "show", chat], None)?;
        let rows = driver.activity(chat.parse().map_err(|_| "chat number")?)?;
        let replies = super::replies_after(&json!({"rows": rows}), ended_at)?;
        if !replies.is_empty() {
            let words = driver.conversation(chat)?.to_lowercase();
            let own_words = json!(replies).to_string().to_lowercase();
            require(
                ["landed", "merged", "pr #", "pull request #"].iter().any(|word| own_words.contains(word)),
                "chat's own final reply reports the landing or pull",
            )?;
            require(
                words.contains("landed") || words.contains("merged"),
                "chat conversation carries the landed report",
            )?;
            break;
        }
        let digest = task_digest(&snapshot)?;
        driver.wait(&["task", "show", chat, "--since", digest], "changed")?;
    }
    driver.met(WHOLE);
    super::super::live::notice("MUST chat says the change landed");
    driver.arm("the rung's chat closing", Duration::from_secs(30));
    driver.ctl(&["chat", "close", chat, "--key", "rung-close"], None)?;
    Ok(())
}

pub mod tests {
    use super::{Driver, Duration, Time};
    use skein_world::{Expired, Figures, testing::deadline::at};
    use std::path::{Path, PathBuf};

    fn driver(start: Time) -> Driver {
        Driver::new_at(
            Path::new("transcript"),
            Path::new("client"),
            PathBuf::from("temper-box"),
            at("the rung's setup", Duration::from_secs(300), start),
        )
    }

    pub fn an_expired_rung_phase_starts_no_further_command() {
        let phase = "the expired rung phase";
        let mut driver = driver(Time::ZERO);
        driver.arm_deadline(phase, at(phase, Duration::ZERO, Time::ZERO));
        let mut opened = Vec::new();
        let mut started = false;
        let result = driver.execute_with(
            Time::ZERO,
            |deadline| {
                assert_eq!(deadline.until(), Time::ZERO);
                Expired::Failed(Figures {
                    what: phase,
                    patience: Duration::ZERO,
                    elapsed: Duration::ZERO,
                    thread_waited: None,
                    thread_cpu: None,
                    tree_waited: None,
                })
            },
            |path| {
                opened.push(path.to_path_buf());
                Ok(())
            },
            |_, _, _| {
                started = true;
                Ok(Vec::new())
            },
        );
        assert!(result.expect_err("the phase expired").starts_with("phase the expired rung phase exceeded its bound"));
        assert_eq!(driver.sequence, 0, "the expired phase did not start its command");
        assert!(opened.is_empty(), "not even call-0.out was opened: {opened:?}");
        assert!(!started);
    }

    pub fn rung_phases_keep_the_request_deadlines_when_selected_again() {
        let start = Time::from_nanos(17);
        let mut driver = driver(start);
        let mut armed = Vec::new();
        driver.request_at(start, |phase, within| {
            let deadline = at(phase, within, start);
            armed.push((phase, deadline.until()));
            deadline
        });
        assert_eq!(driver.requested, Some(start));
        assert_eq!(driver.phases.len(), 10);
        assert_eq!(armed.len(), 10);
        let question = driver.phases[0].1.until();
        let answer = driver.phases[1].1.until();
        assert!(question < answer);
        driver.phase("inbox question");
        assert_eq!(driver.until, question);
        driver.phase("person's answer sent");
        assert_eq!(driver.until, answer);
        for (phase, until) in armed.iter().rev().chain(&armed) {
            driver.phase(phase);
            assert_eq!(driver.until, *until, "{phase} keeps its request deadline");
        }
        driver.phase("person's answer sent");
        assert_eq!(driver.until, answer);
    }
}

pub mod twin_tests {
    use super::{Time, twin_document};
    use skein_fake_llm_domain::{self as llm, api};
    use skein_lib::{Env, Queue, ReplyTo, Token, Wall};

    pub fn the_twin_waits_for_the_childs_result_after_a_forge_notification() {
        for (fresh, prior, expected) in [
            ("2: forge landing before the child's result", 7, "wait"),
            (
                "Result from task 4:\n1d0b14e4db018fed000000000000000300000000\n\
                 pull request #2 of playground/widget\n\
                 Landed: pull request #2 merged into main as 1d0b14e4db01.",
                8,
                "finish",
            ),
        ] {
            assert_eq!(answer(fresh, prior), expected, "fresh input: {fresh}");
        }
    }

    fn answer(fresh: &str, prior: usize) -> String {
        let document = serde_json::to_vec(&twin_document()).expect("twin JSON");
        let document = temper_fake::llm::Document::decode(&document).expect("twin document");
        let mut domain = document.domain(17);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: document.limits() };
        let query = query(fresh, prior);
        let call = Token::new(1);
        let mut out = Queue::uncapped();
        llm::step(&mut domain, &env, llm::Event::Call { reply_to: ReplyTo::new(call), call, query }, &mut out);
        llm::fire(&mut domain, &env, &mut out);
        let Some(llm::Request::Reply { result: Ok(reply), .. }) = out.pop() else { panic!("twin answered") };
        let Some(api::Part::ToolCall { name, .. }) = reply.parts.first() else { panic!("twin called a tool") };
        String::from_utf8(name.to_vec()).expect("tool name")
    }
    fn query(fresh: &str, prior: usize) -> api::Query {
        let mut messages = Vec::new();
        for _ in 0..prior {
            messages.push(api::Message {
                role: api::Role::User,
                parts: Box::new([api::Part::Text { text: b"earlier".as_slice().into() }]),
            });
            messages.push(api::Message {
                role: api::Role::Assistant,
                parts: Box::new([api::Part::Text { text: b"earlier".as_slice().into() }]),
            });
        }
        messages.push(api::Message {
            role: api::Role::User,
            parts: Box::new([api::Part::Text { text: fresh.as_bytes().into() }]),
        });
        api::Query {
            credential: None,
            caching: api::Caching::Unscoped,
            model: b"fake".as_slice().into(),
            system: b"You are a goal: the work your person asked for".as_slice().into(),
            tools: Box::new([]),
            messages: messages.into_boxed_slice(),
            max_tokens: 1000,
            choice: api::ToolChoice::Auto,
        }
    }
}
