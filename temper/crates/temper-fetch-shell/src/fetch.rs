//! The pinned fetch's host: one TLS connection and one body chunk at a time.
use skein_http::client as http;
use skein_io::{
    Io,
    kernel::{Complete, Submit},
};
use skein_lib::{
    Queue, Token,
    stream::{Down, Read, Up},
};
use skein_shell::{Clock, Config, Kernel, Trust, Wait};
use skein_tls::client as tls;
use std::fs;
use std::io;
use std::net::ToSocketAddrs;
use std::os::unix::{
    ffi::OsStrExt,
    fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
};
use std::path::Path;
use temper_fetch::{
    Origin,
    fetch::{self, Event, Failure, Fetch, Request},
};

mod cache;
use cache::{Download, NativeCache, WireRequest};

#[cfg(test)]
mod tests;

const CHUNK: u32 = 16_384;
const HTTP: http::Limits = http::Limits { request: 16_384, head: 16_384, headers: 64, read: CHUNK, send: CHUNK };
const TLS: tls::Limits = tls::Limits { read: 16_384, send: 16_384, records: 131_072 };
const OWNER: Token = Token::new(1);
/// A machine refusal, a file error, or a transport failure naming the whole URL.
#[derive(Debug)]
pub enum Failed {
    Refused(Failure),
    File { file: Box<[u8]>, why: io::ErrorKind },
    Roots(skein_shell::RootsRefusal),
    Trust(skein_tls::Refusal),
    Transport { url: Box<[u8]>, why: String },
}
impl Failed {
    /// Diagnostic words following the command's forge prefix.
    #[must_use]
    pub fn words(&self) -> String {
        match self {
            Self::Refused(why) => String::from_utf8_lossy(&why.words()).into_owned(),
            Self::File { file, why } => format!("{}: {why}", String::from_utf8_lossy(file)),
            Self::Roots(why) => match why {
                skein_shell::RootsRefusal::Unread { path, why } => format!("{}: roots: {why:?}", path.display()),
                other @ (skein_shell::RootsRefusal::NoBundle
                | skein_shell::RootsRefusal::TooMany { .. }
                | skein_shell::RootsRefusal::TooLong { .. }) => format!("roots: {other:?}"),
            },
            Self::Trust(why) => format!("roots: {why:?}"),
            Self::Transport { url, why } => format!("{}: {why}", String::from_utf8_lossy(url)),
        }
    }
}
fn file_error(file: &[u8], why: &io::Error) -> Failed {
    Failed::File { file: file.into(), why: why.kind() }
}

/// Finds Claude Code by size, serializing a missing build's fetch (protocol/setup/owned.md, 6).
pub fn claude_code(
    limits: &skein_io::Limits,
    trust: &Trust,
    file: &Path,
    pin: fetch::Pin,
    bytes: u64,
    out: &mut dyn io::Write,
) -> Result<(), Failed> {
    claude_code_with(
        file,
        bytes,
        |file| fs::metadata(file).map(|metadata| (metadata.is_file(), metadata.len())),
        prepare_claude_cache,
        lock_claude_cache,
        |out| run(limits, trust, Fetch::new(Box::new([pin]), bytes), out),
        out,
    )
}

fn claude_code_with<L>(
    file: &Path,
    bytes: u64,
    mut metadata: impl FnMut(&Path) -> io::Result<(bool, u64)>,
    prepare: impl FnOnce(&Path) -> Result<(), Failed>,
    lock: impl FnOnce(&Path) -> Result<L, Failed>,
    download: impl FnOnce(&mut dyn io::Write) -> Result<(), Failed>,
    out: &mut dyn io::Write,
) -> Result<(), Failed> {
    if claude_present(file.as_os_str().as_bytes(), bytes, metadata(file))? {
        return Ok(());
    }
    prepare(file)?;
    let _held = lock(file)?;
    if claude_present(file.as_os_str().as_bytes(), bytes, metadata(file))? {
        return Ok(());
    }
    match out.write_all(b"fetching Claude Code\n") {
        Ok(()) | Err(_) => {}
    }
    download(out)
}

fn prepare_claude_cache(file: &Path) -> Result<(), Failed> {
    let name = file.as_os_str().as_bytes();
    let parent = file.parent().expect("cache path has a parent");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(parent).map_err(|why| file_error(name, &why))?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|why| file_error(name, &why))
}

