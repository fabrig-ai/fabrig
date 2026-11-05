use crate::connection::{self, Connection, Reply, State, Turn};
use alloc::boxed::Box;
use skein_http::{Header, server as http};
use skein_lib::{
    Env, Queue, Time, Wall, bytes,
    stream::{Down, Fault, Read, Up},
};

const LIMITS: http::Limits = http::Limits { head: 256, headers: 8, body: 32, read: 8, response: 256, send: 8 };
struct Machine {
    c: Connection,
    env: Env<http::Limits>,
    below: Queue<Down>,
}
impl Machine {
    fn new() -> Self {
        Self {
            c: Connection::new(&LIMITS),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS },
            below: Queue::uncapped(),
        }
    }
    fn up(&mut self, event: Up) {
        assert!(connection::up(&mut self.c, &self.env, event, &mut self.below));
    }
    fn request(&mut self) {
        assert!(connection::request(&mut self.c, &self.env, &mut self.below));
    }
    fn turn(&mut self) -> Option<Turn> {
        connection::turn(&mut self.c, &self.env.limits)
    }
    fn ready(&mut self) {
        connection::read_next(&mut self.c);
        assert!(self.c.is_settled());
        assert!(self.c.has_work());
        self.request();
        assert_eq!(self.below.pop(), Some(Down::Demand { read: Read::Nothing, room: LIMITS.response }));
        assert!(!connection::request(&mut self.c, &self.env, &mut self.below));
        assert!(!self.c.has_work());
        self.up(Up::Room);
        assert!(self.c.is_settled(), "room alone does not accept a request");
    }
    fn head(&mut self) -> http::Call {
        self.ready();
        for line in [
            b"GET /path HTTP/1.1\r\n".as_slice(),
            b"Host: forge.test\r\n",
            b"aUtHoRiZaTiOn: Bearer example\r\n",
            b"X-Trace: kept\r\n",
            b"\r\n",
        ] {
            let Some(Down::Demand { room: 0, .. }) = self.below.pop() else {
                unreachable!("request head read");
            };
            self.up(Up::Bytes(bytes::copy_of(line)));
        }
        assert!(self.c.has_work(), "a queued event is work");
        let Some(Turn::Call(call)) = self.turn() else {
            unreachable!("request head");
        };
        assert_eq!(self.turn(), None);
        call
    }
    fn exchange(&mut self) {
        drop(self.head());
        drop(self.c.take_head());
        connection::down(
            &mut self.c,
            &self.env,
            http::Request::Body(Down::Demand { read: Read::Arrived { max: LIMITS.read }, room: 0 }),
            &mut self.below,
        );
        assert_eq!(self.turn(), Some(Turn::Body(Up::End)));
        assert!(self.below.is_empty());
    }
    fn respond(&mut self, status: u16, body: &[u8], bodyless: bool) {
        connection::respond(
            &mut self.c,
            &self.env.limits,
            Reply {
                status,
                headers: Box::new([Header { name: bytes::copy_of(b"X-Reply"), value: bytes::copy_of(b"kept") }]),
                body: bytes::copy_of(body),
                bodyless,
            },
        );
        assert_eq!(self.c.state, State::Streaming);
    }
    fn response_head(&mut self, status: u16) {
        self.request();
        let Some(Down::Send(head)) = self.below.pop() else {
            unreachable!("response head bytes");
        };
        assert!(head.starts_with(b"HTTP/1.1 "));
        assert_eq!(head.get(9..12), Some(skein_lib::Decimal::of(u64::from(status)).as_bytes()));
        assert!(bytes::find(&head, b"X-Reply: kept\r\n").is_some());
    }
}

#[test]
fn a_head_is_kept_with_its_credential_blanked() {
    let mut m = Machine::new();
    let call = m.head();
    assert_eq!(call.header(b"authorization"), Some(b"Bearer example".as_slice()));
    assert!(m.c.head_pending());
    let mut expected = call;
    for header in &mut expected.headers {
        if header.is(b"authorization") {
            header.value = Box::new([]);
        }
    }
    assert_eq!(m.c.take_head(), Some(expected));
    assert!(!m.c.head_pending());
    assert_eq!(m.c.take_head(), None);
    assert!(!m.c.has_work());
}

#[test]
fn up_is_refused_before_start_and_after_close() {
    let mut m = Machine::new();
    assert_eq!(m.c.state, State::New);
    assert!(!m.c.is_settled());
    for state in [State::New, State::Closing, State::Closed] {
        m.c.state = state;
        assert!(!connection::up(&mut m.c, &m.env, Up::Bytes(bytes::copy_of(b"ignored")), &mut m.below));
        assert_eq!(m.c.state, state);
        assert!(m.below.is_empty());
        assert!(!m.c.has_work());
    }
}

