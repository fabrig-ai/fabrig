use crate::tests::{demo, fixtures as f};
use crate::{
    Address, Domain, Event, Place, Request, Step, forge,
    pages::welcome::{self, Setup},
};
use jig_client::{Down, Link, Readout, Told, Up, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn go(root: &mut Domain, step: Step) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(Address::Welcome { step }) })
}
fn deliver(root: &mut Domain, watch: Token, delivery: c::Delivery) -> Queue<Request> {
    up(root, Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) })
}
fn answer(root: &mut Domain, state: w::PlaygroundState) {
    let mut out = Queue::with_capacity(1);
    forge::demo::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut out);
    forge::demo::told(&mut root.forge, &Told::Link { link: Link::Live }, f::limits().forge, &mut out);
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, .. } }) = out.pop() else {
        panic!("demo read")
    };
    let page = {
        let mut parts =
            wire::decode_playground_answer(&demo::answer(b"Change the title"), &w::CEILINGS).unwrap().into_parts();
        parts.state = state;
        Readout::Family {
            family: wire::FORGE,
            record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
        }
    };
    crate::route::told(root, &f::env(&f::limits()), Told::Page { handle, page }, &mut Queue::uncapped());
}
fn opened(owner: bool, step: Step) -> (Domain, Token, Token, Option<Token>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, accounts) = f::start_accounts(&mut root, List::with_capacity(0));
    let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
    frame.projects = List::with_capacity(1);
    frame
        .projects
        .push(c::MeProject {
            project: f::PROJECT,
            name: jig_client::contract::DocumentText::complete(
                jig_client::contract::Text::new(b"Project 1".as_slice().into(), 64).expect("project name"),
            ),
            role: c::HeldRole::Member,
            pool: None,
            templates: List::with_capacity(0),
            creating: None,
            models: List::with_capacity(0),
        })
        .unwrap();
    frame.providers = List::with_capacity(1);
    frame
        .providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(c::Text::new(b"openai".as_slice().into(), 32).unwrap()),
            key_bytes: 256,
            sign_in: true,
            signed_in_plan: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .unwrap();
    frame.deployment_role = Some(if owner { c::HeldRole::Owner } else { c::HeldRole::Member });
    let mut out = deliver(&mut root, me, c::Delivery::Me(frame));
    go(&mut root, step);
    let mut forges = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Open { watch, subject: jig_client::Watchable::Family { .. } } },
        } = request
        {
            forges = Some(watch);
        }
    }
    (root, me, accounts, forges)
}
fn rows(root: &mut Domain, watch: Token, row: Option<(bool, bool)>) -> Queue<Request> {
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    let mut forges = List::with_capacity(1);
    if let Some((finished, playground)) = row {
        forges
            .push(
                w::ForgeRow::new(
                    &limits,
                    w::ForgeRowParts {
                        skein_bools: [false, false, finished, playground],
                        number: wire::FORGE + 1,
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Connected,
                        connection: if finished {
                            w::Connection::Connected(
                                w::Connected::new(
                                    &limits,
                                    w::ConnectedParts { id: 42, login: b"bot".as_slice().into() },
                                )
                                .unwrap(),
                            )
                        } else {
                            w::Connection::NotConnected
                        },
                        offer: w::Offer::NotOffered,
                        first: None,
                        installations: List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: List::with_capacity(0),
                        owned: None,
                        ended_stop: None,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges }).unwrap()),
            },
        },
    )
}
fn account(root: &mut Domain, watch: Token, provider: u32, usable: bool) -> Queue<Request> {
    deliver(root, watch, c::Delivery::OwnerAccounts(Box::new(demo::accounts(Some(provider), usable, false))))
}