fn lock_claude_cache(file: &Path) -> Result<fs::File, Failed> {
    let mut lock_name = file.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&lock_name)
        .map_err(|why| file_error(lock_name.as_bytes(), &why))?;
    lock.lock().map_err(|why| file_error(lock_name.as_bytes(), &why))?;
    Ok(lock)
}

fn claude_present(file: &[u8], bytes: u64, metadata: io::Result<(bool, u64)>) -> Result<bool, Failed> {
    match metadata {
        Ok((true, length)) if length == bytes => Ok(true),
        Ok(_) => Err(Failed::Refused(Failure::Present { file: file.into() })),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(why) => Err(file_error(file, &why)),
    }
}

fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}

/// Runs the machine; an interrupted body leaves its part to be truncated next time.
pub fn run(limits: &skein_io::Limits, trust: &Trust, machine: Fetch, out: &mut dyn io::Write) -> Result<(), Failed> {
    let mut files = Download::new(NativeCache);
    let mut wire: Option<Connection> = None;
    let result = run_with(
        machine,
        || tls_config(trust),
        |config, request| {
            files.answer(request, |request| match request {
                WireRequest::Get { origin, path } => {
                    if let Some(mut old) = wire.take() {
                        old.close();
                    }
                    wire = Some(Connection::new(limits, config, &origin, &path)?);
                    wire.as_mut().expect("new connection").answer()
                }
                WireRequest::Read => {
                    let connection = wire.as_mut().expect("body follows Get");
                    connection.demand();
                    connection.answer()
                }
            })
        },
        out,
    );
    if let Some(mut connection) = wire {
        connection.close();
    }
    files.close_part();
    result
}

fn tls_config(trust: &Trust) -> Result<skein_tls::Config, Failed> {
    let roots = skein_shell::read_trust_roots(
        trust,
        &skein_shell::RootBounds { bundle_bytes: 1 << 20, certificates: 1024, certificate_bytes: 65_536 },
    )
    .map_err(Failed::Roots)?;
    let (config, _) = skein_tls::Config::from_der(&roots.certificates, &[b"http/1.1"]).map_err(Failed::Trust)?;
    Ok(config)
}

/// Drives fetch over supplied roots and answers to its cache and download requests.
pub fn run_with<R>(
    mut machine: Fetch,
    roots: impl FnOnce() -> Result<R, Failed>,
    mut answer: impl FnMut(&R, Request) -> Result<Event, Failed>,
    out: &mut dyn io::Write,
) -> Result<(), Failed> {
    let roots = roots()?;
    let mut event = Event::Begin;
    let mut requests = Queue::uncapped();
    loop {
        fetch::step(&mut machine, event, &mut requests);
        assert_eq!(requests.len(), 1, "one fetch request per event");
        event = match requests.pop().expect("one fetch request") {
            Request::Say(fetched) => {
                match out.write_all(&fetched.line()) {
                    Ok(()) | Err(_) => {}
                }
                Event::Done
            }
            Request::End(result) => return result.map_err(Failed::Refused),
            request @ (Request::Hash { .. }
            | Request::Get { .. }
            | Request::Open { .. }
            | Request::Read
            | Request::Write { .. }
            | Request::Unpack { .. }
            | Request::Finish { .. }
            | Request::Remove { .. }) => answer(&roots, request)?,
        };
    }
}

fn request_head(origin: &Origin, path: &[u8]) -> http::Call {
    let mut authority = origin.host.to_vec();
    if origin.port != 443 {
        authority.extend_from_slice(format!(":{}", origin.port).as_bytes());
    }
    let headers = Box::new([
        skein_http::Header { name: b"Host".as_slice().into(), value: authority.into_boxed_slice() },
        skein_http::Header { name: b"User-Agent".as_slice().into(), value: b"temper".as_slice().into() },
    ]);
    http::Call { method: skein_http::Method::Get, target: path.into(), headers, body: http::Body::None, close: true }
}

fn body_demand() -> http::Request {
    http::Request::Body(Down::Demand { read: Read::Arrived { max: CHUNK }, room: 0 })
}

