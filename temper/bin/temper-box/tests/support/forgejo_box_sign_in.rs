//! A box's own issuer and its real owned Forgejo (protocol/openid-provider.md, 9).
use super::{
    configuration,
    forge_sign_in::success,
    forgejo_sign_in::{Browser, Landed, Serve, csrf, fixture_bundle},
};
use jig_client_protocol_wire as wire;
use serde_json::Value;
use skein_lib::{
    Reader, Writer,
    base64::{self, Alphabet},
};
use std::{
    fs, io,
    net::SocketAddr,
    os::{
        fd::AsRawFd,
        unix::{fs::PermissionsExt, process::CommandExt},
    },
    path::Path,
    process::{Command, Stdio},
};

pub fn run(root: &Path) {
    if let Err(error) = story(root, false, cfg!(target_os = "linux")).and_then(|()| {
        let failed = root.join("source-failure");
        fs::create_dir(&failed)?;
        fs::set_permissions(&failed, fs::Permissions::from_mode(0o700))?;
        story(&failed, true, false)
    }) {
        super::story_socket::finish(error);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the process story keeps setup, restart and fallback assertions within one owned lifetime"
)]
fn story(root: &Path, fail_source: bool, delayed_engine: bool) -> io::Result<()> {
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let initialized = super::story_command::standalone(
        Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["init", "--box"])
            .args(configuration::flags(root))
            .current_dir(root),
        "initialize the sign-in box",
        super::WITHIN,
    );
    success(&initialized);
    super::own_listener_files(root);
    let path = root.join("temper.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path)?).expect("box JSON");
    let temper: SocketAddr = document["application"]["clients"]["listener"]["socket"]
        .as_str()
        .expect("clients socket")
        .parse()
        .expect("clients address");
    let forge: SocketAddr = document["application"]["machine"]["owned"]["listen"]["socket"]
        .as_str()
        .expect("forge socket")
        .parse()
        .expect("forge address");
    document["engine"]["clients"]["provider"]["clients"][0]["redirect_uri"] =
        format!("http://{forge}/user/oauth2/temper/callback").into();
    let front = if delayed_engine {
        let (proxy, public) = discovery_proxy(root, temper)?;
        document["application"]["clients"]["public"] = format!("http://{public}").into();
        Some((proxy, public))
    } else {
        None
    };
    let temper = front.as_ref().map_or(temper, |(_, public)| *public);
    // A runner is unrelated to the provider's integration with a real Forgejo.
    document["application"]["machine"]["owned"]["runner"] = Value::Null;
    super::write_private(&path, &serde_json::to_vec(&document).expect("box JSON"));
    if fail_source {
        let wrapper = root.join("forgejo-refuses-auth");
        let quoted = temper_forge_setup::sign_in::quote(super::owned::binaries().0.as_os_str().as_encoded_bytes());
        fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\nfor arg do\n  if [ \"$arg\" = auth ]; then exit 23; fi\ndone\nexec {} \"$@\"\n",
                std::str::from_utf8(&quoted).expect("shell word")
            ),
        )?;
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700))?;
        document["application"]["machine"]["owned"]["forgejo"] = wrapper.to_str().expect("wrapper path").into();
        super::write_private(&path, &serde_json::to_vec(&document).expect("failure configuration"));
    }
    fixture_bundle(root);
    super::first_repository::stage_binaries(root);
    let login = document["application"]["machine"]["owned"]["person"]["login"].as_str().expect("person login");
    let email = document["application"]["machine"]["owned"]["person"]["email"].as_str().expect("person email");
    assert!(super::private(&root.join("provider.key")), "private issuer key");
    assert_eq!(fs::metadata(root)?.permissions().mode() & 0o777, 0o700);
    for round in if fail_source { &["first"][..] } else { &["first", "second"][..] } {
        if delayed_engine && *round == "second" {
            delayed_forgejo(root, &path, &document)?;
        }
        let mut running = Serve {
            child: Command::new(skein_scratch::paths::binary("temper-box"))
                .args(["--echo", "steps"])
                .args(configuration::flags(root))
                .current_dir(root)
                .process_group(0)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(fs::File::create(root.join(format!("story-box-{round}.log")))?)
                .spawn()
                .expect("sign-in box"),
            reaped: false,
        };
        let stdout = running.child.stdout.take().expect("printed link pipe");
        let link = super::read_line(skein_io::kernel::Fd::new(stdout.as_raw_fd()), super::WITHIN);
        #[cfg(target_os = "linux")]
        let held_forge = if delayed_engine && *round == "second" {
            Some(start_without_engine(root, running.child.id())?)
        } else {
            None
        };
        // ctl is the client's setup act, with its ordinary session stored privately.
        let signed = super::story_command::output(
            Command::new(skein_scratch::paths::binary("temper-box"))
                .arg("ctl")
                .arg(root.join("ctl"))
                .args(["sign-in", "-"])
                .current_dir(root),
            "the setup client's sign-in",
            super::WITHIN,
            Some(&link),
        );
        success(&signed);
        if *round == "first" {
            success(&ctl_command(root, &["forge", "setup"]));
        }
        success(&ctl_command(root, &["forge", "list", "--until", "settled"]));
        wait_for_start(root, running.child.id())?;
        let listed = ctl_command(root, &["forge", "list", "--json"]);
        success(&listed);
        let row: Value = serde_json::from_slice(&listed.stdout).expect("forges JSON");
        let row = super::first_repository::owned_row(&row);
        assert_eq!(row["state"], "connected");
        assert_eq!(row["first"]["state"], "made");
        let before = users(root, forge);
        // Spending an already used link asks serve to print its next one.
        let mut browser = Browser::default();
        let _spent = link_sign_in(&mut browser, temper, &link)?;
        let link = super::read_line(skein_io::kernel::Fd::new(stdout.as_raw_fd()), super::WITHIN);
        assert_eq!(link_sign_in(&mut browser, temper, &link)?.status, 200);
        if delayed_engine && *round == "second" {
            wait_for_source(&mut browser, temper)?;
        }
        let page = browser.call(&format!("http://{temper}/"), None)?;
        let page = std::str::from_utf8(&page.body).expect("page text");
        if fail_source {
            assert!(!page.contains("name=\"temper-forge-sign-in\""), "failed source has no sign-in address");
            assert!(
                page.contains("<meta name=\"temper-forge-sign-in-off\" content=\"1\">"),
                "failed source has its off fact"
            );
            let log = super::boxed::box_log(root);
            assert_eq!(log.lines().filter(|line| line.contains("no sign-in through temper")).count(), 1, "{log}");
            assert!(log.contains("Forge 1 (Forgejo): started"), "setup succeeds: {log}");
            password_sign_in(root, forge, login)?;
            running.stop();
            continue;
        }
        assert!(
            page.contains(&format!("<meta name=\"temper-forge-sign-in\" content=\"1 forgejo http://{forge} owned\">")),
            "ready source meta"
        );
        assert!(!page.contains("temper-forge-sign-in-off"));
        for press in ["first", "second"] {
            let (landed, redirects) = repository(&mut browser, temper, forge)?;
            assert_eq!(landed.status, 200);
            assert_eq!(landed.address, format!("http://{forge}/work/hello"));
            assert_eq!(redirects, 3, "Forgejo start, issuer go, repository landing");
            signed_in(&mut browser, forge, login)?;
            eprintln!("{round} start, {press} press: {redirects} redirects; repository signed in");
        }
        assert_eq!(users(root, forge), before, "no second account");
        let (_, person) = api(root, forge, &format!("/users/{login}"));
        assert_eq!(person["login"], login);
        external_login(root, &format!("1:{email}"));
        password_sign_in(root, forge, login)?;
        let sources = auth_list(root);
        assert_eq!(
            sources.lines().filter(|line| line.split_whitespace().nth(1) == Some("temper")).count(),
            1,
            "exactly one source after either start"
        );
        let log = super::boxed::box_log(root);
        if delayed_engine && *round == "second" {
            assert!(log.contains("temper's discovery did not answer"), "engine was away at source's first run: {log}");
            #[cfg(target_os = "linux")]
            {
                let (pid, handle) = held_forge.expect("held Forgejo");
                assert_eq!(fs::read_to_string(root.join("delayed-forge.pid"))?, pid.to_string());
                assert!(!skein_shell::keeper::pidfd_exited(handle).expect("same Forgejo is live"));
                skein_shell::keeper::close_keeper_fd(handle);
            }
        } else {
            assert!(!log.contains("no sign-in through temper"), "{log}");
        }
        running.stop();
    }
    eprintln!("box sign-in proof: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    Ok(())
}

