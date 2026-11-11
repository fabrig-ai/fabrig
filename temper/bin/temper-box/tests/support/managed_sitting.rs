//! The managed fake sitting (temper/docs/design/testing/process-tier.md).
//! Connect makes the person's App (temper/docs/design/protocol/setup/github.md, 2).
use super::{
    managed_client::{self, Mode},
    patience::Patience,
    relay::managed_transport::Answer,
    relay_managed::{NAME, OWNER},
};
use serde_json::{Value, json};
use skein_lib::Duration;
use skein_scratch::{Scratch, paths};
use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

// The driver's private output and page bounds, and its readiness/command/page waits.
const BOUND: usize = 16 * 1024 * 1024;
const LINES: usize = 4096;
const LINE_BYTES: u64 = 65536;
const READY: Duration = Duration::from_secs(30);
const COMMAND: Duration = Duration::from_secs(60);
const PAGE: Duration = Duration::from_secs(10);
const STOP: Duration = Duration::from_secs(5);
const SCAN: Duration = Duration::from_secs(45);
const POLL: std::time::Duration = std::time::Duration::from_millis(20);
const DIAGNOSTIC_LINES: usize = 16;
const DIAGNOSTIC_BYTES: usize = 4096;

/// Sign in, Connect and adopt through Ready on the sitting's fake peers.
pub fn run() {
    let scratch = Scratch::new("managed-sitting");
    let root = scratch.path();
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).expect("private sitting scratch");
    let binary = paths::binary("temper-box");
    let mut fake = start_forge(root, &binary);
    let upstream = address(&fake.line("listening "));
    let evidence = root.join("conversion.json");
    let mut relay = start_relay(root, upstream, &evidence);
    let ready = root.join("relay-ready.json");
    let mut patience = relay.patience("managed relay readiness", READY);
    let addresses: Value = loop {
        if let Ok(bytes) = fs::read(&ready)
            && let Ok(value) = serde_json::from_slice(&bytes)
        {
            break value;
        }
        relay.running();
        pause(&relay, &mut patience);
    };
    let api = address(addresses["api"].as_str().expect("relay API address"));
    let git = address(addresses["git"].as_str().expect("relay git address"));
    let [clients] = temper_fixtures::ports::addresses::<1>(&[]);
    configure(root, api, git, upstream, clients);
    let mut command = child_command(root, &binary, "engine");
    command
        .arg("serve")
        .arg("--config")
        .arg(root.join("config"))
        .arg("--data")
        .arg(root.join("data"))
        .arg("--cache")
        .arg(root.join("cache"));
    let mut engine = Process::start(&mut command);
    let link = format!("http://{}", engine.line("http://"));
    let (printed, _) = url(&link, &[clients]);
    assert_eq!(printed, clients, "printed link uses the chosen client port");
    managed_client::ready(root, &link, Mode::SignIn);
    super::write_private(&root.join("client/address"), format!("http://{clients}\n").as_bytes());
    let peers = [(upstream, fake.child.id()), (clients, engine.child.id())];
    connect(root, &binary, clients, &peers);
    managed_client::ready(root, &link, Mode::Adopt);
    engine.stop();
    fake.stop();
    finish_relay(&mut relay, &evidence);
}

fn start_forge(root: &Path, binary: &Path) -> Process {
    let document = json!({
        "listen":{"socket":"127.0.0.1:0"}, "admin_token":root.join("fake-admin"), "owner":OWNER,
        "repositories":[
            {"owner":OWNER,"name":NAME,"landing":"main","ci":null,"id":1},
            {"owner":OWNER,"name":"temper-probe","landing":"main","ci":null,"id":2}
        ],
        "github":{
            "app":{"id":4242,"slug":"temper-test","bot":4243,"key":0},
            "accounts":[
                {"login":"managed-person","id":10,"kind":"user","installation":null},
                {"login":OWNER,"id":30,"kind":"organization","installation":null},
                {"login":"managed-other","id":20,"kind":"user","installation":{"id":1,"repositories":"all"}}
            ], "token_life_seconds":3600
        }
    });
    let document_path = root.join("fake.json");
    super::write_private(&document_path, &serde_json::to_vec(&document).expect("fake document JSON"));
    let mut command = child_command(root, binary, "forge");
    command.args(["fake", "forge", "--document"]).arg(&document_path).args(["--ready", "listening "]);
    Process::start(&mut command)
}

