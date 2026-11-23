//! The process tier's reader of bin/temper-box/tests/fixtures/cluster (protocol/cluster.md, 5).
use super::{StdoutReader, WITHIN, patience::Patience};
use serde_json::Value;
use skein_io::kernel::{Complete, Exit, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Expectation, Method, Mode, Streams},
    real,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::DirBuilderExt,
    path::{Component, Path, PathBuf},
};

// protocol/cluster.md, 5: the retained cluster's readiness and stop bounds.
const READY_SECONDS: u64 = 120;
const STOP_SECONDS: u64 = 20;
const STDERR_TAIL_LINES: usize = 20;
// The stock worker stop bound and the engine's derived fleet grace (configuration/moved.md, 2).
#[must_use]
pub fn fleet_grace_ms() -> u64 {
    temper_service::startup::worker_fleet_grace(&jig_protocol_config::stock()).as_millis()
}
const CGROUPS: &str = "the cluster's tests need cgroups delegated by the user's service manager (temper docs/design/protocol/cluster.md, 5): run them where it delegates them";

/// bin/temper-box/tests/fixtures/cluster/ read, rendered for one test's scratch, and its processes' state.
pub struct Cluster {
    experiment: PathBuf,
    specs: Vec<Value>,
    bindings: BTreeMap<String, String>,
    prepared: BTreeSet<String>,
}
/// A member's rendered files, with any typed data beside its deployment document.
pub struct Rendered {
    pub files: Vec<(PathBuf, Vec<u8>)>,
    pub start: Option<temper_config_shell::Read>,
    providers: Option<(PathBuf, Box<[jig_protocol_config::ProviderSetting]>)>,
    limits: Option<(PathBuf, temper_playground::documents::Configuration)>,
}
impl Rendered {
    pub fn lay(&self) {
        for (path, bytes) in &self.files {
            private_directory(path.parent().expect("file parent"));
            super::write_private(path, bytes);
        }
        if let Some((path, providers)) = &self.providers {
            temper_agent_fixtures::providers::remember(path, providers);
        }
        if let Some((path, limits)) = &self.limits {
            super::limit_settings::remember(path, limits);
        }
    }
}

/// Compose the engine fixture as the running engine does, without opening it.
#[must_use]
pub fn figures(document: &Value) -> jig_protocol_worker_wire::Figures {
    figures_with_limits(&temper_playground::documents::Configuration::from_jig(document.clone()))
}
#[must_use]
pub fn figures_with_limits(
    document: &temper_playground::documents::Configuration,
) -> jig_protocol_worker_wire::Figures {
    let bytes = serde_json::to_vec(&document["engine"]).expect("engine JSON");
    let parsed = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
    )
    .expect("engine document");
    let providers = document
        .get("world_providers")
        .map_or_else(temper_service::stock::providers, temper_agent_fixtures::providers::values);
    let given = temper_service::stock::Given::Stock.with_providers(providers, Box::new([]));
    let engine = temper_service::startup::engine_over(
        &parsed,
        0,
        &jig_protocol_config::Directories::working(),
        &given,
        &mut document.jig.clone(),
    )
    .expect("fixture engine");
    let own = document.limits;
    let mut figures = temper_engine_service_vocabulary::figures::Composer::new(&engine, &own)
        .compose(&temper_service::stock::agents());
    // The fixture's worker settings each admit one place.
    figures.places = 1;
    figures
}
/// Agent metadata for fixture checks; no worker configuration file is supplied.
#[must_use]
pub fn agent_configuration(engine: &Value, data: &Path) -> Value {
    let bytes = temper_worker_shell::agent::figures::configuration(&figures(engine), None, data, &[])
        .expect("written fixture agent");
    serde_json::from_slice(&bytes).expect("agent JSON")
}

/// The first start records revision 1; apply the fixture's whole values before its first chat.
#[must_use]
pub fn settings_change(file: &Path) -> super::ctl::Call {
    super::ctl::Call::new(&[
        "settings",
        "apply",
        file.to_str().expect("cluster settings FILE path"),
        "--against",
        "1",
        "--key",
        "cluster-settings",
    ])
}
fn private_directory(path: &Path) {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(path).expect("private cluster directory");
}
/// Worker settings from a description, applied by its owner after sign-in.
#[must_use]
pub fn worker_settings(file: &Path) -> Vec<Value> {
    let document: Value =
        serde_json::from_slice(&fs::read(file).expect("description settings")).expect("settings JSON");
    document["items"]
        .as_array()
        .expect("settings items")
        .iter()
        .filter(|item| item["setting"]["kind"] == "worker")
        .cloned()
        .collect()
}
/// The owner admits the fixture's workers through ctl (draft/config.md, 6.7).
#[must_use]
pub fn worker_adds(file: &Path) -> Vec<super::ctl::Call> {
    worker_settings(file)
        .iter()
        .map(|row| {
            let value = &row["set"];
            super::ctl::Call::new(&[
                "worker",
                "add",
                value["name"].as_str().expect("worker name"),
                "--places",
                &value["places"].as_u64().expect("worker places").to_string(),
                "--digest",
                value["digest"].as_str().expect("worker digest"),
                "--key",
                &format!("add-{}", value["name"].as_str().expect("worker name")),
            ])
        })
        .collect()
}
/// Link-only process stories still establish membership through the client's settings door.
pub fn admit_workers(members: &Members, host: &mut Root, directory: &Path) -> Result<(), String> {
    let link = format!("http://{}", members.capture("engine", "link").ok_or("engine link")?);
    let mut client = super::ctl::Calls::linked(&link, directory);
    client.push(super::ctl::Step::Run(super::ctl::Call::sign_in()));
    for call in worker_adds(&members.dir("engine").join("settings.json")) {
        client.push(super::ctl::Step::Run(call));
    }
    let Root::Client { calls, .. } = host else { return Err("client root".into()) };
    *calls = Some(Box::new(client));
    Ok(())
}

