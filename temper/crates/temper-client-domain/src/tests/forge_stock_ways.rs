//! Ask temper's watched ways and their settings decisions (client/forge/records.md and client/pages/settings.md, 2).
use super::fixtures as f;
use crate::{
    Address, Domain, Event, Place, Request,
    forge::stock_ways::{self, State, WayAct},
    pages,
};
use jig_client::{Down, Handle, Key, Readable, Readout, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

const OTHER: u32 = 17;
const NEWEST: u32 = 8;
const NEXT: u32 = 9;
// The fixture has two ways, each with two items, and at most one Yours tab.
const WAYS: u32 = 2;
const ITEMS: u32 = 2;
// client/forge/records.md bounds a path at 24 stops.
const STOPS: u32 = 24;
const TABS: u32 = 3;
// Each act is one ChangeSettings decision.
const DECISIONS: usize = 1;
const OLD: &[c::AuthorityExecutor] = &[c::AuthorityExecutor::Charter(1), c::AuthorityExecutor::Procedure(5)];
const PARTIAL: &[c::AuthorityExecutor] =
    &[c::AuthorityExecutor::Charter(1), c::AuthorityExecutor::Procedure(5), c::AuthorityExecutor::Role(0)];
const COMPLETE: &[c::AuthorityExecutor] = &[
    c::AuthorityExecutor::Charter(1),
    c::AuthorityExecutor::Procedure(5),
    c::AuthorityExecutor::Charter(5),
    c::AuthorityExecutor::Procedure(6),
    c::AuthorityExecutor::Role(0),
    c::AuthorityExecutor::Job(3),
    c::AuthorityExecutor::Job(4),
];
const COMPLETED_PARTIAL: &[c::AuthorityExecutor] = &[
    c::AuthorityExecutor::Charter(1),
    c::AuthorityExecutor::Procedure(5),
    c::AuthorityExecutor::Role(0),
    c::AuthorityExecutor::Charter(5),
    c::AuthorityExecutor::Procedure(6),
    c::AuthorityExecutor::Job(3),
    c::AuthorityExecutor::Job(4),
];
struct WayWords {
    name: &'static [u8],
    headline: &'static [u8],
    line: &'static [u8],
    branch: &'static [u8],
}
const WORDS: &[WayWords] = &[
    WayWords { name: b"You approve", headline: b"You approve every change", line: b"You decide", branch: b"approval" },
    WayWords {
        name: b"Agents review",
        headline: b"Agents review every change",
        line: b"Code lands",
        branch: b"review",
    },
];
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
    fixture_me(f::chats::me())
}
fn fixture_me(identity: c::Me) -> Fixture {
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
        Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(identity))) },
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
        w::BranchPathParts { repository: Some(b"owner/repo".as_slice().into()), branch: branch.into(), stops: stops() },
    )
    .unwrap()
}
fn stops() -> List<w::Stop> {
    let mut stops = List::with_capacity(STOPS);
    for (actor, step, words) in [
        (w::Actor::Agent, w::PathStep::Produce, b"produce".as_slice()),
        (w::Actor::Person, w::PathStep::Review, b"review".as_slice()),
        (w::Actor::Bar, w::PathStep::Branch, b"main".as_slice()),
    ] {
        stops
            .push(
                w::Stop::new(
                    &w::CEILINGS,
                    w::StopParts {
                        needed: false,
                        over: Box::default(),
                        actor,
                        step,
                        words: words.into(),
                        opens: w::Opens::Nothing,
                        card: w::Card::new(
                            &w::CEILINGS,
                            w::CardParts {
                                who: b"The actor".as_slice().into(),
                                name: Box::default(),
                                facts: List::with_capacity(0),
                            },
                        )
                        .unwrap(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    stops
}
fn panel(project: u32, current: Option<u32>, may_change: bool) -> w::WaysPanel {
    panel_delegation(project, current, may_change, false)
}
fn panel_delegation(project: u32, current: Option<u32>, may_change: bool, delegation: bool) -> w::WaysPanel {
    let mut ways = List::with_capacity(WAYS);
    for WayWords { name, headline, line, branch } in WORDS {
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
                    headline: (*headline).into(),
                    name: (*name).into(),
                    line: (*line).into(),
                    path: path(branch),
                    items,
                    delegation,
                    roles: List::with_capacity(0),
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    w::WaysPanel::new(
        &w::CEILINGS,
        w::WaysPanelParts { words: None, project, ways, current, own: path(b"own"), may_change },
    )
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
fn reads(out: &Queue<pages::Request>) -> Box<[(Handle, c::Read)]> {
    let mut reads = List::uncapped();
    for request in out {
        if let pages::Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }) = request {
            reads.push((*handle, read.clone())).unwrap();
        }
    }
    reads.into_boxed()
}
fn read_answer(f: &mut Fixture, handle: Handle, page: Readout) -> Queue<pages::Request> {
    page_event(f, pages::Event::Told { told: Told::Page { handle, page } })
}
fn policy(ceiling: &[c::AuthorityExecutor], owner: &[c::AuthorityExecutor]) -> Box<c::SettingPolicy> {
    let mut found = None;
    for value in jig_client_test_records::settings::settings_values().into_boxed() {
        match value {
            c::SettingValue::Policy(policy) => found = Some(policy),
            c::SettingValue::Charter(_)
            | c::SettingValue::Template(_)
            | c::SettingValue::Rules(_)
            | c::SettingValue::Model(_)
            | c::SettingValue::Application(_) => {}
        }
    }
    let mut policy = found.expect("whole policy fixture");
    policy.ceiling.delegation.kinds = List::from_boxed(ceiling.into());
    let mut authority = jig_client_test_records::settings::settings_authority();
    authority.delegation.kinds = List::from_boxed(owner.into());
    let mut roles = List::uncapped();
    for role in &policy.roles {
        roles.push(role.clone()).unwrap();
    }
    roles.push(c::PolicyEditRole { number: 0, authority, period_spend: 301, requests: 511, decides: 31 }).unwrap();
    policy.roles = roles;
    policy
}
fn policy_page(policy: Box<c::SettingPolicy>) -> Readout {
    policy_document(f::PROJECT, NEXT, Some(c::SettingValue::Policy(policy)))
}
fn policy_document(project: u32, revision: u32, value: Option<c::SettingValue>) -> Readout {
    Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
        setting: c::Setting::ProjectPolicy { project },
        revision,
        value,
        application: None,
        proposed: None,
    }))))
}
fn way_items(policy: Option<Box<c::SettingPolicy>>) -> List<c::SettingItem> {
    let mut items = List::uncapped();
    for item in [
        c::SettingItem {
            setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 4 },
            edit: c::SettingEdit::Removed,
        },
        c::SettingItem {
            setting: c::Setting::Application { kind: 7, scope: f::PROJECT, number: 1 },
            edit: c::SettingEdit::Set(c::SettingSet {
                value: c::SettingValue::Application(b"whole value".as_slice().into()),
            }),
        },
    ] {
        items.push(item).unwrap();
    }
    if let Some(policy) = policy {
        items
            .push(c::SettingItem {
                setting: c::Setting::ProjectPolicy { project: f::PROJECT },
                edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Policy(policy) }),
            })
            .unwrap();
    }
    items
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
    for address in [Address::Ask { project: f::PROJECT }, Address::Settings { project: f::PROJECT }] {
        for input in [
            Event::Went { place: Place::Address(Address::Home { project: f::PROJECT }) },
            Event::Went { place: Place::Address(Address::Ask { project: OTHER }) },
            Event::Went { place: Place::Address(Address::Settings { project: OTHER }) },
            Event::Went { place: Place::Missing },
        ] {
            let mut f = fixture();
            go(&mut f.root, address.clone());
            assert_eq!(f.root.pages().stock_ways().unwrap().state(), &State::Loading);
            assert_eq!(f.root.pages().stock_ways().unwrap().shown(), None);
            let out = event(&mut f.root, input);
            assert!(closed(&out, f.watch));
        }
    }
    let mut f = fixture();
    show(&mut f, &panel(f::PROJECT, Some(0), true));
    for address in [
        Address::Settings { project: f::PROJECT },
        Address::Ask { project: f::PROJECT },
        Address::Settings { project: f::PROJECT },
    ] {
        let page = f.root.pages().stock_ways().unwrap();
        let before = (page.shown(), page.open());
        let out = go(&mut f.root, address);
        assert!(!closed(&out, f.watch));
        let page = f.root.pages().stock_ways().unwrap();
        assert_eq!((page.shown(), page.open()), before);
        act(&mut f, pages::Act::WayTab { project: f::PROJECT, tab: 1 });
        act(&mut f, pages::Act::WayStop { project: f::PROJECT, stop: 0 });
        assert_eq!(f.root.pages().stock_ways().unwrap().shown(), Some(1));
        assert_eq!(f.root.pages().stock_ways().unwrap().open(), Some(0));
        act(&mut f, pages::Act::WayTab { project: OTHER, tab: 0 });
        act(&mut f, pages::Act::WayStop { project: OTHER, stop: 1 });
        assert_eq!(f.root.pages().stock_ways().unwrap().shown(), Some(1));
        assert_eq!(f.root.pages().stock_ways().unwrap().open(), Some(0));
    }
    go(&mut f.root, Address::Home { project: f::PROJECT });
    up(&mut f.root, Up::WatchEnded { watch: f.watch, end: c::WatchEnd::Closed });
    let out = go(&mut f.root, Address::Settings { project: f::PROJECT });
    let watch = opened(&out, &stock_ways::watch(f::PROJECT, &f::limits().forge).unwrap()).expect("Settings ways watch");
    assert_ne!(watch, f.watch);
    up(&mut f.root, Up::Watched { watch, event: c::WatchEvent::Opened });
    let out = go(&mut f.root, Address::Home { project: f::PROJECT });
    assert!(closed(&out, watch));
}

