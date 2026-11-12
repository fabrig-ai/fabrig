//! Isolated copies and deletion, measured at the pin (forge/setup/forgejo.md, 2, “A copy”).
use crate::{
    Result,
    cases::Recorder,
    facts::{Fact, Outcome, add},
    fixture::Fixture,
    json,
    mask::{self, Mask},
    setup,
    write::{self, SecretKind, Secrets},
};
use skein_throwaway_forgejo::{Exchange, Forgejo, Init, Outgoing, Settings};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::Path,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
const MIGRATE: &str = "/api/v1/repos/migrate";
const SOURCE: &str = "/api/v1/repos/fixture/source";
const WORKFLOW: &str = "setup-actions-reads-forgejo-workflows-alone-when-both-are-there";

fn body(name: &str, from: &str) -> Result<String> {
    Ok(String::from_utf8(
        json::write(|out| {
            out.object_start();
            for (key, value) in
                [("clone_addr", from), ("repo_owner", "copies"), ("repo_name", name), ("service", "git")]
            {
                out.key(key.as_bytes());
                out.string(value.as_bytes());
            }
            out.key(b"private");
            out.boolean(true);
            for key in ["mirror", "issues", "labels", "milestones", "pull_requests", "releases", "wiki", "lfs"] {
                out.key(key.as_bytes());
                out.boolean(false);
            }
            out.object_end();
        })?
        .into_vec(),
    )?)
}
fn keep(r: &mut Recorder<'_>, name: &str, exchange: &Exchange) -> Result<()> {
    write::check(name, &exchange.sent, &exchange.body, r.secrets)?;
    let request = mask::request(r.mask, exchange);
    let response = mask::response(r.mask, exchange);
    write::case(r.out, name, &request, &response, r.secrets)?;
    Ok(())
}
fn send(r: &mut Recorder<'_>, name: &str, method: &str, target: &str, body: Option<&str>) -> Result<Exchange> {
    let exchange =
        r.f.instance
            .exchange_patient(
                &Outgoing { method, target, body: body.map(str::as_bytes), token: Some(&r.f.owner.token), range: None },
                skein_lib::Duration::from_secs(60),
            )
            .map_err(|e| crate::failure(&e))?;
    keep(r, name, &exchange)?;
    Ok(exchange)
}
fn read(r: &mut Recorder<'_>, name: &str, repository: &str, suffix: &str) -> Result<Exchange> {
    send(r, name, "GET", &format!("/api/v1/repos/copies/{repository}{suffix}"), None)
}
fn describe(exchange: &Exchange) -> String {
    let fields = crate::json(exchange)
        .map(|j| {
            format!(
                "message={:?}, default_branch={:?}, empty={:?}",
                j.root().get("message").str(),
                j.root().get("default_branch").str(),
                j.root().get("empty").bool()
            )
        })
        .unwrap_or_default();
    format!("HTTP {} {fields}", exchange.status)
}
fn no_branch(exchange: &Exchange) -> bool {
    exchange.status == 404
}
fn refused_and_left_empty(refusal: &Exchange, read: &Exchange) -> Result<bool> {
    Ok(refusal.status >= 400
        && crate::json(refusal)?.root().get("message").str().is_some_and(|message| !message.is_empty())
        && read.status == 200
        && crate::json(read)?.root().get("empty").bool() == Some(true))
}
fn fact(facts: &mut Vec<Fact>, name: &'static str, holds: bool, exchanges: &[&Exchange]) {
    add(facts, name, holds, &exchanges.iter().map(|x| describe(x)).collect::<Vec<_>>().join("; "));
}

