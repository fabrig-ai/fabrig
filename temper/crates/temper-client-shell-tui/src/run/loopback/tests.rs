//! A browser's lines and the run's effects at the skein-io seam, without sockets.
use super::*;
use skein_lib::{
    Duration,
    stream::{Delimiter, Down, Read, Up},
};

const PORT: u16 = 43210;
const BOUND: Token = Token::new(1);
const NOW: Time = Time::from_nanos(100);
const WALL: Wall = Wall::EPOCH;
const HOST: &[u8] = b"127.0.0.1:43210";
const TARGET: &[u8] = b"/approved?grant=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn addr(port: u16) -> io::Addr {
    io::Addr::Ip(([127, 0, 0, 1], port).into())
}

fn tell(rig: &mut Loopback, at: Time, event: io::Event) {
    assert!(rig.owns(&event), "event routed to the listener: {event:?}");
    rig.event(at, WALL, event);
}

fn listen_request(rig: &mut Loopback) {
    assert_eq!(rig.requests.pop(), Some(io::Request::Listen { owner: LISTENER, addr: addr(0) }));
    assert!(rig.requests.is_empty());
}

fn listening(rig: &mut Loopback) {
    tell(rig, NOW, io::Event::Listening { owner: LISTENER, listener: BOUND, addr: addr(PORT) });
}

fn open() -> Loopback {
    let mut rig = Loopback::new();
    assert!(rig.listen(true).is_none());
    listen_request(&mut rig);
    listening(&mut rig);
    assert!(matches!(rig.reports.pop(), Some((None, host::Report::Listening { port: Some(PORT) }))));
    rig
}

fn accepted(rig: &mut Loopback, socket: Token) {
    tell(rig, NOW, io::Event::Accepted { owner: LISTENER, socket, peer: addr(52914) });
}

fn accept(rig: &mut Loopback, socket: Token) -> Token {
    accepted(rig, socket);
    let Some(io::Request::Bind { socket: bound, owner }) = rig.requests.pop() else {
        panic!("bind the accepted socket")
    };
    assert_eq!(bound, socket);
    assert_eq!(
        rig.requests.pop(),
        Some(io::Request::Stream {
            stream: socket,
            down: Down::Demand { read: Read::Nothing, room: limits::RETURN_HEAD },
        })
    );
    owner
}

fn up(rig: &mut Loopback, now: Time, owner: Token, up: Up) {
    tell(rig, now, io::Event::Stream { owner, up });
}

fn scan(rig: &mut Loopback, socket: Token, max: u32) {
    assert_eq!(
        rig.requests.pop(),
        Some(io::Request::Stream {
            stream: socket,
            down: Down::Demand { read: Read::Scan { until: Delimiter::LF, max }, room: 0 },
        })
    );
}

fn begin(rig: &mut Loopback, socket: Token) -> Token {
    let owner = accept(rig, socket);
    up(rig, NOW, owner, Up::Room);
    scan(rig, socket, limits::RETURN_HEAD);
    owner
}

fn line(rig: &mut Loopback, owner: Token, bytes: &[u8]) {
    up(rig, NOW, owner, Up::Bytes(bytes.into()));
}

fn returned(rig: &mut Loopback, owner: Token, get: bool, target: &[u8], host: Option<&[u8]>) {
    let Some((Some(answer_to), host::Report::Returned(returned))) = rig.reports.pop() else {
        panic!("one returned head")
    };
    assert_eq!(answer_to, owner);
    assert_eq!(returned.get, get);
    assert_eq!(returned.target.as_ref(), target);
    assert_eq!(returned.host.as_deref(), host);
    assert!(rig.reports.is_empty());
}

fn assert_closed_with_wire(rig: &mut Loopback, socket: Token, wire: &[u8]) {
    assert_eq!(rig.requests.pop(), Some(io::Request::Stream { stream: socket, down: Down::Send(wire.into()) }));
    assert_eq!(rig.requests.pop(), Some(io::Request::Close { entity: socket }));
    assert!(rig.requests.is_empty(), "exactly one response, then close");
}

