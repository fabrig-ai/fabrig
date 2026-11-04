//! The browser's same-origin HTTP host (skein/docs/design/playground.md, 4).
mod connection;
mod jar;

use alloc::collections::{BTreeMap, VecDeque};
use connection::Connection;
use skein_http::{Header, Method, client as http};
use skein_io::{self as io, kernel};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::Host;
use skein_ui_web::{bridge, fetch, transport::StreamDuty};

/// What the application gives its visitor (skein/docs/design/playground.md, 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub listener: core::net::SocketAddr,
    /// The application's name for its listener (`name:port`).
    pub host: Box<[u8]>,
    pub exchanges: u32,
    pub cookies: u32,
    /// The most bytes of one cookie's name and value.
    pub cookie_bytes: u32,
    pub io: io::Limits,
    pub http: http::Limits,
}

// Fetch has no Debug surface; the host's connection map exposes its lifecycle.
struct Book(fetch::Fetch);
impl core::fmt::Debug for Book {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("Fetch").finish_non_exhaustive()
    }
}

/// A program of the world: fetch's inputs in, fetch's reports out, over skein's HTTP client.
#[derive(Debug)]
pub struct Visitor {
    config: Config,
    io: io::Io,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    inputs: VecDeque<bridge::Input>,
    reports: Vec<fetch::Report>,
    fetch: Book,
    jar: jar::Jar,
    /// Tickets also route closing connections after Fetch has retired them.
    connections: BTreeMap<Token, Connection>,
}
impl Visitor {
    #[must_use]
    pub fn new(config: &Config) -> Self {
        assert!(http::largest_read(&config.http) <= config.io.largest_read(), "HTTP reads fit io");
        assert!(http::largest_room(&config.http) <= config.io.largest_room(), "HTTP writes fit io");
        let operations = io::operations(&config.io).expect("bounded io");
        Self {
            config: config.clone(),
            io: io::Io::new(&config.io),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            submissions: Queue::with_capacity(operations),
            completions: Queue::with_capacity(operations),
            inputs: VecDeque::new(),
            reports: Vec::new(),
            fetch: Book(fetch::Fetch::new(config.exchanges)),
            jar: jar::Jar::new(config.cookies, config.cookie_bytes),
            connections: BTreeMap::new(),
        }
    }
    /// An input between steps; it acts on it at its next turn (skein/docs/design/world.md, 5).
    pub fn enter(&mut self, input: bridge::Input) {
        self.inputs.push_back(input);
    }
    /// Its reports since the last take, in order.
    pub fn take(&mut self, reports: &mut Vec<fetch::Report>) {
        reports.append(&mut self.reports);
    }

    fn report(&mut self, report: Option<fetch::Report>) {
        self.reports.extend(report);
    }

    fn end(&mut self, ticket: u64) {
        let report = if self.fetch.0.stream_ticket() == Some(ticket) {
            self.fetch.0.end_stream(ticket)
        } else {
            self.fetch.0.fail(ticket)
        };
        self.report(report);
        if let Some(connection) = self.connections.get_mut(&Token::new(ticket)) {
            connection.close(&mut self.requests);
        }
    }

    fn input(&mut self, input: bridge::Input, wall: Wall) {
        match input {
            bridge::Input::Exchange(exchange) => {
                if let Some(ticket) = self.fetch.0.begin(
                    exchange.id,
                    exchange.bound,
                    &exchange.keep.iter().map(String::as_str).collect::<Vec<_>>(),
                ) {
                    self.open(ticket, exchange, false, wall);
                } else {
                    self.reports.push(fetch::Report::Failed { exchange: exchange.id });
                }
            }
            bridge::Input::Abandon { exchange } => {
                if let Some(ticket) = self.fetch.0.abandon(exchange)
                    && let Some(connection) = self.connections.get_mut(&Token::new(ticket))
                {
                    connection.close(&mut self.requests);
                }
            }
            bridge::Input::Stream(StreamDuty::Open { path, headers, bound }) => {
                if let Some(ticket) = self.fetch.0.stream_ticket() {
                    self.end(ticket);
                }
                let ticket = self.fetch.0.open_stream(bound);
                let headers = headers
                    .into_iter()
                    .map(|(name, value)| {
                        Ok((String::from_utf8(name.into_vec())?, String::from_utf8(value.into_vec())?))
                    })
                    .collect::<Result<Vec<_>, alloc::string::FromUtf8Error>>();
                match headers {
                    Ok(headers) => self.open(
                        ticket,
                        bridge::Exchange {
                            id: 0,
                            path,
                            method: "GET".into(),
                            headers,
                            body: Box::from([]),
                            bound,
                            cache: fetch::Cache::Browser,
                            keep: Vec::new(),
                        },
                        true,
                        wall,
                    ),
                    Err(_) => self.end(ticket),
                }
            }
            bridge::Input::Stream(StreamDuty::Close) => {
                if let Some(ticket) = self.fetch.0.stream_ticket() {
                    self.end(ticket);
                }
            }
            bridge::Input::ReadOn => {
                let report = self.fetch.0.read_on();
                if report.is_some() {
                    self.report(report);
                } else if let Some(ticket) = self.fetch.0.stream_ticket()
                    && let Some(connection) = self.connections.get_mut(&Token::new(ticket))
                {
                    connection.read_stream();
                }
            }
        }
    }

