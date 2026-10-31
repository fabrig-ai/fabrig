use super::fixtures as f;
use crate::{
    forge::{self, playground},
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
    assert!(!playground::told(side, &Told::Link { link: Link::Live }, f::limits().forge, &mut out));
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, read } }) = out.pop() else {
        panic!("playground read");
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
fn the_playground_is_read_once_per_live_link_and_replaced_pages_are_dropped() {
    for (row, gap, reads) in [
        ("live again", Link::Live, false),
        ("offline then live", Link::Offline, true),
        ("signed out then live", Link::SignedOut, true),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let first = read(&mut side);
        let mut out = Queue::with_capacity(1);
        assert!(!playground::told(&mut side, &Told::Link { link: gap }, f::limits().forge, &mut out), "{row}");
        assert!(out.is_empty(), "{row}");
        if reads {
            let next = read(&mut side);
            assert_ne!(first, next, "{row}");
            assert!(playground::told(
                &mut side,
                &Told::Page { handle: first, page: page() },
                f::limits().forge,
                &mut out
            ));
            assert!(side.playground().is_none(), "{row}: replaced page dropped");
            playground::told(&mut side, &Told::Page { handle: next, page: page() }, f::limits().forge, &mut out);
            assert!(side.playground().is_some(), "{row}");
        } else {
            assert!(!playground::told(&mut side, &Told::Link { link: Link::Live }, f::limits().forge, &mut out));
            assert!(out.is_empty(), "{row}");
        }
    }
}
#[test]
fn a_playground_page_replaces_the_last_answer_or_clears_it() {
    let mut another_version = answer(CUE);
    *another_version.get_mut(1).unwrap() = u8::try_from(wire::CLIENT_VERSION + 1).unwrap();
    for (row, page, present) in [
        ("playground", page(), true),
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
        assert!(!side.playground_answered(), "{row}: no answer yet");
        playground::told(
            &mut side,
            &Told::Page { handle: first, page: self::page() },
            f::limits().forge,
            &mut Queue::with_capacity(1),
        );
        let shown = forge::shown(&side);
        playground::told(
            &mut side,
            &Told::Link { link: Link::Offline },
            f::limits().forge,
            &mut Queue::with_capacity(1),
        );
        assert!(side.playground().is_some(), "{row}: keep the last answer across the gap");
        assert!(side.playground_answered(), "{row}: keep that the read answered across the gap");
        let handle = read(&mut side);
        assert!(
            playground::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1)),
            "{row}"
        );
        assert_eq!(side.playground().is_some(), present, "{row}");
        assert_eq!(forge::shown(&side) != shown, !present, "{row}: redraw only on change");
    }
}
pub(super) fn accounts(provider: Option<u32>, usable: bool, cut: bool) -> c::Accounts {
    let mut sources = List::with_capacity(1);
    sources
        .push(c::AccountSource {
            name: None,
            number: 1,
            place: 1,
            kind: c::AccountKind::Key,
            state: c::AccountState::Granted,
            usable,
            expiring: false,
            attention: false,
            spent_until: None,
        })
        .unwrap();
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::AccountLine {
            number: 1,
            place: 1,
            uses: None,
            plan: false,
            provider,
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(f::acts::text(b"Model")),
            holder: c::AccountHolder::Party(f::PARTY),
            sources,
            cut,
        })
        .unwrap();
    c::Accounts {
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
        (account, c::Delivery::Accounts(Box::new(accounts(Some(PROVIDER), true, false)))),
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
    deliver_playground_read(&mut root, Readout::Family { family: wire::FORGE, record: Some(answer(cue)) });
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
        ("playground project", CUE, false, f::PROJECT, true, 0, 1),
        ("at the composer's bound", &at_bound, false, f::PROJECT, true, 0, 1),
        ("above the receiving fixture", &above_receiving_fixture, false, f::PROJECT, true, 0, 1),
        ("pending", CUE, true, f::PROJECT, true, 0, 0),
        ("other project", CUE, false, f::PROJECT + 1, true, 0, 0),
        ("another provider", CUE, false, f::PROJECT, false, 0, 1),
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
            deliver_playground_read(&mut root, Readout::Family { family: wire::FORGE, record: Some(record) });
        }
        if !model {
            event(
                &mut root,
                crate::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch: account,
                            event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(accounts(
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
fn the_first_playground_read_answer_counts_even_when_it_has_no_record() {
    for page in [
        Readout::Family { family: wire::FORGE, record: None },
        Readout::NotServed { family: Some(wire::FORGE), served: None },
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let handle = read(&mut side);
        let before = forge::shown(&side);
        playground::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.playground_answered());
        assert!(side.playground().is_none());
        assert_eq!(side.playground_provider(), None);
        assert!(forge::shown(&side) > before, "an answered absence redraws the welcome");
        playground::told(
            &mut side,
            &Told::Link { link: Link::Offline },
            f::limits().forge,
            &mut Queue::with_capacity(1),
        );
        assert!(side.playground_answered());
    }
}

pub(super) type DeploymentAccount = (Option<u32>, bool, bool, c::AccountHolder);
pub(super) const PLAYGROUND_ACCOUNT: DeploymentAccount = (Some(PROVIDER), true, false, c::AccountHolder::Deployment);
pub(super) const OTHER_ACCOUNT: DeploymentAccount = (Some(PROVIDER + 1), true, false, c::AccountHolder::Deployment);
pub(super) fn deployment(rows: &[DeploymentAccount]) -> c::Deployment {
    let bounds = jig_client_protocol_link::Limits::new(f::limits().client.domain, 8).unwrap();
    let mut document = jig_client_protocol_link::translate::documents::deployment(
        jig_client_test_records::owner_deployment_record(),
        &bounds,
    )
    .unwrap();
    document.accounts = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for (provider, usable, cut, holder) in rows {
        let personal = accounts(*provider, *usable, *cut);
        let line = personal.accounts.get(0).unwrap();
        document
            .accounts
            .push(c::Account {
                number: if *provider == Some(PROVIDER) { 42 } else { 43 },
                place: 0,
                provider: *provider,
                provider_name: None,
                manage_usage: None,
                display: line.display.clone(),
                holder: *holder,
                uses: None,
                plan: false,
                sources: line.sources.clone(),
                generation: None,
                state: c::AccountState::Granted,
                usable: *usable,
                expiring: false,
                attention: false,
                spent_until: None,
                runs: List::with_capacity(0),
                waiting: List::with_capacity(0),
                cut: *cut,
            })
            .unwrap();
    }
    document
}
pub(super) fn deliver_deployment(
    root: &mut crate::Domain,
    watch: skein_lib::Token,
    document: c::Deployment,
) -> Queue<crate::Request> {
    event(
        root,
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(document))),
                },
            },
        },
    )
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
pub(super) fn deliver_playground_read(root: &mut crate::Domain, page: Readout) -> Queue<crate::Request> {
    playground::told(
        &mut root.forge,
        &Told::Link { link: Link::Offline },
        f::limits().forge,
        &mut Queue::with_capacity(1),
    );
    let handle = read(&mut root.forge);
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Page { handle, page }, &mut out);
    crate::route::playground_progress(root, &f::env(&f::limits()), &mut out);
    out
}
pub(super) fn removed(out: &mut Queue<crate::Request>) -> List<skein_lib::Token> {
    let mut attempts = List::with_capacity(4);
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
        {
            assert_eq!(
                *ask,
                c::Ask::AccountChange {
                    holder: c::AskAccountHolder::Deployment,
                    change: c::AccountChange::Remove { account: 42 }
                }
            );
            attempts.push(attempt).unwrap();
        }
    }
    attempts
}

