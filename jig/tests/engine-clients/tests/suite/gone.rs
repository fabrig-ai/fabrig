//! A caller gone releases its connection without cancelling its work (protocol/client/wire.md, 2.2).
use jig_client_protocol_wire::v21 as w;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Event, FromApplication, Served, ToApplication};
use skein_lib::{Reader, Token, Writer};

fn signed() -> Peer {
    let mut settings = fixtures::settings();
    settings.served = skein_lib::List::with_capacity(2);
    settings.served.push(Served { family: None, first: 21, last: 21 }).expect("jig");
    settings.served.push(Served { family: Some(1), first: 1, last: 1 }).expect("family");
    let mut server = fixtures::server_limits();
    server.peer_end = jig_protocol_engine_clients_http::server::PeerEnd::Connection;
    Peer::signed_in_with_server_bounds(
        &settings,
        fixtures::FIRST_SIGN_IN,
        fixtures::limits(),
        fixtures::http_limits(),
        server,
    )
}

fn request() -> Box<[u8]> {
    let limits = fixtures::limits().wire;
    let mut keyed = w::Keyed::decode(&limits, &mut Reader::new(&super::requests::carrying_request(b"")))
        .expect("family request")
        .into_parts();
    keyed.value = None;
    let keyed = w::Keyed::new(&limits, keyed).expect("request without a value");
    let mut bytes = Writer::new(skein_lib::length::usize_of(keyed.measure()));
    keyed.encode(&mut bytes).expect("keyed bytes");
    bytes.finish()
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Leave,
    Stop,
    Close,
}

#[test]
fn a_close_does_not_wait_for_a_family_request_whose_caller_has_gone() {
    use Step::{Close, Leave, Stop};
    for (leaf, leave) in
        [("cut", Peer::cut_exchange as fn(&mut Peer, Token)), ("end", Peer::end_exchange as fn(&mut Peer, Token))]
    {
        for order in [[Leave, Stop, Close], [Stop, Leave, Close], [Stop, Close, Leave]] {
            let mut peer = signed();
            let socket = peer.authenticated_post(b"/jig/v1/request", request());
            let Some(ToApplication::Request { .. }) = peer.take_application() else {
                panic!("application request");
            };
            let mut left = false;
            for step in order {
                match step {
                    Leave => {
                        leave(&mut peer, socket);
                        left = true;
                    }
                    Stop => peer.event(Event::Stop),
                    Close => {
                        peer.event(Event::Close);
                        if !left {
                            assert_eq!(peer.closed(), 0, "{leaf} {order:?}: caller still there");
                        }
                    }
                }
            }
            assert_eq!(peer.closed(), 1, "{leaf} {order:?}: caller gone");
            assert_eq!(peer.answered(socket), None, "{leaf} {order:?}");
        }
    }
}

#[test]
fn an_answer_that_comes_after_its_caller_has_gone_settles_with_no_reply() {
    let mut peer = signed();
    let socket = peer.authenticated_post(b"/jig/v1/request", super::requests::carrying_request(b"a carried key"));
    let number = peer.keep_carried();
    let Some(ToApplication::Request { reply_to, .. }) = peer.take_application() else {
        panic!("application request");
    };
    peer.cut_exchange(socket);
    peer.event(Event::Application(FromApplication::Answered { to: reply_to, family: 1, record: Box::new([7]) }));
    assert_eq!(peer.answered(socket), None);
    assert_eq!(peer.carried(number).map(skein_lib::Secret::expose), Some(b"a carried key".as_slice()), "used, so kept");
    peer.event(Event::Close);
    assert_eq!(peer.closed(), 1);
}
