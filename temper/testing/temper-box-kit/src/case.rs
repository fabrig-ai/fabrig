//! The reference case on the cluster's processes (testing/process-tier.md and testing/reference-case.md).
mod project;
use super::{
    Ended, Scratch, WITHIN, cluster, ctl,
    story_socket::{Scope, Socket},
};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Error, Finish, Line, Report, Rule, Script};
use skein_io::kernel::Exit;
use skein_lib::{Duration, Time, Wall, base64};
use skein_shell::{Clock, Host};
use skein_world::{Referee, real};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use temper_real_loop_world::{
    calls,
    case::{
        acts,
        oracle::{Kit, Observed, Told},
    },
    forge::Fixture,
};

// testing/process-tier.md: pass at least every 20 ms while PLACED's release is due.
const PLACED_RELEASE_INTERVAL: Duration = Duration::from_millis(20);

pub struct Run {
    calls: Vec<Ended>,
    observed_at: Vec<Option<Wall>>,
    told: Vec<Told>,
    kit: Kit,
    workers: BTreeMap<u64, String>,
    steps: Vec<calls::Step>,
    first_workers: BTreeMap<u64, String>,
}
impl Run {
    #[must_use]
    pub fn calls(&self) -> &[Ended] {
        &self.calls
    }
    #[must_use]
    pub fn observed(&self, reads: &[(acts::Read, usize)]) -> Observed {
        let documents = reads
            .iter()
            .filter_map(|(read, position)| {
                let call = self.calls.get(*position)?;
                if call.exit != Some(Exit::Code(0)) {
                    return None;
                }
                Some((*read, serde_json::from_slice(&call.stdout).expect("case document")))
            })
            .collect();
        let observed_at = reads
            .iter()
            .filter_map(|(read, position)| Some((*read, self.observed_at.get(*position).copied().flatten()?)))
            .collect();
        Observed { documents, observed_at, told: self.told.clone(), commands: Vec::new(), kit: self.kit.clone() }
            .with_commands(&self.steps)
    }
    #[must_use]
    pub fn first_placed(&self) -> &BTreeMap<u64, String> {
        &self.first_workers
    }
    /// Placement only where a document retains a live observation channel.
    #[must_use]
    pub fn placed(&self) -> &BTreeMap<u64, String> {
        &self.workers
    }
}
fn read(path: &Path) -> Value {
    read_document(path).unwrap_or_else(|why| panic!("{why}"))
}
fn read_document(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|why| format!("prepared document: {why:?}"))?;
    serde_json::from_slice(&bytes).map_err(|why| format!("JSON: {why:?}"))
}
fn call_json(ended: &Ended) -> Result<Value, String> {
    serde_json::from_slice(&ended.stdout).map_err(|error| {
        format!("ctl JSON: {error}; exit {:?}; stderr: {}", ended.exit, String::from_utf8_lossy(&ended.stderr))
    })
}
fn write(path: &Path, document: &Value) {
    super::write_private(path, &serde_json::to_vec(document).expect("JSON"));
}
fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("case UTF-8")
}
fn encode(scripts: &[Script], rules: &[Box<[Rule]>]) -> Value {
    assert_eq!(scripts.len(), rules.len(), "index-parallel turn rules");
    scripts
        .iter()
        .zip(rules)
        .map(|(script, rules)| {
            let turns: Vec<_> = script
                .turns
                .iter()
                .map(|turn| {
                    let lines: Vec<_> = turn
                        .lines
                        .iter()
                        .map(|line| match line {
                            Line::Take { name, before, after } => json!({
                                "take": text(name), "before": text(before), "after": text(after)
                            }),
                            Line::Text { text: words } => json!({"text": text(words)}),
                            Line::Call { name, arguments } => json!({
                                "call": text(name),
                                "arguments": serde_json::from_slice::<Value>(arguments).expect("arguments")
                            }),
                            Line::Pieces { pieces } => json!({"pieces":pieces.iter().map(|piece| json!({
                                "text":text(&piece.text),"hold":piece.hold.as_deref().map(text)
                            })).collect::<Vec<_>>()}),
                            Line::Fail { error, report, credentials } => {
                                assert_eq!(*report, Report::none(), "case failure report unsupported: {report:?}");
                                let error = match error {
                                    Error::Overloaded => json!("overloaded"),
                                    Error::RateLimited { retry_after } => json!({"rate_limited": {
                                        "retry_after_ms": retry_after.as_nanos() / 1_000_000
                                    }}),
                                    Error::Unavailable => json!("unavailable"),
                                    Error::ContextTooLong => json!("context_too_long"),
                                    Error::Unauthorized => json!("unauthorized"),
                                    Error::Exhausted { retry_after } => json!({"exhausted": {
                                        "retry_after_ms": retry_after.map(|delay| delay.as_nanos() / 1_000_000)
                                    }}),
                                    Error::Refused => json!("refused"),
                                    Error::Unsupported => json!("unsupported"),
                                    Error::InvalidRequest => json!("invalid_request"),
                                };
                                json!({"fail": {"error": error, "credentials": credentials}})
                            }
                            other @ Line::Opaque { .. } => {
                                panic!("case script line unsupported: {other:?}")
                            }
                        })
                        .collect();
                    let finish = match turn.finish {
                        Finish::Stop => "stop",
                        Finish::ToolCalls => "tool_calls",
                        Finish::Length => "length",
                        Finish::ContentFilter => "content_filter",
                    };
                    json!({"lines": lines, "finish": finish, "tokens": turn.tokens})
                })
                .collect();
            let mut script = json!({"cue": text(&script.cue), "turns": turns});
            if !rules.is_empty() {
                script["inputs"] = rules.iter().map(|rule| json!({"cue":text(&rule.cue),"turn":rule.turn})).collect();
            }
            script
        })
        .collect()
}
/// Bring the cluster fixture's combined document to tests/real-loop/src/fixtures.rs's base
/// before applying the case fixture once, as that world's cluster does.
fn base(value: &mut Value) {
    temper_real_loop_world::settings::stage_policies(value);
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine").remove(member);
    }
}
fn process_settings(value: &mut Value) -> Value {
    let given = temper_real_loop_world::fixtures::own_defaults(value, &value["world_settings"]);
    let temper_service::stock::Given::Own { jig: defaults, forge, .. } = given else { panic!("case defaults") };
    let mut items = temper_real_loop_world::case::defaults::items(&defaults);
    // Project 1 is made after sign-in; give its rules deployment scope for this story.
    let mut forge = forge;
    for item in &mut forge {
        if item.kind == 2 {
            item.kind = 1;
            item.scope = 0;
        }
    }
    items.as_array_mut().expect("case settings").extend(temper_real_loop_world::settings_items::forge_items(&forge));
    value.as_object_mut().expect("deployment").remove("world_settings");
    // Settings and the project are established by the owner after sign-in.
    // Fixture records supply credentials without adopting a project before sign-in.
    value["engine"]["projects"] = json!([]);
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine").remove(member);
    }
    for (name, minimum) in [
        // Nine slots hold the stock charters.
        ("tasks.charters", 16),
        ("core.templates", 6),
        ("authority.grants", 16),
        ("notes.scopes", 32),
        ("core.policy_bytes", 8192),
    ] {
        let held = value["engine"]["limits"]["overrides"][name].as_u64().unwrap_or(0);
        value["engine"]["limits"]["overrides"][name] = held.max(minimum).into();
    }
    items
}

