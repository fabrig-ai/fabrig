//! The shipped bundle over the real loop (testing-strategy.md, 2.9 and 2.10).
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

pub const PAGE: &[u8] = b"<!doctype html><title>t</title>";
pub const POLICY: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
pub fn fixture_bundle(directory: &Path) -> PathBuf {
    let bundle = directory.join("browser");
    let digest = bundle.join("0123456789abcdef");
    fs::create_dir_all(digest.join("files")).expect("bundle directory");
    fs::write(bundle.join("index.html"), PAGE).expect("bundle page");
    fs::write(digest.join("boot.js"), b"boot();").expect("bundle script");
    fs::write(digest.join("files/robots.txt"), b"User-agent: *\nDisallow:\n").expect("bundle root file");
    bundle
}
pub fn configuration(directory: &Path, bundle: &Path) {
    super::support::configuration(directory);
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
    value["workers"] = serde_json::json!([]);
    value["application"]["workers"] = serde_json::json!([]);
    value["service"]["components"] = serde_json::json!(["clients"]);
    value["engine"]["hosts"]["listener"] = serde_json::json!([]);
    value["application"]["server"]["bundle"] = bundle.to_str().expect("bundle path").into();
    fs::write(path, serde_json::to_vec(&value).expect("browser JSON")).expect("browser config");
}
pub fn field<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value.as_str())
}

use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Delegate, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Streams},
    real,
};

const OP: Token = Token::new(1);
const CLOSE: Token = Token::new(2);
const LINK_BYTES: usize = 4096;
const RESPONSE_BYTES: usize = 131_072; // bounded head and 64 KiB body
const PATIENCE: Duration = Duration::from_secs(20);