#[test]
fn tabs_preserve_the_reads_order_and_append_yours() {
    for (current, expected, shown, words) in [
        (Some(1), [false, true].as_slice(), 1, None),
        (None, [false, false, true].as_slice(), 2, Some((b"Our headline".as_slice(), b"Our lede".as_slice()))),
    ] {
        let mut f = fixture();
        assert_eq!(f.root.pages().stock_ways().unwrap().words(), None);
        let mut parts = panel(f::PROJECT, current, true).into_parts();
        parts.words = match words {
            Some((headline, lede)) => Some(
                w::ProjectWords::new(
                    &w::CEILINGS,
                    w::ProjectWordsParts { headline: headline.into(), lede: lede.into() },
                )
                .unwrap(),
            ),
            None => None,
        };
        let panel = w::WaysPanel::new(&w::CEILINGS, parts).unwrap();
        show(&mut f, &panel);
        assert_eq!(f.root.pages().stock_ways().unwrap().words(), panel.words().as_ref());
        assert_eq!(f.root.pages().stock_ways().unwrap().shown(), Some(shown));
        assert_eq!(f.root.pages().stock_ways().unwrap().open(), None);
        let tabs = f.root.pages().stock_ways().unwrap().tabs();
        let mut current_tabs = List::with_capacity(TABS);
        for tab in &tabs {
            current_tabs.push(tab.current).unwrap();
        }
        assert_eq!(current_tabs.as_slice(), expected);
        for (index, WayWords { name, headline, line, branch }) in WORDS.iter().enumerate() {
            let tab = tabs.get(u32::try_from(index).unwrap()).unwrap();
            assert_eq!(tab.index, Some(u32::try_from(index).unwrap()));
            assert_eq!((&*tab.name, &*tab.headline, &*tab.line, &tab.path), (*name, *headline, *line, &path(branch)));
        }
        if current.is_none() {
            let yours = tabs.get(TABS.checked_sub(1).unwrap()).unwrap();
            assert_eq!(
                (&*yours.name, &*yours.headline, &*yours.line, &yours.path, yours.index),
                (b"Yours".as_slice(), b"".as_slice(), b"".as_slice(), &path(b"own"), None)
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
        ("act out", Some(0), false, true, true, [Some(WayAct::Keep), Some(WayAct::Use), None]),
    ] {
        let mut f = fixture();
        document(&mut f, NEWEST, configured);
        show(&mut f, &panel(f::PROJECT, current, may_change));
        if pending {
            use_way(&mut f);
        }
        let page = f.root.pages().stock_ways().unwrap();
        assert_eq!([page.act(0), page.act(1), page.act(2)], expected, "{name}");
        assert_eq!(page.sending(), pending, "{name}");
    }
}

#[test]
fn configured_ways_follow_the_pages_own_project() {
    let mut f = fixture();
    document(&mut f, NEWEST, true);
    for (project, settings) in [(f::PROJECT, false), (OTHER, true), (OTHER, false), (f::PROJECT, true)] {
        let address = if settings { Address::Settings { project } } else { Address::Ask { project } };
        let out = go(&mut f.root, address);
        if let Some(watch) = opened(&out, &stock_ways::watch(project, &f::limits().forge).unwrap()) {
            f.watch = watch;
            up(&mut f.root, Up::Watched { watch, event: c::WatchEvent::Opened });
        }
        show(&mut f, &panel(project, Some(0), true));
        assert_eq!(
            f.root.pages().stock_ways().unwrap().act(0),
            if project == f::PROJECT { None } else { Some(WayAct::Keep) },
            "project={project}"
        );
    }
}

#[test]
fn use_and_keep_send_one_ordered_decision_at_newest() {
    for (current, action, kept) in [
        (Some(0), pages::Act::UseWay { project: f::PROJECT, way: 1 }, false),
        (Some(1), pages::Act::KeepWay { project: f::PROJECT }, true),
    ] {
        let mut f = fixture();
        show(&mut f, &panel(f::PROJECT, current, true));
        document(&mut f, NEXT, false);
        let out = act(&mut f, action);
        assert!(reads(&out).is_empty(), "a way without delegation reads nothing");
        let sent = sends(out);
        assert_eq!(sent.len(), DECISIONS);
        let change = &sent.first().unwrap().1;
        assert_eq!((change.against, &change.running), (NEXT, &c::Running::Stays));
        assert_eq!(change.items, way_items(None));
        assert!(f.root.pages().stock_ways().unwrap().sending());
        let retried = sends(answer(&mut f, sent.first().unwrap().0, refused(stale())));
        let key = retried.first().unwrap().0;
        assert!(f.root.pages().stock_ways().unwrap().sending());
        let mut refreshed = panel(f::PROJECT, None, true).into_parts();
        let mut renamed = refreshed.ways.get(1).unwrap().clone().into_parts();
        renamed.name = b"A newer name".as_slice().into();
        *refreshed.ways.get_mut(1).unwrap() = w::StockWay::new(&w::CEILINGS, renamed).unwrap();
        show(&mut f, &w::WaysPanel::new(&w::CEILINGS, refreshed).unwrap());
        answer(&mut f, key, c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: NEXT })));
        assert!(!f.root.pages().stock_ways().unwrap().sending());
        assert_eq!(
            f.root.pages().needs().toast(),
            Some(&pages::needs::Toasted::Way { name: b"Agents review".as_slice().into(), revision: NEXT, kept })
        );
        act(&mut f, pages::Act::Needs(pages::needs::Act::Dismiss));
        answer(&mut f, key, c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: NEXT })));
        assert_eq!(f.root.pages().needs().toast(), None, "an old answer is ignored");
        show(&mut f, &panel(f::PROJECT, None, true));
        assert_eq!(f.root.pages().needs().toast(), None, "a toast is handed over once");
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
fn delegation_reads_policy_then_sends_only_missing_executors() {
    for (current, action) in [
        (Some(0), pages::Act::UseWay { project: f::PROJECT, way: 1 }),
        (Some(1), pages::Act::KeepWay { project: f::PROJECT }),
    ] {
        for (ceiling, owner, expected) in [
            (OLD, OLD, Some((COMPLETE, COMPLETE))),
            (PARTIAL, PARTIAL, Some((COMPLETED_PARTIAL, COMPLETED_PARTIAL))),
            (COMPLETE, OLD, Some((COMPLETE, COMPLETE))),
            (OLD, COMPLETE, Some((COMPLETE, COMPLETE))),
            (COMPLETE, COMPLETE, None),
        ] {
            let mut f = fixture();
            show(&mut f, &panel_delegation(f::PROJECT, current, true, true));
            document(&mut f, NEXT, false);
            let out = act(&mut f, action.clone());
            let reads = reads(&out);
            assert_eq!(reads.len(), DECISIONS);
            let (handle, read) = reads.first().unwrap();
            assert_eq!(
                read,
                &c::Read::Setting(c::ReadSetting {
                    setting: c::Setting::ProjectPolicy { project: f::PROJECT },
                    revision: NEXT,
                    proposal: None,
                })
            );
            assert!(sends(out).is_empty(), "the policy must arrive before sending");
            assert!(f.root.pages().stock_ways().unwrap().sending());
            assert_eq!(
                f.root.pages().stock_ways().unwrap().act(1),
                Some(if current == Some(1) { WayAct::Keep } else { WayAct::Use })
            );
            let sent = sends(read_answer(&mut f, *handle, policy_page(policy(ceiling, owner))));
            let expected_policy = match expected {
                Some((ceiling, owner)) => Some(policy(ceiling, owner)),
                None => None,
            };
            assert_eq!(sent.len(), DECISIONS);
            assert_eq!(
                sent.first().unwrap().1,
                c::ChangeSettings { against: NEXT, items: way_items(expected_policy), running: c::Running::Stays }
            );
        }
    }
}

