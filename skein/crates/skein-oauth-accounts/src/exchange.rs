//! One issuer exchange and its HTTP/TLS socket (oauth/README.md, sections 2 and 3, and oauth/accounts.md, section 4).
//! Keeps child handoffs and one bounded response document; knows no grant policy
//! or durable store. The component routes io events and advances one child at a
//! time, then consumes the client's terminal before closing the connection.

use crate::{Endpoint, Limits, Transport};

use alloc::boxed::Box;
use skein_http::{Header, Method, client as http};
use skein_io::{Event as IoEvent, Request as IoRequest};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, List, Queue, Secret, Time, bytes, len_u64};
use skein_link::{self as link, Link, Moved, Start, Upload};
use skein_oauth as oauth;
use skein_tls::client as tls;

pub(crate) enum Stage {
    Loading,
    Locking,
    Reading,
    Running,
    Keeping { candidate: oauth::SavedToken, missing: Box<[u8]>, profile: oauth::ProfileClaims },
    Forgetting { revoked: bool },
    Finished,
}

pub(crate) enum Purpose {
    Refresh,
    SignOut,
    SignIn { waiting: bool },
}

#[derive(PartialEq, Eq)]
pub(crate) enum OwnerLock {
    Released,
    Waiting { until: Time },
    Held,
}

pub(crate) struct Exchange {
    pub(crate) account: u32,
    /// Its admission number, which orders the walks that serve the oldest
    /// exchange first; `None` once retired, as the slab still visits it
    /// until reclaim and every walk skips it.
    pub(crate) admitted: Option<u64>,
    pub(crate) sign_in_deadline: Option<Time>,
    pub(crate) client: oauth::Client,
    pub(crate) above: Queue<oauth::Request>,
    pub(crate) web: Option<Web>,
    pub(crate) stage: Stage,
    pub(crate) pending_grant: bool,
    pub(crate) aborted: bool,
    pub(crate) purpose: Purpose,
    pub(crate) visit: Option<Secret>,
    pub(crate) state_refusals: u32,
    pub(crate) sign_in: Option<oauth::Event>,
    pub(crate) return_location: Option<Box<[u8]>>,
    pub(crate) subject: Option<Box<[u8]>>,
    pub(crate) listener: Option<crate::listener::Listener>,
    pub(crate) file_active: bool,
    pub(crate) lock: OwnerLock,
    pub(crate) lock_candidate: Option<(oauth::SavedToken, Box<[u8]>, oauth::ProfileClaims)>,
}

impl Exchange {
    pub(crate) fn signing_in(&self) -> bool {
        match self.purpose {
            Purpose::Refresh | Purpose::SignOut => false,
            Purpose::SignIn { .. } => true,
        }
    }
    pub(crate) fn signing_out(&self) -> bool {
        match self.purpose {
            Purpose::SignOut => true,
            Purpose::Refresh | Purpose::SignIn { .. } => false,
        }
    }
    pub(crate) fn waiting(&self) -> bool {
        match self.purpose {
            Purpose::Refresh | Purpose::SignOut => false,
            Purpose::SignIn { waiting } => waiting,
        }
    }
    pub(crate) fn received(&mut self) {
        match &mut self.purpose {
            Purpose::SignIn { waiting } => *waiting = false,
            Purpose::Refresh | Purpose::SignOut => {}
        }
    }

    pub(crate) fn new(account: u32, client: oauth::Client, pending_grant: bool) -> Exchange {
        Exchange {
            account,
            admitted: None,
            sign_in_deadline: None,
            client,
            above: Queue::uncapped(),
            web: None,
            stage: Stage::Running,
            pending_grant,
            aborted: false,
            purpose: Purpose::Refresh,
            visit: None,
            state_refusals: 0,
            sign_in: None,
            return_location: None,
            subject: None,
            listener: None,
            file_active: false,
            lock: OwnerLock::Released,
            lock_candidate: None,
        }
    }

    pub(crate) fn next_deadline(&self) -> Option<Time> {
        if self.waiting() && self.sign_in.is_some() { self.sign_in_deadline } else { self.client.next_deadline() }
    }