struct Reader {
    output: Option<Fd>,
    socket: Option<Fd>,
    address: Option<std::net::SocketAddr>,
    link: Vec<u8>,
    response: Vec<u8>,
    finished: bool,
    closing: u32,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Reader {
    fn new(input: Fd, output: Fd) -> Self {
        let mut reader = Self {
            output: Some(output),
            socket: None,
            address: None,
            link: Vec::new(),
            response: Vec::new(),
            finished: false,
            closing: 0,
            completions: Queue::uncapped(),
            submissions: Queue::uncapped(),
        };
        reader.close(input);
        reader.submit(Op::PipeRead { fd: output, buf: vec![0; LINK_BYTES].into_boxed_slice() });
        reader
    }
    fn submit(&mut self, kind: Op) {
        self.submissions.push(Submit { op: OP, kind });
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submissions.push(Submit { op: CLOSE, kind: Op::Close { fd } });
    }
    fn receive(&mut self, fd: Fd) {
        self.submit(Op::Recv { fd, buf: vec![0; LINK_BYTES].into_boxed_slice() });
    }
    fn completed(&mut self, complete: Complete) {
        let done = complete.result.expect("bundle exchange completes");
        if complete.op == CLOSE {
            self.closing -= 1;
            return;
        }
        match (complete.kind, done) {
            (Op::PipeRead { fd, buf }, Done::Count(count)) => {
                assert!(count > 0, "jig-test prints its link before closing stdout");
                self.link.extend_from_slice(&buf[..usize::try_from(count).expect("read count")]);
                assert!(self.link.len() <= LINK_BYTES, "bounded printed link");
                if let Some(link) = crate::support::printed_link(&self.link) {
                    let url = std::str::from_utf8(link).expect("printed URL");
                    let host = url.strip_prefix("http://").expect("loopback URL").split('/').next().expect("host");
                    let address: std::net::SocketAddr = host.parse().expect("literal address");
                    self.address = Some(address);
                    self.output = None;
                    self.close(fd);
                    self.submit(Op::Socket { family: skein_io::kernel::Family::of(&address) });
                } else {
                    self.submit(Op::PipeRead { fd, buf });
                }
            }
            (Op::Socket { .. }, Done::Fd(fd)) => {
                self.socket = Some(fd);
                self.submit(Op::Connect { fd, addr: self.address.expect("printed address") });
            }
            (Op::Connect { fd, .. }, Done::Nothing) => {
                let host = self.address.expect("printed address");
                let bytes = format!("GET /tasks/7 HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
                    .into_bytes()
                    .into_boxed_slice();
                self.submit(Op::Send { fd, bytes, from: 0 });
            }
            (Op::Send { fd, bytes, from }, Done::Count(count)) => {
                let from = from + count;
                if usize::try_from(from).expect("sent bytes") == bytes.len() {
                    self.receive(fd);
                } else {
                    self.submit(Op::Send { fd, bytes, from });
                }
            }
            (Op::Recv { fd, buf }, Done::Count(count)) => {
                self.response.extend_from_slice(&buf[..usize::try_from(count).expect("received count")]);
                assert!(self.response.len() <= RESPONSE_BYTES, "bounded HTTP response");
                if count == 0 {
                    self.finished = true;
                    self.socket = None;
                    self.close(fd);
                } else {
                    self.receive(fd);
                }
            }
            (kind, done) => panic!("unexpected bundle completion: {kind:?}, {done:?}"),
        }
    }
}
impl Host for Reader {
    fn iterate(&mut self, _: Time, _: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.completed(complete);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        !self.completions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.output.is_none()
            && self.socket.is_none()
            && self.closing == 0
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        4
    } // stdout/socket operation and at most three closes
}
enum Process {
    Binary(Box<Binary>),
    Reader(Box<Reader>),
}
impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(binary) => binary.as_ref(),
            Self::Reader(reader) => reader.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(binary) => binary.as_mut(),
            Self::Reader(reader) => reader.as_mut(),
        }
    }
}
struct Judge {
    stopped: bool,
    passed: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Process]) {
        let [Process::Binary(binary), Process::Reader(reader)] = procs else { panic!("bundle world") };
        if reader.finished && !self.stopped {
            binary.signal(Signal::Terminate);
            self.stopped = true;
        }
        if let Some(exit) = binary.exit_status() {
            assert!(
                reader.finished,
                "jig-test exited before the page: {:?}: {}",
                exit,
                String::from_utf8_lossy(binary.stderr())
            );
            assert_eq!(exit, Exit::Code(0), "bundle server exits cleanly");
            self.passed = true;
        }
    }
    fn observe(&mut self, _: Time, _: &[Process]) {}
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn overdue(&self, _: Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.passed
    }
}

pub fn page(directory: &Path) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let clock = Clock::new();
    let mut binary = crate::support::start(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let mut world = real::World::new(Judge { stopped: false, passed: false });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Reader(Box::new(Reader::new(input, output))));
    // The real loop's Deadline classifies an unanswered startup, socket
    // or exit wait under the shared starvation rule before cleanup.
    let outcome = world.run(&clock, PATIENCE);
    assert!(outcome.procs.iter().all(Host::is_empty), "bundle world settles");
    let Process::Reader(reader) = &outcome.procs[1] else { panic!("HTTP reader") };
    response(&reader.response)
}
fn response(bytes: &[u8]) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut reader = BufReader::new(bytes);
    let mut line = String::new();
    reader.read_line(&mut line).expect("HTTP status");
    let status = line.split_whitespace().nth(1).expect("status").parse().expect("status number");
    let mut headers = Vec::new();
    loop {
        line.clear();
        assert!(reader.read_line(&mut line).expect("HTTP field") > 0, "head complete");
        if line == "\r\n" {
            break;
        }
        assert!(headers.len() < 32 && line.len() <= LINK_BYTES, "bounded head");
        let (name, value) = line.trim_end().split_once(':').expect("field");
        headers.push((name.into(), value.trim().into()));
    }
    let length = field(&headers, "content-length").expect("body length").parse::<usize>().expect("length number");
    assert!(length <= 65536, "bounded body");
    let mut body = vec![0; length];
    reader.read_exact(&mut body).expect("HTTP body");
    (status, headers, body)
}
