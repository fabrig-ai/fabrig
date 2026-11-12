//! Page facts and acts (temper/docs/design/client/pages/told.md).
#![expect(clippy::disallowed_methods, reason = "ordinary page tests inspect requests and table values")]
use super::*;
use crate::tests::fixtures as f;
use jig_client::{Down, Family, UnlinkedRequest, Up};
use skein_lib::{List, Token};
const KEY: [u8; 65] = [4; 65];
const ENDPOINT: &[u8] = b"https://fcm.googleapis.com/subscription/private";
fn browser() -> Browser {
    Browser {
        capable: true,
        permission: Permission::Ask,
        endpoint: None,
        user_agent: b"Mozilla/5.0 (iPhone) Version/17 Safari/605.1".as_slice().into(),
    }
}
fn row() -> devices::Row {
    devices::Row {
        number: 7,
        label: b"Safari on iPhone".as_slice().into(),
        at: Wall::from_nanos(10),
        endpoint: ENDPOINT.into(),
    }
}
fn page() -> Told {
    let mut page = Told::new();
    page.party = Some(f::PARTY);
    page.browser = Some(browser());
    page.rows = Some(Box::new([row()]));
    page
}
#[test]
fn labels_name_both_browser_and_system_or_neither() {
    for (ua, expected) in [
        (
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) Version/17.0 Mobile/15 Safari/604.1",
            "Safari on iPhone",
        ),
        ("Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/120.0", "Firefox on Linux"),
        ("Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Chrome/120.0 Mobile Safari/537.36", "Chrome on Android"),
        ("Mozilla/5.0 (Windows NT 10.0) Chrome/120 Safari/537.36 Edg/120", "Edge on Windows"),
        ("Version/17.0 Safari/604.1", "This browser"),
        ("Unknown on Linux", "This browser"),
    ] {
        assert_eq!(&*label(ua.as_bytes()), expected.as_bytes(), "{ua}");
    }
}
#[test]
fn browser_states_wait_for_reads_and_match_only_this_endpoint() {
    let mut page = page();
    assert_eq!(page.state(None), State::Unknown);
    assert_eq!(page.state(Some(None)), State::NotSetUp);
    for (capable, permission, endpoint, expected) in [
        (false, Permission::Ask, None, State::Cannot { home_screen: true }),
        (true, Permission::Denied, None, State::Blocked),
        (true, Permission::Ask, None, State::Off),
        (true, Permission::Granted, Some(b"https://other.test/private".as_slice()), State::Off),
        (true, Permission::Granted, Some(ENDPOINT), State::On { number: 7, at: Wall::from_nanos(10) }),
    ] {
        page.browser = Some(Browser { capable, permission, endpoint: endpoint.map(Into::into), ..browser() });
        assert_eq!(page.state(Some(Some(KEY))), expected);
        assert_eq!(page.state(Some(None)), State::NotSetUp, "deployment off wins over every browser state");
    }
    page.rows = None;
    assert_eq!(page.state(Some(Some(KEY))), State::Unknown);
}
fn asked(out: &mut Queue<Request>) -> (Key, w::Request) {
    let Some(Request::Client(jig_client::Event::Send {
        key, request: jig_client::Ask::Family { scope, record }, ..
    })) = out.pop()
    else {
        panic!("own family send: {out:?}")
    };
    assert_eq!(scope, jig_client::contract::Scope::Own);
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    let record = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap();
    (key, record.into_parts().request)
}
fn answered(page: &mut Told, key: Key, outcome: w::Outcome, out: &mut Queue<Request>) {
    let root = crate::unlinked(&f::limits(), f::SEED);
    let record = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap();
    assert!(page.told(
        root.client(),
        &Tell::Answered {
            key,
            outcome: Outcome::Answered(jig_client::Answered::Family(Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&record)
            }))
        },
        f::limits().forge,
        out
    ));
}
#[test]
fn subscription_acts_send_keys_and_drop_only_after_removal_answers() {
    let mut serial = 1;
    let mut out = Queue::uncapped();
    let mut page = page();
    page.act(Act::TurnOn, Some(KEY), &mut serial, f::limits().forge, &mut out);
    let Some(Request::Push(Duty::Subscribe { token, key })) = out.pop() else { panic!("subscribe") };
    assert_eq!(key, KEY);
    assert_eq!(page.state(Some(Some(KEY))), State::TurningOn);
    page.report(
        Report::Subscribed {
            token,
            subscription: Subscription { endpoint: ENDPOINT.into(), p256dh: KEY, auth: [8; 16] },
        },
        &mut serial,
        f::limits().forge,
        &mut out,
    );
    let (key, request) = asked(&mut out);
    let w::Request::AddDevice(value) = request else { panic!("add device") };
    let value = value.into_parts();
    assert_eq!(&*value.endpoint, ENDPOINT);
    assert_eq!(value.p256dh, KEY);
    assert_eq!(value.auth, [8; 16]);
    assert_eq!(&*value.label, b"Safari on iPhone");
    answered(
        &mut page,
        key,
        w::Outcome::DeviceAdded(w::DeviceAdded::new(&w::CEILINGS, w::DeviceAddedParts { number: 7 }).unwrap()),
        &mut out,
    );
    page.act(Act::TurnOff, Some(KEY), &mut serial, f::limits().forge, &mut out);
    let (key, request) = asked(&mut out);
    let w::Request::RemoveDevice(value) = request else { panic!("remove") };
    assert_eq!(value.number(), 7);
    assert!(out.is_empty(), "do not drop before the answer");
    answered(&mut page, key, w::Outcome::DeviceRemoved, &mut out);
    assert_eq!(out.pop(), Some(Request::Push(Duty::Drop { token: key.number })));
    page.report(Report::Dropped { token: key.number }, &mut serial, f::limits().forge, &mut out);
    assert!(page.browser().unwrap().endpoint.is_none());
    page.act(Act::Remove { number: 7 }, Some(KEY), &mut serial, f::limits().forge, &mut out);
    let (key, _) = asked(&mut out);
    answered(&mut page, key, w::Outcome::DeviceRemoved, &mut out);
    assert!(out.is_empty(), "Remove does not touch a browser subscription");
    for (reason, expected) in [
        (w::DeviceRefusal::Off, Refusal::Off),
        (w::DeviceRefusal::Service, Refusal::Service),
        (w::DeviceRefusal::Malformed, Refusal::Malformed),
        (w::DeviceRefusal::Person, Refusal::Person),
        (w::DeviceRefusal::TooMany, Refusal::TooMany),
    ] {
        page.act(Act::TurnOn, Some(KEY), &mut serial, f::limits().forge, &mut out);
        let Some(Request::Push(Duty::Subscribe { token, .. })) = out.pop() else { panic!("subscribe again") };
        page.report(
            Report::Subscribed {
                token,
                subscription: Subscription { endpoint: ENDPOINT.into(), p256dh: KEY, auth: [8; 16] },
            },
            &mut serial,
            f::limits().forge,
            &mut out,
        );
        let (key, _) = asked(&mut out);
        answered(
            &mut page,
            key,
            w::Outcome::DeviceRefused(w::DeviceRefused::new(&w::CEILINGS, w::DeviceRefusedParts { reason }).unwrap()),
            &mut out,
        );
        assert_eq!(page.refusal(), Some(expected));
        page.browser.as_mut().unwrap().endpoint = None;
    }
    page.act(Act::TurnOn, Some(KEY), &mut serial, f::limits().forge, &mut out);
    let Some(Request::Push(Duty::Subscribe { token, .. })) = out.pop() else { panic!("permission prompt") };
    page.report(Report::Refused { token, why: Refusal::Blocked }, &mut serial, f::limits().forge, &mut out);
    assert_eq!(page.state(Some(Some(KEY))), State::Blocked);
}
fn event(root: &mut crate::Domain, event: crate::Event) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut crate::Domain, up: Up) -> Queue<crate::Request> {
    event(root, crate::Event::Client { event: jig_client::Event::Up { up } })
}
#[test]
fn devices_are_own_scope_watched_only_while_the_page_is_open() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::uncapped());
    f::confirm(&mut root, me, f::PARTY, &mut Queue::uncapped());
    let open = crate::Event::Went { place: crate::Place::Address(crate::Address::Told) };
    let out = event(&mut root, open);
    let watch: Token = out
        .iter()
        .find_map(|request| {
            if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
                request
                && *subject == devices::watch(f::limits().forge).unwrap()
            {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("devices watch");
    up(&mut root, Up::Watched { watch, event: jig_client::contract::WatchEvent::Opened });
    let mut rows = List::with_capacity(w::CEILINGS.devices_rows);
    rows.push(
        w::DeviceRow::new(
            &w::CEILINGS,
            w::DeviceRowParts {
                number: 7,
                label: b"Safari on iPhone".as_slice().into(),
                at: 10,
                endpoint: ENDPOINT.into(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    let document = w::Devices::new(&w::CEILINGS, w::DevicesParts { rows }).unwrap();
    up(
        &mut root,
        Up::FamilyDelivered {
            watch,
            record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::devices(&document) },
        },
    );
    assert_eq!(root.pages().told().unwrap().rows(), Some([row()].as_slice()));
    let same = event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Told) });
    assert!(!same.iter().any(|request| {
        if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { subject, .. } } } =
            request
        {
            *subject == devices::watch(f::limits().forge).unwrap()
        } else {
            false
        }
    }));
    let out = event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Accounts) });
    assert!(out.iter().any(|request| {
        if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Close { watch: closed } } } =
            request
        {
            *closed == watch
        } else {
            false
        }
    }));
    assert!(root.pages().told().is_none());
    event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Told) });
    let left = event(&mut root, crate::Event::Went { place: crate::Place::Missing });
    assert!(root.pages().told().is_none());
    assert!(!left.iter().any(|request| {
        if let crate::Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Open { subject, .. } } } =
            request
        {
            *subject == devices::watch(f::limits().forge).unwrap()
        } else {
            false
        }
    }));
    assert_eq!(&*crate::address::print(&crate::Address::Told), b"/told");
    assert_eq!(crate::address::parse(b"/told"), crate::Place::Address(crate::Address::Told));
}
