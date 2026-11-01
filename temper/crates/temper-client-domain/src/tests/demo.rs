use super::fixtures as f;
use crate::{
    forge::{self, demo},
    pages::Act,
};
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};
const PROVIDER: u32 = 17;
const CUE: &[u8] = b"Change the title";
const DRAFT: &[u8] = b"My draft";
pub(super) fn answer(cue: &[u8]) -> Box<[u8]> {
    let mut cues = List::with_capacity(1);
    cues.push(cue.into()).unwrap();
    wire::playground_answer(
        &w::PlaygroundAnswer::new(
            &w::CEILINGS,
            w::PlaygroundAnswerParts {
                state: w::PlaygroundState::Running,
                project: f::PROJECT,
                forge: wire::FORGE,
                provider: PROVIDER,
                owner: b"acme".as_slice().into(),
                name: b"shop".as_slice().into(),
                cues,
            },
        )
        .unwrap(),
    )
}
fn page() -> Readout {
    Readout::Family { family: wire::FORGE, record: Some(answer(CUE)) }
}
pub(super) fn page_with_state(state: w::PlaygroundState) -> Readout {
    let mut parts = wire::decode_playground_answer(&answer(CUE), &w::CEILINGS).unwrap().into_parts();
    parts.state = state;
    Readout::Family {
        family: wire::FORGE,
        record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
    }
}
fn read(side: &mut forge::Side) -> Handle {
    let mut out = Queue::with_capacity(1);
    assert!(!demo::told(side, &Told::Link { link: Link::Live }, f::limits().forge, &mut out));
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, read } }) = out.pop() else {
        panic!("demo read");
    };
    let Readable::Family { scope, window, record } = read else { panic!("family read") };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::Playground);
    assert_ne!(handle.number, 0, "zero belongs to me");
    assert!(out.is_empty());
    handle
}
#[test]
fn the_demo_is_read_once_per_live_link_and_replaced_pages_are_dropped() {
    for (row, gap, reads) in [
        ("live again", Link::Live, false),
        ("offline then live", Link::Offline, true),
        ("signed out then live", Link::SignedOut, true),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let first = read(&mut side);
        let mut out = Queue::with_capacity(1);
        assert!(!demo::told(&mut side, &Told::Link { link: gap }, f::limits().forge, &mut out), "{row}");
        assert!(out.is_empty(), "{row}");
        if reads {
            let next = read(&mut side);
            assert_ne!(first, next, "{row}");
            assert!(demo::told(&mut side, &Told::Page { handle: first, page: page() }, f::limits().forge, &mut out));
            assert!(side.demo().is_none(), "{row}: replaced page dropped");
            demo::told(&mut side, &Told::Page { handle: next, page: page() }, f::limits().forge, &mut out);
            assert!(side.demo().is_some(), "{row}");
        } else {
            assert!(!demo::told(&mut side, &Told::Link { link: Link::Live }, f::limits().forge, &mut out));
            assert!(out.is_empty(), "{row}");
        }
    }
}
#[test]
fn a_demo_page_replaces_the_last_answer_or_clears_it() {
    let mut another_version = answer(CUE);
    *another_version.get_mut(1).unwrap() = u8::try_from(wire::CLIENT_VERSION + 1).unwrap();
    for (row, page, present) in [
        ("demo", page(), true),
        ("none", Readout::Family { family: wire::FORGE, record: None }, false),
        ("not served", Readout::NotServed { family: Some(wire::FORGE), served: None }, false),
        ("limit", Readout::Limit, false),
        ("undecodable", Readout::Family { family: wire::FORGE, record: Some(Box::new([255])) }, false),
        ("another version", Readout::Family { family: wire::FORGE, record: Some(another_version) }, false),
        ("empty cue", Readout::Family { family: wire::FORGE, record: Some(answer(b"")) }, false),
        ("wrong family", Readout::Family { family: 99, record: Some(answer(CUE)) }, false),
        ("jig page", Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)), false),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let first = read(&mut side);
        assert!(!side.demo_answered(), "{row}: no answer yet");
        demo::told(
            &mut side,
            &Told::Page { handle: first, page: self::page() },
            f::limits().forge,
            &mut Queue::with_capacity(1),
        );
        let shown = forge::shown(&side);
        demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo().is_some(), "{row}: keep the last answer across the gap");
        assert!(side.demo_answered(), "{row}: keep that the read answered across the gap");
        let handle = read(&mut side);
        assert!(
            demo::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1)),
            "{row}"
        );
        assert_eq!(side.demo().is_some(), present, "{row}");
        assert_eq!(forge::shown(&side) != shown, !present, "{row}: redraw only on change");
    }
}
pub(super) fn accounts(provider: Option<u32>, usable: bool, cut: bool) -> c::OwnerAccounts {
    let mut sources = List::with_capacity(1);
    sources
        .push(c::OwnerAccountSource {
            name: None,
            number: 1,
            place: 1,
            kind: c::OwnerAccountKind::Key,
            state: c::OwnerAccountState::Granted,
            usable,
            expiring: false,
            attention: false,
            spent_until: None,
        })
        .unwrap();
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::OwnerAccountLine {
            number: 1,
            place: 1,
            uses: None,
            plan: false,
            provider,
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(f::acts::text(b"Model")),
            holder: c::OwnerAccountHolder::Party(f::PARTY),
            sources,
            cut,
        })
        .unwrap();
    c::OwnerAccounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts,
        signing: List::with_capacity(0),
        last_end: None,
        usable,
        settled: true,
        cut: false,
    }
}