fn start_relay(root: &Path, upstream: SocketAddr, evidence: &Path) -> Process {
    let mut command = child_command(root, &std::env::current_exe().expect("sitting harness"), "relay");
    command
        .args(["--ignored", "--exact", "managed_relay", "--nocapture"])
        .stdin(Stdio::piped())
        .env(temper_test_env::MANAGED_MODE.name, "fake")
        .env(temper_test_env::MANAGED_SCRATCH.name, root)
        .env(temper_test_env::MANAGED_UPSTREAM.name, upstream.to_string())
        .env(
            temper_test_env::MANAGED_PREFIX.name,
            format!("temper/live-{}/", root.file_name().expect("run name").to_string_lossy()),
        )
        .env(temper_test_env::MANAGED_EVIDENCE.name, evidence)
        .env(temper_test_env::MANAGED_PERSON.name, "managed-person");
    Process::start(&mut command)
}

fn finish_relay(relay: &mut Process, evidence: &Path) {
    relay.running();
    let mut input = relay.child.stdin.take().expect("relay control pipe");
    input
        .write_all(b"{\"App_key_DER\":[],\"App_key_base64\":[],\"JWT\":[],\"installation_token\":[]}\n")
        .expect("private scan command");
    drop(input);
    assert!(relay.wait(SCAN).success(), "relay secret scan exits successfully");
    relay.line("no secret bytes in the relay log or the record");
    let metadata = fs::metadata(evidence).expect("redacted conversion evidence exists");
    assert!(metadata.is_file(), "evidence is a regular file");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600, "private evidence");
}

fn configure(root: &Path, api: SocketAddr, git: SocketAddr, upstream: SocketAddr, clients: SocketAddr) {
    let fixtures = paths::package().join("tests/support/github");
    let mut template = fs::read_to_string(fixtures.join("engine.json")).expect("engine template");
    for (name, value) in [
        ("proc.forge.ready.address", api),
        ("proc.llm.ready.address", address("127.0.0.1:9")),
        ("story.clients", clients),
    ] {
        template = template.replace(&format!("{{{name}}}"), &value.to_string());
    }
    let template = template.replace("{{", "{").replace("}}", "}").replace("\"acme\"", &format!("\"{OWNER}\""));
    let mut value: Value = serde_json::from_str(&template).expect("rendered engine template");
    value["engine"]["limits"]["overrides"]["people.projects"] = 3.into();
    // Bind the machine endpoints as the process GitHub story does.
    value["application"]["machine"]["github"] = json!({
        "api":{"socket":api.to_string()},"transport":"plaintext",
        "remote":format!("http://{git}"),"web":format!("http://{upstream}")
    });
    private_directory(&root.join("data/secrets"));
    super::write_private(
        &root.join("data/secrets/w1.digest"),
        &fs::read(fixtures.join("w1.digest")).expect("worker digest fixture"),
    );
    private_directory(&root.join("config/charters"));
    super::write_private(&root.join("config/charters/chat.txt"), b"No model calls in this sitting.\n");
    super::write_private(&root.join("config/temper.json"), &serde_json::to_vec(&value).expect("engine JSON"));
}

fn connect(root: &Path, binary: &Path, clients: SocketAddr, peers: &[(SocketAddr, u32); 2]) {
    let output = ctl(root, binary, &["forge", "connect", "github", "--key", "managed-connect"]);
    let link = output.lines().find(|line| line.starts_with("http://")).expect("Connect link");
    let form = page("GET", link, "", peers, 200);
    let action = attribute(tag(&form.body, "form", None), "action");
    let manifest = attribute(tag(&form.body, "input", Some(("name", "manifest"))), "value");
    let manifest_json: Value = serde_json::from_str(&manifest).expect("manifest JSON");
    for name in ["redirect_url", "setup_url"] {
        url(manifest_json[name].as_str().expect("manifest callback"), &[clients]);
    }
    let created = page("POST", &action, &format!("manifest={}", form_encode(&manifest)), peers, 200);
    let create = attribute(tag(&created.body, "a", Some(("id", "create"))), "href");
    let redirect = page("GET", &create, "", peers, 302);
    let install = format!("{}&account={OWNER}&repository_ids=1", redirect.location);
    let accepted = page("GET", &install, "", peers, 302);
    let finish = page("GET", &accepted.location, "", peers, 200);
    assert!(finish.body.contains("temper has the installation"), "selected installation accepted");
    ctl(root, binary, &["forge", "list", "--until", "settled"]);
    ctl(root, binary, &["project", "make", "Sitting", "--key", "managed-project"]);
}

