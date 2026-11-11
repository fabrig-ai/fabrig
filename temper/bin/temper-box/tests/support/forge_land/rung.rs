//! The ladder's real-model walks and their process proof (testing/live-tier.md).
//!
//! Like `forge_land`'s oracle, this driver uses bounded child commands. Its box
//! outlives the test on interruption so proof-live's final return hook can still
//! export the held account before terminating the box. Credentials go to ctl
//! and curl by path alone. No bot credential or store is read by the driver.
use super::super::configuration::turn_bounds;
use super::super::patience::Patience;
use serde_json::{Value, json};
use skein_lib::{Duration, Time};
use skein_world::{Deadline, Expired, Starved};

use std::{
    fs,
    io::Write,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("configuration bytes")).expect("configuration JSON")
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
/// recipe (temper/docs/development/qa.md, 3; protocol/configuration/settings.md, 7),
/// before the first start.
pub fn deployment(paths: &[PathBuf; 3]) {
    let _ = super::live_deployment(paths, super::LiveModel::ChatGpt { account: None });
    let config = paths[0].join("temper.json");
    let agent = paths[0].join("agent.json");
    let mut value = read_json(&config);
    let mut worker = read_json(&agent);
    for table in ["providers", "endpoints"] {
        value["engine"]["llm"][table].as_array_mut().expect("LLM table").retain(|row| row["name"] != "anthropic");
    }
    worker["endpoints"].as_array_mut().expect("endpoints").retain(|endpoint| endpoint["name"] != "anthropic");
    worker["endpoints"][0]["limits"]["output"] = 32_000.into();
    worker["endpoints"][0]["limits"]["reasoning_item"] = 32_768.into();
    // B0's carriers follow its one-call recipe.
    worker["profile"]["declared"]["calls_per_response"] = 1.into();
    let turn = turn_bounds::turn_body(&worker);
    turn_limits(&mut value, turn);
    // Walk 1's admitted declarations, not a phase-bound increase.
    value["engine"]["memory_bytes"] = 817_372_775_016_049_u64.into();
    value["workers"][0]["memory_bytes"] = 3_700_000_000_000_u64.into();
    assert_eq!(value["engine"]["limits"]["overrides"]["people.requests"], 256);
    assert_eq!(value["engine"]["limits"]["overrides"]["hub.turn_bytes"], turn);
    assert_eq!(value["application"]["limits"]["overrides"]["root.loads.loads"], 32);
    assert_eq!(worker["endpoints"][0]["limits"]["output"], 32_000);
    assert_eq!(worker["endpoints"][0]["limits"]["reasoning_item"], 32_768);
    write_json(&config, &value);
    write_json(&agent, &worker);
    let mut directories = super::super::configuration::directories(paths[0].parent().expect("deployment root"));
    directories.config = paths[0].as_os_str().as_bytes().into();
    directories.data = paths[1].as_os_str().as_bytes().into();
    directories.cache = paths[2].as_os_str().as_bytes().into();
    temper_box_shell::boxed_operations_in(&[], b"/", &directories, &fs::read(&config).expect("configuration"))
        .unwrap_or_else(|message| panic!("B0 setup refused: {message}"));
}

fn turn_limits(value: &mut Value, turn: u32) {
    turn_bounds::apply(value, turn);
    value["engine"]["limits"]["overrides"]["people.requests"] = 256.into();
    value["application"]["limits"]["overrides"]["root.loads.loads"] = 32.into();
}

/// A private command transcript, with a hard wall bound and no pipe backpressure.
struct Driver {
    binary: PathBuf,
    store: PathBuf,
    client: PathBuf,
    sequence: usize,
    until: Time,
    deadline: Option<Deadline>,
    phases: Vec<(&'static str, Deadline)>,
    active: Option<usize>,
    starved: Option<Starved>,
    in_world: bool,
    phase: &'static str,
    requested: Option<Time>,
    api: String,
    header: PathBuf,
    project: String,
    forge: u64,
}

impl Driver {
    fn new(store: &Path, client: &Path) -> Self {
        Self {
            binary: skein_scratch::paths::binary("temper-box"),
            store: store.into(),
            client: client.into(),
            sequence: 0,
            until: now().saturating_add(Duration::from_secs(300)),
            deadline: None,
            phases: Vec::new(),
            active: None,
            starved: None,
            in_world: true,
            phase: "setup",
            requested: None,
            api: String::new(),
            header: PathBuf::new(),
            project: String::new(),
            forge: 0,
        }
    }