#[test]
fn policy_read_failure_keeps_refusal_and_sends_nothing() {
    const REFUSED: &[u8] = b"The project policy read was refused.";
    const FAILED: &[u8] = b"The project policy could not be read whole.";
    for (page, why) in [
        (Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), REFUSED),
        (Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)), FAILED),
        (Readout::Jig(c::ReadResult::None(c::ReadNone::TooLarge { bytes: 4096 })), FAILED),
        (Readout::Limit, FAILED),
        (Readout::NotServed { family: None, served: None }, FAILED),
        (policy_document(OTHER, NEXT, Some(c::SettingValue::Policy(policy(OLD, OLD)))), FAILED),
        (policy_document(f::PROJECT, NEWEST, Some(c::SettingValue::Policy(policy(OLD, OLD)))), FAILED),
        (policy_document(f::PROJECT, NEXT, None), FAILED),
        (policy_document(f::PROJECT, NEXT, Some(c::SettingValue::Application(Box::default()))), FAILED),
    ] {
        let mut f = fixture();
        show(&mut f, &panel_delegation(f::PROJECT, Some(0), true, true));
        document(&mut f, NEXT, false);
        let out = act(&mut f, pages::Act::UseWay { project: f::PROJECT, way: 1 });
        let handle = reads(&out).first().unwrap().0;
        assert!(sends(out).is_empty());
        assert!(read_answer(&mut f, handle, page).is_empty());
        let refusal = c::Refusal::Judged {
            at: 0,
            setting: c::Setting::ProjectPolicy { project: f::PROJECT },
            why: c::Text::new(why.into(), 128).unwrap(),
        };
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), Some(&refusal));
        assert!(!f.root.pages().stock_ways().unwrap().sending());
        assert_eq!(f.root.pages().needs().toast(), None);
        document(&mut f, NEXT, false);
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), Some(&refusal));
        assert_eq!(f.root.pages().stock_ways().unwrap().act(1), Some(WayAct::Use));
        act(&mut f, pages::Act::UseWay { project: f::PROJECT, way: 1 });
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
        read_answer(&mut f, handle, Readout::Limit);
        show(&mut f, &panel_delegation(f::PROJECT, Some(0), true, true));
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
    }
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
        assert!(!f.root.pages().stock_ways().unwrap().sending());
        assert_eq!(f.root.pages().needs().toast(), None);
        document(&mut f, NEXT, false);
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), Some(&refusal));
        let (key, _) = use_way(&mut f);
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
        answer(&mut f, key, refused(c::Refusal::Provider));
        show(&mut f, &panel(f::PROJECT, Some(1), true));
        assert_eq!(f.root.pages().stock_ways().unwrap().refused(), None);
    }
    for answer_value in
        [c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Stopped { task: 9 })), refused(c::Refusal::Provider)]
    {
        let mut f = fixture();
        show(&mut f, &panel(f::PROJECT, Some(0), true));
        let (key, _) = use_way(&mut f);
        answer(&mut f, key, answer_value);
        assert!(!f.root.pages().stock_ways().unwrap().sending());
        assert_eq!(f.root.pages().needs().toast(), None);
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
        assert_eq!(page.words(), None);
        assert_eq!((page.shown(), page.open()), (None, None));
        assert_eq!(page.act(0), None);
    }
    let mut f = fixture();
    up(&mut f.root, Up::WatchEnded { watch: f.watch, end: c::WatchEnd::SubjectEnded });
    assert_eq!(f.root.pages().stock_ways().unwrap().state(), &State::Ended);
}

