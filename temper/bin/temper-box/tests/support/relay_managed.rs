//! Sitting-only policy and redaction (testing/live-tier.md; protocol/setup/github.md, 2).
use super::relay::{
    Guard,
    managed_transport::{Answer, Request, Transport},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    net::{SocketAddr, TcpListener},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

/// The sitting's repository: fixed names, never inputs, so no run can point the guard at another
/// account. The fake run is the live sitting's dry run, and shares them.
pub const OWNER: &str = "fabrig-ai";
pub const NAME: &str = "temper-probe-2";
fn invalid() -> io::Error {
    io::Error::other("managed relay invalid request or response")
}
/// Diagnostic data is fixed vocabulary plus numeric counts, never upstream text.
#[derive(Debug)]
struct Refusal {
    reason: &'static str,
    rows: Option<usize>,
    total_count: Option<u64>,
}
impl Refusal {
    fn label(reason: &'static str) -> Self {
        Self { reason, rows: None, total_count: None }
    }
}
#[derive(Default)]
struct State {
    converted: bool,
    app: Option<u64>,
    slug: String,
    installations: BTreeSet<u64>,
    tokens: Vec<String>,
    checked: bool,
    repository: Option<u64>,
    pulls: BTreeSet<u64>,
    commits: BTreeSet<u64>,
    count: usize,
    delivery_polls: usize,
    secrets: Vec<(&'static str, Vec<u8>)>,
}

fn keep_secret(secrets: &mut Vec<(&'static str, Vec<u8>)>, field: &'static str, bytes: &[u8]) {
    if !bytes.is_empty() && !secrets.iter().any(|(name, known)| *name == field && known == bytes) {
        secrets.push((field, bytes.to_vec()));
    }
}

fn secret_hit<'a>(bytes: &[u8], secrets: &'a [(&'static str, Vec<u8>)]) -> Option<&'a str> {
    secrets.iter().find_map(|(field, secret)| {
        (!secret.is_empty() && bytes.windows(secret.len()).any(|part| part == secret)).then_some(*field)
    })
}

fn keep_text(secrets: &mut Vec<(&'static str, Vec<u8>)>, field: &'static str, text: &str) {
    keep_secret(secrets, field, text.as_bytes());
    // JSON journals escape multiline PEMs. Check their serialized bytes as well.
    let encoded = serde_json::to_vec(text).expect("JSON string");
    keep_secret(secrets, field, &encoded[1..encoded.len() - 1]);
}
struct Shared {
    fake: bool,
    transport: Transport,
    guard: Guard,
    /// The login that makes the App: Connect makes it under the person's own account
    /// (protocol/setup/github.md, 2), so a conversion's owner is a User, never the guard's owner.
    person: String,
    state: Mutex<State>,
    journal: Mutex<File>,
    evidence: Mutex<File>,
}
impl State {
    fn charge_request(&mut self, method: &str, path: &str, auth: &str, git: bool) -> bool {
        // Delivery polling continues while the person works. Only these JWT reads
        // have their own budget; git and every other route share the original cap.
        let delivery_poll = !git
            && method == "GET"
            && self.jwt(auth)
            && (path == "/app/hook/deliveries"
                || path.strip_prefix("/app/hook/deliveries/").and_then(positive).is_some());
        let (count, cap) = if delivery_poll { (&mut self.delivery_polls, 5000) } else { (&mut self.count, 400) };
        if *count >= cap {
            return false;
        }
        *count += 1;
        true
    }
    fn remember_auth(&mut self, auth: &str) -> io::Result<()> {
        if let Some(token) = auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("token ")) {
            let field = if token.split('.').count() == 3 { "JWT" } else { "installation_token" };
            keep_text(&mut self.secrets, field, token);
        } else if let Some(encoded) = auth.strip_prefix("Basic ") {
            let decoded = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, encoded.as_bytes(), 8192)
                .map_err(|_| invalid())?;
            if let Some(token) = decoded.as_ref().strip_prefix(b"x-access-token:") {
                keep_secret(&mut self.secrets, "installation_token", token);
            }
        }
        Ok(())
    }
    fn jwt(&self, authorization: &str) -> bool {
        let Some(encoded) = authorization.strip_prefix("Bearer ").and_then(|s| s.split('.').nth(1)) else {
            return false;
        };
        let Ok(bytes) = skein_lib::base64::decode(skein_lib::base64::Alphabet::Url, encoded.as_bytes(), 2048) else {
            return false;
        };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else { return false };
        self.app.is_some_and(|id| {
            value["iss"] == id || value["iss"].as_str().and_then(|s| s.parse::<u64>().ok()) == Some(id)
        })
    }
    fn token(&self, authorization: &str, git: bool) -> bool {
        if git {
            let Some(encoded) = authorization.strip_prefix("Basic ") else { return false };
            let Ok(bytes) = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, encoded.as_bytes(), 8192)
            else {
                return false;
            };
            return self.tokens.iter().any(|token| bytes.as_ref() == format!("x-access-token:{token}").as_bytes());
        }
        authorization
            .strip_prefix("Bearer ")
            .or_else(|| authorization.strip_prefix("token "))
            .is_some_and(|token| self.tokens.iter().any(|known| known == token))
    }
    fn learn_repositories(&mut self, value: &Value, guard: &Guard) -> Result<(), Refusal> {
        self.checked = false;
        let rows =
            value["repositories"].as_array().ok_or_else(|| Refusal::label("learn:installation-repositories-array"))?;
        if value["total_count"] != 1
            || rows.len() != 1
            || rows[0]["owner"]["login"] != guard.owner
            || rows[0]["name"] != guard.repository
            || rows[0].get("full_name").is_some_and(|name| name != &format!("{}/{}", guard.owner, guard.repository))
        {
            return Err(Refusal {
                reason: "learn:installation-repositories-not-exactly-probe-2",
                rows: Some(rows.len()),
                total_count: value["total_count"].as_u64(),
            });
        }
        self.repository =
            Some(rows[0]["id"].as_u64().ok_or_else(|| Refusal::label("learn:installation-repository-id"))?);
        self.checked = true;
        Ok(())
    }
}
fn positive(text: &str) -> Option<u64> {
    text.parse().ok().filter(|n| *n > 0)
}
fn installation(path: &str) -> Option<u64> {
    positive(path.strip_prefix("/app/installations/")?)
}
impl Shared {
    fn repo(&self) -> String {
        format!("/repos/{}", self.full_name())
    }
    fn full_name(&self) -> String {
        format!("{}/{}", self.guard.owner, self.guard.repository)
    }
    fn git_allowed(&self, state: &State, method: &str, target: &str, body: &[u8], auth: &str) -> bool {
        let path = target.split('?').next().unwrap_or("");
        if !state.checked || !state.token(auth, true) {
            return false;
        }
        if self.fake && method == "POST" && path.starts_with("/.fake/git/") {
            let value: Value = match serde_json::from_slice(body) {
                Ok(value) => value,
                Err(_) => return false,
            };
            let known = |name: &str| value[name].as_u64().is_some_and(|id| state.commits.contains(&id));
            return match path {
                "/.fake/git/clone" => value == json!({"repository":self.full_name()}),
                "/.fake/git/objects" => value["commits"].as_array().is_some_and(|ids| {
                    !ids.is_empty() && ids.iter().all(|id| id.as_u64().is_some_and(|id| state.commits.contains(&id)))
                }),
                "/.fake/git/commit" => known("parent") && value["tree"].is_array() && value["message"].is_string(),
                "/.fake/git/create" | "/.fake/git/push" => {
                    value["repository"] == self.full_name()
                        && value["branch"] == format!("{}managed", self.guard.prefix)
                        && known("commit")
                }
                _ => false,
            };
        }
        self.guard.allows("git", method, target, Some(body), None)
    }
    fn allows(&self, request: &Request, git: bool) -> io::Result<bool> {
        let (method, target) = request.route()?;
        let path = target.split('?').next().ok_or_else(invalid)?;
        // Reject encoded separators, control bytes, dot segments and ambiguous paths before any dispatch.
        if path.replace("%5Bbot%5D", "[bot]").contains(['%', '\\', '#'])
            || path.contains("//")
            || path.split('/').any(|s| matches!(s, "." | ".."))
            || !path.starts_with('/')
            || target.bytes().any(|b| b <= 32 || b == 127)
            || request.header("content-encoding")?.is_some()
        {
            return Ok(false);
        }
        let body = request.body()?;
        let auth = request.header("authorization")?.unwrap_or("");
        let mut state = self.state.lock().map_err(|_| invalid())?;
        state.remember_auth(auth)?;
        if !state.charge_request(method, path, auth, git) {
            return Ok(false);
        }
        if git {
            return Ok(self.git_allowed(&state, method, target, &body, auth));
        }
        if method == "POST" && path.starts_with("/app-manifests/") && path.ends_with("/conversions") {
            let code = &path[15..path.len() - 12];
            if state.converted || code.is_empty() || !code.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Ok(false);
            }
            state.converted = true; // Includes failed attempts: the relay admits conversion exactly once.
            return Ok(true);
        }
        if method == "GET"
            && auth.is_empty()
            && (path == format!("/users/{}[bot]", state.slug) || path == format!("/users/{}%5Bbot%5D", state.slug))
        {
            return Ok(state.app.is_some());
        }
        if state.jwt(auth) {
            if method == "GET" {
                return Ok(matches!(path, "/app" | "/app/installations" | "/app/hook/config" | "/app/hook/deliveries")
                    || installation(path).is_some()
                    || path.strip_prefix("/app/hook/deliveries/").and_then(positive).is_some()
                    || path == format!("/users/{}[bot]", state.slug)
                    || path == format!("/users/{}%5Bbot%5D", state.slug));
            }
            if method == "POST"
                && let Some(id) = path
                    .strip_prefix("/app/installations/")
                    .and_then(|s| s.strip_suffix("/access_tokens"))
                    .and_then(positive)
            {
                return Ok(state.installations.contains(&id)
                    && (body.is_empty()
                        || serde_json::from_slice::<Value>(&body)
                            .is_ok_and(|v| v == json!({}) || v == json!({"permissions":{"contents":"write"}}))));
            }
            return Ok(false);
        }
        if !state.token(auth, false) {
            return Ok(false);
        }
        if method == "GET" && path == "/installation/repositories" {
            return Ok(true);
        }
        if method == "DELETE" && path == "/installation/token" {
            return Ok(true);
        }
        if method == "POST"
            && path == "/graphql"
            && serde_json::from_slice::<Value>(&body).is_ok_and(|v| {
                v["query"].as_str().is_some_and(|q| q.as_bytes() == skein_forge::github::document::VIEWER)
            })
        {
            return Ok(true);
        }
        if !state.checked {
            return Ok(false);
        }
        if method == "GET"
            && path.strip_prefix("/repositories/").and_then(positive).is_some_and(|id| state.repository == Some(id))
        {
            return Ok(true);
        }
        let repo = self.repo();
        if method == "GET" && (path == repo || path.starts_with(&format!("{repo}/"))) {
            return Ok(true);
        }
        let value = serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null);
        if method == "POST" && path == format!("{repo}/pulls") {
            return Ok(value["head"] == format!("{}managed", self.guard.prefix) && value["base"] == "main");
        }
        if method == "PATCH"
            && let Some(number) = path
                .strip_prefix(&format!("{repo}/pulls/"))
                .or_else(|| path.strip_prefix(&format!("{repo}/issues/")))
                .and_then(positive)
        {
            return Ok(state.pulls.contains(&number) && value == json!({"state":"closed"}));
        }
        Ok(method == "DELETE" && path == format!("{repo}/git/refs/heads/{}managed", self.guard.prefix))
    }
    fn learn(&self, request: &Request, answer: &Answer) -> Result<(), Refusal> {
        let (method, target) = request.route().map_err(|_| Refusal::label("learn:request-route"))?;
        let path = target.split('?').next().ok_or_else(|| Refusal::label("learn:request-path"))?;
        if !matches!(answer.status().map_err(|_| Refusal::label("learn:response-status"))?, 200 | 201) {
            return Ok(());
        }
        let value: Value =
            match serde_json::from_slice(&answer.body().map_err(|_| Refusal::label("learn:response-body"))?) {
                Ok(value) => value,
                Err(_) => return Ok(()),
            };
        let mut state = self.state.lock().map_err(|_| Refusal::label("learn:state-lock"))?;
        if self.fake
            && method == "POST"
            && path == "/.fake/git/clone"
            && let Some(rows) = value["branches"].as_array()
        {
            for row in rows {
                if let Some(id) = row["commit"].as_u64() {
                    state.commits.insert(id);
                }
            }
        }
        if self.fake && method == "POST" && path == "/.fake/git/commit" {
            state.commits.insert(value["commit"].as_u64().ok_or_else(|| Refusal::label("learn:git-commit-id"))?);
        }
        if method == "POST" && path.starts_with("/app-manifests/") {
            for field in ["pem", "webhook_secret", "client_secret", "client_id"] {
                if let Some(text) = value[field].as_str() {
                    keep_text(&mut state.secrets, field, text);
                    if field == "pem" {
                        let encoded: String = text.lines().filter(|line| !line.starts_with("-----")).collect();
                        let der =
                            skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, encoded.as_bytes(), 65536)
                                .map_err(|_| Refusal::label("learn:conversion-pem"))?;
                        keep_secret(&mut state.secrets, "App_key_DER", der.as_ref());
                        keep_text(&mut state.secrets, "App_key_base64", &encoded);
                    }
                }
            }
            if value["owner"]["login"] != self.person {
                return Err(Refusal::label("learn:conversion-owner"));
            }
            state.app = Some(value["id"].as_u64().ok_or_else(|| Refusal::label("learn:conversion-app-id"))?);
            state.slug = value["slug"].as_str().ok_or_else(|| Refusal::label("learn:conversion-slug"))?.into();
            let sanitized = redact(&value, "");
            let mut file = self.evidence.lock().map_err(|_| Refusal::label("learn:evidence-lock"))?;
            serde_json::to_writer_pretty(&mut *file, &sanitized).map_err(|_| Refusal::label("learn:evidence-json"))?;
            file.write_all(b"\n").map_err(|_| Refusal::label("learn:evidence-write"))?;
            file.sync_all().map_err(|_| Refusal::label("learn:evidence-sync"))?;
        }
        if method == "GET"
            && installation(path).is_some()
            && value["account"]["login"] == self.guard.owner
            && (value.get("app_id").is_none() || value["app_id"].as_u64() == state.app)
        {
            state.installations.insert(value["id"].as_u64().ok_or_else(|| Refusal::label("learn:installation-id"))?);
        }
        if method == "POST" && path.ends_with("/access_tokens") {
            let token = value["token"].as_str().ok_or_else(|| Refusal::label("learn:installation-token"))?;
            keep_text(&mut state.secrets, "installation_token", token);
            state.tokens.push(token.into());
        }
        if method == "GET" && path == "/installation/repositories" {
            state.learn_repositories(&value, &self.guard)?;
        }
        let repo = self.repo();
        if method == "GET"
            && path == format!("{repo}/pulls")
            && let Some(rows) = value.as_array()
        {
            for row in rows {
                if row["head"]["ref"] == format!("{}managed", self.guard.prefix)
                    && (row["head"]["repo"]["full_name"] == self.full_name()
                        || self.fake && row["head"].get("repo").is_none())
                {
                    state
                        .pulls
                        .insert(row["number"].as_u64().ok_or_else(|| Refusal::label("learn:listed-pull-number"))?);
                }
            }
        }
        if method == "POST" && path == format!("{repo}/pulls") {
            state.pulls.insert(value["number"].as_u64().ok_or_else(|| Refusal::label("learn:created-pull-number"))?);
        }
        Ok(())
    }
    fn serve(&self, socket: std::net::TcpStream, git: bool) -> io::Result<()> {
        let mut socket = super::socket::Socket::deferred(
            socket,
            skein_lib::Duration::from_secs(30),
            ["the managed relay client's request", "the managed relay client's reply"],
        );
        let request = Request::read(&mut socket)?;
        let (method, target) = request.route()?;
        let accepted = self.allows(&request, git)?;
        let mut upstream_status = None;
        let mut refusal = None;
        let answer = if accepted {
            let answer = self.transport.exchange(&request, git)?;
            let status = answer.status()?;
            upstream_status = Some(status);
            // A redirected client could otherwise leave this private guard entirely.
            if (300..400).contains(&status) && status != 304 {
                refusal = Some(Refusal::label("redirect"));
                Answer::refuse()
            } else if let Err(reason) = self.learn(&request, &answer) {
                refusal = Some(reason);
                Answer::refuse()
            } else {
                answer
            }
        } else {
            refusal = Some(Refusal::label("guard"));
            Answer::refuse()
        };
        let path = if target.starts_with("/app-manifests/") {
            "/app-manifests/[code]/conversions"
        } else {
            target.split('?').next().ok_or_else(invalid)?
        };
        let mut journal = self.journal.lock().map_err(|_| invalid())?;
        serde_json::to_writer(
            &mut *journal,
            &json!({"listener":if git {"git"} else {"api"}, "method":method, "path":path, "status":answer.status()?, "refused":!accepted || answer.status()? == 403,
                "upstream_status":upstream_status, "reason":refusal.as_ref().map(|r| r.reason),
                "rows":refusal.as_ref().and_then(|r| r.rows), "total_count":refusal.as_ref().and_then(|r| r.total_count),
                "request_body_bytes":request.body()?.len(), "response_body_bytes":answer.body()?.len(),
                "upstream_request_bytes":if accepted { request.upstream_bytes(git)? } else { 0 }}),
        )?;
        journal.write_all(b"\n")?;
        journal.flush()?;
        answer.send(&mut socket, method)
    }

    fn scan(&self, extra: &Value) -> io::Result<()> {
        let mut state = self.state.lock().map_err(|_| invalid())?;
        for field in ["App_key_DER", "App_key_base64", "JWT", "installation_token"] {
            if let Some(values) = extra[field].as_array() {
                for value in values {
                    let text = value.as_str().ok_or_else(invalid)?;
                    if field == "App_key_DER" {
                        let bytes =
                            skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, text.as_bytes(), 65536)
                                .map_err(|_| invalid())?;
                        keep_secret(&mut state.secrets, field, bytes.as_ref());
                    } else {
                        keep_text(&mut state.secrets, field, text);
                    }
                }
            }
        }
        for file in [&self.journal, &self.evidence] {
            let mut file = file.lock().map_err(|_| invalid())?;
            file.flush()?;
            file.rewind()?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            if let Some(field) = secret_hit(&bytes, &state.secrets) {
                eprintln!("secret bytes found: {field}");
                return Err(invalid());
            }
        }
        Ok(())
    }
}
/// Each node keeps its JSON type. Only the owner's explicit allowlist keeps values.
fn redact(value: &Value, path: &str) -> Value {
    let allowed =
        matches!(path, "id" | "slug" | "name" | "html_url" | "owner.login" | "owner.type" | "permissions" | "events")
            || path.starts_with("permissions.")
            || path.starts_with("events.");
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(name, value)| {
                    let next = if path.is_empty() { name.clone() } else { format!("{path}.{name}") };
                    (name.clone(), redact(value, &next))
                })
                .collect(),
        ),
        Value::Array(rows) => Value::Array(rows.iter().map(|v| redact(v, &format!("{path}.[]"))).collect()),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            let kind = match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) | Value::Object(_) => unreachable!(),
            };
            if allowed { json!({"type":kind,"value":value}) } else { json!({"type":kind}) }
        }
    }
}
#[expect(clippy::disallowed_methods, reason = "the sitting's relay serves its listeners on owned threads")]
pub fn run() -> io::Result<()> {
    let directory = PathBuf::from(temper_test_env::value(&temper_test_env::MANAGED_SCRATCH).ok_or_else(invalid)?);
    let prefix = temper_test_env::value(&temper_test_env::MANAGED_PREFIX)
        .ok_or_else(invalid)?
        .into_string()
        .map_err(|_| invalid())?;
    let mode = temper_test_env::value(&temper_test_env::MANAGED_MODE)
        .ok_or_else(invalid)?
        .into_string()
        .map_err(|_| invalid())?;
    let person = temper_test_env::value(&temper_test_env::MANAGED_PERSON)
        .ok_or_else(invalid)?
        .into_string()
        .map_err(|_| invalid())?;
    if person.is_empty() {
        return Err(invalid());
    }
    let fake = match mode.as_str() {
        "fake" => Some(
            temper_test_env::value(&temper_test_env::MANAGED_UPSTREAM)
                .ok_or_else(invalid)?
                .into_string()
                .map_err(|_| invalid())?
                .parse::<SocketAddr>()
                .map_err(|_| invalid())?,
        ),
        "live" => None,
        _ => return Err(invalid()),
    };
    let evidence = PathBuf::from(temper_test_env::value(&temper_test_env::MANAGED_EVIDENCE).ok_or_else(invalid)?);
    let shared = Arc::new(Shared {
        fake: fake.is_some(),
        transport: Transport::new(fake)?,
        guard: Guard { owner: OWNER.into(), repository: NAME.into(), installation: 0, prefix },
        person,
        state: Mutex::new(State::default()),
        journal: Mutex::new(
            OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .mode(0o600)
                .open(directory.join("relay.jsonl"))?,
        ),
        evidence: Mutex::new(OpenOptions::new().create_new(true).read(true).write(true).mode(0o600).open(evidence)?),
    });
    let api = TcpListener::bind("127.0.0.1:0")?;
    let git = TcpListener::bind("127.0.0.1:0")?;
    let addresses = json!({"api":api.local_addr()?.to_string(), "git":git.local_addr()?.to_string()});
    super::write_private(&directory.join("relay-ready.json"), &serde_json::to_vec(&addresses)?);
    let stopping = Arc::new(AtomicBool::new(false));
    let mut listeners = Vec::new();
    for (listener, git) in [(api, false), (git, true)] {
        listener.set_nonblocking(true)?;
        let shared = shared.clone();
        let stopping = stopping.clone();
        listeners.push(thread::spawn(move || {
            let mut workers = Vec::new();
            while !stopping.load(Ordering::Acquire) {
                let socket = match listener.accept() {
                    Ok((socket, _)) => socket,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    Err(_) => break,
                };
                let shared = shared.clone();
                workers.push(thread::spawn(move || {
                    if shared.serve(socket, git).is_err() {
                        // Neither errors nor request dumps can expose keys, JWTs, codes or bodies.
                        if let Ok(mut journal) = shared.journal.lock() {
                            let _ = journal.write_all(b"{\"error\":\"transport or framing failed\",\"upstream_status\":null,\"reason\":null}\n");
                        }
                    }
                }));
            }
            for worker in workers {
                worker.join().map_err(|_| invalid())?;
            }
            Ok::<(), io::Error>(())
        }));
    }
    // The owner sends additional in-memory credentials through this private pipe.
    // Stop admission and drain every worker before reading either artifact.
    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    stopping.store(true, Ordering::Release);
    for listener in listeners {
        listener.join().map_err(|_| invalid())??;
    }
    let extra: Value = serde_json::from_str(&command).map_err(|_| invalid())?;
    shared.scan(&extra)?;
    println!("no secret bytes in the relay log or the record");
    Ok(())
}

