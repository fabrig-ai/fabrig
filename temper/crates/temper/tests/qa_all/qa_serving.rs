//! What temper's clients' listener promises a browser (client/browser.md, 2.2): its routes, its bundle, the
//! page's policy and the start's refusals of a bad bundle. A QA pass: aimed at the design, run by `--profile qa`.
use crate::support;
use skein_io::kernel::{Exit, Signal};
use skein_lib::Duration;
use skein_world::{Deadline, Expired, end_to_end::Binary};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

const PAGE: &[u8] = b"<!doctype html><head><title>qa</title></head><div id=\"app\"></div>\n";
// The page as served: the sign-in ways before its `</head>`, none configured (protocol/sign-in.md, 7).
const SERVED: &[u8] =
    b"<!doctype html><head><title>qa</title><meta name=\"temper-sign-in\" content=\"\"></head><div id=\"app\"></div>\n";
// A page with no `</head>` for the sign-in ways to go before.
const HEADLESS: &[u8] = b"<!doctype html><title>qa</title><div id=\"app\"></div>\n";
// A digest directory's name: 16 hex digits (ui-web.md, 4.2).
const DIGEST: &str = "0123456789abcdef";
const OTHER_DIGEST: &str = "fedcba9876543210";
const ROBOTS: &[u8] = b"User-agent: *\nDisallow: /\n";
// The digest directory's files, each with its type from the closed list (ui-web.md, 4.4).
const FILES: &[(&str, &[u8], &str)] = &[
    ("boot.js", b"import init from \"./qa.js\"; init();\n", "text/javascript"),
    ("qa.js", b"export default function init() {}\n", "text/javascript"),
    ("qa_bg.wasm", b"\0asm\x01\0\0\0", "application/wasm"),
    ("skein-ui.css", b"body { margin: 0; }\n", "text/css"),
    ("snippets/qa/one.js", b"export const one = 1;\n", "text/javascript"),
    ("files/hello.txt", b"hello\n", "text/plain; charset=utf-8"),
    ("files/mark.svg", b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n", "image/svg+xml"),
    ("files/robots.txt", ROBOTS, "text/plain; charset=utf-8"),
];
// The bundle's files, its page among them, and their bytes in all (configuration.md, section 7).
const BUNDLE_FILES: usize = FILES.len() + 1;
const BUNDLE_BYTES: usize = {
    let (mut total, mut at) = (PAGE.len(), 0);
    while at < FILES.len() {
        total += FILES[at].1.len();
        at += 1;
    }
    total
};
// The page's policy, on every answer temper gives (browser.md, 2.2).
const POLICY: [(&str, &str); 5] = [
    (
        "content-security-policy",
        "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; \
         connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
    ),
    ("x-content-type-options", "nosniff"),
    ("cross-origin-opener-policy", "same-origin"),
    ("cross-origin-resource-policy", "same-origin"),
    ("referrer-policy", "no-referrer"),
];
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const OCTETS: (&str, &str) = ("Content-Type", "application/octet-stream");
const FOREIGN: (&str, &str) = ("Origin", "https://evil.example");

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}
impl Reply {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(held, _)| held == name).map(|(_, value)| value.as_str())
    }
    fn policy(&self, what: &str) {
        for (name, value) in POLICY {
            assert_eq!(self.header(name), Some(value), "{name} on {what}");
        }
    }
}

