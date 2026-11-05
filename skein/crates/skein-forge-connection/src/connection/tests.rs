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
    c.link.connecting(Token::new(100), &mut io);
    c.connect(&env);
    let mut sent = false;
    for _ in 0_u32..64 {
        match c.step(&env, &mut io, false) {
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
    assert_eq!(c.phase, Phase::Handshaking, "TLS writes keep the handshake phase");
    assert!(!c.written, "no exchange byte was written");
    c.timeout();
    let Some(Ended::Failed(failure)) = c.take_ended() else { unreachable!("timeout reports failure") };
    assert_eq!(failure, Failure::Unseen);
}

#[test]
fn a_connected_notice_keeps_the_in_flight_exchange() {
    let mut h = f::ConnectionHarness::new();
    h.start_forgejo(skein_forge::forgejo::request::Request::User, None).expect("user request");
    h.pump();
    h.connection.connect(&h.env);
    assert!(h.connection.in_flight(), "the exchange still belongs to its holder");
    h.connection.stream(&h.env, Up::Room);
    h.pump();
    assert!(h.connection.written, "the request wrote bytes");
    assert_eq!(h.connection.phase, Phase::Reading);
}