fn event(root: &mut crate::Domain, event: crate::Event) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn act(root: &mut crate::Domain, act: Act) -> Queue<crate::Request> {
    event(root, crate::Event::Act { action: crate::Action::Pages(act) })
}
fn opened(cue: &[u8]) -> (crate::Domain, skein_lib::Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, account) = f::start_accounts(&mut root, List::with_capacity(0));
    for (watch, delivery) in [
        (me, c::Delivery::Me(Box::new(f::chats::me()))),
        (account, c::Delivery::OwnerAccounts(Box::new(accounts(Some(PROVIDER), true, false)))),
    ] {
        event(
            &mut root,
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
                },
            },
        );
    }
    deliver_demo_read(&mut root, Readout::Family { family: wire::FORGE, record: Some(answer(cue)) });
    event(
        &mut root,
        crate::Event::Went {
            place: crate::Place::Address(crate::Address::Chats { filter: crate::ChatsFilter::All, start: false }),
        },
    );
    act(&mut root, Act::ChatWords { words: DRAFT.into() });
    (root, account)
}
#[test]
fn a_suggestion_starts_one_chat_with_its_cue_and_preserves_the_draft() {
    let mut at_bound = skein_lib::bytes::zeroed(usize::try_from(wire::CUE_BYTES).unwrap());
    at_bound.fill(b'a');
    let above_receiving_fixture = [b'a'; 33];
    for (row, words, pending, project, model, cue, count) in [
        ("demo project", CUE, false, f::PROJECT, true, 0, 1),
        ("at the composer's bound", &at_bound, false, f::PROJECT, true, 0, 1),
        ("above the receiving fixture", &above_receiving_fixture, false, f::PROJECT, true, 0, 1),
        ("pending", CUE, true, f::PROJECT, true, 0, 0),
        ("other project", CUE, false, f::PROJECT + 1, true, 0, 0),
        ("real account", CUE, false, f::PROJECT, false, 0, 1),
        ("past cues", CUE, false, f::PROJECT, true, 1, 0),
    ] {
        let (mut root, account) = opened(words);
        if pending {
            act(&mut root, Act::StartChat);
        }
        if project != f::PROJECT {
            let limits = forge::wire_limits(&f::limits().forge).unwrap();
            let mut parts = wire::decode_playground_answer(&answer(CUE), &limits).unwrap().into_parts();
            parts.project = project;
            let record = wire::playground_answer(&w::PlaygroundAnswer::new(&limits, parts).unwrap());
            deliver_demo_read(&mut root, Readout::Family { family: wire::FORGE, record: Some(record) });
        }
        if !model {
            event(
                &mut root,
                crate::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch: account,
                            event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(accounts(
                                Some(PROVIDER + 1),
                                true,
                                false,
                            )))),
                        },
                    },
                },
            );
        }
        let mut out = act(&mut root, Act::Suggest { cue });
        let mut sent = List::with_capacity(1);
        while let Some(request) = out.pop() {
            if let crate::Request::Client {
                request:
                    jig_client::Request::Down { down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. } },
            } = request
            {
                sent.push(*ask).expect("one chat start");
            }
        }
        assert_eq!(usize::try_from(sent.len()).unwrap(), count, "{row}");
        if count == 1 {
            assert_eq!(
                sent.as_slice(),
                [jig_client::shapes::chat(
                    &f::chats::me(),
                    f::PROJECT,
                    c::Text::new(words.into(), wire::CUE_BYTES).unwrap()
                )
                .unwrap()],
                "{row}"
            );
        }
        assert_eq!(root.pages().chats().unwrap().words.as_ref(), DRAFT, "{row}");
    }
}

