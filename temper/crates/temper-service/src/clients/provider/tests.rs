//! The server route's request bound and a detached call's late answer.
#![expect(clippy::disallowed_macros, reason = "ordinary tests allocate a request at the route bound")]
use super::*;
use crate::tests::fixtures;

fn clients() -> Clients {
    let mut clients = fixtures::clients();
    clients.provider_documents = Some(Documents { discovery: b"{}".as_slice().into(), keys: b"{}".as_slice().into() });
    clients
}
#[test]
fn a_closed_call_drops_its_late_answer_without_affecting_another_call() {
    let mut clients = clients();
    let first = Token::new(91);
    let second = Token::new(92);
    let head = fixtures::bundle::head(Method::Get, b"/oauth/userinfo", &[]);
    clients.provider_call(first, head);
    let exchange = clients.provider_replies.get(&first).unwrap().exchange.unwrap();
    clients.provider_event(first, http::Event::Closed);
    let head = fixtures::bundle::head(Method::Get, b"/oauth/userinfo", &[]);
    clients.provider_call(second, head);
    let other = clients.provider_replies.get(&second).unwrap().exchange;
    assert_ne!(other, Some(exchange));
    clients.provider_answer(
        exchange,
        kit::ProviderAnswer { status: 200, challenge: None, body: Secret::new(b"{}".as_slice().into()) },
    );
    assert!(!clients.provider_replies.contains_key(&first));
    let reply = clients.provider_replies.get(&second).unwrap();
    assert_eq!(reply.exchange, other);
    assert!(reply.body.is_none());
}
#[test]
fn a_streamed_body_at_the_bound_reaches_jig_and_one_byte_past_does_not() {
    for past in [false, true] {
        let mut clients = clients();
        let bound = kit::provider_limits(&clients.env.limits).request_bytes;
        let call = Token::new(91);
        let mut head = fixtures::bundle::head(Method::Post, b"/oauth/token", &[]);
        head.body = http::Body::Chunked;
        clients.provider_call(call, head);
        let before = clients.out.len();
        let body = Box::<[u8]>::from(alloc::vec![b'x'; usize::try_from(bound).unwrap()]);
        clients.provider_event(call, http::Event::Body(Up::Bytes(body)));
        if past {
            clients.provider_event(call, http::Event::Body(Up::Bytes(b"x".as_slice().into())));
        }
        clients.provider_event(call, http::Event::Body(Up::End));
        if past {
            assert_eq!(clients.out.len(), before, "nothing sent to jig");
            assert!(clients.provider_replies.get(&call).unwrap().exchange.is_none());
        } else {
            assert_eq!(clients.out.len(), before.checked_add(1).unwrap(), "jig answers the bounded body");
        }
    }
}
