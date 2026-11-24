//! Socket shutdown witnesses; the upstream is loopback plaintext.
use crate::support;
use skein_lib::Duration;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use support::relay::Relay;
use support::relay_fixture::guard;
use support::socket::Socket;
fn head(socket: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut b = [0];
        socket.read_exact(&mut b).expect("stub head");
        bytes.push(b[0]);
    }
    bytes
}
fn answer(socket: &mut impl Read) -> Vec<u8> {
    let bytes = head(socket);
    let text = std::str::from_utf8(&bytes).expect("head");
    let length: usize =
        text.lines().find_map(|line| line.strip_prefix("Content-Length: ")).expect("length").parse().expect("length");
    let mut body = vec![0; length];
    socket.read_exact(&mut body).expect("answer");
    body
}
#[test]
#[expect(clippy::disallowed_methods, reason = "native relay quiesce with an owned loopback socket thread")]
fn quiesce_joins_with_a_git_client_still_open() {
    let scratch = support::Scratch::new("relay-quiesce");
    let upstream = TcpListener::bind("127.0.0.1:0").expect("stub");
    let mut relay = Relay::plaintext(guard(), scratch.path(), upstream.local_addr().expect("stub address"));
    let stub = thread::spawn(move || {
        let (socket, _) = upstream.accept().expect("stub accepts");
        let mut socket = Socket::reading(socket, Duration::from_secs(2), "the offline relay stub's request");
        head(&mut socket);
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\ngit").expect("stub response");
    });
    let git = TcpStream::connect(relay.git).expect("git client");
    let mut git = Socket::reading(git, Duration::from_secs(2), "the offline relay git client's reply");
    git.write_all(b"GET /owner/probe.git/info/refs?service=git-upload-pack HTTP/1.1\r\nHost: local\r\n\r\n")
        .expect("git request");
    assert_eq!(answer(&mut git), b"git");
    stub.join().expect("stub joined");
    relay.quiesce();
    assert_eq!(relay.refused(), [] as [support::relay::Row; 0]);
    drop(git);
    drop(relay);
    scratch.remove();
}

#[test]
#[expect(clippy::disallowed_methods, reason = "offline relay framing against an owned loopback socket thread")]
fn a_closing_client_hears_the_relay_end_its_connection() {
    let scratch = support::Scratch::new("relay-close");
    let upstream = TcpListener::bind("127.0.0.1:0").expect("stub");
    let relay = Relay::plaintext(guard(), scratch.path(), upstream.local_addr().expect("stub address"));
    let stub = thread::spawn(move || {
        for _ in 0..2 {
            let (socket, _) = upstream.accept().expect("stub accepts");
            let mut socket = Socket::reading(socket, Duration::from_secs(2), "the offline relay stub's request");
            head(&mut socket);
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok").expect("stub response");
        }
    });
    // The engine's setup host asks with Connection: close, then half-closes and drains until the peer ends.
    let asked = TcpStream::connect(relay.api).expect("client");
    let mut asked = Socket::reading(asked, Duration::from_secs(2), "the relay's requested close");
    asked.write_all(b"GET /app HTTP/1.1\r\nHost: original\r\nConnection: close\r\n\r\n").expect("request");
    assert_eq!(answer(&mut asked), b"ok");
    assert_eq!(asked.read(&mut [0; 1]).expect("the relay ends an asked close"), 0);
    let halved = TcpStream::connect(relay.api).expect("client");
    let mut halved = Socket::reading(halved, Duration::from_secs(2), "the relay's half-close response");
    halved.write_all(b"GET /app HTTP/1.1\r\nHost: original\r\n\r\n").expect("request");
    assert_eq!(answer(&mut halved), b"ok");
    halved.shutdown(std::net::Shutdown::Write).expect("half-close");
    assert_eq!(halved.read(&mut [0; 1]).expect("the relay ends a half-closed connection"), 0);
    stub.join().expect("stub joined");
    assert_eq!((relay.refused()).len(), 0);
    drop(relay);
    scratch.remove();
}