fn contained(base: &Path, path: &str) -> PathBuf {
    assert!(
        !path.is_empty() && Path::new(path).components().all(|c| matches!(c, Component::Normal(_))),
        "cluster path: {path}"
    );
    base.join(path)
}
fn string(value: &Value) -> &str {
    value.as_str().expect("cluster string")
}
fn list(value: &Value) -> &[Value] {
    value.as_array().expect("cluster list")
}
impl Cluster {
    /// Read the description and require delegated cgroups before any member starts.
    #[must_use]
    pub fn read(scratch: &Path) -> Self {
        probe(scratch);
        Self::lay_description_from(scratch, &Self::fixture())
    }
    /// Read a story's own fixture-format description, with the same cgroup preflight.
    #[must_use]
    pub fn read_from(scratch: &Path, directory: &Path) -> Self {
        probe(scratch);
        Self::lay_description_from(scratch, directory)
    }
    /// Reuse the GitHub story's templates with a relay in place of its forge process.
    #[must_use]
    pub(crate) fn github_relay(scratch: &Path, git: std::net::SocketAddr) -> Self {
        let directory = skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/github");
        let mut cluster = Self::read_from(scratch, &directory);
        cluster.specs.retain(|spec| spec["name"] != "forge");
        for spec in &mut cluster.specs {
            if let Some(after) = spec["after"].as_array_mut() {
                after.retain(|name| name != "forge");
            }
        }
        cluster.bind("proc.forge.ready.address", &git.to_string());
        cluster
    }
    fn fixture() -> PathBuf {
        skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/cluster")
    }

    /// Compose a cluster description from its template with unopened artifact paths.
    #[must_use]
    pub fn description_from(scratch: &Path, directory: &Path) -> Self {
        Self::description_with_binary(scratch, directory, Path::new("/fixture/temper-box"))
    }

    /// Lay the native description's process directories and fake git link, without a cgroup probe.
    #[must_use]
    pub fn lay_description_from(scratch: &Path, directory: &Path) -> Self {
        let binary = skein_scratch::paths::binary("temper-box");
        let cluster = Self::description_with_binary(scratch, directory, &binary);
        std::os::unix::fs::symlink(binary, scratch.join("git")).expect("fake git link");
        for spec in &cluster.specs {
            private_directory(&cluster.dir(string(&spec["name"])));
        }
        cluster
    }

