//! Admission under the existing connection bound (jig/docs/design/protocol/client/default-http.md, 4).
use super::*;
use skein_http::server;
use skein_lib::stream::Down;

fn enqueue(clients: &mut Clients, call: Token) {
    enqueue_peer(clients, call, None);
}
fn enqueue_peer(clients: &mut Clients, call: Token, peer: Option<::core::net::IpAddr>) {
    let head = server::Call {
        method: skein_http::Method::Post,
        target: b"/jig/v1/read".as_slice().into(),
        headers: Box::new([
            skein_http::Header { name: b"host".as_slice().into(), value: b"127.0.0.1:7100".as_slice().into() },
            skein_http::Header {
                name: b"content-type".as_slice().into(),
                value: b"application/octet-stream".as_slice().into(),
            },
        ]),
        body: server::Body::Length(1),
        version: skein_http::Version::Http11,
    };
    clients.server_out.push(crate::server::Routed::Call { call, head, peer });
    clients.server_requests();
}

#[test]
fn a_call_past_the_room_waits_without_reading_and_a_closed_waiter_is_forgotten() {
    let mut clients = crate::tests::fixtures::clients();
    let mut hosts = List::with_capacity(1);
    hosts.push(Box::from(b"127.0.0.1:7100".as_slice())).expect("one listener");
    clients.mount.down(&clients.mount_env, http::Event::Listening { hosts });
    let capacity = clients.mount_env.limits.calls;
    for number in 0..capacity {
        enqueue(&mut clients, Token::new(u64::from(number)));
        assert!(clients.admit_waiting());
    }
    assert_eq!(clients.mount.room(), 0);
    let closed = Token::new(u64::from(capacity));
    enqueue(&mut clients, closed);
    assert_eq!(clients.waiting.len(), 1);
    assert!(!clients.admit_waiting(), "a full mapping takes no fifth call");
    assert!(clients.replies.is_empty(), "ordinary pressure is not a 503");
    clients.server_out.push(crate::server::Routed::Http { call: closed, event: server::Event::Closed });
    clients.server_requests();
    assert!(clients.waiting.is_empty(), "a dead page's waiting head is forgotten");

    let waiting = Token::new(u64::from(capacity).checked_add(1).expect("next connection"));
    enqueue(&mut clients, waiting);
    while clients.mount.up(&clients.mount_env, &mut clients.mount_out) {}
    while let Some(request) = clients.mount_out.pop() {
        let http::Request::Http { call, .. } = request else { panic!("only demands for withheld bodies") };
        assert_ne!(call, waiting, "the waiting body stays unread");
    }
    clients.mount.down(&clients.mount_env, http::Event::Http { call: Token::new(0), event: server::Event::Closed });
    while clients.mount.up(&clients.mount_env, &mut clients.mount_out) {}
    clients.mount_out.clear();
    assert!(clients.admit_waiting());
    assert!(clients.waiting.is_empty());
    assert_eq!(clients.mount.room(), 0, "the existing bound stays full");
    assert!(clients.mount.up(&clients.mount_env, &mut clients.mount_out));
    let Some(http::Request::Http { call, request: server::Request::Body(Down::Demand { .. }) }) =
        clients.mount_out.pop()
    else {
        panic!("the admitted body is now asked for")
    };
    assert_eq!(call, waiting);
}

#[test]
fn a_waiting_call_hands_its_peer_to_the_mount() {
    let mut clients = crate::tests::fixtures::clients();
    let mut hosts = List::with_capacity(1);
    hosts.push(Box::from(b"127.0.0.1:7100".as_slice())).expect("one listener");
    clients.mount.down(&clients.mount_env, http::Event::Listening { hosts });
    let first = Token::new(1);
    let second = Token::new(2);
    let first_peer = Some(::core::net::IpAddr::V4(::core::net::Ipv4Addr::new(203, 0, 113, 7)));
    let second_peer = Some(::core::net::IpAddr::V6(::core::net::Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)));
    enqueue_peer(&mut clients, first, first_peer);
    enqueue_peer(&mut clients, second, second_peer);
    assert_eq!(clients.waiting.len(), 2);
    let mut origins = List::uncapped();
    for _ in 0_u8..2 {
        assert!(clients.admit_waiting());
        while clients.mount.up(&clients.mount_env, &mut clients.mount_out) {}
        let Some(http::Request::Http { call, request: server::Request::Body(_) }) = clients.mount_out.pop() else {
            panic!("admitted call asks for its body");
        };
        clients.mount.down(
            &clients.mount_env,
            http::Event::Http { call, event: server::Event::Body(skein_lib::stream::Up::End) },
        );
        while clients.mount.up(&clients.mount_env, &mut clients.mount_out) {}
        let Some(http::Request::Component(kit::FromTransport::Exchange { exchange, via, source, .. })) =
            clients.mount_out.pop()
        else {
            panic!("admitted exchange");
        };
        origins.push((exchange, via, source)).expect("admitted exchange origin");
    }
    assert_eq!(
        origins.into_boxed().as_ref(),
        &[(first, kit::Via::Listener, first_peer), (second, kit::Via::Listener, second_peer)]
    );
}

#[test]
fn a_client_stream_failure_reaches_the_operator_notes() {
    for standalone in [false, true] {
        let mut service = crate::tests::fixtures::demo_service();
        let engine = service.engine.as_mut().expect("engine");
        engine.clients.starting = None;
        engine.clients.out.push(kit::Request::StreamFailed {
            sign_in: 41,
            why: kit::StreamFailure::QueueBound { queued_bytes: 65_536 },
        });
        engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
        let note =
            if standalone { service.notes().pop() } else { service.engine.as_mut().expect("engine").notes().pop() };
        assert_eq!(
            note,
            Some(crate::Note::ClientStreamFailed {
                sign_in: 41,
                why: kit::StreamFailure::QueueBound { queued_bytes: 65_536 },
            })
        );
    }
}