#[derive(Clone, Copy)]
enum Layout {
    Whole,
    Headless,
    NoPage,
    NoDigest,
    Empty,
    Missing,
}
fn lay(root: &Path, layout: Layout) {
    let (page, digest) = match layout {
        Layout::Whole | Layout::Headless => (true, true),
        Layout::NoPage => (false, true),
        Layout::NoDigest => (true, false),
        Layout::Empty | Layout::Missing => (false, false),
    };
    fs::create_dir_all(root).expect("bundle directory");
    if page {
        let bytes = if matches!(layout, Layout::Headless) { HEADLESS } else { PAGE };
        fs::write(root.join("index.html"), bytes).expect("page");
    }
    for (path, bytes, _) in FILES.iter().filter(|_| digest) {
        let file = root.join(DIGEST).join(path);
        fs::create_dir_all(file.parent().expect("file's directory")).expect("digest directory");
        fs::write(file, bytes).expect("bundle file");
    }
    if matches!(layout, Layout::Missing) {
        fs::remove_dir(root).expect("no directory");
    }
}
fn configure(directory: &Path, edit: impl FnOnce(&mut serde_json::Value)) {
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("temper.json")).expect("deployment")).expect("deployment JSON");
    edit(&mut value["application"]);
    support::write_private(&directory.join("temper.json"), &serde_json::to_vec(&value).expect("deployment JSON"));
}
fn deployment(case: &str, layout: Layout) -> support::Scratch {
    let scratch = support::Scratch::new(case);
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let bundle = scratch.path().join("bundle");
    lay(&bundle, layout);
    configure(scratch.path(), |application| application["clients"]["bundle"] = bundle.to_str().expect("path").into());
    scratch
}

/// A read that goes on after a signal interrupts it.
fn read(stream: &mut TcpStream, chunk: &mut [u8], deadline: &Deadline) -> Result<usize, Expired> {
    loop {
        if deadline.passed() {
            return Err(deadline.expired());
        }
        let timeout =
            std::time::Duration::from_nanos(deadline.remaining().as_nanos()).max(std::time::Duration::from_millis(1));
        stream.set_read_timeout(Some(timeout)).expect("read timeout");
        match stream.read(chunk) {
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => panic!("an answer's bytes: {error}"),
            Ok(count) => return Ok(count),
        }
    }
}

/// `temper serve`, running, with its listener's port and its printed link's route.
struct Served {
    serve: Option<Binary>,
    link: support::terminal::Link,
    scratch: support::Scratch,
    port: u16,
    route: String,
}
impl Served {
    fn start(case: &str, bundle: bool) -> Self {
        let scratch = if bundle {
            deployment(case, Layout::Whole)
        } else {
            let scratch = support::Scratch::new(case);
            support::configuration(scratch.path(), &support::Forge::Unreached);
            scratch
        };
        let mut serve = support::serve(scratch.path());
        let link = support::terminal::link(&mut serve, support::WITHIN);
        let text = String::from_utf8(link.as_ref().to_vec()).expect("a link in text");
        let rest = text.strip_prefix("http://127.0.0.1:").expect("a loopback link");
        let (port, route) = rest.split_at(rest.find('/').expect("a route"));
        let port = port.parse().expect("a port");
        Self { serve: Some(serve), link, scratch, port, route: route.to_owned() }
    }
    fn stop(self) {
        drop((self.serve, self.link));
        self.scratch.remove();
    }
    fn send(&mut self, method: &str, path: &str, headers: &[(&str, &str)]) -> Reply {
        let host = format!("Host: 127.0.0.1:{}", self.port);
        let mut lines = vec![format!("{method} {path} HTTP/1.1"), "Connection: close".to_owned()];
        if !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("host")) {
            lines.push(host);
        }
        lines.extend(headers.iter().map(|(name, value)| format!("{name}: {value}")));
        if !["GET", "HEAD", "OPTIONS"].contains(&method) {
            lines.push("Content-Length: 0".to_owned());
        }
        let request = lines.join("\r\n") + "\r\n";
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).expect("the listener accepts");
        stream.write_all(format!("{request}\r\n").as_bytes()).expect("request sent");
        let mut deadline = Deadline::arm("the served HTTP answer", Duration::from_secs(10));
        deadline.watch(self.serve.as_ref().expect("running serve"));
        let result = (|| -> Result<Reply, Expired> {
            let mut raw = Vec::new();
            let end = loop {
                if let Some(at) = skein_lib::bytes::find(&raw, b"\r\n\r\n") {
                    break at + 4;
                }
                let mut chunk = [0; 4096];
                let count = read(&mut stream, &mut chunk, &deadline)?;
                assert_ne!(count, 0, "an answer, not a close: {method} {path}");
                raw.extend_from_slice(&chunk[..count]);
            };
            let head = String::from_utf8(raw[..end].to_vec()).expect("a head in text");
            let mut lines = head.split("\r\n");
            let status = lines.next().and_then(|line| line.split(' ').nth(1)).and_then(|code| code.parse().ok());
            let headers: Vec<(String, String)> = lines
                .filter_map(|line| line.split_once(':'))
                .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
                .collect();
            let mut reply = Reply { status: status.expect("a status"), headers, body: raw[end..].to_vec() };
            let length = reply.header("content-length").map(|length| length.parse::<usize>().expect("a length"));
            let bodiless = method == "HEAD" || reply.status == 304;
            while !bodiless && length.is_none_or(|length| reply.body.len() < length) {
                let mut chunk = [0; 4096];
                let count = read(&mut stream, &mut chunk, &deadline)?;
                if count == 0 {
                    break;
                }
                reply.body.extend_from_slice(&chunk[..count]);
            }
            Ok(reply)
        })();
        drop(stream);
        match result {
            Ok(reply) => reply,
            Err(Expired::Starved(starved)) => {
                drop(self.serve.take());
                starved.pass();
            }
            Err(Expired::Failed(figures)) => panic!("an answer's bytes: {method} {path}: {figures}"),
        }
    }
    fn get(&mut self, path: &str) -> Reply {
        self.send("GET", path, &[])
    }
}

