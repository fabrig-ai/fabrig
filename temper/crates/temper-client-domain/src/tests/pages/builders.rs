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
fn role_gated_deployment_watches_wait_for_me_and_leave_in_its_step() {
    for address in [Address::Accounts, Address::Builders { project: fixtures::PROJECT }] {
        let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
        let me = fixtures::start(&mut root, List::uncapped());
        let starting = event(&mut root, Event::Went { place: Place::Address(address) });
        assert!(deployment_watch(&starting).is_none());
        let mut live = Queue::uncapped();
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            Told::Link { link: jig_client::Link::Live },
            &mut live,
        );
        assert!(root.pages().me(root.client()).is_none());
        assert!(deployment_watch(&live).is_none(), "live alone names no role");
        let with_me = deliver(&mut root, me, frame(Some(c::HeldRole::Owner)));
        assert!(deployment_watch(&with_me).is_some(), "me and live ask in the same step");
    }
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
        event(&mut root, Event::Client { event: jig_client::Event::Up { up: Up::WatchEnded { watch, end } } });
        assert!(crate::shown(&root) > before, "an ended read redraws the unknown title");
        assert!(match root.pages().builders().unwrap().read(root.client()) {
            Read::Unknown => true,
            Read::NoRole | Read::Hosts(_) => false,
        });
    }
}

use crate::{
    Action,
    pages::{self, builders::Act},
};
use jig_client::{Outcome, Readable, Readout, Told, Up};