fn confirmed() -> forge::Side {
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
    root.forge
}
fn choose(side: &mut forge::Side, act: forge::Act) -> Queue<forge::Request> {
    let mut out = Queue::uncapped();
    forge::step(
        side,
        &skein_lib::Env { limits: f::limits().forge, now: f::env(&f::limits()).now, wall: f::env(&f::limits()).wall },
        forge::Event::Act { act },
        &mut out,
    );
    out
}
fn sent(out: &mut Queue<forge::Request>, expected: w::Request) -> skein_lib::Token {
    let Some(forge::Request::Send { request, ask: jig_client::Ask::Family { scope, record }, .. }) = out.pop() else {
        panic!("playground request")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request(), &expected);
    assert!(out.is_empty());
    request
}
fn refresh(side: &mut forge::Side, page: Readout) -> Queue<forge::Request> {
    playground::told(side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::uncapped());
    let handle = read(side);
    let mut out = Queue::uncapped();
    assert!(playground::told(side, &Told::Page { handle, page }, f::limits().forge, &mut out));
    out
}
fn result(side: &mut forge::Side, request: skein_lib::Token, outcome: w::Outcome) -> Queue<forge::Request> {
    let mut out = Queue::uncapped();
    assert!(playground::told(
        side,
        &Told::Answered {
            key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() },
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
            })),
        },
        f::limits().forge,
        &mut out
    ));
    out
}
fn end(side: &mut forge::Side) -> skein_lib::Token {
    refresh(side, page());
    sent(&mut choose(side, forge::Act::EndPlayground), w::Request::EndPlayground)
}
#[test]
fn try_the_playground_sends_one_start() {
    let mut side = confirmed();
    refresh(&mut side, Readout::Family { family: wire::FORGE, record: None });
    sent(&mut choose(&mut side, forge::Act::StartPlayground), w::Request::StartPlayground);
    assert!(side.playground_starting());
    assert!(choose(&mut side, forge::Act::StartPlayground).is_empty());
}
#[test]
fn a_running_read_sends_no_start() {
    let mut side = confirmed();
    assert!(refresh(&mut side, page()).is_empty());
    assert!(choose(&mut side, forge::Act::StartPlayground).is_empty());
    assert!(!side.playground_starting());
    assert_eq!(side.playground().unwrap().project, Some(f::PROJECT));
}
#[test]
fn end_sends_one_end_and_shows_leaving_until_the_read_is_none() {
    let mut side = confirmed();
    end(&mut side);
    assert!(side.leaving());
    assert!(choose(&mut side, forge::Act::EndPlayground).is_empty());
    refresh(&mut side, page_with_state(w::PlaygroundState::Resetting));
    assert!(side.leaving());
    refresh(&mut side, Readout::Limit);
    assert!(side.leaving(), "an unavailable read does not finish a reset");
    let mut out = refresh(&mut side, Readout::Family { family: wire::FORGE, record: None });
    assert!(!side.leaving());
    let Some(forge::Request::PlaygroundReset) = out.pop() else { panic!("reset navigation") };
    assert!(out.is_empty());
}
#[test]
fn a_resetting_read_after_a_restart_shows_leaving() {
    let mut side = confirmed();
    let mut out = refresh(&mut side, page_with_state(w::PlaygroundState::Resetting));
    assert!(side.leaving());
    assert!(choose(&mut side, forge::Act::EndPlayground).is_empty());
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, .. } }) = out.pop() else {
        panic!("reset reread")
    };
    assert!(out.is_empty());
    playground::told(
        &mut side,
        &Told::Page { handle, page: Readout::Family { family: wire::FORGE, record: None } },
        f::limits().forge,
        &mut out,
    );
    assert!(!side.leaving());
    let Some(forge::Request::PlaygroundReset) = out.pop() else { panic!("reset navigation") };
    assert!(out.is_empty());
}

