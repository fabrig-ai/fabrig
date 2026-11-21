//! Scripted GitHub answers and forges documents, without a browser or a forge.
#![expect(clippy::disallowed_methods, reason = "test assertions inspect scripted documents and emitted requests")]
use super::{connect, fixtures as f};
use crate::{Address, Choice, Domain, Event, Opening, Place, Request, Step, Tab, forge};
use jig_client::{Answered, Down, Family, Outcome, Told, Up};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const LINK: &[u8] = b"http://127.0.0.1:8367/forges/2/github/manifest?state=fixture";

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn go(root: &mut Domain, address: Address) {
    event(root, Event::Went { place: Place::Address(address) });
}
fn row(app: w::AppWay, ids: &[(u64, bool, bool)], ended: Option<&[u8]>) -> w::ForgeRow {
    let mut installations = List::uncapped();
    for (id, stray, suspended) in ids {
        installations.push(w::Installation { id: *id, account: None, stray: *stray, suspended: *suspended }).unwrap();
    }
    w::ForgeRow {
        starts_with: w::StartsWith::Nothing,
        makings: List::with_capacity(0),
        number: 2,
        host: b"github.example".as_slice().into(),
        kind: w::Kind::Github,
        mode: w::Mode::Connected,
        connection: w::Connection::Connected(w::Connected { id: 9, login: b"app[bot]".as_slice().into() }),
        offer: w::Offer::NotOffered,
        first: None,
        installations,
        ended: ended.map(|why| w::ConnectStopped { why: why.into() }),
        app: Some(app),
        web: None,
        repositories: List::with_capacity(0),
        owned: None,
        admin: false,
        read_only: false,
        ended_stop: None,
    }
}
fn document(root: &mut Domain, watch: Token, row: w::ForgeRow) -> Queue<Request> {
    let mut rows = List::with_capacity(1);
    rows.push(row).unwrap();
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(&w::Forges { forges: rows, full: List::with_capacity(0), linked: false }),
                    },
                },
            },
        },
    )
}
fn press(root: &mut Domain, act: forge::Act) -> w::Request {
    let mut out = event(root, Event::Forge(forge::Event::Act { act }));
    assert_eq!(out.pop(), Some(Request::Tab { tab: Tab::Open { opening: Opening::GitHub } }));
    let mut sent = None;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { request, .. } } } =
            request
        {
            assert!(sent.is_none(), "one connect or install");
            let jig_client::Ask::Family { record, .. } = request else { panic!("family") };
            sent = Some(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request);
        }
        if let Request::Tab { .. } | Request::Leave { .. } = request {
            panic!("no other departure at the press");
        }
    }
    sent.expect("the press sends its request after opening the tab")
}
fn answered(root: &mut Domain, outcome: Outcome) -> Queue<Request> {
    let key = root.forge().app().unwrap().key;
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut out);
    out
}
fn outcome(outcome: w::Outcome) -> Outcome {
    Outcome::Answered(Answered::Family(Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer { outcome }),
    }))
}
fn link(root: &mut Domain) -> Queue<Request> {
    answered(root, outcome(w::Outcome::Link(w::Link { address: LINK.into() })))
}
fn tabs(out: &Queue<Request>) -> List<Tab> {
    let mut tabs = List::uncapped();
    for request in out {
        if let Request::Leave { .. } = request {
            panic!("GitHub never leaves temper's page");
        }
        if let Request::Tab { tab } = request {
            tabs.push(tab.clone()).unwrap();
        }
    }
    tabs
}
fn waiting(root: &Domain) -> bool {
    root.forge().app().unwrap().way.is_some()
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the fixture refuses every request except the two GitHub departures"
)]
fn github_departures_open_before_sending_without_back_and_point_only_once() {
    for (address, act, app, install) in [
        (
            Address::Welcome { step: Step::Forge { forge: None } },
            forge::Act::Forges(forge::forges::Act::GitHub),
            None,
            false,
        ),
        (Address::ConnectForge { forge: None }, forge::Act::Forges(forge::forges::Act::GitHub), None, false),
        (
            Address::Welcome { step: Step::Forge { forge: None } },
            forge::Act::Forges(forge::forges::Act::GoOn { forge: 2 }),
            Some(w::AppWay::Manifest),
            false,
        ),
        (
            Address::Welcome { step: Step::Forge { forge: None } },
            forge::Act::Forges(forge::forges::Act::GoOn { forge: 2 }),
            Some(w::AppWay::Pasted),
            true,
        ),
        (
            Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(2)) } },
            forge::Act::Install { forge: 2 },
            Some(w::AppWay::Manifest),
            true,
        ),
        (Address::Forges, forge::Act::Install { forge: 2 }, Some(w::AppWay::Pasted), true),
    ] {
        let (mut root, _, watch) = connect::opened(true);
        if let Some(app) = app {
            document(&mut root, watch, row(app, &[], None));
        }
        go(&mut root, address.clone());
        match press(&mut root, act) {
            w::Request::Install(asked) => {
                assert!(install);
                assert_eq!(asked.forge, 2);
                assert_eq!(asked.back, None);
            }
            w::Request::Connect(asked) => {
                assert!(!install);
                let w::How::GithubManifest(manifest) = asked.how else { panic!("manifest") };
                assert_eq!(manifest.back, None);
            }
            _ => panic!("connect or install"),
        }
        let point = if install { Tab::Point { to: LINK.into() } } else { Tab::PointOnOrigin { to: LINK.into() } };
        assert_eq!(tabs(&link(&mut root)).as_slice(), &[point]);
        assert_eq!(root.pages().address(), &address);
        assert_eq!(root.forge().app().unwrap().way.as_ref().unwrap().address.as_deref(), Some(LINK));
        assert!(tabs(&link(&mut root)).is_empty(), "a repeated answer never points again");
    }
}
#[test]
fn every_answer_without_a_link_closes_the_github_tab() {
    for answer in [
        outcome(w::Outcome::Connecting),
        outcome(w::Outcome::Unknown),
        outcome(w::Outcome::ConnectRefused(w::ConnectRefused { why: w::ForgeRefusal::Busy })),
        Outcome::Lost,
    ] {
        let (mut root, _, _) = connect::opened(true);
        press(&mut root, forge::Act::Forges(forge::forges::Act::GitHub));
        assert_eq!(tabs(&answered(&mut root, answer)).as_slice(), &[Tab::Close]);
        assert!(!waiting(&root));
    }
}
#[test]
fn a_refused_github_tab_keeps_its_answer_as_a_link_without_pointing() {
    let (mut root, _, _) = connect::opened(true);
    press(&mut root, forge::Act::Forges(forge::forges::Act::GitHub));
    event(&mut root, Event::TabRefused);
    assert!(root.forge().app().unwrap().way.as_ref().unwrap().refused);
    assert!(tabs(&link(&mut root)).is_empty());
    assert_eq!(root.forge().app().unwrap().way.as_ref().unwrap().address.as_deref(), Some(LINK));
}
#[test]
fn an_old_ending_waits_until_a_document_clears_it_then_the_next_ending_opens_the_row() {
    let (mut root, _, watch) = connect::opened(true);
    document(&mut root, watch, row(w::AppWay::Pasted, &[(2, false, false)], Some(b"old")));
    go(&mut root, Address::ConnectForge { forge: None });
    press(&mut root, forge::Act::Install { forge: 2 });
    link(&mut root);
    document(&mut root, watch, row(w::AppWay::Pasted, &[(2, false, false)], Some(b"old")));
    assert!(waiting(&root), "an ending from before the press is not this way's end");
    document(&mut root, watch, row(w::AppWay::Pasted, &[(2, false, false)], None));
    assert!(waiting(&root));
    let why = b"the link to GitHub expired before it was finished";
    let out = document(&mut root, watch, row(w::AppWay::Pasted, &[(2, false, false)], Some(why)));
    assert!(!waiting(&root));
    assert_eq!(root.pages().address(), &Address::Forges);
    assert_eq!(root.pages().forges_opened(), Some(2));
    assert!(out.iter().any(|request| *request == Request::Show { address: Address::Forges, replace: true }));
    let forge::forges::State::Rows(rows) = root.forges() else { panic!("rows") };
    assert_eq!(rows.first().unwrap().ended.as_deref(), Some(why.as_slice()));
    assert!(tabs(&out).is_empty(), "a pointed tab is never closed by its watch");
}
#[test]
fn only_an_installation_newly_accepted_since_the_press_ends_the_way() {
    for new in [false, true] {
        let (mut root, _, watch) = connect::opened(true);
        go(&mut root, Address::ConnectForge { forge: None });
        if new {
            press(&mut root, forge::Act::Forges(forge::forges::Act::GitHub));
        } else {
            document(&mut root, watch, row(w::AppWay::Manifest, &[(2, false, false)], None));
            press(&mut root, forge::Act::Install { forge: 2 });
        }
        link(&mut root);
        let kept = if new { &[][..] } else { &[(2, false, false)][..] };
        document(&mut root, watch, row(w::AppWay::Manifest, kept, None));
        assert!(waiting(&root));
        let mut ids = List::with_capacity(3);
        for id in kept {
            ids.push(*id).unwrap();
        }
        ids.push((3, true, false)).unwrap();
        ids.push((4, false, true)).unwrap();
        document(&mut root, watch, row(w::AppWay::Manifest, ids.as_slice(), None));
        assert!(waiting(&root), "strays and suspended installations do not finish the way");
        ids.iter_mut().nth(kept.len()).unwrap().1 = false;
        let out = document(&mut root, watch, row(w::AppWay::Manifest, ids.as_slice(), None));
        assert!(!waiting(&root));
        assert_eq!(root.forge().app().unwrap().forge, 2, "a new forge learns its row number");
        assert_eq!(root.pages().address(), &Address::Forges);
        assert_eq!(root.pages().forges_opened(), Some(2));
        assert!(tabs(&out).is_empty());
    }
}

