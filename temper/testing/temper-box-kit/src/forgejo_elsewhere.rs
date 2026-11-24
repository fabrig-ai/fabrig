//! A second deployment connects to the owned fixture solely as a Forgejo elsewhere
//! (temper/docs/design/protocol/setup/testing.md, 4).
use super::{Forge, Process, Scratch, ctl, forgejo::Throwaway, write_private};
use serde_json::{Value, json};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Mode},
    real,
};
use std::{
    fs,
    io::{Read, Write},
    net::SocketAddr,
    os::{
        linux::net::SocketAddrExt,
        unix::{
            ffi::OsStrExt,
            fs::PermissionsExt,
            net::{SocketAddr as UnixAddress, UnixListener, UnixStream},
        },
    },
    path::Path,
};

const STORY: &str =
    "a_forgejo_elsewhere_connected_as_its_admin_makes_its_first_repository_and_is_found_after_a_restart";
const SETTINGS: &str = "config/elsewhere-providers.json";
const CONNECTED: &str = "elsewhere-connected.json";

pub fn restart() {
    if let Some(root) = temper_test_env::value(&temper_test_env::E4_LIFE) {
        life(Path::new(&root), temper_test_env::value(&temper_test_env::E4_RESTART).is_some());
        return;
    }
    let Some(capture) = temper_test_env::value(&temper_test_env::E4_CAPTURE) else {
        let scratch = Scratch::new("forgejo_elsewhere_capture");
        let name = format!("temper-e4-{}", std::process::id());
        let address = UnixAddress::from_abstract_name(name.as_bytes()).expect("short capture address");
        let listener = UnixListener::bind_addr(&address).expect("capture socket");
        let child = std::process::Command::new(std::env::current_exe().expect("process test binary"))
            .args(["--exact", STORY, "--nocapture"])
            .env(temper_test_env::E4_CAPTURE.name, &name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("captured complete story");
        // The child opens this before setup: even setup failure closes the stream.
        let (mut stream, _) = listener.accept().expect("capture child");
        let mut token = Vec::new();
        stream.read_to_end(&mut token).expect("private scan witness");
        let output = child.wait_with_output().expect("complete story output");
        if token.is_empty() {
            assert!(output.status.success(), "story failed before minting a credential (output redacted)");
        } else {
            no_token(&output.stdout, &token);
            no_token(&output.stderr, &token);
            std::io::stdout().write_all(&output.stdout).expect("scanned support stdout");
            std::io::stderr().write_all(&output.stderr).expect("scanned support stderr");
            assert!(output.status.success(), "complete elsewhere story failed");
        }
        scratch.remove();
        return;
    };
    let address = UnixAddress::from_abstract_name(capture.as_bytes()).expect("capture address");
    let mut capture = UnixStream::connect_addr(&address).expect("private capture witness");
    let clock = Clock::new();
    let began = clock.now().now;
    let elsewhere = Scratch::new("forgejo_elsewhere_host");
    let forge = Throwaway::start_without_runner(&elsewhere);
    let scratch = Scratch::new("forgejo_elsewhere");
    for name in ["config", "data", "cache"] {
        fs::create_dir(scratch.path().join(name)).expect("fresh root");
        fs::set_permissions(scratch.path().join(name), fs::Permissions::from_mode(0o700)).expect("private root");
    }
    configure(scratch.path(), &forge.forge());
    // Keep the fixture's TLS endpoint distinct without any model exchange.
    let [endpoint] = super::reserve_listeners(&[]);
    configure_provider(scratch.path(), endpoint.local_addr().expect("unused provider endpoint"));
    let paste = scratch.path().join("admin-paste");
    forge.admin_token(&paste);
    let token = fs::read(&paste).expect("private disposable token");
    capture.write_all(&token).expect("private capture witness");
    drop(capture);
    assert_eq!(fs::metadata(&paste).expect("paste metadata").permissions().mode() & 0o777, 0o600);
    // Re-exec captures the support's logs and stage timings too. Neither the
    // child arguments nor its environment contains credential bytes.
    child(scratch.path(), false, &token);
    inspect(scratch.path(), &token);
    forge.witness_authenticates();
    let repository = observe_repository(&forge);
    let connected: Value = serde_json::from_slice(&fs::read(scratch.path().join(CONNECTED)).expect("connected row"))
        .expect("connected JSON");
    assert_eq!(repository.0, connected["row"]["first"]["id"].as_u64().expect("made repository id"));
    let url = hook_url(scratch.path());
    let hook = assert_hook(&forge, &url);

    fs::remove_file(&paste).expect("delete paste before restart");
    forge.remove_repository_hook("elsewhere", "first", hook);
    assert_eq!(forge.repository_observation("elsewhere", "first", "hooks"), json!([]));
    child(scratch.path(), true, &token);
    inspect(scratch.path(), &token);
    forge.witness_authenticates();
    assert!(!paste.exists(), "restart has no paste");
    assert_eq!(observe_repository(&forge), repository, "same repository, with main unchanged");
    assert_ne!(assert_hook(&forge, &url), hook, "restart made the hook again");
    forge.stop();
    scratch.remove();
    elsewhere.remove();
    eprintln!("elsewhere story: {:?}", clock.now().now.saturating_since(began));
}

fn hook_url(root: &Path) -> String {
    let value: Value = super::start_document(
        &root.join("config").join(temper_config_shell::FILE),
        temper_config_shell::Shape::Standalone,
    );
    let socket =
        value.pointer("/application/webhooks/listener/socket").and_then(Value::as_str).expect("listener socket");
    format!("http://{socket}/hooks/1")
}

fn observe_repository(forge: &Throwaway) -> (u64, Value) {
    let repositories = forge.repositories_in("elsewhere");
    let repositories = repositories.as_array().expect("repository list");
    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0]["full_name"], "elsewhere/first");
    let id = repositories[0]["id"].as_u64().expect("repository id");
    let commits = forge.repository_observation("elsewhere", "first", "commits?sha=main&limit=50");
    assert!(commits[0]["sha"].is_string(), "main has a commit");
    assert_eq!(forge.repository_observation("elsewhere", "first", "pulls?state=all&limit=50"), json!([]));
    (id, commits)
}

