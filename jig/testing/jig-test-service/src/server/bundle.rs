//! Bundle routing (jig/docs/design/protocol/service.md, 7 and 11.4; skein/docs/design/ui/web/bundle.md, 4).
use super::{Limits, Routed, Server};
use alloc::boxed::Box;
pub use bundle::{File, compressed_of, kind};
use jig_protocol_engine_clients_http::bundle::{self, field, target};
use skein_http::{Method, server as http};
use skein_lib::{List, Map, Queue, Token, bytes, stream::Up};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Mount,
    File,
    Gzip,
    Page,
    NotModified,
    Root,
    NotFound,
    NotAllowed,
}
#[derive(Debug)]
pub struct Bundle {
    pub page: Box<[u8]>,
    pub etag: Box<[u8]>,
    pub digest: Box<[u8]>,
    pub files: Map<Box<[u8]>, File>,
}
#[derive(Debug)]
struct Reply {
    answer: Answer,
    path: Box<[u8]>,
    offset: usize,
}
impl Bundle {
    fn file(&self, path: &[u8]) -> Option<&File> {
        bundle::file(&self.files, &self.digest, path)
    }
    fn content(&self, reply: &Reply) -> &[u8] {
        match reply.answer {
            Answer::File | Answer::Root => &self.file(&reply.path).expect("routed file").bytes,
            Answer::Gzip => self.file(&reply.path).expect("routed file").gzip.as_deref().expect("routed form"),
            Answer::Page => &self.page,
            Answer::Mount | Answer::NotModified | Answer::NotFound | Answer::NotAllowed => b"",
        }
    }
}
pub(super) fn route(bundle: Option<&Bundle>, call: &http::Call) -> Answer {
    if jig_protocol_engine_clients_http::claims(&crate::clients::http_settings(), call) {
        return Answer::Mount;
    }
    if call.method != Method::Get {
        return Answer::NotAllowed;
    }
    let Some(bundle) = bundle else { return Answer::NotFound };
    let path = target(call);
    if path.starts_with(b"/bundle/") {
        let Some(file) = bundle.file(path) else { return Answer::NotFound };
        return if file.gzip.is_some() && call.accepts_gzip() { Answer::Gzip } else { Answer::File };
    }
    if path == b"/robots.txt" && bundle.file(path).is_none() {
        return Answer::NotFound;
    }
    for header in &call.headers {
        if header.name.eq_ignore_ascii_case(b"if-none-match") && header.value == bundle.etag {
            return Answer::NotModified;
        }
    }
    if path == b"/robots.txt" { Answer::Root } else { Answer::Page }
}
#[derive(Debug)]
pub struct Pages {
    bundle: Option<Bundle>,
    replies: Map<Token, Reply>,
    send: u32,
}
impl Pages {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self { bundle: None, replies: Map::with_capacity(limits.connections), send: limits.http.send }
    }
    pub fn serve(&mut self, bundle: Bundle) {
        self.bundle = Some(bundle);
    }
    #[must_use]
    pub fn route(&self, head: &http::Call) -> Answer {
        route(self.bundle.as_ref(), head)
    }
    pub fn call(
        &mut self,
        server: &mut Server,
        call: Token,
        head: &http::Call,
        answer: Answer,
        out: &mut Queue<Routed>,
    ) {
        let bundle = self.bundle.as_ref();
        let reply = Reply { answer, path: bytes::copy_of(target(head)), offset: 0 };
        let mut headers = List::with_capacity(4);
        let mut length = 0;
        let status = match answer {
            Answer::Page | Answer::Gzip | Answer::File | Answer::Root => {
                let bundle = bundle.expect("routed bundle");
                let kind = match answer {
                    Answer::Gzip | Answer::File | Answer::Root => bundle.file(&reply.path).expect("routed file").kind,
                    Answer::Page => b"text/html; charset=utf-8",
                    Answer::Mount | Answer::NotModified | Answer::NotFound | Answer::NotAllowed => {
                        unreachable!("content answer")
                    }
                };
                field(&mut headers, b"Content-Type", kind);
                if answer == Answer::Gzip {
                    field(&mut headers, b"Content-Encoding", b"gzip");
                }
                if (answer == Answer::Gzip || answer == Answer::File)
                    && bundle.file(&reply.path).expect("routed file").gzip.is_some()
                {
                    field(&mut headers, b"Vary", b"Accept-Encoding");
                }
                length = bundle.content(&reply).len();
                200
            }
            Answer::NotModified => 304,
            Answer::NotFound => 404,
            Answer::NotAllowed => 405,
            Answer::Mount => unreachable!("mount answers itself"),
        };
        match answer {
            Answer::Gzip | Answer::File => {
                field(&mut headers, b"Cache-Control", b"public, max-age=31536000, immutable");
            }
            Answer::Page | Answer::Root | Answer::NotModified => {
                field(&mut headers, b"Cache-Control", b"no-cache");
                field(&mut headers, b"ETag", &bundle.expect("revalidated bundle").etag);
            }
            Answer::Mount | Answer::NotFound | Answer::NotAllowed => {}
        }
        let response = http::Response {
            status,
            headers: headers.into_boxed(),
            body: if status == 304 {
                http::Body::None
            } else {
                http::Body::Length(u64::try_from(length).expect("bounded file"))
            },
            close: false,
        };
        server.answer(call, http::Request::Discard, out);
        server.answer(call, http::Request::Respond(response), out);
        if length != 0 {
            bundle::demand(server, call, length, self.send, out);
        }
        self.replies.insert(call, reply).expect("bounded server calls");
    }
    fn send(&mut self, server: &mut Server, call: Token, out: &mut Queue<Routed>) {
        let reply = self.replies.get_mut(&call).expect("own reply");
        let body = self.bundle.as_ref().expect("body's bundle").content(reply);
        reply.offset = bundle::chunk(server, call, body, reply.offset, self.send, out);
    }
    #[must_use]
    pub fn owns(&self, call: Token) -> bool {
        self.replies.get(&call).is_some()
    }
    pub fn event(&mut self, server: &mut Server, call: Token, event: http::Event, out: &mut Queue<Routed>) {
        match event {
            http::Event::Reply(Up::Room) => self.send(server, call, out),
            http::Event::Done(_) | http::Event::Ended => {
                self.replies.remove(&call);
                server.done(call, out);
            }
            http::Event::Refused(_)
            | http::Event::Failed(_)
            | http::Event::Closed
            | http::Event::Reply(Up::Failed(_))
            | http::Event::Body(Up::Failed(_)) => {
                self.replies.remove(&call);
                server.abandon(call, out);
            }
            http::Event::Body(_) | http::Event::Reply(_) => {}
            http::Event::Call(_) => unreachable!("a head is routed, not an event"),
        }
    }
}
