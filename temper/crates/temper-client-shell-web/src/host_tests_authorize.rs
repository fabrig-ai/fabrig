//! The shipped host carries Authorize exactly once and replaces only a successful departure.
use super::*;
use jig_client_protocol_wire::v19 as w;
use skein_lib::Reader;
const QUERY: &[u8] = b"client_id=forgejo&state=a+b%20c";
fn sent(host: &mut Host) -> u32 {
    let path = [b"/oauth/authorize?".as_slice(), QUERY].concat();
    host.went(&path);
    assert!(host.show.is_none(), "authorization addresses are never canonicalized");
    let mut sent = None;
    for request in host.requests() {
        if let Asked::Exchange { id, call, .. } = request
            && call.path.as_ref() == b"/jig/v1/authorize"
        {
            assert!(sent.is_none(), "one authorization exchange");
            let record =
                w::Authorize::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("authorization record");
            assert_eq!(record.query(), QUERY, "raw query goes to jig");
            sent = Some(id);
        }
    }
    host.due();
    assert!(
        !host
            .requests()
            .iter()
            .any(|ask| matches!(ask, Asked::Exchange { call, .. } if call.path.as_ref() == b"/jig/v1/authorize")),
        "no second authorization on a clock wake"
    );
    sent.expect("authorization sent once signed in")
}
#[test]
fn authorization_go_replaces_the_address_once() {
    let mut host = super::departure::linked();
    host.show = None;
    let exchange = sent(&mut host);
    let location = b"http://localhost:9999/callback?code=test&state=a";
    let answer = w::AuthorizeAnswer::Go(
        w::AuthorizeGo::new(&w::CEILINGS, w::AuthorizeGoParts { location: location.as_slice().into() })
            .expect("callback"),
    );
    host.heard(Heard::Answered {
        exchange,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
    });
    host.requests();
    assert_eq!(host.replace.take().as_deref(), Some(location.as_slice()));
    assert!(host.navigate.is_none());
    host.due();
    host.requests();
    assert!(host.replace.is_none());
}
#[test]
fn a_lost_authorization_draws_busy_without_departing() {
    let mut host = super::departure::linked();
    host.show = None;
    let exchange = sent(&mut host);
    host.heard(Heard::Failed { exchange });
    host.requests();
    assert_eq!(host.root.authorization().expect("authorization").state(), root::authorize::State::Busy);
    assert!(host.navigate.is_none() && host.replace.is_none());
}
#[test]
fn oversized_authorization_is_refused_in_place_without_an_exchange() {
    let mut host = super::departure::linked();
    host.show = None;
    let mut path = b"/oauth/authorize?".to_vec();
    path.resize(
        b"/oauth/authorize?".len()
            + usize::try_from(root::authorize::QUERY_BYTES).expect("query ceiling fits usize")
            + 1,
        b'x',
    );
    host.went(&path);
    assert!(host.show.is_none());
    assert!(
        !host
            .requests()
            .iter()
            .any(|ask| matches!(ask, Asked::Exchange { call, .. } if call.path.as_ref() == b"/jig/v1/authorize"))
    );
    assert_eq!(
        host.root.authorization().expect("authorization").state(),
        root::authorize::State::Refused(jig_client::AuthorizeWhy::Malformed)
    );
}

#[test]
fn an_oversized_initial_address_sends_nothing_even_for_transport_sign_in() {
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH, temper_client_view::sign_in::Ways::NONE);
    let mut path = b"/oauth/authorize?".to_vec();
    path.resize(
        b"/oauth/authorize?".len()
            + usize::try_from(root::authorize::QUERY_BYTES).expect("query ceiling fits usize")
            + 1,
        b'x',
    );
    host.start(&path, Vec::new(), None);
    assert!(host.requests().is_empty(), "the address is refused before any exchange");
    assert!(host.show.is_none());
}
