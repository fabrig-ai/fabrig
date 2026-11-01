//! Black-box contracts from lab-browser.md, section 8.
use skein_lab_browser::json;
#[expect(dead_code, reason = "the shared reader exposes answer helpers for capture stories")]
mod support;

use json::Value;
use skein_scratch::Scratch;
use std::os::unix::process::CommandExt;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Command, Stdio},
};
use support::{Child, Reader, field, text};

struct Hand {
    _scratch: Scratch,
    child: Child,
    reader: Reader,
    address: SocketAddr,
    origin: String,
    _server: Child,
}
impl Hand {
    fn start(html: &str) -> Self {
        Self::with_peers(html, &[])
    }
    fn with_peers(html: &str, peers: &[String]) -> Self {
        let scratch = Scratch::new("browser-paths");
        for dir in ["profile", "out", "www"] {
            fs::create_dir(scratch.path().join(dir)).expect("fixture directory");
        }
        fs::write(scratch.path().join("www/index.html"), html).expect("static fixture");
        fs::write(scratch.path().join("www/favicon.ico"), []).expect("favicon");
        let mut server = Child(
            Command::new("python3")
                .args(["-u", "-m", "http.server", "0", "--bind", "127.0.0.1", "--directory"])
                .arg(scratch.path().join("www"))
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .process_group(0)
                .spawn()
                .expect("standard HTTP server"),
        );
        let mut line = String::new();
        BufReader::new(server.0.stdout.take().expect("server stdout")).read_line(&mut line).expect("server ready");
        let port = line.split(" port ").nth(1).expect("server port").split_whitespace().next().expect("port");
        let origin = format!("http://127.0.0.1:{port}");
        let stderr = scratch.path().join("stderr");
        let mut child = Child(
            Command::new(skein_scratch::paths::binary("skein-lab-browser"))
                .arg("hand")
                .arg("--chromium")
                .arg(support::chromium())
                .arg("--fonts")
                .arg(support::fonts())
                .arg("--scratch")
                .arg(scratch.path().join("profile"))
                .arg("--out")
                .arg(scratch.path().join("out"))
                .arg("--origin")
                .arg(&origin)
                .args(peers.iter().flat_map(|peer| ["--peer", peer.as_str()]))
                .args(["--size", "320x240", "--listen", "127.0.0.1:0"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(File::create(&stderr).expect("stderr"))
                .process_group(0)
                .spawn()
                .expect("hand process"),
        );
        let mut reader =
            Reader::new(child.0.stdout.take().expect("hand stdout"), stderr, vec![child.0.id(), server.0.id()]);
        let ready = String::from_utf8(reader.line()).expect("ready UTF-8");
        let address: SocketAddr =
            ready.strip_prefix("listening ").expect("8.1 ready line").parse().expect("bound address");
        assert!(address.ip().is_loopback() && address.port() != 0);
        Self { _scratch: scratch, child, reader, address, origin, _server: server }
    }
    fn connect(&self) -> TcpStream {
        TcpStream::connect(self.address).expect("connect hand")
    }
    fn send(&self, stream: &mut TcpStream, bytes: &[u8], why: &str) {
        let deadline = self.reader.deadline("the hand socket write", skein_lib::Duration::from_secs(6));
        deadline.write(stream, bytes, why);
    }
    fn call(&self, line: &str) -> Value {
        let mut stream = self.connect();
        self.send(&mut stream, format!("{line}\n").as_bytes(), "send call");
        answer(self, stream)
    }
    fn go(&self) {
        let result = self.call(r#"{"call":"go","path":"/index.html"}"#);
        assert_eq!(text(field(&result, "went")), format!("{}/index.html", self.origin));
    }
    fn stop(&mut self, signal: &str) {
        assert!(
            Command::new("kill")
                .args([signal, &self.child.0.id().to_string()])
                .status()
                .expect("signal hand")
                .success()
        );
        let deadline = self.reader.deadline("the hand shutdown exit", skein_lib::Duration::from_secs(10));
        loop {
            if let Some(status) = self.child.0.try_wait().expect("wait hand") {
                deadline.verdict(status.success());
                assert!(status.success(), "8.1: {status}; {}", self.reader.stderr());
                return;
            }
            deadline.check(&format!("8.1 shutdown deadline; {}", self.reader.stderr()));
            self.reader.pause();
        }
    }
}
fn answer(hand: &Hand, stream: TcpStream) -> Value {
    let deadline = hand.reader.deadline("the hand socket answer and close", skein_lib::Duration::from_secs(28));
    let mut reader = BufReader::new(stream);
    let mut line = Vec::new();
    deadline.read_line(&mut reader, &mut line, "answer before deadline");
    assert_eq!(line.pop(), Some(b'\n'), "8.2 answer newline");
    let result = json::parse(&line).expect("answer JSON");
    assert_eq!(json::encode(&result), line, "canonical answer");
    let mut rest = Vec::new();
    deadline.read_end(&mut reader, &mut rest, "8.2 closes after one answer", false);
    assert!(rest.is_empty(), "8.2 one answer a connection");
    result
}

#[test]
fn sigint_ends_the_ready_hand_with_status_zero() {
    let mut hand = Hand::start("<h1>Fixture</h1>");
    assert_eq!(text(field(&hand.call(r#"{"call":"address"}"#), "address")), "about:blank");
    hand.stop("-INT");
}

#[test]
fn a_named_peer_is_reached_and_any_other_port_is_still_refused() {
    let peer = TcpListener::bind("127.0.0.1:0").expect("peer listener");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let peer_address = peer.local_addr().expect("peer address");
    let forbidden = TcpListener::bind("127.0.0.1:0").expect("canary listener");
    forbidden.set_nonblocking(true).expect("nonblocking canary");
    let forbidden_address = forbidden.local_addr().expect("canary address");
    let mut hand = Hand::with_peers(
        &format!(
            r#"<h1>Fixture</h1><a href="http://{peer_address}/link">Peer</a><script>fetch('http://{forbidden_address}/fetch').catch(()=>{{}})</script>"#
        ),
        &[format!("http://{peer_address}")],
    );
    hand.go();
    let pressed = hand.call(r#"{"call":"press","in":[],"match":"exact","name":"Peer","role":"link"}"#);
    let _ = field(&pressed, "pressed");
    let deadline = hand.reader.deadline("the named peer connection", skein_lib::Duration::from_secs(6));
    let stream = loop {
        match peer.accept() {
            Ok((stream, _)) => break stream,
            Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock),
        }
        deadline.check("8.5 named peer reached");
        hand.reader.pause();
    };
    let deadline = hand.reader.deadline("the named peer request", skein_lib::Duration::from_secs(6));
    let mut request = Vec::new();
    let mut peer_reader = BufReader::new(stream);
    deadline.read_line(&mut peer_reader, &mut request, "peer request");
    assert_eq!(request, b"GET /link HTTP/1.1\r\n");
    let mut stream = peer_reader.into_inner();
    let body = "<h1>Named peer</h1>";
    write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .expect("peer response");
    let met = hand.call(
        r#"{"call":"wait","gone":false,"in":[],"match":"exact","name":"Named peer","role":"heading","seconds":6}"#,
    );
    let _ = field(&met, "met");
    let trouble = hand.call(r#"{"call":"trouble"}"#);
    let Value::Array(entries) = field(&trouble, "trouble") else { panic!("{trouble:?}") };
    assert!(
        entries.iter().any(|entry| text(field(entry, "kind")) == "network"
            && text(field(entry, "text")).contains(&forbidden_address.port().to_string())),
        "8.5: {trouble:?}"
    );
    assert!(
        !entries.iter().any(|entry| text(field(entry, "kind")) == "network"
            && text(field(entry, "text")).contains(&peer_address.port().to_string())),
        "8.5: {trouble:?}"
    );
    assert_eq!(forbidden.accept().expect_err("8.5 unnamed port never reached").kind(), std::io::ErrorKind::WouldBlock);
    hand.stop("-TERM");
}

#[test]
fn browser_death_ends_hand_with_diagnostic_and_status_one() {
    let mut hand = Hand::start("<h1>Fixture</h1>");
    // The kernel lists this process's own children; no process environment is read.
    let pid = hand.child.0.id();
    let children = fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")).expect("hand child PIDs");
    assert!(!children.trim().is_empty(), "8.1 spawned browser");
    // Hand is our unreaped child. Bind each candidate before checking that
    // the opened identity is still its child, then signal only kept pidfds.
    let browsers: Vec<_> = children
        .split_whitespace()
        .map(|child| {
            let child = child.parse().expect("browser PID");
            skein_shell::open_child_pidfd(child, pid).expect("validate hand's browser identity")
        })
        .collect();
    for browser in browsers {
        let sent =
            skein_shell::signal_kept_child(browser, skein_io::kernel::Signal::Kill, skein_io::kernel::Target::Child);
        skein_shell::close_keeper_fd(browser);
        assert!(sent.is_ok(), "kill kept browser: {sent:?}");
    }
    let deadline = hand.reader.deadline("the hand exit after browser death", skein_lib::Duration::from_secs(10));
    loop {
        if let Some(status) = hand.child.0.try_wait().expect("wait hand") {
            assert_eq!(status.code(), Some(1), "8.1 browser death status");
            assert!(!hand.reader.stderr().trim().is_empty(), "8.1 browser death diagnostic");
            break;
        }
        deadline.check("8.1 browser death must end hand");
        hand.reader.pause();
    }
}
