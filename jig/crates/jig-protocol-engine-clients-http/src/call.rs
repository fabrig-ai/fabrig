//! One HTTP server and its bounded request and response (protocol/client/wire.md, 2).
use crate::{Limits, Request, dispatch::Route};
use alloc::boxed::Box;
use skein_http::server as http;
use skein_lib::{
    Env, Intake, Queue, Token, bytes,
    length::len_u64,
    stream::{Down, Read},
};
#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the public room is independent of browser and call lifecycle state")]
pub(crate) struct Call {
    pub name: Token,
    pub method: skein_http::Method,
    pub public: bool,
    pub stream: Option<crate::stream::Stream>,
    pub events: Queue<http::Event>,
    pub requests: Queue<http::Request>,
    pub body_kind: http::Body,
    pub credential: Option<skein_lib::Secret>,
    pub browser: bool,
    /// Whether the credential came by the cookie (protocol/client/default-http.md, 3).
    pub cookie: bool,
    pub via: crate::Via,
    pub body: Intake,
    pub route: Route,
    pub response: Box<[u8]>,
    pub offset: usize,
    pub closed: bool,
    pub streaming: bool,
    pub reply_owed: bool,
}
impl Call {
    pub fn new(name: Token, limits: &Limits) -> Self {
        Self {
            name,
            method: skein_http::Method::Get,
            public: false,
            stream: None,
            events: Queue::with_capacity(8),
            requests: Queue::with_capacity(8),
            body_kind: http::Body::None,
            credential: None,
            browser: false,
            cookie: false,
            via: crate::Via::Listener,
            body: Intake::with_capacity(limits.body),
            route: Route::Missing,
            response: Box::new([]),
            offset: 0,
            closed: false,
            streaming: false,
            reply_owed: false,
        }
    }
    pub fn read(&mut self, limits: &Limits) {
        let read = match self.body_kind {
            http::Body::Length(length) => Read::Fill(
                u32::try_from(length)
                    .expect("checked HTTP body")
                    .saturating_sub(self.body.len())
                    .min(limits.read)
                    .max(1),
            ),
            http::Body::None | http::Body::Chunked => Read::Arrived { max: limits.read },
        };
        self.requests.push(http::Request::Body(Down::Demand { read, room: 0 }));
    }

    pub fn respond(&mut self, status: u16, body: Box<[u8]>, limits: &Limits) {
        self.respond_with_cookie(status, body, None, limits);
    }
    pub fn respond_with_cookie(
        &mut self,
        status: u16,
        body: Box<[u8]>,
        cookie: Option<skein_http::Header>,
        limits: &Limits,
    ) {
        let content = skein_http::Header {
            name: bytes::copy_of(b"content-type"),
            value: bytes::copy_of(b"application/octet-stream"),
        };
        let cache = skein_http::Header { name: bytes::copy_of(b"cache-control"), value: bytes::copy_of(b"no-store") };
        let nosniff =
            skein_http::Header { name: bytes::copy_of(b"x-content-type-options"), value: bytes::copy_of(b"nosniff") };
        let headers = match cookie {
            Some(cookie) => Box::from([content, cache, nosniff, cookie]),
            None => Box::from([content, cache, nosniff]),
        };
        let response = http::Response { status, headers, body: http::Body::Length(len_u64(body.len())), close: false };
        self.respond_with(response, body, limits);
    }
    pub fn respond_with(&mut self, response: http::Response, body: Box<[u8]>, limits: &Limits) {
        self.response = body;
        self.offset = 0;
        self.requests.push(http::Request::Discard);
        self.requests.push(http::Request::Respond(response));
        // The server completes HEAD at its response head, with no reply stream.
        if self.method == skein_http::Method::Head {
            return;
        }
        if self.response.is_empty() {
            self.requests.push(http::Request::Reply(Down::Finish));
        } else {
            self.room(limits);
        }
    }
    fn room(&mut self, limits: &Limits) {
        let left = self.response.len().checked_sub(self.offset).expect("response offset");
        self.requests.push(http::Request::Reply(Down::Demand {
            read: Read::Nothing,
            room: limits.send.min(u32::try_from(left).expect("bounded response")),
        }));
    }
    pub fn send(&mut self, limits: &Limits) {
        let end = self.offset.saturating_add(skein_lib::length::usize_of(limits.send)).min(self.response.len());
        self.requests.push(http::Request::Reply(Down::Send(bytes::copy_of(
            self.response.get(self.offset..end).expect("response slice"),
        ))));
        self.offset = end;
        if end == self.response.len() {
            self.requests.push(http::Request::Reply(Down::Finish));
        } else {
            self.room(limits);
        }
    }
    fn finish(&mut self, out: &mut Queue<Request>) {
        self.closed = true;
        if self.streaming {
            out.push(Request::Component(jig_protocol_engine_clients::FromTransport::StreamGone { stream: self.name }));
        } else if self.reply_owed {
            out.push(Request::Component(jig_protocol_engine_clients::FromTransport::ExchangeGone {
                exchange: self.name,
            }));
        }
        out.push(Request::Http { call: self.name, request: http::Request::Next });
    }
    pub fn http(&mut self, env: &Env<Limits>, event: http::Event, out: &mut Queue<Request>) {
        use skein_lib::stream::Up;
        match event {
            http::Event::Body(Up::Bytes(bytes)) => {
                if u64::from(self.body.len()).saturating_add(len_u64(bytes.len()))
                    > u64::from(crate::dispatch::body_bound(&self.route, env.limits.body))
                    || self.body.append(&bytes).is_err()
                {
                    self.route = Route::Missing;
                    self.respond(413, Box::new([]), &env.limits);
                } else {
                    self.read(&env.limits);
                }
            }
            http::Event::Body(Up::End) => {
                if self.route != Route::Missing {
                    if self.public {
                        let Route::Public(read) = core::mem::replace(&mut self.route, Route::Missing) else {
                            unreachable!("public route")
                        };
                        self.reply_owed = true;
                        out.push(Request::Component(jig_protocol_engine_clients::FromTransport::PublicRead {
                            exchange: self.name,
                            read,
                        }));
                        return;
                    }

                    let body = self.body.meet(Read::Fill(self.body.len())).unwrap_or_default();
                    let credential = self.credential.take();
                    let message = match crate::dispatch::message(&self.route, body) {
                        Some(message) => {
                            self.reply_owed = true;
                            jig_protocol_engine_clients::FromTransport::Exchange {
                                browser: self.cookie,
                                exchange: self.name,
                                credential,
                                message,
                            }
                        }
                        None => {
                            self.streaming = true;
                            jig_protocol_engine_clients::FromTransport::StreamOpen { stream: self.name, credential }
                        }
                    };
                    out.push(Request::Component(message));
                }
            }
            http::Event::Reply(up) if self.stream.is_some() => {
                let stream = self.stream.as_mut().expect("stream writer");
                skein_http::sse::writer::up(
                    &mut stream.writer,
                    &crate::stream::env(env),
                    up,
                    &mut stream.above,
                    &mut stream.below,
                );
            }
            http::Event::Reply(Up::Room) => self.send(&env.limits),
            http::Event::Done(_)
            | http::Event::Ended
            | http::Event::Failed(_)
            | http::Event::Refused(_)
            | http::Event::Closed
            | http::Event::Body(Up::Failed(_) | Up::Room)
            | http::Event::Reply(Up::Failed(_) | Up::End | Up::Bytes(_)) => self.finish(out),
            http::Event::Call(_) => unreachable!("server supplies one head"),
        }
    }
}