#[test]
fn the_first_demo_read_answer_counts_even_when_it_has_no_record() {
    for page in [
        Readout::Family { family: wire::FORGE, record: None },
        Readout::NotServed { family: Some(wire::FORGE), served: None },
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let handle = read(&mut side);
        let before = forge::shown(&side);
        demo::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo_answered());
        assert!(side.demo().is_none());
        assert_eq!(side.demo_provider(), None);
        assert!(forge::shown(&side) > before, "an answered absence redraws the welcome");
        demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo_answered());
    }
}

fn demo_side(state: w::PlaygroundState) -> forge::Side {
    let mut side = forge::Side::new(&f::limits().forge, f::SEED);
    let mut parts = wire::decode_playground_answer(&answer(CUE), &w::CEILINGS).unwrap().into_parts();
    parts.state = state;
    let handle = read(&mut side);
    demo::told(
        &mut side,
        &Told::Page {
            handle,
            page: Readout::Family {
                family: wire::FORGE,
                record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
            },
        },
        f::limits().forge,
        &mut Queue::with_capacity(4),
    );
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    event(
        &mut root,
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(f::chats::me()))),
                },
            },
        },
    );
    forge::me::told(
        &mut side,
        root.client(),
        &Told::Changed { handle: forge::me::handle() },
        &mut Queue::with_capacity(4),
    );
    side
}
fn started(side: &mut forge::Side) -> skein_lib::Token {
    let mut out = Queue::with_capacity(4);
    demo::start(side, f::limits().forge, &mut out);
    let Some(forge::Request::Send { request, ask: jig_client::Ask::Family { scope, record }, .. }) = out.pop() else {
        panic!("start sent")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request(), &w::Request::StartPlayground);
    assert!(out.is_empty());
    request
}
fn answered(side: &mut forge::Side, request: skein_lib::Token, outcome: w::Outcome) {
    let value = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap();
    let mut out = Queue::with_capacity(4);
    demo::told(
        side,
        &Told::Answered {
            key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() },
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&value),
            })),
        },
        f::limits().forge,
        &mut out,
    );
    let Some(forge::Request::Client { asked: jig_client::Event::Read { .. } }) = out.pop() else {
        panic!("start answer rereads the demo")
    };
    assert!(out.is_empty());
}
#[test]
fn demo_state_decodes_and_start_is_single_across_pages_with_a_new_retry_key() {
    let mut side = demo_side(w::PlaygroundState::NotStarted);
    assert_eq!(side.demo().unwrap().state, demo::DemoState::NotStarted);
    assert!(!side.demo_running());
    let first = started(&mut side);
    assert!(side.demo_running());
    assert!(side.demo_starting());
    let mut out = Queue::with_capacity(4);
    forge::step(
        &mut side,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::limits().forge },
        forge::Event::Left,
        &mut out,
    );
    demo::start(&mut side, f::limits().forge, &mut out);
    assert!(out.is_empty());
    answered(
        &mut side,
        first,
        w::Outcome::StartFailed(
            w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
        ),
    );
    assert_eq!(side.demo_failure(), Some(&demo::StartFailure::Account));
    assert!(side.demo_running());
    assert!(!side.demo_starting());
    let retry = started(&mut side);
    assert_ne!(first, retry);
    assert!(side.demo_failure().is_none());
}
#[test]
fn a_running_read_sends_no_start_and_a_persons_start_shows_its_failure() {
    let mut side = demo_side(w::PlaygroundState::Running);
    let mut out = Queue::with_capacity(4);
    demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut out);
    let handle = read(&mut side);
    demo::told(&mut side, &Told::Page { handle, page: page() }, f::limits().forge, &mut out);
    assert!(out.is_empty(), "running read sends no start");
    demo::start(&mut side, f::limits().forge, &mut out);
    assert!(out.is_empty(), "a person's start while running sends nothing");
    assert!(!side.demo_starting());

    let mut side = demo_side(w::PlaygroundState::NotStarted);
    let request = started(&mut side);
    assert!(side.demo_starting());
    answered(
        &mut side,
        request,
        w::Outcome::StartFailed(
            w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::NotConnected }).unwrap(),
        ),
    );
    assert!(!side.demo_starting());
    assert_eq!(side.demo_failure(), Some(&demo::StartFailure::NotConnected));
}
#[test]
fn running_unknown_and_not_offered_each_read_the_demo_again() {
    for outcome in [
        w::Outcome::Running,
        w::Outcome::Unknown,
        w::Outcome::StartRefused(
            w::StartRefused::new(&w::CEILINGS, w::StartRefusedParts { why: w::StartRefusal::NotOffered }).unwrap(),
        ),
    ] {
        let mut side = demo_side(w::PlaygroundState::NotStarted);
        let request = started(&mut side);
        answered(&mut side, request, outcome);
        assert!(!side.demo_starting());
        assert!(side.demo_failure().is_none());
    }
}
#[test]
fn another_operations_outcome_is_not_a_start_answer() {
    let value = jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::SettingUp }).unwrap()),
    });
    assert!(demo::answer(&value, &f::limits().forge).is_none());
}