#[test]
fn every_get_the_routes_leave_over_is_the_page_with_its_etag_under_no_cache() {
    let mut served = Served::start("page", true);
    // `/jigsaw` is not under `/jig`, and `/bundle` not under `/bundle/` (browser.md, 2.2, steps 1 and 4).
    let paths = ["/", "/projects/7/goals", "/jigsaw", "/bundle", "/bundlex/a", "/favicon.ico", "/index.html", "/?q=1"];
    let mut etag = None;
    for path in paths {
        let reply = served.get(path);
        assert_eq!(reply.status, 200, "{path}");
        assert_eq!(reply.header("content-type"), Some("text/html; charset=utf-8"), "{path}");
        assert_eq!(reply.header("cache-control"), Some("no-cache"), "{path}");
        assert_eq!(reply.body, SERVED, "{path}");
        let held = reply.header("etag").expect("an ETag");
        assert_eq!(*etag.get_or_insert(held.to_owned()), held, "one ETag for one page, at {path}");
        reply.policy(path);
    }
    served.stop();
}

#[test]
fn the_printed_links_route_answers_the_page_each_time_it_is_read() {
    let mut served = Served::start("link", true);
    for read in 1..=2 {
        let reply = served.get(&served.route.clone());
        assert_eq!((reply.status, reply.body.as_slice()), (200, SERVED), "read {read} of {}", served.route);
        assert_eq!(reply.header("cache-control"), Some("no-cache"), "read {read}");
        assert!(reply.header("etag").is_some(), "read {read}");
        reply.policy(&served.route);
    }
    served.stop();
}

#[test]
fn a_matching_if_none_match_is_304_where_a_stale_one_is_the_page() {
    let mut served = Served::start("etag", true);
    let etag = served.get("/").header("etag").expect("an ETag").to_owned();
    let stale = "\"stale\"";
    let rows = [
        (etag.clone(), 304),
        (format!("{stale}, {etag}"), 304),
        (format!("{etag}, {stale}"), 304),
        (format!("W/{etag}"), 304),
        ("*".to_owned(), 304),
        (stale.to_owned(), 200),
        (etag.trim_matches('"').to_owned(), 200),
        (String::new(), 200),
    ];
    let mut wrong = Vec::new();
    for (value, status) in rows {
        let reply = served.send("GET", "/projects/7", &[("If-None-Match", &value)]);
        if reply.status != status {
            wrong.push(format!("If-None-Match: {value} gave {}, not {status}", reply.status));
        }
        assert_eq!(reply.body.is_empty(), reply.status == 304, "a 304 holds no body, a 200 the page: {value}");
        assert_eq!(reply.header("etag"), Some(etag.as_str()), "{value}");
        assert_eq!(reply.header("cache-control"), Some("no-cache"), "{value}");
        reply.policy(&value);
    }
    assert!(wrong.is_empty(), "{}", wrong.join("; "));
    served.stop();
}