pub fn run(
    workers: &[&str],
    fixture: Fixture,
    scripts: fn(SocketAddr) -> Box<[Script]>,
    rules: fn() -> Box<[Box<[Rule]>]>,
    steps: Vec<calls::Step>,
    placed: &[u64],
) -> Run {
    assert!(workers == ["ws"] || workers == ["ws", "s1", "s2"], "case cast");
    let cast = if workers.len() == 1 { vec!["ws"] } else { vec!["ws", "s1", "s2"] };
    if cast.len() == 1 {
        assert!(placed.is_empty(), "one worker needs no placement mapping");
    }
    let late = steps.iter().any(|step| matches!(step, calls::Step::AddWorker { .. }));
    let clock = Clock::new();
    let began = clock.now().now;
    let scratch = Scratch::new(&format!("case_process_{}", std::process::id()));
    let mut description = cluster::Cluster::read(scratch.path());
    description.rename("w1", "ws");
    if cast.len() > 1 {
        description.rename("w2", "s1");
        description.copy("s1", "s2");
    }
    description.record_llm();
    let [hosts] = super::free_addresses(&[]);
    description.bind("proc.engine.ready.workers", &hosts.to_string());
    let forge = description.prepare("forge");
    let mut document = read(&forge.join("forge.json"));
    let kit = (fixture.setup)();
    let ci = match kit.checks.cue {
        Some(skein_fake_forge_domain::api::Cue::Declared { path }) => json!({"declared":text(&path)}),
        Some(skein_fake_forge_domain::api::Cue::File { path, green }) => {
            json!({"path":text(&path),"green":text(&green)})
        }
        Some(
            skein_fake_forge_domain::api::Cue::Message { .. }
            | skein_fake_forge_domain::api::Cue::PullFile { .. }
            | skein_fake_forge_domain::api::Cue::Tests { .. },
        ) => panic!("case forge cue unsupported"),
        None => Value::Null,
    };
    document["owner"] = "acme".into();
    document["repositories"] = json!([
        {"id":kit.id,"name":"app","landing":text(&kit.default),"ci":ci,"tree":kit.tree.iter().map(|file| json!({"path":text(&file.path),"content":text(&file.content)})).collect::<Vec<_>>()},
        {"id":1,"name":"spare","landing":"main","ci":null}
    ]);
    write(&forge.join("forge.json"), &document);
    let git = scratch.path().join("git");
    let (mut members, roots) = description.members();
    members.up(&["forge"]);
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = real::World::new(Judge {
        members,
        finding: None,
        cast,
        stage: Stage::Forge,
        deadline: began.saturating_add(WITHIN),
        marked: began,
        phases: Vec::new(),
        fixture,
        scripts,
        rules,
        hosts,
        git,
        steps: steps.into(),
        ran: Vec::new(),
        calls: Vec::new(),
        observed_at: Vec::new(),
        slots: Vec::new(),
        client: scratch.path().join("client"),
        provisioning: false,
        live: None,
        placed: placed.to_vec(),
        run: result.clone(),
        stopped: false,
        late,
        control: None,
        placed_release: None,
        assigned: BTreeSet::new(),
        channels: BTreeMap::new(),
        workers: BTreeMap::new(),
        first_workers: BTreeMap::new(),
        processed: 0,
        stopped_links: None,
        admin_token: Vec::new(),
        settings: Value::Null,
        project: None,
    });
    for root in roots {
        world.spawn(|| root);
    }
    world.spawn(cluster::Root::client);
    let outcome = world.run(&clock, WITHIN);
    assert!(outcome.procs.iter().all(Host::is_empty), "every root empty");
    scratch.remove();
    result.borrow_mut().take().expect("case observations")
}
#[derive(Debug)]
enum Stage {
    Forge,
    Llm,
    Engine,
    Ws,
    S1,
    S2,
    Added { target: usize },
    Placed,
    BeforeAdd,
    Stopping,
    Starting { target: usize },
    BeforeStop,
    Acts,
    Provision,
    ReadRules,
    ChangeSettings,
    Live,
    Accounts,
    Stop,
}
struct Judge {
    members: cluster::Members,
    finding: Option<String>,
    cast: Vec<&'static str>,
    stage: Stage,
    deadline: Time,
    marked: Time,
    phases: Vec<(&'static str, skein_lib::Duration)>,
    fixture: Fixture,
    scripts: fn(SocketAddr) -> Box<[Script]>,
    rules: fn() -> Box<[Box<[Rule]>]>,
    hosts: SocketAddr,
    git: PathBuf,
    steps: VecDeque<calls::Step>,
    ran: Vec<calls::Step>,
    calls: Vec<Ended>,
    observed_at: Vec<Option<Wall>>,
    slots: Vec<usize>,
    client: PathBuf,
    provisioning: bool,
    live: Option<Value>,
    placed: Vec<u64>,
    run: std::rc::Rc<std::cell::RefCell<Option<Run>>>,
    stopped: bool,
    late: bool,
    control: Option<SocketAddr>,
    placed_release: Option<Time>,
    assigned: BTreeSet<String>,
    channels: BTreeMap<u64, u64>,
    workers: BTreeMap<u64, String>,
    first_workers: BTreeMap<u64, String>,
    processed: usize,
    stopped_links: Option<usize>,
    admin_token: Vec<u8>,
    settings: Value,
    project: Option<u32>,
}
impl Judge {
    #[expect(clippy::too_many_lines, reason = "the case adapter keeps its staged process actions together")]
    fn advance_world(&mut self, now: Time, roots: &mut [cluster::Root]) -> Result<(), String> {
        self.members.act(now, roots);
        if self.members.failed() {
            self.stage = Stage::Stop;
            self.placed_release = None;
            return Ok(());
        }
        if let Some(links) = self.stopped_links {
            if !self.members.ended("ws") {
                return Err("ws remains ended until StartWorker".into());
            }
            if self.members.linked("ws") != links {
                return Err(format!(
                    "ws did not relink before StartWorker\n  left: {:?}\n right: {:?}",
                    self.members.linked("ws"),
                    links
                ));
            }
        }
        let root = roots.last_mut().ok_or("client root")?;
        match self.stage {
            Stage::Forge if self.members.is_up() => self.start_llm(now)?,
            Stage::Llm if self.members.is_up() => {
                self.mark("llm ready", now);
                self.configure()?;
                self.members.up(&["engine"]);
                self.stage = Stage::Engine;
            }
            Stage::Engine if self.members.is_up() && root.is_empty() => {
                self.mark("engine ready", now);
                self.members.up(&["ws"]);
                self.stage = Stage::Ws;
            }
            Stage::Ws if self.members.is_up() => self.ws_ready(now, root)?,
            Stage::S1 if self.members.is_up() => {
                self.mark("s1 ready", now);
                if self.late {
                    self.queue(root)?;
                    self.stage = Stage::Acts;
                } else {
                    self.members.up(&["s2"]);
                    self.stage = Stage::S2;
                }
            }
            Stage::S2 if self.members.is_up() => {
                self.queue(root)?;
                self.stage = Stage::Acts;
            }
            Stage::BeforeAdd if Self::calls(root)?.foreground_idle() => self.add_worker(root)?,
            Stage::Added { target } if self.members.linked("s2") == target => {
                let task = temper_real_loop_world::case::BENCHMARK.to_string();
                let batch = Self::calls(root)?;
                batch.push(ctl::Step::Run(ctl::Call::new(&["task", "show", &task, "--until", "replied", "--json"])));
                batch.push(ctl::Step::Run(ctl::Call::new(&["account", "list", "--json"])));
                self.stage = Stage::Placed;
            }
            Stage::Placed if Self::calls(root)?.foreground_idle() => self.record_placement(root)?,
            Stage::BeforeStop if Self::calls(root)?.foreground_idle() => {
                self.map_channels(&call_json(Self::calls(root)?.ends().last().ok_or("first life hosts")?)?)?;
                self.first_workers = std::mem::take(&mut self.workers);
                self.channels.clear();
                self.members.stop_one("ws");
                self.stage = Stage::Stopping;
            }
            Stage::Stopping if self.members.ended("ws") && Self::calls(root)?.foreground_idle() => {
                self.stopped_links = Some(self.members.linked("ws"));
                self.stage = Stage::Acts;
            }
            Stage::Starting { target } if self.members.linked("ws") == target => self.stage = Stage::Acts,
            Stage::Acts => self.advance(now, root)?,
            Stage::Accounts if Self::calls(root)?.foreground_idle() => self.finish_acts(now, root)?,
            Stage::Provision if Self::calls(root)?.foreground_idle() => self.read_startup_rules(root)?,
            Stage::ReadRules if Self::calls(root)?.foreground_idle() => self.change_settings(root)?,
            Stage::ChangeSettings if Self::calls(root)?.foreground_idle() => self.make_project(root)?,
            Stage::Live if Self::calls(root)?.foreground_idle() => {
                let read = Self::calls(root)?.ends().last().ok_or("director's running read")?;
                success(read)?;
                live_channels(&call_json(read)?, &mut self.channels);
                self.live = Some(json!({"chat_act":self.ran.len()}));
                self.stage = Stage::Acts;
            }
            Stage::Forge
            | Stage::Llm
            | Stage::Engine
            | Stage::Ws
            | Stage::S1
            | Stage::S2
            | Stage::BeforeAdd
            | Stage::Added { .. }
            | Stage::Placed
            | Stage::BeforeStop
            | Stage::Stopping
            | Stage::Starting { .. }
            | Stage::Provision
            | Stage::ReadRules
            | Stage::ChangeSettings
            | Stage::Live
            | Stage::Accounts
            | Stage::Stop => {}
        }
        self.release_placed(now)?;

        Ok(())
    }
}
impl Judge {
    fn mark(&mut self, phase: &'static str, now: Time) {
        self.phases.push((phase, now.saturating_since(self.marked)));
        self.marked = now;
    }
    fn calls(root: &mut cluster::Root) -> Result<&mut ctl::Calls, String> {
        let cluster::Root::Client { calls: Some(calls), .. } = root else { return Err("client calls".into()) };
        Ok(calls)
    }
    fn queue(&self, root: &mut cluster::Root) -> Result<(), String> {
        let link = format!("http://{}", self.members.capture("engine", "link").ok_or("engine link")?);
        let mut batch = ctl::Calls::linked(&link, &self.client);
        batch.hold();
        let cluster::Root::Client { calls, .. } = root else { return Err("client root".into()) };
        *calls = Some(Box::new(batch));

        Ok(())
    }
    fn read_startup_rules(&mut self, root: &mut cluster::Root) -> Result<(), String> {
        for call in Self::calls(root)?.ends() {
            success(call)?;
        }
        Self::calls(root)?.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "rules", "--json"])));
        self.stage = Stage::ReadRules;

        Ok(())
    }
    fn change_settings(&mut self, root: &mut cluster::Root) -> Result<(), String> {
        let read = Self::calls(root)?.ends().last().ok_or("startup rules")?;
        success(read)?;
        let original = call_json(read)?;
        for item in self.settings.as_array_mut().ok_or("case setting items")? {
            if item["setting"]["kind"] == "rules" {
                item["set"]["requirements"] = original["value"]["requirements"].clone();
            }
        }
        let file = self.client.parent().ok_or("case scratch")?.join("case-settings.json");
        write(&file, &json!({"items":self.settings}));
        Self::calls(root)?.push(ctl::Step::Run(ctl::Call::new(&[
            "settings",
            "apply",
            file.to_str().ok_or("settings path")?,
            "--against",
            "1",
            "--key",
            "case-settings",
        ])));
        self.stage = Stage::ChangeSettings;

        Ok(())
    }
    fn make_project(&mut self, root: &mut cluster::Root) -> Result<(), String> {
        success(Self::calls(root)?.ends().last().ok_or("case settings changed")?)?;
        let made = project::make(&self.client);
        if u64::from(made) != temper_real_loop_world::case::PROJECT {
            return Err(format!(
                "the case keeps its project identity\n  left: {:?}\n right: {:?}",
                u64::from(made),
                temper_real_loop_world::case::PROJECT
            ));
        }
        self.project = Some(made);
        self.stage = Stage::Acts;

        Ok(())
    }
    fn configure(&mut self) -> Result<(), String> {
        let engine_dir = self.members.prepare("engine");
        let ws_dir = self.members.prepare("ws");
        let mut value = read_document(&engine_dir.join("config/temper.json"))?;
        let ws = read_document(&ws_dir.join("config/temper.json"))?;
        let mut agent = read_document(&ws_dir.join("config/agent.json"))?;
        // With a control listener, the fake's ready line names provider then control.
        let mut addresses = self.members.capture("llm", "address").ok_or("LLM addresses")?.split_whitespace();
        let llm: SocketAddr =
            addresses.next().ok_or("LLM provider")?.parse().map_err(|why| format!("provider socket: {why:?}"))?;
        let control = addresses
            .next()
            .map(str::parse::<SocketAddr>)
            .transpose()
            .map_err(|why| format!("control socket: {why:?}"))?;
        if control != self.control {
            return Err(format!(
                "LLM control ready at its reserved address\n  left: {:?}\n right: {:?}",
                control, self.control
            ));
        }
        if addresses.next().is_some() {
            return Err("only provider and control addresses".into());
        }
        value["engine"]["llm"]["endpoints"][0]["address"] = json!({"socket":llm.to_string()});
        agent["endpoints"][0]["address"] = llm.to_string().into();
        value["workers"] = ws["workers"].clone();
        value["application"]["workers"] = ws["application"]["workers"].clone();
        base(&mut value);
        let forge = self
            .members
            .capture("forge", "address")
            .ok_or("forge address")?
            .parse()
            .map_err(|why| format!("socket: {why:?}"))?;
        // The fake forge removes its token file when it ends, so the token is read while it runs.
        let document = read_document(&self.members.dir("forge").join("forge.json"))?;
        self.admin_token = fs::read(document["admin_token"].as_str().ok_or("admin token file")?)
            .map_err(|why| format!("fake admin token: {why:?}"))?;
        (self.fixture.configure)(&mut value, &mut agent, forge);
        self.settings = process_settings(&mut value);
        value["application"]["webhooks"] = json!({"listener":{"socket":"127.0.0.1:0"}});
        super::configuration::turn_bounds::apply(&mut value, super::configuration::turn_bounds::turn_body(&agent));
        value["engine"]["hosts"]["listener"] = json!([{"socket":self.hosts.to_string()}]);
        for key in ["fleet.workers", "fleet.attempts"] {
            // tests/real-loop/src/cluster.rs: one admitted position/attempt per worker.
            value["engine"]["limits"]["overrides"][key] = 3.into();
        }
        // accounts' program hosts hold documents.lines rows (jig documents/programs.rs `hosts`);
        // six lines also carry all stock charters in the creation offer.
        value["engine"]["limits"]["overrides"]["documents.lines"] = 7.into();
        value["engine"]["hosts"]["workers"][0]["name"] = "ws".into();
        value["engine"]["hosts"]["workers"].as_array_mut().ok_or("hosts")?.truncate(self.cast.len());
        let secret = b"s2-case-fixture-secret";
        if self.cast.len() > 1 {
            value["engine"]["hosts"]["workers"][1]["name"] = "s1".into();
            value["engine"]["hosts"]["workers"]
                .as_array_mut()
                .ok_or("hosts")?
                .push(json!({"name":"s2","digest":"secrets/s2.digest"}));
            let mut hash = skein_lib::sha256::Sha256::new();
            hash.update(secret);
            let mut digest = String::new();
            for byte in hash.finish() {
                use std::fmt::Write as _;
                write!(digest, "{byte:02x}").map_err(|why| format!("hex: {why:?}"))?;
            }
            super::write_private(&engine_dir.join("data/secrets/s2.digest"), digest.as_bytes());
        }
        for &name in &self.cast {
            let dir = if name == "ws" { ws_dir.clone() } else { self.members.prepare(name) };
            let mut worker = read_document(&dir.join("config/temper.json"))?;
            if name == "s2" {
                // Copy keeps the source templates; give the new member its own paths and secret.
                let source = self.members.dir("s1");
                worker = serde_json::from_str(
                    &serde_json::to_string(&worker)
                        .map_err(|why| format!("worker: {why:?}"))?
                        .replace(source.to_str().ok_or("source path")?, dir.to_str().ok_or("worker path")?),
                )
                .map_err(|why| format!("copied worker: {why:?}"))?;
                fs::remove_file(dir.join("data/secrets/w2.secret"))
                    .map_err(|why| format!("remove copied secret: {why:?}"))?;
                worker["workers"][0]["secret"] = "secrets/s2.secret".into();
                super::write_private(&dir.join("data/secrets/s2.secret"), secret);
            }
            let mut shaped = value["workers"][0].clone();
            // Preserve this member's independent paths and secret, copying only fixture policy.
            shaped["name"] = name.into();
            shaped["secret"] = worker["workers"][0]["secret"].clone();
            shaped["agent"] = worker["workers"][0]["agent"].clone();
            worker["workers"] = json!([shaped]);
            worker["application"]["limits"] = value["application"]["limits"].clone();
            worker["application"]["workers"][0]["git"] = self.git.to_str().ok_or("git path")?.into();
            write(&dir.join("config/temper.json"), &worker);
            write(&dir.join("config/agent.json"), &agent);
        }
        value["workers"] = json!([]);
        value["application"]["workers"] = json!([]);
        write(&engine_dir.join("config/temper.json"), &value);
        super::forgejo::bootstrap(&engine_dir.join("config"), &engine_dir.join("data"), &engine_dir.join("cache"));
        seed_forge(&engine_dir.join("data"), forge, &self.admin_token);

        Ok(())
    }
    fn arguments(&self, argv: &calls::Argv) -> Result<Vec<String>, String> {
        argv.0
            .iter()
            .enumerate()
            .map(|(position, arg)| {
                Ok(match arg {
                    calls::Arg::GithubApp => return Err("GithubApp unsupported".into()),
                    calls::Arg::Lit(word) => (*word).into(),
                    calls::Arg::Number(_)
                        if position > 0 && matches!(&argv.0[position - 1], calls::Arg::Lit("--project")) =>
                    {
                        self.project.ok_or("ProjectMade before acts")?.to_string()
                    }
                    calls::Arg::Number(n) => n.to_string(),
                    calls::Arg::File(name) => {
                        self.client.parent().ok_or("scratch")?.join(name).to_str().ok_or("file path")?.into()
                    }
                    calls::Arg::Task(n) => std::str::from_utf8(&self.calls[*n].stdout)
                        .map_err(|why| format!("case UTF-8: {why:?}"))?
                        .split("done: task ")
                        .nth(1)
                        .ok_or("created task")?
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                        .ok_or("task")?
                        .into(),
                    calls::Arg::Delegate(n) => {
                        let v = call_json(&self.calls[*n])?;
                        let tree = &v["tree"];
                        let root = &tree["root"]["task"];
                        let children: Vec<_> = tree["rows"]
                            .as_array()
                            .ok_or("tree rows")?
                            .iter()
                            .filter(|row| &row["parent"] == root)
                            .collect();
                        if children.len() != 1 {
                            return Err(format!(
                                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                                children.len(),
                                1
                            ));
                        }
                        children[0]["chip"]["task"].as_u64().ok_or("child")?.to_string()
                    }
                    calls::Arg::Digest(n) => {
                        call_json(&self.calls[*n])?["task"]["digest"].as_str().ok_or("digest")?.into()
                    }
                    calls::Arg::ActivityBefore(n) => {
                        let v = call_json(&self.calls[*n])?;
                        v.pointer("/activity/before/value")
                            .or_else(|| v.pointer("/before/value"))
                            .and_then(Value::as_str)
                            .ok_or("before")?
                            .into()
                    }
                    calls::Arg::InboxAfter(n) => {
                        let v = call_json(&self.calls[*n])?;
                        let cursor =
                            v.pointer("/needs/after/value").and_then(Value::as_str).ok_or("next needs page")?;
                        format!("00{cursor}")
                    }
                })
            })
            .collect()
    }
    fn prepare_calls(&mut self, root: &mut cluster::Root) -> Result<bool, String> {
        let batch = Self::calls(root)?;
        if !batch.foreground_idle() {
            return Ok(false);
        }
        self.calls = self
            .slots
            .iter()
            .map(|slot| {
                let ended = &batch.ends()[*slot];
                Ended { exit: ended.exit, stdout: ended.stdout.clone(), stderr: ended.stderr.clone() }
            })
            .collect();
        self.observed_at = self.slots.iter().map(|slot| batch.observed_at()[*slot]).collect();
        for ended in &batch.ends()[self.processed..] {
            if ended.exit.is_none() {
                continue;
            }
            success(ended)?;
            if !self.placed.is_empty()
                && let Ok(document) = serde_json::from_slice::<Value>(&ended.stdout)
            {
                live_channels(&document, &mut self.channels);
            }
        }
        self.processed = batch.ends().len();
        if !self.provisioning && self.calls.len() == 1 {
            self.provisioning = true;
            let key = self.members.dir("llm").join("llm.key");
            batch.push(ctl::Step::Run(ctl::Call::new(&[
                "account",
                "add-key",
                "1",
                key.to_str().ok_or("key")?,
                "--deployment",
                "--key",
                "case-llm",
            ])));
            batch.push(ctl::Step::Run(ctl::Call::new(&["account", "list", "--until", "usable"])));
            self.stage = Stage::Provision;
            return Ok(false);
        }
        if !self.placed.is_empty()
            && matches!(self.ran.last(), Some(calls::Step::Call { argv, .. }) if self.arguments(argv)?[0] == "chat")
            && !matches!(self.live.as_ref(), Some(value) if value["chat_act"] == self.ran.len())
        {
            // The harness's own read, as the provisioning is: the director's chat
            // can end before an act reads it live, and an ended document names no
            // worker, so read its running document once.
            let director = std::str::from_utf8(&self.calls.last().ok_or("chat call")?.stdout)
                .map_err(|why| format!("case UTF-8: {why:?}"))?
                .split("done: task ")
                .nth(1)
                .ok_or("chat task")?
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .ok_or("task")?
                .to_owned();
            batch.push(ctl::Step::Run(ctl::Call::new(&[
                "task",
                "show",
                &director,
                "--until",
                "phase:running",
                "--json",
            ])));
            self.stage = Stage::Live;
            return Ok(false);
        }
        Ok(true)
    }
    #[expect(clippy::too_many_lines, reason = "the case adapter explicitly refuses unsupported real-loop steps")]
    fn advance(&mut self, now: Time, root: &mut cluster::Root) -> Result<(), String> {
        use calls::Step;
        if !self.prepare_calls(root)? {
            return Ok(());
        }
        let batch = Self::calls(root)?;
        let Some(step) = self.steps.pop_front() else {
            self.mark("acts", now);
            batch.push(ctl::Step::Run(ctl::Call::new(&["account", "list", "--json"])));
            self.stage = Stage::Accounts;
            return Ok(());
        };
        match &step {
            Step::SignIn { at } => {
                if at.0 != 1 {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", at.0, 1));
                }
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::Run(ctl::Call::sign_in()));
            }
            Step::Call { at, argv } => {
                if at.0 != 1 {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", at.0, 1));
                }
                let mut args = self.arguments(argv)?;
                if args.iter().any(|arg| matches!(arg.as_str(), "--until" | "--since"))
                    && !args.iter().any(|arg| arg == "--json")
                {
                    args.push("--json".into());
                }
                if matches!(self.steps.front(), Some(Step::StopWorker { name: "ws" })) {
                    let director = temper_real_loop_world::case::HANG.to_string();
                    if args != ["task", "show", &director, "--until", "replied", "--json"] {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            args,
                            ["task", "show", &director, "--until", "replied", "--json"]
                        ));
                    }
                    batch.push(ctl::Step::Run(ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>())));
                    // Replied also accepts WaitingForYou while work is still writing.
                    // After the committed-read barrier, retain the exact waiting
                    // observation the oracle requires before stopping ws.
                    self.slots.push(batch.ends().len() + 1);
                    batch.push(ctl::Step::Until(
                        ctl::Call::new(&["task", "show", &director, "--json"]),
                        b"\"work\":\"waiting\"",
                    ));
                } else {
                    self.slots.push(batch.ends().len());
                    batch.push(ctl::Step::Run(ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>())));
                }
            }
            Step::Start { at, argv } => {
                if at.0 != 1 {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", at.0, 1));
                }
                let args = self.arguments(argv)?;
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::Start(ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>())));
            }
            Step::Until { at, argv, holds, early: _ } => {
                if at.0 != 1 {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", at.0, 1));
                }
                let args = self.arguments(argv)?;
                self.slots.push(batch.ends().len());
                batch.push(ctl::Step::UntilMatching(
                    ctl::Call::new(&args.iter().map(String::as_str).collect::<Vec<_>>()),
                    *holds,
                ));
            }
            Step::Join { call } => {
                if *call >= self.slots.len() {
                    return Err("assertion failed: *call < self.slots.len()".into());
                }
                batch.push(ctl::Step::Join);
            }
            Step::AddWorker { name } => {
                if *name != "s2" {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", name, "s2"));
                }
                batch.push(ctl::Step::Run(ctl::Call::new(&["account", "list", "--json"])));
                self.stage = Stage::BeforeAdd;
            }
            Step::StopWorker { name } => {
                if *name != "ws" {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", name, "ws"));
                }
                batch.push(ctl::Step::Run(ctl::Call::new(&["account", "list", "--json"])));
                self.stage = Stage::BeforeStop;
            }
            Step::StartWorker { name } => {
                if *name != "ws" {
                    return Err(format!("assertion `left == right` failed\n  left: {:?}\n right: {:?}", name, "ws"));
                }
                if !self.members.ended(name) {
                    return Err(format!("{name} stayed ended through the chat's ending"));
                }
                if Some(self.members.linked(name)) != self.stopped_links {
                    return Err(format!(
                        "{name} stayed unlinked\n  left: {:?}\n right: {:?}",
                        Some(self.members.linked(name)),
                        self.stopped_links
                    ));
                }
                self.stopped_links = None;
                let target = self.members.linked(name) + 1;
                self.members.restart(name);
                self.stage = Stage::Starting { target };
            }
            Step::EndsLeaving(_) => return Err("EndsLeaving requires the real-loop completion judge".into()),
            Step::FixtureReady => return Err("FixtureReady unsupported".into()),
            Step::Advance => return Err("Advance unsupported".into()),
            Step::Push { .. } => return Err("Push unsupported".into()),
            Step::ReleaseHold { .. } => return Err("ReleaseHold unsupported".into()),
            Step::StopEngine | Step::StartEngine { .. } | Step::WorkerCancelled { .. } => {
                return Err("engine restart steps unsupported".into());
            }
            Step::Probe { .. } => return Err("Probe unsupported".into()),
            Step::Release { .. } => return Err("Release unsupported".into()),
            Step::Contended { .. } => return Err("Contended unsupported".into()),
            Step::SameParty { .. } => return Err("SameParty unsupported".into()),
            Step::Held { .. } => return Err("Held unsupported".into()),
            Step::Kill { .. } => return Err("Kill unsupported".into()),
            Step::Watching { call } => {
                let slot = *self.slots.get(*call).ok_or("Watching names a started call")?;
                if batch.ends()[slot].exit.is_some() {
                    return Err(format!("Watching call {call} already ended"));
                }
                // No delivery barrier: ended documents are retained (testing/process-tier.md).
            }
            Step::Terminal(_) => return Err("Terminal unsupported".into()),
            Step::Browse { .. } => return Err("Browse unsupported".into()),
            Step::Authorize => return Err("Authorize unsupported".into()),
            Step::AuthorizeHeld => return Err("AuthorizeHeld unsupported".into()),
            Step::HoldIssuerSocket => return Err("HoldIssuerSocket unsupported".into()),
            Step::IssuerRejected => return Err("IssuerRejected unsupported".into()),
            Step::Return => return Err("Return unsupported".into()),
            Step::ClaudeTurn { .. } => return Err("ClaudeTurn unsupported".into()),
        }
        // The oracle's commands are the acts' verbs (Observed::with_commands).
        self.ran.push(step);

        Ok(())
    }
}
impl Judge {
    fn release_placed(&mut self, now: Time) -> Result<(), String> {
        if matches!(self.stage, Stage::Accounts | Stage::Stop) {
            self.placed_release = None;
            return Ok(());
        }
        if let Some(control) = self.control {
            // The placement read follows s2's start; all three are started by then.
            // Cache the cumulative predicate instead of reading placement on each pass.
            if self.placed_release.is_none() && !["ws", "s1", "s2"].iter().all(|name| self.assigned.contains(*name)) {
                return Ok(());
            }
            release(control)?;
            self.placed_release = Some(now.saturating_add(PLACED_RELEASE_INTERVAL));
        }

        Ok(())
    }
    fn start_llm(&mut self, now: Time) -> Result<(), String> {
        self.mark("forge ready", now);
        let address = self
            .members
            .capture("forge", "address")
            .ok_or("forge address")?
            .parse()
            .map_err(|why| format!("socket: {why:?}"))?;
        let directory = self.members.prepare("llm");
        let path = directory.join("llm.json");
        let mut document = read_document(&path)?;
        document["models"] =
            json!([{"name":"fake","window":1_048_576,"output":8192},{"name":"owner","window":1_048_576,"output":8192}]);
        document["added_keys"] =
            json!([self.client.parent().ok_or("scratch")?.join(temper_real_loop_world::case::SECOND_KEY_FILE)]);
        document["cued"] = encode(&(self.scripts)(address), &(self.rules)());
        if self.cast.len() <= 1 && has_holds(&document["cued"]) {
            return Err("one worker has no placed hold".into());
        }
        if has_holds(&document["cued"]) {
            let [control] = super::free_addresses(&[address, self.hosts]);
            self.control = Some(control);
            document["control"] = json!({"socket":control.to_string()});
        }
        write(&path, &document);
        self.members.up(&["llm"]);
        self.stage = Stage::Llm;

        Ok(())
    }
    fn ws_ready(&mut self, now: Time, root: &mut cluster::Root) -> Result<(), String> {
        self.mark("ws ready", now);
        if self.cast.len() == 1 {
            self.queue(root)?;
            self.stage = Stage::Acts;
        } else {
            self.members.up(&["s1"]);
            self.stage = Stage::S1;
        }

        Ok(())
    }
    fn add_worker(&mut self, root: &mut cluster::Root) -> Result<(), String> {
        let document = call_json(Self::calls(root)?.ends().last().ok_or("before add")?)?;
        self.map_channels(&document)?;
        for name in ["ws", "s1"] {
            if !self.assigned.contains(name) {
                return Err(format!("{name} held a run before s2 started"));
            }
        }
        let target = self.members.linked("s2") + 1;
        self.members.up(&["s2"]);
        self.stage = Stage::Added { target };

        Ok(())
    }
    fn record_placement(&mut self, root: &mut cluster::Root) -> Result<(), String> {
        let batch = Self::calls(root)?;
        for call in &batch.ends()[self.processed..] {
            success(call)?;
            live_channels(&call_json(call)?, &mut self.channels);
        }
        self.processed = batch.ends().len();
        let accounts = call_json(batch.ends().last().ok_or("placement hosts")?)?;
        self.map_channels(&accounts)?;
        if !["ws", "s1", "s2"].iter().all(|name| self.assigned.contains(*name)) {
            return Err(format!(
                "every started worker held a run: assigned {:?}; channels {:?}; placed {:?}; accounts {accounts}",
                self.assigned, self.channels, self.workers
            ));
        }
        self.stage = Stage::Acts;

        Ok(())
    }
    fn finish_acts(&mut self, now: Time, root: &mut cluster::Root) -> Result<(), String> {
        let accounts = Self::calls(root)?.ends().last().ok_or("post-act accounts")?;
        success(accounts)?;
        let accounts = call_json(accounts)?;
        self.collect(&accounts)?;
        self.mark("reads", now);
        Self::calls(root)?.release();
        self.members.stop();
        self.stage = Stage::Stop;

        Ok(())
    }
}
impl Referee<cluster::Root> for Judge {
    fn act(&mut self, now: Time, roots: &mut [cluster::Root]) {
        if self.finding.is_none() {
            self.finding = self.advance_world(now, roots).err();
        }
    }

