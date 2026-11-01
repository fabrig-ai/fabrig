//! Empty stores offer forge kinds, and made forges continue the welcome (temper/docs/design/draft/playground-reset.md, 3).
use super::{connect_forge as cf, fixtures as f};
use jig_client::{self as client, Down, Up};
use skein_lib::{List, Queue, Token};
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, Choice, Step, forge};
use temper_client_protocol_wire::{self as wire, v1 as w};

const WELCOME: Address = Address::Welcome { step: Step::Forge { forge: None } };
fn event(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        event,
        &mut out,
    );
    out
}
fn deliver(root: &mut d::Domain, watch: Token, owned: w::Offer, rows: List<w::ForgeRow>) -> Queue<d::Request> {
    event(
        root,
        d::Event::Client {
            event: client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(
                            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, owned }).unwrap(),
                        ),
                    },
                },
            },
        },
    )
}
fn empty(address: Address, offer: w::Offer) -> (d::Domain, Token) {
    let (mut root, watch) = f::forge_root_watch(address, List::with_capacity(0));
    deliver(&mut root, watch, offer, List::with_capacity(0));
    (root, watch)
}
fn press(root: &d::Domain, role: Role, name: &[u8]) -> d::Action {
    let mut view = f::browser_page_view(root);
    let node = f::find(view.tree(), role, name).expect("kind control");
    crate::act(&mut view, node, Act::Press).expect("kind binding")
}
fn assert_choices(root: &d::Domain) {
    let view = f::browser_page_view(root);
    for (role, name) in [
        (Role::Region, b"GitHub".as_slice()),
        (Role::Button, b"Connect GitHub"),
        (Role::Region, b"Forgejo"),
        (Role::Button, b"Run on this machine"),
        (Role::Region, b"A Forgejo elsewhere"),
    ] {
        assert!(f::find(view.tree(), role, name).is_some(), "{role:?}: {name:?}");
    }
    assert_eq!(
        press(root, Role::Button, b"Connect GitHub"),
        d::Action::Forge(forge::Act::ConnectManifest { forge: None, back: b"/welcome?at=forge".as_slice().into() })
    );
    assert_eq!(press(root, Role::Button, b"Run on this machine"), d::Action::Forge(forge::Act::SetUpAtOnce));
}
#[test]
fn the_welcome_with_no_row_draws_github_and_run_on_this_machine() {
    let (root, _) = empty(WELCOME, w::Offer::Offered);
    assert_choices(&root);
    let view = f::browser_page_view(&root);
    assert!(f::find(view.tree(), Role::Link, b"Connect as its admin").is_none());
}
#[test]
fn connect_a_forge_with_no_row_offers_a_forgejo_by_its_address() {
    let (mut root, _) = empty(Address::ConnectForge { forge: None }, w::Offer::Offered);
    assert_eq!(
        press(&root, Role::Link, b"Connect as its admin"),
        d::Action::Pages(d::pages::Act::Go { address: Address::ConnectForge { forge: Some(Choice::Forgejo) } })
    );
    event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::ConnectForge { forge: Some(Choice::Forgejo) }) },
    );
    let mut view = f::browser_page_view(&root);
    let field = f::find(view.tree(), Role::TextField, b"Forgejo address").expect("new address field");
    assert_eq!(
        crate::act(&mut view, field, Act::Input { value: b"https://forge.example".as_slice().into() }),
        Some(d::Action::Forge(forge::Act::AdminWrite {
            field: forge::admin::Field::Address,
            text: b"https://forge.example".as_slice().into()
        }))
    );
    assert!(f::find(view.tree(), Role::TextField, "The site admin’s token".as_bytes()).is_some());
    assert_eq!(
        press(&root, Role::Button, b"Connect as its admin"),
        d::Action::Forge(forge::Act::ConnectAdmin { forge: None })
    );
}
#[test]
fn after_a_start_over_the_welcome_draws_the_kinds_again() {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(
        3,
        w::Kind::Forgejo,
        w::Mode::Owned,
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
        ),
        w::Offer::NotOffered,
    ))
    .unwrap();
    let (mut root, watch) = f::forge_root_watch(WELCOME, rows);
    let action = press(&root, Role::Button, b"Start over");
    event(&mut root, d::Event::Act { action });
    deliver(&mut root, watch, w::Offer::Offered, List::with_capacity(0));
    assert_choices(&root);
}
#[test]
fn the_just_made_github_continues_its_installation() {
    let (mut root, watch) = empty(WELCOME, w::Offer::Offered);
    let action = press(&root, Role::Button, b"Connect GitHub");
    let out = event(&mut root, d::Event::Act { action });
    let mut attempt = None;
    for request in &out {
        if let d::Request::Client { request: client::Request::Down { down: Down::Send { attempt: sent, .. } } } =
            request
        {
            attempt = Some(*sent);
        }
    }
    event(
        &mut root,
        d::Event::Client {
            event: client::Event::Up {
                up: Up::Answer {
                    attempt: attempt.expect("manifest sent"),
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(client::Answered::Family(client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(
                            &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Connecting }).unwrap(),
                        ),
                    })),
                },
            },
        },
    );
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(
        7,
        w::Kind::Github,
        w::Mode::Managed,
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 11, login: b"temper-app".as_slice().into() })
                .unwrap(),
        ),
        w::Offer::NotOffered,
    ))
    .unwrap();
    let out = deliver(&mut root, watch, w::Offer::Offered, rows);
    let address = Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(7)) } };
    let mut replaced = false;
    for request in &out {
        if let d::Request::Show { address: shown, replace: true } = request {
            replaced |= *shown == address;
        }
    }
    assert!(replaced);
    // Even before the shell echoes Show as Went, the followed row is this session's live step.
    let view = f::browser_page_view(&root);
    assert!(f::find(view.tree(), Role::Button, b"Start over").is_none());
    event(&mut root, d::Event::Went { place: d::Place::Address(address.clone()) });
    let view = f::browser_page_view(&root);
    assert!(f::find(view.tree(), Role::Button, b"Install on GitHub").is_some());
    assert_eq!(
        press(&root, Role::Button, b"Install on GitHub"),
        d::Action::Forge(forge::Act::Install { forge: 7, back: d::address::print(&address) })
    );
}

#[test]
fn an_owned_kind_outside_the_box_opens_details_in_its_flow() {
    for (address, expected) in [
        (WELCOME, Address::Welcome { step: Step::Forge { forge: Some(Choice::Owned) } }),
        (Address::ConnectForge { forge: None }, Address::ConnectForge { forge: Some(Choice::Owned) }),
    ] {
        let (mut root, _) = empty(address, w::Offer::NotUnderTheBox);
        assert_eq!(
            press(&root, Role::Link, b"Run on this machine"),
            d::Action::Pages(d::pages::Act::Go { address: expected.clone() })
        );
        event(&mut root, d::Event::Went { place: d::Place::Address(expected) });
        let view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Button, b"Set up Forgejo here").is_none());
    }
}
