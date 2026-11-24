//! The QA runner starts the shipped binary, then reads files and serves requests.
//! Fake issuer cap logging: temper/docs/design/testing/process-tier.md.
use crate::{listeners, support};
use serde_json::{Value, json};
use skein_world::{Deadline, Expired};
use std::{
    cell::RefCell,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    os::{fd::AsRawFd, unix::fs::PermissionsExt},
    path::Path,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

fn command(kind: &str, document: &Path) -> Command {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command
        .env_clear()
        .args(["fake", kind, "--document"])
        .arg(document)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}
fn llm_document(directory: &Path, tls: bool) -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"key":directory.join("key"),"tls":if tls {json!({"root":directory.join("root.der")})}else{Value::Null},
        "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],
        "asked":[{"cue":"say hello","turns":["hello from QA"]}],"otherwise":["fallback"]})
}
fn forge_document(directory: &Path) -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"admin_token":directory.join("forge/admin_token"),"owner":"fake",
        "repositories":[{"name":"widget","landing":"main","ci":null}]})
}
fn document(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).expect("valid fixture")).expect("valid fixture");
}
struct Running {
    child: RefCell<Option<Child>>,
    sockets: RefCell<Vec<Option<TcpStream>>>,
    address: SocketAddr,
}
impl Running {
    fn start(kind: &str, document: &Path, observations: Option<&Path>) -> Result<Self, String> {
        Self::command(command(kind, document), observations)
    }
    fn command(mut command: Command, observations: Option<&Path>) -> Result<Self, String> {
        command.args(["--ready", "ready ", "--seed", "23"]);
        if let Some(path) = observations {
            command.arg("--observations").arg(path);
        }
        let child = command.spawn().expect("valid fixture");
        let mut run = Self {
            child: RefCell::new(Some(child)),
            sockets: RefCell::new(Vec::new()),
            address: "127.0.0.1:0".parse().expect("valid fixture"),
        };
        run.address = temper_box_kit::fake_readiness::child(run.child.get_mut(), &command)?.address;
        Ok(run)
    }
    fn stop(self) {
        let stderr = self.stop_with_stderr();
        assert!(stderr.is_empty(), "{}", String::from_utf8_lossy(&stderr));
    }
    fn stop_with_stderr(mut self) -> Vec<u8> {
        self.sockets.get_mut().clear();
        let child = self.child.get_mut().take().expect("valid fixture");
        assert!(
            Command::new("/bin/kill")
                .args(["-TERM", &child.id().to_string()])
                .status()
                .expect("valid fixture")
                .success()
        );
        let ended = child.wait_with_output().expect("valid fixture");
        assert_eq!(ended.status.code(), Some(0), "{}", String::from_utf8_lossy(&ended.stderr));
        assert!(ended.stdout.is_empty(), "stdout holds only the ready line");
        ended.stderr
    }
    fn deadline(&self, what: &'static str) -> Deadline {
        let mut deadline = Deadline::arm(what, skein_lib::Duration::from_millis(300));
        deadline.watch_process(self.child.borrow().as_ref().expect("running fake").id());
        deadline
    }
    fn end(&self) {
        assert!(
            self.child.borrow_mut().as_mut().expect("running fake").try_wait().expect("fake status").is_none(),
            "the fake exited before teardown"
        );
        self.sockets.borrow_mut().clear();
        if let Some(mut child) = self.child.borrow_mut().take() {
            child.kill().expect("stop the unanswered fake");
            child.wait().expect("reap the unanswered fake");
        }
    }
    fn expired(&self, deadline: &Deadline) -> ! {
        assert!(
            self.child.borrow_mut().as_mut().expect("running fake").try_wait().expect("fake status").is_none(),
            "the fake exited before answering"
        );
        match deadline.expired() {
            Expired::Starved(starved) => {
                self.end();
                starved.pass();
            }
            Expired::Failed(figures) => panic!("the fake did not answer: {figures}"),
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Some(child) = self.child.get_mut().as_mut() {
            let _killed = child.kill();
            let _ended = child.wait();
        }
    }
}
/// The fake owns its sockets so a failed wait can close them before reaping it.
struct Socket<'a> {
    run: &'a Running,
    slot: usize,
    deadline: Deadline,
}
impl Socket<'_> {
    fn reset(&mut self, what: &'static str) {
        self.deadline = self.run.deadline(what);
    }
}
fn timeout(deadline: &Deadline) -> Duration {
    Duration::from_nanos(deadline.remaining().as_nanos()).max(Duration::from_millis(1))
}
fn timed_out(why: &std::io::Error) -> bool {
    matches!(why.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock)
}
impl Read for Socket<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            if self.deadline.passed() {
                self.run.expired(&self.deadline);
            }
            let result = {
                let mut sockets = self.run.sockets.borrow_mut();
                let socket = sockets[self.slot].as_mut().expect("open socket");
                socket.set_read_timeout(Some(timeout(&self.deadline)))?;
                socket.read(bytes)
            };
            match result {
                Err(why) if timed_out(&why) => {}
                other => return other,
            }
        }
    }
}
impl Write for Socket<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            if self.deadline.passed() {
                self.run.expired(&self.deadline);
            }
            let result = {
                let mut sockets = self.run.sockets.borrow_mut();
                let socket = sockets[self.slot].as_mut().expect("open socket");
                socket.set_write_timeout(Some(timeout(&self.deadline)))?;
                socket.write(bytes)
            };
            match result {
                Err(why) if timed_out(&why) => {}
                other => return other,
            }
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Drop for Socket<'_> {
    fn drop(&mut self) {
        if let Some(socket) = self.run.sockets.borrow_mut().get_mut(self.slot) {
            let _closed = socket.take();
        }
    }
}
fn stream(run: &Running, address: SocketAddr) -> Socket<'_> {
    let deadline = run.deadline("the fake socket's connection");
    let socket = loop {
        match TcpStream::connect_timeout(&address, timeout(&deadline)) {
            Ok(socket) => break socket,
            Err(why) if timed_out(&why) => {
                if deadline.passed() {
                    run.expired(&deadline);
                }
            }
            Err(why) => panic!("fake socket connection: {why}"),
        }
    };
    let mut sockets = run.sockets.borrow_mut();
    let slot = sockets.len();
    sockets.push(Some(socket));
    Socket { run, slot, deadline: run.deadline("the fake socket's request") }
}
fn http(method: &str, target: &str, authorization: &str, body: &[u8]) -> Vec<u8> {
    let mut request=format!("{method} {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",body.len()).into_bytes();
    if !authorization.is_empty() {
        request.extend_from_slice(format!("Authorization: {authorization}\r\n").as_bytes());
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(body);
    request
}
fn request(
    run: &Running,
    address: SocketAddr,
    method: &str,
    target: &str,
    authorization: &str,
    body: &[u8],
) -> Vec<u8> {
    let mut stream = stream(run, address);
    stream.write_all(&http(method, target, authorization, body)).expect("valid fixture");
    let mut response = Vec::new();
    stream.reset("the fake socket's reply");
    stream.read_to_end(&mut response).expect("valid fixture");
    response
}
fn body(response: &[u8]) -> Value {
    let boundary = response.windows(4).position(|part| part == b"\r\n\r\n").expect("valid fixture") + 4;
    serde_json::from_slice(&response[boundary..]).expect("valid fixture")
}
fn private_hex(path: &Path, expected: &[u8]) {
    let bytes = fs::read(path).expect("valid fixture");
    assert_eq!(bytes.len(), 40);
    assert!(bytes.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    assert_eq!(bytes, expected);
    assert_eq!(fs::metadata(path).expect("valid fixture").permissions().mode() & 0o777, 0o600);
}
#[test]
fn refused_start() {
    let scratch = support::Scratch::new("fake_refused");
    let dir = scratch.path();
    let doc = dir.join("document.json");
    let added_key = dir.join("second.key");
    let mut value = llm_document(dir, true);
    value["added_keys"] = json!([added_key]);
    document(&doc, &value);
    support::write_private(&added_key, b"existing");
    let before = snapshot(dir);
    let output = command("llm", &doc).output().expect("refused fake");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        format!("temper fake llm: {}: retained output differs from expected seeded bytes\n", added_key.display())
            .as_bytes(),
    );
    assert_eq!(snapshot(dir), before, "refused startup restores the files");
    scratch.remove();
}

fn deployment_command(kind: &str, directory: &Path) -> Command {
    let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
    command
        .env_clear()
        .args(["fake", kind])
        .args(support::configuration::flags(directory))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}
fn forge_setup(directory: &Path, listen: &str) {
    let setup = serde_json::json!({"kind":"fake","listen":{"socket":listen},"remote":format!("http://{listen}"),"git":"demo/bin/git"});
    support::configuration::setup(directory, 1, setup);
    fs::create_dir_all(directory.join("box/groups")).expect("groups");
    support::write_private(&directory.join("box/groups/1"), b"fake\n");
}
fn snapshot(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, (u32, Option<Vec<u8>>)> {
    let mut found = std::collections::BTreeMap::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).expect("valid fixture") {
            let entry = entry.expect("valid fixture");
            let meta = entry.metadata().expect("valid fixture");
            let path = entry.path();
            let bytes = if meta.is_dir() {
                directories.push(path.clone());
                None
            } else {
                Some(fs::read(&path).expect("valid fixture"))
            };
            found.insert(path, (meta.permissions().mode() & 0o777, bytes));
        }
    }
    found
}
#[test]
fn forge_state_keeps_an_api_write_only_after_an_orderly_stop() {
    let scratch = support::Scratch::new("forge-state");
    let doc = scratch.path().join("forge.json");
    let state = scratch.path().join("forge.state");
    document(&doc, &forge_document(scratch.path()));
    let start = || {
        let mut cmd = command("forge", &doc);
        cmd.arg("--state").arg(&state);
        Running::command(cmd, None).unwrap_or_else(|why| panic!("{why}"))
    };
    let run = start();
    let token = temper_fake::credential::credential(23, temper_fake::credential::ADMIN_TOKEN);
    let auth = format!("token {}", std::str::from_utf8(&token).expect("hex token"));
    let _grant = request(
        &run,
        run.address,
        "PUT",
        "/api/v1/repos/fake/widget/collaborators/temper-admin",
        &auth,
        br#"{"permission":"admin"}"#,
    );
    let _issue = request(
        &run,
        run.address,
        "POST",
        "/api/v1/repos/fake/widget/issues",
        &auth,
        br#"{"title":"Kept across stop","body":"Kept body"}"#,
    );
    run.stop();
    assert_eq!(fs::metadata(&state).expect("state written at stop").permissions().mode() & 0o777, 0o600);
    let before = fs::read(&state).expect("image");
    let run = start();
    let response = request(&run, run.address, "GET", "/api/v1/repos/fake/widget/issues/1", &auth, b"");
    assert_eq!(body(&response)["title"], "Kept across stop");
    run.end(); // SIGKILL: the prior orderly image is left untouched.
    assert_eq!(fs::read(&state).expect("kept image"), before);
    fs::remove_file(&state).expect("remove prior image");
    let run = start();
    run.end();
    assert!(!state.exists(), "SIGKILL writes no image");
}
#[test]
fn forge_state_refusal_is_one_line_and_leaves_the_file_unchanged() {
    use std::os::unix::fs::symlink;
    let scratch = support::Scratch::new("forge-state-refused");
    let doc = scratch.path().join("forge.json");
    let state = scratch.path().join("forge.state");
    document(&doc, &forge_document(scratch.path()));
    let target = scratch.path().join("extra");
    let original = b"not an image";
    support::write_private(&target, original);
    symlink(&target, &state).expect("link");
    let output = command("forge", &doc).arg("--state").arg(&state).output().expect("fake");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.starts_with(&format!("temper-box fake forge: state {} refused: ", state.display())), "{stderr}");
    assert_eq!(fs::read_link(&state).expect("unchanged link"), target);
    assert_eq!(fs::read(&state).expect("unchanged bad image"), original);
    scratch.remove();
}
#[test]
fn forge_run() {
    let scratch = support::Scratch::new("fake_forge_run");
    let dir = scratch.path();
    let (listen, token_path, token) = prepared_forge(dir);
    let mut command = deployment_command("forge", dir);
    command.arg("1");
    let run = Running::command(command, Some(&dir.join("observations"))).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(run.address, listen, "ready on fake.listen");
    run.stop();
    let meta = fs::metadata(&token_path).expect("prepared token kept at stop");
    assert!(meta.is_file());
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    assert_eq!(fs::read(token_path).expect("prepared token"), token);
    scratch.remove();
}