fn store(row: Option<(bool, bool)>, model: bool) -> Domain {
    let (mut root, _, accounts, forges) = opened(true, Step::Model { way: None });
    answer(&mut root, w::PlaygroundState::NotStarted);
    account(&mut root, accounts, 18, model);
    rows(&mut root, forges.unwrap(), row);
    root
}
#[test]
fn no_forge_and_no_model_is_empty() {
    let root = store(None, false);
    assert_eq!(welcome::setup(&root), Some(Setup::Empty));
    assert_eq!(welcome::decision(&root), Some(false));
    assert!(welcome::playground_offered(&root));
}
#[test]
fn a_real_account_alone_is_unfinished_at_forge_and_keeps_the_playground_offered() {
    let root = store(None, true);
    assert_eq!(welcome::setup(&root), Some(Setup::Unfinished(Step::Forge { forge: None })));
    assert_eq!(welcome::decision(&root), Some(false));
    assert!(welcome::playground_offered(&root));
}
#[test]
fn a_forge_not_finished_is_unfinished_at_forge_and_withdraws_the_playground() {
    let root = store(Some((false, false)), false);
    assert_eq!(welcome::setup(&root), Some(Setup::Unfinished(Step::Forge { forge: None })));
    assert_eq!(welcome::decision(&root), Some(false));
    assert!(!welcome::playground_offered(&root));
}
#[test]
fn a_finished_forge_without_a_model_is_unfinished_at_model() {
    let root = store(Some((true, false)), false);
    assert_eq!(welcome::setup(&root), Some(Setup::Unfinished(Step::Model { way: None })));
    assert_eq!(welcome::decision(&root), Some(false));
    assert!(!welcome::playground_offered(&root));
}
#[test]
fn a_finished_forge_and_a_model_is_whole() {
    let root = store(Some((true, false)), true);
    assert_eq!(welcome::setup(&root), Some(Setup::Whole));
    assert_eq!(welcome::decision(&root), Some(true));
    assert!(!welcome::playground_offered(&root));
}
#[test]
fn the_playgrounds_forge_is_no_real_forge() {
    let root = store(Some((true, true)), false);
    assert_eq!(welcome::setup(&root), Some(Setup::Empty));
    assert_eq!(welcome::decision(&root), Some(false));
    assert!(welcome::playground_offered(&root));
}
#[test]
fn a_running_playground_decides_over() {
    for state in [w::PlaygroundState::Running, w::PlaygroundState::Resetting] {
        let mut root = store(None, false);
        answer(&mut root, state);
        assert_eq!(welcome::decision(&root), Some(true));
        assert!(!welcome::playground_offered(&root));
        let mut out = go(&mut root, Step::Welcome);
        assert_eq!(
            out.pop(),
            Some(Request::Show {
                address: Address::Chats { filter: crate::ChatsFilter::All, start: false },
                replace: true,
            })
        );
        assert!(out.is_empty());
    }
}
#[test]
fn the_welcome_opens_at_the_step_left_undone_once() {
    for (row, model, step) in [
        (Some((true, false)), false, Step::Model { way: None }),
        (Some((false, false)), true, Step::Forge { forge: None }),
    ] {
        let mut root = store(row, model);
        let mut out = go(&mut root, Step::Welcome);
        assert_eq!(out.pop(), Some(Request::Show { address: Address::Welcome { step }, replace: true }));
        assert!(out.is_empty());
        let mut again = Queue::uncapped();
        crate::fire(&mut root, &f::env(&f::limits()), &mut again);
        assert!(again.is_empty());
        assert!(go(&mut root, Step::Forge { forge: None }).is_empty());
    }
}

#[test]
fn setup_waits_for_its_facts_even_while_the_playground_runs() {
    let root = crate::unlinked(&f::limits(), f::SEED);
    assert_eq!(welcome::setup(&root), None);
    assert_eq!(welcome::decision(&root), None);
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, w::PlaygroundState::Running);
    assert_eq!(welcome::setup(&root), None);
    assert_eq!(welcome::decision(&root), None);
    rows(&mut root, forges.unwrap(), None);
    assert_eq!(welcome::setup(&root), None);
    assert_eq!(welcome::decision(&root), None);
    account(&mut root, accounts, 18, false);
    assert_eq!(welcome::setup(&root), Some(Setup::Empty));
    assert_eq!(welcome::decision(&root), Some(true));
}
#[test]
fn a_non_owner_needs_only_a_real_model() {
    for (model, expected) in [(false, Setup::Empty), (true, Setup::Whole)] {
        let (mut root, _, accounts, _) = opened(false, Step::Welcome);
        answer(&mut root, w::PlaygroundState::NotStarted);
        account(&mut root, accounts, 18, model);
        assert_eq!(welcome::setup(&root), Some(expected));
        assert_eq!(welcome::decision(&root), Some(model));
    }
}
