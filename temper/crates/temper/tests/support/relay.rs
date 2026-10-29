//! Owner-run GitHub relay: bounded HTTP, skein TLS, one guard and ledger (domain/testing.md, 2.7).
use super::socket::Socket;
use skein_lib::Duration;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

pub const CAP: usize = 400;
pub const TEARDOWN: usize = 20;
const BOUND: usize = 16 * 1024 * 1024;
const HEAD: usize = 64 * 1024;
type Result<T> = io::Result<T>;
pub type Row = (String, String, String, u16);
fn invalid() -> io::Error {
    io::Error::other("relay: invalid or oversized HTTP message")
}

#[derive(Clone)]
pub struct Guard {
    pub owner: String,
    pub repository: String,
    pub installation: u64,
    /// The run's branch prefix: the only refs a write may name.
    pub prefix: String,
}
impl Guard {
    /// `body` is `None` when the request's content is encoded and so unread; `id` is the probe
    /// repository's, once a read of it has shown it.
    #[must_use]
    pub fn allows(&self, listener: &str, method: &str, target: &str, body: Option<&[u8]>, id: Option<u64>) -> bool {
        let path = target.split('?').next().unwrap_or("");
        if !path.starts_with('/')
            || path.split('/').any(|segment| segment == "." || segment == "..")
            || path.to_ascii_lowercase().contains("%2e")
            || path.contains("//")
            || path.contains(['\\', '#'])
            || path.bytes().any(|b| b <= 32 || b == 127)
        {
            return false;
        }
        let repo = format!("/repos/{}/{}", self.owner, self.repository);
        if listener == "git" {
            let prefix = format!("/{}/{}.git/", self.owner, self.repository);
            return match (method, path.strip_prefix(&prefix)) {
                ("GET", Some("info/refs")) | ("POST", Some("git-upload-pack")) => true,
                ("POST", Some("git-receive-pack")) => body.is_some_and(|body| self.pushes(body)),
                _ => false,
            };
        }
        if listener != "api" {
            return false;
        }
        if path == repo {
            return method == "GET";
        }
        if let Some(rest) = path.strip_prefix(&format!("{repo}/")).filter(|rest| !rest.is_empty()) {
            return matches!(method, "GET" | "HEAD") || self.writes(method, rest, body);
        }
        if method == "POST" {
            return match path {
                "/graphql" => body.is_some_and(|body| self.reads(body)),
                _ => path == format!("/app/installations/{}/access_tokens", self.installation),
            };
        }
        if method == "DELETE" {
            return path == "/installation/token";
        }
        if method != "GET" {
            return false;
        }
        if matches!(
            path,
            "/app" | "/app/hook/config" | "/app/hook/deliveries" | "/app/installations" | "/installation/repositories"
        ) {
            return true;
        }
        for prefix in ["/app/hook/deliveries/", "/app/installations/", "/user/"] {
            if path.strip_prefix(prefix).is_some_and(number) {
                return true;
            }
        }
        if path.strip_prefix("/users/").is_some_and(|s| !s.is_empty() && !s.contains('/')) {
            return true;
        }
        // Settings by id (skein's Request::Repository): the probe's own id alone.
        path.strip_prefix("/repositories/").is_some_and(|s| id.is_some_and(|id| s == id.to_string()))
    }
    /// The probe's writes are temper's own routes (skein-forge github/request.rs), refs the run's.
    fn writes(&self, method: &str, rest: &str, body: Option<&[u8]>) -> bool {
        let parts: Vec<&str> = rest.split('/').collect();
        match (method, parts.as_slice()) {
            ("POST", ["pulls" | "issues"]) => true,
            ("POST", ["git", "refs"]) => body
                .and_then(|body| serde_json::from_slice::<serde_json::Value>(body).ok())
                .is_some_and(|value| value["ref"].as_str().is_some_and(|name| self.held(name))),
            ("DELETE", ["git", "refs", "heads", ..]) => self.held(&format!("refs/{}", &rest["git/refs/".len()..])),
            ("PATCH", ["pulls" | "issues", n])
            | ("PUT", ["pulls", n, "merge" | "update-branch"])
            | ("POST", ["pulls", n, "reviews"] | ["issues", n, "comments"])
            | ("POST" | "DELETE", ["pulls", n, "requested_reviewers"]) => number(n),
            ("POST", ["statuses", sha]) => sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
            _ => false,
        }
    }
    fn held(&self, name: &str) -> bool {
        name.strip_prefix("refs/heads/")
            .and_then(|branch| branch.strip_prefix(self.prefix.as_str()))
            .is_some_and(|rest| !rest.is_empty())
    }
    /// GraphQL carries skein's two read documents alone, the pull's on the probe; never a mutation.
    fn reads(&self, body: &[u8]) -> bool {
        use skein_forge::github::document::{PULL, VIEWER};
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else { return false };
        match value["query"].as_str().map(str::as_bytes) {
            Some(VIEWER) => true,
            Some(PULL) => value["variables"]["owner"] == self.owner && value["variables"]["name"] == self.repository,
            _ => false,
        }
    }
    /// A receive-pack's commands (pkt-lines up to the flush) name the run's branches alone.
    fn pushes(&self, body: &[u8]) -> bool {
        let mut at = 0;
        let mut commands = 0;
        loop {
            let Some(length) = body
                .get(at..at + 4)
                .and_then(|hex| std::str::from_utf8(hex).ok())
                .and_then(|hex| usize::from_str_radix(hex, 16).ok())
            else {
                return false;
            };
            if length == 0 {
                return commands > 0;
            }
            let Some(line) = body.get(at + 4..at + length).filter(|_| length > 4) else { return false };
            let command = line.split(|b| *b == 0).next().unwrap_or(line);
            let command = command.strip_suffix(b"\n").unwrap_or(command);
            let mut fields = command.splitn(3, |b| *b == b' ');
            let (Some(_), Some(_), Some(name)) = (fields.next(), fields.next(), fields.next()) else { return false };
            if !std::str::from_utf8(name).is_ok_and(|name| self.held(name)) {
                return false;
            }
            commands += 1;
            at += length;
        }
    }
}
fn number(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

#[derive(Default)]
pub struct Counter {
    count: usize,
}
impl Counter {
    /// Take a forwarding slot; refusals never consume the cleanup reserve.
    pub fn take(&mut self, teardown: bool) -> std::result::Result<(), u16> {
        if self.count >= if teardown { CAP } else { CAP - TEARDOWN } {
            return Err(429);
        }
        self.count += 1;
        Ok(())
    }
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }
}
struct Ledger {
    counter: Counter,
    journal: Vec<Row>,
    refused: Vec<Row>,
    tokens: Vec<Vec<u8>>,
    merge_heads: Vec<String>,
    /// The probe repository's id, from the first read of it.
    repository: Option<u64>,
    file: File,
}
struct Shared {
    guard: Guard,
    ledger: Mutex<Ledger>,
    directory: PathBuf,
    upstream: Upstream,
    stop: AtomicBool,
}
#[derive(Clone)]
enum Upstream {
    Tls(skein_tls::Config),
    Plain(SocketAddr),
}

