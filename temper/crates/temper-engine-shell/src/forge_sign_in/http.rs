//! Bounded, non-redirecting source probes (temper/docs/design/protocol/openid-provider.md, 9).
//! The box's version barrier is private and reads only a status; these probes
//! also need Location and the public issuer's TLS, on skein's HTTP client.
use skein_http::client as http;
use skein_io::{
    Io,
    kernel::{Complete, Submit},
};
use skein_lib::{
    Queue, Time, Token,
    stream::{Down, Up},
};
use skein_shell::{Clock, Config, Kernel, Wait};
use skein_tls::client as tls;
use std::net::ToSocketAddrs;

const HTTP: http::Limits = http::Limits { request: 16_384, head: 16_384, headers: 64, read: 16_384, send: 16_384 };
const TLS: tls::Limits = tls::Limits { read: 16_384, send: 16_384, records: 131_072 };
const OWNER: Token = Token::new(1);

/// Reads the response head once, never following Location or reading its body.
pub(super) fn get(url: &[u8], until: Time) -> Result<http::Response, String> {
    let text = std::str::from_utf8(url).map_err(|why| why.to_string())?;
    let (secure, rest) = if let Some(rest) = text.strip_prefix("https://") {
        (true, rest)
    } else {
        (false, text.strip_prefix("http://").ok_or("HTTP origin required")?)
    };
    let (authority, path) = rest.split_once('/').ok_or("HTTP path required")?;
    let port = if secure { 443 } else { 80 };
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest.split_once(']').ok_or("IPv6 host required")?;
        let port = match suffix.strip_prefix(':') {
            Some(port) => port.parse::<u16>().map_err(|why| why.to_string())?,
            None if suffix.is_empty() => port,
            None => return Err("invalid port".into()),
        };
        (host, port)
    } else if let Some((host, port)) = authority.rsplit_once(':') {
        (host, port.parse::<u16>().map_err(|why| why.to_string())?)
    } else {
        (authority, port)
    };
    let address = (host, port).to_socket_addrs().map_err(|why| why.to_string())?.next().ok_or("no address")?;
    let transport = if secure {
        let trust = crate::serve::trust(None)?;
        let name = skein_tls::Name::new(host).ok_or("invalid TLS name")?;
        Some(tls::Client::new(&trust, name, &TLS))
    } else {
        None
    };
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).io;
    let operations = skein_io::operations(&limits).expect("usable io limits");
    let kernel = Kernel::open(Config { operations }).map_err(|why| format!("HTTP kernel: {why:?}"))?;
    let mut connection = Connection {
        io: Io::new(&limits),
        limits,
        kernel,
        clock: Clock::new(),
        socket: None,
        transport,
        http: http::Client::new(&HTTP),
        http_up: Queue::with_capacity(8),
        tls_up: Queue::with_capacity(8),
        plain: Queue::with_capacity(8),
        cipher: Queue::with_capacity(8),
        events: Queue::uncapped(),
        complete: Queue::with_capacity(operations),
        subs: Queue::with_capacity(operations),
        call: Some(http::Call {
            method: skein_http::Method::Get,
            target: format!("/{path}").into_bytes().into_boxed_slice(),
            headers: Box::new([skein_http::Header {
                name: b"Host".as_slice().into(),
                value: authority.as_bytes().into(),
            }]),
            body: http::Body::None,
            close: true,
        }),
        result: None,
    };
    if connection.clock.now().now >= until {
        return Err("source deadline expired".into());
    }
    connection.io_down(skein_io::Request::Connect { owner: OWNER, addr: skein_io::Addr::Ip(address) });
    let result = connection.answer(until);
    connection.close();
    result
}