fn delayed_forgejo(root: &Path, path: &Path, document: &Value) -> io::Result<()> {
    let mut document = document.clone();
    let wrapper = root.join("delayed-forgejo");
    let quoted = temper_forge_setup::sign_in::quote(super::owned::binaries().0.as_os_str().as_encoded_bytes());
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nfor arg do\n if [ \"$arg\" = web ]; then\n  printf '%s' $$ > '{0}/delayed-forge.pid'\n  while [ ! -f '{0}/start-forge' ]; do sleep 0.01; done\n fi\ndone\nexec {1} \"$@\"\n",
            root.display(),
            std::str::from_utf8(&quoted).expect("quoted program")
        ),
    )?;
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700))?;
    document["application"]["machine"]["owned"]["forgejo"] = wrapper.to_str().expect("wrapper").into();
    document["application"]["limits"]["overrides"]["setup.deadline_ms"] = 200.into();
    super::write_private(path, &serde_json::to_vec(&document).expect("delayed-engine configuration"));
    Ok(())
}

#[cfg(target_os = "linux")]
fn start_without_engine(root: &Path, parent: u32) -> io::Result<(u32, skein_io::kernel::Fd)> {
    // Inspect this box's own children only, never another process's environment.
    let children = fs::read_to_string(format!("/proc/{parent}/task/{parent}/children"))?;
    let engine = children
        .split_whitespace()
        .find(|pid| {
            fs::read(format!("/proc/{pid}/cmdline"))
                .is_ok_and(|args| args.windows(b"internal\0serve\0".len()).any(|part| part == b"internal\0serve\0"))
        })
        .expect("this box's engine");
    let signal = |signal: &str| {
        assert!(Command::new("/bin/kill").args([signal, engine]).status().expect("signal own engine").success());
    };
    let deadline = skein_world::Deadline::arm("the prepared Forgejo waiting at its start gate", super::WITHIN);
    while !root.join("delayed-forge.pid").exists() {
        if deadline.passed() {
            return Err(wait_expired(&deadline));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    signal("-STOP");
    let result = (|| {
        fs::write(root.join("engine-down"), b"")?;
        fs::write(root.join("start-forge"), b"")?;
        wait_for_log(root, parent, "temper's discovery did not answer")?;
        let pid = fs::read_to_string(root.join("delayed-forge.pid"))?.parse().expect("Forgejo pid");
        let handle = skein_shell::keeper::open_pidfd(pid).expect("retain the live Forgejo");
        assert!(!skein_shell::keeper::pidfd_exited(handle).expect("Forgejo still live"));
        Ok((pid, handle))
    })();
    signal("-CONT");
    fs::remove_file(root.join("engine-down"))?;
    result
}

#[cfg(target_os = "linux")]
fn wait_for_log(root: &Path, pid: u32, text: &str) -> io::Result<()> {
    let mut deadline = skein_world::Deadline::arm("the source while the engine is held down", super::WITHIN);
    deadline.watch_process(pid);
    loop {
        if super::boxed::box_log(root).contains(text) {
            return Ok(());
        }
        if deadline.passed() {
            return Err(wait_expired(&deadline));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn wait_for_source(browser: &mut Browser, temper: SocketAddr) -> io::Result<()> {
    let deadline = skein_world::Deadline::arm("the source after the engine returns", super::WITHIN);
    loop {
        let page = browser.call(&format!("http://{temper}/"), None)?;
        if String::from_utf8_lossy(&page.body).contains("name=\"temper-forge-sign-in\"") {
            return Ok(());
        }
        if deadline.passed() {
            return Err(wait_expired(&deadline));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn wait_expired(deadline: &skein_world::Deadline) -> io::Error {
    match deadline.expired() {
        skein_world::Expired::Starved(starved) => io::Error::other(super::story_socket::StarvedSocket(starved)),
        skein_world::Expired::Failed(figures) => io::Error::new(io::ErrorKind::TimedOut, figures.to_string()),
    }
}

/// A single-loop HTTP front supplies the deployment's 502 while the real engine
/// is paused, then forwards to that same engine. It never logs a request.
fn discovery_proxy(root: &Path, upstream: SocketAddr) -> io::Result<(super::parent::Guard, SocketAddr)> {
    use std::os::fd::OwnedFd;
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let public = listener.local_addr()?;
    let script = r"import http.client,http.server,pathlib,socket,sys
root,host,port=sys.argv[1:]
class Peer(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args): pass
 def do_GET(self): self.forward()
 def do_POST(self): self.forward()
 def forward(self):
  if pathlib.Path(root,'engine-down').exists():
   self.send_response(502); self.send_header('Content-Length','0'); self.end_headers(); return
  body=self.rfile.read(int(self.headers.get('Content-Length','0')))
  peer=http.client.HTTPConnection(host,int(port))
  peer.request(self.command,self.path,body,dict(self.headers))
  answer=peer.getresponse(); data=answer.read()
  self.send_response(answer.status)
  for name,value in answer.getheaders():
   if name.lower() not in ('connection','transfer-encoding','content-length'): self.send_header(name,value)
  self.send_header('Content-Length',str(len(data))); self.end_headers(); self.wfile.write(data); peer.close()
server=http.server.HTTPServer(('127.0.0.1',0),Peer,bind_and_activate=False)
server.socket=socket.socket(fileno=0); server.server_address=server.socket.getsockname()
print('ready',flush=True)
server.serve_forever()
";
    let child = Command::new("python3")
        .args(["-I", "-B", "-c", script])
        .arg(root)
        .arg(upstream.ip().to_string())
        .arg(upstream.port().to_string())
        .stdin(Stdio::from(OwnedFd::from(listener)))
        .stdout(Stdio::piped())
        .stderr(fs::File::create(root.join("discovery-proxy.log"))?)
        .spawn()?;
    let mut proxy = super::parent::Guard::new(child);
    let pipe = proxy.child.as_mut().expect("proxy child").stdout.as_ref().expect("proxy stdout");
    assert_eq!(super::read_line(skein_io::kernel::Fd::new(pipe.as_raw_fd()), super::WITHIN), b"ready\n");
    Ok((proxy, public))
}

fn ctl_command(root: &Path, args: &[&str]) -> std::process::Output {
    super::story_command::output(
        Command::new(skein_scratch::paths::binary("temper-box"))
            .arg("ctl")
            .arg(root.join("ctl"))
            .args(args)
            .current_dir(root),
        "the owned setup act",
        super::boxed::BOX_DEADLINE,
        None,
    )
}

fn wait_for_start(root: &Path, pid: u32) -> io::Result<()> {
    let mut deadline = skein_world::Deadline::arm("the restored source's final outcome", super::WITHIN);
    deadline.watch_process(pid);
    loop {
        if super::boxed::box_log(root).contains("Forge 1 (Forgejo): started") {
            return Ok(());
        }
        if deadline.passed() {
            return match deadline.expired() {
                skein_world::Expired::Starved(starved) => {
                    Err(io::Error::other(super::story_socket::StarvedSocket(starved)))
                }
                skein_world::Expired::Failed(figures) => {
                    Err(io::Error::new(io::ErrorKind::TimedOut, figures.to_string()))
                }
            };
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn api(root: &Path, forge: SocketAddr, route: &str) -> (u16, Value) {
    super::first_repository::header_request(
        &format!("http://{forge}/api/v1"),
        root.join("forge-1/person_read.header").to_str().expect("header path"),
        "GET",
        route,
        None,
    )
}

fn users(root: &Path, forge: SocketAddr) -> Value {
    let (status, users) = api(root, forge, "/users/search?limit=50");
    assert_eq!(status, 200);
    users["data"]
        .as_array()
        .expect("user list")
        .iter()
        .map(|user| serde_json::json!({"id":user["id"], "login":user["login"]}))
        .collect::<Vec<_>>()
        .into()
}

fn auth_list(root: &Path) -> String {
    let output = super::story_command::output(
        Command::new(super::owned::binaries().0)
            .arg("--work-path")
            .arg(root.join("forge-1/forgejo"))
            .arg("--config")
            .arg(root.join("forge-1/forgejo/app.ini"))
            .args(["admin", "auth", "list"]),
        "the real source list",
        super::WITHIN,
        None,
    );
    success(&output);
    String::from_utf8(output.stdout).expect("source list text")
}

fn link_sign_in(browser: &mut Browser, temper: SocketAddr, link: &[u8]) -> io::Result<super::forgejo_sign_in::Answer> {
    let limits = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small).clients.wire;
    let link = std::str::from_utf8(link).expect("link text").trim();
    let secret = base64::decode(Alphabet::Url, link.split_once("secret=").expect("printed secret").1.as_bytes(), 32)
        .expect("link encoding");
    let record = wire::LinkSignIn::new(&limits, wire::LinkSignInParts { secret: skein_lib::Secret::new(secret) })
        .expect("link record");
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut out).expect("link wire");
    browser.call(&format!("http://{temper}/jig/v1/sign-in/link"), Some(&out.finish()))
}

fn repository(browser: &mut Browser, temper: SocketAddr, forge: SocketAddr) -> io::Result<(Landed, usize)> {
    let address = format!("http://{forge}/user/oauth2/temper?redirect_to=%2Fwork%2Fhello");
    let opened = browser.call(&address, None)?;
    assert_eq!(opened.status, 307);
    let mut redirects = usize::from(opened.location.is_some());
    let authorization = opened.location.expect("issuer redirect");
    assert!(authorization.starts_with(&format!("http://{temper}/oauth/authorize?")), "every press passes the issuer");
    assert_eq!(browser.call(&authorization, None)?.status, 200);
    let limits = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small).clients.wire;
    let record = wire::Authorize::new(
        &limits,
        wire::AuthorizeParts { query: authorization.split_once('?').expect("query").1.as_bytes().into() },
    )
    .expect("authorize record");
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut out).expect("authorize wire");
    let authorized = browser.call(&format!("http://{temper}/jig/v1/authorize"), Some(&out.finish()))?;
    assert_eq!(authorized.status, 200);
    let wire::AuthorizeAnswer::Go(go) =
        wire::AuthorizeAnswer::decode(&limits, &mut Reader::new(&authorized.body)).expect("authorize answer")
    else {
        panic!("owner receives go")
    };
    redirects += 1; // The client replaces the issuer page with its answer.
    let callback = std::str::from_utf8(go.location()).expect("callback URL");
    assert!(callback.starts_with(&format!("http://{forge}/user/oauth2/temper/callback?")));
    let answer = browser.call(callback, None)?;
    assert_eq!(answer.status, 303);
    assert_eq!(answer.location.as_deref(), Some(format!("http://{forge}/work/hello").as_str()), "no Link account hop");
    redirects += usize::from(answer.location.is_some());
    let address = answer.location.expect("repository redirect");
    let answer = browser.call(&address, None)?;
    let landed = Landed { status: answer.status, address, body: answer.body };
    Ok((landed, redirects))
}

fn signed_in(browser: &mut Browser, forge: SocketAddr, login: &str) -> io::Result<()> {
    let settings = browser.call(&format!("http://{forge}/user/settings"), None)?;
    assert_eq!(settings.status, 200);
    assert!(
        std::str::from_utf8(&settings.body).expect("profile page").contains(&format!("value=\"{login}\"")),
        "private profile names the person"
    );
    Ok(())
}

fn password_sign_in(root: &Path, forge: SocketAddr, login: &str) -> io::Result<()> {
    // This story alone reads the throwaway person's password; never print it.
    let password = fs::read_to_string(root.join("forge-1/person_password"))?;
    let mut browser = Browser::default();
    let page = browser.call(&format!("http://{forge}/user/login"), None)?;
    assert_eq!(page.status, 200);
    let csrf = csrf(&page.body).unwrap_or_default();
    let answer = browser.form(
        &format!("http://{forge}/user/login"),
        &[("user_name", login), ("password", password.trim()), ("_csrf", &csrf)],
    )?;
    assert!(matches!(answer.status, 302 | 303));
    signed_in(&mut browser, forge, login)
}

fn external_login(root: &Path, subject: &str) {
    // Existing internal accounts keep their password login_name; the linked
    // provider subject is a separate Forgejo identity, not a credential.
    let output = super::story_command::output(Command::new("python3")
        .args(["-I", "-B", "-c", "import sqlite3,sys; c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True); print('\\n'.join(x[0] for x in c.execute('select external_id from external_login_user')))"])
        .arg(root.join("forge-1/forgejo/forgejo.db")), "the external identity's subject", super::WITHIN, None);
    success(&output);
    assert_eq!(String::from_utf8(output.stdout).expect("external subject").trim(), subject);
}