fn assert_hook(forge: &Throwaway, url: &str) -> u64 {
    let value = forge.repository_observation("elsewhere", "first", "hooks");
    let hooks = value.as_array().expect("repository hooks");
    assert_eq!(hooks.len(), 1, "one hook on elsewhere/first");
    assert_eq!(hooks[0]["config"]["url"], url, "hook at the listener");
    hooks[0]["id"].as_u64().expect("hook id")
}

fn child(root: &Path, restarted: bool, token: &[u8]) {
    let mut command = std::process::Command::new(std::env::current_exe().expect("process test binary"));
    command.args(["--exact", STORY, "--nocapture"]).env(temper_test_env::E4_LIFE.name, root);
    if restarted {
        command.env(temper_test_env::E4_RESTART.name, "1");
    }
    let output = command.output().expect("captured story life");
    no_token(&output.stdout, token);
    no_token(&output.stderr, token);
    // Only release diagnostics after the redacted byte scan.
    std::io::stdout().write_all(&output.stdout).expect("support stdout");
    std::io::stderr().write_all(&output.stderr).expect("support stderr");
    assert!(output.status.success(), "elsewhere engine life failed");
}

fn configure(root: &Path, forge: &Forge<'_>) {
    let config = root.join("config");
    super::lay_configuration(&config, forge);
    let Forge::Throwaway { api, git, .. } = forge else { panic!("elsewhere fixture") };
    let value = super::limit_settings::configuration(&config);
    assert!(
        value.pointer("/application/webhooks/listener/socket").and_then(Value::as_str).is_some(),
        "listener configured for the hooks' URL"
    );
    fs::rename(config.join("worker.secret"), root.join("data/worker.secret")).expect("data worker secret");
    let copy = root.join("data/agents.setting");
    let agents = fs::read(config.join("agents.setting")).expect("fixture agents setting");
    fs::rename(config.join("agents.setting"), &copy).expect("data agents setting");
    fs::remove_dir_all(config.join("secrets")).expect("remove obsolete file credentials");
    super::limit_settings::remember(&config, &value);
    super::forgejo::bootstrap(&config, &root.join("data"), &root.join("cache"));
    // Bootstrap restores stock settings; the first life still needs the fixture's startup figures.
    write_private(&copy, &agents);
    let mut record = temper_real_loop_world::fixtures::forgejo_record(*api);
    record.place.git = git.as_bytes().into();
    record.mode = temper_engine_domain_forge::Mode::Managed;
    record.bot.login = b"elsewhere-bot".as_slice().into();
    record.bot.email = b"elsewhere-bot@example.invalid".as_slice().into();
    temper_real_loop_world::fixtures::seed_record(&root.join("data/state"), record, &[]);
}

