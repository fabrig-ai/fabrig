//! One list request on an endpoint (skein/docs/design/llm-connection.md, 4.3). Keeps one link
//! and one bounded body; it never knows who asked. `start`, `up` and `fire` drive it;
//! `answer` gives its one answer after its socket has settled.

use alloc::boxed::Box;
use core::{fmt, mem};
use skein_http::{Header, Method, client as http};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall, bytes, earliest};
use skein_link::{self as link, Link, Moved, Start};
use skein_llm::{Credential, Provider, list};
use skein_tls::client as tls;

use crate::{Deadlines, Endpoint, Lower, LowerEvent, Transport};

/// Bounds on the response and the protocol edges used by one exchange.
#[derive(Clone, Copy, Debug)]
pub struct ExchangeLimits {
    pub http: http::Limits,
    pub tls: tls::Limits,
    pub list: list::Limits,
    pub close_timeout: Duration,
    /// Wall-clock snapshot for TLS certificate validation, supplied by the owner.
    pub wall: Wall,
}

impl ExchangeLimits {
    pub const DEFAULT: Self = Self {
        http: http::Limits { request: 8192, head: 8192, headers: 32, read: 8192, send: 8192 },
        tls: tls::Limits { read: 8192, send: 8192, records: tls::MAX_RECORD },
        list: list::Limits::DEFAULT,
        close_timeout: Duration::from_secs(1),
        wall: Wall::EPOCH,
    };
}

/// One answer, carrying neither a credential nor a provider's refusal body.
#[derive(PartialEq, Eq, Debug)]
pub enum ListAnswer {
    Listed(Box<[list::Listed]>),
    Refused { status: u16 },
    Unreachable,
    Unreadable,
    Deadline,
}

enum State {
    New,
    Connecting { whole: Option<Time>, until: Option<Time> },
    Handshaking { whole: Option<Time>, until: Option<Time> },
    Reading { whole: Option<Time> },
    Closing { answer: ListAnswer },
    Closed { answer: Option<ListAnswer> },
}

/// One GET and one socket, owned until io reports its settlement.
pub struct Exchange {
    token: Token,
    address: skein_io::kernel::Addr,
    provider: Provider,
    deadlines: Deadlines,
    limits: ExchangeLimits,
    state: State,
    link: Link,
    http: http::Client,
    events: Queue<http::Event>,
    requests: Queue<http::Request>,
    plain: Queue<Down>,
    document: List<u8>,
    now: Time,
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "HTTP and its queues can hold a credential; show only the link and body length"
)]
impl fmt::Debug for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Exchange").field("link", &self.link).field("body_bytes", &self.document.len()).finish()
    }
}

impl Exchange {
    #[must_use]
    pub fn new(
        token: Token,
        endpoint: &Endpoint,
        provider: Provider,
        credential: Option<Credential>,
        deadlines: Deadlines,
        limits: &ExchangeLimits,
    ) -> Exchange {
        assert!(http::valid(&limits.http), "valid HTTP edges");
        assert!(tls::valid(&limits.tls), "valid TLS edges");
        let tls = match &endpoint.transport {
            Transport::Plaintext => {
                assert!(endpoint.address.ip().is_loopback(), "plaintext is confined to loopback");
                None
            }
            Transport::Tls { server_name, trust } => {
                assert!(http::largest_read(&limits.http) <= limits.tls.read, "HTTP fits TLS intake");
                assert!(http::largest_room(&limits.http) <= limits.tls.send, "HTTP fits TLS room");
                Some(tls::Client::new(trust, server_name.clone(), &limits.tls))
            }
        };
        let mut requests = Queue::uncapped();
        let mut state = State::New;
        match request(endpoint, provider, credential.as_ref(), limits) {
            Ok(call) => requests.push(http::Request::Call(call)),
            Err(()) => state = State::Closed { answer: Some(ListAnswer::Unreadable) },
        }
        Exchange {
            token,
            address: endpoint.address,
            provider,
            deadlines,
            limits: *limits,
            state,
            link: Link::new(tls),
            http: http::Client::new(&limits.http),
            events: Queue::uncapped(),
            requests,
            plain: Queue::uncapped(),
            document: List::with_capacity(limits.list.body),
            now: Time::ZERO,
        }
    }

