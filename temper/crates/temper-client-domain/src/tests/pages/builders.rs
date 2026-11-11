#![expect(
    clippy::disallowed_methods,
    reason = "ordinary tests select transport requests and assert borrowed read variants"
)]
use super::{
    accounts::{deliver, event},
    fixtures,
};
use crate::{Address, Domain, Event, Place, Request, pages::builders::Read};
use jig_client::{Down, UnlinkedRequest, Watchable, contract as c};
use skein_lib::{List, Queue, Token};

fn frame(role: Option<c::HeldRole>) -> c::WatchEvent {
    let c::WatchEvent::Delivered(c::Delivery::Me(mut me)) = fixtures::frame(fixtures::PARTY) else { unreachable!() };
    me.deployment_role = role;
    c::WatchEvent::Delivered(c::Delivery::Me(me))
}
fn opened_at(role: Option<c::HeldRole>, address: Address) -> (Domain, Token, Queue<Request>) {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    deliver(&mut root, me, frame(role));
    let out = event(&mut root, Event::Went { place: Place::Address(address) });
    (root, me, out)
}
fn opened(role: Option<c::HeldRole>) -> (Domain, Token, Queue<Request>) {
    opened_at(role, Address::Builders { project: fixtures::PROJECT })
}
fn deployment_watch(out: &Queue<Request>) -> Option<Token> {
    out.iter().find_map(|r| match r {
        Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch, subject } } }
            if *subject == Watchable::Jig(c::Subject::Deployment) =>
        {
            Some(*watch)
        }
        Request::Client { .. }
        | Request::Unlinked { .. }
        | Request::Show { .. }
        | Request::Leave { .. }
        | Request::SignOut
        | Request::Dropped { .. }
        | Request::Push(_)
            | Request::Screen { .. }
        | Request::KeepLook { .. }
        | Request::KeepSignIn { .. }
        | Request::Tab { .. }
        | Request::Copy { .. }
        | Request::Fact { .. } => None,
    })
}
fn closes(out: &Queue<Request>, watch: Token) -> bool {
    for request in out {
        if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Close { watch: closed } } } = request
            && *closed == watch
        {
            return true;
        }
    }
    false
}
fn document() -> c::Deployment {
    let link =
        jig_client_protocol_link::Limits::new(fixtures::limits().client.domain, fixtures::limits().client.exchanges)
            .unwrap();
    jig_client_protocol_link::translate::documents::deployment(jig_client_test_records::deployment_rows_record(), &link)
        .unwrap()
}

#[test]
fn the_watch_opens_for_a_role_and_closes_on_leaving() {
    for (role, address) in [
        (c::HeldRole::Owner, Address::Builders { project: fixtures::PROJECT }),
        (c::HeldRole::Policy { role: 1 }, Address::Limits),
    ] {
        let (mut root, _, out) = opened_at(Some(role), address.clone());
        let watch = deployment_watch(&out).expect("role opens deployment");
        assert!(match root.pages().builders().unwrap().read(root.client()) {
            Read::Unknown => true,
            Read::NoRole | Read::Hosts(_) => false,
        });
        let doc = document();
        let before = crate::shown(&root);
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))));
        assert!(crate::shown(&root) > before, "a deployment delivery redraws the tab");
        assert!(match root.pages().builders().unwrap().read(root.client()) {
            Read::Hosts(_) => true,
            Read::Unknown | Read::NoRole => false,
        });
        // Re-entering the same tab neither restarts nor duplicates its read.
        let out = event(&mut root, Event::Went { place: Place::Address(address) });
        assert!(deployment_watch(&out).is_none());
        assert!(!closes(&out, watch));
        let out =
            event(&mut root, Event::Went { place: Place::Address(Address::Settings { project: fixtures::PROJECT }) });
        assert!(closes(&out, watch), "{out:?}");
        assert!(root.pages().builders().is_none());
    }
}
#[test]
fn no_role_watches_nothing_and_reads_no_role() {
    for address in [Address::Builders { project: fixtures::PROJECT }, Address::Limits] {
        let (root, _, out) = opened_at(None, address);
        assert!(deployment_watch(&out).is_none());
        assert!(match root.pages().builders().unwrap().read(root.client()) {
            Read::NoRole => true,
            Read::Unknown | Read::Hosts(_) => false,
        });
    }
}
#[test]
fn losing_the_role_closes_it() {
    let (mut root, me, out) = opened(Some(c::HeldRole::Owner));
    let watch = deployment_watch(&out).unwrap();
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(document()))));
    let out = deliver(&mut root, me, frame(None));
    assert!(closes(&out, watch), "{out:?}");
    assert!(match root.pages().builders().unwrap().read(root.client()) {
        Read::NoRole => true,
        Read::Unknown | Read::Hosts(_) => false,
    });
}
#[test]
fn an_ended_read_is_unknown() {
    for end in [c::WatchEnd::Closed, c::WatchEnd::NotAllowed] {
        let (mut root, _, out) = opened(Some(c::HeldRole::Owner));
        let watch = deployment_watch(&out).unwrap();
        let doc = document();
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))));
        let before = crate::shown(&root);
        event(
            &mut root,
            Event::Client { event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end } } },
        );
        assert!(crate::shown(&root) > before, "an ended read redraws the unknown title");
        assert!(match root.pages().builders().unwrap().read(root.client()) {
            Read::Unknown => true,
            Read::NoRole | Read::Hosts(_) => false,
        });
    }
}