    fn owned(store: &Path, client: &Path) -> Self {
        let mut driver = Self::new(store, client);
        driver.in_world = false;
        driver.arm("the rung's setup", Duration::from_secs(300));
        driver
    }

    fn arm(&mut self, phase: &'static str, within: Duration) {
        self.phase = phase;
        let deadline = Deadline::arm(phase, within);
        self.until = deadline.until();
        self.deadline = Some(deadline);
        self.active = None;
    }

    fn deadline(&mut self) -> Option<&mut Deadline> {
        match self.active {
            Some(index) => Some(&mut self.phases[index].1),
            None => self.deadline.as_mut(),
        }
    }

    fn check_wait(&mut self) -> Result<(), String> {
        if now() < self.until {
            return Ok(());
        }
        let message = format!("phase {} exceeded its bound", self.phase);
        match self.deadline().map(|deadline| deadline.expired()) {
            Some(Expired::Starved(starved)) => {
                let message = format!("{message}: {}", starved.figures());
                self.starved = Some(starved);
                Err(message)
            }
            Some(Expired::Failed(figures)) => Err(format!("{message}: {figures}")),
            // The offline setup's 300 s bound exceeds its 240 s world patience.
            None => Err(message),
        }
    }

    fn request(&mut self) {
        self.requested = Some(now());
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
        .map(|(phase, milliseconds)| (phase, Deadline::arm(phase, Duration::from_millis(milliseconds))))
        .collect();
    }

    fn execute(&mut self, command: Command, input: Option<&str>, keep: bool) -> Result<Vec<u8>, String> {
        self.execute_within(command, input, keep, None)
    }

