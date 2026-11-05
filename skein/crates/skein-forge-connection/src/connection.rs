//! Forge connection (forge-connection.md, sections 3 and 8), on skein/docs/design/link.md, 3.
use alloc::boxed::Box;
use skein_forge::decode;
pub(crate) use skein_forge::plan::Failure;

/// A forge connection's state; the component arms its deadline.
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
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{
    Env, Intake, List, Queue, Time, bytes,
    stream::{Down, Read, Up},
};
use skein_link::{Link, Moved as LinkMoved, Start, Upload};
pub(crate) mod forgejo;
pub(crate) mod github;

/// One movement of the stack (forge-connection.md, 8.1).
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
/// Whether a connection still belongs to a configured forge (skein/docs/design/forge-connection.md, 3.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Membership {
    Configured,
    Removed,
}
/// Whose a connection is (forge-connection.md, 7.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Use {
    /// The calls' pool.
    Calls,
    /// A Forgejo forge's own connection, which a call to that forge may take.
    Own,
    /// A GitHub forge's own connection, outside the calls' count.
    Reserved,
}
/// One connection's stack, socket identity and pending output.
#[expect(
    clippy::struct_excessive_bools,
    reason = "phase flags and HTTP exchange presence have independent transitions"
)]
pub(crate) struct Connection {
    pub(crate) forge: u16,
    pub(crate) role: Use,
    pub(crate) origin: Option<u32>,
    pub(crate) draining: bool,
    pub(crate) phase: Phase,
    pub(crate) deadline: Option<Time>,
    pub(crate) link: Link,
    pub(crate) closing: bool,
    pub(crate) membership: Membership,
    pub(crate) written: bool,
    http: http::Client,
    json: tokenizer::Tokenizer,
    shared: Shared,
    http_up: Queue<http::Event>,
    http_down: Queue<Down>,
    json_up: Queue<tokenizer::Event>,
    json_down: Queue<Down>,
    json_next: Option<tokenizer::Request>,
    json_body: JsonBody,
    ended: Option<Ended>,
    exchanging: bool,
    receive: Receiving,
    upload: Option<Upload>,
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
            role: Use::Calls,
            origin,
            draining: false,
            phase: Phase::Connecting,
            deadline: Some(deadline),
            link,
            closing: false,
            membership: Membership::Configured,
            written: false,
            http,
            json: tokenizer::Tokenizer::new(&limits::documents(l).json),
            shared,
            http_up: Queue::uncapped(),
            http_down: Queue::uncapped(),
            json_up: Queue::uncapped(),
            json_down: Queue::uncapped(),
            json_next: None,
            json_body: JsonBody {
                intake: Intake::with_capacity(l.chunk.max(5)),
                read: None,
                below: false,
                ended: false,
            },
            ended: None,
            exchanging: false,
            receive: Receiving::Empty,
            upload: None,
            raw_cap: shared.answer_bytes,
            json_limits: limits::documents(l).json,
        }
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
    pub(crate) fn send(&mut self, env: &Env<Limits>, call: http::Call, upload: Option<Box<[u8]>>) {
        self.clear_json_body();
        self.json_limits = limits::documents(&env.limits).json;
        self.ended = None;
        self.exchanging = true;
        self.upload = Upload::of(upload);
        self.written = false;
        self.phase = Phase::Sending { written: false };
        self.http_down(env, http::Request::Call(call));
        if self.upload.is_some() {
            self.http_down(
                env,
                http::Request::Upload(Down::Demand { read: Read::Nothing, room: self.http_env(env).limits.send }),
            );
        }
    }
    pub fn drain_redirect(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) -> bool {
        if !self.draining {
            return false;
        }
        if self.deadline <= Some(env.now) {
            self.timeout();
        }
        self.drain(env, io);
        if self.phase == Phase::Idle || self.phase == Phase::Closed {
            self.draining = false;
        }
        self.ready()
    }
    pub fn discard_redirect(&mut self, env: &Env<Limits>) {
        if self.phase == Phase::Reading {
            self.clear_json_body();
            self.draining = true;
            self.receive = Receiving::Empty;
            self.json_next = None;
            self.json_up.clear();
            self.json_down.clear();
            self.http_down(env, http::Request::Discard);
        }
    }
    fn link_env(&self, env: &Env<Limits>) -> Env<skein_link::Limits> {
        env.with(skein_link::Limits { tls: env.limits.tls, close_timeout: self.shared.io.close_timeout })
    }
    pub fn connect(&mut self, env: &Env<Limits>) {
        self.phase = match self.link.connected(&self.link_env(env)) {
            Start::Handshaking => Phase::Handshaking,
            Start::Ready => Phase::Idle,
            Start::Closing => Phase::Closed,
        };
    }
    pub fn stream(&mut self, env: &Env<Limits>, event: Up) {
        self.link.up(&self.link_env(env), event);
    }
    fn http_env(&self, env: &Env<Limits>) -> Env<http::Limits> {
        Env { now: env.now, wall: env.wall, limits: limits::http(&env.limits, self.link.encrypted(), &self.shared.io) }
    }
    fn http_down(&mut self, env: &Env<Limits>, request: http::Request) {
        let h = self.http_env(env);
        http::down(&mut self.http, &h, request, &mut self.http_up, &mut self.http_down);
    }
    pub(crate) fn json_request(&mut self, env: &Env<Limits>, request: tokenizer::Request) {
        let j = Env { now: env.now, wall: env.wall, limits: self.json_limits };
        tokenizer::down(&mut self.json, &j, request, &mut self.json_up, &mut self.json_down);
    }
    /// One child-machine movement, in the holder's bounded pump.
    pub(crate) fn step(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>, held: bool) -> Moved {
        let held = held || self.phase == Phase::Closed;
        if !held || self.closing {
            match self.link.pump(io) {
                LinkMoved::Sent => {
                    if self.exchanging {
                        self.written = true;
                        self.phase =
                            if self.upload.is_some() { Phase::Sending { written: true } } else { Phase::Reading };
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
        if let Some(request) = self.json_next.take() {
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
            return self.http_event(env, event);
        }
        Moved::Nothing
    }
    /// Moves a stack whose holder has released its exchange.
    pub(crate) fn drain(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) {
        for _ in 0_u32..64 {
            match self.step(env, io, false) {
                Moved::Nothing => break,
                Moved::Finished { reuse, raw } => {
                    drop(raw);
                    self.finish(env, reuse, false);
                }
                Moved::Within | Moved::Head(_) | Moved::Token(_) => {}
            }
        }
        self.ended = None;
    }
    pub(crate) fn set_json_next(&mut self, request: tokenizer::Request) {
        self.json_next = Some(request);
    }
    pub(crate) fn begin_json(&mut self, env: &Env<Limits>, limits: tokenizer::Limits) {
        self.receive = Receiving::Json;
        self.json_limits = limits;
        self.json = tokenizer::Tokenizer::new(&limits);
        self.json_request(env, tokenizer::Request::Next);
    }
    pub(crate) fn discard(&mut self, env: &Env<Limits>) {
        self.receive = Receiving::Empty;
        self.http_down(env, http::Request::Discard);
    }
    pub(crate) fn finish(&mut self, env: &Env<Limits>, reuse: http::Reuse, closes: bool) {
        let close = reuse == http::Reuse::Close || closes;
        self.exchanging = false;
        self.clear_json_body();
        self.upload = None;
        self.phase = if close { Phase::Closed } else { Phase::Idle };
        self.deadline = if close { None } else { Some(env.now.saturating_add(env.limits.idle)) };
    }
    fn clear_json_body(&mut self) {
        self.json_body.intake.clear();
        self.json_body.read = None;
        self.json_body.below = false;
        self.json_body.ended = false;
    }
    fn json_demand(&mut self, env: &Env<Limits>, down: Down) {
        match down {
            Down::Demand { read, room } => {
                assert_eq!(room, 0, "JSON bodies only read");
                if read == Read::Nothing {
                    self.json_body.read = None;
                    self.json_body.intake.clear();
                    if self.json_body.below {
                        self.json_body.below = false;
                        self.http_down(env, http::Request::Body(Down::Demand { read, room: 0 }));
                    }
                } else {
                    assert!(self.json_body.read.is_none(), "one tokenizer demand at a time");
                    self.json_body.read = Some(read);
                }
            }
            Down::Send(_) | Down::Finish => unreachable!("JSON bodies only read"),
        }
    }
    /// One local answer or refill is one of the pump's bounded movements.
    fn pump_json_body(&mut self, env: &Env<Limits>) -> bool {
        let Some(read) = self.json_body.read else { return false };
        let event = if let Some(bytes) = self.json_body.intake.meet(read) {
            Up::Bytes(bytes)
        } else if self.json_body.ended {
            self.json_body.intake.clear();
            Up::End
        } else {
            // HTTP may already have its final Bytes, End and Done queued.
            // Consume those before asking its body stream for another read.
            if self.json_body.below || !self.http_up.is_empty() {
                return false;
            }
            let max = self.json_body.intake.room();
            assert!(max > 0, "a full intake meets every tokenizer demand");
            self.json_body.below = true;
            self.http_down(env, http::Request::Body(Down::Demand { read: Read::Arrived { max }, room: 0 }));
            return true;
        };
        self.json_body.read = None;
        let j = Env { now: env.now, wall: env.wall, limits: self.json_limits };
        tokenizer::up(&mut self.json, &j, event, &mut self.json_up, &mut self.json_down);
        true
    }
    fn link_event(&mut self, env: &Env<Limits>, event: skein_link::Event) {
        match event {
            skein_link::Event::Ready => self.phase = Phase::Idle,
            skein_link::Event::Stream(event) => {
                let h = self.http_env(env);
                http::up(&mut self.http, &h, event, &mut self.http_up, &mut self.http_down);
            }
            skein_link::Event::Failed => self.failed(if self.written { Failure::Seen } else { Failure::Unseen }),
        }
    }
    fn json_body_event(&mut self, event: Up) {
        match event {
            Up::Bytes(bytes) => {
                assert!(self.json_body.below, "one answer to a JSON body refill");
                self.json_body.below = false;
                self.json_body.intake.append(&bytes).expect("refill fits the remaining intake room");
            }
            Up::End => {
                self.json_body.below = false;
                self.json_body.ended = true;
            }
            Up::Failed(_) => self.failed(Failure::Seen),
            Up::Room => unreachable!("JSON bodies only read"),
        }
    }
    fn http_event(&mut self, env: &Env<Limits>, event: http::Event) -> Moved {
        match event {
            http::Event::Response(response) => return Moved::Head(response),
            http::Event::Body(event) => match &mut self.receive {
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
                            self.receive = Receiving::Empty;
                            self.malformed(decode::Malformed::Bound { field: b"log" });
                            self.phase = Phase::Closed;
                            self.deadline = None;
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
                            let grow = match core::mem::replace(&mut self.receive, Receiving::Empty) {
                                Receiving::Raw { grow, .. } => grow,
                                Receiving::Json | Receiving::Tail { .. } | Receiving::Empty => {
                                    unreachable!("raw body")
                                }
                            };
                            self.phase = Phase::Closed;
                            self.deadline = None;
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
                let raw = match core::mem::replace(&mut self.receive, Receiving::Empty) {
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
        self.receive = match keep {
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
        let count = match &self.receive {
            Receiving::Raw { grow, remaining, .. } => match remaining {
                Some(left) => grow.room().min(env.limits.chunk).min(u32::try_from(*left).unwrap_or(u32::MAX)).max(1),
                None => 1,
            },
            Receiving::Tail { remaining, .. } => match remaining {
                Some(left) => env.limits.chunk.min(u32::try_from(*left).unwrap_or(u32::MAX)).max(1),
                None => 1,
            },
            Receiving::Json | Receiving::Empty => return,
        };
        self.http_down(env, http::Request::Body(Down::Demand { read: Read::Fill(count), room: 0 }));
    }
    fn upload(&mut self, env: &Env<Limits>, event: Up) {
        match event {
            Up::Room => {
                let room = self.http_env(env).limits.send;
                let Some(upload) = &mut self.upload else { return };
                let (piece, next) = upload.room(room);
                self.http_down(env, http::Request::Upload(piece));
                if next == Down::Finish {
                    self.upload = None;
                    self.phase = Phase::Reading;
                }
                self.http_down(env, http::Request::Upload(next));
            }
            Up::Failed(_) | Up::End => {
                self.upload = None;
            }
            Up::Bytes(_) => unreachable!("uploads only write"),
        }
    }
    pub(crate) fn malformed_json(&mut self, error: decode::Malformed) {
        self.clear_json_body();
        self.receive = Receiving::Empty;
        self.phase = Phase::Closed;
        self.deadline = None;
        self.malformed(error);
    }
    fn malformed(&mut self, error: decode::Malformed) {
        self.exchanging = false;
        self.ended = Some(Ended::Malformed(error));
    }
    pub fn failed(&mut self, failure: Failure) {
        if self.phase == Phase::Closed {
            return;
        }
        self.clear_json_body();
        self.phase = Phase::Closed;
        self.deadline = None;
        self.exchanging = false;
        self.ended = Some(Ended::Failed(failure));
    }
    pub fn timeout(&mut self) {
        self.failed(if self.written { Failure::Seen } else { Failure::Unseen });
    }
    pub fn in_flight(&self) -> bool {
        self.exchanging
            || match self.phase {
                Phase::Connecting | Phase::Handshaking => true,
                Phase::Closed | Phase::Sending { .. } | Phase::Reading | Phase::Idle => false,
            }
    }
    pub fn close(&mut self, env: &Env<Limits>, abort: bool, io: &mut Queue<skein_io::Request>) {
        if self.closing {
            return;
        }
        self.clear_json_body();
        self.phase = Phase::Closed;
        self.deadline = None;
        self.exchanging = false;
        self.closing = true;
        self.link.close(&self.link_env(env), abort, io);
    }
    /// Aborts the connection, including a close still under way (programming-model.md, 5.2).
    pub fn abort(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) {
        if !self.closing {
            self.close(env, true, io);
            return;
        }
        self.link.close(&self.link_env(env), true, io);
    }
    pub(crate) fn take_ended(&mut self) -> Option<Ended> {
        self.ended.take()
    }
    pub fn ready(&self) -> bool {
        if self.phase == Phase::Closed {
            return self.ended.is_some() || self.link.has_work();
        }
        !self.http_up.is_empty()
            || !self.http_down.is_empty()
            || self.link.has_work()
            || !self.json_up.is_empty()
            || !self.json_down.is_empty()
            || self.ended.is_some()
            || self.json_next.is_some()
            || (self.json_body.read.is_some() && !self.json_body.below)
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
            .field("phase", &self.phase)
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