    fn watch(&mut self, now: Time, roots: &[cluster::Root]) -> Result<(), String> {
        if let Some(finding) = &self.finding {
            return Err(finding.clone());
        }
        let (_, members) = roots.split_last().ok_or("client root")?;
        self.members.observe(now, members);
        self.members.verdict()?;
        if self.members.failed() {
            return Ok(());
        }
        if let Some(stopped) = self.members.stopped() {
            let names = stopped.iter().map(|member| member.name.as_str()).collect::<Vec<_>>();
            let expected = if self.cast.len() == 1 {
                vec!["ws", "engine", "llm", "forge"]
            } else if self.late {
                vec!["s2", "s1", "ws", "engine", "llm", "forge"]
            } else {
                // ws's first life by its act, then the cast in reverse start order: ws's second life last started.
                vec!["ws", "ws", "s2", "s1", "engine", "llm", "forge"]
            };
            if names != expected {
                return Err(format!("assertion `left == right` failed\n  left: {names:?}\n right: {expected:?}"));
            }
            for &worker in &self.cast {
                if self.members.linked(worker)
                    != if self.cast.len() > 1 && !self.late && worker == "ws" { 2 } else { 1 }
                {
                    return Err(format!(
                        "{worker} linked once per life\n  left: {:?}\n right: {:?}",
                        self.members.linked(worker),
                        if self.cast.len() > 1 && !self.late && worker == "ws" { 2 } else { 1 }
                    ));
                }
            }
            for member in stopped {
                if self.cast.contains(&member.name.as_str())
                    && member.stderr.split(|byte| *byte == b'\n').filter(|line| line.starts_with(b"linked ")).count()
                        != 1
                {
                    return Err(format!(
                        "{} life {} linked once\n  left: {:?}\n right: {:?}",
                        member.name,
                        member.life,
                        member.stderr.split(|byte| *byte == b'\n').filter(|line| line.starts_with(b"linked ")).count(),
                        1
                    ));
                }
                if member.exit != Some(Exit::Code(0)) {
                    return Err(format!(
                        "{}: {}\n  left: {:?}\n right: {:?}",
                        member.name,
                        String::from_utf8_lossy(&member.stderr),
                        member.exit,
                        Some(Exit::Code(0))
                    ));
                }
                if member.killed || !member.survivors.is_empty() {
                    return Err("no surviving process".into());
                }
            }
            if !self.stopped {
                for member in ["llm", "forge"] {
                    let bytes = fs::read(self.members.dir(member).join("observations.jsonl"))
                        .map_err(|why| format!("final records: {why:?}"))?;
                    let last = bytes
                        .split(|byte| *byte == b'\n')
                        .rfind(|line| !line.is_empty())
                        .ok_or_else(|| "final checkpoint".to_owned())?;
                    if serde_json::from_slice::<Value>(last).map_err(|why| format!("checkpoint: {why:?}"))?
                        != json!({"observations_lost":0})
                    {
                        return Err(format!(
                            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                            serde_json::from_slice::<Value>(last).map_err(|why| format!("checkpoint: {why:?}"))?,
                            json!({"observations_lost":0})
                        ));
                    }
                }
                if self.late {
                    self.check_workspaces()?;
                } else {
                    self.check_tokens()?;
                }
                self.stopped = true;
                self.mark("stop", now);
                eprintln!("case processes: {:?}", self.phases);
            }
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        Some(self.members.next_deadline().into_iter().chain(self.placed_release).fold(self.deadline, Time::min))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline).then(|| {
            let flight = self
                .ran
                .last()
                .map_or_else(|| "none".into(), |step| format!("{}: {}", self.ran.len() - 1, describe(step)));
            let next = self
                .steps
                .front()
                .map_or_else(|| "none".into(), |step| format!("{}: {}", self.ran.len(), describe(step)));
            let message = format!("case processes waited in {:?}; act in flight {flight}; next act {next}", self.stage);
            eprintln!("{message}");
            message
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.members.verdict().unwrap_or(false)
    }
}
fn describe(step: &calls::Step) -> String {
    use calls::Step;
    match step {
        Step::ClaudeTurn { source } => format!("ClaudeTurn {source}"),
        Step::EndsLeaving(leftovers) => format!("EndsLeaving {leftovers:?}"),
        Step::FixtureReady => "FixtureReady".into(),
        Step::Browse { call } => format!("Browse call {call}"),
        Step::Authorize => "Authorize".into(),
        Step::AuthorizeHeld => "AuthorizeHeld".into(),
        Step::HoldIssuerSocket => "HoldIssuerSocket".into(),
        Step::IssuerRejected => "IssuerRejected".into(),
        Step::Return => "Return".into(),
        Step::Terminal(_) => "Terminal".into(),
        Step::ReleaseHold { name } => format!("ReleaseHold {}", String::from_utf8_lossy(name)),
        Step::Advance => "Advance".into(),
        Step::Push { path, .. } => format!("Push {}", String::from_utf8_lossy(path)),
        Step::AddWorker { name } => format!("AddWorker {name}"),
        Step::StopEngine => "StopEngine".into(),
        Step::StartEngine { .. } => "StartEngine".into(),
        Step::WorkerCancelled { name } => format!("WorkerCancelled {name}"),
        Step::StopWorker { name } => format!("StopWorker {name}"),
        Step::StartWorker { name } => format!("StartWorker {name}"),
        Step::Probe { at } => format!("Probe at {}", at.0),
        Step::Release { at } => format!("Release at {}", at.0),
        Step::Contended { calls } => format!("Contended {calls:?}"),
        Step::SignIn { at } => format!("SignIn at {}", at.0),
        Step::SameParty { at } => format!("SameParty at {}", at.0),
        Step::Held { name } => format!("Held {}", String::from_utf8_lossy(name)),
        Step::Kill { call } => format!("Kill call {call}"),
        Step::Call { at, argv } => format!("Call at {}: {}", at.0, describe_argv(argv)),
        Step::Start { at, argv } => format!("Start at {}: {}", at.0, describe_argv(argv)),
        Step::Join { call } => format!("Join call {call}"),
        Step::Watching { call } => format!("Watching call {call}"),
        Step::Until { at, argv, holds, early } => format!(
            "Until at {}: {}; holds {:?}; early {:?}",
            at.0,
            describe_argv(argv),
            holds,
            String::from_utf8_lossy(early),
        ),
    }
}
fn describe_argv(argv: &calls::Argv) -> String {
    argv.0
        .iter()
        .map(|arg| match arg {
            calls::Arg::Lit(word) => (*word).into(),
            calls::Arg::GithubApp => "github-app".into(),
            calls::Arg::Number(number) => number.to_string(),
            calls::Arg::File(name) => format!("file({name})"),
            calls::Arg::Task(call) => format!("task(call {call})"),
            calls::Arg::Delegate(call) => format!("delegate(call {call})"),
            calls::Arg::Digest(call) => format!("digest(call {call})"),
            calls::Arg::ActivityBefore(call) => format!("activity-before(call {call})"),
            calls::Arg::InboxAfter(call) => format!("inbox-after(call {call})"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn success(call: &Ended) -> Result<(), String> {
    if call.exit != Some(Exit::Code(0)) {
        return Err(format!(
            "call: {} {}\n  left: {:?}\n right: {:?}",
            String::from_utf8_lossy(&call.stderr),
            String::from_utf8_lossy(&call.stdout),
            call.exit,
            Some(Exit::Code(0))
        ));
    }
    Ok(())
}
impl Judge {
    fn collect(&mut self, accounts: &Value) -> Result<(), String> {
        let path = self.members.dir("llm").join("observations.jsonl");
        let mut told = Vec::new();
        let mut checkpoints = 0;
        for line in fs::read(path)
            .map_err(|why| format!("LLM records: {why:?}"))?
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            use temper_fake::llm::observations::Observation;
            match temper_fake::llm::observations::decode(line).map_err(|why| format!("query record: {why:?}"))? {
                Observation::Query(query) => {
                    if let Some(query) = Told::new(&query.system, &query.model, query.credential, &query.messages) {
                        told.push(query);
                    }
                }
                Observation::Checkpoint { lost } => {
                    if lost != 0 {
                        return Err(format!("lossless query stream\n  left: {:?}\n right: {:?}", lost, 0));
                    }
                    checkpoints += 1;
                }
                Observation::Full { .. } => return Err("LLM record full".into()),
            }
        }
        if checkpoints == 0 {
            return Err("query checkpoint".into());
        }
        let rows: Vec<Value> = fs::read_to_string(self.members.dir("forge").join("observations.jsonl"))
            .map_err(|why| format!("forge records: {why:?}"))?
            .lines()
            .map(|line| serde_json::from_str(line).map_err(|why| format!("forge JSON: {why:?}")))
            .collect::<Result<_, _>>()?;
        if rows.last().ok_or("forge checkpoint")?["observations_lost"] != 0 {
            return Err(format!(
                "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                rows.last().ok_or("forge checkpoint")?["observations_lost"],
                0
            ));
        }
        let kit = if self.late {
            let merged = rows
                .iter()
                .rev()
                .find(|row| row["observation"] == "merged" && row["repository"] == "acme/app")
                .ok_or("kit merged")?;
            let head = merged["head"].as_u64().ok_or("merged head")?;
            let merge = merged["commit"].as_u64().ok_or("merge commit")?;
            let mut checks = BTreeMap::new();
            for row in &rows {
                if std::ptr::eq(row, merged) {
                    break;
                }
                if row["observation"] == "reported" && row["repository"] == "acme/app" && row["commit"] == head {
                    checks.insert(row["context"].as_str().ok_or("check context")?, row["state"] == "passed");
                }
            }
            let green = !checks.is_empty() && checks.values().all(|passed| *passed);
            let address = self
                .members
                .capture("forge", "address")
                .ok_or("forge address")?
                .parse()
                .map_err(|why| format!("socket: {why:?}"))?;
            let objects = super::github::objects(address, merge);
            let object = objects["objects"]
                .as_array()
                .ok_or("objects")?
                .iter()
                .find(|object| object["commit"] == merge)
                .ok_or("merge object")?;
            let main = object["tree"]
                .as_array()
                .ok_or("main tree")?
                .iter()
                .map(|file| Ok((decode(&file["path"])?, decode(&file["content"])?)))
                .collect::<Result<_, String>>()?;
            Kit { records: Vec::new(), main, merged_green: green, pulls: Vec::new() }
        } else {
            if rows.iter().any(|row| row["observation"] == "merged") {
                return Err("restart story merges nothing".into());
            }
            Kit { records: Vec::new(), main: BTreeMap::new(), merged_green: false, pulls: Vec::new() }
        };
        for call in &self.calls {
            success(call)?;
        }
        if !self.placed.is_empty() {
            self.map_channels(accounts)?;
        }
        let workers = std::mem::take(&mut self.workers);
        *self.run.borrow_mut() = Some(Run {
            calls: std::mem::take(&mut self.calls),
            observed_at: std::mem::take(&mut self.observed_at),
            told,
            kit,
            workers,
            steps: std::mem::take(&mut self.ran),
            first_workers: std::mem::take(&mut self.first_workers),
        });

        Ok(())
    }
}
impl Judge {
    fn check_tokens(&self) -> Result<(), String> {
        let token = &self.admin_token;
        if token.is_empty() {
            return Err(format!("assertion `left != right` failed\n  left: {:?}\n right: {:?}", (token).len(), 0));
        }
        for &name in &self.cast {
            let dir = self.members.dir(name);
            let worker = read_document(&dir.join("config/temper.json"))?;
            let secret = worker["workers"][0]["secret"].as_str().ok_or_else(|| "link secret".to_owned())?;
            for path in [dir.join("config/temper.json"), dir.join("config/agent.json"), dir.join("data").join(secret)] {
                let bytes = fs::read(&path).map_err(|why| format!("worker input: {why:?}"))?;
                if bytes.windows(token.len()).any(|window| window == token) {
                    return Err(format!("{name}: forge admin token in {}", path.display()));
                }
            }
        }

        Ok(())
    }
    fn check_workspaces(&self) -> Result<(), String> {
        use temper_real_loop_world::case;
        let workspaces = |worker| -> Result<PathBuf, String> {
            let directory = self.members.dir(worker);
            let document = read_document(&directory.join("config/temper.json"))?;
            let root = document["application"]["workers"][0]["root"].as_str().ok_or("worker root")?;
            Ok(directory.join("data").join(root).join("workspaces"))
        };
        if fs::read_dir(workspaces("ws")?).map_err(|why| format!("ws workspaces: {why:?}"))?.next().is_some() {
            return Err("ws holds no workspace".into());
        }
        let directories = fs::read_dir(workspaces("s1")?).map_err(|why| format!("s1 workspaces: {why:?}"))?;
        for entry in directories {
            let path = entry.map_err(|why| format!("workspace: {why:?}"))?.path().join("app/docs/design/export.md");
            if fs::read(path).is_ok_and(|words| words == case::DESIGN_WORDS.as_bytes()) {
                return Ok(());
            }
        }
        Err("s1's kit workspace holds the design".into())
    }
    fn map_channels(&mut self, accounts: &Value) -> Result<(), String> {
        let deployment = accounts
            .pointer("/accounts/deployment_facts/program_hosts")
            .and_then(Value::as_array)
            .ok_or("worker channels")?;
        let party =
            accounts.pointer("/accounts/program_hosts").and_then(Value::as_array).ok_or("party worker channels")?;
        // Both lists are supplied host facts in this one read; rendering may cut the later list.
        for (task, channel) in std::mem::take(&mut self.channels) {
            if let Some(host) = deployment.iter().chain(party).find(|host| host["channel"] == channel) {
                let name = host["name"].as_str().ok_or("worker name")?;
                self.assigned.insert(name.into());
                if self.placed.contains(&task) {
                    self.workers.insert(task, name.into());
                }
            } else {
                self.channels.insert(task, channel);
            }
        }
        Ok(())
    }
}

fn decode(value: &Value) -> Result<Vec<u8>, String> {
    Ok(base64::decode(
        base64::Alphabet::Standard,
        value.as_str().ok_or("base64")?.as_bytes(),
        temper_fake::forge::limits().limits.content_bytes,
    )
    .map_err(|why| format!("file bytes: {why:?}"))?
    .into_vec())
}
fn live_channels(value: &Value, channels: &mut BTreeMap<u64, u64>) {
    if let (Some(task), Some(channel)) = (
        value.get("task").and_then(Value::as_u64),
        value.pointer("/details/owner/details/observation/channel").and_then(Value::as_u64),
    ) {
        channels.insert(task, channel);
    }
    for child in
        value.as_object().into_iter().flat_map(serde_json::Map::values).chain(value.as_array().into_iter().flatten())
    {
        live_channels(child, channels);
    }
}

fn has_holds(value: &Value) -> bool {
    value.get("hold").is_some_and(|hold| !hold.is_null())
        || value
            .as_object()
            .into_iter()
            .flat_map(serde_json::Map::values)
            .chain(value.as_array().into_iter().flatten())
            .any(has_holds)
}
fn release(address: SocketAddr) -> Result<(), String> {
    let within = skein_lib::Duration::from_secs(10);
    let mut socket = Socket::connect("PLACED control connect", within, address, Scope::World)
        .map_err(|why| format!("PLACED control connect: {why:?}"))?;
    let name = text(temper_real_loop_world::cluster::PLACED);
    let request =
        format!("POST /release/{name} HTTP/1.1\r\nHost: {address}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    socket
        .write_all("the PLACED release request", within, request.as_bytes())
        .map_err(|why| format!("PLACED release: {why:?}"))?;
    let response = socket
        .read_to_end("the PLACED release reply", within, 4096)
        .map_err(|why| format!("PLACED response: {why:?}"))?;
    if !(response.starts_with(b"HTTP/1.1 204 ")) {
        return Err(format!("PLACED release: {}", String::from_utf8_lossy(&response)));
    }
    Ok(())
}

// The repository owner grants the fixture bot access to the existing repository.
fn permit_repository(address: SocketAddr, token: &[u8]) {
    let within = skein_lib::Duration::from_secs(10);
    let mut socket = Socket::connect("fixture forge", within, address, Scope::World).expect("fixture forge");
    let body = br#"{"permission":"write"}"#;
    let request = format!(
        "PUT /api/v1/repos/acme/app/collaborators/temper HTTP/1.1\r\nHost: {address}\r\nAuthorization: token {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        text(token),
        body.len()
    );
    socket.write_all("the fixture collaborator request", within, request.as_bytes()).expect("collaborator head");
    socket.write_all("the fixture collaborator body", within, body).expect("collaborator body");
    let response = socket.read_to_end("the fixture collaborator reply", within, 65_537).expect("collaborator response");
    assert!(response.len() <= 65_536);
    assert!(response.starts_with(b"HTTP/1.1 204 "), "fixture bot collaborator: {}", String::from_utf8_lossy(&response));
}

// The managed fake is provisioned outside the service, then kept as ordinary records.
fn forge_request(address: SocketAddr, token: &[u8], method: &str, path: &str, body: &Value, status: u16) -> Value {
    let within = skein_lib::Duration::from_secs(10);
    let mut socket = Socket::connect("fixture forge", within, address, Scope::World).expect("fixture forge");
    let body = serde_json::to_vec(body).expect("fixture request JSON");
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nAuthorization: token {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        text(token),
        body.len()
    );
    socket.write_all("fixture request", within, request.as_bytes()).expect("request head");
    socket.write_all("fixture body", within, &body).expect("request body");
    let response = socket.read_to_end("fixture response", within, 65_537).expect("response");
    assert!(response.len() <= 65_536);
    assert!(response.starts_with(format!("HTTP/1.1 {status} ").as_bytes()), "fixture {method} {path} status");
    let body = response.windows(4).position(|part| part == b"\r\n\r\n").expect("response head") + 4;
    serde_json::from_slice(&response[body..]).expect("fixture response JSON")
}

fn seed_forge(directory: &Path, forge: SocketAddr, admin: &[u8]) {
    let bot = forge_request(
        forge,
        admin,
        "POST",
        "/api/v1/admin/users",
        &json!({"username":"temper","email":"temper@example.test","password":"case-fixture-password","full_name":"temper:bot","must_change_password":false}),
        201,
    );
    let token = forge_request(
        forge,
        admin,
        "POST",
        "/api/v1/admin/users/temper/tokens",
        &json!({"name":"case","scopes":["all"]}),
        201,
    );
    let mut record = temper_real_loop_world::fixtures::forgejo_record(forge);
    record.bot.login = b"temper".as_slice().into();
    record.mode = temper_engine_domain_forge::Mode::Managed;
    temper_real_loop_world::fixtures::seed_forgejo(
        &directory.join("state"),
        record,
        bot["id"].as_u64().expect("fixture bot id"),
        token["sha1"].as_str().expect("fixture bot token").as_bytes(),
        temper_real_loop_world::forge::HOOK_SECRET,
        Some(admin),
    );
    permit_repository(forge, admin);
}