fn up(root: &mut crate::Domain, up: jig_client::Up) -> Queue<crate::Request> {
    event(root, crate::Event::Client { event: jig_client::Event::Up { up } })
}
fn deliver_accounts(
    root: &mut crate::Domain,
    watch: skein_lib::Token,
    document: c::OwnerAccounts,
) -> Queue<crate::Request> {
    up(
        root,
        jig_client::Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(document))),
        },
    )
}
fn ending_accounts(other_usable: bool) -> c::OwnerAccounts {
    accounts(Some(PROVIDER + 1), other_usable, false)
}
pub(super) fn deployment_watch(out: &mut Queue<crate::Request>) -> Option<skein_lib::Token> {
    let mut watch = None;
    let mut rest = Queue::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(c::Subject::Deployment) },
                },
        } = request
        {
            assert!(watch.replace(token).is_none(), "one deployment watch");
        } else {
            rest.push(request);
        }
    }
    *out = rest;
    watch
}

fn ending_root(
    role: Option<c::HeldRole>,
) -> (crate::Domain, skein_lib::Token, skein_lib::Token, Option<skein_lib::Token>) {
    ending_root_with_page(role, page())
}
fn ending_root_with_page(
    role: Option<c::HeldRole>,
    page: Readout,
) -> (crate::Domain, skein_lib::Token, skein_lib::Token, Option<skein_lib::Token>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, account) = f::start_accounts(&mut root, List::with_capacity(0));
    let mut frame = f::chats::me();
    frame.deployment_role = role;
    let mut out = up(
        &mut root,
        jig_client::Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) },
    );
    let mut forges = None;
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { .. } },
                },
        } = request
        {
            forges = Some(watch);
        }
    }
    let mut out = deliver_demo_read(&mut root, page);
    let watch = deployment_watch(&mut out);
    (root, account, forges.expect("deployment forges"), watch)
}
pub(super) fn deliver_demo_read(root: &mut crate::Domain, page: Readout) -> Queue<crate::Request> {
    demo::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
    let handle = read(&mut root.forge);
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Page { handle, page }, &mut out);
    crate::route::demo_progress(root, &f::env(&f::limits()), &mut out);
    out
}
fn forge_rows(root: &mut crate::Domain, watch: skein_lib::Token, connected: bool) -> Queue<crate::Request> {
    up(root, jig_client::Up::FamilyDelivered { watch, record: forge_document(connected) })
}
fn forge_document(connected: bool) -> jig_client::Family {
    let mut forges = List::with_capacity(1);
    let connection = if connected {
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"bot".as_slice().into() }).unwrap(),
        )
    } else {
        w::Connection::Connecting
    };
    forges
        .push(
            w::ForgeRow::new(
                &w::CEILINGS,
                w::ForgeRowParts {
                    number: wire::FORGE + 1,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Owned,
                    connection,
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations: List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: None,
                    repositories: List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
    jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges }).unwrap()),
    }
}
fn removed(out: &Queue<crate::Request>) -> List<skein_lib::Token> {
    let mut attempts = List::with_capacity(4);
    for request in out {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
        {
            assert_eq!(
                **ask,
                c::Ask::AccountChange {
                    holder: c::AccountHolder::Deployment,
                    change: c::AccountChange::Remove { account: 42 }
                }
            );
            attempts.push(*attempt).unwrap();
        }
    }
    attempts
}