/// Stage the complete fixture provider in the data directory and build its ctl edit therefrom.
pub fn configure_provider(root: &Path, address: SocketAddr) {
    configure_provider_with(
        root,
        address,
        temper_agent_fixtures::providers::read(&root.join("data")),
        temper_service_shell::providers::write,
        write_private,
    );
}

/// Stage the split deployment's provider copies and its provider-only ctl change through supplied sinks.
pub fn configure_provider_with(
    root: &Path,
    address: SocketAddr,
    providers: Value,
    keep: impl FnMut(&Path, &[jig_protocol_config::ProviderSetting]) -> std::io::Result<bool>,
    write: impl FnOnce(&Path, &[u8]),
) {
    let providers = super::llm::rebind_with(&root.join("data"), providers, address, keep);
    let items = super::provider_setup::items(&temper_agent_fixtures::providers::flat(&providers));
    assert!(!items.is_empty(), "the fixture has a provider to keep");
    write(&root.join(SETTINGS), &serde_json::to_vec(&json!({"items":items})).expect("provider settings"));
}

fn provider_edits(root: &Path) -> Vec<Value> {
    let settings: Value =
        serde_json::from_slice(&fs::read(root.join(SETTINGS)).expect("provider edits")).expect("settings JSON");
    settings["items"].as_array().expect("provider items").clone()
}

fn setup(root: &Path, restarted: bool, providers: &[Value]) -> Vec<ctl::Step> {
    let mut steps = Vec::new();
    if restarted {
        let kept: Value =
            serde_json::from_slice(&fs::read(root.join(CONNECTED)).expect("kept project")).expect("connected JSON");
        steps.push(ctl::Step::Run(project_show(kept["project"].as_u64().expect("project number"))));
    } else {
        // The client keeps its sign-in across the restart (client/ctl/command-line.md, 1.1).
        steps.push(ctl::Step::Run(ctl::Call::sign_in()));
        steps.push(ctl::Step::Run(ctl::Call::new(&[
            "forge",
            "connect",
            "1",
            "forgejo-admin",
            root.join("admin-paste").to_str().expect("paste path"),
            "--first",
            "elsewhere/first",
            "--label",
            "temper-host",
            "--key",
            "elsewhere-connect",
            "--json",
        ])));
        steps.push(ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "made:1"])));
        steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "--json"])));
        let file = root.join(SETTINGS);
        steps.push(ctl::Step::Then(Box::new(move |calls| {
            ctl::settings_change(calls.last().expect("settings inventory"), &file, "elsewhere-provider-settings")
        })));
        steps.push(ctl::Step::Then(Box::new(|calls| project_show(adopted_project(&calls[1])))));
    }
    steps.push(ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "connected:1"])));
    steps.push(ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])));
    for edit in providers {
        let number = edit["setting"]["number"].as_u64().expect("provider number").to_string();
        steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "provider", &number, "--json"])));
    }
    steps
}

fn project_show(project: u64) -> ctl::Call {
    ctl::Call::new(&["project", "show", &project.to_string(), "--json"])
}