#[test]
fn the_reset_opens_the_welcome_at_forge() {
    for answered in [true, false] {
        let mut root = crate::unlinked(&f::limits(), f::SEED);
        root.forge = confirmed();
        let request = end(&mut root.forge);
        let mut child = if answered {
            result(&mut root.forge, request, w::Outcome::Reset)
        } else {
            refresh(&mut root.forge, Readout::Family { family: wire::FORGE, record: None })
        };
        assert!(!root.forge().leaving());
        let mut out = Queue::uncapped();
        crate::route::route_forge(&mut root, &f::env(&f::limits()), &mut child, &mut out);
        let mut addresses = List::with_capacity(1);
        while let Some(request) = out.pop() {
            if let crate::Request::Show { address, .. } = request {
                addresses.push(address).unwrap();
            }
        }
        assert_eq!(addresses.as_slice(), [crate::Address::Welcome { step: crate::Step::Forge { forge: None } }]);
        assert_eq!(crate::address::print(addresses.get(0).unwrap()).as_ref(), b"/welcome?at=forge");
    }
}
#[test]
fn not_offered_ends_leaving_and_rereads_without_navigation() {
    let mut side = confirmed();
    let request = end(&mut side);
    let mut out = result(
        &mut side,
        request,
        w::Outcome::EndRefused(
            w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
        ),
    );
    assert!(!side.leaving());
    let Some(forge::Request::Client { asked: jig_client::Event::Read { .. } }) = out.pop() else {
        panic!("playground reread")
    };
    assert!(out.is_empty());
}
#[test]
fn no_account_removal_is_ever_sent_by_the_client() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    let mut frame = f::chats::me();
    frame.deployment_role = Some(c::HeldRole::Owner);
    event(
        &mut root,
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))),
                },
            },
        },
    );
    let mut out = deliver_playground_read(&mut root, page());
    let watch = deployment_watch(&mut out).expect("owner deployment watch");
    for document in [deployment(&[PLAYGROUND_ACCOUNT, OTHER_ACCOUNT]), deployment(&[OTHER_ACCOUNT]), deployment(&[])] {
        let mut out = deliver_deployment(&mut root, watch, document);
        assert!(removed(&mut out).is_empty());
        assert!(root.forge().playground().is_some(), "a real account does not end the playground");
    }
    let mut out = event(&mut root, crate::Event::Act { action: crate::Action::Forge(forge::Act::EndPlayground) });
    assert!(removed(&mut out).is_empty());
    let mut out = deliver_playground_read(&mut root, Readout::Family { family: wire::FORGE, record: None });
    assert!(removed(&mut out).is_empty());
}