pub struct Relay {
    pub api: SocketAddr,
    pub git: SocketAddr,
    shared: Arc<Shared>,
    listeners: Vec<JoinHandle<()>>,
    connections: Arc<Mutex<Vec<TcpStream>>>,
    threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
}
impl Relay {
    #[must_use]
    pub fn start(guard: Guard) -> Self {
        let roots = skein_shell::read_trust_roots(
            &skein_shell::Trust::Machine { ssl_cert_file: None },
            &skein_shell::RootBounds { bundle_bytes: 4_194_304, certificates: 1024, certificate_bytes: 65536 },
        )
        .expect("machine roots");
        let (tls, _) = skein_tls::Config::from_der(&roots.certificates, &[b"http/1.1"]).expect("relay TLS");
        Self::open(guard, Path::new(&std::env::var_os("TMPDIR").expect("private live scratch")), Upstream::Tls(tls))
    }
    /// Local-only connector used by the default suite; never loads trust or resolves a host.
    #[must_use]
    pub fn plaintext(guard: Guard, directory: &Path, address: SocketAddr) -> Self {
        assert!(address.ip().is_loopback());
        Self::open(guard, directory, Upstream::Plain(address))
    }
    #[expect(clippy::disallowed_methods, reason = "the relay is a real socket peer on owned test threads")]
    fn open(guard: Guard, directory: &Path, upstream: Upstream) -> Self {
        let api = TcpListener::bind("127.0.0.1:0").expect("relay API");
        let git = TcpListener::bind("127.0.0.1:0").expect("relay git");
        let addresses = [api.local_addr().expect("API address"), git.local_addr().expect("git address")];
        let file = OpenOptions::new()
            .create_new(true)
            .mode(0o600)
            .write(true)
            .open(directory.join("relay.jsonl"))
            .expect("private relay journal");
        super::write_private(&directory.join("count.txt"), b"0\n");
        let shared = Arc::new(Shared {
            guard,
            directory: directory.into(),
            upstream,
            stop: AtomicBool::new(false),
            ledger: Mutex::new(Ledger {
                counter: Counter::default(),
                journal: vec![],
                refused: vec![],
                tokens: vec![],
                merge_heads: vec![],
                repository: None,
                file,
            }),
        });
        let connections = Arc::new(Mutex::new(Vec::new()));
        let threads = Arc::new(Mutex::new(Vec::new()));
        let mut listeners = Vec::new();
        for (name, listener) in [("api", api), ("git", git)] {
            let state = shared.clone();
            let sockets = connections.clone();
            let children = threads.clone();
            listeners.push(thread::spawn(move || {
                for accepted in listener.incoming() {
                    if state.stop.load(Ordering::Acquire) {
                        break;
                    }
                    let Ok(socket) = accepted else { break };
                    sockets.lock().expect("sockets").push(socket.try_clone().expect("socket clone"));
                    let state = state.clone();
                    children.lock().expect("threads").push(thread::spawn(move || serve(&state, name, socket)));
                }
            }));
        }
        Self { api: addresses[0], git: addresses[1], shared, listeners, connections, threads }
    }
    #[must_use]
    pub fn count(&self) -> usize {
        self.shared.ledger.lock().expect("ledger").counter.count()
    }
    #[must_use]
    pub fn journal(&self) -> Vec<Row> {
        self.shared.ledger.lock().expect("ledger").journal.clone()
    }
    #[must_use]
    pub fn refused(&self) -> Vec<Row> {
        self.shared.ledger.lock().expect("ledger").refused.clone()
    }
    #[must_use]
    pub fn tokens(&self) -> Vec<Vec<u8>> {
        self.shared.ledger.lock().expect("ledger").tokens.clone()
    }
    /// Expected heads submitted by temper's merge calls, retained only in memory.
    #[must_use]
    pub fn merge_heads(&self) -> Vec<String> {
        self.shared.ledger.lock().expect("ledger").merge_heads.clone()
    }
    pub fn ask(&self, method: &str, path: &str, authorization: &[u8], body: &[u8]) -> Result<(u16, Vec<u8>)> {
        self.request(false, method, path, authorization, body)
    }
    pub fn teardown(&self, method: &str, path: &str, authorization: &[u8], body: &[u8]) -> Result<(u16, Vec<u8>)> {
        self.request(true, method, path, authorization, body)
    }
    fn request(
        &self,
        teardown: bool,
        method: &str,
        path: &str,
        authorization: &[u8],
        body: &[u8],
    ) -> Result<(u16, Vec<u8>)> {
        if authorization.iter().any(|b| matches!(b, b'\r' | b'\n')) || path.bytes().any(|b| b <= 32) {
            return Err(invalid());
        }
        let mut wire = format!("{method} {path} HTTP/1.1\r\nHost: api.github.com\r\nUser-Agent: temper-live-relay\r\nAccept: application/vnd.github+json\r\nContent-Type: application/json\r\nAuthorization: ").into_bytes();
        wire.extend_from_slice(authorization);
        wire.extend_from_slice(format!("\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes());
        wire.extend_from_slice(body);
        if wire.len() > BOUND {
            return Err(invalid());
        }
        let head = wire.windows(4).position(|s| s == b"\r\n\r\n").ok_or_else(invalid)? + 4;
        let message = Message { wire, head };
        let answer = self.shared.exchange("api", &message, teardown)?;
        let status = answer.status()?;
        let body = if method == "HEAD" || matches!(status, 204 | 304) { Vec::new() } else { answer.body()? };
        Ok((status, body))
    }
}
impl Relay {
    /// Stop and join client threads before cleanup, so every in-flight mint is known.
    pub fn quiesce(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if !self.listeners.is_empty() {
            for address in [self.api, self.git] {
                let _ = TcpStream::connect(address);
            }
        }
        for handle in self.listeners.drain(..) {
            let _ = handle.join();
        }
        for socket in self.connections.lock().expect("connections").iter() {
            let _ = socket.shutdown(Shutdown::Both);
        }
        for handle in self.threads.lock().expect("threads").drain(..) {
            let _ = handle.join();
        }
    }
}
impl Drop for Relay {
    fn drop(&mut self) {
        self.quiesce();
    }
}
impl Shared {
    fn record(&self, listener: &str, method: &str, path: &str, status: u16, refusal: bool) -> Result<()> {
        let mut ledger = self.ledger.lock().map_err(|_| invalid())?;
        let row = (listener.into(), method.into(), path.into(), status);
        serde_json::to_writer(&mut ledger.file, &row)?;
        ledger.file.write_all(b"\n")?;
        ledger.file.flush()?;
        if refusal {
            ledger.refused.push(row.clone());
        }
        ledger.journal.push(row);
        Ok(())
    }
    fn exchange(&self, listener: &str, request: &Message, teardown: bool) -> Result<Message> {
        let (method, target) = request.request()?;
        let path = target.split('?').next().ok_or_else(invalid)?;
        let body = if request.header("content-encoding")?.is_some() { None } else { Some(request.body()?) };
        let id = self.ledger.lock().map_err(|_| invalid())?.repository;
        let refusal = if self.guard.allows(listener, method, target, body.as_deref(), id) {
            let mut ledger = self.ledger.lock().map_err(|_| invalid())?;
            let refusal = ledger.counter.take(teardown).err();
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .mode(0o600)
                .open(self.directory.join("count.next"))?
                .write_all(format!("{}\n", ledger.counter.count()).as_bytes())?;
            fs::rename(self.directory.join("count.next"), self.directory.join("count.txt"))?;
            refusal
        } else {
            Some(403)
        };
        if let Some(status) = refusal {
            self.record(listener, method, path, status, true)?;
            return Ok(Message::answer(status, b"relay refused request"));
        }
        if listener == "api" && method == "PUT" && path.ends_with("/merge") {
            let value: serde_json::Value = serde_json::from_slice(&request.body()?)?;
            let sha = value["sha"].as_str().ok_or_else(invalid)?;
            if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid());
            }
            self.ledger.lock().map_err(|_| invalid())?.merge_heads.push(sha.into());
        }
        let host = if listener == "api" { "api.github.com" } else { "github.com" };
        let wire = request.with_host(host)?;
        let answer = match &self.upstream {
            Upstream::Tls(tls) => tls_exchange(tls, host, &wire, method),
            Upstream::Plain(address) => {
                let mut socket = Socket::connect(
                    *address,
                    "the relay upstream's connection",
                    Duration::from_secs(30),
                    ["the relay upstream's reply", "the relay upstream's request"],
                    true,
                    None,
                )?;
                socket.write_all(&wire)?;
                read_message(&mut io::BufReader::new(socket), Some(method))?.ok_or_else(invalid)
            }
        };
        let answer = match answer {
            Ok(answer) => answer,
            Err(error) => {
                // An io error names no credential; the path has no query.
                eprintln!("live relay: upstream failed for {method} {path}: {error}");
                self.record(listener, method, path, 502, true)?;
                return Ok(Message::answer(502, b"relay upstream failed"));
            }
        };
        let status = answer.status()?;
        if status == 201
            && method == "POST"
            && path == format!("/app/installations/{}/access_tokens", self.guard.installation)
        {
            let value: serde_json::Value = serde_json::from_slice(&answer.body()?)?;
            let token = value["token"].as_str().ok_or_else(invalid)?.as_bytes().to_vec();
            self.ledger.lock().map_err(|_| invalid())?.tokens.push(token);
        }
        if status == 200
            && answer.header("content-encoding")?.is_none()
            && listener == "api"
            && method == "GET"
            && path == format!("/repos/{}/{}", self.guard.owner, self.guard.repository)
        {
            let value: serde_json::Value = serde_json::from_slice(&answer.body()?)?;
            let id = value["id"].as_u64().ok_or_else(invalid)?;
            let mut ledger = self.ledger.lock().map_err(|_| invalid())?;
            if ledger.repository.is_some_and(|known| known != id) {
                return Err(invalid());
            }
            ledger.repository = Some(id);
        }
        self.record(listener, method, path, status, false)?;
        Ok(answer)
    }
}
fn serve(shared: &Shared, listener: &str, socket: TcpStream) {
    // The relay serves world-owned children; only their owner can pass after cleanup.
    let mut socket = io::BufReader::new(Socket::deferred(
        socket,
        Duration::from_secs(60),
        ["the relay client's request", "the relay client's reply"],
    ));
    let result = (|| -> Result<()> {
        while !shared.stop.load(Ordering::Acquire) {
            let Some(mut request) = read_head(&mut socket)? else { break };
            if request.header("expect")?.is_some_and(|s| s.eq_ignore_ascii_case("100-continue")) {
                socket.get_mut().write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
            }
            read_body(&mut socket, &mut request, false)?;
            let (method, _) = request.request()?;
            let close = request
                .header("connection")?
                .is_some_and(|value| value.split(',').any(|token| token.trim().eq_ignore_ascii_case("close")));
            let answer = shared.exchange(listener, &request, false)?;
            socket.get_mut().write_all(&answer.framed(method)?)?;
            if close {
                break;
            }
        }
        Ok(())
    })();
    // The quiesce list holds a clone: only a shutdown sends the FIN a graceful close waits for.
    let _ = socket.get_ref().shutdown(Shutdown::Both);
    if result.is_err() && !shared.stop.load(Ordering::Acquire) {
        let _ = shared.record(listener, "INVALID", "/", 400, true);
    }
}