fn worker(number: u32, name: &[u8], withdrawn: bool) -> c::WorkerState {
    c::WorkerState {
        number,
        name: name.into(),
        places: 4,
        digest: Some([0xab; 32]),
        withdrawn,
        linked: false,
        since: None,
        pause: c::WorkerPause::Taking,
    }
}
fn ready() -> (Domain, Token, Token) {
    let (mut root, me, out) = opened(Some(c::HeldRole::Owner));
    let watch = deployment_watch(&out).unwrap();
    let mut doc = document();
    doc.worker_settings = List::from_boxed(Box::new([worker(1, b"w1", false), worker(3, b"w3", true)]));
    doc.hosts.clear();
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))));
    (root, me, watch)
}
fn act(root: &mut Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(pages::Act::Builders(act)) })
}
fn read(out: &Queue<Request>) -> Token {
    let mut reads = out.iter().filter_map(|request| match request {
        Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Read { read, what } } } => {
            assert_eq!(*what, Readable::Jig(c::Read::Settings));
            Some(*read)
        }
        Request::Push(_)
        | Request::Screen { .. }
        | Request::KeepLook { .. }
        | Request::KeepSignIn { .. }
        | Request::Tab { .. }
        | Request::Copy { .. }
        | Request::Client { .. }
        | Request::Unlinked { .. }
        | Request::Show { .. }
        | Request::Leave { .. }
        | Request::SignOut
        | Request::Dropped { .. }
        | Request::Fact { .. } => None,
    });
    let read = reads.next().expect("a press reads the newest settings before sending");
    assert!(reads.next().is_none());
    read
}
fn settings(root: &mut Domain, read: Token, newest: u32) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::ReadDone {
                    read,
                    result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
                        newest,
                        settings: List::uncapped(),
                        held: List::uncapped(),
                        pending: List::uncapped(),
                    })))),
                },
            },
        },
    )
}
fn change(out: &Queue<Request>) -> c::ChangeSettings {
    let mut changes = out.iter().filter_map(|request| match request {
        Request::Unlinked {
            request: UnlinkedRequest::Down { down: Down::Send { request: jig_client::Ask::Jig(ask), .. } },
        } => {
            let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("not ChangeSettings: {ask:?}") };
            Some(change.clone())
        }
        Request::Push(_)
        | Request::Screen { .. }
        | Request::KeepLook { .. }
        | Request::KeepSignIn { .. }
        | Request::Tab { .. }
        | Request::Copy { .. }
        | Request::Client { .. }
        | Request::Unlinked { .. }
        | Request::Show { .. }
        | Request::Leave { .. }
        | Request::SignOut
        | Request::Dropped { .. }
        | Request::Fact { .. } => None,
    });
    let change = changes.next().expect("one ChangeSettings");
    assert!(changes.next().is_none());
    change
}
fn answer(root: &mut Domain, outcome: Outcome) {
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: root.pages().serial };
    crate::route::told(
        root,
        &fixtures::env(&fixtures::limits()),
        Told::Answered { key, outcome },
        &mut Queue::uncapped(),
    );
}
fn added(root: &mut Domain, name: &[u8]) {
    act(root, Act::Name(name.into()));
    act(root, Act::AddPlaces(b"4".as_slice().into()));
    act(root, Act::Digest(b"abababababababababababababababababababababababababababababababab".as_slice().into()));
}
#[test]
fn a_places_change_reads_the_newest_revision_and_keeps_the_worker_whole() {
    let (mut root, _, _) = ready();
    act(&mut root, Act::Places { number: 1, value: b"8".as_slice().into() });
    let out = act(&mut root, Act::Change(1));
    let reading = read(&out);
    assert!(root.pages().builders().unwrap().pending(Some(1)));
    assert!(act(&mut root, Act::Change(1)).is_empty());
    let sent = settings(&mut root, reading, 17);
    let change = change(&sent);
    assert_eq!(change.against, 17);
    assert_eq!(change.running, c::Running::Stays);
    assert_eq!(
        change.items.as_slice(),
        &[c::SettingItem {
            setting: c::Setting::Worker(c::SettingNumber { number: 1 }),
            edit: c::SettingEdit::Set(c::SettingSet {
                value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
                    number: 1,
                    name: b"w1".as_slice().into(),
                    places: 8,
                    digest: Some([0xab; 32]),
                    withdrawn: false,
                }))
            }),
        }]
    );
    let Read::Hosts(doc) = root.pages().builders().unwrap().read(root.client()) else { panic!("deployment") };
    assert_eq!(doc.worker_settings.get(0).unwrap().places, 4, "no optimistic setting change");
    assert!(act(&mut root, Act::Change(1)).is_empty());
}
#[test]
fn remove_sends_removed_and_the_box_cannot_be_removed() {
    let (mut root, _, watch) = ready();
    let out = act(&mut root, Act::Remove(1));
    let sent = settings(&mut root, read(&out), 7);
    assert_eq!(
        change(&sent).items.as_slice(),
        &[c::SettingItem {
            setting: c::Setting::Worker(c::SettingNumber { number: 1 }),
            edit: c::SettingEdit::Removed,
        }]
    );
    answer(
        &mut root,
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
            c::OutcomeSettingsChanged { revision: 8 },
        )))),
    );
    let Read::Hosts(doc) = root.pages().builders().unwrap().read(root.client()) else { panic!("deployment") };
    let mut doc = doc.clone();
    doc.worker_settings.get_mut(0).unwrap().digest = None;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))));
    assert!(act(&mut root, Act::Remove(1)).is_empty());
}
#[test]
fn add_uses_a_free_number_or_the_withdrawn_names_number_and_clears_on_success() {
    for (name, number) in [(b"w2".as_slice(), 2), (b"w3", 3)] {
        let (mut root, _, _) = ready();
        added(&mut root, name);
        let out = act(&mut root, Act::Add);
        let sent = settings(&mut root, read(&out), 11);
        assert_eq!(
            change(&sent),
            c::ChangeSettings {
                against: 11,
                running: c::Running::Stays,
                items: List::from_boxed(Box::new([c::SettingItem {
                    setting: c::Setting::Worker(c::SettingNumber { number }),
                    edit: c::SettingEdit::Set(c::SettingSet {
                        value: c::SettingValue::Worker(Box::new(c::WorkerSetting {
                            number,
                            name: name.into(),
                            places: 4,
                            digest: Some([0xab; 32]),
                            withdrawn: false,
                        }))
                    }),
                }])),
            }
        );
        answer(
            &mut root,
            Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
                c::OutcomeSettingsChanged { revision: 12 },
            )))),
        );
        let page = root.pages().builders().unwrap();
        assert!(!page.busy());
        assert!(page.name().is_empty() && page.add_places().is_empty() && page.digest().is_empty());
    }
}
#[test]
fn forms_and_owner_gate_the_asks() {
    let (mut root, me, _) = ready();
    for bad in [
        Act::Name(b"-w2".as_slice().into()),
        Act::Name(b"w1".as_slice().into()),
        Act::AddPlaces(b"0".as_slice().into()),
        Act::AddPlaces(b"4294967296".as_slice().into()),
        Act::Digest(b"ABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABAB".as_slice().into()),
        Act::Digest(b"ab".as_slice().into()),
    ] {
        added(&mut root, b"w2");
        act(&mut root, bad);
        assert!(act(&mut root, Act::Add).is_empty());
    }
    for value in [b"4".as_slice(), b"0", b"-1", b"8x"] {
        act(&mut root, Act::Places { number: 1, value: value.into() });
        assert!(act(&mut root, Act::Change(1)).is_empty());
    }
    deliver(&mut root, me, frame(Some(c::HeldRole::Policy { role: 1 })));
    added(&mut root, b"w2");
    assert!(act(&mut root, Act::Add).is_empty());
    assert!(act(&mut root, Act::Remove(1)).is_empty());
    assert!(act(&mut root, Act::Places { number: 1, value: b"8".as_slice().into() }).is_empty());
}
#[test]
fn a_refusal_stays_at_its_row_or_form_with_the_fields_kept() {
    for row in [Some(1), None] {
        let (mut root, _, _) = ready();
        added(&mut root, b"w2");
        act(&mut root, Act::Places { number: 1, value: b"8".as_slice().into() });
        let action = if row.is_some() { Act::Remove(1) } else { Act::Add };
        let out = act(&mut root, action.clone());
        settings(&mut root, read(&out), 1);
        let refused = Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Malformed {
                at: 0,
                why: c::Malformed::Settings(c::MalformedSettings {
                    setting: c::Setting::Worker(c::SettingNumber { number: 1 }),
                    fault: c::SettingFault::Linked,
                    detail: None,
                }),
            },
            at: None,
        }));
        answer(&mut root, refused.clone());
        let page = root.pages().builders().unwrap();
        assert_eq!(page.refused(row), Some(&refused));
        assert!(!page.busy());
        assert_eq!(page.name(), b"w2");
        assert!(page.refused(Some(3)).is_none());
        act(&mut root, action);
        assert!(root.pages().builders().unwrap().refused(row).is_none());
    }
}
