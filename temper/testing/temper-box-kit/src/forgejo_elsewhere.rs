//! A second deployment connects to the owned fixture solely as a Forgejo elsewhere.
//! Its no-review rule replaces the adopted project's stock rule
//! (temper/docs/design/protocol/configuration/settings.md, 3 and temper/docs/design/protocol/configuration/stock.md).
use super::{Forge, Process, Scratch, ctl, forgejo::Throwaway, write_private};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
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
    os::{
        linux::net::SocketAddrExt,
        unix::{
            ffi::OsStrExt,
            fs::PermissionsExt,
            net::{SocketAddr, UnixListener, UnixStream},
        },
    },
    path::{Path, PathBuf},
};

const STORY: &str = "a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart";
const EDIT: &[u8] = b"Changed by the elsewhere chat.\n";

#[expect(
    clippy::too_many_lines,
    reason = "one process story keeps adoption, deleted hooks and restart observations together"
)]
pub fn restart() {
    if let Some(root) = temper_test_env::value(&temper_test_env::E4_LIFE) {
        life(Path::new(&root), temper_test_env::value(&temper_test_env::E4_RESTART).is_some());
        return;
    }
    let Some(capture) = temper_test_env::value(&temper_test_env::E4_CAPTURE) else {
        let scratch = Scratch::new("forgejo_elsewhere_capture");
        let name = format!("temper-e4-{}", std::process::id());
        let address = SocketAddr::from_abstract_name(name.as_bytes()).expect("short capture address");
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
    let address = SocketAddr::from_abstract_name(capture.as_bytes()).expect("capture address");
    let mut capture = UnixStream::connect_addr(&address).expect("private capture witness");
    let clock = Clock::new();
    let began = clock.now().now;
    let elsewhere = Scratch::new("forgejo_elsewhere_host");
    let forge = Throwaway::start_bare(&elsewhere);
    let scratch = Scratch::new("forgejo_elsewhere");
    for name in ["config", "data", "cache"] {
        fs::create_dir(scratch.path().join(name)).expect("fresh root");
        fs::set_permissions(scratch.path().join(name), fs::Permissions::from_mode(0o700)).expect("private root");
    }
    configure(scratch.path(), &forge.forge());
    let paste = scratch.path().join("admin-paste");
    forge.admin_token(&paste);
    let token = fs::read(&paste).expect("private disposable token");
    capture.write_all(&token).expect("private capture witness");
    drop(capture);
    assert_eq!(fs::metadata(&paste).expect("paste metadata").permissions().mode() & 0o777, 0o600);
    // Re-exec captures the support's logs and stage timings too. Neither the
    // child arguments nor its environment contains credential bytes.
    forge.adoption_repositories();
    child(scratch.path(), false, &token);
    inspect(scratch.path(), &token);
    forge.witness_authenticates();
    let pull = forge.merged_pull_in("elsewhere", "first").expect("merged elsewhere pull");
    assert_eq!(forge.job_state_in("elsewhere", "first", pull), "success");
    assert!(
        forge.main_readme_in("elsewhere", "first").windows(EDIT.len()).any(|part| part == EDIT),
        "main holds the chat's edit"
    );
    let commits = forge.repository_observation("elsewhere", "first", "commits?sha=main&limit=50");
    assert_eq!(forge.repositories_in("elsewhere").as_array().expect("repositories").len(), 1);
    assert_eq!(
        forge.repository_observation("elsewhere", "first", "pulls?state=all&limit=50").as_array().expect("pulls").len(),
        1
    );
    let url = hook_url(scratch.path());
    assert_hook(&forge, "elsewhere", "first", &url);
    for name in ["open", "closed"] {
        assert_eq!(
            forge.repository_observation("other", name, "collaborators/elsewhere-bot/permission")["permission"],
            "write"
        );
        assert_hook(&forge, "other", name, &url);
        forge.remove_adoption_collaborator(name);
        forge.remove_adoption_hooks(name);
    }
    fs::remove_file(&paste).expect("delete paste before restart");
    child(scratch.path(), true, &token);
    inspect(scratch.path(), &token);
    forge.witness_authenticates();
    assert!(!paste.exists(), "restart has no paste");
    assert_eq!(forge.merged_pull_in("elsewhere", "first"), Some(pull));
    assert_eq!(forge.repositories_in("elsewhere").as_array().expect("repositories").len(), 1);
    assert_eq!(
        forge.repository_observation("elsewhere", "first", "commits?sha=main&limit=50"),
        commits,
        "restart creates no workflow commit"
    );
    assert_eq!(
        forge.repository_observation("elsewhere", "first", "pulls?state=all&limit=50").as_array().expect("pulls").len(),
        1
    );
    for name in ["open", "closed"] {
        assert_eq!(
            forge.repository_observation("other", name, "collaborators/elsewhere-bot/permission")["permission"],
            "write"
        );
        assert_hook(&forge, "other", name, &url);
    }
    forge.stop();
    scratch.remove();
    elsewhere.remove();
    eprintln!("elsewhere story: {:?}", clock.now().now.saturating_since(began));
}

fn hook_url(root: &Path) -> String {
    let value: Value =
        serde_json::from_slice(&fs::read(root.join("config/temper.json")).expect("configuration")).expect("JSON");
    let socket =
        value.pointer("/application/webhooks/listener/socket").and_then(Value::as_str).expect("listener socket");
    format!("http://{socket}/hooks/1")
}

fn assert_hook(forge: &Throwaway, owner: &str, name: &str, url: &str) {
    let value = forge.repository_observation(owner, name, "hooks");
    let hooks = value.as_array().expect("repository hooks");
    assert_eq!(hooks.len(), 1, "one hook on {owner}/{name}");
    assert_eq!(hooks[0]["config"]["url"], url, "hook at the listener");
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
    super::configuration(&config, forge);
    let Forge::Throwaway { api, git, .. } = forge else { panic!("elsewhere fixture") };
    let path = config.join("temper.json");
    let bytes = fs::read_to_string(&path).expect("configuration").replace("\"acme\"", "\"elsewhere\"");
    let mut value: Value = serde_json::from_str(&bytes).expect("JSON");
    assert!(
        value.pointer("/application/webhooks/listener/socket").and_then(Value::as_str).is_some(),
        "listener configured for the hooks' URL"
    );
    // The first repository and both outside adoptions are retained across restart.
    value["application"]["limits"]["overrides"]["forge.repositories"] = 3.into();
    value["application"]["limits"]["overrides"]["forge.client.repositories"] = 3.into();
    value["engine"]["deployment"]["directory"] = json!(root.join("data"));
    let digest = value["engine"]["hosts"]["workers"][0]["digest"].as_str().expect("worker digest");
    write_private(&root.join("data/worker.digest"), &fs::read(config.join(digest)).expect("fixture digest"));
    value["engine"]["hosts"]["workers"][0]["digest"] = "worker.digest".into();
    fs::rename(config.join("worker.secret"), root.join("data/worker.secret")).expect("data worker secret");
    fs::remove_dir_all(config.join("secrets")).expect("remove obsolete file credentials");
    write_private(&path, &serde_json::to_vec(&value).expect("managed configuration"));
    super::forgejo::bootstrap(&config, &root.join("data"), &root.join("cache"));
    let mut record = temper_real_loop_world::fixtures::forgejo_record(*api);
    record.place.git = git.as_bytes().into();
    record.mode = temper_engine_domain_forge::Mode::Managed;
    record.bot.login = b"elsewhere-bot".as_slice().into();
    record.bot.email = b"elsewhere-bot@example.invalid".as_slice().into();
    temper_real_loop_world::fixtures::seed_record(&root.join("data"), record, &[]);
}

fn scripts(repository: &str, readme: &[u8]) -> Box<[Script]> {
    let call = |name: &[u8], arguments: Value| Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(&arguments).expect("tool JSON").into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let coding = format!("@elsewhere-coder Edit {repository} README.md");
    let original = std::str::from_utf8(readme).expect("README words");
    let edited = format!("{original}\n\n{}", std::str::from_utf8(EDIT).expect("edit words"));
    Box::new([
        Script {
            cue: b"@elsewhere-chat".as_slice().into(),
            turns: Box::new([
                call(b"change", json!({"words":coding,"repository":repository})),
                call(b"wait", json!({})),
                Turn {
                    lines: Box::new([Line::Text { text: b"The change landed.".as_slice().into() }]),
                    finish: Finish::Stop,
                    tokens: 20,
                },
            ]),
        },
        Script {
            cue: coding.into_bytes().into(),
            turns: Box::new([
                call(b"read", json!({"path":"README.md"})),
                call(b"read", json!({"path":".forgejo/workflows/ci.yml"})),
                call(b"edit", json!({"path":"README.md","edits":[{"old":original,"new":edited}]})),
                call(
                    b"finish",
                    json!({"form":"change","fields":{"title":"Update elsewhere README","body":"Changed by the elsewhere chat."}}),
                ),
            ]),
        },
    ])
}

fn setup(root: &Path, restarted: bool) -> Vec<ctl::Step> {
    let mut steps = vec![ctl::Step::Run(ctl::Call::sign_in())];
    if !restarted {
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
        let directory = root.to_path_buf();
        steps.push(ctl::Step::Then(Box::new(move |calls| {
            let file = project_settings(&directory, adopted_project(&calls[1]));
            ctl::settings_change(&calls[3], &file, "elsewhere-landing-settings")
        })));
    }
    let directory = root.to_path_buf();
    steps.push(ctl::Step::Then(Box::new(move |_| {
        let project = landing_project(&directory).to_string();
        ctl::Call::new(&["settings", "show", "landing-rule", "1", "--project", &project, "--json"])
    })));
    steps.push(ctl::Step::Run(ctl::Call::new(&["settings", "show", "forge-charter", "1", "--json"])));
    steps.push(ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "connected:1"])));
    steps.push(ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])));
    steps
}