#[test]
fn a_request_other_than_get_is_405_with_the_policy_before_the_bundle_or_the_page() {
    let mut served = Served::start("not_get", true);
    let file = format!("/bundle/{DIGEST}/boot.js");
    let missing = format!("/bundle/{OTHER_DIGEST}/boot.js");
    let rows = [
        ("POST", "/"),
        ("PUT", "/projects/7"),
        ("DELETE", "/robots.txt"),
        ("PATCH", file.as_str()),
        ("POST", missing.as_str()),
        ("POST", "/bundle/"),
        ("HEAD", "/"),
        ("OPTIONS", "/"),
    ];
    for (method, path) in rows {
        let reply = served.send(method, path, &[]);
        assert_eq!(reply.status, 405, "{method} {path}");
        reply.policy(&format!("{method} {path}"));
    }
    served.stop();
}

#[test]
fn jigs_mapping_claims_every_path_under_jig_before_the_405_or_the_page() {
    let mut served = Served::start("jig_routes", true);
    let rows = [
        ("GET", "/jig/nothing-here"),
        ("GET", "/jig/"),
        ("POST", "/jig/nothing-here"),
        ("DELETE", "/jig/v1/nothing/here"),
        ("PUT", "/jig/a/b/c"),
        ("OPTIONS", "/jig/v1/read"),
    ];
    for (method, path) in rows {
        let reply = served.send(method, path, &[OCTETS, FOREIGN]);
        assert_ne!(reply.status, 405, "{method} {path} is the mapping's, not the 405");
        assert_ne!(reply.body, SERVED, "{method} {path} is the mapping's, not the page");
        assert_eq!(reply.header("cache-control"), Some("no-store"), "the mapping's answer: {method} {path}");
        assert!(reply.headers.iter().all(|(name, _)| !name.starts_with("access-control-")), "{method} {path}");
        reply.policy(path);
    }
    served.stop();
}

#[test]
fn a_host_off_the_allow_list_is_421_under_jig_with_the_policy() {
    let mut served = Served::start("host", true);
    let link = "/jig/v1/sign-in/link?secret=qa";
    let wrong_port = format!("127.0.0.1:{}", served.port ^ 1);
    let rows = [
        ("POST", "evil.example", link),
        ("GET", "evil.example", "/jig/nothing-here"),
        ("GET", wrong_port.as_str(), "/jig/nothing-here"),
        ("GET", "127.0.0.1", "/jig/nothing-here"),
        ("POST", "evil.example", "/jig/nothing-here"),
    ];
    for (method, host, path) in rows {
        let reply = served.send(method, path, &[("Host", host), OCTETS]);
        assert_eq!(reply.status, 421, "{method} {path} with Host: {host}");
        reply.policy(host);
    }
    served.stop();
}

// The cookie `jig-sign-in` carries the sign-in's credential (jig's client.md, 17.3), which only a sign-in's answer
// holds, and the process tier has nothing that makes that record: the stand-in cookie below authenticates nobody.
#[test]
#[ignore = "needs a signed-in credential: the process tier encodes no sign-in record to get one"]
fn a_cookie_without_jigs_browser_header_is_403_with_nothing_done() {
    let mut served = Served::start("cookie", true);
    let cookie = ("Cookie", "jig-sign-in=AAAA");
    let rows = [("POST", "/jig/v1/read", vec![cookie, OCTETS]), ("GET", "/jig/v1/stream", vec![cookie])];
    for (method, path, headers) in rows {
        let reply = served.send(method, path, &[headers.as_slice(), &[FOREIGN]].concat());
        assert_eq!(reply.status, 403, "{method} {path} with {headers:?}");
        assert!(reply.headers.iter().all(|(name, _)| !name.starts_with("access-control-")), "{method} {path}");
        reply.policy(path);
    }
    served.stop();
}