#[test]
fn a_starting_playground_can_have_no_project() {
    let mut side = confirmed();
    let mut parts = wire::decode_playground_answer(&answer(CUE), &w::CEILINGS).unwrap().into_parts();
    parts.state = w::PlaygroundState::Starting;
    parts.project = None;
    refresh(
        &mut side,
        Readout::Family {
            family: wire::FORGE,
            record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
        },
    );
    assert!(side.playground_starting());
    assert_eq!(side.playground().unwrap().project, None);
    assert_eq!(side.playground_provider(), Some(PROVIDER));
    assert!(choose(&mut side, forge::Act::StartPlayground).is_empty());
}

#[test]
fn a_failed_start_can_retry_with_a_new_key_before_resetting() {
    let mut side = confirmed();
    refresh(&mut side, Readout::Family { family: wire::FORGE, record: None });
    let first = sent(&mut choose(&mut side, forge::Act::StartPlayground), w::Request::StartPlayground);
    result(
        &mut side,
        first,
        w::Outcome::StartFailed(
            w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
        ),
    );
    assert_eq!(side.playground_failure(), Some(&playground::StartFailure::Account));
    assert!(!side.playground_starting());
    refresh(&mut side, page_with_state(w::PlaygroundState::Starting));
    let retry = sent(&mut choose(&mut side, forge::Act::StartPlayground), w::Request::StartPlayground);
    assert_ne!(first, retry);
    assert!(side.playground_failure().is_none());
    assert!(choose(&mut side, forge::Act::StartPlayground).is_empty());
    refresh(&mut side, page_with_state(w::PlaygroundState::Resetting));
    assert!(choose(&mut side, forge::Act::StartPlayground).is_empty());
}