#[test]
fn without_an_opener_or_after_a_failed_bind_there_is_no_port() {
    let mut rig = Loopback::new();
    assert!(matches!(rig.listen(false), Some(host::Report::Listening { port: None })));
    assert!(rig.requests.is_empty(), "without an opener the duty does not try to listen");
    assert!(rig.reports.is_empty(), "no asynchronous report");
    assert!(rig.listen(true).is_none());
    listen_request(&mut rig);
    tell(&mut rig, NOW, io::Event::Failed { owner: LISTENER, error: io::Error::Other });
    tell(&mut rig, NOW, io::Event::Closed { owner: LISTENER });
    assert!(matches!(rig.reports.pop(), Some((None, host::Report::Listening { port: None }))));
    assert!(rig.reports.is_empty());
    assert!(rig.requests.is_empty(), "a failed listen does not retry on its own");
}

#[test]
fn an_answer_flushes_and_closes_even_when_the_same_turn_unlistens() {
    const DATE: &[u8] = b"Date: Thu, 01 Jan 1970 00:00:00 GMT\r\n";
    const PRIVACY: &[u8] = b"Referrer-Policy: no-referrer\r\nCache-Control: no-store\r\n";
    const END: &[u8] = b"Content-Length: 0\r\nConnection: close\r\n\r\n";
    const TO: &[u8] = b"https://engine.example/terminal?code=ABCD-EFGH";
    for (index, (answer, status, location)) in [
        (
            Some(Answer::Redirect { to: TO.into() }),
            b"HTTP/1.1 303 See Other\r\n".as_slice(),
            [b"Location: ".as_slice(), TO, b"\r\n"].concat(),
        ),
        (Some(Answer::NotFound), b"HTTP/1.1 404 Not Found\r\n", Vec::new()),
        (None, b"HTTP/1.1 404 Not Found\r\n", Vec::new()),
    ]
    .into_iter()
    .enumerate()
    {
        let mut rig = open();
        let socket = Token::new(2);
        let owner = begin(&mut rig, socket);
        let idle = Token::new(3);
        let _idle = accept(&mut rig, idle);
        let (first, host) = if index == 1 {
            (b"POST /other HTTP/1.0\r\n".to_vec(), None)
        } else {
            ([b"GET ".as_slice(), TARGET, b" HTTP/1.1\r\n"].concat(), Some(HOST))
        };
        let mut left = limits::RETURN_HEAD - u32::try_from(first.len()).expect("small fixture");
        line(&mut rig, owner, &first);
        scan(&mut rig, socket, left);
        if let Some(host) = host {
            let field = [b"hOsT: ".as_slice(), host, b"\r\n"].concat();
            line(&mut rig, owner, &field);
            left -= u32::try_from(field.len()).expect("small fixture");
            scan(&mut rig, socket, left);
        }
        // No request body is demanded even when the browser announces one.
        let length = b"Content-Length: 40\r\n";
        line(&mut rig, owner, length);
        scan(&mut rig, socket, left - u32::try_from(length.len()).expect("small fixture"));
        line(&mut rig, owner, b"\r\n");
        returned(&mut rig, owner, index != 1, if index == 1 { b"/other" } else { TARGET }, host);
        assert!(rig.requests.is_empty(), "not answered before the host's duty");
        assert_eq!(rig.deadline(), Some(NOW.saturating_add(limits::RETURN_WAIT)), "only the unread head still waits");
        rig.unlisten(NOW, WALL);
        assert_eq!(rig.requests.pop(), Some(io::Request::Abort { entity: BOUND }));
        assert_eq!(
            rig.requests.pop(),
            Some(io::Request::Stream { stream: idle, down: Down::Demand { read: Read::Nothing, room: 0 } })
        );
        assert_eq!(rig.requests.pop(), Some(io::Request::Abort { entity: idle }));
        assert_eq!(rig.deadline(), None);
        rig.answer(NOW, WALL, owner, answer);
        assert_closed_with_wire(&mut rig, socket, &[status, DATE, PRIVACY, &location, END].concat());
    }
}

#[test]
fn four_open_returns_leave_the_fifth_rejected_until_one_closes() {
    let mut rig = open();
    let owners: Vec<_> = (2..6).map(|raw| accept(&mut rig, Token::new(raw))).collect();
    assert_eq!(owners.len(), 4);
    let fifth = Token::new(6);
    accepted(&mut rig, fifth);
    assert_eq!(rig.requests.pop(), Some(io::Request::Reject { socket: fifth }));
    assert!(rig.requests.is_empty());
    let owner = *owners.first().expect("first connection");
    tell(&mut rig, NOW, io::Event::Closed { owner });
    let new_owner = accept(&mut rig, Token::new(7));
    assert!(!owners.contains(&new_owner));
}