#[test]
fn tabs_and_stop_cards_switch_in_place() {
    use pages::Act::{WayStop, WayTab};
    for (name, actions, shown, open) in [
        ("switch tab", [WayTab { project: f::PROJECT, tab: 1 }].as_slice(), 1, None),
        ("same tab closes", [WayTab { project: f::PROJECT, tab: 0 }].as_slice(), 0, None),
        ("past tabs", [WayTab { project: f::PROJECT, tab: WAYS }].as_slice(), 0, Some(0)),
        ("other project's tab", [WayTab { project: OTHER, tab: 1 }].as_slice(), 0, Some(0)),
        ("close stop", [WayStop { project: f::PROJECT, stop: 0 }].as_slice(), 0, None),
        ("move stop", [WayStop { project: f::PROJECT, stop: 1 }].as_slice(), 0, Some(1)),
        ("bar", [WayStop { project: f::PROJECT, stop: 2 }].as_slice(), 0, Some(0)),
        ("past path", [WayStop { project: f::PROJECT, stop: 3 }].as_slice(), 0, Some(0)),
        ("other project's stop", [WayStop { project: OTHER, stop: 1 }].as_slice(), 0, Some(0)),
    ] {
        let mut f = fixture();
        assert!(act(&mut f, WayStop { project: f::PROJECT, stop: 0 }).is_empty());
        assert!(act(&mut f, WayTab { project: f::PROJECT, tab: 0 }).is_empty());
        show(&mut f, &panel(f::PROJECT, Some(0), true));
        assert!(act(&mut f, WayStop { project: f::PROJECT, stop: 0 }).is_empty());
        assert_eq!(f.root.pages().stock_ways().unwrap().open(), Some(0), "opens: {name}");
        for action in actions {
            assert!(act(&mut f, action.clone()).is_empty(), "{name}");
        }
        let page = f.root.pages().stock_ways().unwrap();
        assert_eq!((page.shown(), page.open()), (Some(shown), open), "{name}");
        assert_eq!(f.root.pages().address(), &Address::Ask { project: f::PROJECT });
    }
}