/// Records only migration facts and their supporting captures on two fresh loopback instances.
/// # Errors
/// Returns a binary, provisioning, proxy, exchange or secret-scan failure.
pub fn run(binary: &Path, runner: Option<&Path>, scratch: &Path, out: &Path) -> Result<()> {
    let forgejo = Forgejo::from_path(binary).map_err(|e| format!("{e:?}"))?;
    let mut settings = Settings::small();
    settings.local_migrations = true;
    let destination = Fixture::bare(&forgejo, &scratch.join("destination"), &settings)?;
    let mut source = Fixture::bare(&forgejo, &scratch.join("source"), &Settings::small())?;
    source.instance.create_repository(&source.owner, "source", Init::Readme).map_err(|e| crate::failure(&e))?;
    source.instance.create_repository(&source.owner, "empty", Init::Empty).map_err(|e| crate::failure(&e))?;
    source.instance.create_repository(&source.owner, "private", Init::Readme).map_err(|e| crate::failure(&e))?;
    source.api("PATCH", "/api/v1/repos/fixture/private", br#"{"private":true}"#)?;
    source.branch(SOURCE, "other", "main")?;
    source.api("PATCH", SOURCE, br#"{"default_branch":"other"}"#)?;
    source.api("POST", &format!("{SOURCE}/issues"), br#"{"title":"source issue"}"#)?;
    for (directory, name) in [(".forgejo", "forgejo-only"), (".github", "github-only")] {
        source.put(
            SOURCE,
            "other",
            &format!("{directory}/workflows/{name}.yaml"),
            &format!(
                "name: {name}\non: push\njobs:\n  check:\n    runs-on: kit-host\n    steps:\n      - run: exit 0\n"
            ),
        )?;
    }
    destination.api("POST", "/api/v1/admin/users/fixture/orgs", br#"{"username":"copies","visibility":"private"}"#)?;
    let mut secrets = Secrets::from_instance(&destination.instance);
    for secret in source.instance.secrets() {
        secrets.add(SecretKind::ApiToken, &secret);
    }
    let mut mask = Mask::new(destination.instance.port());
    // All source URLs pass through this proxy, giving masking one listener port.
    let proxy = Stall::start(source.instance.port(), false)?;
    mask.listener(proxy.port);
    let from = format!("http://127.0.0.1:{}/fixture/source.git", proxy.port);
    let mut r = Recorder { f: &destination, out, secrets: &mut secrets, mask: &mut mask };
    let mut facts = Vec::new();
    let unencrypted =
        send(&mut r, "setup-migrate-unencrypted-default", "POST", MIGRATE, Some(&body("unencrypted", &from)?))?;
    fact(
        &mut facts,
        "setup-migrate-loopback-http-needs-allow-unencrypted",
        unencrypted.status >= 400
            && crate::json(&unencrypted)?
                .root()
                .get("message")
                .str()
                .is_some_and(|message| message.contains("unencrypted transfer protocols are disabled")),
        &[&unencrypted],
    );
    allow_loopback_http(&destination)?;
    allow_loopback_http(&source)?;
    ordinary(&mut r, &source, &from, &mut facts)?;
    scopes(&mut r, &from, &mut facts)?;
    interrupted(&mut r, source.instance.port(), &mut facts)?;
    disconnected(&mut r, source.instance.port(), false, &mut facts)?;
    disconnected(&mut r, source.instance.port(), true, &mut facts)?;
    r.mask.listener(proxy.port);
    workflows(&mut r, runner, &from, &mut facts)?;
    let lines = facts.iter().map(Fact::line).collect::<String>();
    let lines = r.mask.bytes(lines.as_bytes());
    write::check("facts.txt", b"", &lines, r.secrets)?;
    std::fs::write(out.join("facts.txt"), &lines)?;
    println!("migration: Forgejo {} sha256 {}; {} measured facts", forgejo.version(), forgejo.sha256(), facts.len());
    source.instance.stop().map_err(|e| crate::failure(&e))?;
    destination.instance.stop().map_err(|e| crate::failure(&e))?;
    Ok(())
}

fn allow_loopback_http(f: &Fixture) -> Result<()> {
    // Append without reading app.ini, which also holds the fixture's credentials.
    // ALLOW_LOCALNETWORKS remains the caller's setting; only transport changes here.
    f.instance.kill().map_err(|e| crate::failure(&e))?;
    std::fs::OpenOptions::new()
        .append(true)
        .open(f.instance.scratch().join("app.ini"))?
        .write_all(b"[migrations]\nALLOW_UNENCRYPTED = true\n")?;
    f.instance.restart().map_err(|e| crate::failure(&e))
}

fn ordinary(r: &mut Recorder<'_>, source: &Fixture, from: &str, facts: &mut Vec<Fact>) -> Result<()> {
    let made = send(r, "setup-migrate-created", "POST", MIGRATE, Some(&body("whole", from)?))?;
    let repository = read(r, "setup-migrate-created-read", "whole", "")?;
    let branch = read(r, "setup-migrate-created-default", "whole", "/branches/other")?;
    let main = read(r, "setup-migrate-created-main", "whole", "/branches/main")?;
    let raw = read(r, "setup-migrate-created-readme", "whole", "/raw/README.md?ref=other")?;
    let row = crate::json(&made)?;
    fact(
        facts,
        "setup-migrate-answers-201-whole-and-private",
        made.status == 201
            && repository.status == 200
            && branch.status == 200
            && main.status == 200
            && raw.status == 200
            && row.root().get("private").bool() == Some(true),
        &[&made, &branch, &main, &raw],
    );
    let source_branch = source.get(&format!("{SOURCE}/branches/other"))?;
    let copied_branch = crate::json(&branch)?;
    fact(
        facts,
        "setup-migrate-keeps-the-sources-default-branch",
        row.root().get("default_branch").is("other")
            && copied_branch.root().get("commit").get("id").str() == source_branch.root().get("commit").get("id").str(),
        &[&made, &branch],
    );
    let issues = read(r, "setup-migrate-created-issues", "whole", "/issues?state=all&page=1&limit=2")?;
    fact(
        facts,
        "setup-migrate-brings-nothing-but-git",
        issues.status == 200 && crate::json(&issues)?.root().items().is_empty(),
        &[&issues],
    );
    let taken = send(r, "setup-migrate-name-taken", "POST", MIGRATE, Some(&body("whole", from)?))?;
    fact(facts, "setup-migrate-of-a-name-taken-is-409", taken.status == 409, &[&taken]);
    let missing = send(
        r,
        "setup-migrate-missing-source",
        "POST",
        MIGRATE,
        Some(&body("missing", &from.replace("source.git", "missing.git"))?),
    )?;
    let missing_read = read(r, "setup-migrate-missing-source-read", "missing", "")?;
    fact(
        facts,
        "setup-migrate-of-a-missing-source-says-why-and-leaves-an-empty-repository",
        refused_and_left_empty(&missing, &missing_read)?,
        &[&missing, &missing_read],
    );
    let private = send(
        r,
        "setup-migrate-private-source",
        "POST",
        MIGRATE,
        Some(&body("private", &from.replace("source.git", "private.git"))?),
    )?;
    let private_read = read(r, "setup-migrate-private-source-read", "private", "")?;
    fact(
        facts,
        "setup-migrate-of-a-private-source-says-why-and-leaves-an-empty-repository",
        refused_and_left_empty(&private, &private_read)?,
        &[&private, &private_read],
    );
    let empty = send(
        r,
        "setup-migrate-empty-source",
        "POST",
        MIGRATE,
        Some(&body("empty", &from.replace("source.git", "empty.git"))?),
    )?;
    let empty_branch = read(r, "setup-migrate-empty-source-branch", "empty", "/branches/main")?;
    fact(
        facts,
        "setup-migrate-of-a-source-with-no-commit-is-201-with-no-branch",
        empty.status == 201 && no_branch(&empty_branch),
        &[&empty, &empty_branch],
    );
    // The shared throwaway's app.ini writes [server] OFFLINE_MODE = true.
    fact(facts, "setup-migrate-runs-with-offline-mode", made.status == 201, &[&made]);
    local_and_deletion(r, source, from, facts)
}

fn local_and_deletion(r: &mut Recorder<'_>, source: &Fixture, from: &str, facts: &mut Vec<Fact>) -> Result<()> {
    source.api("POST", "/api/v1/admin/users/fixture/orgs", br#"{"username":"copies","visibility":"private"}"#)?;
    let local =
        source.exchange("POST", MIGRATE, Some(body("local", from)?.as_bytes()), Some(&source.owner.token), None)?;
    // This response's Host is the source's port; mask it with its own primary map.
    let mut source_mask = Mask::new(source.instance.port());
    source_mask
        .listener(from.split(':').nth(2).and_then(|s| s.split('/').next()).ok_or("source port missing")?.parse()?);
    write::check("setup-migrate-local-refused", &local.sent, &local.body, r.secrets)?;
    write::case(
        r.out,
        "setup-migrate-local-refused",
        &mask::request(&mut source_mask, &local),
        &mask::response(&mut source_mask, &local),
        r.secrets,
    )?;
    fact(
        facts,
        "setup-migrate-of-a-local-address-is-refused-by-default",
        local.status >= 400
            && crate::json(&local)?
                .root()
                .get("message")
                .str()
                .is_some_and(|message| message == "You can not import from disallowed hosts."),
        &[&local],
    );
    let deleted = send(r, "setup-repository-delete", "DELETE", "/api/v1/repos/copies/whole", None)?;
    let again = send(r, "setup-repository-delete-again", "DELETE", "/api/v1/repos/copies/whole", None)?;
    let gone = read(r, "setup-repository-delete-read", "whole", "")?;
    fact(
        facts,
        "setup-repository-delete-is-204-then-404",
        deleted.status == 204 && again.status == 404 && gone.status == 404,
        &[&deleted, &again, &gone],
    );
    let user_org = send(r, "setup-migrate-org-of-user", "GET", "/api/v1/orgs/writer", None)?;
    let create_org = send(
        r,
        "setup-migrate-org-of-user-create",
        "POST",
        "/api/v1/admin/users/fixture/orgs",
        Some(r#"{"username":"writer","visibility":"private"}"#),
    )?;
    fact(
        facts,
        "setup-org-of-a-users-name-reads-404-and-making-it-is-refused",
        user_org.status == 404 && create_org.status >= 400,
        &[&user_org, &create_org],
    );
    Ok(())
}

fn scopes(r: &mut Recorder<'_>, from: &str, facts: &mut Vec<Fact>) -> Result<()> {
    for (name, scopes) in
        [("least", vec!["write:repository"]), ("without", vec!["read:issue"]), ("admin", setup::ADMIN.to_vec())]
    {
        let token = setup::scoped(r.f, r.secrets, "fixture", &format!("migrate-scope-{name}"), &scopes)?;
        let repository = format!("scope-{name}");
        let request = body(&repository, from)?;
        let made =
            r.send(&format!("setup-migrate-scope-{name}"), "POST", MIGRATE, Some(&request), Some(&token), None)?;
        // The refused migrate must not prevent measuring DELETE's independent scope check.
        if made.status != 201 {
            send(r, &format!("setup-migrate-scope-{name}-seed"), "POST", MIGRATE, Some(&request))?;
        }
        let deleted = r.send(
            &format!("setup-repository-delete-scope-{name}"),
            "DELETE",
            &format!("/api/v1/repos/copies/{repository}"),
            None,
            Some(&token),
            None,
        )?;
        let expected = if name == "without" { 403 } else { 201 };
        let delete_expected = if name == "without" { 403 } else { 204 };
        let denied_message = |x: &Exchange| -> bool {
            name != "without"
                || crate::json(x).is_ok_and(|j| {
                    j.root()
                        .get("message")
                        .is("token does not have at least one of required scope(s): [write:repository]")
                })
        };
        fact(
            facts,
            match name {
                "least" => "setup-scope-migrate-is-write-repository",
                "without" => "setup-scope-migrate-without-write-repository-is-refused",
                _ => "setup-scope-migrate-admin-is-admitted",
            },
            made.status == expected && denied_message(&made),
            &[&made],
        );
        fact(
            facts,
            match name {
                "least" => "setup-scope-delete-repository-is-write-repository",
                "without" => "setup-scope-delete-repository-without-write-repository-is-refused",
                _ => "setup-scope-delete-repository-admin-is-admitted",
            },
            deleted.status == delete_expected && denied_message(&deleted),
            &[&deleted],
        );
    }
    for (mode, scope, expected) in
        [("organization", "write:organization", 204), ("read-organization", "read:organization", 403)]
    {
        let name = format!("scope-delete-{mode}");
        send(r, &format!("setup-migrate-{name}-seed"), "POST", MIGRATE, Some(&body(&name, from)?))?;
        let token = setup::scoped(r.f, r.secrets, "fixture", &format!("delete-{mode}"), &[scope])?;
        let deleted = r.send(
            &format!("setup-repository-delete-scope-{mode}"),
            "DELETE",
            &format!("/api/v1/repos/copies/{name}"),
            None,
            Some(&token),
            None,
        )?;
        fact(
            facts,
            if mode == "organization" {
                "setup-scope-delete-repository-is-write-organization"
            } else {
                "setup-scope-delete-repository-read-organization-is-refused"
            },
            deleted.status == expected,
            &[&deleted],
        );
    }
    Ok(())
}

fn workflows(r: &mut Recorder<'_>, runner: Option<&Path>, from: &str, facts: &mut Vec<Fact>) -> Result<()> {
    let Some(binary) = runner else {
        facts.push(Fact {
            name: WORKFLOW,
            outcome: Outcome::Unchecked("--runner absent; source contains both workflow directories".into()),
        });
        return Ok(());
    };
    let (_runner, _version) = crate::runner::start(r.f, binary, r.secrets)?;
    crate::registration::online(r.f)?;
    send(r, "setup-migrate-workflows-created", "POST", MIGRATE, Some(&body("workflows", from)?))?;
    r.f.api("PATCH", "/api/v1/repos/copies/workflows", br#"{"has_actions":true}"#)?;
    r.f.put("/api/v1/repos/copies/workflows", "other", "trigger.txt", "trigger both directories\n")?;
    let mut runs = read(r, "setup-migrate-workflows-runs", "workflows", "/actions/runs?page=1&limit=2")?;
    for _ in 0..120 {
        let json = crate::json(&runs)?;
        let rows = json.root().get("workflow_runs").items();
        if !rows.is_empty() && rows.iter().all(|row| row.get("status").is("success") || row.get("status").is("failure"))
        {
            break;
        }
        thread::sleep(Duration::from_millis(500));
        runs = read(r, "setup-migrate-workflows-runs", "workflows", "/actions/runs?page=1&limit=2")?;
    }
    let json = crate::json(&runs)?;
    let rows = json.root().get("workflow_runs").items();
    fact(
        facts,
        WORKFLOW,
        runs.status == 200
            && rows.len() == 1
            && rows[0].get("workflow_id").is("forgejo-only.yaml")
            && rows[0].get("status").is("success"),
        &[&runs],
    );
    Ok(())
}

/// A loopback HTTP proxy holds upload-pack's response, after discovery and repository creation.
struct Stall {
    port: u16,
    arrived: mpsc::Receiver<()>,
    gate: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Stall {
    #[expect(
        clippy::disallowed_methods,
        reason = "the recorder holds real source sockets independently of the live migration"
    )]
    fn start(source: u16, hold: bool) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let (tx, arrived) = mpsc::channel();
        let gate = Arc::new((Mutex::new(!hold), Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_gate = gate.clone();
        let worker_stop = stop.clone();
        let worker = thread::spawn(move || {
            while !worker_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let tx = tx.clone();
                        let gate = worker_gate.clone();
                        thread::spawn(move || {
                            let _result = proxy(stream, source, &tx, &gate);
                        });
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(5)),
                    Err(_) => break,
                }
            }
        });
        Ok(Self { port, arrived, gate, stop, worker: Some(worker) })
    }
    fn waiting(&self) -> Result<()> {
        self.arrived
            .recv_timeout(Duration::from_secs(20))
            .map_err(|_| "source upload-pack did not reach the stall".into())
    }
    fn release(&self) {
        *self.gate.0.lock().expect("stall gate") = true;
        self.gate.1.notify_all();
    }
}
impl Drop for Stall {
    fn drop(&mut self) {
        self.release();
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().expect("proxy accept worker");
        }
    }
}
fn proxy(
    mut client: TcpStream,
    source: u16,
    arrived: &mpsc::Sender<()>,
    gate: &(Mutex<bool>, Condvar),
) -> io::Result<()> {
    client.set_read_timeout(Some(Duration::from_secs(20)))?;
    let mut reader = BufReader::new(client.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let hold = line.starts_with("POST ") && line.contains("/git-upload-pack ");
    let mut head = line.into_bytes();
    let mut length = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        if line == "\r\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:") {
            length = value.trim().parse().map_err(io::Error::other)?;
        }
        if !lower.starts_with("connection:") {
            head.extend_from_slice(line.as_bytes());
        }
    }
    head.extend_from_slice(b"Connection: close\r\n\r\n");
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let mut upstream = TcpStream::connect(("127.0.0.1", source))?;
    upstream.set_read_timeout(Some(Duration::from_secs(20)))?;
    upstream.write_all(&head)?;
    upstream.write_all(&body)?;
    let mut first = [0; 4096];
    let count = upstream.read(&mut first)?;
    if hold {
        let _sent = arrived.send(());
        let ready = gate.0.lock().expect("stall gate");
        let (_guard, result) =
            gate.1.wait_timeout_while(ready, Duration::from_secs(20), |ready| !*ready).expect("stall release");
        if result.timed_out() {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "stall not released"));
        }
    }
    client.write_all(&first[..count])?;
    io::copy(&mut upstream, &mut client)?;
    Ok(())
}