fn prepared_forge(directory: &Path) -> (SocketAddr, std::path::PathBuf, Vec<u8>) {
    let listen = support::free_addresses::<1>(&[])[0];
    forge_setup(directory, &listen.to_string());
    support::write_file(
        &directory.join(temper_config_shell::FILE),
        &support::configuration::file(),
        temper_config_shell::Shape::Standalone,
    );
    let prepared = support::owned::prepare(directory, 1);
    assert_eq!(prepared.status.code(), Some(0), "{}", String::from_utf8_lossy(&prepared.stderr));
    let path = directory.join("playground-1/admin_token");
    let token = fs::read(&path).expect("prepared token");
    (listen, path, token)
}

const CHAT:&[u8]=br#"{"model":"qa","store":false,"instructions":"","input":[{"role":"user","content":[{"type":"input_text","text":"say hello"}]}],"stream":true}"#;
#[test]
fn llm_run() {
    let scratch = support::Scratch::new("fake_llm_tls");
    let dir = scratch.path();
    let doc = dir.join("llm.json");
    document(&doc, &llm_document(dir, true));
    let run = Running::start("llm", &doc, None).unwrap_or_else(|why| panic!("{why}"));
    let key = temper_fake::credential::credential(23, temper_fake::credential::KEY);
    let key_path = dir.join("key");
    private_hex(&key_path, &key);
    let chat: Value = serde_json::from_slice(CHAT).expect("valid fixture");
    let chat = serde_json::to_vec(&chat).expect("valid fixture");
    let before = snapshot(dir);
    let auth = format!("Bearer {}", std::str::from_utf8(&key).expect("valid fixture"));
    for (credential, status, text) in
        [(&*auth, b"HTTP/1.1 200", "hello from QA"), ("Bearer wrong", b"HTTP/1.1 401", "invalid_api_key")]
    {
        assert_eq!(fs::read(dir.join("root.der")).expect("valid fixture"), skein_fake_peers::transport::ROOT);
        assert_eq!(fs::metadata(dir.join("root.der")).expect("valid fixture").permissions().mode() & 0o777, 0o600);
        let mut roots = rustls_std::RootCertStore::empty();
        roots
            .add(rustls_std::pki_types::CertificateDer::from(fs::read(dir.join("root.der")).expect("valid fixture")))
            .expect("valid fixture");
        let config =
            rustls_std::ClientConfig::builder_with_provider(Arc::new(rustls_std::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .expect("valid fixture")
                .with_root_certificates(roots)
                .with_no_client_auth();
        let connection = rustls_std::ClientConnection::new(
            Arc::new(config),
            rustls_std::pki_types::ServerName::try_from("skein.test").expect("valid fixture"),
        )
        .expect("valid fixture");
        let mut stream = rustls_std::StreamOwned::new(connection, stream(&run, run.address));
        stream.write_all(&http("POST", "/v1/responses", credential, &chat)).expect("valid fixture");
        let mut response = Vec::new();
        stream.sock.reset("the fake TLS socket's reply");
        match stream.read_to_end(&mut response) {
            Ok(_) => {}
            Err(why) if why.kind() == std::io::ErrorKind::UnexpectedEof => {}
            Err(why) => panic!("TLS response: {why}"),
        }
        assert!(response.starts_with(status), "{}", String::from_utf8_lossy(&response));
        assert!(String::from_utf8_lossy(&response).contains(text), "{}", String::from_utf8_lossy(&response));
    }
    run.stop();
    assert_eq!(snapshot(dir), before, "TLS files and modes retained after stop");
    assert_eq!(fs::read(key_path).expect("key retained"), key);
    scratch.remove();

    let scratch = support::Scratch::new("fake_llm_demo");
    let dir = scratch.path();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("box's held scripted listener");
    let listen = listener.local_addr().expect("chosen scripted address");
    fs::create_dir(dir.join("demo")).expect("demo directory");
    let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    support::write_private(&dir.join("demo/llm.key"), format!("{key}\n").as_bytes());
    let mut command = crate::listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(["fake", "llm"]).args(support::configuration::flags(dir));
    let run = Running::command(command, None).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(run.address, listen, "ready on the box's inherited scripted listener");
    let reply = request(&run, run.address, "POST", "/v1/responses", &format!("Bearer {key}"), &chat);
    assert!(reply.starts_with(b"HTTP/1.1 200"), "{}", String::from_utf8_lossy(&reply));
    assert!(String::from_utf8_lossy(&reply).contains("I am the scripted model"));
    run.stop();
    scratch.remove();
}
#[test]
fn ready_write_failure_stops_and_keeps_outputs() {
    let scratch = support::Scratch::new("fake_ready_failure");
    let doc = scratch.path().join("llm.json");
    document(&doc, &llm_document(scratch.path(), false));
    let (reader, writer) = std::io::pipe().expect("ready pipe");
    drop(reader);
    let output = command("llm", &doc).args(["--ready", ""]).stdout(writer).output().expect("valid fixture");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"temper fake llm: stdout: Broken pipe (os error 32)\n");
    assert!(scratch.path().join("key").exists());
    scratch.remove();
}

#[test]
fn git_run() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    use temper_worker_protocol_git::invocation::{self, Command as GitCommand};

    let scratch = support::Scratch::new("fake_git");
    let dir = scratch.path();
    let binary = skein_scratch::paths::binary("temper-box");
    let git = dir.join("bin/git");
    fs::create_dir(dir.join("bin")).expect("git's directory");
    std::os::unix::fs::symlink(&binary, &git).expect("git link");
    let home = dir.join("home");
    let work = dir.join("work");
    fs::create_dir(&home).expect("git's HOME");
    fs::create_dir(&work).expect("worker directory");
    let before = snapshot(&work);
    let arguments = invocation::arguments(&GitCommand::Version);
    let environment = invocation::environment(home.as_os_str().as_bytes(), None, None);
    for (name, program, prefix) in [("version", &git, &[][..]), ("verb", &binary, &["fake", "git"][..])] {
        let mut command = Command::new(program);
        command
            .env_clear()
            .current_dir(&work)
            .stdin(Stdio::piped())
            .args(prefix)
            .args(arguments.iter().map(|word| OsStr::from_bytes(word)));
        for entry in &environment {
            let split = entry.iter().position(|b| *b == b'=').expect("environment entry");
            command.env(OsStr::from_bytes(&entry[..split]), OsStr::from_bytes(&entry[split + 1..]));
        }
        let output = command.output().expect("git exits");
        assert_eq!(output.status.code(), Some(0), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, b"git version 2.47.3\n", "{name}: stdout");
        assert_eq!(output.stderr, b"", "{name}: stderr");
        assert_eq!(snapshot(&work), before, "{name}: nothing written");
    }
    scratch.remove();
}