#[test]
fn a_missing_or_late_head_is_aborted_without_an_answer() {
    let mut rig = open();
    let silent = Token::new(2);
    let sleepy = Token::new(3);
    let _silent = accept(&mut rig, silent);
    let owner = begin(&mut rig, sleepy);
    line(&mut rig, owner, b"GET /other HTTP/1.0\r\n");
    scan(&mut rig, sleepy, limits::RETURN_HEAD - 21);
    let at = NOW.saturating_add(Duration::from_secs(10));
    assert_eq!(rig.deadline(), Some(at));
    rig.fire(Time::from_nanos(at.as_nanos() - 1), WALL);
    assert!(rig.requests.is_empty());
    up(&mut rig, at, owner, Up::Bytes(b"\r\n".as_slice().into()));
    rig.fire(at, WALL);
    for socket in [sleepy, silent] {
        assert_eq!(
            rig.requests.pop(),
            Some(io::Request::Stream { stream: socket, down: Down::Demand { read: Read::Nothing, room: 0 } })
        );
        assert_eq!(rig.requests.pop(), Some(io::Request::Abort { entity: socket }));
    }
    assert!(rig.requests.is_empty());
    assert!(rig.reports.is_empty());
    assert_eq!(rig.deadline(), None);
}

#[test]
fn the_head_limit_admits_the_last_byte_but_never_reports_an_overlong_head() {
    const START: &[u8] = b"GET / HTTP/1.0\r\n";
    const HEAD: u32 = 16 * 1024;
    for over in [false, true] {
        let mut rig = open();
        let socket = Token::new(2);
        let owner = accept(&mut rig, socket);
        up(&mut rig, NOW, owner, Up::Room);
        scan(&mut rig, socket, HEAD);
        line(&mut rig, owner, START);
        let left = HEAD - u32::try_from(START.len()).expect("small fixture");
        scan(&mut rig, socket, left);
        let mut field = b"X-Fill: ".to_vec();
        let tail = 2 - u32::from(over);
        field.resize(usize::try_from(left - tail - 2).expect("field size"), b'x');
        field.extend_from_slice(b"\r\n");
        line(&mut rig, owner, &field);
        scan(&mut rig, socket, tail);
        // The io scan of CRLF at max=1 delivers only CR.
        line(&mut rig, owner, if over { b"\r" } else { b"\r\n" });
        if over {
            assert!(rig.reports.is_empty());
            let Some(io::Request::Stream { stream, down: Down::Send(bytes) }) = rig.requests.pop() else {
                panic!("skein-http's refusal goes to the wire")
            };
            assert_eq!(stream, socket);
            assert!(bytes.starts_with(b"HTTP/1.1 431 "));
            assert_eq!(rig.requests.pop(), Some(io::Request::Close { entity: socket }));
        } else {
            returned(&mut rig, owner, true, b"/", None);
        }
        assert!(rig.requests.is_empty());
    }
}

#[test]
fn restarting_does_not_reuse_a_cancelled_bind_or_accept() {
    let mut rig = Loopback::new();
    assert!(rig.listen(true).is_none());
    listen_request(&mut rig);
    rig.unlisten(NOW, WALL);
    assert!(rig.listen(true).is_none());
    assert!(rig.requests.is_empty(), "wait for the old listener's token");
    listening(&mut rig);
    assert_eq!(rig.requests.pop(), Some(io::Request::Abort { entity: BOUND }));
    let socket = Token::new(2);
    accepted(&mut rig, socket);
    assert_eq!(rig.requests.pop(), Some(io::Request::Reject { socket }));
    tell(&mut rig, NOW, io::Event::Failed { owner: LISTENER, error: io::Error::Other });
    assert!(rig.reports.is_empty(), "the cancelled listener's events are not reported");
    tell(&mut rig, NOW, io::Event::Closed { owner: LISTENER });
    listen_request(&mut rig);
    listening(&mut rig);
    assert!(matches!(rig.reports.pop(), Some((None, host::Report::Listening { port: Some(PORT) }))));
}
