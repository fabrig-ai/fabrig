//! A newer build at the same paths: temper/docs/design/protocol/setup/box.md, 1 and 5.
//! Linux supplies executable identities and retained process-tree observations.
use crate::support::{
    self, Ended, boxed,
    ctl::{self, Call, Step},
    parent::Guard,
    patience::Patience,
};
use serde_json::{Value, json};
use skein_io::kernel::Exit;
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

const SOURCE: &str = "restart-claude";
const LIVE: &str = "new-build-live";
const MARK: &str = "new-build-page-mark";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Identity {
    device: u64,
    inode: u64,
}
impl Identity {
    fn of(path: &Path) -> Self {
        Self::metadata(&fs::metadata(path).expect("file identity"))
    }

    fn metadata(metadata: &fs::Metadata) -> Self {
        Self { device: metadata.dev(), inode: metadata.ino() }
    }
}

struct Installed {
    scratch: support::Scratch,
    paths: [PathBuf; 3],
    files: Vec<(PathBuf, PathBuf)>,
}
impl Installed {
    fn new() -> Self {
        let scratch = support::Scratch::new("newer-build");
        let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
        for path in &paths {
            super::private(path);
        }
        let source = scratch.path().join("bundle-source");
        super::bundle(&source, super::SOURCE_PAGE);
        let bin = scratch.path().join("bin");
        super::private(&bin);
        super::private(&bin.join("bundle/0000000000000001"));
        let binary = skein_scratch::paths::binary("temper-box");
        let adapter = binary.parent().expect("built binaries").join("fake-claude");
        let mut files = vec![(binary, bin.join("temper-box")), (adapter, bin.join("fake-claude"))];
        for name in ["index.html", "0000000000000001/main.js"] {
            files.push((source.join(name), bin.join("bundle").join(name)));
        }
        let installed = Self { scratch, paths, files };
        installed.copy();
        installed
    }

    fn copy(&self) {
        for (source, target) in &self.files {
            fs::copy(source, target).expect("install build file");
        }
        for name in ["temper-box", "fake-claude"] {
            fs::set_permissions(self.bin().join(name), fs::Permissions::from_mode(0o700)).expect("private executable");
        }
    }

    fn bin(&self) -> PathBuf {
        self.scratch.path().join("bin")
    }

    fn config(&self) -> PathBuf {
        self.paths[1].join("box/fake-backends/config")
    }

    fn flags(&self) -> Vec<String> {
        super::flags(&self.paths[0], &self.paths[1], &self.paths[2], Some(&self.bin().join("bundle")))
    }

    fn probe(&self, old: Vec<Identity>) -> Probe {
        Probe {
            program: self.bin().join("temper-box"),
            adapter: self.bin().join("fake-claude"),
            old,
        }
    }

    /// Keep unlinked files open so the filesystem cannot reuse their old inodes.
    fn replace(&self) -> Vec<File> {
        let kept: Vec<_> = self.files.iter().map(|(_, target)| File::open(target).expect("old build file")).collect();
        let page = format!("<!doctype html><html><head></head><body>{MARK}</body></html>");
        super::write(&self.scratch.path().join("bundle-source/index.html"), page.as_bytes());
        for (_, target) in &self.files {
            fs::remove_file(target).expect("unlink stopped build file");
        }
        self.copy();
        for ((_, target), old) in self.files.iter().zip(&kept) {
            assert_ne!(Identity::of(target), Identity::metadata(&old.metadata().expect("retained old inode")));
        }
        kept
    }
}

#[derive(Clone)]
struct Probe {
    program: PathBuf,
    adapter: PathBuf,
    old: Vec<Identity>,
}
impl Probe {
    fn tree(&self, guard: &Guard) -> Vec<u32> {
        let root = boxed::find(&guard.pids(), &self.program, b"--fake-backends").expect("installed box leader");
        let mut tree = vec![root];
        let mut index = 0;
        while let Some(&parent) = tree.get(index) {
            tree.extend(guard.children(parent));
            index += 1;
        }
        tree
    }

    fn check(&self, pids: &[u32]) {
        let programs = [&self.program, &self.adapter];
        let identities = programs.map(|path| Identity::of(path));
        let mut seen = Vec::new();
        for pid in pids {
            let exe = PathBuf::from(format!("/proc/{pid}/exe"));
            let path = fs::read_link(&exe).expect("live process executable");
            assert!(!path.as_os_str().as_encoded_bytes().ends_with(b" (deleted)"), "PID {pid}: deleted exe");
            let name = path.file_name().expect("executable name").to_string_lossy();
            assert!(!name.contains("forgejo") && !name.contains("runner"), "PID {pid}: Forgejo or runner");
            let identity = Identity::of(&exe);
            assert!(!self.old.contains(&identity), "PID {pid}: old build inode");
            assert!(
                identities.contains(&identity),
                "PID {pid}: executable outside the installed bin files: {}",
                path.display()
            );
            seen.push(identity);
        }
        assert!(seen.contains(&identities[0]), "the installed box is running");
        assert!(boxed::find(pids, &self.program, b"serve").is_some(), "the new engine is running");
    }

