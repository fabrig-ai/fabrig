//! The ctl's kernel face keeps files in memory and carries HTTP bodies to the real clients component.
use jig_client::{Event, Message, Request};
use jig_client_kit::harness::ClientId;
use jig_client_protocol_link::Streamed;
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Kind, Op, OpenHow, Stat, Submit};
use skein_lib::{
    Duration, Secret, Time, Token, Wall, Writer,
    base64::{self, Alphabet},
};
use skein_shell::Host;
use std::collections::{BTreeMap, VecDeque};
use temper_client_shell_cli::{
    arguments,
    host::{Invocation, OneShot, Startup},
    limits,
};
use temper_client_world::{
    fixtures,
    front::{self, Joint},
};

struct File {
    bytes: Vec<u8>,
    mode: u32,
    kind: Kind,
}
#[derive(Default)]
struct Socket {
    request: Vec<u8>,
    reply: VecDeque<u8>,
    answered: bool,
    stream_opened: bool,
}
pub(super) struct Answer {
    pub(super) exit: Exit,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

pub(super) struct Machine {
    front: Joint,
    files: BTreeMap<String, File>,
    descriptors: BTreeMap<Fd, String>,
    sockets: BTreeMap<Fd, Socket>,
    exchanges: BTreeMap<Token, Fd>,
    next_exchange: u64,
    stream: Option<Fd>,
    next_fd: i32,
    serial: usize,
    now: Time,
    held: Vec<Submit>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdin: VecDeque<u8>,
    redirected: Option<(String, Vec<u8>)>,
}
impl Machine {
    pub(super) fn new() -> Self {
        Self::with_front(Joint::with_service_party(fixtures::SEED, fixtures::domain_limits()))
    }
    pub(super) fn findings() -> Self {
        Self::with_front(Joint::with_finding_services(fixtures::SEED, fixtures::domain_limits()))
    }
    fn with_front(front: Joint) -> Self {
        let mut files = BTreeMap::new();
        for name in ["", "owner", "agent", "docs"] {
            files.insert(name.to_owned(), File { bytes: Vec::new(), mode: 0o700, kind: Kind::Directory });
        }
        files.insert(
            "printed".to_owned(),
            File { bytes: front::printed_link().into_vec(), mode: 0o600, kind: Kind::File },
        );
        Self {
            front,
            files,
            descriptors: BTreeMap::new(),
            sockets: BTreeMap::new(),
            exchanges: BTreeMap::new(),
            next_exchange: 0,
            stream: None,
            next_fd: 100,
            serial: 0,
            now: Time::ZERO,
            held: Vec::new(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdin: VecDeque::new(),
            redirected: None,
        }
    }
    pub(super) fn private_file(&self, path: &str) {
        let file = self.files.get(path).expect("ctl wrote file");
        assert_eq!(file.mode, 0o600);
        assert_ne!(file.bytes.len(), 0);
    }
    pub(super) fn call(&mut self, directory: &str, words: &[&[u8]]) -> String {
        self.call_input(directory, words, b"")
    }
    pub(super) fn call_input(&mut self, directory: &str, words: &[&[u8]], input: &[u8]) -> String {
        let answer = self.run_input(directory, words, input);
        assert_eq!(answer.exit, Exit::Code(0), "ctl failed: {}", String::from_utf8_lossy(&answer.stderr));
        assert!(answer.stderr.is_empty(), "ctl stderr: {}", String::from_utf8_lossy(&answer.stderr));
        String::from_utf8(answer.stdout).expect("ctl UTF-8")
    }
    pub(super) fn run(&mut self, directory: &str, words: &[&[u8]]) -> Answer {
        self.run_input(directory, words, b"")
    }
    pub(super) fn run_input(&mut self, directory: &str, words: &[&[u8]], input: &[u8]) -> Answer {
        self.stdin = input.iter().copied().collect();
        self.serial += 1;
        let who = ClientId(self.serial);
        let native = limits::native();
        self.front.link_client(who, native.root.domain.client.domain);
        self.descriptors = BTreeMap::from([(Fd::new(1), String::new()), (Fd::new(6), String::new())]);
        self.sockets.clear();
        self.exchanges.clear();
        self.stream = None;
        self.stdout.clear();
        self.stderr.clear();
        let parsed = arguments::parse(words).expect("ctl argv");
        let file_parent = arguments::file_root(&parsed).map(|_| Fd::new(6));
        let mut host = OneShot::new(
            Invocation { directory: Some(directory.as_bytes().into()), parsed },
            Startup {
                parent: Some(Fd::new(1)),
                file_parent,
                stdin: Fd::new(2),
                stdout: Fd::new(4),
                stderr: Fd::new(3),
                signals: Fd::new(5),
                user: 1000,
                seed: fixtures::SEED,
            },
        )
        .expect("ctl limits");
        for _ in 0..2000 {
            host.iterate(self.now, Wall::EPOCH);
            let mut work = std::mem::take(&mut self.held);
            while let Some(submit) = host.submissions().pop() {
                work.push(submit);
            }
            for mut submit in work {
                if matches!(&submit.kind, Op::ReadSignal { .. })
                    || matches!(&submit.kind, Op::Recv { fd, .. } if self.sockets.get(fd).expect("socket").reply.is_empty())
                {
                    self.held.push(submit);
                    continue;
                }
                let result = self.answer(who, &mut host, &mut submit.kind);
                host.completions().push(Complete { op: submit.op, kind: submit.kind, result });
            }
            self.hear(who);
            if let Some(exit) = host.exit() {
                assert_eq!(self.held.len(), 0);
                return Answer {
                    exit,
                    stdout: std::mem::take(&mut self.stdout),
                    stderr: std::mem::take(&mut self.stderr),
                };
            }
            self.now = self.now.saturating_add(Duration::from_nanos(1));
        }
        panic!("ctl did not settle");
    }
    // Relay the simulated kernel's output writes to stdin without opening the output file.
    pub(super) fn redirect_file(&mut self, path: &str) {
        assert!(self.redirected.replace((path.to_owned(), Vec::new())).is_none());
    }
    pub(super) fn take_redirected(&mut self) -> Vec<u8> {
        self.redirected.take().expect("redirected output").1
    }
    pub(super) fn assert_service_link_private(&self, link: &[u8], output: &[&[u8]]) {
        let encoded = link
            .strip_prefix(b"http://127.0.0.1:8367/sign-in#service=")
            .and_then(|tail| tail.strip_suffix(b"\n"))
            .expect("one service link with a newline");
        let raw = base64::decode(Alphabet::Url, encoded, 40).expect("base64url service credential");
        assert_eq!(raw.len(), 40);
        let contains = |bytes: &[u8]| {
            bytes.windows(encoded.len()).any(|part| part == encoded)
                || bytes.windows(raw.len()).any(|part| part == raw.as_ref())
        };
        assert!(!output.iter().any(|bytes| contains(bytes)), "service credential leaked to ctl output");
        for (path, file) in &self.files {
            if path.starts_with("owner/") || path.starts_with("agent/") {
                assert!(!contains(&file.bytes), "service credential leaked to a DIRECTORY store");
            }
        }
    }
    fn allocate(&mut self) -> Fd {
        let fd = Fd::new(self.next_fd);
        self.next_fd += 1;
        fd
    }
    fn path(&self, root: Fd, leaf: &[u8]) -> String {
        let root = self.descriptors.get(&root).expect("directory fd");
        let leaf = std::str::from_utf8(leaf).expect("fixture path");
        if leaf == "." {
            root.clone()
        } else if root.is_empty() {
            leaf.to_owned()
        } else {
            format!("{root}/{leaf}")
        }
    }
    #[expect(clippy::too_many_lines, reason = "the small kernel face names each file and socket operation together")]
    fn answer(&mut self, who: ClientId, host: &mut OneShot, op: &mut Op) -> Result<Done, Error> {
        match op {
            Op::Open { root, path, how } => {
                let path = self.path(*root, path);
                if let OpenHow::Create { mode } | OpenHow::CreateNoFollow { mode } = how {
                    if self.files.contains_key(&path) {
                        return Err(Error::Exists);
                    }
                    self.files.insert(
                        path.clone(),
                        File { bytes: Vec::new(), mode: mode.unwrap_or(0o600), kind: Kind::File },
                    );
                }
                if !self.files.contains_key(&path) {
                    return Err(Error::NotFound);
                }
                let fd = self.allocate();
                self.descriptors.insert(fd, path);
                Ok(Done::Fd(fd))
            }
            Op::Stat { fd } => {
                let file = self.files.get(&self.descriptors[fd]).expect("file");
                Ok(Done::Stat(Stat {
                    kind: file.kind,
                    size: file.bytes.len() as u64,
                    mode: file.mode,
                    owner: 1000,
                    links: if file.kind == Kind::Directory { 2 } else { 1 },
                }))
            }
            Op::Read { fd, buf, at } => {
                let file = &self.files[&self.descriptors[fd]].bytes;
                let start = usize::try_from(*at).expect("file offset").min(file.len());
                let count = buf.len().min(file.len() - start);
                buf[..count].copy_from_slice(&file[start..start + count]);
                Ok(Done::Count(u32::try_from(count).expect("read count")))
            }
            Op::Write { fd, bytes, from, at } => {
                let bytes = &bytes[*from as usize..];
                let path = &self.descriptors[fd];
                let start = usize::try_from(*at).expect("write offset");
                let file = &mut self.files.get_mut(path).expect("written file").bytes;
                file.resize(start + bytes.len(), 0);
                file[start..].copy_from_slice(bytes);
                if let Some((target, redirected)) = &mut self.redirected
                    && target == path
                {
                    redirected.resize(start + bytes.len(), 0);
                    redirected[start..].copy_from_slice(bytes);
                }
                Ok(Done::Count(u32::try_from(bytes.len()).expect("write count")))
            }
            Op::Rename { from_dir, from, to_dir, to, no_replace } => {
                let from = self.path(*from_dir, from);
                let to = self.path(*to_dir, to);
                if *no_replace && self.files.contains_key(&to) {
                    return Err(Error::Exists);
                }
                let file = self.files.remove(&from).expect("temporary file");
                self.files.insert(to, file);
                Ok(Done::Nothing)
            }
            Op::Remove { dir, name, .. } => {
                let path = self.path(*dir, name);
                self.files.remove(&path).expect("removed file");
                Ok(Done::Nothing)
            }
            Op::Lock { .. } | Op::Sync { .. } | Op::Shutdown { .. } | Op::Close { .. } | Op::Connect { .. } => {
                Ok(Done::Nothing)
            }
            Op::Socket { .. } => {
                let fd = self.allocate();
                self.sockets.insert(fd, Socket::default());
                Ok(Done::Fd(fd))
            }
            Op::Send { fd, bytes, from } => {
                let bytes = &bytes[*from as usize..];
                self.sockets.get_mut(fd).expect("socket").request.extend_from_slice(bytes);
                self.http(who, *fd);
                Ok(Done::Count(u32::try_from(bytes.len()).expect("send count")))
            }
            Op::Recv { fd, buf } => {
                let reply = &mut self.sockets.get_mut(fd).expect("socket").reply;
                let count = buf.len().min(reply.len());
                for byte in &mut buf[..count] {
                    *byte = reply.pop_front().expect("reply byte");
                }
                Ok(Done::Count(u32::try_from(count).expect("receive count")))
            }
            Op::PipeRead { buf, .. } => {
                let count = buf.len().min(self.stdin.len());
                for byte in &mut buf[..count] {
                    *byte = self.stdin.pop_front().expect("stdin byte");
                }
                Ok(Done::Count(u32::try_from(count).expect("input count")))
            }
            Op::PipeWrite { fd, bytes, from } => {
                let bytes = &bytes[*from as usize..];
                if *fd == Fd::new(3) {
                    self.stderr.extend_from_slice(bytes);
                } else {
                    assert_eq!(*fd, Fd::new(4));
                    self.stdout.extend_from_slice(bytes);
                }
                Ok(Done::Count(u32::try_from(bytes.len()).expect("pipe count")))
            }
            Op::Cancel { target } => {
                if let Some(at) = self.held.iter().position(|submit| submit.op == *target) {
                    let submit = self.held.remove(at);
                    host.completions().push(Complete {
                        op: submit.op,
                        kind: submit.kind,
                        result: Err(Error::Cancelled),
                    });
                    Ok(Done::Nothing)
                } else {
                    Err(Error::NotFound)
                }
            }
            _ => panic!("operation outside the ctl's file and socket face"),
        }
    }
    fn http(&mut self, who: ClientId, fd: Fd) {
        let socket = self.sockets.get_mut(&fd).expect("HTTP socket");
        if socket.answered {
            return;
        }
        let Some(end) = socket.request.windows(4).position(|bytes| bytes == b"\r\n\r\n") else { return };
        let head = std::str::from_utf8(&socket.request[..end]).expect("HTTP head");
        let length = head
            .lines()
            .find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length: ").map(str::to_owned))
            .map_or(0, |length| length.parse::<usize>().expect("body length"));
        if socket.request.len() < end + 4 + length {
            return;
        }
        let path = head.split_whitespace().nth(1).expect("HTTP route");
        let (path, query) = path.split_once('?').unwrap_or((path, ""));
        let mut replaces = None;
        let mut version = None;
        for field in query.split('&') {
            if let Some(hex) = field.strip_prefix("replaces=") {
                let name: Vec<u8> = hex
                    .as_bytes()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16).expect("hex name"))
                    .collect();
                replaces = Some(<[u8; 16]>::try_from(name.as_slice()).expect("stream name"));
            } else if let Some(value) = field.strip_prefix("version=") {
                version = Some(value.parse::<u32>().expect("wire version"));
            } else {
                assert!(field.is_empty(), "query outside service-party story");
            }
        }
        let credential = head.lines().find_map(|line| {
            line.strip_prefix("Authorization: Bearer ")
                .map(|value| Secret::new(base64::decode(Alphabet::Url, value.as_bytes(), 40).expect("bearer")))
        });
        let body = socket.request[end + 4..].into();
        socket.answered = true;
        let message = match path {
            "/jig/v1/sign-in/link" => Message::SignInLink(body),
            "/jig/v1/sign-in/deployment" => Message::SignInService,
            "/jig/v1/request" => Message::Request(body),
            "/jig/v1/watch" => Message::Watch(body),
            "/jig/v1/read" => Message::Read(body),
            "/jig/v1/batch" => {
                socket.reply.extend(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");
                Message::Batch(body)
            }
            "/jig/v1/unwatch" => Message::Unwatch(body),
            "/jig/v1/stream-gone" => Message::StreamGone(body),
            "/jig/v1/stream" => {
                assert_eq!(version, Some(jig_client_protocol_wire::VERSION));
                self.stream = Some(fd);
                self.front.carry(
                    who,
                    self.now,
                    Request::OpenStream { credential, replaces, version: jig_client_protocol_wire::VERSION },
                );
                self.hear(who);
                return;
            }
            _ => panic!("route outside service-party story"),
        };
        self.next_exchange += 1;
        let exchange = Token::new(self.next_exchange);
        self.exchanges.insert(exchange, fd);
        self.front.carry(who, self.now, Request::Exchange { exchange, credential, message });
        self.hear(who);
    }
    fn hear(&mut self, who: ClientId) {
        for event in self.front.heard(who, self.now) {
            match event {
                Event::Reply { exchange, message } => {
                    let fd = self.exchanges.remove(&exchange).expect("exchange socket");
                    let reply = &mut self.sockets.get_mut(&fd).expect("reply socket").reply;
                    reply.extend(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", message.len()).bytes());
                    reply.extend(message.iter().copied());
                }
                Event::Part { exchange, message } => {
                    let fd = self.exchanges[&exchange];
                    let reply = &mut self.sockets.get_mut(&fd).expect("batch socket").reply;
                    let frame =
                        [u32::try_from(message.len()).expect("part size").to_be_bytes().as_slice(), message.as_ref()]
                            .concat();
                    reply.extend(format!("{:x}\r\n", frame.len()).bytes());
                    reply.extend(frame);
                    reply.extend(b"\r\n");
                }
                Event::Done { exchange } => {
                    let fd = self.exchanges.remove(&exchange).expect("batch socket");
                    self.sockets.get_mut(&fd).expect("batch socket").reply.extend(b"0\r\n\r\n");
                }
                Event::Streamed { message: Streamed::Record(bytes) } => {
                    let length =
                        base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("record size"))
                            .expect("encoded size");
                    let mut encoded = Writer::new(length as usize);
                    base64::encode(Alphabet::Standard, &bytes, &mut encoded).expect("stream encoding");
                    let socket = self.sockets.get_mut(&self.stream.expect("stream fd")).expect("stream socket");
                    if !socket.stream_opened {
                        socket.reply.extend(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n");
                        socket.stream_opened = true;
                    }
                    let reply = &mut socket.reply;
                    reply.extend(b"data: ");
                    reply.extend(encoded.finish().iter().copied());
                    reply.extend(b"\n\n");
                }
                Event::Streamed { message: Streamed::Heartbeat } => {}
                Event::Streamed { message: Streamed::SignedOut } => {
                    let socket = self.sockets.get_mut(&self.stream.expect("stream fd")).expect("stream socket");
                    assert!(!socket.stream_opened, "the next ctl call opens a fresh stream");
                    socket.reply.extend(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n");
                }
                _ => panic!("unexpected component event in service-party story"),
            }
        }
    }
}