fn life(root: &Path, restarted: bool) {
    let clock = Clock::new();
    let began = clock.now().now;
    let command = Command {
        program: skein_scratch::paths::binary("temper-box"),
        arguments: vec![
            "serve".into(),
            "--config".into(),
            root.join("config").into(),
            "--data".into(),
            root.join("data").into(),
            "--cache".into(),
            root.join("cache").into(),
        ],
        directory: root.into(),
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
    };
    let mut engine = Binary::start(command, Mode::Pipes, 65_536).expect("elsewhere engine");
    let streams = engine.take_streams().expect("engine pipes");
    let fd = skein_shell::open_root(root).expect("story root");
    let mut fds = streams.descriptors();
    fds.push(fd);
    let providers = provider_edits(root);
    let steps = setup(root, restarted, &providers);
    let mut world = real::World::new(Judge {
        failed: None,
        stopped: false,
        settled: false,
        logged: 0,
        deadline: super::patience::Patience::watching(
            "the elsewhere deployment's life",
            Duration::from_secs(240),
            &engine,
        ),
        timed_out: None,
    });
    world.spawn_with_fds(engine.descriptors(), || Process::Binary(Box::new(engine)));
    world.spawn_with_fds(fds, || {
        let mut calls = ctl::Calls::new(fd, steps, Some(&streams), None);
        calls.hold();
        Process::Calls(Box::new(calls))
    });
    let ended = world.run(&clock, Duration::from_secs(250));
    assert!(ended.procs.iter().all(Host::is_empty), "engine and ctl drained");
    let Process::Binary(engine) = &ended.procs[0] else { panic!("engine") };
    let Process::Calls(calls) = &ended.procs[1] else { panic!("ctl") };
    // The parent captures these raw bytes before releasing any diagnostic.
    let mut output = std::io::stdout();
    output.write_all(engine.stderr()).expect("engine log");
    output.write_all(calls.stdout()).expect("engine stdout");
    for call in calls.ends() {
        output.write_all(&call.stdout).expect("ctl stdout");
        output.write_all(&call.stderr).expect("ctl stderr");
        assert_eq!(call.exit, Some(Exit::Code(0)), "ctl call failed");
    }
    assert_eq!(engine.exit_status(), Some(Exit::Code(0)), "engine stopped");
    connected(root, restarted, calls.ends(), &providers);
    eprintln!("elsewhere life restarted={restarted}: {:?}", clock.now().now.saturating_since(began));
}