fn ctl(root: &Path, binary: &Path, words: &[&str]) -> String {
    let mut command = environment(root, binary);
    command.arg("ctl").arg(root.join("client")).args(words);
    let mut process = Process::start(&mut command);
    let status = process.wait(COMMAND);
    assert!(status.success(), "{}", process.failure(status, "during ctl"));
    process.stdout()
}

fn private_directory(path: &Path) {
    fs::create_dir_all(path).expect("sitting directory");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private sitting directory");
}

fn child_command(root: &Path, binary: &Path, name: &str) -> Command {
    let cwd = root.join(name);
    private_directory(&cwd);
    let mut command = environment(root, binary);
    command.current_dir(cwd);
    command
}

fn environment(root: &Path, binary: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("PATH", temper_test_env::value(&temper_shell::environment::PATH).expect("child executable search path"))
        .env("HOME", root)
        .env("TMPDIR", root)
        .env("LANG", "C.UTF-8")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

#[derive(Default)]
struct Lines {
    rows: VecDeque<(bool, String)>,
    diagnostics: VecDeque<String>,
    bytes: usize,
    failed: bool,
    pem: bool,
}

impl Lines {
    fn record(&mut self, stdout: bool, line: String) {
        if !stdout {
            if line.starts_with("-----BEGIN ") {
                self.pem = true;
            }
            let safe = if self.pem { "[redacted]\n".into() } else { diagnostic(&line) };
            if line.starts_with("-----END ") {
                self.pem = false;
            }
            if self.diagnostics.len() == DIAGNOSTIC_LINES {
                self.diagnostics.pop_front();
            }
            self.diagnostics.push_back(safe);
        }
        self.rows.push_back((stdout, line));
    }
}

struct Process {
    child: Child,
    name: String,
    lines: Arc<Mutex<Lines>>,
    readers: Vec<JoinHandle<()>>,
}
impl Process {
    fn start(command: &mut Command) -> Self {
        let name = command
            .get_current_dir()
            .and_then(Path::file_name)
            .expect("named sitting child")
            .to_string_lossy()
            .into_owned();
        let child = command.spawn().expect("start private sitting process");
        let mut process = Self { child, name, lines: Arc::default(), readers: Vec::new() };
        let stdout = process.child.stdout.take().expect("private stdout");
        let stderr = process.child.stderr.take().expect("private stderr");
        process.readers.push(drain(stdout, true, process.lines.clone()));
        process.readers.push(drain(stderr, false, process.lines.clone()));
        process
    }

    fn patience(&self, what: &'static str, within: Duration) -> Patience {
        let mut patience = Patience::arm(what, within);
        patience.watch_process(self.child.id());
        patience
    }

    fn running(&mut self) {
        if let Some(status) = self.child.try_wait().expect("poll sitting process") {
            self.join();
            panic!("{}", self.failure(status, "before readiness"));
        }
        assert!(!self.lines.lock().expect("private output lock").failed, "private process output bound or encoding");
    }

    fn failure(&self, status: ExitStatus, when: &str) -> String {
        let lines = self.lines.lock().expect("private output lock");
        format!(
            "sitting child {} (pid {}) ended {when}: {status}; stderr (redacted):\n{}",
            self.name,
            self.child.id(),
            lines.diagnostics.iter().cloned().collect::<String>()
        )
    }

    fn line(&mut self, prefix: &str) -> String {
        let mut patience = self.patience("managed process ready line", READY);
        loop {
            {
                let mut lines = self.lines.lock().expect("private output lock");
                assert!(!lines.failed, "private process output bound or encoding");
                while let Some((_, line)) = lines.rows.pop_front() {
                    if let Some(value) = line.strip_prefix(prefix) {
                        return value.trim_end().to_owned();
                    }
                }
            }
            self.running();
            pause(self, &mut patience);
        }
    }

    fn wait(&mut self, within: Duration) -> ExitStatus {
        let mut patience = self.patience("managed process exit", within);
        loop {
            if let Some(status) = self.child.try_wait().expect("poll sitting exit") {
                self.join();
                assert!(
                    !self.lines.lock().expect("private output lock").failed,
                    "private process output bound or encoding"
                );
                return status;
            }
            pause(self, &mut patience);
        }
    }

    fn stdout(&self) -> String {
        self.lines
            .lock()
            .expect("private output lock")
            .rows
            .iter()
            .filter(|(stdout, _)| *stdout)
            .map(|(_, line)| line.as_str())
            .collect()
    }

    fn join(&mut self) {
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
    }

    fn stop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = Command::new("/usr/bin/kill")
                .args(["-TERM", &self.child.id().to_string()])
                .env_clear()
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let mut patience = self.patience("managed child SIGTERM", STOP);
            loop {
                if self.child.try_wait().ok().flatten().is_some() {
                    break;
                }
                if patience.failed().is_some() || patience.deferred() {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    break;
                }
                thread::sleep(POLL);
            }
        }
        self.join();
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.stop();
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the sitting drains and joins each private child pipe as its driver does"
)]
fn drain(pipe: impl Read + Send + 'static, stdout: bool, lines: Arc<Mutex<Lines>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut pipe = BufReader::new(pipe);
        loop {
            let mut bytes = Vec::new();
            let read = pipe.by_ref().take(LINE_BYTES + 1).read_until(b'\n', &mut bytes);
            if read.as_ref().is_ok_and(|count| *count == 0) {
                break;
            }
            let mut lines = lines.lock().expect("private output lock");
            lines.bytes = lines.bytes.saturating_add(bytes.len());
            if read.is_err() {
                lines.failed = true;
                break;
            }
            if bytes.len() > usize::try_from(LINE_BYTES).expect("line bound")
                || lines.bytes > BOUND
                || lines.rows.len() >= LINES
            {
                lines.failed = true;
                continue;
            }
            match String::from_utf8(bytes) {
                Ok(line) => lines.record(stdout, line),
                Err(_) => lines.failed = true,
            }
        }
    })
}