    pub fn start(&mut self, now: Time, below: &mut Queue<Lower>) {
        self.now = now;
        let state = mem::replace(&mut self.state, State::Closed { answer: None });
        self.state = match state {
            State::New => {
                below.push(Lower::Connect { owner: self.token, addr: skein_io::Addr::Ip(self.address) });
                State::Connecting { whole: at(now, self.deadlines.whole), until: at(now, self.deadlines.connect) }
            }
            State::Connecting { .. }
            | State::Handshaking { .. }
            | State::Reading { .. }
            | State::Closing { .. }
            | State::Closed { .. } => state,
        };
        self.fire(now, below);
    }

    pub fn up(&mut self, event: LowerEvent, now: Time, below: &mut Queue<Lower>) {
        self.now = now;
        match event {
            LowerEvent::Connecting { socket, .. } => self.link.connecting(socket, below),
            LowerEvent::Connected { .. } => {
                let state = mem::replace(&mut self.state, State::Closed { answer: None });
                self.state = match state {
                    State::Connecting { whole, .. } => match self.link.connected(&self.link_env()) {
                        Start::Ready => State::Reading { whole },
                        Start::Handshaking => State::Handshaking { whole, until: at(now, self.deadlines.handshake) },
                        Start::Closing => unreachable!("an active exchange has an open link"),
                    },
                    State::New
                    | State::Handshaking { .. }
                    | State::Reading { .. }
                    | State::Closing { .. }
                    | State::Closed { .. } => state,
                };
            }
            LowerEvent::Stream { up, .. } => self.link.up(&self.link_env(), up),
            LowerEvent::Failed { .. } => self.finish(ListAnswer::Unreachable, true, below),
            LowerEvent::Closed { .. } => {
                self.drive(below);
                let state = mem::replace(&mut self.state, State::Closed { answer: None });
                self.state = match state {
                    State::Closing { answer } => State::Closed { answer: Some(answer) },
                    State::Closed { answer } => State::Closed { answer },
                    State::New | State::Connecting { .. } | State::Handshaking { .. } | State::Reading { .. } => {
                        State::Closed { answer: Some(ListAnswer::Unreachable) }
                    }
                };
                self.link.closed();
                self.clear();
            }
            LowerEvent::Listening { .. }
            | LowerEvent::Accepted { .. }
            | LowerEvent::Output { .. }
            | LowerEvent::Spawned { .. }
            | LowerEvent::Exited { .. }
            | LowerEvent::Usage { .. }
            | LowerEvent::Resized
            | LowerEvent::Window { .. }
            | LowerEvent::Shutdown { .. } => {}
        }
        self.fire(now, below);
    }

    pub fn fire(&mut self, now: Time, below: &mut Queue<Lower>) {
        self.now = now;
        if let Some(until) = self.policy_deadline()
            && now >= until
        {
            self.finish(ListAnswer::Deadline, true, below);
        }
        self.link.fire(now, below);
        self.drive(below);
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let deadline = earliest(self.policy_deadline(), self.link.next_deadline());
        if self.link.has_work()
            || !self.events.is_empty()
            || !self.plain.is_empty()
            || (self.reading() && !self.requests.is_empty())
        {
            earliest(deadline, Some(self.now))
        } else {
            deadline
        }
    }

    pub fn cancel(&mut self, below: &mut Queue<Lower>) {
        match self.state {
            State::New => {
                self.state = State::Closed { answer: Some(ListAnswer::Deadline) };
                self.clear();
            }
            State::Closing { .. } => {
                self.state = State::Closing { answer: ListAnswer::Deadline };
                self.link.close(&self.link_env(), true, below);
            }
            State::Connecting { .. } | State::Handshaking { .. } | State::Reading { .. } => {
                self.finish(ListAnswer::Deadline, true, below);
            }
            State::Closed { .. } => {}
        }
    }