fn adopted_project(call: &super::Ended) -> u64 {
    check_adopted_project(call).unwrap_or_else(|why| panic!("{why}"))
}
/// Read the terminal connection answer and verify the adopted repository's identity.
pub fn check_adopted_project(call: &super::Ended) -> Result<u64, String> {
    Ok({
        // Progress and the terminal answer are separate JSON lines, as for setup.
        let answer = call
            .stdout
            .split(|byte| *byte == b'\n')
            .rfind(|line| !line.is_empty())
            .ok_or(String::from("adoption answer"))?;
        let connected: Value = serde_json::from_slice(answer).map_err(|error| format!("adoption JSON: {error:?}"))?;
        {
            let (left, right) = (&(connected["outcome"]["adopted"]["owner"]), &("elsewhere"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(connected["outcome"]["adopted"]["name"]), &("first"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        connected["project"].as_u64().ok_or(String::from("adoption's project number"))?
    })
}

fn connected(root: &Path, restarted: bool, ends: &[super::Ended], providers: &[Value]) {
    let (reads, values) = ends.split_at(ends.len() - providers.len());
    for (answer, edit) in values.iter().zip(providers) {
        let provider = ctl::json(answer);
        assert_eq!(provider["setting"], edit["setting"]);
        assert_eq!(provider["value"], edit["set"], "the whole applied provider is kept");
    }
    let project = ctl::json(&reads[reads.len() - 3]);
    let forges = ctl::json(&reads[reads.len() - 1]);
    let rows = forges["forges"].as_array().expect("forge rows");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row["number"], 1);
    assert_eq!(row["mode"], "managed");
    assert_eq!(row["state"], "connected");
    assert_eq!(row["first"]["state"], "made");
    assert_eq!(row["first"]["org"], "elsewhere");
    assert_eq!(row["first"]["repository"], "first");
    let resources = project["resources"].as_array().expect("adopted project's resources");
    assert!(resources.iter().any(|resource| resource["home"] == true && resource["role"] == "owned"));
    let kept = json!({"project":project["number"],"resources":resources,"row":row});
    let path = root.join(CONNECTED);
    if restarted {
        let before: Value = serde_json::from_slice(&fs::read(path).expect("first connected row")).expect("first JSON");
        assert_eq!(kept, before, "the forge, first repository and adopted project are unchanged");
        let mut log = Vec::new();
        let copy = temper_service_shell::providers::read(&root.join("data/providers.setting"), Box::new([]), &mut log);
        assert!(log.is_empty(), "the production reader decoded the startup copy");
        let copy: Value =
            serde_json::from_slice(&temper_service_shell::providers::render(&copy)).expect("decoded startup providers");
        let values = copy["providers"].as_array().expect("startup providers");
        for edit in providers {
            let found = values.iter().find(|value| value["number"] == edit["setting"]["number"]);
            assert_eq!(found, Some(&edit["set"]), "startup copy has the whole kept provider");
        }
    } else {
        assert_eq!(project["number"], adopted_project(&ends[1]));
        write_private(&path, &serde_json::to_vec(&kept).expect("connected JSON"));
    }
}

pub mod tests {
    /// The connect answer selects the project to read, not a default project or a forge/repository id.
    pub fn reads_the_connected_project() {
        let call = super::super::Ended {
            exit: Some(super::Exit::Code(0)),
            stdout: b"{\"connecting\":{}}\n{\"project\":7,\"outcome\":{\"adopted\":{\"owner\":\"elsewhere\",\"name\":\"first\",\"repository\":42}}}\n".to_vec(),
            stderr: Vec::new(),
        };
        assert_eq!(super::adopted_project(&call), 7, "reads the project selected by the connect answer");
    }
}

struct Judge {
    failed: Option<String>,
    stopped: bool,
    settled: bool,
    logged: usize,
    deadline: super::patience::Patience,
    timed_out: Option<String>,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        if self.failed.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let Some(Process::Binary(engine)) = procs.first() else { return Err(String::from("engine")) };
            self.deadline.watch(engine);
            if !self.settled
                && !procs.iter().all(Host::is_empty)
                && let Some(figures) = self.deadline.failed()
            {
                self.timed_out = Some(format!("elsewhere life deadline: {figures}"));
            }
            if engine.exit_status().is_some() && !self.stopped || self.timed_out.is_some() {
                eprintln!("engine stderr: {}", String::from_utf8_lossy(engine.stderr()));
            }
            if !self.stopped && engine.exit_status().is_some() {
                return Err(String::from("engine ended before ctl completed"));
            }
            let Some(Process::Calls(calls)) = procs.get_mut(1) else { return Err(String::from("ctl")) };
            while self.logged < calls.ends().len() {
                let call = &calls.ends()[self.logged];
                eprintln!(
                    "ctl {} exit {:?}: {} {}",
                    self.logged,
                    call.exit,
                    String::from_utf8_lossy(&call.stdout),
                    String::from_utf8_lossy(&call.stderr)
                );
                if call.exit != Some(Exit::Code(0)) {
                    return Err(format!("elsewhere prerequisite call {} failed", self.logged));
                }
                self.logged += 1;
            }
            if !calls.foreground_idle() {
                return Ok(());
            }
            if !self.stopped {
                calls.release();
                let Some(Process::Binary(engine)) = procs.first_mut() else { return Err(String::from("engine")) };
                engine.signal(Signal::Terminate);
                self.stopped = true;
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Process]) -> Result<(), String> {
        super::ctl::check(procs)?;
        if let Some(why) = self.failed.take() {
            return Err(why);
        }
        if let Some(why) = self.timed_out.take() {
            return Err(why);
        }
        self.settled = procs.iter().all(Host::is_empty);

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.settled { None } else { self.deadline.until() }
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        if self.settled { None } else { self.timed_out.clone() }
    }
    fn passed(&self) -> bool {
        self.failed.is_none() && self.timed_out.is_none() && self.settled
    }
}

fn no_token(bytes: &[u8], token: &[u8]) {
    assert!(!bytes.windows(token.len()).any(|part| part == token), "disposable credential escaped (redacted)");
}
fn inspect(root: &Path, token: &[u8]) {
    // jig's initial generation keeps its secrets separately from its store.
    let excluded = root.join("data/state/generations/1/secrets");
    let mut pending = vec![root.join("config"), root.join("data"), root.join("cache"), root.join(CONNECTED)];
    while let Some(path) = pending.pop() {
        if path == excluded {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).expect("deployment metadata");
        if metadata.is_dir() {
            pending.extend(fs::read_dir(path).expect("deployment entries").map(|entry| entry.expect("entry").path()));
        } else if metadata.is_file() {
            no_token(&fs::read(path).expect("deployment file"), token);
        }
    }
    let mut pending = vec![root.join("data/state/generations/1/store")];
    let mut files = 0;
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).expect("store metadata");
        assert!(!metadata.file_type().is_symlink(), "store has no symlinks");
        let mode = metadata.permissions().mode() & 0o777;
        if metadata.is_dir() {
            assert_eq!(mode, 0o700, "private store directory");
            pending.extend(fs::read_dir(path).expect("store entries").map(|entry| entry.expect("entry").path()));
        } else {
            assert!(metadata.is_file(), "regular store file");
            assert_eq!(mode, 0o600, "private store file");
            files += 1;
        }
    }
    assert!(files > 0, "observed store files");
}