    fn description_with_binary(scratch: &Path, directory: &Path, binary: &Path) -> Self {
        let experiment = directory.canonicalize().expect("description directory");
        let value: Value =
            serde_json::from_slice(&fs::read(experiment.join("experiment.json")).expect("cluster description"))
                .expect("cluster JSON");
        assert_eq!(value["format"], "lab-experiment/1");
        assert_eq!(value["cost"], "temper-m1-calibration");
        assert!(list(&value["dirs"]).is_empty(), "cluster has no run directories");
        let specs = list(&value["processes"]).to_vec();
        let git = scratch.join("git");
        let mut bindings = BTreeMap::from([
            ("artifact.git".into(), git.to_str().expect("git path").into()),
            ("artifact.temper".into(), binary.display().to_string()),
            ("experiment".into(), experiment.to_str().expect("experiment path").into()),
            ("artifact.temper-engine".into(), binary.with_file_name("temper-engine").display().to_string()),
            ("artifact.temper-worker".into(), binary.with_file_name("temper-worker").display().to_string()),
        ]);
        for spec in &specs {
            let name = string(&spec["name"]);
            let dir = contained(scratch, name);
            assert!(
                bindings.insert(format!("proc.{name}.dir"), dir.to_str().expect("process path").into()).is_none(),
                "duplicate process {name}"
            );
            assert!(
                spec["memory_bytes"].is_null() || spec["memory_bytes"].as_u64().is_some_and(|n| n > 0),
                "{name}'s memory bound"
            );
        }
        let mut cluster = Self { experiment, specs, bindings, prepared: BTreeSet::new() };
        for index in 0..cluster.specs.len() {
            if list(&cluster.specs[index]["ready"])
                .iter()
                .any(|ready| ready["prefix"].as_str().is_some_and(|prefix| prefix.starts_with("linked ")))
            {
                let name = string(&cluster.specs[index]["name"]).to_owned();
                cluster.git_on_path(&name, scratch);
            }
        }
        cluster
    }
    /// The deployment's binaries in place of the box's for the engine and the workers,
    /// and temper-agent as each worker's agent (protocol/cluster.md, 5 and 6).
    pub fn deployment_binaries(&mut self) {
        let temper = skein_scratch::paths::binary("temper-box");
        for name in ["temper-engine", "temper-worker", "temper-agent"] {
            let binary = temper.with_file_name(name);
            assert!(binary.is_file(), "build the deployment's binary: cargo build -p {name} --bin {name}");
            self.bind(&format!("artifact.{name}"), binary.to_str().expect("deployment binary path"));
        }
        for spec in &mut self.specs {
            if spec["name"] == "engine" {
                spec["argv"][0] = "{artifact.temper-engine}".into();
            } else if list(&spec["ready"])
                .iter()
                .any(|ready| ready["prefix"].as_str().is_some_and(|prefix| prefix.starts_with("linked ")))
            {
                spec["argv"][0] = "{artifact.temper-worker}".into();
            }
        }
    }
    /// Put the fixture's git first on this worker's search path.
    pub fn git_on_path(&mut self, process: &str, directory: &Path) {
        let index = self.index(process);
        self.specs[index]["env"]["PATH"] = format!("{}:/usr/bin:/bin", directory.display()).into();
    }
    /// Rename one member and its template bindings before any files are rendered.
    pub fn rename(&mut self, member: &str, name: &str) {
        let index = self.index(member);
        let old = self.dir(member);
        let new = old.parent().expect("scratch").join(name);
        fs::rename(&old, &new).expect("rename member directory");
        self.bindings.remove(&format!("proc.{member}.dir"));
        self.bindings.insert(format!("proc.{name}.dir"), new.to_str().expect("member path").into());
        for spec in &mut self.specs {
            if spec["name"] == member {
                spec["name"] = name.into();
                for ready in spec["ready"].as_array_mut().expect("ready captures") {
                    if let Some(prefix) = ready["prefix"].as_str() {
                        ready["prefix"] =
                            prefix.replace(&format!("linked {member} "), &format!("linked {name} ")).into();
                    }
                }
            }
            if let Some(after) = spec.get_mut("after").and_then(Value::as_array_mut) {
                for predecessor in after {
                    if predecessor == member {
                        *predecessor = name.into();
                    }
                }
            }
            // The template sources remain the fixture's files; argv/env bindings use the new member.
            for arg in spec["argv"].as_array_mut().expect("argv") {
                *arg = string(arg).replace(&format!("proc.{member}."), &format!("proc.{name}.")).into();
            }
        }
        // The fixture's source templates keep their old member placeholders.
        self.bindings.insert(format!("proc.{member}.dir"), new.to_str().expect("member path").into());
        assert_eq!(self.specs[index]["name"], name);
    }
    /// Copy one member's description under a new name before reserving world roots.
    /// Source templates stay the same; the story shapes the copied documents after preparation.
    pub fn copy(&mut self, member: &str, name: &str) {
        assert!(!self.specs.iter().any(|spec| spec["name"] == name), "duplicate process {name}");
        let mut spec = self.specs[self.index(member)].clone();
        let directory = self.dir(member).parent().expect("scratch").join(name);
        private_directory(&directory);
        assert!(
            self.bindings.insert(format!("proc.{name}.dir"), directory.to_str().expect("member path").into()).is_none(),
            "duplicate binding {name}"
        );
        spec["name"] = name.into();
        for arg in spec["argv"].as_array_mut().expect("argv") {
            *arg = string(arg).replace(&format!("proc.{member}."), &format!("proc.{name}.")).into();
        }
        for ready in spec["ready"].as_array_mut().expect("ready captures") {
            if let Some(prefix) = ready["prefix"].as_str() {
                ready["prefix"] = prefix.replace(&format!("linked {member} "), &format!("linked {name} ")).into();
            }
        }
        self.specs.push(spec);
    }
    /// Prepare files once; starting this member retains the story's edits.
    pub fn prepare(&mut self, process: &str) -> PathBuf {
        assert!(!self.prepared.contains(process), "prepare once");
        self.lay_files(process);
        self.prepared.insert(process.into());
        self.dir(process)
    }
    /// Opt in to a member's query stream without changing other stories' argv.
    pub fn record_llm(&mut self) {
        let i = self.index("llm");
        self.specs[i]["argv"]
            .as_array_mut()
            .expect("argv")
            .extend([Value::from("--observations"), Value::from("{proc.llm.dir}/observations.jsonl")]);
    }
    /// A story-owned template binding, fixed before rendering any document.
    pub fn bind(&mut self, name: &str, value: &str) {
        assert!(self.bindings.insert(name.into(), value.into()).is_none(), "duplicate binding {name}");
    }
    /// The study's settings FILE; the native client applies it after owner sign-in.
    #[must_use]
    pub fn settings_file(&self) -> PathBuf {
        self.dir("engine").join("settings.json")
    }
    /// The process's directory, `{proc.P.dir}`.
    #[must_use]
    pub fn dir(&self, process: &str) -> PathBuf {
        self.bindings.get(&format!("proc.{process}.dir")).expect("named process directory").into()
    }
    fn index(&self, name: &str) -> usize {
        self.specs.iter().position(|s| s["name"] == name).unwrap_or_else(|| panic!("unknown cluster process {name}"))
    }
    fn render(&self, text: &str) -> String {
        let mut chars = text.chars().peekable();
        let mut out = String::new();
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.peek() == Some(&'{') => {
                    chars.next();
                    out.push('{');
                }
                '}' if chars.peek() == Some(&'}') => {
                    chars.next();
                    out.push('}');
                }
                '{' => {
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) if c != '{' => name.push(c),
                            _ => panic!("malformed cluster binding {{{name}"),
                        }
                    }
                    out.push_str(
                        self.bindings
                            .get(&name)
                            .unwrap_or_else(|| panic!("unknown or unavailable cluster binding {{{name}}}")),
                    );
                }
                '}' => panic!("unescaped cluster brace"),
                c => out.push(c),
            }
        }
        out
    }
    /// The member's files and start document, with no provider key and no scratch writes.
    #[must_use]
    pub fn render_files(&self, process: &str) -> Rendered {
        self.render_files_with_key(process, false)
    }

    fn render_files_with_key(&self, process: &str, provider_key: bool) -> Rendered {
        let spec = &self.specs[self.index(process)];
        let mut rendered = Rendered { files: Vec::new(), start: None, providers: None, limits: None };
        for file in spec.get("files").map_or(&[][..], list) {
            let source = contained(&self.experiment, string(&file["from"]));
            let target = contained(&self.dir(process), string(&file["path"]));
            let bytes = self.render(&fs::read_to_string(&source).expect("cluster template")).into_bytes();
            let bytes = if target.file_name().is_some_and(|name| name == temper_config_shell::FILE) {
                let shape = if process == "engine" {
                    temper_config_shell::Shape::EngineAlone
                } else {
                    temper_config_shell::Shape::WorkerAlone
                };
                let facts =
                    temper_config_shell::Facts { login: "ada".into(), hostname: process.into(), provider_key: false };
                let mut file = temper_config_shell::parse(&bytes, shape, &facts).expect("cluster configuration").file;
                if shape == temper_config_shell::Shape::WorkerAlone {
                    file.name = Some(process.into());
                }
                let bytes = temper_config_shell::render(&file, shape);
                rendered.start = Some(super::configuration::parse_with_key(&bytes, shape, provider_key));
                bytes.into_vec()
            } else {
                bytes
            };
            if file["path"] == "data/providers.setting" {
                rendered.providers = Some((
                    target.parent().expect("provider data directory").into(),
                    temper_agent_fixtures::providers::parse(&bytes),
                ));
            }
            rendered.files.push((target, bytes));
        }
        if process == "engine" {
            let config = self.dir(process).join("config");
            let start = rendered.start.as_ref().expect("engine start");
            let mut value = temper_playground::documents::Configuration::from_jig(
                serde_json::from_slice(&start.document).expect("engine start document"),
            );
            super::configuration::cluster_limits(
                &mut value,
                self.experiment.ends_with("cluster-turn"),
                self.experiment.ends_with("github"),
            );
            rendered.limits = Some((config, value));
            if let Some(settings) = self.render_settings() {
                rendered.files.push((self.settings_file(), settings));
            }
        }
        rendered
    }

    fn render_settings(&self) -> Option<Vec<u8>> {
        let settings = self.experiment.join("settings.json");
        if !settings.exists() {
            return None;
        }
        let mut value: Value =
            serde_json::from_slice(&fs::read(settings).expect("cluster settings")).expect("settings JSON");
        let address = self.bindings.get("proc.forge.ready.address").expect("forge ready address");
        let mut hex = String::new();
        for byte in address.bytes() {
            use std::fmt::Write as _;
            write!(&mut hex, "{byte:02x}").expect("forge address hex");
        }
        for item in value["items"].as_array_mut().expect("cluster settings items") {
            if item["setting"]["kind"] == "landing-rule" {
                item["set"]["pattern"]["segments"][1] = hex.clone().into();
            }
        }
        Some(serde_json::to_vec(&value).expect("bound settings JSON"))
    }

    fn lay_files(&self, process: &str) {
        let root = self.dir(process);
        let provider_key = root.join("data/provider.key").is_file() || root.join("config/provider.key").is_file();
        self.render_files_with_key(process, provider_key).lay();
    }

    fn command(&self, process: &str) -> Command {
        if !self.prepared.contains(process) {
            self.lay_files(process);
        }
        if process == "engine" && !self.dir(process).join("data/fixture-limits-applied.json").exists() {
            let root = self.dir(process);
            super::limit_settings::install_paths(&[root.join("config"), root.join("data"), root.join("cache")]);
        }
        let spec = &self.specs[self.index(process)];
        let argv: Vec<_> = list(&spec["argv"]).iter().map(|v| self.render(string(v))).collect();
        let (program, arguments) = argv.split_first().expect("cluster argv");
        assert!(Path::new(program).is_absolute(), "absolute cluster executable: {program}");
        Command {
            program: program.into(),
            arguments: arguments.iter().map(Into::into).collect(),
            directory: self.dir(process),
            environment: spec["env"]
                .as_object()
                .expect("cluster environment")
                .iter()
                .map(|(k, v)| (k.into(), self.render(string(v)).into()))
                .collect(),
        }
    }
    /// Render the engine alone for a client story, without starting its peers or rendering workers.
    #[must_use]
    pub fn engine_alone(scratch: &Path) -> PathBuf {
        let mut cluster = Self::lay_description_from(scratch, &Self::fixture());
        // These peers are unused by sign-in and me; the engine still reads its LLM trust file.
        cluster.bind("proc.forge.ready.address", "127.0.0.1:3");
        cluster.bind("proc.llm.ready.address", "127.0.0.1:1");
        super::write_private(&cluster.dir("llm").join("root.der"), skein_tls_kit::pki::ROOT);
        cluster.lay_files("engine");
        cluster.dir("engine")
    }
    /// Compose the cluster with explicit ready captures, without laying files or probing cgroups.
    #[must_use]
    pub fn documents(scratch: &Path, captures: &[(&str, &str, &str)]) -> Self {
        let mut cluster = Self::description_from(scratch, &Self::fixture());
        cluster.ready_captures(captures);
        cluster
    }

    /// Lay the same cluster's engine and workers for a story that needs their files on disk.
    #[must_use]
    pub fn lay_documents(scratch: &Path, captures: &[(&str, &str, &str)]) -> Self {
        let mut cluster = Self::lay_description_from(scratch, &Self::fixture());
        cluster.ready_captures(captures);
        for process in ["engine", "w1", "w2"] {
            cluster.lay_files(process);
        }
        cluster
    }

    fn ready_captures(&mut self, captures: &[(&str, &str, &str)]) {
        for (process, capture, value) in captures {
            self.bind(&format!("proc.{process}.ready.{capture}"), value);
        }
    }
    /// Reserve a world root for each member before running the world.
    #[must_use]
    pub fn members(self) -> (Members, Vec<Root>) {
        self.members_within(WITHIN)
    }
    /// The enclosing world's patience decides whether an inner wait can defer to it.
    #[must_use]
    pub fn members_within(self, world_patience: Duration) -> (Members, Vec<Root>) {
        let count = self.specs.len();
        let hosts = (0..count).map(|_| Root::Process(Box::new(Member::idle()))).collect();
        (
            Members {
                cluster: self,
                world_patience,
                requested: vec![false; count],
                states: (0..count).map(|_| State::Waiting).collect(),
                actions: Vec::new(),
                linked: vec![0; count],
                linked_through: vec![0; count],
                started: Vec::new(),
                stopping: false,
                stopped: Vec::new(),
                failure: None,
                settling_client: false,
            },
            hosts,
        )
    }
}