fn life(root: &Path, restarted: bool) {
    let clock = Clock::new();
    let began = clock.now().now;
    let config = root.join("config");
    let peer =
        (!restarted).then(|| super::fake_llm_played(super::llm::requested(scripts("elsewhere/first", b"# first"))));
    if let Some(peer) = &peer {
        peer.configure(&config);
    }
    let command = Command {
        program: skein_scratch::paths::binary("temper-box"),
        arguments: vec![
            "serve".into(),
            "--config".into(),
            config.into(),
            "--data".into(),
            root.join("data").into(),
            "--cache".into(),
            root.join("cache").into(),
        ],
        directory: root.into(),
        environment: Vec::new(),
    };
    let mut engine = Binary::start(command, Mode::Pipes, 65_536).expect("elsewhere engine");
    let streams = engine.take_streams().expect("engine pipes");
    let fd = skein_shell::open_root(root).expect("story root");
    let mut fds = streams.descriptors();
    fds.push(fd);
    let steps = setup(root, restarted);
    let mut world = real::World::new(Judge {
        failed: None,
        root: root.into(),
        restarted,
        stage: 0,
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
    if let Some(peer) = peer {
        peer.spawn(&mut world);
    }
    let ended = world.run(&clock, Duration::from_secs(250));
    assert!(ended.procs.iter().all(Host::is_empty), "engine, ctl and fake drained");
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
    let task = if restarted { calls.ends().last() } else { calls.ends().get(calls.ends().len() - 3) }
        .expect("landed task read");
    if !restarted {
        for answer in &calls.ends()[calls.ends().len() - 2..] {
            let answer = ctl::json(answer);
            assert_eq!(answer["adopted"]["kinds"]["push"], true, "adoption grants write");
        }
    }
    let task = String::from_utf8_lossy(&task.stdout);
    let (landed, _) = ctl::landed_result(&task);
    assert!(
        landed.starts_with("result: Landed · pull request #1 · merge commit "),
        "same landing is visible through ctl"
    );
    eprintln!("elsewhere life restarted={restarted}: {:?}", clock.now().now.saturating_since(began));
}

fn landing_settings(root: &Path) -> Value {
    check_landing_settings(root).unwrap_or_else(|why| panic!("{why}"))
}
fn check_landing_settings(root: &Path) -> Result<Value, String> {
    Ok({
        serde_json::from_slice(
            &fs::read(root.join("config/native-landing-settings.json"))
                .map_err(|error| format!("landing settings: {error:?}"))?,
        )
        .map_err(|error| format!("landing settings JSON: {error:?}"))?
    })
}

fn project_settings(root: &Path, project: u64) -> PathBuf {
    let mut settings = landing_settings(root);
    settings["items"][0]["setting"]["project"] = project.into();
    let file = root.join("config/native-landing-settings.json");
    write_private(&file, &serde_json::to_vec(&settings).expect("project landing settings"));
    file
}

fn landing_project(root: &Path) -> u64 {
    check_landing_project(root).unwrap_or_else(|why| panic!("{why}"))
}
fn check_landing_project(root: &Path) -> Result<u64, String> {
    Ok({
        check_landing_settings(root)?["items"][0]["setting"]["project"]
            .as_u64()
            .ok_or(String::from("landing rule's project"))?
    })
}

fn adopted_project(call: &super::Ended) -> u64 {
    check_adopted_project(call).unwrap_or_else(|why| panic!("{why}"))
}
fn check_adopted_project(call: &super::Ended) -> Result<u64, String> {
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

fn check_landing_calls(ends: &[super::Ended], project: u64) -> Result<(), String> {
    let rule = super::check_ctl_json(&ends[ends.len() - 4])?;
    let producer = super::check_ctl_json(&ends[ends.len() - 3])?;
    {
        let (left, right) = (&(rule["setting"]), &(json!({"kind":"landing-rule","project":project,"number":1})));
        if left != right {
            return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
        }
    }
    for criterion in ["enforce", "ci", "up_to_date"] {
        if rule["value"][criterion] != true {
            return Err(format!("{criterion} stays enforced"));
        }
    }
    if rule["value"]["gates"] != json!([]) {
        return Err(String::from("the story supplies no review exchange"));
    }
    if producer["value"] != 2 {
        return Err(String::from("the coding producer is kept across restart"));
    }

    Ok(())
}

fn connected(calls: &mut ctl::Calls, root: &Path, restarted: bool) -> Result<(), String> {
    let ends = calls.ends();
    check_landing_calls(ends, check_landing_project(root)?)?;
    let forges = super::check_ctl_json(ends.last().ok_or(String::from("forges read"))?)?;
    let row = &forges["forges"][0];
    {
        let (left, right) = (&(row["state"]), &("connected"));
        if left != right {
            return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
        }
    }
    {
        let (left, right) = (&(row["mode"]), &("managed"));
        if left != right {
            return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
        }
    }
    if restarted {
        // temper/docs/design/protocol/setup/owned.md, 7: keep the first intent across restarts.
        {
            let (left, right) = (&(row["first"]["state"]), &("made"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(row["first"]["org"]), &("elsewhere"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(row["first"]["repository"]), &("first"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        calls.push(ctl::Step::Run(ctl::Call::new(&["task", "show", "2"])));
    } else {
        {
            let (left, right) = (&(row["first"]["state"]), &("made"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        start_chat(calls, root)?;
    }

    Ok(())
}

fn start_chat(calls: &mut ctl::Calls, root: &Path) -> Result<(), String> {
    let project = check_adopted_project(&calls.ends()[1])?;
    if check_landing_project(root)? != project {
        return Err(String::from("the chat uses the project's no-review rule"));
    }
    let key = root.join("llm-key");
    write_private(&key, jig_channel_kit::fixtures::KEY);
    for call in [
        ctl::Call::new(&[
            "account",
            "add-key",
            "1",
            key.to_str().ok_or(String::from("key path"))?,
            "--key",
            "elsewhere-model",
        ]),
        ctl::Call::new(&["account", "list", "--until", "usable"]),
        ctl::Call::new(&[
            "chat",
            "start",
            "--project",
            &project.to_string(),
            "--key",
            "elsewhere-chat",
            "@elsewhere-chat Land a README change in elsewhere/first",
            "--json",
        ]),
        ctl::Call::new(&["task", "show", "1", "--until", "replied", "--json"]),
        ctl::Call::new(&["task", "show", "2", "--until", "ended", "--json"]),
        ctl::Call::new(&["task", "show", "2"]),
    ] {
        calls.push(ctl::Step::Run(call));
    }
    for (name, key) in [("open", "elsewhere-adopt-open"), ("closed", "elsewhere-adopt-closed")] {
        calls.push(ctl::Step::Run(ctl::Call::new(&[
            "repository",
            "adopt",
            "--project",
            &project.to_string(),
            "--owner",
            "other",
            "--name",
            name,
            "--landing",
            "main",
            "--no-home",
            "--key",
            key,
            "--json",
        ])));
    }

    Ok(())
}

/// Project selection is checked without starting a deployment or its forge.
pub mod tests {
    /// The connect answer selects the rule's project and preserves its criteria and producer.
    pub fn settings_follow_the_connected_project() {
        let scratch = super::Scratch::new("elsewhere_project_rule");
        let config = scratch.path().join("config");
        super::fs::create_dir(&config).expect("fixture configuration");
        super::super::owned::producer_landing_settings(&config, 2);
        let before = super::landing_settings(scratch.path());
        let call = super::super::Ended {
            exit: Some(super::Exit::Code(0)),
            stdout: b"{\"connecting\":{}}\n{\"project\":7,\"outcome\":{\"adopted\":{\"owner\":\"elsewhere\",\"name\":\"first\"}}}\n".to_vec(),
            stderr: Vec::new(),
        };
        super::project_settings(scratch.path(), super::adopted_project(&call));
        let after = super::landing_settings(scratch.path());
        assert_eq!(super::landing_project(scratch.path()), 7, "the rule follows the connect answer");
        assert_eq!(after["items"][0]["set"], before["items"][0]["set"]);
        assert_eq!(after["items"][1], before["items"][1]);
        scratch.remove();
    }
}

struct Judge {
    failed: Option<String>,
    root: PathBuf,
    restarted: bool,
    stage: u8,
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
            if self.stage == 0 {
                connected(calls, &self.root, self.restarted)?;
                self.stage = 1;
            } else if !self.stopped {
                calls.release();
                let Some(Process::Binary(engine)) = procs.first_mut() else { return Err(String::from("engine")) };
                engine.signal(Signal::Terminate);
                if let Some(Process::Peer(peer)) = procs.get_mut(2) {
                    peer.shutdown();
                }
                self.stopped = true;
            }

            Ok(())
        })();
        if let Err(why) = result {
            self.failed = Some(why);
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Process]) -> Result<(), String> {
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
    let excluded = root.join("data/generations/1/secrets");
    let mut pending = vec![root.join("config"), root.join("data"), root.join("cache")];
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
    let mut pending = vec![root.join("data/generations/1/store")];
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