    fn execute_within(
        &mut self,
        mut command: Command,
        input: Option<&str>,
        keep: bool,
        within: Option<Duration>,
    ) -> Result<Vec<u8>, String> {
        self.check_wait()?;
        let number = self.sequence;
        self.sequence += 1;
        let stdout = self.store.join(format!("call-{number}.out"));
        let stderr = self.store.join(format!("call-{number}.err"));
        command
            .current_dir(&self.store)
            .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(fs::File::create(&stdout).map_err(|e| e.to_string())?)
            .stderr(fs::File::create(&stderr).map_err(|e| e.to_string())?);
        let mut request_wait =
            within.filter(|_| !self.in_world).map(|within| Deadline::arm("the rung's forge request", within));
        let mut world_request =
            within.filter(|_| self.in_world).map(|within| Patience::arm("the rung's forge request", within));
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        if let Some(deadline) = self.deadline() {
            deadline.watch_process(child.id());
        }
        if let Some(deadline) = &mut request_wait {
            deadline.watch_process(child.id());
        }
        if let Some(patience) = &mut world_request {
            patience.watch_process(child.id());
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
            if let Some(patience) = &mut world_request
                && let Some(figures) = patience.failed()
            {
                super::super::parent::end_child(&mut child);
                return Err(format!("forge request exceeded its bound: {figures}"));
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
            let mut locked = Command::new("flock");
            locked
                .arg("-x")
                .arg(self.store.join("login.lock"))
                .args(["sh", "-c", "test ! -e \"$1\" || exit 1; shift; exec \"$@\"", "rung-import"])
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

/// The guard's return through the installed give-back, which a box still up answers
/// (temper/docs/design/testing/live-tier.md).
pub fn return_through_hook(binary: &Path, store: &Path) -> Result<(), String> {
    let client = store.join("client");
    let mut driver = Driver::owned(store, &client);
    driver.binary = binary.into();
    give_back(&mut driver)
}

fn give_back(driver: &mut Driver) -> Result<(), String> {
    driver.arm("the held record's return", Duration::from_secs(60));
    let output = driver.execute(Command::new(driver.store.join("give-back")), None, true)?;
    super::super::live::notice(&String::from_utf8_lossy(&output));
    Ok(())
}

/// The guard's return with no box up: a box over the store for this alone, a new client signed
/// in by the box's link, then give-back. The box stops on any ending; the store stays.
pub fn return_on_a_new_box(binary: &Path, store: &Path, paths: &[PathBuf; 3], account: &Path) -> Result<(), String> {
    let client = store.join("return-client");
    fs::create_dir_all(&client).map_err(|error| format!("return client: {error}"))?;
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("private return client: {error}"))?;
    super::super::held_login::recovery_hook(binary, store, &client, account);
    let mut driver = Driver::owned(store, &client);
    driver.binary = binary.into();
    let started = now();
    driver.arm("the held record's new box", Duration::from_secs(240));
    let mut guard = super::super::parent::Guard::new(start_box(binary, store, paths));
    let boxed = guard.child.as_mut().expect("return box");
    if let Some(deadline) = driver.deadline() {
        deadline.watch_process(boxed.id());
    }
    let returned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let link = return_link(&mut driver, boxed)?;
        super::super::live::notice(&format!(
            "RETURN box answered in {}ms",
            now().saturating_since(started).as_nanos() / 1_000_000
        ));
        driver.ctl(&["sign-in", "-"], Some(&link))?;
        give_back(&mut driver)
    }));
    stop_return_box(boxed);
    returned.unwrap_or_else(|panic| std::panic::resume_unwind(panic))
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
    run_walk(&store, &paths, &account, false);
}

fn start_box(binary: &Path, store: &Path, paths: &[PathBuf; 3]) -> Child {
    Command::new("setsid")
        .arg(binary)
        .args(super::deployment_flags(paths))
        .args(["--echo", "steps"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .current_dir(store)
        .stdin(Stdio::null())
        .stdout(fs::File::create(store.join("box.out")).expect("box output"))
        .stderr(fs::File::create(store.join("box.err")).expect("box errors"))
        .spawn()
        .expect("rung box")
}

/// Both faces import, walk, return the account, then stop through these acts.
fn run_walk(store: &Path, paths: &[PathBuf; 3], account: &Path, offline: bool) {
    let _ = super::cache_forgejo_binaries(&paths[2]);
    let client = store.join("client");
    fs::create_dir(&client).expect("fresh client");
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client");
    let mut driver = Driver::owned(store, &client);
    super::super::held_login::return_hook(&driver.binary, store, &driver.client, account);
    let mut boxed = start_box(&driver.binary, store, paths);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| walk(&mut driver, paths, account, offline)));
    // No ordinary failure or panic bypasses the return. The launcher's copy of
    // this act covers a signal to the test process itself.
    let starved = driver.starved.take();
    driver.arm("the rung's held login return", Duration::from_secs(60));
    let returned = driver.execute(Command::new(store.join("give-back")), None, true);
    if let Err(message) = returned {
        super::super::live::notice(&format!("LOGIN NOT BACK: {message}; box retained for recovery"));
        panic!("LOGIN NOT BACK: {message}; box retained for recovery");
    }
    super::super::live::notice("MUST login at its original path; stat regular, nonempty, 0600");
    let stopped = stop_box(&mut boxed);
    let result = result.unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    stopped.expect("box stopped after login return");
    if let Some(starved) = starved {
        super::super::parent::end_child(&mut boxed);
        starved.pass();
    }
    result.unwrap_or_else(|message| {
        super::super::live::notice(&format!("FAILURE {message}"));
        panic!("{message}")
    });
}

fn twin_peer() -> super::super::llm::Listening {
    super::super::sign_in::plan_document(&serde_json::to_vec(&twin_document()).expect("adapted twin JSON"))
}

fn twin_document() -> Value {
    let mut document: Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/real-loop/tests/fixtures/b0/llm.json"
    )))
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
                    let identity = args.contains_key("head")
                        || args.contains_key("merge")
                        || (call.starts_with("read_pull") && args.contains_key("number"));
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
    use skein_io::kernel::Exit;
    let donor = root.join("donor");
    fs::create_dir(&donor).expect("fake account donor");
    let donor_paths = ["config", "data", "cache"].map(|name| donor.join(name));
    deployment(&donor_paths);
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
        read_json(&donor_paths[0].join("temper.json"))["engine"]["llm"]["providers"][0]["sources"]["sign_in"].clone();
    let config = paths[0].join("temper.json");
    let mut destination = read_json(&config);
    destination["engine"]["llm"]["providers"][0]["sources"]["sign_in"] = registration;
    write_json(&config, &destination);
}