fn diagnostic(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if ["secret", "token", "authorization", "bearer", "key", "http://", "https://", "-----"]
        .iter()
        .any(|word| lower.contains(word))
    {
        return "[redacted]\n".into();
    }
    let mut safe: String = line
        .char_indices()
        .take_while(|(at, character)| at + character.len_utf8() < DIAGNOSTIC_BYTES / DIAGNOSTIC_LINES)
        .map(|(_, character)| character)
        .collect();
    if !safe.ends_with('\n') {
        safe.push('\n');
    }
    safe
}

#[test]
fn startup_diagnostics_redact_credentials_and_pem_bodies() {
    for line in [
        "secret=private",
        "TOKEN private",
        "Authorization: private",
        "Bearer private",
        "key: private",
        "http://localhost/private",
        "https://localhost/private",
        "-----BEGIN PRIVATE KEY-----",
    ] {
        assert_eq!(diagnostic(line), "[redacted]\n");
    }
    assert_eq!(diagnostic("serve: configuration refused\n"), "serve: configuration refused\n");
    let mut lines = Lines::default();
    for line in ["-----BEGIN PRIVATE KEY-----", "encoded-private-body", "-----END PRIVATE KEY-----"] {
        lines.record(false, line.into());
    }
    assert_eq!(lines.diagnostics.into_iter().collect::<String>(), "[redacted]\n".repeat(3));
}

#[test]
fn startup_diagnostics_keep_a_bounded_utf8_tail_after_readiness_consumes_output() {
    let mut lines = Lines::default();
    lines.record(true, "private stdout\n".into());
    for _ in 0..=DIAGNOSTIC_LINES {
        lines.record(false, "é".repeat(DIAGNOSTIC_BYTES));
    }
    lines.rows.clear();
    assert_eq!(lines.diagnostics.len(), DIAGNOSTIC_LINES);
    assert!(!lines.diagnostics.iter().any(|line| line.contains("stdout")));
    assert!(lines.diagnostics.iter().map(String::len).sum::<usize>() <= DIAGNOSTIC_BYTES);
}