fn demo_reread(out: &mut Queue<crate::Request>) -> skein_lib::Token {
    let mut read = None;
    let mut remaining = Queue::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Read { read: token, what: Readable::Family { record, .. } },
                },
        } = &request
        {
            assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::Playground);
            assert!(read.replace(*token).is_none());
        } else {
            remaining.push(request);
        }
    }
    *out = remaining;
    read.expect("Demo reread")
}

fn deliver_role(root: &mut crate::Domain, me: skein_lib::Token, role: Option<c::HeldRole>) -> Queue<crate::Request> {
    let mut frame = f::chats::me();
    frame.deployment_role = role;
    up(root, jig_client::Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) })
}

#[test]
fn a_resetting_read_is_independent_of_document_order() {
    for resetting_first in [true, false] {
        let mut root = crate::unlinked(&f::limits(), f::SEED);
        let (me, account) = f::start_accounts(&mut root, List::with_capacity(0));
        if resetting_first {
            assert!(deliver_demo_read(&mut root, page_with_state(w::PlaygroundState::Resetting)).is_empty());
        }
        let mut out = deliver_role(&mut root, me, Some(c::HeldRole::Owner));
        assert!(deployment_watch(&mut out).is_none());
        let forges = family_watch(&out);
        forge_rows(&mut root, forges, false);
        deliver_accounts(&mut root, account, accounts(Some(PROVIDER), true, false));
        if !resetting_first {
            assert!(deliver_demo_read(&mut root, page_with_state(w::PlaygroundState::Resetting)).is_empty());
        }
        assert!(root.forge().leaving());
        assert_eq!(root.forge().demo_provider(), Some(PROVIDER));
        assert_eq!(root.forge().demo().unwrap().state, demo::DemoState::Resetting);
        assert_eq!(root.demo().unwrap().state, demo::DemoState::Resetting);
        assert_eq!(root.pages().project(), Some(f::PROJECT));
        assert_eq!(crate::pages::welcome::model_usable(&root), Some(false), "stale demo account is excluded");
        for act in [Act::Playground, Act::RetryDemo] {
            assert!(event(&mut root, crate::Event::Act { action: crate::Action::Pages(act) }).is_empty());
        }
        let mut out = deliver_accounts(&mut root, account, ending_accounts(true));
        assert!(deployment_watch(&mut out).is_none());
        assert!(removed(&out).is_empty());
        assert!(forge_rows(&mut root, forges, true).is_empty(), "a real forge never triggers a playground read");
    }
}
fn family_watch(out: &Queue<crate::Request>) -> skein_lib::Token {
    for request in out {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { .. } },
                },
        } = request
        {
            return *watch;
        }
    }
    panic!("forges watch")
}
#[test]
fn resetting_and_running_reads_keep_the_project_until_the_reset_finishes() {
    const OTHER_PROJECT: u32 = 2;
    for (state, expected) in
        [(w::PlaygroundState::Resetting, Some(f::PROJECT)), (w::PlaygroundState::Running, Some(f::PROJECT))]
    {
        let mut root = crate::unlinked(&f::limits(), f::SEED);
        let (watch, _) = f::start_accounts(&mut root, List::with_capacity(0));
        deliver_demo_read(&mut root, page_with_state(state));
        let mut me = f::chats::me();
        let mut other = me.projects.get(0).unwrap().clone();
        other.project = OTHER_PROJECT;
        let mut projects = List::with_capacity(2);
        projects.push(me.projects.get(0).unwrap().clone()).unwrap();
        projects.push(other).unwrap();
        me.projects = projects;
        up(
            &mut root,
            jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
        );
        assert_eq!(root.pages().project(), expected);
    }
}

