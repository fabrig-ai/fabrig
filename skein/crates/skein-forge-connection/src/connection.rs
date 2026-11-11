//! Forge connection (forge/connection.md, sections 3 and 8), on skein/docs/design/net/link.md, 3.
use alloc::boxed::Box;
use skein_forge::decode;
pub(crate) use skein_forge::plan::Failure;

/// A forge connection's phase; the component arms its deadline.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    /// Nothing held.
    Closed,
    /// io's connect, until the call's deadline (or a start read's).
    Connecting,
    /// TLS's handshake, until the same deadline.
    Handshaking,
    /// The request going down; `written` once its first byte went.
    Sending { written: bool },
    /// The head, then the body as the decoder demands.
    Reading,
    /// Kept for its forge until `idle`.
    Idle,
}

use crate::{
    grow::{Grow, LogTail},
    limits::{self, Limits, Shared},
    settings::{Forge, Transport},
};
use State::{Closed, Connecting, Handshaking, Idle, Reading, Sending};
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{
    Env, Intake, List, Queue, Time, bytes,
    stream::{Down, Read, Up},
};
use skein_link::{Link, Moved as LinkMoved, Start, Upload};
mod conditional;
use crate::kept::Store;
use conditional::Conditional;
pub(crate) mod forgejo;
pub(crate) mod github;

