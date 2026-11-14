use super::*;
use skein_lib::{Writer, stream::Read};

#[test]
fn answering_a_quiet_wire_changes_no_schedule() {
    let ordinary = exchange(false);
    let probed = exchange(true);
    for (token, before) in &ordinary.sockets {
        let after = probed.sockets.get(token).expect("same socket");
        assert!(before.checked.request && before.checked.response, "a message crossed");
        assert!(after.checked.request && after.checked.response, "a message crossed with quiet probes");
        assert_eq!(before.shell.trace, after.shell.trace);
        assert_eq!(before.engine.trace, after.engine.trace);
    }
}

fn exchange(probe_quiet: bool) -> Net {
    const CLIENT: ClientId = ClientId(1);
    const CONN: Conn = Conn(1);
    const OWNER: Token = Token::new(1);
    const REQUEST: &[u8] = b"POST /jig/v1/sign-out HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
    // Enough attempts for this seeded exchange's fragments and delayed room.
    const ATTEMPTS: usize = 512;
    let answer = jig_client_protocol_wire::v22::SignOutAnswer::SignedOut;
    let mut writer = Writer::new(skein_lib::length::usize_of(answer.measure()));
    answer.encode(&mut writer).expect("measured answer");
    let body = writer.finish();
    let mut response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    response.extend_from_slice(&body);
    let request_room = u32::try_from(REQUEST.len()).expect("small request");
    let response_room = u32::try_from(response.len()).expect("small response");
    let mut net = Net::new(7);
    let mut rules = Rules::default();
    let accepted = net.down(CLIENT, CONN, Down::Demand { read: Read::Nothing, room: request_room });
    let Some(Event::Accepted { socket, .. }) = accepted else { panic!("accepted socket") };
    net.io(Request::Bind { socket, owner: OWNER });
    net.io(Request::Stream { stream: socket, down: Down::Demand { read: Read::Fill(request_room), room: 0 } });
    for _ in 0..ATTEMPTS {
        if probe_quiet {
            for socket in net.sockets.values_mut() {
                for wire in [&mut socket.shell, &mut socket.engine] {
                    if !wire.work_pending() {
                        assert_eq!(wire.answer(), None, "a quiet wire has no answer");
                    }
                }
            }
        }
        let (events, clients, _) = net.advance(&mut rules);
        for (_, _, up) in clients {
            if let Up::Room = up {
                net.down(CLIENT, CONN, Down::Send(REQUEST.into()));
                net.down(CLIENT, CONN, Down::Demand { read: Read::Fill(response_room), room: 0 });
            }
        }
        for event in events {
            if let Event::Stream { up, .. } = event {
                match up {
                    Up::Bytes(_) => {
                        net.io(Request::Stream {
                            stream: socket,
                            down: Down::Demand { read: Read::Nothing, room: response_room },
                        });
                    }
                    Up::Room => {
                        net.io(Request::Stream {
                            stream: socket,
                            down: Down::Send(response.clone().into_boxed_slice()),
                        });
                    }
                    Up::End | Up::Failed(_) => panic!("uncut exchange"),
                }
            }
        }
        if net.sockets.values().all(|socket| socket.checked.response) {
            break;
        }
    }
    net
}

#[test]
fn a_message_read_as_it_arrives_answers_as_whole_does() {
    let rows: &[(&str, &[u8])] = &[
        ("mixed case", b"HTTP/1.1 200 OK\r\ncOnTeNt-LeNgTh: 4\r\n\r\nbody"),
        ("no length", b"HTTP/1.1 200 OK\r\nOther: value\r\n\r\nbody"),
        ("non UTF-8 head", b"HTTP/1.1 200 \xff\r\nContent-Length: 4\r\n\r\nbody"),
        ("past length", b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\nab"),
    ];
    for (name, http) in rows {
        let mut message = Message::default();
        for end in 0..=http.len() {
            let prefix = &http[..end];
            assert_eq!(message.whole(prefix), whole(prefix), "{name} at {end}");
            assert_eq!(message.whole(prefix), whole(prefix), "{name} unchanged at {end}");
        }
    }
}

#[test]
fn records_fed_byte_by_byte_match_records_fed_at_once() {
    let mut http = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
    // Chunk boundaries split the data prefix, base64 value, CRLF and blank line.
    // The first multi-digit chunk header and every CRLF also arrive byte by byte.
    for chunk in [b": a long comment\r\ndat".as_slice(), b"a: YQ==\r", b"\n\r\ndata: ", b"YmM=\r\n\r", b"\n"] {
        http.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
        http.extend_from_slice(chunk);
        http.extend_from_slice(b"\r\n");
    }
    http.extend_from_slice(b"0\r\n\r\n");
    let mut once = Records::default();
    once.feed(&http);
    let mut incremental = Records::default();
    for end in 0..=http.len() {
        incremental.feed(&http[..end]);
        incremental.feed(&http[..end]);
    }
    let expected = vec![Box::from(*b"a"), Box::from(*b"bc")];
    assert_eq!(once.values, expected);
    assert_eq!(incremental.values, once.values);
}