#[test]
fn resetting_reads_as_resetting() {
    let side = demo_side(w::PlaygroundState::Resetting);
    assert_eq!(side.demo().unwrap().state, demo::DemoState::Resetting);
    assert!(side.leaving());
    assert!(!side.playground_running());
}
fn terminal(outcome: w::Outcome) -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }))
}
fn end_told(
    root: &mut crate::Domain,
    request: skein_lib::Token,
    outcome: jig_client::Outcome,
) -> Queue<crate::Request> {
    let mut out = Queue::uncapped();
    crate::route::told(
        root,
        &f::env(&f::limits()),
        Told::Answered {
            key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() },
            outcome,
        },
        &mut out,
    );
    out
}
#[test]
fn set_up_temper_sends_one_end_and_leaves_until_its_answer() {
    let (mut root, _) = opened(CUE);
    assert!(root.forge().playground_running());
    let mut out = act(&mut root, Act::LeavePlayground);
    let request = root.forge().ending().unwrap().request;
    assert!(root.forge().leaving());
    assert_eq!(root.demo().unwrap().project, f::PROJECT);
    assert!(!root.forge().playground_running());
    assert!(act(&mut root, Act::LeavePlayground).is_empty());
    // The public action retains one keyed deployment request through navigation.
    assert!(
        root.client()
            .pending(jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() })
            .is_some()
    );
    let jig_client::Ask::Family { scope, record } = demo::end_ask(&f::limits().forge).unwrap() else {
        panic!("family request")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request(), &w::Request::EndPlayground);
    event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Forges) });
    assert!(root.forge().leaving());
    assert!(!out.is_empty());
    out = end_told(&mut root, request, terminal(w::Outcome::NotOffered));
    demo_reread(&mut out);
    assert!(!root.forge().leaving());
    assert_eq!(root.forge().ending().unwrap().answer, Some(demo::EndAnswer::NotOffered));
}
fn forge_navigation(out: &Queue<crate::Request>) -> usize {
    let mut count = 0_usize;
    for request in out {
        if let crate::Request::Show {
            address: crate::Address::Welcome { step: crate::Step::Forge { forge: None } },
            replace: false,
        } = request
        {
            count = count.checked_add(1).expect("navigation count fits");
        }
    }
    count
}
#[test]
fn reset_reads_again_and_opens_the_welcomes_forge_step_once() {
    let (mut root, _) = opened(CUE);
    act(&mut root, Act::LeavePlayground);
    let request = root.forge().ending().unwrap().request;
    let mut out = end_told(&mut root, request, terminal(w::Outcome::Reset));
    assert_eq!(forge_navigation(&out), 1);
    let read = demo_reread(&mut out);
    assert_eq!(forge_navigation(&end_told(&mut root, request, terminal(w::Outcome::Reset))), 0);
    let out = up(&mut root, jig_client::Up::ReadDone { read, result: page_with_state(w::PlaygroundState::NotStarted) });
    assert_eq!(forge_navigation(&out), 0);
    assert!(!root.forge().leaving());
}
#[test]
fn not_offered_and_unknown_read_again() {
    for outcome in [terminal(w::Outcome::NotOffered), terminal(w::Outcome::Unknown), jig_client::Outcome::Unknown] {
        let (mut root, _) = opened(CUE);
        act(&mut root, Act::LeavePlayground);
        let request = root.forge().ending().unwrap().request;
        let mut out = end_told(&mut root, request, outcome);
        assert_eq!(forge_navigation(&out), 0);
        let read = demo_reread(&mut out);
        up(&mut root, jig_client::Up::ReadDone { read, result: page() });
        assert!(root.forge().playground_running());
    }
}
#[test]
fn a_resetting_read_reads_again_every_two_seconds_then_opens_the_forge_step() {
    use skein_lib::{Duration, Time};
    let (mut root, _, _, _) =
        ending_root_with_page(Some(c::HeldRole::Owner), page_with_state(w::PlaygroundState::Resetting));
    assert_eq!(root.demo().unwrap().state, demo::DemoState::Resetting);
    let mut env = f::env(&f::limits());
    for second in [2, 4] {
        assert_eq!(
            forge::next_deadline(root.forge()),
            Some(Time::ZERO.checked_add(Duration::from_secs(second)).unwrap())
        );
        let mut out = Queue::uncapped();
        env.now = Time::ZERO.checked_add(Duration::from_millis(second * 1000 - 1)).unwrap();
        crate::fire(&mut root, &env, &mut out);
        assert!(out.is_empty(), "no early poll");
        env.now = Time::ZERO.checked_add(Duration::from_secs(second)).unwrap();
        crate::fire(&mut root, &env, &mut out);
        let read = demo_reread(&mut out);
        assert_eq!(forge::next_deadline(root.forge()), None, "one read in flight");
        crate::fire(&mut root, &env, &mut out);
        assert!(out.is_empty());
        let result = if second == 2 {
            page_with_state(w::PlaygroundState::Resetting)
        } else {
            page_with_state(w::PlaygroundState::NotStarted)
        };
        crate::step(
            &mut root,
            &env,
            crate::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result } } },
            &mut out,
        );
        assert_eq!(forge_navigation(&out), usize::from(second == 4));
    }
    let mut out = Queue::uncapped();
    crate::fire(&mut root, &env, &mut out);
    assert_eq!(forge_navigation(&out), 0);
    assert_eq!(forge::next_deadline(root.forge()), None);
}
fn start_project(project: u32) -> (crate::Domain, Queue<crate::Request>, skein_lib::Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me_watch, _) = f::start_accounts(&mut root, List::with_capacity(0));
    let mut me = f::chats::me();
    if project == 0 {
        me.projects = List::with_capacity(0);
    }
    up(
        &mut root,
        jig_client::Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
    );
    let mut parts = wire::decode_playground_answer(&answer(CUE), &w::CEILINGS).unwrap().into_parts();
    parts.project = project;
    parts.state = w::PlaygroundState::NotStarted;
    deliver_demo_read(
        &mut root,
        Readout::Family {
            family: wire::FORGE,
            record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
        },
    );
    let out = act(&mut root, Act::Playground);
    (root, out, me_watch)
}
#[test]
fn a_start_with_no_project_selects_the_read_project_once_running() {
    for read_first in [false, true] {
        let (mut root, _, me_watch) = start_project(0);
        assert_eq!(root.pages().project(), None);
        let request = root.forge().demo_request().unwrap();
        let mut out = end_told(&mut root, request, terminal(w::Outcome::Running));
        assert_eq!(root.pages().project(), None);
        let read = demo_reread(&mut out);
        if read_first {
            up(&mut root, jig_client::Up::ReadDone { read, result: page() });
            assert_eq!(root.pages().project(), None, "wait until Me names the project");
        }
        let mut me = f::chats::me();
        let original = me.projects.get(0).unwrap().clone();
        let mut other = original.clone();
        other.project = 2;
        me.projects = List::with_capacity(2);
        me.projects.push(other).unwrap();
        me.projects.push(original).unwrap();
        up(
            &mut root,
            jig_client::Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
        );
        if !read_first {
            assert_eq!(root.pages().project(), Some(2));
            up(&mut root, jig_client::Up::ReadDone { read, result: page() });
        }
        assert_eq!(root.pages().project(), Some(f::PROJECT));
        act(&mut root, Act::Project { project: 2 });
        assert_eq!(root.pages().project(), Some(2), "automatic selection happens once");
    }
}

