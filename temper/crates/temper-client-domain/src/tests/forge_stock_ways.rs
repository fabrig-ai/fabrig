//! Ask temper's watched ways and their settings decisions (client/README.md, 4.6 and 5.13).
use super::fixtures as f;
use crate::{
    Address, Domain, Event, Place, Request,
    forge::stock_ways::{self, State, WayAct},
    pages,
};
use jig_client::{Down, Key, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

const OTHER: u32 = 17;
const NEWEST: u32 = 8;
const NEXT: u32 = 9;
// The fixture has two ways, each with two items, and at most one Yours tab.
const WAYS: u32 = 2;
const ITEMS: u32 = 2;
// client/README.md, 4.6 bounds a path at 24 stops.
const STOPS: u32 = 24;
const TABS: u32 = 3;
// Each act is one ChangeSettings decision.
const DECISIONS: usize = 1;
const WORDS: &[(&[u8], &[u8], &[u8])] =
    &[(b"You approve", b"You decide", b"approval"), (b"Agents review", b"Code lands", b"review")];
struct Fixture {
    root: Domain,
    watch: Token,
    settings: Token,
}
fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn go(root: &mut Domain, address: Address) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(address) })
}
fn opened(out: &Queue<Request>, wanted: &Watchable) -> Option<Token> {
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && subject == wanted
        {
            return Some(*watch);
        }
    }
    None
}
fn fixture() -> Fixture {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::with_capacity(0));
    up(
        &mut root,
        Up::FamilyServed {
            family: wire::FORGE,
            versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
        },
    );
    let out = up(
        &mut root,
        Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(f::chats::me()))) },
    );
    let settings = opened(&out, &Watchable::Jig(c::Subject::Settings)).expect("session settings watch");
    let out = go(&mut root, Address::Ask { project: f::PROJECT });
    let watch = opened(&out, &stock_ways::watch(f::PROJECT, &f::limits().forge).unwrap()).expect("Ask ways watch");
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    let mut fixture = Fixture { root, watch, settings };
    document(&mut fixture, NEWEST, false);
    fixture
}
fn document(f: &mut Fixture, newest: u32, configured: bool) {
    let mut settings = List::uncapped();
    if configured {
        settings
            .push(c::SettingLine {
                setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 1 },
                name: None,
                last: Some(newest),
                follows: c::SettingFollows::Change,
                application: None,
            })
            .unwrap();
    }
    up(
        &mut f.root,
        Up::Watched {
            watch: f.settings,
            event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest,
                settings,
                held: List::uncapped(),
                pending: List::uncapped(),
            }))),
        },
    );
}
fn path(branch: &[u8]) -> w::BranchPath {
    w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts {
            repository: Some(b"owner/repo".as_slice().into()),
            branch: branch.into(),
            stops: List::with_capacity(STOPS),
        },
    )
    .unwrap()
}
fn panel(project: u32, current: Option<u32>, may_change: bool) -> w::WaysPanel {
    let mut ways = List::with_capacity(WAYS);
    for (name, line, branch) in WORDS {
        let mut items = List::with_capacity(ITEMS);
        for (kind, number, edit) in [
            (2, 4, w::SettingEdit::Removed),
            (
                7,
                1,
                w::SettingEdit::Set(
                    w::SettingValue::new(
                        &w::CEILINGS,
                        w::SettingValueParts { value: b"whole value".as_slice().into() },
                    )
                    .unwrap(),
                ),
            ),
        ] {
            items
                .push(
                    w::ProposalItem::new(&w::CEILINGS, w::ProposalItemParts { kind, scope: project, number, edit })
                        .unwrap(),
                )
                .unwrap();
        }
        ways.push(
            w::StockWay::new(
                &w::CEILINGS,
                w::StockWayParts {
                    name: (*name).into(),
                    line: (*line).into(),
                    path: path(branch),
                    items,
                    delegation: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    w::WaysPanel::new(&w::CEILINGS, w::WaysPanelParts { project, ways, current, own: path(b"own"), may_change })
        .unwrap()
}
fn show(f: &mut Fixture, panel: &w::WaysPanel) -> Queue<Request> {
    family(f, wire::FORGE, wire::CLIENT_VERSION, wire::ways_panel(panel))
}
fn family(f: &mut Fixture, family: u16, version: u32, bytes: Box<[u8]>) -> Queue<Request> {
    up(&mut f.root, Up::FamilyDelivered { watch: f.watch, record: jig_client::Family { family, version, bytes } })
}
fn page_event(f: &mut Fixture, event: pages::Event) -> Queue<pages::Request> {
    let mut out = Queue::uncapped();
    pages::step(&mut f.root.pages, &f::env(&f::limits()), &f.root.client, event, &mut out);
    out
}
fn act(f: &mut Fixture, act: pages::Act) -> Queue<pages::Request> {
    page_event(f, pages::Event::Act { act })
}
fn sends(out: Queue<pages::Request>) -> Box<[(Key, c::ChangeSettings)]> {
    let mut sent = List::uncapped();
    for request in &out {
        if let pages::Request::Client(jig_client::Event::Send { key, request: jig_client::Ask::Jig(ask), .. }) = request
        {
            let c::Ask::ChangeSettings(change) = ask.as_ref() else {
                panic!("only changes settings");
            };
            sent.push((*key, change.clone())).unwrap();
        }
    }
    sent.into_boxed()
}
fn use_way(f: &mut Fixture) -> (Key, c::ChangeSettings) {
    sends(act(f, pages::Act::UseWay { project: f::PROJECT, way: 1 })).first().expect("offered way sends").clone()
}
fn answer(f: &mut Fixture, key: Key, answer: c::Answer) -> Queue<pages::Request> {
    page_event(
        f,
        pages::Event::Told {
            told: Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) },
        },
    )
}
fn refused(refusal: c::Refusal) -> c::Answer {
    c::Answer::Refused { refusal, at: None }
}
fn stale() -> c::Refusal {
    c::Refusal::Stale(c::RefusalStale { newest: NEXT })
}
fn closed(out: &Queue<Request>, wanted: Token) -> bool {
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && *watch == wanted
        {
            return true;
        }
    }
    false
}

#[test]
fn ways_watch_has_project_scope_and_the_pages_lifetime() {
    let Watchable::Family { scope, window, record } = stock_ways::watch(f::PROJECT, &f::limits().forge).unwrap() else {
        panic!("family watch");
    };
    assert_eq!(scope, c::Scope::Project(f::PROJECT));
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(
        wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch(),
        &w::Watch::Ways(w::WaysSubject::new(&w::CEILINGS, w::WaysSubjectParts { project: f::PROJECT }).unwrap())
    );
    for input in [
        Event::Went { place: Place::Address(Address::Home { project: f::PROJECT }) },
        Event::Went { place: Place::Address(Address::Ask { project: OTHER }) },
        Event::Went { place: Place::Missing },
    ] {
        let mut f = fixture();
        assert_eq!(f.root.pages().stock_ways().unwrap().state(), &State::Loading);
        let out = event(&mut f.root, input);
        assert!(closed(&out, f.watch));
    }
    let mut f = fixture();
    let out = go(&mut f.root, Address::Ask { project: f::PROJECT });
    assert!(!closed(&out, f.watch));
}

#[test]
fn tabs_preserve_the_reads_order_and_append_yours() {
    for (current, expected) in [(Some(1), [false, true].as_slice()), (None, [false, false, true].as_slice())] {
        let mut f = fixture();
        show(&mut f, &panel(f::PROJECT, current, true));
        let tabs = f.root.pages().stock_ways().unwrap().tabs();
        let mut current_tabs = List::with_capacity(TABS);
        for tab in &tabs {
            current_tabs.push(tab.current).unwrap();
        }
        assert_eq!(current_tabs.as_slice(), expected);
        for (index, (name, line, branch)) in WORDS.iter().enumerate() {
            let tab = tabs.get(u32::try_from(index).unwrap()).unwrap();
            assert_eq!(tab.index, Some(u32::try_from(index).unwrap()));
            assert_eq!((&*tab.name, &*tab.line, &tab.path), (*name, *line, &path(branch)));
        }
        if current.is_none() {
            let yours = tabs.get(TABS.checked_sub(1).unwrap()).unwrap();
            assert_eq!(
                (&*yours.name, &*yours.line, &yours.path, yours.index),
                (b"Yours".as_slice(), b"".as_slice(), &path(b"own"), None)
            );
        }
    }
}

#[test]
fn acts_follow_current_step_permission_and_pending_state() {
    for (name, current, configured, may_change, pending, expected) in [
        ("step left", Some(0), false, true, false, [Some(WayAct::Keep), Some(WayAct::Use), None]),
        ("step done", Some(0), true, true, false, [None, Some(WayAct::Use), None]),
        ("Yours", None, false, true, false, [Some(WayAct::Use), Some(WayAct::Use), None]),
        ("read only", Some(0), false, false, false, [None, None, None]),
        ("act out", Some(0), false, true, true, [None, None, None]),
    ] {
        let mut f = fixture();
        document(&mut f, NEWEST, configured);
        show(&mut f, &panel(f::PROJECT, current, may_change));
        if pending {
            use_way(&mut f);
        }
        let page = f.root.pages().stock_ways().unwrap();
        assert_eq!([page.act(0), page.act(1), page.act(2)], expected, "{name}");
    }
}

#[test]
fn use_and_keep_send_one_ordered_decision_at_newest() {
    for (current, action) in [
        (Some(0), pages::Act::UseWay { project: f::PROJECT, way: 1 }),
        (Some(1), pages::Act::KeepWay { project: f::PROJECT }),
    ] {
        let mut f = fixture();
        show(&mut f, &panel(f::PROJECT, current, true));
        document(&mut f, NEXT, false);
        let sent = sends(act(&mut f, action));
        assert_eq!(sent.len(), DECISIONS);
        let change = &sent.first().unwrap().1;
        assert_eq!((change.against, &change.running), (NEXT, &c::Running::Stays));
        assert_eq!(
            change.items.as_slice(),
            &[
                c::SettingItem {
                    setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 4 },
                    edit: c::SettingEdit::Removed
                },
                c::SettingItem {
                    setting: c::Setting::Application { kind: 7, scope: f::PROJECT, number: 1 },
                    edit: c::SettingEdit::Set(c::SettingSet {
                        value: c::SettingValue::Application(b"whole value".as_slice().into())
                    })
                },
            ]
        );
    }
    for (current, configured, may_change, action) in [
        (Some(0), false, true, pages::Act::UseWay { project: f::PROJECT, way: 0 }),
        (Some(0), true, true, pages::Act::KeepWay { project: f::PROJECT }),
        (None, false, true, pages::Act::KeepWay { project: f::PROJECT }),
        (Some(0), false, false, pages::Act::UseWay { project: f::PROJECT, way: 1 }),
        (Some(0), false, true, pages::Act::UseWay { project: OTHER, way: 1 }),
        (Some(0), false, true, pages::Act::KeepWay { project: OTHER }),
        (Some(0), false, true, pages::Act::UseWay { project: f::PROJECT, way: WAYS }),
    ] {
        let mut f = fixture();
        document(&mut f, NEWEST, configured);
        show(&mut f, &panel(f::PROJECT, current, may_change));
        assert!(sends(act(&mut f, action)).is_empty());
    }
    let mut f = fixture();
    show(&mut f, &panel(f::PROJECT, Some(0), true));
    use_way(&mut f);
    assert!(sends(act(&mut f, pages::Act::KeepWay { project: f::PROJECT })).is_empty());
}