/// One movement of the stack (forge/connection.md, 8.1).
pub(crate) enum Moved {
    Within,
    Nothing,
    Head(http::Response),
    Token(tokenizer::Event),
    Finished { reuse: http::Reuse, raw: Payload },
}
pub(crate) enum Payload {
    Json,
    Empty,
    Raw { bytes: Box<[u8]>, more: bool },
}
pub(crate) enum Ended {
    Failed(Failure),
    Malformed(decode::Malformed),
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Keep {
    Head(u32),
    Tail(u32),
}
/// One exchange's response body state.
#[derive(Debug)]
enum Receiving {
    Json,
    Raw { grow: Grow, max: u32, remaining: Option<u64> },
    Tail { tail: LogTail, read: u64, remaining: Option<u64> },
    Empty,
}
/// The body stream below the tokenizer, bounded by the existing read cap.
struct JsonBody {
    intake: Intake,
    read: Option<Read>,
    below: bool,
    ended: bool,
}
/// Whether a connection still belongs to a configured forge (skein/docs/design/forge/connection.md, 3.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Membership {
    Configured,
    Removed,
}
/// Whose a connection is (forge/connection.md, 7.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Use {
    /// The calls' pool.
    Calls,
    /// A Forgejo forge's own connection, which a call to that forge may take.
    Own,
    /// A GitHub forge's own connection, outside the calls' count.
    Reserved,
}
/// A connection's phases hold their exchange (skein/docs/design/forge/connection.md, 8.1).
enum State {
    // Redirect draining survives its ending.
    Closed { closing: bool, ended: Option<Ended>, draining: bool },
    Connecting { until: Option<Time> },
    Handshaking { until: Option<Time> },
    Sending { exchange: Exchange, until: Option<Time> },
    Reading { exchange: Exchange, draining: bool, until: Option<Time> },
    Idle { draining: bool, until: Option<Time> },
}
struct Exchange {
    conditional: Conditional,
    upload: Option<Upload>,
    receive: Receiving,
    json_body: JsonBody,
    json_next: Option<tokenizer::Request>,
}
impl State {
    fn take(&mut self) -> Self {
        core::mem::replace(self, Closed { closing: false, ended: None, draining: false })
    }
    fn exchange_mut(&mut self) -> Option<&mut Exchange> {
        match self {
            Sending { exchange, .. } | Reading { exchange, .. } => Some(exchange),
            Closed { .. } | Idle { .. } | Connecting { .. } | Handshaking { .. } => None,
        }
    }
    fn into_exchange(self) -> Option<Exchange> {
        match self {
            Sending { exchange, .. } | Reading { exchange, .. } => Some(exchange),
            Closed { .. } | Idle { .. } | Connecting { .. } | Handshaking { .. } => None,
        }
    }
    fn until(&self) -> Option<Time> {
        match self {
            Closed { .. } => None,
            Connecting { until }
            | Handshaking { until }
            | Sending { until, .. }
            | Reading { until, .. }
            | Idle { until, .. } => *until,
        }
    }
    fn idle(self) -> Self {
        let until = self.until();
        Idle { draining: self.draining(), until }
    }
    fn end(self, ended: Option<Ended>, closing: bool) -> Self {
        let draining = self.draining();
        let (ended, closing) = match self {
            Closed { ended: previous, closing: requested, .. } => (ended.or(previous), closing || requested),
            Connecting { .. } | Handshaking { .. } | Sending { .. } | Reading { .. } | Idle { .. } => (ended, closing),
        };
        Closed { closing, ended, draining }
    }
    fn draining(&self) -> bool {
        match self {
            Reading { draining, .. } | Idle { draining, .. } | Closed { draining, .. } => *draining,
            Connecting { .. } | Handshaking { .. } | Sending { .. } => false,
        }
    }
}
/// One connection's stack, socket identity and pending output.
pub struct Connection {
    pub(crate) forge: u16,
    pub(crate) role: Use,
    pub(crate) origin: Option<u32>,
    state: State,
    pub(crate) identity: Option<u64>,
    reused: u32,
    pub(crate) link: Link,
    pub(crate) membership: Membership,
    // The last send remains observable in Idle and Closed after its exchange is released.
    pub(crate) written: bool,
    http: http::Client,
    json: tokenizer::Tokenizer,
    shared: Shared,
    http_up: Queue<http::Event>,
    http_down: Queue<Down>,
    json_up: Queue<tokenizer::Event>,
    json_down: Queue<Down>,
    raw_cap: u32,
    json_limits: tokenizer::Limits,
}
impl Connection {
    #[cfg(test)]
    pub fn new(forge: &Forge, l: &Limits, shared: Shared, deadline: Time) -> Self {
        Self::at_origin(forge, None, l, shared, deadline)
    }
    #[expect(clippy::manual_map, reason = "the programming model excludes closures")]
    pub fn at_origin(forge: &Forge, origin: Option<u32>, l: &Limits, shared: Shared, deadline: Time) -> Self {
        let configured = match origin {
            Some(origin) => Some(forge.log_origins.get(origin).expect("configured log origin")),
            None => None,
        };
        let tls = match (&forge.transport, configured) {
            (Transport::Plaintext { .. } | Transport::Tls { .. }, Some(configured)) if !configured.origin.tls => None,
            (Transport::Tls { trust, .. }, Some(configured)) => Some(skein_tls::client::Client::new(
                trust,
                crate::settings::log_name(&configured.origin).expect("checked DNS name"),
                &l.tls,
            )),
            (Transport::Plaintext { .. }, Some(_)) => unreachable!("HTTPS origin requires trust"),
            (Transport::Plaintext { .. }, None) => None,
            (Transport::Tls { trust, server_name }, None) => {
                Some(skein_tls::client::Client::new(trust, server_name.clone(), &l.tls))
            }
        };
        let link = Link::new(tls);
        let http = http::Client::new(&limits::http(l, link.encrypted(), &shared.io));
        Self {
            forge: forge.number,
            identity: None,
            reused: 0,
            role: Use::Calls,
            origin,
            state: Connecting { until: Some(deadline) },
            link,
            membership: Membership::Configured,
            written: false,
            http,
            json: tokenizer::Tokenizer::new(&limits::documents(l).json),
            shared,
            http_up: Queue::uncapped(),
            http_down: Queue::uncapped(),
            json_up: Queue::uncapped(),
            json_down: Queue::uncapped(),
            raw_cap: shared.answer_bytes,
            json_limits: limits::documents(l).json,
        }
    }
    pub(crate) fn deadline(&self) -> Option<Time> {
        self.state.until()
    }
    pub(crate) fn until(&mut self, until: Option<Time>) {
        match &mut self.state {
            Closed { .. } => {}
            Connecting { until: held }
            | Handshaking { until: held }
            | Sending { until: held, .. }
            | Reading { until: held, .. }
            | Idle { until: held, .. } => *held = until,
        }
    }
    pub fn phase(&self) -> Phase {
        match self.state {
            Closed { .. } => Phase::Closed,
            Connecting { .. } => Phase::Connecting,
            Handshaking { .. } => Phase::Handshaking,
            Sending { .. } => Phase::Sending { written: self.written },
            Reading { .. } => Phase::Reading,
            Idle { .. } => Phase::Idle,
        }
    }
    pub(crate) fn closing(&self) -> bool {
        match self.state {
            Closed { closing, .. } => closing,
            Connecting { .. } | Handshaking { .. } | Sending { .. } | Reading { .. } | Idle { .. } => false,
        }
    }
    pub(crate) fn draining(&self) -> bool {
        self.state.draining()
    }
    fn exchange(&mut self) -> &mut Exchange {
        self.state.exchange_mut().expect("an HTTP event belongs to an exchange")
    }
    pub(crate) fn host(&self, forge: &Forge) -> Box<[u8]> {
        match self.origin {
            Some(origin) => crate::settings::log_host(forge.log_origins.get(origin).expect("configured origin")),
            None => crate::settings::host(forge).expect("checked forge host"),
        }
    }
    pub(crate) fn body(headers: &mut List<skein_http::Header>, upload: Option<&[u8]>) -> http::Body {
        match upload {
            Some(body) => {
                headers.push(field(b"Content-Type", b"application/json")).expect("request header capacity");
                http::Body::Length(skein_lib::len_u64(body.len()))
            }
            None => http::Body::None,
        }
    }
    pub(crate) fn send(
        &mut self,
        env: &Env<Limits>,
        mut call: http::Call,
        upload: Option<Box<[u8]>>,
        kept: &mut Store,
    ) {
        self.json_limits = limits::documents(&env.limits).json;
        self.written = false;
        let intake = Intake::with_capacity(env.limits.chunk.max(5));
        let json_body = JsonBody { intake, read: None, below: false, ended: false };
        let conditional = Conditional::new(self, &mut call, kept);
        let exchange =
            Exchange { conditional, upload: Upload::of(upload), receive: Receiving::Empty, json_body, json_next: None };
        self.state = Sending { exchange, until: self.deadline() };
        self.http_down(env, http::Request::Call(call));
        if self.exchange().upload.is_some() {
            let demand = Down::Demand { read: Read::Nothing, room: self.http_env(env).limits.send };
            self.http_down(env, http::Request::Upload(demand));
        }
    }
    pub(crate) fn drain_redirect(
        &mut self,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
        kept: &mut Store,
    ) -> bool {
        if !self.draining() {
            return false;
        }
        if self.deadline() <= Some(env.now) {
            self.timeout();
        }
        self.drain(env, io, kept);
        match &mut self.state {
            Idle { draining, .. } | Closed { draining, .. } => *draining = false,
            Connecting { .. } | Handshaking { .. } | Sending { .. } | Reading { .. } => {}
        }
        self.ready()
    }
    pub fn discard_redirect(&mut self, env: &Env<Limits>) {
        self.state = match self.state.take() {
            Reading { mut exchange, until, .. } => {
                exchange.json_body.intake.clear();
                exchange.json_body.read = None;
                exchange.json_body.below = false;
                exchange.json_body.ended = false;
                exchange.receive = Receiving::Empty;
                exchange.json_next = None;
                self.json_up.clear();
                self.json_down.clear();
                self.http_down(env, http::Request::Discard);
                Reading { exchange, draining: true, until }
            }
            state @ (Closed { .. } | Connecting { .. } | Handshaking { .. } | Sending { .. } | Idle { .. }) => state,
        };
    }
    fn link_env(&self, env: &Env<Limits>) -> Env<skein_link::Limits> {
        env.with(skein_link::Limits { tls: env.limits.tls, close_timeout: self.shared.io.close_timeout })
    }
    pub fn connect(&mut self, env: &Env<Limits>) {
        self.state = match self.state.take() {
            state @ Connecting { until } => match self.link.connected(&self.link_env(env)) {
                Start::Handshaking => Handshaking { until },
                Start::Ready => state.idle(),
                Start::Closing => state,
            },
            state @ Closed { .. } => match self.link.connected(&self.link_env(env)) {
                Start::Handshaking | Start::Ready => unreachable!("a closed connection's link is ending"),
                Start::Closing => state,
            },
            Handshaking { .. } | Sending { .. } | Reading { .. } | Idle { .. } => {
                unreachable!("a connect completes once")
            }
        };
    }
    pub fn stream(&mut self, env: &Env<Limits>, event: Up) {
        self.link.up(&self.link_env(env), event);
    }
    fn http_env(&self, env: &Env<Limits>) -> Env<http::Limits> {
        env.with(limits::http(&env.limits, self.link.encrypted(), &self.shared.io))
    }
    fn http_down(&mut self, env: &Env<Limits>, request: http::Request) {
        if self.replay_demand(&request) {
            return;
        }
        let h = self.http_env(env);
        http::down(&mut self.http, &h, request, &mut self.http_up, &mut self.http_down);
    }
    pub(crate) fn json_request(&mut self, env: &Env<Limits>, request: tokenizer::Request) {
        let j = env.with(self.json_limits);
        tokenizer::down(&mut self.json, &j, request, &mut self.json_up, &mut self.json_down);
    }
    /// One child-machine movement, in the holder's bounded pump.
    pub(crate) fn step(
        &mut self,
        env: &Env<Limits>,
        io: &mut Queue<skein_io::Request>,
        held: bool,
        kept: &mut Store,
    ) -> Moved {
        let held = held || self.phase() == Phase::Closed;
        if !held || self.closing() {
            match self.link.pump(io) {
                LinkMoved::Sent => {
                    if self.state.exchange_mut().is_some() {
                        self.written = true;
                        let until = self.deadline();
                        let state = self.state.take();
                        let draining = state.draining();
                        let exchange = state.into_exchange().expect("a send belongs to an exchange");
                        self.state = if exchange.upload.is_some() {
                            Sending { exchange, until }
                        } else {
                            Reading { exchange, draining, until }
                        };
                    }
                    return Moved::Within;
                }
                LinkMoved::Above(event) => {
                    self.link_event(env, event);
                    return Moved::Within;
                }
                LinkMoved::Within => return Moved::Within,
                LinkMoved::Nothing => {}
            }
        }
        if held {
            return Moved::Nothing;
        }
        if let Some(down) = self.http_down.pop() {
            self.link.down(&self.link_env(env), down);
            return Moved::Within;
        }
        if let Some(down) = self.json_down.pop() {
            self.json_demand(env, down);
            return Moved::Within;
        }
        if let Some(exchange) = self.state.exchange_mut()
            && let Some(request) = exchange.json_next.take()
        {
            self.json_request(env, request);
            return Moved::Within;
        }
        if let Some(event) = self.json_up.pop() {
            return Moved::Token(event);
        }
        if self.pump_json_body(env) {
            return Moved::Within;
        }
        if let Some(event) = self.http_up.pop() {
            return self.http_event(env, event, kept);
        }
        self.pump_replay(env, kept).unwrap_or(Moved::Nothing)
    }
    /// Moves a stack whose holder has released its exchange.
    pub(crate) fn drain(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>, kept: &mut Store) {
        for _ in 0_u32..64 {
            match self.step(env, io, false, kept) {
                Moved::Nothing => break,
                Moved::Finished { reuse, .. } => self.finish(env, reuse, false),
                Moved::Within | Moved::Head(_) | Moved::Token(_) => {}
            }
        }
        let _ended = self.take_ended();
    }
    pub(crate) fn set_json_next(&mut self, request: tokenizer::Request) {
        self.exchange().json_next = Some(request);
    }
    pub(crate) fn begin_json(&mut self, env: &Env<Limits>, limits: tokenizer::Limits) {
        self.exchange().receive = Receiving::Json;
        self.json_limits = limits;
        self.json = tokenizer::Tokenizer::new(&limits);
        self.json_request(env, tokenizer::Request::Next);
    }
    pub(crate) fn discard(&mut self, env: &Env<Limits>) {
        self.cancel_capture();
        self.exchange().receive = Receiving::Empty;
        self.http_down(env, http::Request::Discard);
    }
    pub(crate) fn finish(&mut self, env: &Env<Limits>, reuse: http::Reuse, closes: bool) {
        let close = reuse == http::Reuse::Close || closes;
        let draining = self.draining();
        let until = Some(env.now.saturating_add(env.limits.idle));
        self.state = if close { self.state.take().end(None, false) } else { Idle { draining, until } };
    }
    fn json_demand(&mut self, env: &Env<Limits>, down: Down) {
        let Some(exchange) = self.state.exchange_mut() else { return };
        match down {
            Down::Demand { read, room } => {
                assert_eq!(room, 0, "JSON bodies only read");
                if read == Read::Nothing {
                    exchange.json_body.read = None;
                    exchange.json_body.intake.clear();
                    if exchange.json_body.below {
                        exchange.json_body.below = false;
                        self.http_down(env, http::Request::Body(Down::Demand { read, room: 0 }));
                    }
                } else {
                    assert!(exchange.json_body.read.is_none(), "one tokenizer demand at a time");
                    exchange.json_body.read = Some(read);
                }
            }
            Down::Send(_) | Down::Finish => unreachable!("JSON bodies only read"),
        }
    }
    /// One local answer or refill is one of the pump's bounded movements.
    fn pump_json_body(&mut self, env: &Env<Limits>) -> bool {
        let Some(exchange) = self.state.exchange_mut() else { return false };
        let Some(read) = exchange.json_body.read else { return false };
        let event = if let Some(bytes) = exchange.json_body.intake.meet(read) {
            Up::Bytes(bytes)
        } else if exchange.json_body.ended {
            exchange.json_body.intake.clear();
            Up::End
        } else {
            // HTTP may already have its final Bytes, End and Done queued.
            // Consume those before asking its body stream for another read.
            if exchange.json_body.below || !self.http_up.is_empty() {
                return false;
            }
            let max = exchange.json_body.intake.room();
            assert!(max > 0, "a full intake meets every tokenizer demand");
            exchange.json_body.below = true;
            self.http_down(env, http::Request::Body(Down::Demand { read: Read::Arrived { max }, room: 0 }));
            return true;
        };
        exchange.json_body.read = None;
        let j = env.with(self.json_limits);
        tokenizer::up(&mut self.json, &j, event, &mut self.json_up, &mut self.json_down);
        true
    }
    fn link_event(&mut self, env: &Env<Limits>, event: skein_link::Event) {
        match event {
            skein_link::Event::Ready => {
                self.state = self.state.take().idle();
            }
            skein_link::Event::Stream(event) => {
                let h = self.http_env(env);
                http::up(&mut self.http, &h, event, &mut self.http_up, &mut self.http_down);
            }
            skein_link::Event::Failed => self.failed(if self.written { Failure::Seen } else { Failure::Unseen }),
        }
    }
    fn json_body_event(&mut self, event: Up) {
        let exchange = self.exchange();
        match event {
            Up::Bytes(bytes) => {
                assert!(exchange.json_body.below, "one answer to a JSON body refill");
                exchange.json_body.below = false;
                exchange.json_body.intake.append(&bytes).expect("refill fits the remaining intake room");
            }
            Up::End => {
                exchange.json_body.below = false;
                exchange.json_body.ended = true;
            }
            Up::Failed(_) => self.failed(Failure::Seen),
            Up::Room => unreachable!("JSON bodies only read"),
        }
    }
    fn http_event(&mut self, env: &Env<Limits>, event: http::Event, kept: &mut Store) -> Moved {
        match &event {
            http::Event::Body(event) => self.capture(event, kept),
            http::Event::Response(_)
            | http::Event::Upload(_)
            | http::Event::Done(_)
            | http::Event::Failed(_)
            | http::Event::Closed => {}
        }
        match event {
            http::Event::Response(response) => return Moved::Head(self.conditional_head(env, response, kept)),
            http::Event::Body(event) => match &mut self.exchange().receive {
                Receiving::Json => self.json_body_event(event),
                Receiving::Empty => {
                    drop(event);
                }
                Receiving::Tail { tail, read, remaining } => match event {
                    Up::Bytes(piece) => {
                        let length = skein_lib::len_u64(piece.len());
                        *read = read.saturating_add(length);
                        if let Some(left) = remaining {
                            *left = left.saturating_sub(length);
                        }
                        if *read > u64::from(env.limits.body_bytes_in) {
                            self.exchange().receive = Receiving::Empty;
                            self.malformed_json(decode::Malformed::Bound { field: b"log" });
                        } else {
                            tail.write(&piece);
                            self.raw_demand(env);
                        }
                    }
                    Up::End => {}
                    Up::Failed(_) => self.failed(Failure::Seen),
                    Up::Room => unreachable!("Tail bodies only read"),
                },
                Receiving::Raw { grow, max, remaining } => match event {
                    Up::Bytes(piece) => {
                        if let Some(left) = remaining {
                            *left = left.saturating_sub(skein_lib::len_u64(piece.len()));
                        }
                        let more = grow.write(&piece).is_err() || grow.len() > *max;
                        if more {
                            let grow = match core::mem::replace(&mut self.exchange().receive, Receiving::Empty) {
                                Receiving::Raw { grow, .. } => grow,
                                Receiving::Json | Receiving::Tail { .. } | Receiving::Empty => {
                                    unreachable!("raw body")
                                }
                            };
                            self.state =
                                Closed { closing: self.closing(), ended: self.take_ended(), draining: self.draining() };
                            return Moved::Finished {
                                reuse: http::Reuse::Close,
                                raw: Payload::Raw { bytes: grow.finish(), more: true },
                            };
                        }
                        self.raw_demand(env);
                    }
                    Up::End => {}
                    Up::Failed(_) => self.failed(Failure::Seen),
                    Up::Room => unreachable!("raw bodies only read"),
                },
            },
            http::Event::Upload(event) => self.upload(env, event),
            http::Event::Done(reuse) => {
                if self.replay_done(reuse) {
                    return Moved::Within;
                }
                self.end_kept(kept);
                let raw = match core::mem::replace(&mut self.exchange().receive, Receiving::Empty) {
                    Receiving::Json => Payload::Json,
                    Receiving::Empty => Payload::Empty,
                    Receiving::Raw { grow, max, .. } => {
                        let more = grow.len() > max;
                        Payload::Raw { bytes: grow.finish(), more }
                    }
                    Receiving::Tail { tail, .. } => {
                        let more = tail.dropped();
                        Payload::Raw { bytes: tail.finish(), more }
                    }
                };
                return Moved::Finished { reuse, raw };
            }
            http::Event::Failed(error) => self.failed(match error {
                http::Error::Closed(_) | http::Error::Refused(_) => Failure::Unseen,
                http::Error::Truncated { .. }
                | http::Error::Stream(_)
                | http::Error::Status
                | http::Error::Version
                | http::Error::Header
                | http::Error::HeadTooLong
                | http::Error::TooManyHeaders
                | http::Error::Framing
                | http::Error::ChunkSize
                | http::Error::Chunk
                | http::Error::Trailer
                | http::Error::Upgrade => Failure::Seen,
            }),
            http::Event::Closed => {}
        }
        Moved::Within
    }
    pub(crate) fn begin_raw(
        &mut self,
        env: &Env<Limits>,
        framing: http::Framing,
        keep: Keep,
        cap: u32,
        failed_step: bool,
    ) {
        self.raw_cap = cap;
        let remaining = match framing {
            http::Framing::Length(length) => Some(length),
            http::Framing::Empty => Some(0),
            http::Framing::Chunked | http::Framing::UntilEnd => None,
        };
        self.exchange().receive = match keep {
            Keep::Head(max) => Receiving::Raw {
                grow: Grow::new(env.limits.chunk, max.saturating_add(1).min(self.raw_cap)),
                max,
                remaining,
            },
            Keep::Tail(max) => Receiving::Tail {
                tail: if failed_step {
                    LogTail::failed_step(max.min(self.raw_cap))
                } else {
                    LogTail::new(max.min(self.raw_cap))
                },
                read: 0,
                remaining,
            },
        };
        self.raw_demand(env);
    }
    fn raw_demand(&mut self, env: &Env<Limits>) {
        let Some(exchange) = self.state.exchange_mut() else { return };
        let (room, remaining) = match &exchange.receive {
            Receiving::Raw { grow, remaining, .. } => (grow.room().min(env.limits.chunk), *remaining),
            Receiving::Tail { remaining, .. } => (env.limits.chunk, *remaining),
            Receiving::Json | Receiving::Empty => return,
        };
        let count = match remaining {
            Some(left) => room.min(u32::try_from(left).unwrap_or(u32::MAX)).max(1),
            None => 1,
        };
        self.http_down(env, http::Request::Body(Down::Demand { read: Read::Fill(count), room: 0 }));
    }
    fn upload(&mut self, env: &Env<Limits>, event: Up) {
        let room = self.http_env(env).limits.send;
        let Some(exchange) = self.state.exchange_mut() else { return };
        match event {
            Up::Room => {
                let Some(upload) = &mut exchange.upload else { return };
                let (piece, next) = upload.room(room);
                self.http_down(env, http::Request::Upload(piece));
                if next == Down::Finish {
                    let until = self.deadline();
                    let mut exchange = self.state.take().into_exchange().expect("upload belongs to an exchange");
                    exchange.upload = None;
                    self.state = Reading { exchange, draining: false, until };
                }
                self.http_down(env, http::Request::Upload(next));
            }
            Up::Failed(_) | Up::End => exchange.upload = None,
            Up::Bytes(_) => unreachable!("uploads only write"),
        }
    }
    pub(crate) fn malformed_json(&mut self, error: decode::Malformed) {
        self.state = self.state.take().end(Some(Ended::Malformed(error)), false);
    }
    pub fn failed(&mut self, failure: Failure) {
        if self.phase() == Phase::Closed {
            return;
        }
        self.state = self.state.take().end(Some(Ended::Failed(failure)), false);
    }
    pub fn timeout(&mut self) {
        self.failed(if self.written { Failure::Seen } else { Failure::Unseen });
    }
    pub fn in_flight(&self) -> bool {
        match &self.state {
            Closed { .. } | Idle { .. } => false,
            Connecting { .. } | Handshaking { .. } | Sending { .. } | Reading { .. } => true,
        }
    }
    pub fn close(&mut self, env: &Env<Limits>, abort: bool, io: &mut Queue<skein_io::Request>) {
        if self.closing() {
            return;
        }
        self.state = self.state.take().end(None, true);
        self.link.close(&self.link_env(env), abort, io);
    }
    /// Aborts the connection, including a close still under way (programming-model.md, 5.2).
    pub fn abort(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) {
        if !self.closing() {
            self.close(env, true, io);
            return;
        }
        self.link.close(&self.link_env(env), true, io);
    }
    pub(crate) fn take_ended(&mut self) -> Option<Ended> {
        match &mut self.state {
            Closed { ended, .. } => ended.take(),
            Connecting { .. } | Handshaking { .. } | Sending { .. } | Reading { .. } | Idle { .. } => None,
        }
    }
    pub fn ready(&self) -> bool {
        let exchange = match &self.state {
            Closed { ended, .. } => return ended.is_some() || self.link.has_work(),
            Sending { exchange, .. } | Reading { exchange, .. } => Some(exchange),
            Idle { .. } | Connecting { .. } | Handshaking { .. } => None,
        };
        if let Some(exchange) = exchange
            && (exchange.json_next.is_some() || (exchange.json_body.read.is_some() && !exchange.json_body.below))
        {
            return true;
        }
        self.replay_ready()
            || !self.http_up.is_empty()
            || !self.http_down.is_empty()
            || self.link.has_work()
            || !self.json_up.is_empty()
            || !self.json_down.is_empty()
    }
    /// The answer's remaining payload room, set by the holding plan before its raw read.
    pub fn raw_room(&mut self, room: u32) {
        self.raw_cap = room;
    }
}

pub(crate) fn field(name: &[u8], value: &[u8]) -> skein_http::Header {
    skein_http::Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }
}

impl core::fmt::Debug for Connection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Connection")
            .field("forge", &self.forge)
            .field("phase", &self.phase())
            .field("deadline", &self.deadline())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
