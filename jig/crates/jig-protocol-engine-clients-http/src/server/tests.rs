//! Idle connections yield their room without dropping a head already received (service.md, 11.4).
use super::*;

const FIRST: Token = Token::new(71);
const SECOND: Token = Token::new(72);
const THIRD: Token = Token::new(73);

fn address() -> io::Addr {
    io::Addr::Ip(SocketAddr::from(([127, 0, 0, 1], 7100)))
}
fn accepted(server: &mut Server, socket: Token, out: &mut Queue<Routed>) {
    server.up(io::Event::Accepted { owner: LISTENER, socket, peer: address() }, out);
}
fn started() -> (Server, Token, Queue<Routed>) {
    let limits = Limits {
        connections: 1,
        http: http::Limits { head: 4096, headers: 32, body: 4096, read: 4096, response: 4096, send: 4096 },
        stall: Duration::from_secs(30),
        peer_end: PeerEnd::Connection,
    };
    let mut server = Server::new(address(), &limits);
    let mut out = Queue::uncapped();
    assert!(server.advance(&mut out));
    out.clear();
    server.up(io::Event::Listening { owner: LISTENER, listener: Token::new(70), addr: address() }, &mut out);
    accepted(&mut server, FIRST, &mut out);
    let Some(Routed::Io(io::Request::Bind { owner, socket: FIRST })) = out.pop() else { panic!("first bind") };
    settle(&mut server, owner, &mut out);
    (server, owner, out)
}
fn settle(server: &mut Server, owner: Token, out: &mut Queue<Routed>) {
    while server.advance(out) {
        while let Some(event) = out.pop() {
            if let Routed::Io(io::Request::Stream { down: Down::Demand { room, .. }, .. }) = event
                && room > 0
            {
                server.up(io::Event::Stream { owner, up: Up::Room }, out);
            }
        }
    }
}
fn yielded(server: &mut Server, out: &mut Queue<Routed>) {
    accepted(server, SECOND, out);
    let Some(Routed::Io(io::Request::Close { entity })) = out.pop() else {
        panic!("idle socket closes to admit a fresh client")
    };
    assert_eq!(entity, FIRST);
    assert!(out.is_empty(), "new socket waits for the idle socket's terminal event: {out:?}");
}

fn rejected(out: &mut Queue<Routed>, expected: Token) {
    let Some(Routed::Io(io::Request::Reject { socket })) = out.pop() else {
        panic!("received work is retained and excess arrivals are rejected")
    };
    assert_eq!(socket, expected);
}

#[test]
fn an_idle_connection_yields_its_room_to_one_fresh_client() {
    let (mut server, owner, mut out) = started();
    yielded(&mut server, &mut out);
    accepted(&mut server, THIRD, &mut out);
    rejected(&mut out, THIRD);
    server.up(io::Event::Closed { owner }, &mut out);
    let Some(Routed::Io(io::Request::Bind { socket, .. })) = out.pop() else {
        panic!("fresh client binds after room is reclaimed")
    };
    assert_eq!(socket, SECOND);
    assert_eq!(server.connections.len(), 1, "connection capacity stays one");
    assert!(out.is_empty());
}

#[test]
fn a_partial_head_and_an_active_call_keep_their_connections() {
    for head in [b"GET / HTTP/1.1\r\n".as_slice(), b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"] {
        let (mut server, owner, mut out) = started();
        for line in head.split_inclusive(|byte| *byte == b'\n') {
            server.up(io::Event::Stream { owner, up: Up::Bytes(line.into()) }, &mut out);
            settle(&mut server, owner, &mut out);
        }
        accepted(&mut server, SECOND, &mut out);
        rejected(&mut out, SECOND);
        assert!(out.is_empty());
    }
}

#[test]
fn stopping_rejects_the_socket_waiting_for_an_idle_connection_to_close() {
    let (mut server, owner, mut out) = started();
    yielded(&mut server, &mut out);
    server.stop(&mut out);
    rejected(&mut out, SECOND);
    out.clear();
    server.up(io::Event::Closed { owner }, &mut out);
    assert!(out.is_empty(), "stopping never binds the waiting socket");
}

#[test]
fn a_reused_connection_yields_after_its_response() {
    let (mut server, owner, mut out) = started();
    for line in [b"GET / HTTP/1.1\r\n".as_slice(), b"Host: localhost\r\n", b"\r\n"] {
        server.up(io::Event::Stream { owner, up: Up::Bytes(line.into()) }, &mut out);
        settle(&mut server, owner, &mut out);
    }
    let call = server.connections.get(&owner).unwrap().call.expect("active call");
    server.answer(call, http::Request::Discard, &mut out);
    settle(&mut server, owner, &mut out);
    server.answer(
        call,
        http::Request::Respond(http::Response {
            status: 204,
            headers: Box::new([]),
            body: http::Body::None,
            close: false,
        }),
        &mut out,
    );
    settle(&mut server, owner, &mut out);
    assert_eq!(server.connections.get(&owner).unwrap().reuse, Some(http::Reuse::Keep));
    server.done(call, &mut out);
    settle(&mut server, owner, &mut out);
    yielded(&mut server, &mut out);
}