#[test]
fn a_failed_start_selects_the_project_it_made() {
    for read_first in [false, true] {
        let (mut root, _, me_watch) = start_project(0);
        assert_eq!(root.pages().project(), None);
        let request = root.forge().demo_request().unwrap();
        let mut out = end_told(
            &mut root,
            request,
            terminal(w::Outcome::StartFailed(
                w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
            )),
        );
        assert_eq!(root.pages().project(), None);
        assert_eq!(root.forge().demo_failure(), Some(&demo::StartFailure::Account));
        let read = demo_reread(&mut out);
        if read_first {
            up(&mut root, jig_client::Up::ReadDone { read, result: page() });
            assert_eq!(root.pages().project(), None, "wait until Me names the project");
        }
        let mut me = f::chats::me();
        let original = me.projects.get(0).unwrap().clone();
        let mut other = original.clone();
        other.project = 2;
        me.projects = List::with_capacity(2);
        me.projects.push(other).unwrap();
        me.projects.push(original).unwrap();
        up(
            &mut root,
            jig_client::Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
        );
        if !read_first {
            assert_eq!(root.pages().project(), Some(2));
            up(&mut root, jig_client::Up::ReadDone { read, result: page() });
        }
        assert_eq!(root.pages().project(), Some(f::PROJECT));
        assert_eq!(root.forge().demo_failure(), Some(&demo::StartFailure::Account));
        act(&mut root, Act::Project { project: 2 });
        assert_eq!(root.pages().project(), Some(2), "automatic selection happens once");
    }
}

