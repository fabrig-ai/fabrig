//! Bounded heads parsed by skein-http (temper/docs/design/protocol/sign-in.md, 2.4).
use skein_http::{client, server};
use skein_lib::{Env, Queue, Time, Wall, stream::Up};

pub(super) const HEAD: u32 = 16_384;
pub(super) const MESSAGE: usize = 262_144;

fn head_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|part| part == b"\r\n\r\n").map(|at| at + 4)
}

pub(super) fn request(bytes: &[u8]) -> Option<(server::Call, usize)> {
    let end = head_end(bytes)?;
    let limits = server::Limits {
        head: HEAD,
        headers: 64,
        body: u64::try_from(MESSAGE).expect("message bound"),
        read: HEAD,
        response: HEAD,
        send: HEAD,
    };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut parser = server::Server::new(&limits);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    server::down(&mut parser, &env, server::Request::Next, &mut above, &mut below);
    below.clear();
    server::up(&mut parser, &env, Up::Room, &mut above, &mut below);
    for line in bytes[..end].split_inclusive(|byte| *byte == b'\n') {
        below.clear();
        server::up(&mut parser, &env, Up::Bytes(line.into()), &mut above, &mut below);
    }
    match above.pop().expect("skein-http answers a complete request head") {
        server::Event::Call(call) => {
            let length = match call.body {
                server::Body::None => 0,
                server::Body::Length(n) => usize::try_from(n).expect("bounded body"),
                server::Body::Chunked => panic!("world requests use Content-Length"),
            };
            (bytes.len() >= end + length).then_some((call, end + length))
        }
        event @ (server::Event::Ended
        | server::Event::Body(_)
        | server::Event::Reply(_)
        | server::Event::Refused(_)
        | server::Event::Done(_)
        | server::Event::Failed(_)
        | server::Event::Closed) => panic!("world request rejected: {event:?}"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Answer {
    pub status: u16,
    pub headers: Box<[skein_http::Header]>,
    pub body: Box<[u8]>,
}
impl Answer {
    pub fn header(&self, name: &[u8]) -> Option<&[u8]> {
        self.headers.iter().find(|h| h.is(name)).map(|h| h.value.as_ref())
    }
    pub fn wire(&self) -> Box<[u8]> {
        let mut bytes = format!("HTTP/1.1 {}\r\n", self.status).into_bytes();
        for h in &self.headers {
            bytes.extend_from_slice(&h.name);
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(&h.value);
            bytes.extend_from_slice(b"\r\n");
        }
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(&self.body);
        bytes.into()
    }
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            headers: vec![
                header(b"Content-Length", b"0"),
                header(b"Cache-Control", b"no-store"),
                header(b"X-Content-Type-Options", b"nosniff"),
            ]
            .into(),
            body: Box::default(),
        }
    }
}
pub(super) fn header(name: &[u8], value: &[u8]) -> skein_http::Header {
    skein_http::Header { name: name.into(), value: value.into() }
}

pub(super) fn answer(bytes: &[u8], method: skein_http::Method) -> Option<Answer> {
    let end = head_end(bytes)?;
    let limits = client::Limits { request: HEAD, head: HEAD, headers: 64, read: HEAD, send: HEAD };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut parser = client::Client::new(&limits);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    client::down(
        &mut parser,
        &env,
        client::Request::Call(client::Call {
            method,
            target: b"/".as_slice().into(),
            headers: Box::new([header(b"Host", b"world")]),
            body: client::Body::None,
            close: false,
        }),
        &mut above,
        &mut below,
    );
    below.clear();
    client::up(&mut parser, &env, Up::Room, &mut above, &mut below);
    below.clear();
    for line in bytes[..end].split_inclusive(|byte| *byte == b'\n') {
        client::up(&mut parser, &env, Up::Bytes(line.into()), &mut above, &mut below);
        below.clear();
    }
    let response = match above.pop().expect("skein-http answers a complete response head") {
        client::Event::Response(response) => response,
        event @ (client::Event::Upload(_)
        | client::Event::Body(_)
        | client::Event::Done(_)
        | client::Event::Failed(_)
        | client::Event::Closed) => panic!("world response rejected: {event:?}"),
    };
    let length = match response.framing {
        client::Framing::Empty => 0,
        client::Framing::Length(n) => usize::try_from(n).expect("bounded answer"),
        client::Framing::Chunked | client::Framing::UntilEnd => panic!("world answers use Content-Length"),
    };
    assert!(end + length <= MESSAGE, "bounded world answer");
    (bytes.len() >= end + length).then(|| Answer {
        status: response.status,
        headers: response.headers,
        body: bytes[end..end + length].into(),
    })
}
