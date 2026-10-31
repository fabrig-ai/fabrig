//! GitHub's kept App across a cut setup (temper/docs/design/draft/playground-reset.md, 4, 6.4 and 8).
use crate::support::{
    self, Ended, boxed,
    ctl::{self, Call, Step},
};
use skein_io::kernel::Exit;
use skein_lib::Duration;
use std::{
    cell::RefCell,
    fmt::Write as _,
    io::{Read, Write},
    net::SocketAddr,
    path::PathBuf,
    rc::Rc,
};

const CONVERTED_APP: u64 = 3;

struct Setup {
    scratch: support::Scratch,
    paths: [PathBuf; 3],
    bundle: PathBuf,
}
impl Setup {
    fn new(name: &str) -> Self {
        let scratch = support::Scratch::new(name);
        let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
        for path in &paths {
            std::fs::create_dir_all(path).expect("fixture directory");
        }
        let bundle = scratch.path().join("bundle");
        std::fs::create_dir_all(bundle.join("0000000000000001")).expect("bundle directory");
        support::write_private(&bundle.join("index.html"), b"<!doctype html><html><body>GitHub setup</body></html>");
        support::write_private(&bundle.join("0000000000000001/main.js"), b"// process fixture\n");
        Self { scratch, paths, bundle }
    }
    fn run(&self, steps: Vec<Step>) -> Vec<Ended> {
        let mut flags = vec!["--fake-backends".into(), "--bundle".into(), self.bundle.display().to_string()];
        for (flag, path) in ["--config", "--data", "--cache"].into_iter().zip(&self.paths) {
            flags.extend([flag.into(), path.display().to_string()]);
        }
        let count = steps.len();
        let mut all = vec![Step::Run(Call::sign_in())];
        all.extend(steps);
        let seen = boxed::run_ctl(self.scratch.path(), flags, self.scratch.path(), all, boxed::BOX_DEADLINE, |_| ());
        let diagnostic = String::from_utf8_lossy(&seen.stderr);
        assert_eq!(seen.exit, Some(Exit::Code(0)), "box SIGTERM drains: {diagnostic}");
        assert_eq!(seen.left, Some(Vec::new()), "all supervised children drain: {diagnostic}");
        assert_eq!(seen.calls.len(), count + 1, "whole story: {diagnostic}");
        assert!(seen.checked.is_some(), "live checkpoint reached before SIGTERM: {diagnostic}");
        for (index, call) in seen.calls.iter().enumerate() {
            assert_eq!(
                call.exit,
                Some(Exit::Code(0)),
                "ctl {index}: {}; {}; box: {diagnostic}",
                String::from_utf8_lossy(&call.stdout),
                String::from_utf8_lossy(&call.stderr)
            );
        }
        seen.calls.into_iter().skip(1).collect()
    }
    fn cut(&self) -> App {
        let converted = Rc::new(RefCell::new(None));
        let app = converted.clone();
        let fixture = self.paths[1].join("box/fake-backends/config/github.json");
        let checking = fixture.clone();
        let seen = self.run(vec![
            Step::Run(Call::new(&["forge", "connect", "github", "--key", "github-create"])),
            Step::Then(Box::new(move |calls| {
                let url = address(calls.last().expect("manifest address"));
                let install = convert(&url);
                assert!(install.contains("/apps/"), "conversion reaches installation");
                let made = App::new(&url, &install);
                app_count_is_one(&checking, &made);
                *app.borrow_mut() = Some(made);
                // Do not visit the installation: the box stops after this kept read.
                wait_forges()
            })),
        ]);
        let app = converted.borrow_mut().take().expect("one manifest conversion completed");
        listed(&app, &seen[1]);
        keep_fake_app(&fixture, &app);
        app
    }
}
#[derive(Clone)]
struct App {
    forge: String,
    slug: String,
    callback: String,
}
impl App {
    fn new(manifest: &str, installation: &str) -> Self {
        let forge =
            manifest.split("/forges/").nth(1).expect("forge route").split('/').next().expect("forge number").to_owned();
        let slug =
            installation.split("/apps/").nth(1).expect("App route").split('/').next().expect("App slug").to_owned();
        let callback = format!("{}/setup", manifest.split("/manifest?").next().expect("manifest route"));
        Self { forge, slug, callback }
    }
}