#[expect(
    clippy::disallowed_methods,
    reason = "the recorder holds a real migration request open while it stops and restarts its loopback Forgejo"
)]
fn interrupted(r: &mut Recorder<'_>, source: u16, facts: &mut Vec<Fact>) -> Result<()> {
    let stall = Stall::start(source, true)?;
    r.mask.listener(stall.port);
    let from = format!("http://127.0.0.1:{}/fixture/source.git", stall.port);
    let request = body("cut", &from)?;
    let instance = &r.f.instance;
    let token = &r.f.owner.token;
    thread::scope(|scope| -> Result<()> {
        let client = scope.spawn(|| {
            instance.exchange_patient(
                &Outgoing::json("POST", MIGRATE, Some(token), request.as_bytes()),
                skein_lib::Duration::from_secs(60),
            )
        });
        stall.waiting()?;
        let mid = read(r, "setup-migrate-cut-mid", "cut", "")?;
        let default =
            crate::json(&mid)?.root().get("default_branch").str().ok_or("mid-copy default missing")?.to_owned();
        let mid_branch = read(r, "setup-migrate-cut-mid-branch", "cut", &format!("/branches/{default}"))?;
        instance.kill().map_err(|e| crate::failure(&e))?;
        stall.release();
        let result = client.join().expect("migration client worker");
        if let Ok(answer) = &result {
            keep(r, "setup-migrate-cut-answer", answer)?;
        }
        instance.restart().map_err(|e| crate::failure(&e))?;
        let after = read(r, "setup-migrate-cut-after", "cut", "")?;
        let default =
            crate::json(&after)?.root().get("default_branch").str().ok_or("interrupted default missing")?.to_owned();
        let branch = read(r, "setup-migrate-cut-after-branch", "cut", &format!("/branches/{default}"))?;
        fact(
            facts,
            "setup-migrate-cut-short-leaves-a-repository-with-no-branch",
            mid.status == 200 && no_branch(&mid_branch) && after.status == 200 && no_branch(&branch),
            &[&mid, &mid_branch, &after, &branch],
        );
        let deleted = send(r, "setup-migrate-cut-delete", "DELETE", "/api/v1/repos/copies/cut", None)?;
        let copied = send(r, "setup-migrate-cut-copy-again", "POST", MIGRATE, Some(&request))?;
        let branch = read(r, "setup-migrate-cut-copy-again-branch", "cut", "/branches/other")?;
        fact(
            facts,
            "setup-migrate-cut-short-deletes-and-copies-again",
            deleted.status == 204 && copied.status == 201 && branch.status == 200,
            &[&deleted, &copied, &branch],
        );
        Ok(())
    })
}