#[test]
fn a_start_with_a_project_asks_nothing_more() {
    let (mut root, mut out, _) = start_project(f::PROJECT);
    assert_eq!(root.pages().project(), Some(f::PROJECT));
    for request in &out {
        if let crate::Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Read { .. } } } =
            request
        {
            panic!("the recorded path asks for no extra read");
        }
    }
    let request = root.forge().demo_request().unwrap();
    out = end_told(&mut root, request, terminal(w::Outcome::Running));
    demo_reread(&mut out); // The recorded path's existing post-start read, with no extra read.
    assert!(out.is_empty());
}

fn scripted_deployment() -> c::OwnerDeployment {
    let bounds = jig_client_protocol_link::Limits::new(f::limits().client.domain, 8).unwrap();
    let mut document = jig_client_protocol_link::translate::documents::owner_deployment(
        jig_client_test_records::owner_deployment_record(),
        &bounds,
    )
    .unwrap();
    let personal = accounts(Some(PROVIDER), true, false);
    let line = personal.accounts.get(0).unwrap();
    document.accounts = List::with_capacity(1);
    document
        .accounts
        .push(c::OwnerAccount {
            number: 42,
            place: 0,
            provider: Some(PROVIDER),
            provider_name: None,
            manage_usage: None,
            display: line.display.clone(),
            holder: c::OwnerAccountHolder::Deployment,
            uses: None,
            plan: false,
            sources: line.sources.clone(),
            generation: None,
            state: c::OwnerAccountState::Granted,
            usable: true,
            expiring: false,
            attention: false,
            spent_until: None,
            runs: List::with_capacity(0),
            waiting: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    document
}

#[test]
fn a_real_account_never_removes_the_scripted_account() {
    for role in [None, Some(c::HeldRole::Member), Some(c::HeldRole::Maintainer), Some(c::HeldRole::Owner)] {
        let (mut root, account, forges, watch) = ending_root(role);
        assert!(watch.is_none(), "the playground opens no cleanup watch");
        if role.is_some() {
            forge_rows(&mut root, forges, false);
        }
        let deployment = if role == Some(c::HeldRole::Owner) {
            let mut out =
                event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Accounts) });
            Some(deployment_watch(&mut out).expect("Accounts owns its deployment watch"))
        } else {
            None
        };
        for usable in [true, false, true] {
            let mut out = deliver_accounts(&mut root, account, ending_accounts(usable));
            assert!(removed(&out).is_empty(), "{role:?}: no scripted account removal");
            assert!(deployment_watch(&mut out).is_none(), "no cleanup watch after account changes");
            if let Some(watch) = deployment {
                let out = up(
                    &mut root,
                    jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(scripted_deployment()))),
                    },
                );
                assert!(removed(&out).is_empty(), "the delivered scripted account stays");
                assert_eq!(root.pages().accounts().unwrap().deployment(root.client()).unwrap().accounts.len(), 1);
            }
            assert_eq!(root.demo().unwrap().provider, PROVIDER);
            assert!(root.forge().playground_running());
        }
    }
}

#[test]
fn a_real_forge_connecting_leaves_the_playground_running() {
    let (mut root, account, forges, watch) = ending_root(Some(c::HeldRole::Owner));
    assert!(watch.is_none());
    deliver_accounts(&mut root, account, ending_accounts(true));
    for connected in [false, true, true, false] {
        let out = forge_rows(&mut root, forges, connected);
        assert!(out.is_empty(), "a real forge changes no playground state or watches");
        assert!(removed(&out).is_empty());
        assert_eq!(root.demo().unwrap().project, f::PROJECT);
        assert_eq!(root.pages().project(), Some(f::PROJECT));
        assert!(root.forge().playground_running());
    }
}

#[test]
fn the_playground_forge_serves_no_pages_while_it_runs_or_leaves() {
    for state in [w::PlaygroundState::Running, w::PlaygroundState::Resetting] {
        let side = demo_side(state);
        assert!(!side.serves_pages(wire::FORGE));
        assert!(side.serves_pages(wire::FORGE + 1));
    }
    let (mut root, _) = opened(CUE);
    act(&mut root, Act::LeavePlayground);
    assert!(!root.forge().serves_pages(wire::FORGE), "the pending reset serves no pages");
    deliver_demo_read(&mut root, Readout::Family { family: wire::FORGE, record: None });
    // Without an answer there is no retained retirement filter.
    assert!(root.forge().serves_pages(wire::FORGE));
}