#[cfg(test)]
const REPO: &str = "/repos/acme/temper-probe-2";

#[cfg(test)]
fn policy_shared() -> Shared {
    Shared {
        fake: true,
        transport: Transport::new(Some("127.0.0.1:9".parse().expect("loopback"))).expect("plain transport"),
        guard: Guard {
            owner: "acme".into(),
            repository: "temper-probe-2".into(),
            installation: 0,
            prefix: "temper/live-test/".into(),
        },
        person: "alice".into(),
        state: Mutex::new(State::default()),
        journal: Mutex::new(OpenOptions::new().write(true).open("/dev/null").expect("discard test journal")),
        evidence: Mutex::new(OpenOptions::new().write(true).open("/dev/null").expect("discard evidence")),
    }
}

#[test]
fn a_conversion_is_admitted_from_the_person_alone() {
    let request = Request::from_bytes(b"POST /app-manifests/code/conversions HTTP/1.1\r\nHost: local\r\n\r\n")
        .expect("conversion request");
    for (login, reason, app) in [
        ("acme", Some("learn:conversion-owner"), None),
        ("other", Some("learn:conversion-owner"), None),
        ("alice", None, Some(42)),
    ] {
        let mut shared = policy_shared();
        let path =
            skein_scratch::paths::target_tmp().join(format!("managed-conversion-{}-{login}.json", std::process::id()));
        std::fs::create_dir_all(path.parent().expect("test directory")).expect("test directory");
        shared.evidence = Mutex::new(
            OpenOptions::new().create_new(true).write(true).mode(0o600).open(&path).expect("conversion evidence"),
        );
        let body = json!({"id":42,"slug":"temper","owner":{"login":login,"type":"User"}}).to_string();
        let wire = format!("HTTP/1.1 201 Created\r\nContent-Length: {}\r\n\r\n{body}", body.len());
        let answer = Answer::from_bytes(wire.as_bytes()).expect("conversion answer");
        assert_eq!(shared.learn(&request, &answer).err().map(|refusal| refusal.reason), reason, "{login}");
        assert_eq!(shared.state.lock().expect("state").app, app, "{login}");
        drop(shared);
        std::fs::remove_file(path).expect("remove conversion evidence");
    }
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the loopback relay and upstream serve the synchronous test client on scoped threads"
)]
fn journal_distinguishes_learn_redirect_guard_and_upstream_refusals() {
    use std::net::TcpStream;
    let upstream = TcpListener::bind("127.0.0.1:0").expect("loopback upstream");
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback relay");
    let path = skein_scratch::paths::target_tmp().join(format!("managed-journal-{}.jsonl", std::process::id()));
    std::fs::create_dir_all(path.parent().expect("test directory")).expect("test directory");
    let shared = Shared {
        fake: true,
        transport: Transport::new(Some(upstream.local_addr().expect("upstream address"))).expect("transport"),
        guard: Guard {
            owner: "acme".into(),
            repository: "temper-probe-2".into(),
            installation: 0,
            prefix: "temper/live-test/".into(),
        },
        person: "alice".into(),
        state: Mutex::new(State { tokens: vec!["private-token".into()], ..State::default() }),
        journal: Mutex::new(
            OpenOptions::new().create_new(true).read(true).write(true).mode(0o600).open(&path).expect("journal"),
        ),
        evidence: Mutex::new(OpenOptions::new().write(true).open("/dev/null").expect("discard evidence")),
    };
    let wide = json!({"total_count":2,"repositories":[
        {"owner":{"login":"private-owner"},"name":"private-name"},
        {"owner":{"login":"another-private-owner"},"name":"another-private-name"}]})
    .to_string();
    for (status, body, reason, count) in [
        (200, wide.as_str(), Some("learn:installation-repositories-not-exactly-probe-2"), Some(2)),
        (302, "private-response", Some("redirect"), None),
        (403, "private-response", None, None),
        (0, "", Some("guard"), None),
    ] {
        thread::scope(|scope| {
            if status != 0 {
                let upstream = &upstream;
                scope.spawn(move || {
                    let (socket, _) = upstream.accept().expect("upstream accepts");
                    let mut socket = super::socket::Socket::deferred(
                        socket,
                        skein_lib::Duration::from_secs(30),
                        ["the managed relay stub's request", "the managed relay stub's reply"],
                    );
                    Request::read(&mut socket).expect("forwarded request");
                    write!(socket, "HTTP/1.1 {status} Answer\r\nContent-Length: {}\r\n\r\n{body}", body.len())
                        .expect("upstream response");
                });
            }
            scope.spawn(|| {
                let (socket, _) = listener.accept().expect("relay accepts");
                shared.serve(socket, false).expect("serve");
            });
            let client = TcpStream::connect(listener.local_addr().expect("relay address")).expect("client");
            let mut client = super::socket::Socket::reading(
                client,
                skein_lib::Duration::from_secs(2),
                "the offline managed relay client's reply",
            );
            let token = if status == 0 { "unknown-token" } else { "private-token" };
            write!(client, "GET /installation/repositories?private-query HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer {token}\r\n\r\n").expect("request");
            let mut response = Vec::new();
            client.read_to_end(&mut response).expect("response");
            assert!(response.starts_with(b"HTTP/1.1 403"));
        });
        let journal = std::fs::read_to_string(&path).expect("journal");
        let row: Value = serde_json::from_str(journal.lines().last().expect("row")).expect("JSON row");
        assert_eq!(row["reason"], json!(reason));
        assert_eq!(row["upstream_status"], if status == 0 { Value::Null } else { json!(status) });
        assert_eq!(row["rows"], json!(count));
        assert_eq!(row["total_count"], json!(count));
        assert_eq!(row["path"], "/installation/repositories");
        assert_eq!(row["status"], 403);
        // All fields beyond the existing route/vocabulary are numbers, booleans or null.
        for (key, value) in row.as_object().expect("object") {
            if !matches!(key.as_str(), "listener" | "method" | "path" | "reason") {
                assert!(value.is_number() || value.is_boolean() || value.is_null());
            }
        }
        for private in
            ["private-owner", "private-name", "private-response", "private-token", "unknown-token", "private-query"]
        {
            assert!(!journal.contains(private));
        }
    }
    drop(shared);
    std::fs::remove_file(path).expect("remove journal");
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the loopback relay serves the synchronous test client on a scoped thread"
)]
fn delivery_polls_have_their_own_cap_and_leave_the_git_budget_available() {
    use std::net::TcpStream;
    let path = skein_scratch::paths::target_tmp().join(format!("managed-polls-{}.jsonl", std::process::id()));
    std::fs::create_dir_all(path.parent().expect("test directory")).expect("test directory");
    let shared = Shared {
        fake: true,
        transport: Transport::new(Some("127.0.0.1:9".parse().expect("loopback"))).expect("plain transport"),
        guard: Guard {
            owner: "acme".into(),
            repository: "temper-probe-2".into(),
            installation: 0,
            prefix: "temper/live-test/".into(),
        },
        person: "alice".into(),
        state: Mutex::new(State {
            app: Some(42),
            checked: true,
            tokens: vec!["fake-minted".into()],
            ..State::default()
        }),
        journal: Mutex::new(
            OpenOptions::new().create_new(true).read(true).write(true).mode(0o600).open(&path).expect("journal"),
        ),
        evidence: Mutex::new(OpenOptions::new().write(true).open("/dev/null").expect("discard evidence")),
    };
    // iss=42, recognized exactly as the existing guard recognizes the App's JWT.
    let poll = |target: &str| {
        format!("GET {target} HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer e30.eyJpc3MiOjQyfQ.sig\r\n\r\n")
    };
    let list = Request::from_bytes(poll("/app/hook/deliveries?per_page=100").as_bytes()).expect("poll framing");
    let detail = Request::from_bytes(poll("/app/hook/deliveries/1").as_bytes()).expect("detail framing");
    for _ in 0..500 {
        assert!(shared.allows(&list, false).expect("list poll"));
        assert!(shared.allows(&detail, false).expect("detail poll"));
    }
    let git = Request::from_bytes(b"POST /.fake/git/clone HTTP/1.1\r\nHost: local\r\nAuthorization: Basic eC1hY2Nlc3MtdG9rZW46ZmFrZS1taW50ZWQ=\r\nContent-Length: 36\r\n\r\n{\"repository\":\"acme/temper-probe-2\"}")
        .expect("git framing");
    assert!(shared.allows(&git, true).expect("git after 1000 polls"));
    assert_eq!(shared.state.lock().expect("state").count, 1);
    let other = Request::from_bytes(
        b"GET /installation/repositories HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer fake-minted\r\n\r\n",
    )
    .expect("ordinary framing");
    for _ in 1..400 {
        assert!(shared.allows(&other, false).expect("ordinary request"));
    }
    assert!(!shared.allows(&git, true).expect("git cap stays 400"));
    // Exhausting the ordinary budget does not stop delivery polling either.
    for _ in 1000..5000 {
        assert!(shared.allows(&list, false).expect("independent delivery budget"));
    }
    assert_eq!(shared.state.lock().expect("state").delivery_polls, 5000);
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback relay");
    thread::scope(|scope| {
        scope.spawn(|| {
            let (socket, _) = listener.accept().expect("relay accepts");
            shared.serve(socket, false).expect("serve refused 5001st poll");
        });
        let client = TcpStream::connect(listener.local_addr().expect("relay address")).expect("client");
        let mut client = super::socket::Socket::reading(
            client,
            skein_lib::Duration::from_secs(2),
            "the offline managed relay client's reply",
        );
        client.write_all(poll("/app/hook/deliveries/1").as_bytes()).expect("request");
        let mut response = Vec::new();
        client.read_to_end(&mut response).expect("response");
        assert!(response.starts_with(b"HTTP/1.1 403"));
    });
    let journal = std::fs::read_to_string(&path).expect("journal");
    let row: Value = serde_json::from_str(journal.trim()).expect("one journal row");
    assert_eq!(row["reason"], "guard");
    assert_eq!(row["upstream_status"], Value::Null);
    assert_eq!(row["path"], "/app/hook/deliveries/1");
    drop(shared);
    std::fs::remove_file(path).expect("remove journal");
}