#[test]
fn check_run() {
    let scratch = support::Scratch::startup("fake-check-run");
    let directory = scratch.path();
    let binary = skein_scratch::paths::binary("temper-box");
    let (agent, _) = support::configuration::documents(directory, binary.to_str().expect("a UTF-8 binary path"));
    support::write_private(&directory.join("root.der"), skein_tls_kit::pki::ROOT);
    let data = directory.join("data");
    fs::create_dir(&data).expect("fixture data");
    let mut value = support::configuration::check_document(directory);
    temper_agent_fixtures::apply(&mut value, &data, &serde_json::from_slice(&agent).expect("agent JSON"));
    support::write_file(
        &directory.join(temper_config_shell::FILE),
        &support::configuration::file(),
        temper_config_shell::Shape::Standalone,
    );
    let observations = directory.join("observations");
    fs::write(&observations, b"{\"observations_lost\":0}\n").expect("checkpoint");
    let store = directory.join("copy");
    let cache = directory.join("cache");
    let entries_before = fs::read_dir(&data).expect("fixture entries").count();
    let ended = Command::new(binary)
        .env_clear()
        .args(["fake", "forge", "check", "--config"])
        .arg(directory)
        .arg("--data")
        .arg(&data)
        .arg("--cache")
        .arg(&cache)
        .arg("1")
        .arg(&observations)
        .arg(&store)
        .output()
        .expect("check process");
    assert_eq!(ended.status.code(), Some(2), "{}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.stdout, b"{\"undecided\":\"store unreadable\"}\n");
    assert_eq!(
        ended.stderr,
        format!(
            "{{\"boundary\":{{\"lines\":1,\"lost\":0}}}}\ntemper-box fake forge check: opening {}: errno 2\n",
            store.display()
        )
        .as_bytes()
    );
    assert_eq!(
        fs::read_dir(&data).expect("fixture data").count(),
        entries_before,
        "the check creates no data resources"
    );
    assert!(data.join("agents.setting").is_file());
    assert!(!cache.exists());
    scratch.remove();
}

#[test]
fn the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap() {
    let scratch = support::Scratch::startup("fake-oauth-cap");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let path = scratch.path().join("fake.json");
    document(
        &path,
        &json!({"listen":{"socket":address.to_string()},"redirect_port":1455,"redirect_path":"auth/callback"}),
    );
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(["fake", "oauth", "--document"]).arg(path);
    let run = Running::command(command, None).unwrap_or_else(|why| panic!("{why}"));
    let sockets: Vec<_> = (0..temper_fake::oauth::CONNECTIONS).map(|_| stream(&run, run.address)).collect();
    let extra = stream(&run, run.address);
    let expected =
        format!("temper fake oauth: rejected a connection at the cap of {}", temper_fake::oauth::CONNECTIONS);
    let deadline = run.deadline("the fake issuer's cap rejection log");
    let mut stderr = Vec::new();
    loop {
        if deadline.passed() {
            match deadline.expired() {
                Expired::Starved(starved) => {
                    run.end();
                    starved.pass();
                }
                Expired::Failed(figures) => {
                    panic!("missing stderr line {expected:?}: {figures}; stderr {:?}", String::from_utf8_lossy(&stderr))
                }
            }
        }
        let mut child = run.child.borrow_mut();
        let pipe = child.as_mut().expect("running fake").stderr.as_mut().expect("stderr pipe");
        if !skein_shell::keeper::wait_readable(skein_io::kernel::Fd::new(pipe.as_raw_fd()), deadline.remaining())
            .expect("stderr readiness")
        {
            continue;
        }
        let mut bytes = [0; 1024];
        let count = pipe.read(&mut bytes).expect("stderr read");
        assert_ne!(count, 0, "the fake exited before logging: {:?}", String::from_utf8_lossy(&stderr));
        stderr.extend_from_slice(&bytes[..count]);
        if String::from_utf8_lossy(&stderr).lines().any(|line| line == expected) {
            break;
        }
    }
    drop(extra);
    drop(sockets);
    stderr.extend_from_slice(&run.stop_with_stderr());
    assert_eq!(String::from_utf8(stderr).expect("UTF-8 log"), format!("{expected}\n"));
    scratch.remove();
}

#[test]
fn fakes_claim_their_inherited_listeners_before_the_first_drive() {
    for kind in ["llm", "forge", "oauth"] {
        let scratch = support::Scratch::startup(&format!("fake-inherited-{kind}"));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let mut value = match kind {
            "llm" => llm_document(scratch.path(), false),
            "forge" => forge_document(scratch.path()),
            "oauth" => {
                json!({"listen":{"socket":address.to_string()},"redirect_port":1455,"redirect_path":"auth/callback"})
            }
            _ => unreachable!("table kind"),
        };
        value["listen"] = json!({"socket":address.to_string()});
        let path = scratch.path().join("fake.json");
        document(&path, &value);
        let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
        command.args(["fake", kind, "--document"]).arg(path).args(["--ready", "inherited "]);
        let ended = listeners::probe(command, address, "inherited ");
        assert!(ended.ready.is_some(), "fake ready: {:?}", ended.stderr);
        assert!(ended.reply.expect("fake answers").starts_with(b"HTTP/1.1 "));
        assert!(ended.status.success(), "fake drains: {:?}", ended.stderr);
        scratch.remove();
    }
}
