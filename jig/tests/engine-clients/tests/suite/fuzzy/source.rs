//! Hostile proxy fields (jig/docs/design/protocol/client/default-http.md, 3).
use jig_protocol_engine_clients::{FromTransport, Message, Via};
use jig_protocol_engine_clients_http::{Event, Limits, Mount, Request, Settings};
use skein_http::{
    Header, Method, Version,
    server::{Body, Call, Event as HttpEvent},
};
use skein_lib::{Env, Queue, Rng, Time, Token, Wall, stream::Up};
use std::net::IpAddr;

const PUBLIC: &[u8] = b"app.example.com";
const EXCHANGE: Token = Token::new(1);

fn check(fields: Vec<Vec<u8>>) {
    // The oracle starts at the final field and splits bytes before decoding text:
    // an earlier element need not be UTF-8.
    let expected = fields.last().and_then(|field| {
        let element = field.rsplit(|byte| *byte == b',').next().expect("one element");
        let text = std::str::from_utf8(element).ok()?.trim_matches([' ', '\t']);
        text.parse::<IpAddr>().ok().map(|ip| ip.to_canonical())
    });
    let mut headers = vec![
        Header { name: b"host".as_slice().into(), value: PUBLIC.into() },
        Header { name: b"content-type".as_slice().into(), value: b"application/octet-stream".as_slice().into() },
    ];
    for (index, field) in fields.into_iter().enumerate() {
        headers.push(Header {
            name: if index % 2 == 0 {
                b"X-Forwarded-For".as_slice().into()
            } else {
                b"x-FORWARDED-for".as_slice().into()
            },
            value: field.into_boxed_slice(),
        });
        headers.push(Header { name: b"x-other".as_slice().into(), value: b"192.0.2.9".as_slice().into() });
    }
    let env = Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: Limits { public_calls: 0, calls: 1, streams: 1, body: 1, read: 1, send: 1, event: 1 },
    };
    let mut mount = Mount::new(
        Settings { prefix: b"/jig".as_slice().into(), max_age: 1, public: Some(PUBLIC.into()) },
        &env.limits,
    );
    mount.down(
        &env,
        Event::Call {
            call: EXCHANGE,
            head: Call {
                method: Method::Post,
                target: b"/jig/v1/approval/ask".as_slice().into(),
                version: Version::Http11,
                body: Body::None,
                headers: headers.into_boxed_slice(),
            },
            peer: Some("192.0.2.9".parse().expect("proxy peer")),
        },
    );
    mount.down(&env, Event::Http { call: EXCHANGE, event: HttpEvent::Body(Up::End) });
    let mut out = Queue::uncapped();
    while mount.up(&env, &mut out) {}
    let mut messages = Vec::new();
    while let Some(request) = out.pop() {
        if let Request::Component(message) = request {
            messages.push(message);
        }
    }
    let [FromTransport::Exchange { via, source, message: Message::ApprovalAsk(_), .. }] = messages.as_slice() else {
        panic!("approval exchange: {messages:?}")
    };
    assert_eq!((*via, *source), (Via::Public, expected));
}

#[test]
fn arbitrary_forwarded_fields_never_panic_and_only_the_last_element_is_a_source() {
    let mut rng = Rng::new(0x50a7_ce01);
    for _ in 0..2048 {
        let mut fields = Vec::new();
        for _ in 0..rng.below(5) {
            let mut field = Vec::new();
            for _ in 0..rng.below(97) {
                field.push(u8::try_from(rng.next_u64() & 0xff).expect("low byte"));
            }
            fields.push(field);
        }
        check(fields.clone());
        // Random prefixes also precede readable terminal elements, not just
        // overwhelmingly unreadable random text.
        for suffix in [b", \t::ffff:203.0.113.7 \t".as_slice(), b",2001:db8::1", b",203.0.113.7, "] {
            let mut readable = fields.clone();
            let mut field = vec![0xff];
            field.extend_from_slice(suffix);
            readable.push(field);
            check(readable);
        }
    }
}