#[test]
fn secret_scan_finds_a_planted_journal_secret() {
    let mut secrets = Vec::new();
    keep_text(&mut secrets, "installation_token", "planted-private-token");
    assert_eq!(secret_hit(b"{\"path\":\"/planted-private-token\"}\n", &secrets), Some("installation_token"));
    assert_eq!(secret_hit(b"{\"path\":\"/app\",\"status\":200}\n", &secrets), None);
    keep_text(&mut secrets, "pem", "PRIVATE\nPEM\n");
    assert_eq!(secret_hit(br#"{"path":"PRIVATE\nPEM\n"}"#, &secrets), Some("pem"));
}

#[test]
fn conversion_evidence_keeps_only_the_owners_allowlisted_values() {
    let value = json!({"id":42,"slug":"temper","pem":"private PEM","client_id":"oauth-id",
        "client_secret":"oauth-secret","webhook_secret":"hook-secret",
        "owner":{"login":"alice","type":"User","id":123,"nested":{"secret":false}},
        "permissions":{"contents":"write"},"events":["push"],"unknown":[{"key":"secret"},true,null,42]});
    let result = redact(&value, "");
    assert_eq!(result["id"], json!({"type":"number","value":42}));
    assert_eq!(result["pem"], json!({"type":"string"}));
    assert_eq!(result["client_id"], json!({"type":"string"}));
    assert_eq!(result["owner"]["login"]["value"], "alice");
    assert_eq!(result["owner"]["id"], json!({"type":"number"}));
    assert_eq!(result["permissions"]["contents"]["value"], "write");
    assert_eq!(result["events"][0]["value"], "push");
    assert_eq!(result["unknown"][0]["key"], json!({"type":"string"}));
    assert_eq!(result["unknown"][1], json!({"type":"boolean"}));
    assert_eq!(result["unknown"][2], json!({"type":"null"}));
    for secret in ["private PEM", "oauth-id", "oauth-secret", "hook-secret"] {
        assert!(!result.to_string().contains(secret));
    }
}

#[test]
fn guard_binds_conversion_credentials_repository_and_run_writes() {
    fn request(method: &str, path: &str, auth: &str, body: &Value) -> Request {
        let body = serde_json::to_vec(&body).expect("request JSON");
        let mut wire = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: {auth}\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        wire.extend(body);
        Request::from_bytes(&wire).expect("test framing")
    }
    let shared = policy_shared();
    let allowed = |method: &str, path: &str, auth: &str, body: Value, git: bool| {
        shared.allows(&request(method, path, auth, &body), git).expect("guard")
    };
    assert!(allowed("POST", "/app-manifests/code/conversions", "", json!({}), false));
    assert!(!allowed("POST", "/app-manifests/other/conversions", "", json!({}), false));
    assert!(!allowed("GET", REPO, "Bearer unknown", json!({}), false));
    {
        let mut state = shared.state.lock().expect("state");
        state.tokens.push("fake-minted".into());
        state.repository = Some(1);
        state.commits.insert(1);
    }
    assert!(!allowed("GET", REPO, "Bearer fake-minted", json!({}), false));
    assert!(allowed("GET", "/installation/repositories", "Bearer fake-minted", json!({}), false));
    shared.state.lock().expect("state").checked = true;
    assert!(allowed("GET", REPO, "Bearer fake-minted", json!({}), false));
    assert!(!allowed("GET", "/repos/acme/temper-probe", "Bearer fake-minted", json!({}), false));
    assert!(!allowed("GET", "/repos/acme/temper-probe-2/%2e%2e/other", "Bearer fake-minted", json!({}), false));
    assert!(!allowed("GET", "/repositories/2", "Bearer fake-minted", json!({}), false));
    assert!(allowed(
        "POST",
        &format!("{REPO}/pulls"),
        "Bearer fake-minted",
        json!({"head":"temper/live-test/managed","base":"main"}),
        false
    ));
    assert!(!allowed(
        "POST",
        &format!("{REPO}/pulls"),
        "Bearer fake-minted",
        json!({"head":"main","base":"main"}),
        false
    ));
    assert!(!allowed("PATCH", &format!("{REPO}/pulls/1"), "Bearer fake-minted", json!({"state":"closed"}), false));
    assert!(!allowed("DELETE", &format!("{REPO}/git/refs/heads/main"), "Bearer fake-minted", json!({}), false));
    let basic = "Basic eC1hY2Nlc3MtdG9rZW46ZmFrZS1taW50ZWQ=";
    assert!(allowed(
        "POST",
        "/.fake/git/create",
        basic,
        json!({"repository":"acme/temper-probe-2","branch":"temper/live-test/managed","commit":1}),
        true
    ));
    assert!(!allowed(
        "POST",
        "/.fake/git/create",
        basic,
        json!({"repository":"acme/temper-probe-2","branch":"main","commit":1}),
        true
    ));
    assert!(!allowed(
        "POST",
        "/.fake/git/create",
        basic,
        json!({"repository":"acme/temper-probe","branch":"temper/live-test/managed","commit":1}),
        true
    ));
}