#[test]
fn stale_retries_once_and_refusals_last_until_an_act_or_document() {
    for refusal in [
        stale(),
        c::Refusal::Role { held: Some(c::HeldRole::Observer) },
        c::Refusal::Judged {
            at: 0,
            setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 1 },
            why: c::Text::new(b"The forge needs two approvers".as_slice().into(), 64).unwrap(),
        },
    ] {
        let mut f = fixture();
        show(&mut f, &panel(f::PROJECT, Some(0), true));
        let (key, first) = use_way(&mut f);
        let sent = sends(answer(&mut f, key, refused(stale())));
        assert_eq!(sent.len(), DECISIONS);
        let (key, retry) = sent.first().unwrap();
        assert_eq!((retry.against, &retry.items, &retry.running), (NEXT, &first.items, &c::Running::Stays));
        assert!(sends(answer(&mut f, *key, refused(refusal.clone()))).is_empty());
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), Some(&refusal));
        document(&mut f, NEXT, false);
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), Some(&refusal));
        let (key, _) = use_way(&mut f);
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
        answer(&mut f, key, refused(c::Refusal::Provider));
        show(&mut f, &panel(f::PROJECT, Some(1), true));
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
    }
    let mut f = fixture();
    show(&mut f, &panel(f::PROJECT, Some(0), true));
    let (key, _) = use_way(&mut f);
    answer(&mut f, key, c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: NEXT })));
    assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
    assert_eq!(f.root.pages().stock_ways().unwrap().act(1), Some(WayAct::Use));
    let (key, _) = use_way(&mut f);
    answer(&mut f, key, refused(c::Refusal::Provider));
    up(&mut f.root, Up::WatchEnded { watch: f.watch, end: c::WatchEnd::Dropped });
    assert_eq!(
        f.root.pages().stock_ways().unwrap().refused(),
        Some(&c::Refusal::Provider),
        "a behind watch has no newer document"
    );
}

#[test]
fn wrong_project_invalid_or_ended_documents_end_the_watch() {
    for (family_id, version, bytes) in [
        (wire::FORGE, wire::CLIENT_VERSION, wire::ways_panel(&panel(OTHER, Some(0), true))),
        (wire::FORGE, wire::CLIENT_VERSION, Box::from(b"undecodable".as_slice())),
        (
            wire::FORGE,
            wire::CLIENT_VERSION.checked_add(1).unwrap(),
            wire::ways_panel(&panel(f::PROJECT, Some(0), true)),
        ),
    ] {
        let mut f = fixture();
        show(&mut f, &panel(f::PROJECT, Some(0), true));
        family(&mut f, family_id, version, bytes);
        let page = f.root.pages().stock_ways().unwrap();
        assert_eq!(page.state(), &State::Ended);
        assert!(page.tabs().is_empty());
        assert_eq!(page.act(0), None);
    }
    let mut f = fixture();
    up(&mut f.root, Up::WatchEnded { watch: f.watch, end: c::WatchEnd::SubjectEnded });
    assert_eq!(f.root.pages().stock_ways().unwrap().state(), &State::Ended);
}