fn without_app(ended: Option<&[u8]>) -> w::ForgeRow {
    let mut row = row(w::AppWay::Manifest, &[], ended);
    row.app = None;
    row.connection = w::Connection::NotConnected;
    row
}
fn no_departure(out: &Queue<Request>) {
    assert!(tabs(out).is_empty());
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
            panic!("a document never asks for an install");
        }
    }
}
#[test]
fn a_connect_reopens_as_install_only_after_its_missing_app_appears() {
    const INSTALL: &[u8] = b"https://github.example/install";
    for refused in [false, true] {
        let (mut root, _, watch) = connect::opened(true);
        go(&mut root, Address::Welcome { step: Step::Forge { forge: None } });
        press(&mut root, forge::Act::Forges(forge::forges::Act::GitHub));
        if refused {
            event(&mut root, Event::TabRefused);
        }
        link(&mut root);
        no_departure(&document(&mut root, watch, without_app(Some(b"old"))));
        let pending = root.forge().app().unwrap();
        assert_eq!(pending.forge, 2);
        assert_eq!(pending.install_again(), None);
        assert_eq!(pending.way.as_ref().unwrap().address.as_deref(), Some(LINK));
        assert_eq!(pending.way.as_ref().unwrap().refused, refused);

        no_departure(&document(&mut root, watch, row(w::AppWay::Manifest, &[], Some(b"old"))));
        let forge = root.forge().app().unwrap().install_again().expect("the made App reopens as Install");
        assert_eq!(forge, 2);
        // Once made, the App's appearance is kept even if a later row omits it.
        no_departure(&document(&mut root, watch, without_app(Some(b"old"))));
        assert_eq!(root.forge().app().unwrap().install_again(), Some(forge));
        no_departure(&document(&mut root, watch, row(w::AppWay::Manifest, &[], Some(b"old"))));
        assert_eq!(
            press(&mut root, forge::Act::Install { forge }),
            w::Request::Install(w::Install { forge, back: None })
        );
        assert_eq!(root.forge().app().unwrap().kind, Some(forge::github::WayKind::Install));
        assert_eq!(root.forge().app().unwrap().way.as_ref().unwrap().app, forge::github::WayApp::AtPress);
        assert_eq!(root.forge().app().unwrap().install_again(), None);
        let answer = outcome(w::Outcome::Link(w::Link { address: INSTALL.into() }));
        assert_eq!(tabs(&answered(&mut root, answer.clone())).as_slice(), &[Tab::Point { to: INSTALL.into() }]);
        assert!(tabs(&answered(&mut root, answer)).is_empty());
        document(&mut root, watch, row(w::AppWay::Manifest, &[], Some(b"old")));
        assert!(waiting(&root), "an uncleared ending is not this install's end");
        document(&mut root, watch, row(w::AppWay::Manifest, &[(7, false, false)], Some(b"old")));
        assert!(!waiting(&root), "an installation accepted since the Install press ends its way");
    }
}
#[test]
fn an_install_keeps_its_link_and_a_connect_with_an_app_does_not_switch() {
    for install in [false, true] {
        let (mut root, _, watch) = connect::opened(true);
        document(&mut root, watch, row(w::AppWay::Manifest, &[(2, false, false)], Some(b"old")));
        go(&mut root, Address::Forges);
        press(
            &mut root,
            if install { forge::Act::Install { forge: 2 } } else { forge::Act::ConnectManifest { forge: 2 } },
        );
        link(&mut root);
        for app in [None, Some(w::AppWay::Manifest), Some(w::AppWay::Pasted)] {
            let mut row = row(w::AppWay::Manifest, &[(2, false, false)], Some(b"old"));
            row.app = app;
            no_departure(&document(&mut root, watch, row));
            let pending = root.forge().app().unwrap();
            assert_eq!(pending.install_again(), None);
            let way = pending.way.as_ref().unwrap();
            assert_eq!(way.app, forge::github::WayApp::AtPress);
            assert_eq!(way.address.as_deref(), Some(LINK));
        }
    }
}
#[test]
fn reopening_counts_the_installs_end_from_its_own_press_and_closes_on_no_link() {
    let (mut root, _, watch) = connect::opened(true);
    go(&mut root, Address::Welcome { step: Step::Forge { forge: None } });
    press(&mut root, forge::Act::Forges(forge::forges::Act::GitHub));
    link(&mut root);
    document(&mut root, watch, row(w::AppWay::Manifest, &[], None));
    assert!(root.forge().app().unwrap().way.as_ref().unwrap().cleared);
    let forge = root.forge().app().unwrap().install_again().unwrap();
    press(&mut root, forge::Act::Install { forge });
    link(&mut root);
    document(&mut root, watch, row(w::AppWay::Manifest, &[], Some(b"not cleared since this press")));
    assert!(waiting(&root), "the connect's cleared line does not count for the Install");
    document(&mut root, watch, row(w::AppWay::Manifest, &[], None));
    document(&mut root, watch, row(w::AppWay::Manifest, &[], Some(b"this install ended")));
    assert!(!waiting(&root));

    // Another connect starts without an App; its reopen still closes on a no-link answer.
    document(&mut root, watch, without_app(Some(b"old")));
    press(&mut root, forge::Act::ConnectManifest { forge });
    link(&mut root);
    document(&mut root, watch, row(w::AppWay::Manifest, &[], Some(b"old")));
    let forge = root.forge().app().unwrap().install_again().unwrap();
    press(&mut root, forge::Act::Install { forge });
    assert_eq!(tabs(&answered(&mut root, outcome(w::Outcome::Connecting))).as_slice(), &[Tab::Close]);
    assert!(!waiting(&root));
}

#[test]
fn an_ending_before_the_answer_closes_only_the_unpointed_tab() {
    let (mut root, _, watch) = connect::opened(true);
    document(&mut root, watch, row(w::AppWay::Pasted, &[], None));
    go(&mut root, Address::ConnectForge { forge: Some(Choice::Forge(2)) });
    press(&mut root, forge::Act::Install { forge: 2 });
    document(&mut root, watch, row(w::AppWay::Pasted, &[], None));
    go(&mut root, Address::Accounts);
    let out = document(&mut root, watch, row(w::AppWay::Pasted, &[], Some(b"the link expired")));
    assert!(!waiting(&root));
    assert_eq!(root.pages().address(), &Address::Accounts, "the watch does not take over another page");
    assert_eq!(tabs(&out).as_slice(), &[Tab::Close]);
    assert!(tabs(&link(&mut root)).is_empty(), "the late answer cannot point a closed tab");
}