    fn after_link(&self) {
        let guard = Guard::observing_test();
        self.check(&self.tree(&guard));
    }

    fn during_run(&self) {
        let mut patience = Patience::arm("the restarted box's live program run", support::WITHIN);
        loop {
            let guard = Guard::observing_test();
            let pids = self.tree(&guard);
            patience.watch_process(pids[0]);
            if boxed::find(&pids, &self.program, b"agent").is_some()
                && boxed::find(&pids, &self.adapter, b"-p").is_some()
            {
                self.check(&pids);
                return;
            }
            if let Some(figures) = patience.failed() {
                panic!("new agent and adapter did not start: {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

/// Opening values follow temper/docs/design/client/drive.md, 3.4.
struct Forgejo {
    origin: String,
    admin: PathBuf,
    git: PathBuf,
    header: PathBuf,
}
impl Forgejo {
    fn opening(stderr: &[u8], client: &Path) -> Self {
        let text = std::str::from_utf8(stderr).expect("box opening UTF-8");
        let newest = |prefix| {
            text.lines().rev().find_map(|line| line.strip_prefix(prefix)).expect("box printed its opening value")
        };
        let forgejo = Self {
            origin: newest("box: fake forgejo ").into(),
            admin: newest("box: Forgejo elsewhere admin token file ").into(),
            git: Path::new(newest("box: effective data ")).join("demo/bin/git"),
            header: client.join("person.header"),
        };
        assert!(forgejo.origin.starts_with("http://"), "fake Forgejo's HTTP origin");
        let token = fs::read_to_string(&forgejo.admin).expect("fake admin token file");
        support::write_private(&forgejo.header, format!("Authorization: token {}\n", token.trim_end()).as_bytes());
        assert_eq!(fs::metadata(&forgejo.header).expect("person header").permissions().mode() & 0o777, 0o600);
        forgejo
    }

    fn connect(&self) -> Call {
        Call::new(&[
            "forge", "connect", "forgejo-admin", self.admin.to_str().expect("admin path"),
            "--first", "work/hello", "--api", self.origin.strip_prefix("http://").expect("HTTP origin"),
            "--git", &self.origin, "--git-program", self.git.to_str().expect("fake git path"),
            "--key", "newer-forge", "--json",
        ])
    }

    fn request(&self, method: &str, suffix: &str, body: Option<Value>) -> Value {
        let body = body.map(|value| serde_json::to_string(&value).expect("API request JSON"));
        let (status, answer) = support::first_repository::header_request(
            &format!("{}/api/v1", self.origin),
            self.header.to_str().expect("header path"),
            method,
            &format!("/repos/work/hello{suffix}"),
            body.as_deref(),
        );
        assert_eq!(status, if method == "POST" { 201 } else { 200 }, "{method} {suffix}: {answer}");
        answer
    }

    fn get(&self, suffix: &str) -> Value {
        self.request("GET", suffix, None)
    }

    fn failed(statuses: &Value) {
        assert_eq!(statuses["state"], "failure", "the head's combined verdict");
        let [check] = statuses["statuses"].as_array().expect("head statuses").as_slice() else {
            panic!("one check context")
        };
        assert_eq!(check["context"], "check");
        assert_eq!(check["state"], "failure");
    }

    fn make_failed_pull(&self) {
        let main = self.get("/branches/main");
        // The fake's contents route takes an existing branch, not new_branch.
        self.request("POST", "/branches", Some(json!({
            "new_branch_name": "failed-check", "old_ref_name": main["commit"]["id"],
        })));
        // ZmFpbAo= is the contents API's base64 for "fail\n".
        let committed = self.request("POST", "/contents/.fake-forge/ci", Some(json!({
            "branch": "failed-check", "content": "ZmFpbAo=", "message": "Declare a failing check",
            "author": {"name": "Person", "email": "person@example.invalid"},
            "committer": {"name": "Person", "email": "person@example.invalid"},
        })));
        let pull = self.request("POST", "/pulls", Some(json!({
            "head": "failed-check", "base": "main", "title": "A failed check kept across restart", "body": "",
        })));
        assert_eq!(pull["head"]["ref"], "failed-check");
        assert_eq!(pull["base"]["ref"], "main");
        assert_eq!(pull["head"]["sha"], committed["commit"]["sha"], "the pull holds the declared failing CI");
        let head = pull["head"]["sha"].as_str().expect("new pull head");
        let mut patience = Patience::arm("the fake pull head's failed check", support::WITHIN);
        loop {
            let statuses = self.get(&format!("/commits/{head}/status"));
            if statuses["state"] == "failure" {
                Self::failed(&statuses);
                return;
            }
            if let Some(figures) = patience.failed() {
                panic!("the declared check did not fail: {statuses}; {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Kept {
    forge: Value,
    repository: Value,
    adoption: Value,
    task: Value,
    conversation: Value,
    api_repository: Value,
    pull: Value,
    statuses: Value,
}
fn read(client: &Path, arguments: &[&str]) -> Value {
    let call = ctl::alone(client, Call::new(arguments).json());
    answered(&call);
    ctl::json(&call)
}
fn kept(client: &Path, forgejo: &Forgejo) -> Kept {
    let forges = read(client, &["forge", "list"]);
    let [forge] = forges["forges"].as_array().expect("forge rows").as_slice() else { panic!("one managed forge") };
    assert_eq!(forge["state"], "connected");
    assert_eq!(forge["mode"], "managed");
    let repository = read(client, &["forge", "repositories", "1"]);
    let [row] = repository["rows"].as_array().expect("repository rows").as_slice() else { panic!("one repository") };
    assert_eq!(row["owner"]["login"], "work");
    assert_eq!(row["name"], "hello");
    let project = read(client, &["project", "show", "hello"]);
    let [adoption] = project["resources"].as_array().expect("adopted repositories").as_slice() else {
        panic!("one adopted repository")
    };
    assert_eq!(adoption["role"], "owned");
    assert_eq!(adoption["home"], true);
    let document = read(client, &["task", "show", "1"]);
    assert_eq!(document["chip"]["task"], 1);
    let mut task = serde_json::Map::new();
    // Persisted task facts, not the read's digest or its relative scheduling advice.
    for name in ["chip", "project", "spec", "spec_revision", "instructions", "resources", "requester", "result"] {
        task.insert(name.into(), document.get(name).expect("kept task field").clone());
    }
    let conversation = read(client, &["chat", "show", "1"]);
    assert_eq!(conversation["activity"]["cut"], false);
    let api_repository = forgejo.get("");
    assert_eq!(api_repository["full_name"], "work/hello");
    let pulls = forgejo.get("/pulls?state=all");
    let [pull] = pulls.as_array().expect("API pull requests").as_slice() else { panic!("one pull request") };
    let number = pull["number"].as_u64().expect("pull request number");
    let pull = forgejo.get(&format!("/pulls/{number}"));
    assert_eq!(pull["state"], "open");
    let head = pull["head"]["sha"].as_str().expect("pull head");
    let statuses = forgejo.get(&format!("/commits/{head}/status"));
    Forgejo::failed(&statuses);
    Kept {
        forge: forge.clone(),
        repository,
        adoption: adoption.clone(),
        task: Value::Object(task),
        conversation: conversation["activity"]["rows"].clone(),
        api_repository,
        pull: json!({"number": number, "state": pull["state"], "head": pull["head"]}),
        statuses,
    }
}
fn answered(call: &Ended) {
    assert_eq!(call.exit, Some(Exit::Code(0)), "ctl: {}", support::without_secrets(&call.stderr));
}
fn drained<T>(outcome: &boxed::CtlOutcome<T>, calls: usize) {
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "box: {}", support::without_secrets(&outcome.stderr));
    assert_eq!(outcome.left, Some(Vec::new()), "no process outlives its box");
    assert_eq!(outcome.calls.len(), calls, "every ctl act completed");
    for call in &outcome.calls {
        answered(call);
    }
    support::assert_one_printed_link("newer build", &outcome.stdout, &outcome.stderr);
}

fn page(config: &Path) {
    let document: Value = serde_json::from_slice(&fs::read(config.join("temper.json")).expect("configuration"))
        .expect("configuration JSON");
    let address = document["application"]["clients"]["listener"]["socket"]
        .as_str()
        .expect("clients address")
        .parse()
        .expect("clients socket");
    let mut socket = support::socket::Socket::connect(
        address,
        "the new page's connection",
        support::WITHIN,
        ["the new page's reply", "the new page's request"],
        true,
        None,
    )
    .expect("served page connection");
    socket.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").expect("page request");
    let mut bytes = Vec::new();
    socket.take(65_536).read_to_end(&mut bytes).expect("page reply");
    assert!(bytes.starts_with(b"HTTP/1.1 200 "), "page served successfully");
    let text = String::from_utf8_lossy(&bytes);
    let (_, body) = text.split_once("\r\n\r\n").expect("HTTP body");
    assert!(body.contains(MARK), "the served bundle is the newly installed page");
}

fn first_chat(programs: PathBuf, client: PathBuf) -> Vec<Step> {
    let connect_client = client.clone();
    vec![
        Step::Run(Call::sign_in()),
        Step::ThenBox(Box::new(move |stderr| Forgejo::opening(stderr, &connect_client).connect())),
        Step::Run(Call::new(&["forge", "list", "--until", "made:1", "--json"])),
        Step::ThenBox(Box::new(move |stderr| {
            Forgejo::opening(stderr, &client).make_failed_pull();
            super::private(&programs);
            // The first chat replies normally; the post-restart run stays live until stopped.
            let script = json!({"status":"Truthful", "login":{"Immediate":"claude.ai"}, "logout":"Succeeds",
                "asked":[{"cue":LIVE,"turns":[{"steps":["Hang"]}]}]});
            super::write(&programs.join("fake-claude.story"), &serde_json::to_vec(&script).expect("program story"));
            Call::new(&["account", "declare", "2", "claude_code", SOURCE, "--key", "newer-program"])
        })),
        Step::ProgramLogin(SOURCE),
        Step::Run(Call::new(&["account", "verify", "1", "--source", "program", "--json"])),
        Step::Run(Call::new(&["account", "list", "--until", "usable", "--json"])),
        Step::Run(Call::new(&[
            "chat",
            "start",
            "--project",
            "hello",
            "--model",
            "claude-opus-5-5",
            "--key",
            "before-stop",
            "Hello before the stop",
            "--json",
        ])),
        Step::Run(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
    ]
}

fn after_restart(probe: Probe, config: PathBuf, client: PathBuf, before: Kept) -> Vec<Step> {
    let live_probe = probe.clone();
    vec![
        Step::Run(Call::sign_in()),
        Step::Then(Box::new(move |_| {
            probe.after_link();
            Call::new(&["forge", "list", "--until", "connected:1", "--json"])
        })),
        Step::Run(Call::new(&["account", "list", "--until", "usable", "--json"])),
        Step::ThenBox(Box::new(move |stderr| {
            let forgejo = Forgejo::opening(stderr, &client);
            assert_eq!(kept(&client, &forgejo), before, "ctl and API facts survived unchanged");
            page(&config);
            Call::new(&[
                "chat",
                "start",
                "--project",
                "hello",
                "--model",
                "claude-opus-5-5",
                "--key",
                "after-restart",
                LIVE,
                "--json",
            ])
        })),
        Step::Run(Call::new(&["task", "show", "2", "--until", "phase:running", "--json"])),
        Step::Then(Box::new(move |_| {
            live_probe.during_run();
            Call::new(&["task", "stop", "2", "--key", "end-live-run", "--json"])
        })),
        Step::Run(Call::new(&["task", "show", "2", "--until", "held", "--json"])),
    ]
}

#[test]
fn a_stopped_fake_box_starts_with_new_files_at_the_same_paths() {
    let installed = Installed::new();
    let root = installed.scratch.path();
    let programs = installed.paths[1].join("box/fake-backends/data/programs").join(SOURCE);
    let first = boxed::run_ctl_from(
        &installed.bin().join("temper-box"),
        root,
        installed.flags(),
        root,
        first_chat(programs, root.into()),
        |binary| {
            installed.probe(Vec::new()).after_link();
            (kept(root, &Forgejo::opening(binary.stderr(), root)), Guard::observing_test())
        },
    );
    drained(&first, 9);
    let (before, observed) = first.checked.expect("initial chat and reads completed");
    assert!(observed.has_descendants(), "box tree observed before the stop");
    assert!(observed.all_exited(), "nothing left after the first termination");

    let old_files = installed.replace();
    let old = old_files.iter().map(|file| Identity::metadata(&file.metadata().expect("old file"))).collect();
    let client = root.join("restarted-client");
    super::private(&client);
    let steps = after_restart(installed.probe(old), installed.config(), client.clone(), before);
    let restarted =
        boxed::run_ctl_from(&installed.bin().join("temper-box"), root, installed.flags(), &client, steps, |_| {
            Guard::observing_test()
        });
    drained(&restarted, 7);
    assert!(
        restarted.checked.expect("both executable snapshots and new page checked").all_exited(),
        "nothing left after the restarted box's termination"
    );
    drop(old_files);
    installed.scratch.remove();
}
