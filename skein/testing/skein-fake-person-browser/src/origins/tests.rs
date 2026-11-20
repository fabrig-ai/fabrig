//! The reload is held while its leaving document can still read the stream (browser.md, 9).
use super::*;
use skein_lib::{Time, Wall};

fn get(pages: &mut Pages, socket: Token, path: &[u8]) -> Token {
    let owner = Token::new(pages.next_owner);
    pages.event(io::Event::Accepted {
        owner: pages.owner,
        socket,
        peer: io::Addr::Ip(SocketAddr::from((Ipv4Addr::LOCALHOST, 9000))),
    });
    // Bind and reserve room for the head, then answer each requested line.
    pages.requests.clear();
    room(pages, owner);
    let request = [b"GET ", path, b" HTTP/1.1\r\nHost: test\r\n\r\n"].concat();
    for line in request.split_inclusive(|byte| *byte == b'\n') {
        pages.requests.clear();
        pages.event(io::Event::Stream { owner, up: stream::Up::Bytes(line.into()) });
    }
    owner
}

fn room(pages: &mut Pages, owner: Token) {
    pages.event(io::Event::Stream { owner, up: stream::Up::Room });
}

fn output(pages: &mut Pages) -> BTreeMap<Token, Vec<u8>> {
    let mut sent = BTreeMap::<Token, Vec<u8>>::new();
    while let Some(request) = pages.take() {
        if let Request::Stream { stream, down: stream::Down::Send(bytes) } = request {
            sent.entry(stream).or_default().extend_from_slice(&bytes);
        }
    }
    sent
}

#[test]
fn reload_signals_the_open_stream_and_waits_for_the_leaving_request_before_answering() {
    let document = Token::new(1);
    let stream = Token::new(2);
    let reload = Token::new(3);
    let leaving = Token::new(4);
    // The signal can arrive before or after the initial body has found room.
    for first_sent in [false, true] {
        let fixed = [
            (b"/requests".as_slice(), b"document".as_slice()),
            (b"/request-stream", b"first bytes\n"),
            (b"/request-leave", b"left"),
        ];
        let fixed = fixed
            .into_iter()
            .map(|(path, body)| Page {
                path: path.into(),
                kind: b"text/plain".as_slice().into(),
                body: body.into(),
                headers: vec![],
            })
            .collect();
        let mut pages = Pages::new(fixed, Time::ZERO, Wall::EPOCH);
        pages.requests.clear();
        pages.stream(b"/request-stream".as_slice().into());
        get(&mut pages, document, b"/requests");
        assert!(output(&mut pages).get(&document).expect("initial answer").starts_with(b"HTTP/1.1 200"));
        let stream_owner = get(&mut pages, stream, b"/request-stream");
        assert!(output(&mut pages).get(&stream).expect("stream head").starts_with(b"HTTP/1.1 200"));
        if first_sent {
            room(&mut pages, stream_owner);
            assert_eq!(output(&mut pages).get(&stream).expect("first chunk"), b"c\r\nfirst bytes\n\r\n");
        }
        let reload_owner = get(&mut pages, reload, b"/requests");
        assert!(output(&mut pages).is_empty(), "no reload head or body before the leaving request");
        room(&mut pages, stream_owner);
        let sent = output(&mut pages);
        let chunk = sent.get(&stream).expect("reload signals the stream");
        if first_sent {
            assert_eq!(chunk, b"6\r\nleave\n\r\n");
        } else {
            assert_eq!(chunk, b"12\r\nfirst bytes\nleave\n\r\n");
        }
        assert!(!sent.contains_key(&reload));
        assert_eq!(pages.open_streams(), 1, "signal does not end the stream");
        get(&mut pages, leaving, b"/request-leave");
        let sent = output(&mut pages);
        assert!(sent.get(&leaving).expect("leaving request answered").starts_with(b"HTTP/1.1 200"));
        assert!(sent.get(&reload).expect("reload released").starts_with(b"HTTP/1.1 200"));
        room(&mut pages, reload_owner);
        assert_eq!(output(&mut pages).get(&reload).expect("reload body"), b"document");
    }
}