// The external peer outlives temper. Recreate its converted public facts when
// the box respawns the in-memory fake, without touching temper's kept state.
fn keep_fake_app(fixture: &std::path::Path, app: &App) {
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture).expect("public fake descriptor")).expect("fake JSON");
    document["github"]["app"]["id"] = CONVERTED_APP.into();
    document["github"]["app"]["slug"] = app.slug.clone().into();
    document["github"]["app"]["setup_url"] = app.callback.clone().into();
    document["github"]["accounts"][0]["installation"] = serde_json::Value::Null;
    support::write_private(fixture, &serde_json::to_vec(&document).expect("public fake descriptor"));
}
fn address(call: &Ended) -> String {
    String::from_utf8_lossy(&call.stdout)
        .lines()
        .find(|line| line.starts_with("http://"))
        .expect("loopback browser address")
        .to_owned()
}
fn wait_forges() -> Call {
    Call::new(&["forge", "list", "--until", "settled", "--timeout", "20s"])
}
fn listed(app: &App, call: &Ended) {
    let text = String::from_utf8_lossy(&call.stdout);
    assert!(text.contains(&format!("forge {} ", app.forge)), "unfinished forge kept: {text}");
    assert!(text.contains(&app.slug), "the same App kept: {text}");
    assert!(text.contains("installed nowhere yet"), "the kept App is unfinished: {text}");
}

#[test]
fn a_github_set_up_cut_after_its_app_goes_on_after_a_restart() {
    let setup = Setup::new("github-go-on");
    let app = setup.cut();
    let forge = app.forge.clone();
    let unfinished = app.clone();
    let fixture = setup.paths[1].join("box/fake-backends/config/github.json");
    let complete = app.clone();
    let seen = setup.run(vec![
        Step::Run(wait_forges()),
        Step::Then(Box::new(move |calls| {
            listed(&unfinished, calls.last().expect("unfinished forge read"));
            Call::new(&["forge", "install", &unfinished.forge, "--key", "github-go-on"])
        })),
        Step::Then(Box::new(|calls| {
            install(&address(calls.last().expect("Go on address")));
            Call::new(&["forge", "list", "--until", "settled", "--timeout", "20s", "--json"])
        })),
        Step::Run(Call::new(&[
            "repository",
            "adopt",
            "--project",
            "1",
            "--forge",
            &forge,
            "--owner",
            "fake-person",
            "--name",
            "widget",
            "--landing",
            "main",
            "--key",
            "github-widget",
            "--json",
        ])),
        Step::Then(Box::new(move |_| {
            app_count_is_one(&fixture, &complete);
            Call::new(&["forge", "list"])
        })),
    ]);
    listed(&app, &seen[0]);
    let document = ctl::json(&seen[2]);
    let number: u64 = forge.parse().expect("forge number");
    let row = document["forges"]["forges"]
        .as_array()
        .expect("settled rows")
        .iter()
        .find(|row| row["number"] == number)
        .expect("GitHub row");
    assert_eq!(row["state"], "connected", "installation connected: {row}");
    let adopted = ctl::json(&seen[3]);
    assert_eq!(adopted["adopted"]["forge"], number);
    assert_eq!(adopted["adopted"]["owner"], "fake-person");
    assert_eq!(adopted["adopted"]["name"], "widget");
    assert_eq!(adopted["adopted"]["home"], true);
    assert!(address(&seen[1]).contains(&format!("/apps/{}/", app.slug)), "Go on uses the kept App");
    setup.scratch.remove();
}

#[test]
fn a_github_started_over_names_the_app_as_the_persons() {
    let setup = Setup::new("github-start-over");
    let app = setup.cut();
    let remaining = app.clone();
    let unfinished = app.clone();
    let fixture = setup.paths[1].join("box/fake-backends/config/github.json");
    let seen = setup.run(vec![
        Step::Run(wait_forges()),
        Step::Then(Box::new(move |calls| {
            listed(&unfinished, calls.last().expect("unfinished forge read"));
            Call::new(&["forge", "remove", &unfinished.forge, "--key", "github-remove"])
        })),
        Step::Then(Box::new(move |_| {
            // Removing temper's record leaves the person's App on GitHub.
            app_count_is_one(&fixture, &remaining);
            Call::new(&["forge", "list", "--json"])
        })),
    ]);
    listed(&app, &seen[0]);
    let removed = String::from_utf8_lossy(&seen[1].stdout);
    assert!(removed.contains("removed"), "Start over answers removed: {removed}");
    let rows = ctl::json(&seen[2]);
    let number: u64 = app.forge.parse().expect("forge number");
    assert!(
        rows["forges"].as_array().expect("forge rows").iter().all(|row| row["number"] != number),
        "removed forge absent: {rows}"
    );
    let restarted = setup.run(vec![Step::Run(Call::new(&["forge", "list", "--json"]))]);
    let rows = ctl::json(&restarted[0]);
    assert!(
        rows["forges"].as_array().expect("restarted rows").iter().all(|row| row["number"] != number),
        "removed forge stays absent after another restart: {rows}"
    );
    setup.scratch.remove();
}