    pub(crate) fn has_work(&self) -> bool {
        match self.stage {
            Stage::Locking | Stage::Reading => return false,
            Stage::Loading | Stage::Running | Stage::Keeping { .. } | Stage::Forgetting { .. } | Stage::Finished => {}
        }
        if self.settled()
            || !self.above.is_empty()
            || match &self.listener {
                Some(listener) => listener.has_work() || (self.visit.is_some() && listener.ready()),
                None => self.visit.is_some(),
            }
        {
            return true;
        }
        match &self.web {
            Some(web) => web.has_work(),
            None => false,
        }
    }

    pub(crate) fn settled(&self) -> bool {
        match self.stage {
            Stage::Finished => {
                (match self.lock {
                    OwnerLock::Waiting { .. } => false,
                    OwnerLock::Released | OwnerLock::Held => true,
                }) && !self.file_active
                    && self.web.is_none()
                    && match &self.listener {
                        Some(listener) => listener.settled(),
                        None => true,
                    }
                    && self.above.is_empty()
            }
            Stage::Loading
            | Stage::Locking
            | Stage::Reading
            | Stage::Running
            | Stage::Keeping { .. }
            | Stage::Forgetting { .. } => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Connecting,
    Handshaking,
    Calling,
    Answered,
    Closing,
    Closed,
}

/// HTTP over one link (skein/docs/design/net/link.md, 3).
pub(crate) struct Web {
    link: Link,
    phase: Phase,
    http: http::Client,
    events: Queue<http::Event>,
    requests: Queue<http::Request>,
    plain: Queue<Down>,
    upload: Upload,
    document: List<u8>,
    response: Option<http::Response>,
    pub(crate) answer: Option<oauth::HttpResponse>,
    pub(crate) failure: Option<oauth::Failure>,
    id: u64,
    pub(crate) sent: bool,
}

impl Web {
    pub(crate) fn new(request: oauth::HttpRequest, endpoint: &Endpoint, limits: &Limits) -> Option<Web> {
        let method = match request.method {
            oauth::Method::Post => Method::Post,
            oauth::Method::Get => Method::Get,
        };
        let authority_start = bytes::find(&request.endpoint, b"://")?.checked_add(3)?;
        let rest = request.endpoint.get(authority_start..)?;
        let slash = bytes::find(rest, b"/")?;
        let authority = rest.get(..slash)?;
        let target = rest.get(slash..)?;
        let mut headers = List::with_capacity(2);
        headers.push(Header { name: bytes::copy_of(b"host"), value: bytes::copy_of(authority) }).expect("two headers");
        if method == Method::Post {
            headers
                .push(Header { name: bytes::copy_of(b"content-type"), value: bytes::copy_of(request.content_type) })
                .expect("two headers");
        }
        let mut requests = Queue::uncapped();
        requests.push(http::Request::Call(http::Call {
            method,
            target: bytes::copy_of(target),
            headers: headers.into_boxed(),
            body: if method == Method::Post {
                http::Body::Length(len_u64(request.body.len()))
            } else {
                http::Body::None
            },
            close: true,
        }));
        if method == Method::Post {
            requests.push(http::Request::Upload(Down::Demand { read: Read::Nothing, room: limits.http.send }));
        }
        let tls = match &endpoint.transport {
            Transport::Tls { server_name, trust } => Some(tls::Client::new(trust, server_name.clone(), &limits.tls)),
            Transport::Plaintext => None,
        };
        Some(Web {
            link: Link::new(tls),
            phase: Phase::Connecting,
            http: http::Client::new(&limits.http),
            events: Queue::uncapped(),
            requests,
            plain: Queue::uncapped(),
            upload: Upload::new(request.body),
            document: List::with_capacity(
                limits
                    .client
                    .document
                    .document_bytes
                    .max(limits.client.metadata_bytes)
                    .max(limits.client.key_set_bytes),
            ),
            response: None,
            answer: None,
            failure: None,
            id: request.id,
            sent: false,
        })
    }

    pub(crate) fn up(&mut self, env: &Env<Limits>, event: IoEvent, io: &mut Queue<IoRequest>) {
        match event {
            IoEvent::Connecting { socket, .. } => self.link.connecting(socket, io),
            IoEvent::Connected { .. } => {
                if self.phase == Phase::Connecting {
                    self.phase = match self.link.connected(&link_env(env)) {
                        Start::Handshaking => Phase::Handshaking,
                        Start::Ready => Phase::Calling,
                        Start::Closing => Phase::Closing,
                    };
                }
            }
            IoEvent::Stream { up, .. } => {
                self.link.up(&link_env(env), up);
                if !self.link.encrypted() {
                    self.pump(env, io);
                }
            }
            IoEvent::Failed { .. } => {
                self.transport_failed(env);
                // io owns failed-connect settlement and sends Closed itself.
            }
            IoEvent::Closed { .. } => {
                if self.phase != Phase::Answered && self.phase != Phase::Closing {
                    self.transport_failed(env);
                }
                self.phase = Phase::Closed;
                self.link.closed();
            }
            IoEvent::Listening { .. }
            | IoEvent::Accepted { .. }
            | IoEvent::Output { .. }
            | IoEvent::Spawned { .. }
            | IoEvent::Exited { .. }
            | IoEvent::Usage { .. }
            | IoEvent::Resized
            | IoEvent::Window { .. }
            | IoEvent::Shutdown { .. } => {}
        }
    }

    fn transport_failed(&mut self, env: &Env<Limits>) {
        if self.answer.is_none()
            && self.phase != Phase::Answered
            && self.phase != Phase::Closing
            && self.phase != Phase::Closed
        {
            self.answer = Some(oauth::HttpResponse {
                id: self.id,
                status: 0,
                body: Box::new([]),
                retry_after: Duration::ZERO,
                evidence: if self.sent { oauth::HttpEvidence::Unknown } else { oauth::HttpEvidence::Unsent },
                now: env.now,
                wall: env.wall,
            });
            self.phase = Phase::Answered;
        }
    }

    pub(crate) fn progress(&mut self, env: &Env<Limits>, io: &mut Queue<IoRequest>) {
        let http_env = env.with(env.limits.http);
        if self.phase == Phase::Closed {
            return;
        }
        if self.phase != Phase::Closing
            && let Some(event) = self.events.pop()
        {
            self.http_event(env, event);
            return;
        }
        if self.pump(env, io) {
            return;
        }
        if self.phase != Phase::Calling {
            return;
        }
        if let Some(down) = self.plain.pop() {
            match &down {
                Down::Send(bytes) if !bytes.is_empty() => self.sent = true,
                Down::Send(_) | Down::Demand { .. } | Down::Finish => {}
            }
            self.link.down(&link_env(env), down);
            if !self.link.encrypted() {
                self.pump(env, io);
            }
            return;
        }
        if let Some(request) = self.requests.pop() {
            http::down(&mut self.http, &http_env, request, &mut self.events, &mut self.plain);
        }
    }

    // Plaintext moves cross the link in the same turn as their HTTP source.
    fn pump(&mut self, env: &Env<Limits>, io: &mut Queue<IoRequest>) -> bool {
        match self.link.pump(io) {
            Moved::Above(event) => {
                match event {
                    link::Event::Ready => self.phase = Phase::Calling,
                    link::Event::Stream(up) => {
                        http::up(&mut self.http, &env.with(env.limits.http), up, &mut self.events, &mut self.plain);
                    }
                    link::Event::Failed => {
                        // HTTP keeps a body read whole before a stream failure.
                        // Only a handshake failure bypasses HTTP.
                        if self.phase == Phase::Handshaking {
                            self.transport_failed(env);
                        }
                    }
                }
                true
            }
            Moved::Sent | Moved::Within => true,
            Moved::Nothing => false,
        }
    }

    fn http_event(&mut self, env: &Env<Limits>, event: http::Event) {
        match event {
            http::Event::Response(response) => {
                self.response = Some(response);
                self.requests.push(http::Request::Body(Down::Demand {
                    read: Read::Arrived { max: self.document.room().saturating_add(1) },
                    room: 0,
                }));
            }
            http::Event::Upload(Up::Room) => {
                let (send, next) = self.upload.room(env.limits.http.send);
                self.requests.push(http::Request::Upload(send));
                self.requests.push(http::Request::Upload(next));
            }
            http::Event::Body(Up::Bytes(bytes)) => {
                let length = u32::try_from(bytes.len()).ok();
                let too_large = match length {
                    Some(len) => len > self.document.room(),
                    None => true,
                };
                if too_large {
                    self.failure = Some(oauth::Failure::Limit);
                    self.phase = Phase::Answered;
                } else {
                    for byte in &bytes {
                        self.document.push(*byte).expect("response document admitted");
                    }
                    self.requests.push(http::Request::Body(Down::Demand {
                        read: Read::Arrived { max: self.document.room().saturating_add(1) },
                        room: 0,
                    }));
                }
            }
            http::Event::Done(_) => {
                let response = self.response.take().expect("HTTP done follows response");
                self.answer = Some(oauth::HttpResponse {
                    id: self.id,
                    status: response.status,
                    body: self.document.to_boxed(),
                    retry_after: retry_after(&response),
                    evidence: oauth::HttpEvidence::Response,
                    now: env.now,
                    wall: env.wall,
                });
                self.phase = Phase::Answered;
            }
            http::Event::Failed(_)
            | http::Event::Upload(Up::Failed(_) | Up::End | Up::Bytes(_))
            | http::Event::Body(Up::Failed(_) | Up::Room) => self.transport_failed(env),
            http::Event::Body(Up::End) | http::Event::Closed => {}
        }
    }

    pub(crate) fn closing(&self) -> bool {
        self.phase == Phase::Closing
    }

    pub(crate) fn close(&mut self, env: &Env<Limits>, abort: bool, io: &mut Queue<IoRequest>) {
        if self.phase == Phase::Closed {
            return;
        }
        if abort {
            self.answer = None;
            self.failure = None;
        }
        self.phase = Phase::Closing;
        self.link.close(&link_env(env), abort, io);
    }

    pub(crate) fn next_deadline(&self) -> Option<Time> {
        self.link.next_deadline()
    }

    pub(crate) fn fire(&mut self, now: Time, io: &mut Queue<IoRequest>) {
        self.link.fire(now, io);
    }

    pub(crate) fn closed(&self) -> bool {
        self.phase == Phase::Closed
    }

    pub(crate) fn has_work(&self) -> bool {
        self.link.has_work()
            || match self.phase {
                Phase::Closed | Phase::Answered => self.answer.is_some() || self.failure.is_some(),
                Phase::Closing | Phase::Handshaking | Phase::Connecting => false,
                Phase::Calling => !self.events.is_empty() || !self.requests.is_empty() || !self.plain.is_empty(),
            }
    }
}

fn link_env(env: &Env<Limits>) -> Env<link::Limits> {
    env.with(link::Limits { tls: env.limits.tls, close_timeout: env.limits.io.close_timeout })
}

fn retry_after(response: &http::Response) -> Duration {
    match response.header(b"retry-after") {
        Some(value) => match bytes::decimal(value) {
            Some(seconds) => Duration::from_secs(seconds),
            None => Duration::ZERO,
        },
        None => Duration::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Time, Wall};

    fn token_web(limits: &Limits) -> Web {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
        let endpoint = Endpoint { address: ([127, 0, 0, 1], 31000).into(), transport: Transport::Plaintext };
        let request = oauth::HttpRequest {
            method: oauth::Method::Post,
            id: 7,
            endpoint: bytes::copy_of(b"http://127.0.0.1:31000/token"),
            content_type: b"application/x-www-form-urlencoded",
            body: Box::new([]),
            deadline: Time::ZERO,
        };
        let mut web = Web::new(request, &endpoint, limits).expect("token endpoint");
        while web.requests.pop().is_some() {}
        web.http_event(
            &env,
            http::Event::Response(http::Response {
                version: skein_http::Version::Http11,
                status: 200,
                headers: Box::new([]),
                framing: http::Framing::Chunked,
            }),
        );
        web
    }

    #[test]
    fn a_token_response_read_as_it_arrives_is_the_same_whatever_its_cuts() {
        let mut limits = crate::tests::limits();
        limits.client.document.document_bytes = 32;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        for piece in [1, 21] {
            let mut web = token_web(&limits);
            assert_eq!(
                web.requests.pop(),
                Some(http::Request::Body(Down::Demand { read: Read::Arrived { max: 33 }, room: 0 }))
            );
            for bytes in br#"{"access_token":"ok"}"#.chunks(piece) {
                web.http_event(&env, http::Event::Body(Up::Bytes(bytes::copy_of(bytes))));
                if piece == 1 && web.document.len() < 21 {
                    while web.requests.pop().is_some() {}
                }
            }
            assert_eq!(
                web.requests.pop(),
                Some(http::Request::Body(Down::Demand { read: Read::Arrived { max: 12 }, room: 0 }))
            );
            web.http_event(&env, http::Event::Done(http::Reuse::Close));
            assert_eq!(web.answer.expect("token response").body.as_ref(), br#"{"access_token":"ok"}"#);
        }
    }
}