struct Connection {
    io: Io,
    limits: skein_io::Limits,
    kernel: Kernel,
    clock: Clock,
    socket: Option<Token>,
    transport: Option<tls::Client>,
    http: http::Client,
    http_up: Queue<http::Event>,
    tls_up: Queue<tls::Event>,
    plain: Queue<Down>,
    cipher: Queue<Down>,
    events: Queue<skein_io::Event>,
    complete: Queue<Complete>,
    subs: Queue<Submit>,
    call: Option<http::Call>,
    result: Option<Result<http::Response, String>>,
}
impl Connection {
    fn io_down(&mut self, request: skein_io::Request) {
        skein_io::down(&mut self.io, &self.clock.now().with(self.limits), request, &mut self.subs);
    }
    fn call(&mut self) {
        http::down(
            &mut self.http,
            &self.clock.now().with(HTTP),
            http::Request::Call(self.call.take().expect("one GET")),
            &mut self.http_up,
            &mut self.plain,
        );
    }
    fn answer(&mut self, until: Time) -> Result<http::Response, String> {
        loop {
            self.poll_io(false);
            while self.result.is_none() && self.pump() {}
            self.io.reclaim();
            if let Some(result) = self.result.take() {
                return result;
            }
            if self.clock.now().now >= until {
                return Err("source deadline expired".into());
            }
            self.wait(Some(until));
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
                    skein_io::Event::Connected { .. } if !closing => {
                        if let Some(transport) = &mut self.transport {
                            tls::down(
                                transport,
                                &now.with(TLS),
                                tls::Request::Handshake,
                                &mut self.tls_up,
                                &mut self.cipher,
                            );
                        } else {
                            self.call();
                        }
                    }
                    skein_io::Event::Stream { up, .. } if !closing => {
                        if let Some(transport) = &mut self.transport {
                            tls::up(transport, &now.with(TLS), up, &mut self.tls_up, &mut self.cipher);
                        } else {
                            http::up(&mut self.http, &now.with(HTTP), up, &mut self.http_up, &mut self.plain);
                        }
                    }
                    skein_io::Event::Closed { .. } => {
                        self.socket = None;
                        if !closing && self.result.is_none() {
                            self.result = Some(Err("HTTP connection closed".into()));
                        }
                    }
                    skein_io::Event::Failed { error, .. } if !closing => {
                        self.result = Some(Err(format!("HTTP connection: {error:?}")));
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
                    | skein_io::Event::Shutdown { .. } => unreachable!("HTTP socket only"),
                }
            }
        }
    }
    fn pump(&mut self) -> bool {
        let now = self.clock.now();
        if let Some(event) = self.http_up.pop() {
            match event {
                http::Event::Response(response) => self.result = Some(Ok(response)),
                http::Event::Failed(why) => self.result = Some(Err(format!("HTTP: {why:?}"))),
                http::Event::Body(Up::Failed(why)) => self.result = Some(Err(format!("HTTP body: {why:?}"))),
                http::Event::Body(Up::End) | http::Event::Done(_) | http::Event::Closed => {}
                http::Event::Upload(_) | http::Event::Body(Up::Bytes(_) | Up::Room) => unreachable!("GET head only"),
            }
        } else if let Some(event) = self.tls_up.pop() {
            match event {
                tls::Event::Ready(_) => self.call(),
                tls::Event::Stream(up) => {
                    http::up(&mut self.http, &now.with(HTTP), up, &mut self.http_up, &mut self.plain);
                }
                tls::Event::Failed(why) => self.result = Some(Err(format!("TLS: {why:?}"))),
                tls::Event::Closed => self.result = Some(Err("TLS closed".into())),
            }
        } else if let Some(down) = self.cipher.pop() {
            self.io_down(skein_io::Request::Stream { stream: self.socket.expect("TLS socket"), down });
        } else if let Some(down) = self.plain.pop() {
            if let Some(transport) = &mut self.transport {
                tls::down(transport, &now.with(TLS), tls::Request::Stream(down), &mut self.tls_up, &mut self.cipher);
            } else {
                self.io_down(skein_io::Request::Stream { stream: self.socket.expect("HTTP socket"), down });
            }
        } else {
            return false;
        }
        true
    }
    fn wait(&mut self, until: Option<Time>) {
        let due = match (until, self.io.next_deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, None) | (None, a) => a,
        };
        let wait = if self.io.is_ready() || !self.complete.is_empty() {
            Wait::No
        } else {
            due.map_or(Wait::Forever, Wait::Until)
        };
        self.kernel.submit(&mut self.subs, wait);
    }
    fn close(&mut self) {
        if let Some(socket) = self.socket {
            self.io_down(skein_io::Request::Abort { entity: socket });
        }
        loop {
            self.poll_io(true);
            self.io.reclaim();
            if self.io.is_empty() && self.subs.is_empty() && self.kernel.in_flight() == 0 {
                break;
            }
            self.wait(None);
        }
    }
}
