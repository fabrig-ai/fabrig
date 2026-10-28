//! Signed deliveries against the shipped listener.
use super::{Process, StdoutReader};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::fmt::Write as _;
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    os::fd::AsRawFd,
    path::Path,
};

pub const SECRET: &[u8] = b"fixture-hook-secret";
// protocol/forge.md 5 and protocol/configuration.md 3 give forge 1 the
// listener path; forge 9 is not configured.
pub const KNOWN_FORGE: &str = "/hooks/1";
pub const UNKNOWN_FORGE: &str = "/hooks/9";
// The listener stories allow five seconds for each socket operation.
const SOCKET_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);
pub const PING: &[u8] = b"ping";

#[must_use]
pub fn configuration(directory: &Path) -> SocketAddr {
    super::configuration(directory, &super::Forge::Unreached);
    let [address] = super::free_addresses(&[]);
    let path = directory.join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["webhooks"] = serde_json::json!({"listener":{"socket":address.to_string()}});
    value["application"]["forges"][0]["webhook_secret"] = "secrets/forge-1.webhook-secret".into();
    super::write_private(&directory.join("secrets/forge-1.webhook-secret"), SECRET);
    super::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
    address
}
#[must_use]
pub fn signature(secret: &[u8], body: &[u8]) -> String {
    let mut hmac = skein_lib::sha256::Hmac::new(secret);
    hmac.update(body);
    let mut signature = String::with_capacity(64);
    for byte in hmac.finish() {
        write!(signature, "{byte:02x}").expect("signature text");
    }
    signature
}
pub struct Post<'a> {
    path: &'a str,
    body: &'a [u8],
    event: &'a [u8],
    signed: bool,
}
impl<'a> Post<'a> {
    #[must_use]
    pub fn signed(path: &'a str, body: &'a [u8], event: &'a [u8]) -> Self {
        Self { path, body, event, signed: true }
    }
    #[must_use]
    pub fn unsigned(path: &'a str, body: &'a [u8], event: &'a [u8]) -> Self {
        Self { path, body, event, signed: false }
    }
    fn send(&self, listener: SocketAddr) -> u16 {
        let mut stream = TcpStream::connect_timeout(&listener, SOCKET_DEADLINE).expect("listener");
        stream.set_read_timeout(Some(SOCKET_DEADLINE)).expect("read deadline");
        stream.set_write_timeout(Some(SOCKET_DEADLINE)).expect("write deadline");
        write!(stream, "POST {} HTTP/1.1\r\nHost: {listener}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Forgejo-Event: {}\r\n",
            self.path, self.body.len(), std::str::from_utf8(self.event).expect("event")).expect("request head");
        if self.signed {
            write!(stream, "X-Forgejo-Signature: {}\r\n", signature(SECRET, self.body)).expect("signature");
        }
        stream.write_all(b"\r\n").expect("head end");
        stream.write_all(self.body).expect("body");
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).expect("response");
        line.split_whitespace().nth(1).expect("status").parse().expect("status number")
    }
}
pub struct Ended {
    pub statuses: Vec<u16>,
    pub exit: Option<Exit>,
    pub stderr: Vec<u8>,
}
struct Judge<'a> {
    listener: SocketAddr,
    posts: &'a [Post<'a>],
    statuses: &'a mut Vec<u16>,
    sent: bool,
    settled: bool,
    deadline: Time,
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, _: Time, procs: &mut [Process]) {
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        if !self.sent && super::printed_link(&stdout.bytes).is_some() {
            self.statuses.extend(self.posts.iter().map(|post| post.send(self.listener)));
            let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
            binary.signal(Signal::Terminate);
            self.sent = true;
        }
    }
    fn observe(&mut self, _: Time, procs: &[Process]) {
        self.settled = self.sent && procs.iter().all(Host::is_empty);
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        assert!(
            self.sent || binary.exit_status().is_none(),
            "serve failed: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "webhook delivery or drain did not finish".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
pub fn served(directory: &Path, listener: SocketAddr, posts: &[Post<'_>]) -> Ended {
    let clock = Clock::new();
    let mut binary = super::serve(directory);
    let streams = binary.take_streams().expect("pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes") };
    let within = Duration::from_secs(60);
    let mut statuses = Vec::new();
    let mut world = real::World::new(Judge {
        listener,
        posts,
        statuses: &mut statuses,
        sent: false,
        settled: false,
        deadline: clock.now().now.saturating_add(within),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Stdout(Box::new(StdoutReader::new(input, output))));
    let outcome = world.run(&clock, within);
    assert!(outcome.procs.iter().all(Host::is_empty), "all io settles");
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("binary") };
    Ended { statuses, exit: binary.exit_status(), stderr: binary.stderr().to_vec() }
}

/// One delivery through the complete startup and shutdown of the listener.
#[must_use]
pub fn delivered(scratch: &super::Scratch, post: Post<'_>) -> Ended {
    let listener = configuration(scratch.path());
    served(scratch.path(), listener, &[post])
}

/// One complete delivery, independent of the service's decoder.
pub struct Delivery {
    pub event: String,
    pub signature: String,
    pub body: Vec<u8>,
}
/// Receives one request within the deadline and acknowledges its complete body.
#[must_use]
pub fn receive(listener: &TcpListener, within: Duration) -> Delivery {
    listener.set_nonblocking(true).expect("nonblocking listener");
    let clock = Clock::new();
    let deadline = clock.now().now.saturating_add(within);
    let (stream, _) = loop {
        match listener.accept() {
            Ok(connection) => break connection,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    skein_shell::wait_readable(
                        skein_io::kernel::Fd::new(listener.as_raw_fd()),
                        deadline.saturating_since(clock.now().now),
                    )
                    .expect("webhook listener readiness"),
                    "no webhook arrived within its bound"
                );
            }
            Err(error) => panic!("listener accept: {error}"),
        }
    };
    let remaining = std::time::Duration::from_nanos(deadline.saturating_since(clock.now().now).as_nanos());
    stream.set_read_timeout(Some(remaining)).expect("request deadline");
    stream.set_write_timeout(Some(remaining)).expect("answer deadline");
    let mut reader = BufReader::new(stream);
    let mut delivery = Delivery { event: String::new(), signature: String::new(), body: Vec::new() };
    let mut length = None;
    loop {
        let mut line = String::new();
        assert_ne!(reader.read_line(&mut line).expect("request head"), 0, "head is complete");
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            match name.to_ascii_lowercase().as_str() {
                "x-forgejo-event" => delivery.event = value.trim().into(),
                "x-forgejo-signature" => delivery.signature = value.trim().into(),
                "content-length" => length = Some(value.trim().parse::<usize>().expect("body length")),
                _ => {}
            }
        }
    }
    delivery.body.resize(length.expect("Content-Length"), 0);
    reader.read_exact(&mut delivery.body).expect("complete delivery body");
    reader
        .get_mut()
        .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
        .expect("answer");
    delivery
}