struct TwinPeers {
    stop: std::sync::mpsc::Receiver<()>,
    ended: bool,
    deadline: Time,
    wake: Time,
}

impl skein_world::Referee<super::super::Process> for TwinPeers {
    fn act(&mut self, at: Time, procs: &mut [super::super::Process]) {
        self.wake = at.saturating_add(Duration::from_millis(10));
        if !self.ended && !matches!(self.stop.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)) {
            let [super::super::Process::Issuer(issuer), super::super::Process::Peer(peer)] = procs else {
                panic!("twin peers")
            };
            issuer.shutdown();
            peer.shutdown();
            self.ended = true;
        }
    }
    fn observe(&mut self, _at: Time, _procs: &[super::super::Process]) {}
    fn next_deadline(&self) -> Option<Time> {
        Some(self.deadline.min(self.wake))
    }
    fn overdue(&self, at: Time) -> Option<String> {
        (at >= self.deadline).then(|| "B0 twin peers did not settle".into())
    }
    fn passed(&self) -> bool {
        self.ended
    }
}

/// Real Forgejo, runner, ctl and clock, with only the account/model replaced.
#[expect(
    clippy::disallowed_methods,
    reason = "the fake peers' world serves the synchronous ctl driver on a scoped thread"
)]
pub fn offline_walk(root: &Path) {
    use super::super::{Process, sign_in};
    use skein_shell::{Clock, Host};
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
        let (stop, receiver) = std::sync::mpsc::channel();
        let (ready, listening) = std::sync::mpsc::channel();
        let peers = scope.spawn(move || {
            let clock = Clock::new();
            // Setup plus the unchanged whole-walk bound; not a phase increase.
            let within = Duration::from_secs(700);
            let mut world = skein_world::real::World::new(TwinPeers {
                stop: receiver,
                ended: false,
                deadline: clock.now().now.saturating_add(within),
                wake: clock.now().now,
            });
            world.spawn(|| Process::Issuer(Box::new(issuer)));
            let index = peer.spawn(&mut world);
            ready.send(()).expect("twin peer world ready");
            let outcome = world.run(&clock, within);
            assert!(outcome.procs.iter().all(Host::is_empty), "twin peer io drained");
            let seen = super::peer_seen(&outcome.procs, index);
            assert!(!seen.requests.is_empty(), "twin served the driver");
            assert!(seen.answers.iter().all(Result::is_ok), "every twin answer served");
            assert!(
                seen.schemes.iter().all(|scheme| *scheme == Some(skein_llm::Scheme::Bearer)),
                "plan bearer transport"
            );
        });
        listening.recv().expect("twin peers constructed");
        run_walk(root, &paths, &account, true);
        drop(stop);
        peers.join().expect("independent twin world");
    });
}

fn setup(driver: &mut Driver, paths: &[PathBuf; 3], account: &Path) -> Result<Value, String> {
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

/// All ordinary ctl setup before importing the held account, shared by the
/// live walk and the process proof with the existing fake model.
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
    // Funding raises run project first, then pool (client/ctl/tools.md, 3).
    // Keep the current policy when it already funds the rung's work.
    driver.ctl(&["project", "fund", project, &period.to_string(), "--key", "rung-project-funds"], None)?;
    driver.ctl(&["project", "fund", project, &owner.to_string(), "--role", "0", "--key", "rung-role-funds"], None)?;
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
    let file = driver.store.join("model-settings.json");
    write_json(&file, &json!({"items":[{"setting":setting["setting"], "set":value}]}));
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
    let memory = read_json(&paths[0].join("temper.json"))["engine"]["memory_bytes"].clone();
    super::super::live::notice(&format!("SETUP engine memory={memory}"));
    let baseline = driver.forge_json("branches")?;
    require(baseline.as_array().is_some_and(|b| b.len() == 1 && b[0]["name"] == "main"), "fresh main alone")?;
    driver.forge("raw/src/main.rs?ref=main")?;
    Ok(baseline)
}