#[test]
fn each_bundle_file_is_served_immutable_as_its_type_with_its_bytes() {
    let mut served = Served::start("files", true);
    // A query is no part of the path (ui-web.md, 4.4: each file at `<prefix><digest>/<its path>`).
    let queried = [("boot.js?v=1", FILES[0].1, FILES[0].2)];
    for (path, bytes, kind) in FILES.iter().copied().chain(queried) {
        let url = format!("/bundle/{DIGEST}/{path}");
        let reply = served.get(&url);
        assert_eq!(reply.status, 200, "{url}");
        assert_eq!(reply.header("content-type"), Some(kind), "{url}");
        assert_eq!(reply.header("cache-control"), Some(IMMUTABLE), "{url}");
        assert_eq!(reply.body, bytes, "{url}");
        reply.policy(&url);
    }
    served.stop();
}

#[test]
fn a_path_under_bundle_that_is_no_file_of_the_digest_directory_is_not_found() {
    let mut served = Served::start("not_found", true);
    let paths = [
        "/bundle/".to_owned(),
        "/bundle/index.html".to_owned(),
        format!("/bundle/{OTHER_DIGEST}/boot.js"),
        format!("/bundle/{DIGEST}"),
        format!("/bundle/{DIGEST}/"),
        format!("/bundle/{DIGEST}/files"),
        format!("/bundle/{DIGEST}/missing.js"),
        format!("/bundle/{DIGEST}/index.html"),
        format!("/bundle/{DIGEST}/boot.js/"),
        format!("/bundle//{DIGEST}/boot.js"),
        format!("/bundle/{DIGEST}/../index.html"),
        format!("/bundle/{DIGEST}/../../temper.json"),
        format!("/bundle/{DIGEST}/%2e%2e/%2e%2e/temper.json"),
        format!("/bundle/{DIGEST}/..%2f..%2ftemper.json"),
        format!("/bundle/{DIGEST}/%2E%2E/index.html"),
    ];
    for path in paths {
        let reply = served.get(&path);
        assert_eq!(reply.status, 404, "{path}");
        assert_ne!(reply.body, SERVED, "{path}");
        reply.policy(&path);
    }
    served.stop();
}

#[test]
fn robots_txt_is_the_bundles_root_file_revalidated_disallowing_every_path() {
    let mut served = Served::start("robots", true);
    for path in ["/robots.txt", "/robots.txt?x=1"] {
        let reply = served.get(path);
        assert_eq!(reply.status, 200, "{path}");
        assert_eq!(reply.header("content-type"), Some("text/plain; charset=utf-8"), "{path}");
        assert_eq!(reply.header("cache-control"), Some("no-cache"), "{path}");
        assert!(skein_lib::bytes::find(&reply.body, b"Disallow: /\n").is_some(), "every path: {path}");
        reply.policy(path);
    }
    let etag = served.get("/robots.txt").header("etag").expect("an ETag").to_owned();
    let again = served.send("GET", "/robots.txt", &[("If-None-Match", &etag)]);
    assert_eq!((again.status, again.body.is_empty()), (304, true), "a matching If-None-Match");
    again.policy("the 304");
    served.stop();
}