#[test]
fn the_first_bytes_turn_reading_into_receiving() {
    let mut m = Machine::new();
    m.ready();
    drop(m.below.pop().expect("first line demand"));
    m.up(Up::Bytes(bytes::copy_of(b"GET /path HTTP/1.1\r\n")));
    assert_eq!(m.c.state, State::Receiving);
    assert!(!m.c.is_settled());
    assert_eq!(m.turn(), None);
    let Some(Down::Demand { room: 0, .. }) = m.below.pop() else {
        unreachable!("next head line read");
    };
}

#[test]
fn a_reply_streams_within_the_send_limit_and_finishes() {
    let mut m = Machine::new();
    m.exchange();
    m.env.limits.send = 3;
    m.respond(200, b"abcdefg", false);
    // The initial demand reads this call's limit, rather than new's limit.
    assert_eq!(m.c.requests.len(), 2);
    m.response_head(200);
    for chunk in [b"abc".as_slice(), b"def", b"g"] {
        m.request();
        assert_eq!(
            m.below.pop(),
            Some(Down::Demand { read: Read::Nothing, room: u32::try_from(chunk.len()).expect("small chunk") })
        );
        m.up(Up::Room);
        assert_eq!(m.turn(), Some(Turn::Handled));
        m.request();
        assert_eq!(m.below.pop(), Some(Down::Send(bytes::copy_of(chunk))));
    }
    m.request();
    assert_eq!(m.turn(), Some(Turn::Done(http::Reuse::Keep)));
    assert!(!m.c.has_work());
    assert!(m.below.is_empty());
    assert!(m.c.reply.is_none());
    connection::read_next(&mut m.c);
    assert!(m.c.is_settled());
}

#[test]
fn an_empty_body_finishes_and_a_bodyless_status_does_not() {
    for (status, bodyless) in [(200, false), (204, true), (304, true)] {
        let mut m = Machine::new();
        m.exchange();
        m.respond(status, b"", bodyless);
        let Some(http::Request::Respond(response)) = m.c.requests.pop() else {
            unreachable!("response first");
        };
        assert_eq!(response.body, if bodyless { http::Body::None } else { http::Body::Length(0) });
        assert!(!response.close);
        connection::down(&mut m.c, &m.env, http::Request::Respond(response), &mut m.below);
        if !bodyless {
            assert_eq!(m.c.requests.pop(), Some(http::Request::Reply(Down::Finish)));
            connection::down(&mut m.c, &m.env, http::Request::Reply(Down::Finish), &mut m.below);
        }
        assert!(m.c.requests.is_empty(), "bodyless responses have no reply stream");
        assert_eq!(m.turn(), Some(Turn::Done(http::Reuse::Keep)));
        assert!(m.c.reply.is_none());
    }
}

#[test]
fn a_failed_reply_stream_is_a_failed_turn() {
    let mut m = Machine::new();
    m.exchange();
    m.respond(200, b"body", false);
    m.response_head(200);
    m.request();
    drop(m.below.pop().expect("body room demand"));
    m.up(Up::Failed(Fault::Other));
    assert_eq!(m.turn(), Some(Turn::Failed), "reply stream failure");
    assert_eq!(m.turn(), Some(Turn::Failed), "HTTP exchange failure");
    assert_eq!(m.turn(), None);
    for event in [
        http::Event::Reply(Up::Bytes(bytes::copy_of(b"invalid"))),
        http::Event::Reply(Up::End),
        http::Event::Refused(http::Refusal::TooLong),
        http::Event::Ended,
    ] {
        m.c.events.push(event);
        assert_eq!(m.turn(), Some(Turn::Failed));
    }
}

#[test]
fn close_drops_the_reply_and_both_queues() {
    let mut m = Machine::new();
    m.exchange();
    m.respond(200, b"pending", false);
    assert!(m.c.reply.is_some());
    m.c.events.push(http::Event::Body(Up::End));
    connection::close(&mut m.c, &m.env, &mut m.below);
    assert_eq!(m.c.state, State::Closing);
    assert!(m.c.reply.is_none());
    assert!(m.c.requests.is_empty());
    assert_eq!(m.c.events.len(), 1, "only HTTP's close acknowledgement survives");
    assert_eq!(m.turn(), Some(Turn::Handled));
    assert!(!m.c.has_work());
    connection::close(&mut m.c, &m.env, &mut m.below);
    assert_eq!(m.turn(), None, "close is idempotent");
    m.c.state = State::Closed;
    connection::close(&mut m.c, &m.env, &mut m.below);
    assert_eq!(m.c.state, State::Closed);
    assert!(m.below.is_empty());
}