/// Process-tier proof of the live setup through its last pre-import read.
/// The ordinary harness supervises the existing TLS fake; no account is added
/// and no real provider is contacted. The shared steps still use real Forgejo.
pub fn offline_setup(root: &Path) {
    use super::super::{
        boxed,
        ctl::{Call, Step},
    };
    use skein_io::kernel::Exit;
    use std::{cell::RefCell, rc::Rc};
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
    deployment(&paths);
    let _issuer = super::super::sign_in::patch_configuration(&paths[0]);
    let peer = super::super::sign_in::plan_peer(Box::new([]));
    peer.configure(&paths[0]);
    let _ = super::cache_forgejo_binaries(&paths[2]);
    let driver = Rc::new(RefCell::new(Driver::new(root, &root.join("client"))));
    let tools = driver.clone();
    let prepared = Rc::new(RefCell::new(Ok(())));
    let tool_result = prepared.clone();
    let outcome = boxed::run_ctl_with_llm(
        root,
        super::deployment_flags(&paths),
        root,
        vec![
            Step::Run(Call::sign_in()),
            Step::Then(Box::new(move |_| {
                *tool_result.borrow_mut() = setup_tools(&mut tools.borrow_mut());
                // Return to the harness before the observer reads setup's logs.
                Call::new(&["me", "--json"])
            })),
        ],
        boxed::BOX_DEADLINE,
        |_| -> Result<(), String> {
            prepared.borrow().clone()?;
            let mut driver = driver.borrow_mut();
            setup_observer(&mut driver, &paths)?;
            let accounts = driver.ctl(&["account", "list"], None)?;
            require(accounts["accounts"]["party"] == json!([]), "no account imported by the offline setup")?;
            let project = driver.project.clone();
            let project = driver.ctl(&["project", "show", &project], None)?;
            check_funding(&project)?;
            Ok(())
        },
        boxed::CtlLlm { expected_exits: Vec::new(), peer, on_link: Box::new(|| {}) },
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "box stop: {stderr}");
    assert_eq!(outcome.left, Some(vec![]), "no surviving children: {stderr}");
    for call in &outcome.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    assert_eq!(outcome.calls.len(), 2, "sign-in and setup checkpoint");
    outcome.checked.expect("pre-import checkpoint").expect("all pre-import setup passed");
    assert!(outcome.model.expect("fake model observations").requests.is_empty(), "no model call before import");
    eprintln!("OFFLINE B0 setup passed through account-import boundary, zero model requests; box drained");
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

fn walk(driver: &mut Driver, paths: &[PathBuf; 3], account: &Path, offline: bool) -> Result<(), String> {
    let baseline = setup(driver, paths, account)?;
    if offline {
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

#[cfg(test)]
pub mod tests {
    use super::Driver;
    use std::path::Path;

    pub fn an_expired_rung_phase_starts_no_further_command() {
        let scratch = skein_scratch::Scratch::new("rung-expired-phase");
        let mut driver = Driver::owned(scratch.path(), &scratch.path().join("client"));
        driver.arm("the expired rung phase", skein_lib::Duration::ZERO);
        let command = std::process::Command::new("/bin/true");
        assert!(driver.execute(command, None, true).is_err());
        assert_eq!(driver.sequence, 0, "the expired phase did not start its command");
        assert!(!scratch.path().join("call-0.out").exists());
    }

    pub fn rung_phases_keep_the_request_deadlines_when_selected_again() {
        let mut driver = Driver::owned(Path::new("."), Path::new("client"));
        driver.request();
        assert_eq!(driver.phases.len(), 10);
        let question = driver.phases[0].1.until();
        let answer = driver.phases[1].1.until();
        assert!(question < answer);
        driver.phase("inbox question");
        assert_eq!(driver.until, question);
        driver.phase("person's answer sent");
        assert_eq!(driver.until, answer);
        driver.phase("person's answer sent");
        assert_eq!(driver.until, answer);
    }
}

#[cfg(test)]
mod twin_tests {
    use super::*;
    use skein_fake_llm_domain::{self as llm, api};
    use skein_lib::{Env, Queue, ReplyTo, Token, Wall};

    #[test]
    fn the_twin_waits_for_the_childs_result_after_a_forge_notification() {
        for (fresh, prior, expected) in [
            ("Initial work", 0, "goal"),
            ("this work is to be agreed", 1, "goal"),
            ("Delegated tasks: 3", 2, "wait"),
            ("The repository contains a minimal Rust binary:", 3, "message"),
            (ANSWER, 5, "change"),
            ("2: forge landing before the child's result", 7, "wait"),
            ("4: Result from task 4: Landed: pull request #2", 8, "finish"),
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