/// Original request framing is retained; answers are dechunked for a persistent client connection.
struct Message {
    wire: Vec<u8>,
    head: usize,
}
impl Message {
    fn head(&self) -> Result<&str> {
        std::str::from_utf8(&self.wire[..self.head]).map_err(|_| invalid())
    }
    fn request(&self) -> Result<(&str, &str)> {
        let mut parts = self.head()?.lines().next().ok_or_else(invalid)?.split(' ');
        let method = parts.next().ok_or_else(invalid)?;
        let path = parts.next().ok_or_else(invalid)?;
        if parts.next() != Some("HTTP/1.1") || parts.next().is_some() {
            return Err(invalid());
        }
        Ok((method, path))
    }
    fn status(&self) -> Result<u16> {
        self.head()?.split_whitespace().nth(1).ok_or_else(invalid)?.parse().map_err(|_| invalid())
    }
    fn header(&self, name: &str) -> Result<Option<&str>> {
        let mut found = None;
        for line in self.head()?.split("\r\n").skip(1).filter(|s| !s.is_empty()) {
            let (key, value) = line.split_once(':').ok_or_else(invalid)?;
            if key.eq_ignore_ascii_case(name) {
                if found.is_some() {
                    return Err(invalid());
                }
                found = Some(value.trim());
            }
        }
        Ok(found)
    }
    fn with_host(&self, host: &str) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        for (index, line) in self.head()?.split("\r\n").filter(|s| !s.is_empty()).enumerate() {
            if index == 0 || !line.split_once(':').is_some_and(|(k, _)| k.eq_ignore_ascii_case("host")) {
                output.extend_from_slice(line.as_bytes());
                output.extend_from_slice(b"\r\n");
            }
        }
        output.extend_from_slice(format!("Host: {host}\r\n\r\n").as_bytes());
        output.extend_from_slice(&self.wire[self.head..]);
        Ok(output)
    }
    fn body(&self) -> Result<Vec<u8>> {
        if self.header("transfer-encoding")?.is_some() {
            let mut cursor = io::Cursor::new(&self.wire[self.head..]);
            let mut raw = Vec::new();
            return chunks(&mut cursor, &mut raw);
        }
        Ok(self.wire[self.head..].to_vec())
    }
    fn framed(&self, method: &str) -> Result<Vec<u8>> {
        let body = if method == "HEAD" || matches!(self.status()?, 204 | 304) { Vec::new() } else { self.body()? };
        let mut wire = Vec::new();
        for (index, line) in self.head()?.split("\r\n").filter(|s| !s.is_empty()).enumerate() {
            if index > 0
                && line.split_once(':').is_some_and(|(k, _)| {
                    ["content-length", "transfer-encoding", "connection", "trailer"]
                        .iter()
                        .any(|name| k.eq_ignore_ascii_case(name))
                })
            {
                continue;
            }
            wire.extend_from_slice(line.as_bytes());
            wire.extend_from_slice(b"\r\n");
        }
        if method == "HEAD" {
            if let Some(length) = self.header("content-length")? {
                wire.extend_from_slice(format!("Content-Length: {length}\r\n").as_bytes());
            }
        } else if !matches!(self.status()?, 204 | 304) {
            wire.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
        }
        wire.extend_from_slice(b"\r\n");
        wire.extend_from_slice(&body);
        Ok(wire)
    }
    fn answer(status: u16, body: &[u8]) -> Self {
        let mut wire = format!("HTTP/1.1 {status} Relay\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
        let head = wire.len();
        wire.extend_from_slice(body);
        Self { wire, head }
    }
}
fn line(reader: &mut impl Read, output: &mut Vec<u8>, bound: usize) -> Result<bool> {
    let start = output.len();
    loop {
        let mut byte = [0];
        let count = match reader.read(&mut byte) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            count => count?,
        };
        if count == 0 {
            return if output.len() == start { Ok(false) } else { Err(invalid()) };
        }
        output.push(byte[0]);
        if output.len() > bound {
            return Err(invalid());
        }
        if output.ends_with(b"\r\n") {
            return Ok(true);
        }
    }
}
fn append(reader: &mut impl Read, output: &mut Vec<u8>, count: usize) -> Result<()> {
    let end = output.len().checked_add(count).filter(|n| *n <= BOUND).ok_or_else(invalid)?;
    let start = output.len();
    output.resize(end, 0);
    reader.read_exact(&mut output[start..])
}
fn chunks(reader: &mut impl Read, raw: &mut Vec<u8>) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let start = raw.len();
        if !line(reader, raw, BOUND)? {
            return Err(invalid());
        }
        let size = std::str::from_utf8(&raw[start..raw.len() - 2])
            .map_err(|_| invalid())?
            .split(';')
            .next()
            .ok_or_else(invalid)?;
        let size = usize::from_str_radix(size, 16).map_err(|_| invalid())?;
        if size == 0 {
            loop {
                let start = raw.len();
                if !line(reader, raw, BOUND)? {
                    return Err(invalid());
                }
                if raw.len() == start + 2 {
                    return Ok(body);
                }
            }
        }
        let start = raw.len();
        append(reader, raw, size.checked_add(2).ok_or_else(invalid)?)?;
        if !raw.ends_with(b"\r\n") {
            return Err(invalid());
        }
        body.extend_from_slice(&raw[start..start + size]);
    }
}
fn read_head(reader: &mut impl Read) -> Result<Option<Message>> {
    let mut wire = Vec::new();
    loop {
        if !line(reader, &mut wire, HEAD)? {
            return if wire.is_empty() { Ok(None) } else { Err(invalid()) };
        }
        if wire.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    Ok(Some(Message { head: wire.len(), wire }))
}
fn read_message(reader: &mut impl Read, response_to: Option<&str>) -> Result<Option<Message>> {
    // Bound interim answers too: a peer cannot evade the answer bound with 100s.
    for _ in 0..16 {
        let Some(mut message) = read_head(reader)? else { return Ok(None) };
        if response_to.is_some() {
            let status = message.status()?;
            if (100..200).contains(&status) {
                continue;
            }
            if response_to == Some("HEAD") || matches!(status, 204 | 304) {
                return Ok(Some(message));
            }
        }
        read_body(reader, &mut message, response_to.is_some())?;
        return Ok(Some(message));
    }
    Err(invalid())
}
fn read_body(reader: &mut impl Read, message: &mut Message, answer: bool) -> Result<()> {
    let length = message.header("content-length")?.map(str::parse::<usize>).transpose().map_err(|_| invalid())?;
    let transfer = message.header("transfer-encoding")?;
    if transfer.is_some() && length.is_some() {
        return Err(invalid());
    }
    if let Some(transfer) = transfer {
        if !transfer.eq_ignore_ascii_case("chunked") {
            return Err(invalid());
        }
        chunks(reader, &mut message.wire)?;
    } else if let Some(length) = length {
        append(reader, &mut message.wire, length)?;
    } else if answer {
        reader.take(u64::try_from(BOUND - message.head + 1).map_err(|_| invalid())?).read_to_end(&mut message.wire)?;
        if message.wire.len() > BOUND {
            return Err(invalid());
        }
    }
    Ok(())
}

/// Blocking adapter drives the same skein TLS machine as conformance's Connector.
/// A buffered reader above it parses each byte once, including close-delimited answers.
struct TlsReader {
    socket: Socket<'static>,
    tls: skein_tls::client::Client,
    env: skein_lib::Env<skein_tls::client::Limits>,
    above: skein_lib::Queue<skein_tls::client::Event>,
    below: skein_lib::Queue<skein_lib::stream::Down>,
    intake: skein_lib::Intake,
    wire: Vec<u8>,
    sent: usize,
    pending: usize,
    bytes: Vec<u8>,
    offset: usize,
}
impl Read for TlsReader {
    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        use skein_lib::stream::{Down, Read as Demand, Up};
        use skein_tls::client::{self, Event, Request};
        if output.is_empty() {
            return Ok(0);
        }
        for _ in 0..100_000 {
            if self.offset < self.bytes.len() {
                let count = output.len().min(self.bytes.len() - self.offset);
                output[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
                self.offset += count;
                return Ok(count);
            }
            if let Some(event) = self.above.pop() {
                let demand = match event {
                    Event::Ready(_) => {
                        self.pending = self.wire.len().min(16384);
                        Down::Demand {
                            read: Demand::Nothing,
                            room: u32::try_from(self.pending).map_err(|_| invalid())?,
                        }
                    }
                    Event::Stream(Up::Room) => {
                        let end = self.sent + self.pending;
                        client::down(
                            &mut self.tls,
                            &self.env,
                            Request::Stream(Down::Send(self.wire[self.sent..end].into())),
                            &mut self.above,
                            &mut self.below,
                        );
                        self.sent = end;
                        self.pending = (self.wire.len() - self.sent).min(16384);
                        Down::Demand {
                            read: if self.pending == 0 { Demand::Arrived { max: 16384 } } else { Demand::Nothing },
                            room: u32::try_from(self.pending).map_err(|_| invalid())?,
                        }
                    }
                    Event::Stream(Up::Bytes(bytes)) => {
                        self.bytes = bytes.into_vec();
                        self.offset = 0;
                        Down::Demand { read: Demand::Arrived { max: 16384 }, room: 0 }
                    }
                    Event::Stream(Up::End) => return Ok(0),
                    Event::Stream(Up::Failed(_)) | Event::Failed(_) | Event::Closed => {
                        return Err(io::Error::other("relay TLS failed"));
                    }
                };
                client::down(&mut self.tls, &self.env, Request::Stream(demand), &mut self.above, &mut self.below);
                continue;
            }
            let effect = self.below.pop().ok_or_else(invalid)?;
            let up = match effect {
                Down::Send(bytes) => {
                    self.socket.write_all(&bytes)?;
                    continue;
                }
                Down::Finish => return Ok(0),
                Down::Demand { room, .. } if room > 0 => Up::Room,
                Down::Demand { read, .. } => loop {
                    if let Some(bytes) = self.intake.meet(read) {
                        break Up::Bytes(bytes);
                    }
                    let mut bytes = [0; 16384];
                    // A child's signal interrupts a read; the read is asked again.
                    let count = match self.socket.read(&mut bytes) {
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        count => count?,
                    };
                    if count == 0 {
                        break Up::End;
                    }
                    self.intake.append(&bytes[..count]).map_err(|_| invalid())?;
                },
            };
            client::up(&mut self.tls, &self.env, up, &mut self.above, &mut self.below);
        }
        Err(io::Error::other("relay TLS progress bound"))
    }
}
fn tls_exchange(config: &skein_tls::Config, host: &str, wire: &[u8], method: &str) -> Result<Message> {
    use skein_tls::client::{self, Request};
    let address = (host, 443).to_socket_addrs()?.next().ok_or_else(invalid)?;
    let socket = Socket::connect(
        address,
        "the TLS relay upstream's connection",
        Duration::from_secs(30),
        ["the TLS relay upstream's reply", "the TLS relay upstream's request"],
        true,
        None,
    )?;
    let limits = client::Limits { read: 16384, send: 16384, records: 131_072 };
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_shell::Clock::new().now().wall, limits };
    let mut stream = TlsReader {
        socket,
        tls: client::Client::new(config, skein_tls::Name::new(host).ok_or_else(invalid)?, &limits),
        env,
        above: skein_lib::Queue::with_capacity(16),
        below: skein_lib::Queue::with_capacity(16),
        intake: skein_lib::Intake::with_capacity(131_072),
        wire: wire.into(),
        sent: 0,
        pending: 0,
        bytes: vec![],
        offset: 0,
    };
    client::down(&mut stream.tls, &stream.env, Request::Handshake, &mut stream.above, &mut stream.below);
    read_message(&mut io::BufReader::new(stream), Some(method))?.ok_or_else(invalid)
}