    pub fn answer(&mut self) -> Option<ListAnswer> {
        match &mut self.state {
            State::Closed { answer } => answer.take(),
            State::New
            | State::Connecting { .. }
            | State::Handshaking { .. }
            | State::Reading { .. }
            | State::Closing { .. } => None,
        }
    }

    fn reading(&self) -> bool {
        match self.state {
            State::Reading { .. } => true,
            State::New
            | State::Connecting { .. }
            | State::Handshaking { .. }
            | State::Closing { .. }
            | State::Closed { .. } => false,
        }
    }

    fn policy_deadline(&self) -> Option<Time> {
        match self.state {
            State::Connecting { whole, until } | State::Handshaking { whole, until } => earliest(whole, until),
            State::Reading { whole } => whole,
            State::New | State::Closing { .. } | State::Closed { .. } => None,
        }
    }

    fn link_env(&self) -> Env<link::Limits> {
        Env {
            now: self.now,
            wall: self.limits.wall,
            limits: link::Limits { tls: self.limits.tls, close_timeout: self.limits.close_timeout },
        }
    }

    fn http_env(&self) -> Env<http::Limits> {
        Env { now: self.now, wall: Wall::EPOCH, limits: self.limits.http }
    }

    fn drive(&mut self, below: &mut Queue<Lower>) {
        // A bounded turn; any remaining internal move makes next_deadline immediately due.
        let http_env = self.http_env();
        for _move in 0_u32..64 {
            if let Some(event) = self.events.pop() {
                if self.reading() {
                    self.http_event(event, below);
                }
                continue;
            }
            match self.link.pump(below) {
                Moved::Above(event) => {
                    match event {
                        link::Event::Ready => {
                            let state = mem::replace(&mut self.state, State::Closed { answer: None });
                            self.state = match state {
                                State::Handshaking { whole, .. } => State::Reading { whole },
                                State::New
                                | State::Connecting { .. }
                                | State::Reading { .. }
                                | State::Closing { .. }
                                | State::Closed { .. } => state,
                            };
                        }
                        link::Event::Stream(up) => {
                            if self.reading() {
                                http::up(&mut self.http, &http_env, up, &mut self.events, &mut self.plain);
                            }
                        }
                        link::Event::Failed => self.finish(ListAnswer::Unreachable, true, below),
                    }
                    continue;
                }
                Moved::Within | Moved::Sent => continue,
                Moved::Nothing => {}
            }
            if !self.reading() {
                break;
            }
            if let Some(down) = self.plain.pop() {
                self.link.down(&self.link_env(), down);
            } else if let Some(request) = self.requests.pop() {
                http::down(&mut self.http, &http_env, request, &mut self.events, &mut self.plain);
            } else {
                break;
            }
        }
    }

    fn http_event(&mut self, event: http::Event, below: &mut Queue<Lower>) {
        match event {
            http::Event::Response(response) => {
                if !(200..300).contains(&response.status) {
                    self.finish(ListAnswer::Refused { status: response.status }, true, below);
                } else if !identity(&response) || too_long(response.framing, self.limits.list.body) {
                    self.finish(ListAnswer::Unreadable, true, below);
                } else {
                    self.read_body();
                }
            }
            http::Event::Body(Up::Bytes(bytes)) => {
                if skein_lib::len_u64(bytes.len()) > u64::from(self.document.room()) {
                    self.finish(ListAnswer::Unreadable, true, below);
                } else {
                    for byte in &bytes {
                        self.document.push(*byte).expect("body bytes fit the checked bound");
                    }
                    self.read_body();
                }
            }
            http::Event::Done(_) => {
                let answer = match list::parse(self.provider, self.document.as_slice(), &self.limits.list) {
                    Ok(rows) => ListAnswer::Listed(rows),
                    Err(list::Unreadable) => ListAnswer::Unreadable,
                };
                self.finish(answer, false, below);
            }
            http::Event::Failed(error) => {
                let answer = match error {
                    http::Error::Closed(_) | http::Error::Truncated { .. } | http::Error::Stream(_) => {
                        ListAnswer::Unreachable
                    }
                    http::Error::Refused(_)
                    | http::Error::Status
                    | http::Error::Version
                    | http::Error::Header
                    | http::Error::HeadTooLong
                    | http::Error::TooManyHeaders
                    | http::Error::Framing
                    | http::Error::ChunkSize
                    | http::Error::Chunk
                    | http::Error::Trailer
                    | http::Error::Upgrade => ListAnswer::Unreadable,
                };
                self.finish(answer, true, below);
            }
            http::Event::Body(Up::Failed(_)) => self.finish(ListAnswer::Unreachable, true, below),
            http::Event::Body(Up::End | Up::Room) | http::Event::Upload(_) | http::Event::Closed => {}
        }
    }