pub(super) fn check_calls(roots: &[Root]) -> Result<(), String> {
    for root in roots {
        if let Root::Client { calls: Some(calls), .. } = root {
            calls.check()?;
        }
    }
    Ok(())
}

/// A cluster world root: one described process or the story's client calls.
pub enum Root {
    Process(Box<Member>),
    Client { calls: Option<Box<super::ctl::Calls>>, completions: Queue<Complete>, submissions: Queue<Submit> },
}
impl Root {
    #[must_use]
    pub fn client() -> Self {
        Self::Client { calls: None, completions: Queue::with_capacity(0), submissions: Queue::with_capacity(0) }
    }
    fn member(&self) -> &Member {
        let Self::Process(member) = self else { panic!("described process root") };
        member
    }
    fn member_mut(&mut self) -> &mut Member {
        let Self::Process(member) = self else { panic!("described process root") };
        member
    }
    fn host(&self) -> Option<&dyn Host> {
        match self {
            Self::Process(member) => Some(member.as_ref()),
            Self::Client { calls, .. } => calls.as_deref().map(|c| c as &dyn Host),
        }
    }
    fn host_mut(&mut self) -> Option<&mut dyn Host> {
        match self {
            Self::Process(member) => Some(member.as_mut()),
            Self::Client { calls, .. } => calls.as_deref_mut().map(|c| c as &mut dyn Host),
        }
    }
}
impl Host for Root {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if let Some(host) = self.host_mut() {
            host.iterate(now, wall);
        }
    }
    fn drain(&mut self) {
        if let Some(host) = self.host_mut() {
            host.drain();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        match self {
            Self::Process(member) => member.completions(),
            Self::Client { calls: Some(calls), .. } => calls.completions(),
            Self::Client { completions, .. } => completions,
        }
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        match self {
            Self::Process(member) => member.submissions(),
            Self::Client { calls: Some(calls), .. } => calls.submissions(),
            Self::Client { submissions, .. } => submissions,
        }
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().is_some_and(|h| h.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().and_then(Host::next_mechanism_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().and_then(Host::next_policy_deadline)
    }
    fn is_empty(&self) -> bool {
        self.host().is_none_or(Host::is_empty)
    }
    fn operations(&self) -> u32 {
        self.host().map_or(32, Host::operations)
    }
}

/// A world root: waiting, then the described Binary and its output reader, then ended.
pub struct Member {
    command: Option<Command>,
    starting: bool,
    binary: Option<Binary>,
    stdout: Option<StdoutReader>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Member {
    fn idle() -> Self {
        Self {
            command: None,
            starting: false,
            binary: None,
            stdout: None,
            completions: Queue::with_capacity(16),
            submissions: Queue::with_capacity(16),
        }
    }
}
impl Host for Member {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.starting {
            self.starting = false;
            let saved = self.command.as_ref().expect("rendered member command");
            let command = Command {
                program: saved.program.clone(),
                arguments: saved.arguments.clone(),
                environment: saved.environment.clone(),
                directory: saved.directory.clone(),
            };
            let mut binary = Binary::start_with_tree(command, Mode::Pipes, 65_536, Expectation::Supervise, false)
                .expect("cluster member starts");
            let Streams::Pipes { input, output } = binary.take_streams().expect("cluster pipes") else {
                panic!("pipes")
            };
            self.stdout = Some(StdoutReader::new(input, output));
            self.binary = Some(binary);
        }
        if let (Some(binary), Some(stdout)) = (&mut self.binary, &mut self.stdout) {
            // Two children share this root; the low token bit routes their completions.
            while let Some(mut c) = self.completions.pop() {
                let reader = c.op.raw() & 1 == 1;
                c.op = Token::new(c.op.raw() >> 1);
                if reader {
                    stdout.completions().push(c);
                } else {
                    binary.completions().push(c);
                }
            }
            binary.iterate(now, wall);
            stdout.iterate(now, wall);
            for (host, bit) in [(binary as &mut dyn Host, 0), (stdout as &mut dyn Host, 1)] {
                while let Some(mut s) = host.submissions().pop() {
                    s.op = Token::new(s.op.raw().checked_mul(2).expect("member operation token fits") | bit);
                    self.submissions.push(s);
                }
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.starting
            || !self.completions.is_empty()
            || self.binary.as_ref().is_some_and(|b| b.work_pending(now))
            || self.stdout.as_ref().is_some_and(|s| s.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.binary.as_ref().and_then(Host::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        !self.starting
            && self.binary.as_ref().is_none_or(Host::is_empty)
            && self.stdout.as_ref().is_none_or(Host::is_empty)
    }
    fn operations(&self) -> u32 {
        16
    }
}

/// Read the worker's complete startup diagnostic, never a copy of its formula.
pub fn stop_bounds(stderr: &[u8]) -> Result<(Duration, Duration), String> {
    let mut found = None;
    for line in stderr.split_inclusive(|byte| *byte == b'\n') {
        let Some(line) = line.strip_suffix(b"\n").and_then(|line| line.strip_prefix(b"stop bound ")) else {
            continue;
        };
        let line = std::str::from_utf8(line).map_err(|why| format!("stop bound text: {why}"))?;
        let (stop, drain) = line.split_once(" s, drain ").ok_or("stop bound and drain")?;
        let stop = stop.parse::<u64>().map_err(|why| format!("stop seconds: {why}"))?;
        let drain = drain
            .strip_suffix(" s")
            .ok_or("drain seconds")?
            .parse::<u64>()
            .map_err(|why| format!("drain seconds: {why}"))?;
        if stop == 0 || drain == 0 || stop < drain || found.is_some() {
            return Err("one positive stop bound covering its drain".into());
        }
        found = Some((Duration::from_secs(stop), Duration::from_secs(drain)));
    }
    found.ok_or_else(|| "worker's complete stop bound line".into())
}

/// A wait needs a longer world to own a deferred starvation verdict.
enum MemberWait {
    World(Patience),
    Bound(Time),
    Disarmed,
}
impl MemberWait {
    fn arm(what: &'static str, now: Time, within: Duration, world: Duration) -> Self {
        if world > within { Self::World(Patience::arm(what, within)) } else { Self::Bound(now.saturating_add(within)) }
    }
    fn watching(what: &'static str, now: Time, within: Duration, world: Duration, binary: &Binary) -> Self {
        let mut wait = Self::arm(what, now, within, world);
        wait.watch(binary);
        wait
    }
    fn watch(&mut self, binary: &Binary) {
        if let Self::World(deadline) = self {
            deadline.watch(binary);
        }
    }
    fn failed(&mut self, now: Time) -> Option<String> {
        match self {
            Self::World(deadline) => deadline.failed().map(|figures| format!("; {figures}")),
            Self::Bound(until) => (now >= *until).then(String::new),
            Self::Disarmed => None,
        }
    }
    fn until(&self) -> Option<Time> {
        match self {
            Self::World(deadline) => deadline.until(),
            Self::Bound(until) => Some(*until),
            Self::Disarmed => None,
        }
    }
}

enum State {
    Waiting,
    Starting {
        deadline: MemberWait,
        captures: Vec<Option<String>>,
    },
    Ready,
    Stopping {
        deadline: MemberWait,
        ending: Ending,
        unit_timeout: Option<Duration>,
        swept: bool,
        killed: bool,
        survivors: Vec<u32>,
    },
    Ended,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Cluster,
    Alone,
    Kill,
}
enum Action {
    End(Ending, Option<Duration>),
    Signal(Signal),
}
#[derive(Clone, Copy)]
struct Life {
    member: usize,
    number: usize,
}
/// Every cluster story embeds this half in its own referee.
pub struct Members {
    cluster: Cluster,
    world_patience: Duration,
    requested: Vec<bool>,
    states: Vec<State>,
    started: Vec<Life>,
    actions: Vec<(usize, Action)>,
    linked: Vec<usize>,
    linked_through: Vec<usize>,
    stopping: bool,
    stopped: Vec<Stopped>,
    failure: Option<String>,
    settling_client: bool,
}
/// The recorded exit of one member, in order of exit.
pub struct Stopped {
    pub name: String,
    pub life: usize,
    pub ending: Ending,
    pub exit: Option<Exit>,
    pub killed: bool,
    pub survivors: Vec<u32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
impl Members {
    pub fn prepare(&mut self, process: &str) -> PathBuf {
        self.cluster.prepare(process)
    }
    #[must_use]
    pub fn dir(&self, process: &str) -> PathBuf {
        self.cluster.dir(process)
    }
    /// Choose the worker's executable search path before it starts.
    pub fn git_on_path(&mut self, process: &str, directory: &Path) {
        self.cluster.git_on_path(process, directory);
    }
    /// Start these processes and their predecessors, ties in name order.
    pub fn up(&mut self, processes: &[&str]) {
        assert!(!self.stopping, "up before stop");
        for process in processes {
            self.request(self.cluster.index(process), &mut Vec::new());
        }
    }
    fn request(&mut self, index: usize, visiting: &mut Vec<usize>) {
        assert!(!visiting.contains(&index), "cluster after cycle");
        if self.requested[index] {
            return;
        }
        visiting.push(index);
        let after = self.after(index);
        for predecessor in after {
            self.request(predecessor, visiting);
        }
        visiting.pop();
        self.requested[index] = true;
    }
    fn after(&self, index: usize) -> Vec<usize> {
        self.cluster.specs[index]
            .get("after")
            .map_or(&[][..], list)
            .iter()
            .map(|p| self.cluster.index(string(p)))
            .collect()
    }
    #[must_use]
    pub fn is_up(&self) -> bool {
        !self.stopping
            && self.requested.iter().zip(&self.states).all(|(wanted, state)| !wanted || matches!(state, State::Ready))
    }
    #[must_use]
    pub fn capture(&self, process: &str, capture: &str) -> Option<&str> {
        self.cluster.bindings.get(&format!("proc.{process}.ready.{capture}")).map(String::as_str)
    }
    /// Terminate started leaders, one at a time, in reverse start order.
    pub fn stop(&mut self) {
        self.stopping = true;
    }
    #[must_use]
    pub fn stopped(&self) -> Option<&[Stopped]> {
        (self.stopping && !self.settling_client && self.stopped.len() == self.started.len()).then_some(&self.stopped)
    }
    /// Kill the leader, then its remaining tree, recording an intentional ending.
    // skein-world tree.rs:299–323 signals descendants, then the leader, then
    // cgroup.kill. A live worker can report an agent's exit through
    // temper-worker-domain agents.rs:342–367 → jig-domain-host hosted.rs:747–751
    // → temper-engine-domain link.rs:323–332 as Class::Agent. Kill the leader
    // alone (end_to_end.rs:240), await its exit, then sweep (end_to_end.rs:256),
    // so the engine hears a closed link and records Lost instead.
    pub fn kill(&mut self, process: &str) {
        self.end_one(process, Ending::Kill);
    }
    /// Signal every process in the named member's current tree.
    pub fn signal(&mut self, process: &str, signal: Signal) {
        let i = self.cluster.index(process);
        assert!(matches!(self.states[i], State::Ready), "signal a ready member");
        self.actions.push((i, Action::Signal(signal)));
    }
    /// Stop only this leader, escalating tree cleanup after `STOP_SECONDS`.
    pub fn stop_one(&mut self, process: &str) {
        self.end_one(process, Ending::Alone);
    }
    /// Model KillMode=mixed: TERM only the leader, KILL the remaining tree only
    /// after this wall timeout, even if the leader exits leaving descendants.
    pub fn stop_one_within(&mut self, process: &str, timeout: Duration) {
        let i = self.cluster.index(process);
        assert!(matches!(self.states[i], State::Ready), "stop a ready member");
        self.actions.push((i, Action::End(Ending::Alone, Some(timeout))));
    }
    /// The retained observer, including its stderr and the keeper's process identities.
    #[must_use]
    pub fn binary<'a>(&self, process: &str, hosts: &'a [Root]) -> Option<&'a Binary> {
        hosts[self.cluster.index(process)].member().binary.as_ref()
    }
    fn end_one(&mut self, process: &str, ending: Ending) {
        let i = self.cluster.index(process);
        assert!(matches!(self.states[i], State::Ready), "end a ready member");
        self.actions.push((i, Action::End(ending, None)));
    }
    /// Start a settled member again, retaining its command, files and directories.
    pub fn restart(&mut self, process: &str) {
        let i = self.cluster.index(process);
        assert!(!self.stopping && self.ended(process), "restart a settled member before stop");
        self.states[i] = State::Waiting;
        self.cluster.bindings.retain(|key, _| !key.starts_with(&format!("proc.{process}.ready.")));
    }
    #[must_use]
    pub fn ended(&self, process: &str) -> bool {
        matches!(self.states[self.cluster.index(process)], State::Ended)
    }
    /// Complete stderr lines beginning `linked `, across all lives.
    #[must_use]
    pub fn linked(&self, process: &str) -> usize {
        self.linked[self.cluster.index(process)]
    }
    fn ending(&mut self, i: usize, ending: Ending, unit_timeout: Option<Duration>, now: Time, hosts: &mut [Root]) {
        let binary = hosts[i].member_mut().binary.as_mut().expect("started binary");
        if binary.exit_status().is_none() {
            binary.signal(if ending == Ending::Kill { Signal::Kill } else { Signal::Terminate });
        }
        self.states[i] = State::Stopping {
            deadline: match unit_timeout {
                Some(timeout) => MemberWait::Bound(now.saturating_add(timeout)),
                None => MemberWait::watching(
                    "the cluster member's stopping",
                    now,
                    Duration::from_secs(STOP_SECONDS),
                    self.world_patience,
                    binary,
                ),
            },
            ending,
            unit_timeout,
            swept: false,
            killed: false,
            survivors: Vec::new(),
        };
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the member lifecycle keeps its stop, timeout escalation and ordered starts together"
    )]
    pub fn act(&mut self, now: Time, hosts: &mut [Root]) {
        if self.settling_client {
            if let Some(Root::Client { calls: Some(calls), .. }) = hosts.get_mut(self.states.len()) {
                calls.cancel();
                if !calls.is_empty() {
                    return;
                }
            }
            self.settling_client = false;
        }
        for (i, action) in std::mem::take(&mut self.actions) {
            match action {
                Action::End(ending, timeout) => self.ending(i, ending, timeout, now, hosts),
                Action::Signal(signal) => {
                    hosts[i].member_mut().binary.as_mut().expect("live binary").signal_tree(signal);
                }
            }
        }
        for (i, state) in self.states.iter_mut().enumerate() {
            if matches!(state, State::Waiting | State::Ended) {
                continue;
            }
            let binary = hosts[i].member_mut().binary.as_mut().expect("started member iterated");
            if let State::Stopping { ending: Ending::Kill, swept, .. } = state
                && !*swept
                && binary.exit_status().is_some()
            {
                if binary.counts().is_none() {
                    binary.signal_tree(Signal::Kill);
                }
                *swept = true;
            }
            let survivors = binary.tree_status().live_at_leader_exit.as_deref().unwrap_or(&[]);
            let timed_out =
                if let State::Stopping { deadline, killed: false, .. } = state { deadline.failed(now) } else { None };
            if let Some(figures) = &timed_out {
                let seconds = match state {
                    State::Stopping { unit_timeout: Some(timeout), .. } => timeout.as_millis() / 1000,
                    State::Waiting | State::Starting { .. } | State::Ready | State::Stopping { .. } | State::Ended => {
                        STOP_SECONDS
                    }
                };
                self.failure =
                    Some(format!("{} did not stop within {seconds}s{figures}", string(&self.cluster.specs[i]["name"])));
            }
            let timed_out = timed_out.is_some();
            // skein-world end_to_end.rs:253–260 refreshes tree membership on delivery;
            // an intentional whole-tree kill awaits settlement, not leader-exit cleanup.
            let deliberate_kill = matches!(state, State::Stopping { ending: Ending::Kill, .. });
            let unit_stop = matches!(state, State::Stopping { unit_timeout: Some(_), .. });
            let left_tree = !deliberate_kill
                && !unit_stop
                && binary.exit_status().is_some()
                && (!survivors.is_empty() || !binary.tree_status().live_pids.is_empty());
            if (left_tree || timed_out) && !matches!(state, State::Stopping { killed: true, .. }) {
                let ending = match state {
                    State::Stopping { ending, .. } => *ending,
                    State::Waiting | State::Starting { .. } | State::Ready | State::Ended => Ending::Cluster,
                };
                let mut survivors = survivors.to_vec();
                survivors.extend_from_slice(&binary.tree_status().live_pids);
                survivors.sort_unstable();
                survivors.dedup();
                let deadline = MemberWait::Disarmed;
                *state = State::Stopping { deadline, ending, unit_timeout: None, swept: true, killed: true, survivors };
                binary.signal_tree(Signal::Kill);
                self.stopping = true;
            }
        }
        if self.stopping {
            if let Some(life) = self.started.iter().rev().find(|life| !matches!(self.states[life.member], State::Ended))
            {
                let i = life.member;
                if !matches!(self.states[i], State::Stopping { .. }) {
                    self.ending(i, Ending::Cluster, None, now, hosts);
                }
            }
            return;
        }
        let mut pending: Vec<_> = (0..self.states.len())
            .filter(|&i| {
                self.requested[i]
                    && matches!(self.states[i], State::Waiting)
                    && self.after(i).iter().all(|&p| matches!(self.states[p], State::Ready))
            })
            .collect();
        pending.sort_by_key(|&i| string(&self.cluster.specs[i]["name"]));
        for i in pending {
            let name = string(&self.cluster.specs[i]["name"]);
            let host = hosts[i].member_mut();
            if host.command.is_none() {
                host.command = Some(self.cluster.command(name));
            }
            host.binary = None;
            host.stdout = None;
            host.starting = true;
            self.linked_through[i] = 0;
            self.states[i] = State::Starting {
                deadline: MemberWait::arm(
                    "the cluster member's readiness",
                    now,
                    Duration::from_secs(READY_SECONDS),
                    self.world_patience,
                ),
                captures: vec![None; list(&self.cluster.specs[i]["ready"]).len()],
            };
            let number = self.started.iter().filter(|life| life.member == i).count();
            self.started.push(Life { member: i, number });
        }
    }
    pub fn observe(&mut self, now: Time, hosts: &[Root]) {
        for (i, root) in hosts.iter().enumerate() {
            if matches!(self.states[i], State::Waiting | State::Ended) {
                continue;
            }
            let host = root.member();
            let Some(binary) = &host.binary else { continue };
            let name = string(&self.cluster.specs[i]["name"]);
            for line in binary.stderr()[self.linked_through[i]..].split_inclusive(|byte| *byte == b'\n') {
                if !line.ends_with(b"\n") {
                    break;
                }
                self.linked[i] += usize::from(line.starts_with(b"linked "));
                self.linked_through[i] += line.len();
            }
            assert!(!binary.stderr_overflowed(), "{name}'s stderr exceeded its bound");
            if let State::Starting { deadline, captures } = &mut self.states[i] {
                deadline.watch(binary);
                let rules = list(&self.cluster.specs[i]["ready"]);
                for (rule, captured) in rules.iter().zip(captures.iter_mut()) {
                    if captured.is_some() {
                        continue;
                    }
                    let bytes = match string(&rule["stream"]) {
                        "stdout" => &host.stdout.as_ref().expect("stdout").bytes,
                        "stderr" => binary.stderr(),
                        other => panic!("cluster ready stream {other}"),
                    };
                    // split_inclusive leaves a partial last line unmatched.
                    for line in bytes.split_inclusive(|b| *b == b'\n') {
                        if let Some(line) = line.strip_suffix(b"\n") {
                            let line = line.strip_suffix(b"\r").unwrap_or(line);
                            if let Some(suffix) = line.strip_prefix(string(&rule["prefix"]).as_bytes()) {
                                *captured = Some(std::str::from_utf8(suffix).expect("ready text").into());
                                break;
                            }
                        }
                    }
                }
                if captures.iter().all(Option::is_some) {
                    for (rule, captured) in rules.iter().zip(captures.iter()) {
                        if let Some(capture) = rule["capture"].as_str() {
                            self.cluster.bindings.insert(
                                format!("proc.{name}.ready.{capture}"),
                                captured.as_ref().expect("ready capture").clone(),
                            );
                        }
                    }
                    self.states[i] = State::Ready;
                } else if binary.exit_status().is_none()
                    && let Some(figures) = deadline.failed(now)
                {
                    self.failure = Some(format!(
                        "{name} did not become ready within {READY_SECONDS}s: {}{figures}",
                        String::from_utf8_lossy(binary.stderr())
                    ));
                    self.stopping = true;
                }
            }
            if binary.exit_status().is_some() && !matches!(self.states[i], State::Stopping { .. } | State::Ended) {
                let stderr = String::from_utf8_lossy(binary.stderr());
                let tail: Vec<_> = stderr.lines().rev().take(STDERR_TAIL_LINES).collect();
                let tail = tail.into_iter().rev().collect::<Vec<_>>().join("\n");
                self.failure = Some(format!("{name} exited unexpectedly: {:?}\n{tail}", binary.exit_status()));
                self.stopping = true;
                self.settling_client = true;
                self.states[i] = State::Stopping {
                    deadline: MemberWait::watching(
                        "the cluster member's stopping",
                        now,
                        Duration::from_secs(STOP_SECONDS),
                        self.world_patience,
                        binary,
                    ),
                    ending: Ending::Cluster,
                    unit_timeout: None,
                    swept: false,
                    killed: false,
                    survivors: Vec::new(),
                };
            }
            if host.is_empty()
                && let State::Stopping { ending, killed, survivors, .. } = &self.states[i]
            {
                assert_eq!(binary.counts().expect("settled tree").method, Method::Cgroup, "{CGROUPS}");
                self.stopped.push(Stopped {
                    name: name.into(),
                    life: self.started.iter().rev().find(|life| life.member == i).expect("current life").number,
                    ending: *ending,
                    exit: binary.exit_status(),
                    killed: *killed,
                    survivors: survivors.clone(),
                    stdout: host.stdout.as_ref().expect("stdout").bytes.clone(),
                    stderr: binary.stderr().to_vec(),
                });
                self.states[i] = State::Ended;
            }
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.states
            .iter()
            .filter_map(|s| match s {
                State::Starting { deadline, .. } if !self.stopping => deadline.until(),
                State::Stopping { deadline, killed: false, .. } => deadline.until(),
                State::Waiting | State::Starting { .. } | State::Ready | State::Stopping { .. } | State::Ended => None,
            })
            .min()
    }
    #[must_use]
    pub fn overdue(&self, _now: Time) -> Option<String> {
        None
    } // act escalates, then awaits settlement.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.failure.is_some()
    }
    #[must_use]
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
    /// Return the members' finding only once they have all stopped (testing/process-tier.md, the cluster).
    pub fn verdict(&self) -> Result<bool, String> {
        if self.stopped().is_none() {
            return Ok(false);
        }
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        for stopped in &self.stopped {
            if stopped.killed || !stopped.survivors.is_empty() {
                return Err(format!("{} needed tree cleanup", stopped.name));
            }
        }
        Ok(true)
    }
    #[must_use]
    pub fn passed(&self) -> bool {
        self.verdict() == Ok(true)
    }
}

struct Probe {
    settled: bool,
}
impl Referee<Binary> for Probe {
    fn watch(&mut self, _now: Time, hosts: &[Binary]) -> Result<(), String> {
        self.settled = hosts.iter().all(Host::is_empty);
        Ok(())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
fn probe(scratch: &Path) {
    let clock = Clock::new();
    let binary = Binary::start_with_tree(
        Command {
            program: "/bin/true".into(),
            arguments: Vec::new(),
            environment: Vec::new(),
            directory: scratch.into(),
        },
        Mode::Pipes,
        1024,
        Expectation::Supervise,
        false,
    )
    .unwrap_or_else(|why| panic!("{CGROUPS}: {why:?}"));
    let mut world = real::World::new(Probe { settled: false });
    world.spawn_with_fds(binary.descriptors(), || binary);
    let outcome = world.run(&clock, WITHIN);
    let binary = &outcome.procs[0];
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "cgroup probe exits successfully");
    assert_eq!(binary.counts().expect("probe settled").method, Method::Cgroup, "{CGROUPS}");
}

pub mod verdict_tests {
    use super::Cluster;
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::PathBuf,
    };

    pub fn passed_accepts_only_a_successful_settled_verdict() {
        let cluster = Cluster {
            experiment: PathBuf::new(),
            specs: Vec::new(),
            bindings: BTreeMap::new(),
            prepared: BTreeSet::new(),
        };
        let (mut members, _) = cluster.members();
        members.failure = Some("worker ended unexpectedly".to_owned());
        assert_eq!(members.verdict(), Ok(false), "keep a finding until settlement");
        assert!(!members.passed());
        members.stop();
        assert_eq!(members.verdict(), Err("worker ended unexpectedly".to_owned()));
        assert!(!members.passed(), "a settled finding is not a pass");
        members.failure = None;
        assert_eq!(members.verdict(), Ok(true));
        assert!(members.passed());
    }
}

pub mod deadline_tests {
    use super::{MemberWait, stop_bounds};
    use skein_lib::{Duration, Time};

    pub fn stop_bounds_read_the_printed_seconds_and_need_a_complete_line() {
        assert_eq!(
            stop_bounds(b"figures taken\nstop bound 9 s, drain 3 s\nlinked w1\n"),
            Ok((Duration::from_secs(9), Duration::from_secs(3))),
        );
        assert!(stop_bounds(b"stop bound 9 s, drain 3 s").is_err(), "wait for the newline");
    }

    pub fn a_member_wait_can_defer_only_to_a_longer_world() {
        let within = Duration::from_secs(20);
        for (world, can_defer) in [(19, false), (20, false), (21, true)] {
            let wait = MemberWait::arm("the member's answer", Time::ZERO, within, Duration::from_secs(world));
            assert_eq!(matches!(wait, MemberWait::World(_)), can_defer, "world={world}");
            if let MemberWait::Bound(until) = wait {
                assert_eq!(until, Time::ZERO.saturating_add(within));
            }
        }
    }
}