/// Transport adapters for the managed sitting. The existing relay and its guard are unchanged.
pub mod managed_transport {
    use super::{Message, Result, Socket, Upstream, invalid, read_message, tls_exchange};
    use skein_lib::Duration;
    use std::{
        io::{self, Write},
        net::SocketAddr,
    };

    pub struct Request(Message);
    impl Request {
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            read_message(&mut io::Cursor::new(bytes), None)?.map(Self).ok_or_else(invalid)
        }
        pub fn read(socket: &mut Socket<'_>) -> Result<Self> {
            read_message(&mut io::BufReader::new(socket), None)?.map(Self).ok_or_else(invalid)
        }
        pub fn route(&self) -> Result<(&str, &str)> {
            self.0.request()
        }
        pub fn header(&self, name: &str) -> Result<Option<&str>> {
            self.0.header(name)
        }
        pub fn body(&self) -> Result<Vec<u8>> {
            self.0.body()
        }
        pub fn upstream_bytes(&self, git: bool) -> Result<usize> {
            Ok(self.0.with_host(if git { "github.com" } else { "api.github.com" })?.len())
        }
    }
    pub struct Answer(Message);
    impl Answer {
        pub fn status(&self) -> Result<u16> {
            self.0.status()
        }
        pub fn body(&self) -> Result<Vec<u8>> {
            self.0.body()
        }
        #[must_use]
        pub fn refuse() -> Self {
            Self(Message::answer(403, b"managed relay refused request"))
        }
        pub fn send(&self, socket: &mut Socket<'_>, method: &str) -> Result<()> {
            let mut wire = self.0.framed(method)?;
            // One request per connection: closing is explicit even for git's persistent client.
            let at = wire.windows(4).position(|s| s == b"\r\n\r\n").ok_or_else(invalid)?;
            wire.splice(at + 2..at + 2, b"Connection: close\r\n".iter().copied());
            socket.write_all(&wire)
        }
    }
    pub struct Transport(Upstream);
    impl Transport {
        pub fn new(fake: Option<SocketAddr>) -> Result<Self> {
            if let Some(address) = fake {
                if !address.ip().is_loopback() {
                    return Err(invalid());
                }
                return Ok(Self(Upstream::Plain(address)));
            }
            let roots = skein_shell::read_trust_roots(
                &skein_shell::Trust::Machine { ssl_cert_file: None },
                &skein_shell::RootBounds { bundle_bytes: 4_194_304, certificates: 1024, certificate_bytes: 65536 },
            )
            .map_err(|_| invalid())?;
            let (tls, _) = skein_tls::Config::from_der(&roots.certificates, &[b"http/1.1"]).map_err(|_| invalid())?;
            Ok(Self(Upstream::Tls(tls)))
        }
        pub fn exchange(&self, request: &Request, git: bool) -> Result<Answer> {
            let host = if git { "github.com" } else { "api.github.com" };
            let wire = request.0.with_host(host)?;
            let (method, _) = request.route()?;
            let answer = match &self.0 {
                Upstream::Tls(tls) => tls_exchange(tls, host, &wire, method)?,
                Upstream::Plain(address) => {
                    let mut socket = Socket::connect(
                        *address,
                        "the managed relay upstream's connection",
                        Duration::from_secs(30),
                        ["the managed relay upstream's reply", "the managed relay upstream's request"],
                        true,
                        None,
                    )?;
                    socket.write_all(&wire)?;
                    read_message(&mut io::BufReader::new(socket), Some(method))?.ok_or_else(invalid)?
                }
            };
            Ok(Answer(answer))
        }
    }
}