fn http_answer(event: http::Event) -> Option<Result<Event, String>> {
    match event {
        http::Event::Response(response) => Some(Ok(Event::Answered {
            status: response.status,
            location: response.header(b"location").map(Into::into),
            length: match response.framing {
                http::Framing::Length(n) => Some(n),
                http::Framing::Empty => Some(0),
                http::Framing::Chunked | http::Framing::UntilEnd => None,
            },
        })),
        http::Event::Body(Up::Bytes(bytes)) => Some(Ok(Event::Chunk(bytes))),
        http::Event::Done(_) => Some(Ok(Event::Ended)),
        http::Event::Body(Up::End) | http::Event::Closed => None,
        http::Event::Failed(why) => Some(Err(format!("HTTP: {why:?}"))),
        http::Event::Body(Up::Failed(why)) => Some(Err(format!("HTTP body: {why:?}"))),
        http::Event::Upload(_) | http::Event::Body(Up::Room) => unreachable!("GET has no upload"),
    }
}

pub(crate) struct Connection {
    io: Io,
    limits: skein_io::Limits,
    kernel: Kernel,
    clock: Clock,
    socket: Option<Token>,
    closed: bool,
    tls: tls::Client,
    http: http::Client,
    tls_up: Queue<tls::Event>,
    cipher: Queue<Down>,
    http_up: Queue<http::Event>,
    plain: Queue<Down>,
    events: Queue<skein_io::Event>,
    complete: Queue<Complete>,
    subs: Queue<Submit>,
    call: Option<http::Call>,
    result: Option<Result<Event, String>>,
    url: Box<[u8]>,
}
impl Connection {
    pub(crate) fn new(
        limits: &skein_io::Limits,
        config: &skein_tls::Config,
        origin: &Origin,
        path: &[u8],
    ) -> Result<Self, Failed> {
        let url = fetch::url(origin, path);
        let fail = |why| Failed::Transport { url: url.clone(), why };
        if limits.intake < tls::LARGEST_READ || limits.output < tls::largest_room(&TLS) {
            return Err(fail("io capacity below TLS's largest demand".into()));
        }
        let host = std::str::from_utf8(&origin.host).map_err(|why| fail(why.to_string()))?;
        let host = host.strip_prefix('[').and_then(|host| host.strip_suffix(']')).unwrap_or(host);
        let name = skein_tls::Name::new(host).ok_or_else(|| fail("invalid TLS server name".into()))?;
        let address = (host, origin.port)
            .to_socket_addrs()
            .map_err(|why| fail(format!("connect: {why}")))?
            .next()
            .ok_or_else(|| fail("connect: no address".into()))?;
        let operations = skein_io::operations(limits).expect("usable io limits");
        let kernel = Kernel::open(Config { operations }).map_err(|why| fail(format!("connect kernel: {why:?}")))?;
        let mut connection = Self {
            io: Io::new(limits),
            limits: *limits,
            kernel,
            clock: Clock::new(),
            socket: None,
            closed: false,
            tls: tls::Client::new(config, name, &TLS),
            http: http::Client::new(&HTTP),
            tls_up: Queue::with_capacity(8),
            cipher: Queue::with_capacity(8),
            http_up: Queue::with_capacity(8),
            plain: Queue::with_capacity(8),
            events: Queue::uncapped(),
            complete: Queue::with_capacity(operations),
            subs: Queue::with_capacity(operations),
            call: Some(request_head(origin, path)),
            result: None,
            url,
        };
        connection.io_down(skein_io::Request::Connect { owner: OWNER, addr: skein_io::Addr::Ip(address) });
        Ok(connection)
    }
    fn io_down(&mut self, request: skein_io::Request) {
        let now = self.clock.now();
        skein_io::down(&mut self.io, &now.with(self.limits), request, &mut self.subs);
    }
    pub(crate) fn demand(&mut self) {
        let now = self.clock.now();
        http::down(&mut self.http, &now.with(HTTP), body_demand(), &mut self.http_up, &mut self.plain);
    }
    pub(crate) fn answer(&mut self) -> Result<Event, Failed> {
        loop {
            self.poll_io(false);
            while self.result.is_none() && self.pump() {}
            self.io.reclaim();
            if let Some(result) = self.result.take() {
                if matches!(result, Ok(Event::Ended)) {
                    self.close();
                }
                return result.map_err(|why| Failed::Transport { url: self.url.clone(), why });
            }
            self.wait();
        }
    }
    fn poll_io(&mut self, closing: bool) {
        self.kernel.reap(&mut self.complete);
        let now = self.clock.now();
        let env = now.with(self.limits);
        loop {
            if self.io.is_ready() {
                skein_io::resume(&mut self.io, &env, &mut self.events, &mut self.subs);
            } else if let Some(complete) = self.complete.pop() {
                skein_io::up(&mut self.io, &env, complete, &mut self.events, &mut self.subs);
            } else if self.io.is_due(now.now) {
                skein_io::fire(&mut self.io, &env, &mut self.events, &mut self.subs);
            } else {
                break;
            }
            while let Some(event) = self.events.pop() {
                match event {
                    skein_io::Event::Connecting { socket, .. } => self.socket = Some(socket),
                    skein_io::Event::Connected { .. } if !closing => tls::down(
                        &mut self.tls,
                        &now.with(TLS),
                        tls::Request::Handshake,
                        &mut self.tls_up,
                        &mut self.cipher,
                    ),
                    skein_io::Event::Stream { up, .. } if !closing => {
                        tls::up(&mut self.tls, &now.with(TLS), up, &mut self.tls_up, &mut self.cipher);
                    }
                    skein_io::Event::Closed { .. } => {
                        self.closed = true;
                        self.socket = None;
                        if !closing && self.result.is_none() {
                            self.result = Some(Err("connection closed before the HTTP answer".into()));
                        }
                    }
                    skein_io::Event::Failed { error: why, .. } if !closing => {
                        self.result = Some(Err(format!("connect: {why:?}")));
                    }
                    skein_io::Event::Connected { .. }
                    | skein_io::Event::Stream { .. }
                    | skein_io::Event::Failed { .. } => {}
                    skein_io::Event::Listening { .. }
                    | skein_io::Event::Accepted { .. }
                    | skein_io::Event::Output { .. }
                    | skein_io::Event::Spawned { .. }
                    | skein_io::Event::Exited { .. }
                    | skein_io::Event::Usage { .. }
                    | skein_io::Event::Resized
                    | skein_io::Event::Window { .. }
                    | skein_io::Event::Shutdown { .. } => unreachable!("socket operations only"),
                }
            }
        }
    }
    fn pump(&mut self) -> bool {
        let now = self.clock.now();
        let http_env = now.with(HTTP);
        let tls_env = now.with(TLS);
        if let Some(event) = self.http_up.pop() {
            self.result = http_answer(event);
        } else if let Some(event) = self.tls_up.pop() {
            match event {
                tls::Event::Ready(_) => {
                    let call = self.call.take().expect("one HTTP call");
                    http::down(
                        &mut self.http,
                        &http_env,
                        http::Request::Call(call),
                        &mut self.http_up,
                        &mut self.plain,
                    );
                }
                tls::Event::Stream(up) => http::up(&mut self.http, &http_env, up, &mut self.http_up, &mut self.plain),
                tls::Event::Failed(why) => self.result = Some(Err(format!("TLS: {why:?}"))),
                tls::Event::Closed => self.result = Some(Err("TLS closed before the HTTP answer".into())),
            }
        } else if let Some(down) = self.cipher.pop() {
            self.io_down(skein_io::Request::Stream { stream: self.socket.expect("connected socket"), down });
        } else if let Some(down) = self.plain.pop() {
            tls::down(&mut self.tls, &tls_env, tls::Request::Stream(down), &mut self.tls_up, &mut self.cipher);
        } else {
            return false;
        }
        true
    }
    fn wait(&mut self) {
        let wait = if self.io.is_ready() || !self.complete.is_empty() {
            Wait::No
        } else {
            self.io.next_deadline().map_or(Wait::Forever, Wait::Until)
        };
        self.kernel.submit(&mut self.subs, wait);
    }
    pub(crate) fn close(&mut self) {
        if let Some(socket) = self.socket {
            self.io_down(skein_io::Request::Abort { entity: socket });
        }
        loop {
            self.poll_io(true);
            self.io.reclaim();
            if self.io.is_empty() && self.subs.is_empty() && self.kernel.in_flight() == 0 {
                break;
            }
            self.wait();
        }
    }
}