#[derive(Clone, Copy, Debug)]
enum Refresh {
    Same,
    MoreWays,
    WayKept,
    YoursGone,
    WayGone,
    Words,
    Actor,
    Card,
    Branch,
    FewerStops,
    OtherPath,
}
#[test]
fn later_documents_keep_the_tab_key_and_only_the_same_stops_card() {
    for (change, shown, open) in [
        (Refresh::Same, 2, Some(0)),
        (Refresh::MoreWays, 3, Some(0)),
        (Refresh::WayKept, 1, Some(0)),
        (Refresh::YoursGone, 1, None),
        (Refresh::WayGone, 0, None),
        (Refresh::Words, 2, None),
        (Refresh::Actor, 2, None),
        (Refresh::Card, 2, Some(0)),
        (Refresh::Branch, 2, Some(0)),
        (Refresh::FewerStops, 2, None),
        (Refresh::OtherPath, 2, Some(0)),
    ] {
        let mut f = fixture();
        let first = panel(f::PROJECT, None, true);
        show(&mut f, &first);
        let tab = if let Refresh::WayGone | Refresh::WayKept = change { 1 } else { 2 };
        act(&mut f, pages::Act::WayTab { project: f::PROJECT, tab });
        act(&mut f, pages::Act::WayStop { project: f::PROJECT, stop: 0 });
        let mut next = first.into_parts();
        let mut path = next.own.into_parts();
        match change {
            Refresh::Same => {}
            Refresh::MoreWays => {
                let mut ways = List::with_capacity(TABS);
                for way in &next.ways {
                    ways.push(way.clone()).unwrap();
                }
                ways.push(next.ways.get(0).unwrap().clone()).unwrap();
                next.ways = ways;
            }
            Refresh::WayKept => next.current = Some(0),
            Refresh::YoursGone => next.current = Some(1),
            Refresh::WayGone => {
                next.ways = List::from_boxed(Box::new([next.ways.get(0).unwrap().clone()]));
                next.current = Some(0);
            }
            Refresh::Words | Refresh::Actor | Refresh::Card => {
                let mut stop = path.stops.get(1).unwrap().clone().into_parts();
                match change {
                    Refresh::Words => stop.words = b"changed words".as_slice().into(),
                    Refresh::Actor => stop.actor = w::Actor::Code,
                    Refresh::Card => {
                        let mut card = stop.card.into_parts();
                        card.who = b"New details".as_slice().into();
                        stop.card = w::Card::new(&w::CEILINGS, card).unwrap();
                    }
                    Refresh::Same
                    | Refresh::MoreWays
                    | Refresh::WayKept
                    | Refresh::YoursGone
                    | Refresh::WayGone
                    | Refresh::Branch
                    | Refresh::FewerStops
                    | Refresh::OtherPath => unreachable!(),
                }
                *path.stops.get_mut(1).unwrap() = w::Stop::new(&w::CEILINGS, stop).unwrap();
            }
            Refresh::Branch => path.branch = b"new branch".as_slice().into(),
            Refresh::FewerStops => path.stops = List::from_boxed(Box::new([path.stops.get(0).unwrap().clone()])),
            Refresh::OtherPath => {
                let mut way = next.ways.get(0).unwrap().clone().into_parts();
                way.path = w::BranchPath::new(
                    &w::CEILINGS,
                    w::BranchPathParts {
                        repository: None,
                        branch: b"elsewhere".as_slice().into(),
                        stops: List::with_capacity(0),
                    },
                )
                .unwrap();
                *next.ways.get_mut(0).unwrap() = w::StockWay::new(&w::CEILINGS, way).unwrap();
            }
        }
        next.own = w::BranchPath::new(&w::CEILINGS, path).unwrap();
        show(&mut f, &w::WaysPanel::new(&w::CEILINGS, next).unwrap());
        let page = f.root.pages().stock_ways().unwrap();
        assert_eq!((page.shown(), page.open()), (Some(shown), open), "{change:?}");
    }
}

