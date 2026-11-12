//! Admission under the existing connection bound (jig/docs/design/protocol/client/default-http.md, 4).
use super::*;
use skein_http::server;
use skein_lib::stream::Down;

fn enqueue(clients: &mut Clients, call: Token) {
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
    clients.server_out.push(crate::server::Routed::Call { call, head });
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
fn a_client_stream_failure_reaches_the_operator_notes() {
    let mut service = crate::tests::fixtures::demo_service();
    let engine = service.engine.as_mut().expect("engine");
    engine.clients.starting = None;
    engine
        .clients
        .out
        .push(kit::Request::StreamFailed { sign_in: 41, why: kit::StreamFailure::QueueBound { queued_bytes: 65_536 } });
    engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
    assert_eq!(
        service.notes().pop(),
        Some(crate::Note::ClientStreamFailed {
            sign_in: 41,
            why: kit::StreamFailure::QueueBound { queued_bytes: 65_536 },
        })
    );
}