    fn read_body(&mut self) {
        self.requests.push(http::Request::Body(Down::Demand {
            read: Read::Arrived { max: self.document.room().saturating_add(1).min(self.limits.http.read) },
            room: 0,
        }));
    }

    fn finish(&mut self, answer: ListAnswer, abort: bool, below: &mut Queue<Lower>) {
        match self.state {
            State::Closing { .. } | State::Closed { .. } => return,
            State::New | State::Connecting { .. } | State::Handshaking { .. } | State::Reading { .. } => {}
        }
        self.state = State::Closing { answer };
        self.clear();
        let env = self.http_env();
        http::down(&mut self.http, &env, http::Request::Close, &mut self.events, &mut self.plain);
        self.events.clear();
        self.plain.clear();
        self.link.close(&self.link_env(), abort, below);
    }

    fn clear(&mut self) {
        self.events.clear();
        self.requests.clear();
        self.plain.clear();
        self.document.clear();
    }
}

fn request(
    endpoint: &Endpoint,
    provider: Provider,
    credential: Option<&Credential>,
    limits: &ExchangeLimits,
) -> Result<http::Call, ()> {
    if endpoint.llm.provider != provider {
        return Err(());
    }
    if let Some(credential) = credential
        && skein_lib::len_u64(credential.value.len()) > u64::from(endpoint.credential.value)
    {
        return Err(());
    }
    let fields = match skein_llm::client::credential_headers(&endpoint.llm, credential) {
        Ok(fields) => fields,
        Err(_error) => return Err(()),
    };
    let mut headers = List::with_capacity(limits.http.headers);
    for (name, value) in [
        (b"host".as_slice(), endpoint.llm.authority.as_ref()),
        (b"accept", b"application/json"),
        (b"accept-encoding", b"identity"),
    ] {
        if headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).is_err() {
            return Err(());
        }
    }
    for header in fields.iter().chain(endpoint.llm.headers.iter()) {
        if headers.push(header.clone()).is_err() {
            return Err(());
        }
    }
    Ok(http::Call {
        method: Method::Get,
        target: bytes::copy_of(list::target(provider)),
        headers: headers.into_boxed(),
        body: http::Body::None,
        close: true,
    })
}

fn identity(response: &http::Response) -> bool {
    for header in &response.headers {
        if header.is(b"content-encoding") && !skein_http::header::trim(&header.value).eq_ignore_ascii_case(b"identity")
        {
            return false;
        }
    }
    true
}

fn too_long(framing: http::Framing, bound: u32) -> bool {
    match framing {
        http::Framing::Length(length) => length > u64::from(bound),
        http::Framing::Empty | http::Framing::Chunked | http::Framing::UntilEnd => false,
    }
}

#[expect(clippy::manual_map, reason = "the step subset spells Option conversions with a match")]
fn at(now: Time, duration: Option<Duration>) -> Option<Time> {
    match duration {
        Some(duration) => Some(now.saturating_add(duration)),
        None => None,
    }
}