fn role_me() -> c::Me {
    let mut me = f::chats::me();
    let project = me.projects.get_mut(0).unwrap();
    let template = project.templates.get_mut(0).unwrap();
    template.number = 7;
    template.executor = c::TemplateExecutor::Chosen;
    template.budget = c::TemplateBudget::Given;
    template.flags = c::TaskFlags { ended_by: c::EndedBy::Standing, conversation: false, listed: true, retention: c::Retention::Kept };
    template.label = 3;
    template.grace = None;
    let within = &mut project.creating.as_mut().unwrap().within;
    within.charters = List::from_boxed(Box::new([14]));
    within.budget = 50_000;
    within.maximum_budget = 50_000;
    within.pool_left = 50_000;
    within.authority.authority.spend = 50_000;
    me
}

#[test]
fn draft_first_starts_only_missing_roles_after_settings_and_keeps_settings_on_refusal() {
    const WORDS: &[u8] = b"Routes what comes in to its area's owner, and triages what fits no area.";
    for running in [false, true] {
        let identity = role_me();
        let mut f = fixture_me(identity.clone());
        let mut parts = panel(f::PROJECT, Some(0), true).into_parts();
        let mut way = parts.ways.get(1).unwrap().clone().into_parts();
        way.name = b"Draft first".as_slice().into();
        let role = w::WayRole::new(&w::CEILINGS, w::WayRoleParts { charter: 14, words: WORDS.into(), budget: 25_000, running }).unwrap();
        way.roles = List::from_boxed(Box::new([role]));
        let triager = br#"{"owner":{"charter":14}}"#;
        let mut items = List::with_capacity(3);
        for item in &way.items { items.push(item.clone()).unwrap(); }
        items.push(w::ProposalItem::new(&w::CEILINGS, w::ProposalItemParts {
            kind: 13, scope: f::PROJECT, number: 1,
            edit: w::SettingEdit::Set(w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: triager.as_slice().into() }).unwrap()),
        }).unwrap()).unwrap();
        way.items = items;
        *parts.ways.get_mut(1).unwrap() = w::StockWay::new(&w::CEILINGS, way).unwrap();
        show(&mut f, &w::WaysPanel::new(&w::CEILINGS, parts).unwrap());
        let (key, settings) = use_way(&mut f);
        let mut expected_items = way_items(None);
        expected_items.push(c::SettingItem {
            setting: c::Setting::Application { kind: 13, scope: f::PROJECT, number: 1 },
            edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(triager.as_slice().into()) }),
        }).unwrap();
        assert_eq!(settings.items, expected_items, "settings precede any role request");
        let out = answer(&mut f, key, c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: NEXT })));
        let mut sent = List::uncapped();
        for request in &out {
            if let pages::Request::Client(jig_client::Event::Send { key, request: jig_client::Ask::Jig(ask), .. }) = request {
                sent.push((*key, ask.clone())).unwrap();
            }
        }
        assert_eq!(sent.len(), u32::from(!running));
        assert_eq!(f.root.pages().needs().toast(), Some(&pages::needs::Toasted::Way { name: b"Draft first".as_slice().into(), revision: NEXT, kept: false }));
        if !running {
            let draft = jig_client::shapes::GoalDraft { words: c::Text::new(WORDS.into(), 480).unwrap(), executor: c::Executor::Agent { charter: 14 }, budget: 25_000, deadline: None, priority: None, llm: None };
            let jig_client::shapes::Judged::Create(expected) = jig_client::shapes::set_goal_from(&identity, f::PROJECT, 7, draft, c::Text::new(Box::new([]), 0).unwrap()) else { panic!("offered role") };
            let (key, ask) = sent.get(0).unwrap();
            assert_eq!(ask.as_ref(), &expected);
            let c::Ask::Create { tasks, .. } = ask.as_ref() else { panic!("starts role") };
            let task = tasks.get(0).unwrap();
            assert_eq!((task.executor, task.label, task.authority.spend), (c::Executor::Agent { charter: 14 }, 3, 25_000));
            assert_eq!(task.flags.ended_by, c::EndedBy::Standing);
            assert_eq!(task.spec.words.bytes(), WORDS);
            let refusal = c::Refusal::Judged { at: 0, setting: c::Setting::Charter(c::SettingNumber { number: 14 }), why: c::Text::new(b"The triager cannot start here.".as_slice().into(), 128).unwrap() };
            let out = answer(&mut f, *key, refused(refusal.clone()));
            assert!(out.is_empty(), "role refusal sends no rollback");
            assert_eq!(f.root.pages().stock_ways().unwrap().refused(), Some(&refusal), "the refusal's words reach the page");
        }
        assert!(!f.root.pages().stock_ways().unwrap().sending());
    }
}