struct Response {
    status: u16,
    location: Option<String>,
    body: String,
}
fn request(url: &str, body: Option<&str>) -> Response {
    exchange(url, body, None)
}
fn exchange(url: &str, body: Option<&str>, bearer: Option<&[u8]>) -> Response {
    let (authority, path) = url.strip_prefix("http://").expect("plaintext fixture").split_once('/').expect("URL path");
    let address: SocketAddr = authority.parse().expect("literal loopback address");
    assert!(address.ip().is_loopback(), "fake GitHub only");
    let mut socket = support::socket::Socket::connect(
        address,
        "GitHub setup connection",
        Duration::from_secs(5),
        ["GitHub setup response", "GitHub setup request"],
        true,
        None,
    )
    .expect("fake GitHub connection");
    let method = if body.is_some() { "POST" } else { "GET" };
    write!(socket, "{method} /{path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n").expect("HTTP head");
    if let Some(bearer) = bearer {
        socket.write_all(b"Authorization: Bearer ").expect("fixture authorization header");
        socket.write_all(bearer).expect("fixture bearer");
        write!(
            socket,
            "\r\nUser-Agent: temper-process\r\nX-GitHub-Api-Version: {}\r\n",
            skein_forge::github::pin::API_VERSION
        )
        .expect("GitHub API headers");
    }
    if let Some(body) = body {
        write!(socket, "Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n", body.len())
            .expect("form headers");
    }
    socket.write_all(b"\r\n").expect("header end");
    if let Some(body) = body {
        socket.write_all(body.as_bytes()).expect("manifest form");
    }
    let mut bytes = Vec::new();
    socket.take(65_537).read_to_end(&mut bytes).expect("bounded response");
    assert!(bytes.len() <= 65_536, "response fits");
    let text = String::from_utf8(bytes).expect("HTTP UTF-8");
    let (headers, body) = text.split_once("\r\n\r\n").expect("HTTP headers");
    let status = headers.split_whitespace().nth(1).expect("HTTP status").parse().expect("status number");
    let location = headers
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.eq_ignore_ascii_case("location"))
        .map(|(_, value)| value.trim().to_owned());
    Response { status, location, body: body.into() }
}
fn attribute(tag: &str, name: &str) -> String {
    let marker = format!("{name}=\"");
    tag.split_once(&marker)
        .expect("HTML attribute")
        .1
        .split('"')
        .next()
        .expect("attribute value")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
fn convert(url: &str) -> String {
    let page = request(url, None);
    assert_eq!(page.status, 200, "manifest page");
    let form = page.body.split("<form ").nth(1).expect("manifest form");
    let action = attribute(form, "action");
    let input = page.body.split("<input ").find(|tag| tag.contains("name=\"manifest\"")).expect("manifest input");
    let mut encoded = String::from("manifest=");
    for byte in attribute(input, "value").bytes() {
        write!(encoded, "%{byte:02X}").expect("form encoding");
    }
    let create = request(&action, Some(&encoded));
    assert_eq!(create.status, 200, "GitHub Create page");
    let link = create.body.split("<a ").find(|tag| tag.contains("id=\"create\"")).expect("Create link");
    let conversion = request(&attribute(link, "href"), None);
    assert_eq!(conversion.status, 302, "App conversion saved before installation");
    conversion.location.expect("installation address")
}
fn install(url: &str) {
    let reply = request(url, None);
    assert_eq!(reply.status, 302, "GitHub installs the kept App");
    let done = request(&reply.location.expect("installation callback"), None);
    assert_eq!(done.status, 200, "installation accepted");
}

// The fake starts at App 1, skips bot 2, and increments on every conversion
// (skein/crates/skein-fake-forge-github/src/pages.rs). App 3 proves one creation.
fn app_count_is_one(fixture: &std::path::Path, app: &App) {
    use rustls::pki_types::{PrivateKeyDer, pem::PemObject};
    use skein_forge::github::jwt;
    let bytes = std::fs::read(fixture).expect("public fake GitHub descriptor");
    let document = temper_fake::forge::Document::decode(&bytes).expect("fake descriptor");
    // Generate the fixed fake's signing material; no deployment credential is read.
    let pem = document.github_app_key().expect("fixed fixture key");
    let key = PrivateKeyDer::from_pem_slice(&pem).expect("fixture PEM");
    let now = skein_shell::Clock::new().now().wall;
    let input = jwt::signing_input(
        CONVERTED_APP,
        now,
        skein_lib::Wall::from_nanos(
            now.as_nanos().checked_add(Duration::from_secs(60).as_nanos()).expect("fixture expiry"),
        ),
    )
    .expect("fixture JWT input");
    let signature = skein_tls::sign_rs256(key.secret_der(), &input).expect("fixture signature");
    let token = jwt::token(&input, &signature).expect("fixture JWT");
    let reply = exchange(&format!("http://{}/app", document.listen()), None, Some(token.expose()));
    assert_eq!(reply.status, 200, "the one converted App remains on fake GitHub");
    let identity: serde_json::Value = serde_json::from_str(&reply.body).expect("fake App identity");
    assert_eq!(identity["id"], CONVERTED_APP, "one App conversion, with no second App");
    assert_eq!(identity["slug"], app.slug, "the person's same App remains");
}
