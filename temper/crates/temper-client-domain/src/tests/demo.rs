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
                project: Some(f::PROJECT),
                forge: Some(wire::FORGE),
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
        ("real model keeps suggestions", CUE, false, f::PROJECT, false, 0, 1),
        ("past cues", CUE, false, f::PROJECT, true, 1, 0),
    ] {
        let (mut root, account) = opened(words);
        if pending {
            act(&mut root, Act::StartChat);
        }
        if project != f::PROJECT {
            let limits = forge::wire_limits(&f::limits().forge).unwrap();
            let mut parts = wire::decode_playground_answer(&answer(CUE), &limits).unwrap().into_parts();
            parts.project = Some(project);
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
    demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut out);
    let handle = read(&mut side);
    demo::told(&mut side, &Told::Page { handle, page: page() }, f::limits().forge, &mut out);
    assert_eq!(side.demo().unwrap().state, demo::DemoState::Running);
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

pub(super) fn deliver_demo_read(root: &mut crate::Domain, page: Readout) -> Queue<crate::Request> {
    demo::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
    let handle = read(&mut root.forge);
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Page { handle, page }, &mut out);
    out
}
