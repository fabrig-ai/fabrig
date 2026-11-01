//! The playground stays whole until its reset (temper/docs/design/draft/playground-reset.md, 6.1–6.3).
use super::{demo, fixtures as f};
use crate::{Action, Address, Domain, Event, Place, Request, forge, pages::Act};
use jig_client::{Down, Readout, Up, contract as c};
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
fn act(root: &mut Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(act) })
}
fn answer(root: &mut Domain, state: w::PlaygroundState, forge: Option<u16>, project: Option<u32>) {
    let mut parts =
        wire::decode_playground_answer(&demo::answer(b"Change the title"), &w::CEILINGS).unwrap().into_parts();
    parts.state = state;
    parts.forge = forge;
    parts.project = project;
    demo::deliver_demo_read(
        root,
        Readout::Family {
            family: wire::FORGE,
            record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
        },
    );
}
fn opened() -> (Domain, Token, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, accounts) = f::start_accounts(&mut root, List::with_capacity(0));
    let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { unreachable!() };
    frame.deployment_role = Some(c::HeldRole::Owner);
    let out = up(&mut root, Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(frame)) });
    let mut forges = None;
    for request in &out {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Open { watch, subject: jig_client::Watchable::Family { .. } } },
        } = request
        {
            forges = Some(*watch);
        }
    }
    answer(&mut root, w::PlaygroundState::NotStarted, None, None);
    (root, me, accounts, forges.unwrap())
}

#[test]
fn the_read_follows_the_playgrounds_state() {
    let (mut root, _, _, _) = opened();
    assert!(root.demo().is_none());
    for state in [w::PlaygroundState::Running, w::PlaygroundState::Resetting] {
        answer(&mut root, state.clone(), Some(4), Some(7));
        let demo = root.demo().unwrap();
        assert_eq!((demo.forge, demo.project), (Some(4), Some(7)));
        assert_eq!(
            demo.state,
            match state {
                w::PlaygroundState::Running => forge::demo::DemoState::Running,
                w::PlaygroundState::Resetting => forge::demo::DemoState::Resetting,
                w::PlaygroundState::NotStarted => unreachable!(),
            }
        );
        assert_eq!(root.forge().demo_resetting(), state == w::PlaygroundState::Resetting);
    }
    for request in &act(&mut root, Act::Playground) {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { .. } } } = request {
            panic!("no further start");
        }
    }
    answer(&mut root, w::PlaygroundState::Running, None, None);
    assert_eq!((root.demo().unwrap().forge, root.demo().unwrap().project), (None, None));
    assert!(forge::listed::playground(&root).is_none());
    assert!(root.forge().serves_pages(4));
}

#[test]
fn try_the_playground_sends_start_playground_at_the_deployments_scope() {
    let (mut root, _, _, _) = opened();
    let out = act(&mut root, Act::Playground);
    let mut count = 0_usize;
    let mut chats = false;
    for request in &out {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Send { request: jig_client::Ask::Family { scope, record }, .. } },
        } = request
        {
            count += 1;
            assert_eq!(*scope, c::Scope::Deployment);
            assert_eq!(
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request(),
                &w::Request::StartPlayground
            );
        }
        if let Request::Show { address: Address::Chats { .. }, .. } = request {
            chats = true;
        }
    }
    assert_eq!(count, 1);
    assert!(chats);
    assert!(root.demo().is_some());
    assert!(root.forge().demo_starting());
    for request in &act(&mut root, Act::Playground) {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { .. } } } = request {
            panic!("no further start");
        }
    }
}

#[test]
fn a_connected_forge_and_a_usable_account_end_nothing() {
    let (mut root, _, accounts, forges) = opened();
    answer(&mut root, w::PlaygroundState::Running, Some(4), Some(7));
    let mut rows = List::with_capacity(1);
    rows.push(
        w::ForgeRow::new(
            &w::CEILINGS,
            w::ForgeRowParts {
                skein_bools: [false, false, false, false],
                number: 5,
                kind: w::Kind::Forgejo,
                mode: w::Mode::Owned,
                connection: w::Connection::Connected(
                    w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"bot".as_slice().into() })
                        .unwrap(),
                ),
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
    let forge_out = up(
        &mut root,
        Up::FamilyDelivered {
            watch: forges,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(
                    &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, owned: w::Offer::NotOffered })
                        .unwrap(),
                ),
            },
        },
    );
    let account_out = up(
        &mut root,
        Up::Watched {
            watch: accounts,
            event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(demo::accounts(
                Some(18),
                true,
                false,
            )))),
        },
    );
    assert!(root.demo().is_some());
    for out in [&forge_out, &account_out] {
        for request in out {
            if let Request::Client {
                request:
                    jig_client::Request::Down {
                        down:
                            Down::Read { .. }
                            | Down::Send { .. }
                            | Down::Open { subject: jig_client::Watchable::Jig(c::Subject::Deployment), .. },
                    },
            } = request
            {
                panic!("no second read, account removal, or deployment watch");
            }
        }
    }
}

#[test]
fn the_starting_session_selects_the_playgrounds_project_once_named() {
    for named_first in [false, true] {
        let (mut root, me, _, _) = opened();
        act(&mut root, Act::Playground);
        event(
            &mut root,
            Event::Went { place: Place::Address(Address::Chats { filter: crate::ChatsFilter::All, start: false }) },
        );
        answer(&mut root, w::PlaygroundState::Running, Some(4), None);
        assert_eq!(root.pages().project(), None);
        if named_first {
            answer(&mut root, w::PlaygroundState::Running, Some(4), Some(7));
        }
        let mut frame = f::chats::me();
        let mut first = frame.projects.get(0).unwrap().clone();
        first.project = 8;
        let mut playground = first.clone();
        playground.project = 7;
        let mut projects = List::with_capacity(2);
        projects.push(first).unwrap();
        projects.push(playground).unwrap();
        frame.projects = projects;
        up(&mut root, Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) });
        if !named_first {
            assert_eq!(root.pages().project(), Some(8));
            answer(&mut root, w::PlaygroundState::Running, Some(4), Some(7));
        }
        assert_eq!(root.pages().project(), Some(7));
        act(&mut root, Act::Project { project: 8 });
        answer(&mut root, w::PlaygroundState::Running, Some(4), Some(7));
        assert_eq!(root.pages().project(), Some(8), "select only once");
    }
}