    fn open(&mut self, ticket: u64, exchange: bridge::Exchange, stream: bool, wall: Wall) {
        let Some(method) = Method::from_bytes(exchange.method.as_bytes()) else {
            self.end(ticket);
            return;
        };
        if self.connections.len() >= usize::try_from(self.config.io.sockets).expect("socket count fits")
            || !exchange.path.starts_with(b"/")
            || exchange.path.starts_with(b"//")
            || exchange.path.contains(&b'\\')
        {
            self.end(ticket);
            return;
        }
        let mut headers: Vec<_> = exchange
            .headers
            .into_iter()
            .map(|(name, value)| Header {
                name: name.into_bytes().into_boxed_slice(),
                value: value.into_bytes().into_boxed_slice(),
            })
            .collect();
        headers.push(Header { name: Box::from(b"Host".as_slice()), value: self.config.host.clone() });
        if method != Method::Get && method != Method::Head {
            let mut origin = b"http://".to_vec();
            origin.extend_from_slice(&self.config.host);
            headers.push(Header { name: Box::from(b"Origin".as_slice()), value: origin.into_boxed_slice() });
        }
        if let Some(value) = self.jar.header(&exchange.path, wall) {
            headers.push(Header { name: Box::from(b"Cookie".as_slice()), value });
        }
        let owner = Token::new(ticket);
        let connection = Connection::new(
            &self.config.http,
            http::Call {
                method,
                target: exchange.path,
                headers: headers.into_boxed_slice(),
                body: http::Body::Length(u64::try_from(exchange.body.len()).expect("body length fits")),
                close: true,
            },
            exchange.body,
            stream.then_some(exchange.bound.min(self.config.http.read)),
            exchange.keep,
            exchange.bound,
        );
        self.connections.insert(owner, connection);
        self.requests.push(io::Request::Connect { owner, addr: io::Addr::Ip(self.config.listener) });
    }

    fn event(&mut self, event: io::Event, env: &Env<http::Limits>) {
        let Some(owner) = event.owner() else { return };
        let Some(connection) = self.connections.get_mut(&owner) else { return };
        match event {
            io::Event::Connecting { socket, .. } => connection.connecting(socket, &mut self.requests),
            io::Event::Connected { .. } => connection.connected(),
            io::Event::Stream { up, .. } => connection.up(up, env),
            io::Event::Failed { .. } => self.end(owner.raw()),
            io::Event::Closed { .. } => {
                self.end(owner.raw());
                self.connections.remove(&owner);
            }
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Output { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => {}
        }
    }
}
impl Host for Visitor {
    fn iterate(&mut self, now: Time, wall: Wall) {
        // Inputs precede completions: abandoning suppresses answers already on their way.
        for _ in 0..self.inputs.len() {
            if let Some(input) = self.inputs.pop_front() {
                self.input(input, wall);
            }
        }
        let env = Env { now, wall, limits: self.config.io };
        let http_env = Env { now, wall, limits: self.config.http };
        io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        for _ in 0..self.events.len() {
            if let Some(event) = self.events.pop() {
                self.event(event, &http_env);
            }
        }
        for (owner, connection) in &mut self.connections {
            connection.progress(
                owner.raw(),
                &http_env,
                &mut self.fetch.0,
                &mut self.jar,
                &mut self.reports,
                &mut self.requests,
            );
        }
        io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        self.io.is_ready()
            || !self.inputs.is_empty()
            || !self.events.is_empty()
            || !self.requests.is_empty()
            || !self.completions.is_empty()
            || self.connections.values().any(Connection::work_pending)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.inputs.is_empty()
            && self.events.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
            && self.connections.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.config.io).expect("bounded io")
    }
}