fn disconnected(r: &mut Recorder<'_>, source: u16, delete: bool, facts: &mut Vec<Fact>) -> Result<()> {
    let stall = Stall::start(source, true)?;
    r.mask.listener(stall.port);
    let name = if delete { "deleted-live" } else { "client-gone" };
    let from = format!("http://127.0.0.1:{}/fixture/source.git", stall.port);
    let body = body(name, &from)?;
    let mut client = TcpStream::connect(("127.0.0.1", r.f.instance.port()))?;
    write!(
        client,
        "POST {MIGRATE} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: token {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        r.f.instance.port(),
        r.f.owner.token.value(),
        body.len()
    )?;
    stall.waiting()?;
    let mid = read(r, &format!("setup-migrate-{name}-mid"), name, "")?;
    client.shutdown(Shutdown::Both)?;
    drop(client);
    let deletion = if delete {
        Some(send(r, "setup-migrate-deleted-live-delete", "DELETE", &format!("/api/v1/repos/copies/{name}"), None)?)
    } else {
        None
    };
    stall.release();
    let mut last = read(r, &format!("setup-migrate-{name}-after"), name, "")?;
    let mut branch = read(r, &format!("setup-migrate-{name}-branch"), name, "/branches/other")?;
    // Observe the settled state after the held source bytes are released.
    for _ in 0..100 {
        if !delete && branch.status == 200 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
        last = read(r, &format!("setup-migrate-{name}-after"), name, "")?;
        branch = read(r, &format!("setup-migrate-{name}-branch"), name, "/branches/other")?;
    }
    if let Some(deletion) = deletion {
        let copied = send(r, "setup-migrate-deleted-live-copy-again", "POST", MIGRATE, Some(&body))?;
        let copied_branch = read(r, "setup-migrate-deleted-live-copy-again-branch", name, "/branches/other")?;
        facts.push(Fact { name: "setup-migrate-deleted-while-it-runs", outcome:
            if mid.status == 200 && deletion.status == 204 && last.status == 404 && branch.status == 404 && copied.status == 201 && copied_branch.status == 200 {
                Outcome::Observed("DELETE returned 204; after releasing the live migration, repository and branch stayed 404 for 5 seconds; copying the same name again returned 201 with its branch".into())
            } else { Outcome::Fails([&mid, &deletion, &last, &branch, &copied, &copied_branch].iter().map(|x| describe(x)).collect::<Vec<_>>().join("; ")) }
        });
    } else {
        fact(
            facts,
            "setup-migrate-goes-on-when-its-client-goes",
            mid.status == 200 && last.status == 200 && branch.status == 200,
            &[&mid, &last, &branch],
        );
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn a_migration_request_capture_frames_its_masked_source_address() {
    let body = body("copy", "http://127.0.0.1:41824/fixture/source.git").expect("body");
    let sent =
        format!("POST {MIGRATE} HTTP/1.1\r\nHost: 127.0.0.1:41823\r\nContent-Length: {}\r\n\r\n{body}", body.len())
            .into_bytes();
    let exchange = Exchange { sent, status: 201, head: Vec::new(), body: Vec::new() };
    let mut mask = Mask::new(41823);
    mask.listener(41824);
    let captured = mask::request(&mut mask, &exchange);
    let expected_body = self::body("copy", "http://127.0.0.1:3001/fixture/source.git").expect("body");
    let expected = format!(
        "POST {MIGRATE} HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nContent-Length: {}\r\n\r\n{expected_body}",
        expected_body.len()
    );
    assert_eq!(captured, expected.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::refused_and_left_empty;
    use skein_throwaway_forgejo::Exchange;

    fn exchange(status: u16, body: &str) -> Exchange {
        Exchange { sent: Vec::new(), status, head: Vec::new(), body: body.as_bytes().to_vec() }
    }

    fn recorded(name: &str) -> Exchange {
        let out = skein_scratch::paths::package();
        let bytes = std::fs::read(out.join("transcripts").join(name).join("response")).expect("recorded response");
        let end = bytes.windows(4).position(|w| w == b"\r\n\r\n").expect("case head");
        let head = bytes[..end].to_vec();
        let status = std::str::from_utf8(&head)
            .expect("UTF-8 head")
            .split_whitespace()
            .nth(1)
            .expect("case status")
            .parse()
            .expect("numeric status");
        Exchange { sent: Vec::new(), status, head, body: bytes[end + 4..].to_vec() }
    }

    #[test]
    fn recorded_refused_migration_leaves_an_empty_repository() {
        let refusal = recorded("setup-migrate-missing-source");
        let read = recorded("setup-migrate-missing-source-read");
        assert!(refused_and_left_empty(&refusal, &read).expect("recorded migration judgment"));
    }

    #[test]
    fn refused_migration_requires_an_error_with_a_message() {
        let read = exchange(200, r#"{"empty":true}"#);
        assert!(refused_and_left_empty(&exchange(400, r#"{"message":"refused"}"#), &read).expect("judgment"));
        for (status, body) in [
            (399, r#"{"message":"not an error"}"#),
            (422, "{}"),
            (422, r#"{"message":""}"#),
            (422, r#"{"message":true}"#),
        ] {
            assert!(!refused_and_left_empty(&exchange(status, body), &read).expect("judgment"), "{status}: {body}");
        }
    }

    #[test]
    fn refused_migration_requires_a_successful_read_of_an_empty_repository() {
        let refusal = exchange(422, r#"{"message":"refused"}"#);
        for (status, body) in [
            (404, r#"{"empty":true}"#),
            (201, r#"{"empty":true}"#),
            (200, r#"{"empty":false}"#),
            (200, "{}"),
            (200, r#"{"empty":"true"}"#),
        ] {
            assert!(!refused_and_left_empty(&refusal, &exchange(status, body)).expect("judgment"), "{status}: {body}");
        }
    }
}
