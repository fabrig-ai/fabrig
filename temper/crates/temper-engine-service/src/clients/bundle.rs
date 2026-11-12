//! The browser's bundle, read whole at start, and its closed list of types (client/web/README.md, 2.2; skein's
//! ui/web/bundle.md, 3 and 4), composing jig's bundle helpers (jig/docs/design/protocol/service.md, 11.4).
use super::{Clients, policy};
use alloc::boxed::Box;
use jig_protocol_engine_clients_http::bundle::{self, field, target};
pub use jig_protocol_engine_clients_http::bundle::{File, compressed_of, kind};
use skein_http::{Method, server as http};
use skein_lib::{List, Map, Token, bytes, stream::Up};

/// Temper's fixed explanation when configuration names no browser bundle (client/web/README.md, 2.2).
#[expect(clippy::disallowed_macros, reason = "a fixed page compiled in, not data read at run time")]
pub const NO_BUNDLE_PAGE: &[u8] = include_bytes!("no-bundle.html");

/// A bundle path's bound (jig's testing/jig-test/src/bundle.rs).
pub const NAME_BYTES: u32 = 1024;
/// The bundle and its kept forges' metas (client/web/README.md, 2.2; protocol/openid-provider.md, 7).
#[derive(Debug)]
pub struct Bundle {
    pub page: Box<[u8]>,
    /// Bytes of forge metas immediately before the page's `</head>`.
    pub forge_meta_bytes: usize,
    pub etag: Box<[u8]>,
    pub digest: Box<[u8]>,
    pub files: Map<Box<[u8]>, File>,
}
/// How the clients' listener answers a call the mapping does not claim (client/web/README.md, 2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    File,
    Gzip,
    Page,
    NoBundle,
    NotModified,
    Root,
    NotFound,
    NotAllowed,
    Busy,
}
/// An answer on its way: what it is, the path it serves and how much of its body is sent.
#[derive(Debug)]
pub(crate) struct Reply {
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
            Answer::NoBundle => NO_BUNDLE_PAGE,
            Answer::NotModified | Answer::NotFound | Answer::NotAllowed | Answer::Busy => b"",
        }
    }
}
/// An entity tag without its weakness mark and the blanks around it.
fn strong(tag: &[u8]) -> &[u8] {
    let tag = tag.trim_ascii();
    tag.strip_prefix(b"W/").unwrap_or(tag)
}
/// Whether an `If-None-Match` field holds the page's tag: `*`, or a comma-separated list with a member equal to it,
/// compared weakly (RFC 9110, 13.1.2).
fn revalidates(field: &[u8], etag: &[u8]) -> bool {
    if field.trim_ascii() == b"*" {
        return true;
    }
    let mut rest = field;
    loop {
        let (member, next) = match bytes::find(rest, b",") {
            Some(at) => (rest.get(..at).expect("comma position"), rest.get(at.saturating_add(1)..)),
            None => (rest, None),
        };
        if strong(member) == strong(etag) {
            return true;
        }
        match next {
            Some(next) => rest = next,
            None => return false,
        }
    }
}
/// The answer to a call the mapping does not claim, in the design's order.
pub(crate) fn route(bundle: Option<&Bundle>, head: &http::Call) -> Answer {
    let Some(bundle) = bundle else {
        return if head.method == Method::Get && !target(head).starts_with(b"/bundle/") && target(head) != b"/robots.txt"
        {
            Answer::NoBundle
        } else {
            Answer::NotFound
        };
    };
    if head.method != Method::Get {
        return Answer::NotAllowed;
    }
    let path = target(head);
    if path.starts_with(b"/bundle/") {
        let Some(file) = bundle.file(path) else { return Answer::NotFound };
        return if file.gzip.is_some() && head.accepts_gzip() { Answer::Gzip } else { Answer::File };
    }
    if path == b"/robots.txt" && bundle.file(path).is_none() {
        return Answer::NotFound;
    }
    if let Some(field) = head.header(b"If-None-Match")
        && revalidates(field, &bundle.etag)
    {
        return Answer::NotModified;
    }
    if path == b"/robots.txt" { Answer::Root } else { Answer::Page }
}
impl Clients {
    /// Answers a call the mapping does not claim, and holds the answer until it is sent.
    pub(super) fn serve_call(&mut self, call: Token, head: &http::Call, busy: bool) {
        let answer = if busy { Answer::Busy } else { route(self.bundle.as_ref(), head) };
        let reply = Reply { answer, path: bytes::copy_of(target(head)), offset: 0 };
        let mut headers = List::with_capacity(4);
        let mut length = 0;
        let status = match answer {
            Answer::Page | Answer::Gzip | Answer::File | Answer::Root => {
                let bundle = self.bundle.as_ref().expect("routed bundle");
                let kind = match answer {
                    Answer::Gzip | Answer::File | Answer::Root => bundle.file(&reply.path).expect("routed file").kind,
                    Answer::Page => b"text/html; charset=utf-8",
                    Answer::NoBundle => unreachable!("handled without a bundle"),
                    Answer::NotModified | Answer::NotFound | Answer::NotAllowed | Answer::Busy => {
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
            Answer::NoBundle => {
                field(&mut headers, b"Content-Type", b"text/html; charset=utf-8");
                length = NO_BUNDLE_PAGE.len();
                200
            }
            Answer::NotModified => 304,
            Answer::NotFound => 404,
            Answer::NotAllowed => 405,
            Answer::Busy => 503,
        };
        match answer {
            Answer::Gzip | Answer::File => field(&mut headers, b"Cache-Control", policy::FILES_CACHE),
            Answer::Page | Answer::Root | Answer::NotModified => {
                field(&mut headers, b"Cache-Control", policy::PAGE_CACHE);
                field(&mut headers, b"ETag", &self.bundle.as_ref().expect("revalidated bundle").etag);
            }
            Answer::NoBundle => field(&mut headers, b"Cache-Control", policy::PAGE_CACHE),
            Answer::NotFound | Answer::NotAllowed | Answer::Busy => {}
        }
        let response = http::Response {
            status,
            headers: headers.into_boxed(),
            body: if status == 304 || busy {
                http::Body::None
            } else {
                http::Body::Length(u64::try_from(length).expect("bounded file"))
            },
            close: busy,
        };
        let response = if busy { self.mount.refusal(head) } else { response };
        self.server.answer(call, http::Request::Discard, &mut self.server_out);
        self.server.answer(call, http::Request::Respond(response), &mut self.server_out);
        if length != 0 {
            bundle::demand(&mut self.server, call, length, self.send, &mut self.server_out);
        }
        self.replies.insert(call, reply).expect("bounded server calls");
    }
    /// Takes one event of a call being answered: room for a chunk, or the answer's end.
    pub(super) fn serve_event(&mut self, call: Token, event: http::Event) {
        if self.replies.get(&call).expect("own reply").answer == Answer::Busy {
            match event {
                http::Event::Done(_) | http::Event::Failed(_) | http::Event::Closed => {
                    self.replies.remove(&call);
                    self.server.done(call, &mut self.server_out);
                }
                http::Event::Ended | http::Event::Refused(_) | http::Event::Reply(_) | http::Event::Body(_) => {}
                http::Event::Call(_) => unreachable!("a head is routed, not an event"),
            }
            return;
        }
        match event {
            http::Event::Reply(Up::Room) => self.chunk(call),
            http::Event::Done(_)
            | http::Event::Ended
            | http::Event::Refused(_)
            | http::Event::Failed(_)
            | http::Event::Closed
            | http::Event::Reply(Up::Failed(_))
            | http::Event::Body(Up::Failed(_)) => {
                self.replies.remove(&call);
                self.publish_forge_metas();
                self.server.done(call, &mut self.server_out);
            }
            http::Event::Body(_) | http::Event::Reply(_) => {}
            http::Event::Call(_) => unreachable!("a head is routed, not an event"),
        }
    }
    /// A page already being sent keeps its body until every page reply ends.
    pub(super) fn publish_forge_metas(&mut self) {
        for (_, reply) in &self.replies {
            if reply.answer == Answer::Page {
                return;
            }
        }
        if let Some(metas) = self.pending_forge_metas.take()
            && let Some(bundle) = &mut self.bundle
        {
            bundle.set_forge_metas(&metas).expect("the prepared page has a head and the metas fit");
        }
    }
    fn chunk(&mut self, call: Token) {
        let reply = self.replies.get_mut(&call).expect("own reply");
        let body = match reply.answer {
            Answer::NoBundle => NO_BUNDLE_PAGE,
            Answer::Gzip
            | Answer::File
            | Answer::Page
            | Answer::Root
            | Answer::NotModified
            | Answer::NotFound
            | Answer::NotAllowed
            | Answer::Busy => self.bundle.as_ref().expect("body's bundle").content(reply),
        };
        reply.offset = bundle::chunk(&mut self.server, call, body, reply.offset, self.send, &mut self.server_out);
    }
}

impl Bundle {
    /// Fill the page's sign-in ways before computing its entity tag (protocol/sign-in.md, 7).
    pub fn fill_sign_in(&mut self, google: bool) -> Option<()> {
        let meta = if google {
            b"<meta name=\"temper-sign-in\" content=\"google\">".as_slice()
        } else {
            b"<meta name=\"temper-sign-in\" content=\"\">".as_slice()
        };
        self.insert_meta(meta)
    }
    fn insert_meta(&mut self, meta: &[u8]) -> Option<()> {
        let at = bytes::find(&self.page, b"</head>")?;
        self.replace_metas(at, at, meta)
    }
    /// Set the page's forge metas and entity tag (protocol/openid-provider.md, 7).
    pub(crate) fn set_forge_metas(&mut self, metas: &[u8]) -> Option<()> {
        let end = bytes::find(&self.page, b"</head>")?;
        let at = end.checked_sub(self.forge_meta_bytes)?;
        if self.page.get(at..end)? == metas {
            return Some(());
        }
        self.replace_metas(at, end, metas)?;
        self.forge_meta_bytes = metas.len();
        Some(())
    }
    fn replace_metas(&mut self, at: usize, end: usize, meta: &[u8]) -> Option<()> {
        const HEX: &[u8] = b"0123456789abcdef";
        let length = self.page.len().checked_sub(end.checked_sub(at)?)?.checked_add(meta.len())?;
        let mut out = skein_lib::Writer::new(length);
        out.put(self.page.get(..at)?).ok()?;
        out.put(meta).ok()?;
        out.put(self.page.get(end..)?).ok()?;
        self.page = out.finish();
        // The bundle reader uses FNV-1a, quoted as sixteen lower-case hex digits.
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &self.page {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        let mut tag = skein_lib::Writer::new(18);
        tag.put(b"\"").ok()?;
        for shift in (0_u32..16).rev() {
            let nibble = usize::try_from((hash >> shift.checked_mul(4)?) & 15).ok()?;
            tag.put(core::slice::from_ref(HEX.get(nibble)?)).ok()?;
        }
        tag.put(b"\"").ok()?;
        self.etag = tag.finish();
        Some(())
    }
}
