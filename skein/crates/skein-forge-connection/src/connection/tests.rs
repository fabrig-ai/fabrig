//! A handshake's writes precede the holder's exchange.
use super::*;
use crate::tests::fixtures as f;
use skein_lib::{Token, Wall};

#[test]
fn handshake_bytes_do_not_write_an_exchange() {
    let mut forge = f::settings().forges.into_boxed().into_iter().next().expect("fixture forge");
    forge.transport =
        Transport::Tls { trust: f::trust(), server_name: skein_tls::Name::new("localhost").expect("DNS name") };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: f::limits() };
    let mut c = Connection::new(&forge, &env.limits, f::shared(), env.now);
    let mut io = Queue::uncapped();
    let mut kept = Store::new(env.limits.kept_bytes);
    c.link.connecting(Token::new(100), &mut io);
    c.connect(&env);
    let mut sent = false;
    for _ in 0_u32..64 {
        match c.step(&env, &mut io, false, &mut kept) {
            Moved::Within | Moved::Nothing => {}
            Moved::Head(_) | Moved::Token(_) | Moved::Finished { .. } => unreachable!("no HTTP exchange"),
        }
        while let Some(request) = io.pop() {
            let skein_io::Request::Stream { down, .. } = request else { unreachable!("link emits stream requests") };
            match down {
                Down::Send(bytes) => sent |= !bytes.is_empty(),
                Down::Demand { room, .. } => {
                    if room > 0 {
                        c.stream(&env, Up::Room);
                    }
                }
                Down::Finish => {}
            }
        }
    }
    assert!(sent, "the handshake wrote bytes");
    assert_eq!(c.phase(), Phase::Handshaking, "TLS writes keep the handshake phase");
    assert!(!c.written, "no exchange byte was written");
    c.timeout();
    let Some(Ended::Failed(failure)) = c.take_ended() else { unreachable!("timeout reports failure") };
    assert_eq!(failure, Failure::Unseen);
}

#[test]
fn a_connected_notice_keeps_the_close_under_way() {
    let forge = f::settings().forges.into_boxed().into_iter().next().expect("fixture forge");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: f::limits() };
    let mut c = Connection::new(&forge, &env.limits, f::shared(), env.now);
    let mut io = Queue::uncapped();
    c.link.connecting(Token::new(100), &mut io);
    c.failed(Failure::Unseen);
    c.close(&env, false, &mut io);
    c.connect(&env);
    assert!(c.closing(), "the connected notice keeps the close");
    assert_eq!(c.phase(), Phase::Closed);
    let Some(Ended::Failed(failure)) = c.take_ended() else {
        unreachable!("the failure survives the connected notice")
    };
    assert_eq!(failure, Failure::Unseen);
}

#[test]
#[should_panic(expected = "a connect completes once")]
fn a_second_connect_cannot_drop_a_sending_exchange() {
    let mut h = f::ConnectionHarness::new();
    h.start_forgejo(skein_forge::forgejo::request::Request::User, None).expect("valid request");
    assert_eq!(h.connection.phase(), Phase::Sending { written: false });
    h.connection.connect(&h.env);
}

#[test]
fn an_early_head_and_upload_endings_preserve_the_sending_exchange() {
    for event in [
        http::Event::Response(http::Response {
            version: skein_http::Version::Http11,
            status: 413,
            headers: Box::from([]),
            framing: http::Framing::Empty,
        }),
        http::Event::Upload(Up::Failed(skein_lib::stream::Fault::Reset)),
        http::Event::Upload(Up::End),
    ] {
        let mut h = f::ConnectionHarness::new();
        h.start_forgejo(
            skein_forge::forgejo::request::Request::RemoveReviewers {
                at: skein_forge::forgejo::request::Repo {
                    owner: Box::from(b"fixture".as_slice()),
                    name: Box::from(b"specimen".as_slice()),
                },
                number: 7,
                reviewers: Box::from([Box::from(b"alice".as_slice())]),
            },
            None,
        )
        .expect("upload request");
        let head = match &event {
            http::Event::Response(_) => true,
            http::Event::Upload(_) => false,
            http::Event::Body(_) | http::Event::Done(_) | http::Event::Failed(_) | http::Event::Closed => {
                unreachable!("early head or upload ending")
            }
        };
        let moved = h.connection.http_event(&h.env, event, &mut h.kept);
        match moved {
            Moved::Head(response) => assert_eq!(response.status, 413, "early head reaches the holder"),
            Moved::Within => assert!(!head, "early head is not swallowed"),
            Moved::Nothing | Moved::Token(_) | Moved::Finished { .. } => unreachable!("no completed exchange"),
        }
        assert_eq!(h.connection.phase(), Phase::Sending { written: false }, "no write completed");
        assert!(h.connection.in_flight(), "the exchange remains held");
        h.connection.stream(&h.env, Up::Room);
        for _ in 0_u32..64 {
            match h.connection.step(&h.env, &mut h.io, false, &mut h.kept) {
                Moved::Within | Moved::Nothing => {}
                Moved::Head(_) | Moved::Token(_) | Moved::Finished { .. } => unreachable!("no answer queued"),
            }
            if h.connection.written {
                break;
            }
        }
        assert!(h.connection.written, "the request head wrote bytes");
        let expected = if head { Phase::Sending { written: true } } else { Phase::Reading };
        assert_eq!(h.connection.phase(), expected, "only a live upload keeps sending after the write");
        assert!(h.connection.take_ended().is_none(), "an upload ending is not an exchange failure");
    }
}