fn pause(process: &Process, patience: &mut Patience) {
    if let Some(figures) = patience.failed() {
        panic!(
            "sitting child {} (pid {}) wait: {figures}; stderr (redacted):\n{}",
            process.name,
            process.child.id(),
            process.lines.lock().expect("private output lock").diagnostics.iter().cloned().collect::<String>()
        );
    }
    thread::sleep(POLL);
}

fn address(value: &str) -> SocketAddr {
    let address: SocketAddr = value.parse().expect("numeric sitting address");
    assert!(address.ip().is_loopback(), "sitting peer stays on loopback");
    address
}

fn url<'a>(value: &'a str, allowed: &[SocketAddr]) -> (SocketAddr, &'a str) {
    let value = value.strip_prefix("http://").expect("sitting URL is HTTP");
    let (host, target) = value.split_once('/').expect("sitting URL has a path");
    let peer = address(host);
    assert!(allowed.contains(&peer), "sitting URL names an assigned peer");
    assert!(
        !target.starts_with('/') && !target.contains('#') && !target.contains(['\r', '\n']),
        "sitting URL has one absolute path and no fragment or framing bytes"
    );
    (peer, target)
}

struct Page {
    body: String,
    location: String,
}

fn page(method: &str, link: &str, body: &str, peers: &[(SocketAddr, u32); 2], status: u16) -> Page {
    let (peer, target) = url(link, &peers.map(|(address, _)| address));
    let mut patience = Patience::arm("managed sitting page", PAGE);
    let (_, pid) = peers.iter().find(|(address, _)| *address == peer).expect("assigned page process");
    patience.watch_process(*pid);
    let mut socket = loop {
        let result = match patience.socket_timeout() {
            Some(timeout) => TcpStream::connect_timeout(&peer, timeout),
            None => TcpStream::connect(peer),
        };
        match result {
            Ok(socket) => break socket,
            Err(error) => patience.socket_retry(error).expect("connect assigned sitting page"),
        }
    };
    let request = format!(
        "{method} /{target} HTTP/1.1\r\nHost: {peer}\r\nUser-Agent: temper-managed-sitting\r\nContent-Type: application/x-www-form-urlencoded\r\nAccept: application/vnd.github+json\r\nX-GitHub-Api-Version: 2026-03-10\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut remaining = request.as_bytes();
    while !remaining.is_empty() {
        socket.set_write_timeout(patience.socket_timeout()).expect("page write timeout");
        match socket.write(remaining) {
            Ok(0) => panic!("sitting page write closed"),
            Ok(count) => remaining = &remaining[count..],
            Err(error) => patience.socket_retry(error).expect("write sitting page"),
        }
    }
    let mut bytes = Vec::new();
    loop {
        socket.set_read_timeout(patience.socket_timeout()).expect("page read timeout");
        let mut buffer = [0; 8192];
        match socket.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                assert!(bytes.len() + count <= BOUND, "sitting page bound");
                bytes.extend_from_slice(&buffer[..count]);
            }
            Err(error) => patience.socket_retry(error).expect("read sitting page"),
        }
    }
    let answer = Answer::from_bytes(&bytes).expect("framed sitting page");
    assert_eq!(answer.status().expect("page status"), status, "sitting page status");
    let headers = bytes.windows(4).position(|part| part == b"\r\n\r\n").expect("page headers");
    let headers = std::str::from_utf8(&bytes[..headers]).expect("page header text");
    let location = headers
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .map_or_else(String::new, |(_, value)| value.trim().to_owned());
    Page {
        body: String::from_utf8(answer.body().expect("page body")).unwrap_or_else(|_| panic!("page text")),
        location,
    }
}

fn tag<'a>(body: &'a str, name: &str, selected: Option<(&str, &str)>) -> &'a str {
    let marker = format!("<{name} ");
    body.split(&marker)
        .skip(1)
        .map(|part| part.split_once('>').expect("HTML tag end").0)
        .find(|part| selected.is_none_or(|(key, value)| attribute(part, key) == value))
        .expect("sitting HTML element")
}

fn attribute(tag: &str, name: &str) -> String {
    let marker = format!("{name}=\"");
    let value = tag.split_once(&marker).expect("HTML attribute").1.split_once('"').expect("HTML attribute end").0;
    value.replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

fn form_encode(value: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("form byte");
        }
    }
    encoded
}