#[test]
fn with_bundle_null_gets_serve_the_binarys_own_page_and_files() {
    // The binary's own bundle, or none built with TEMPER_BUNDLE=none (client/browser.md, 2.2 and 2.6).
    let own = temper::bundle::OWN;
    let mut served = Served::start("no_bundle", false);
    let fixture_file = format!("/bundle/{DIGEST}/boot.js");
    let route = served.route.clone();
    // Each row: the request, and its status from a binary without its bundle and from one with it.
    let rows = [
        ("GET", "/", 200, 200),
        ("GET", "/projects/7", 200, 200),
        ("GET", route.as_str(), 200, 200),
        ("GET", "/robots.txt", 404, 200),
        ("GET", fixture_file.as_str(), 404, 404),
        ("GET", "/jigsaw", 200, 200),
        ("HEAD", "/", 404, 405),
        ("POST", "/", 404, 405),
        ("DELETE", "/bundle/x", 404, 405),
        ("PUT", "/robots.txt", 404, 405),
    ];
    let page: &[u8] = match own {
        None => temper_service::clients::bundle::NO_BUNDLE_PAGE,
        Some(_) => temper_client_bundle::FILES.iter().find(|(name, _)| *name == "index.html").expect("embedded page").1,
    };
    for (method, path, without, with) in rows {
        let reply = served.send(method, path, &[]);
        let status = if own.is_some() { with } else { without };
        assert_eq!(reply.status, status, "{method} {path}");
        if method == "GET" && path != "/robots.txt" && !path.starts_with("/bundle/") {
            assert_eq!(reply.header("content-type"), Some("text/html; charset=utf-8"));
            assert_eq!(reply.body, page, "{method} {path}");
        } else if status == 404 {
            assert!(reply.body.is_empty());
        }
        reply.policy(&format!("{method} {path}"));
    }
    if let Some(digest) = own {
        let prefix = format!("{digest}/");
        let (name, bytes) =
            temper_client_bundle::FILES.iter().find(|(name, _)| name.starts_with(&prefix)).expect("embedded file");
        let reply = served.get(&format!("/bundle/{name}"));
        assert_eq!((reply.status, reply.body.as_slice()), (200, *bytes), "{name}");
        reply.policy(name);
    }
    served.stop();
}

/// A start that must refuse: `temper serve`'s exit code, `None` if it was still running after 30 s, and its stderr.
fn refusal(directory: &Path) -> (Option<i32>, String) {
    let mut serve = std::process::Command::new(skein_scratch::paths::binary("temper"))
        .arg("serve")
        .args(support::configuration::flags(directory))
        .current_dir(directory)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("temper serve starts");
    for _ in 0..1500 {
        if serve.try_wait().expect("a wait").is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let code = serve.try_wait().expect("a wait").map(|status| status.code().unwrap_or(-1));
    serve.kill().ok();
    let mut stderr = String::new();
    serve.stderr.take().expect("a pipe").read_to_string(&mut stderr).expect("stderr");
    (code, stderr)
}
/// A start that must go through: the code it exits with when signalled once it printed its link (0), and its stderr.
fn started(directory: &Path) -> (Option<i32>, String) {
    let ended = support::until_link_then(directory, Signal::Terminate, Duration::from_secs(60));
    let linked = support::printed_link(&ended.stdout).is_some();
    let code = (linked && ended.exit == Some(Exit::Code(0))).then_some(0);
    (code, String::from_utf8_lossy(&ended.stderr).into_owned())
}

#[test]
fn the_start_refuses_a_directory_without_a_bundle_or_a_bundle_past_its_limits() {
    // Each row: the directory's layout, the limits it sets, whether the start refuses (configuration.md, 3 and 7).
    let rows = [
        (Layout::Whole, None, None, false),
        (Layout::Headless, None, None, true),
        (Layout::NoPage, None, None, true),
        (Layout::NoDigest, None, None, true),
        (Layout::Empty, None, None, true),
        (Layout::Missing, None, None, true),
        (Layout::Whole, Some(BUNDLE_FILES), None, false),
        (Layout::Whole, Some(BUNDLE_FILES - 1), None, true),
        (Layout::Whole, None, Some(BUNDLE_BYTES), false),
        (Layout::Whole, None, Some(BUNDLE_BYTES - 1), true),
        (Layout::Whole, Some(0), None, true),
        (Layout::Whole, None, Some(0), true),
    ];
    for (row, (layout, files, bytes, refused)) in rows.into_iter().enumerate() {
        let scratch = deployment(&format!("start_{row}"), layout);
        configure(scratch.path(), |application| {
            for (name, limit) in [("clients.bundle_files", files), ("clients.bundle_bytes", bytes)] {
                if let Some(limit) = limit {
                    application["limits"]["overrides"][name] = limit.into();
                }
            }
        });
        let (exit, stderr) = if refused { refusal(scratch.path()) } else { started(scratch.path()) };
        assert_eq!(exit, Some(i32::from(refused)), "row {row}: {stderr}");
        assert!(!refused || !stderr.is_empty(), "a refusal says why: row {row}");
        scratch.remove();
    }
}
