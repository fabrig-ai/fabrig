//! Every page's banner and the playground's lists (temper/docs/design/draft/playground-reset.md, 6.1–6.2).
use super::{connect_forge as cf, fixtures as f};
use skein_lib::{List, Queue};
use skein_ui::Role;
use temper_client_domain::{Action, Address, ChatsFilter, Domain, Event, PlanFilter, Step, pages::Act};
use temper_client_protocol_wire::{self as wire, v1 as w};

const BANNER: &[u8] = b"This is the playground: a scripted model and in-memory forge. Use the wizard to set up temper";
const NOTHING: &[u8] = b"Nothing to add while the playground runs: set up temper to bring your own.";

fn root(address: Address) -> Domain {
    let mut rows = List::with_capacity(1);
    rows.push(cf::row(wire::FORGE, w::Kind::Forgejo, w::Mode::Fake, cf::connected(), w::Offer::NotOffered)).unwrap();
    f::root_at_with_forges(
        address,
        &f::domain_limits(),
        b"Shop",
        Some((f::PROJECT, true, w::PlaygroundState::Running)),
        None,
        Some(rows),
    )
    .0
}
fn banner(view: &crate::View) -> &[skein_ui::Node] {
    let nodes = view.tree().nodes();
    let mut found = None;
    for (at, node) in nodes.iter().enumerate() {
        assert!(node.role != Role::Region || node.name.as_deref() != Some(b"Setup"), "no setup strip");
        if node.role == Role::Region && node.name.as_deref() == Some(b"Playground") {
            assert!(found.is_none(), "one Playground region");
            let end = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
            found = Some(nodes.get(at..end).unwrap());
        }
    }
    found.expect("Playground region")
}

#[test]
fn every_page_has_the_one_line_banner() {
    for address in [
        Address::Welcome { step: Step::Welcome },
        Address::Chats { filter: ChatsFilter::All, start: false },
        Address::Chat { task: 1 },
        Address::Task { task: 9, filter: PlanFilter::All },
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        Address::Adopt { project: f::PROJECT },
        Address::ProjectSettings { project: f::PROJECT },
        Address::Forges,
        Address::ConnectForge { forge: None },
        Address::Accounts,
        Address::ConnectModel { way: None },
    ] {
        let root = root(address.clone());
        let mut view = f::page_view(&root);
        let nodes = banner(&view);
        let mut paragraphs = 0_u32;
        let mut link = None;
        for node in nodes {
            if node.role == Role::Paragraph {
                paragraphs = paragraphs.checked_add(1).unwrap();
                assert_eq!(node.name.as_deref(), Some(BANNER), "{address:?}");
            }
            if node.role == Role::Link {
                assert!(link.is_none(), "one link: {address:?}");
                assert_eq!(node.name.as_deref(), Some(b"set up temper".as_slice()));
                link = Some(node.id);
            }
        }
        assert_eq!(paragraphs, 1, "{address:?}");
        assert_eq!(f::nodes_text(nodes).as_ref(), BANNER, "{address:?}");
        let link = link.unwrap();
        assert_eq!(crate::act(&mut view, link, skein_ui::Act::Press), Some(Action::Pages(Act::LeavePlayground)));
    }
}

#[test]
fn leaving_shows_its_line_and_no_link() {
    for resetting in [false, true] {
        let mut root = if resetting {
            f::demo_forges_state_root(
                Address::Forges,
                cf::rows(cf::connected(), w::Offer::NotOffered),
                w::PlaygroundState::Resetting,
            )
        } else {
            root(Address::Forges)
        };
        if !resetting {
            f::root_event(&mut root, Event::Act { action: Action::Pages(Act::LeavePlayground) });
        }
        assert!(root.forge().demo_leaving());
        let view = f::page_view(&root);
        let nodes = banner(&view);
        assert_eq!(f::nodes_text(nodes).as_ref(), "Leaving the playground…".as_bytes());
        for node in nodes {
            assert!(node.role != Role::Link && node.role != Role::Button);
        }
    }
}

#[test]
fn no_setup_strip_without_a_forge_or_model() {
    let root = f::forge_root(
        Address::Chats { filter: ChatsFilter::All, start: false },
        cf::rows(w::Connection::NotConnected, w::Offer::Offered),
    );
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Region, b"Setup").is_none());
    assert!(f::find(view.tree(), Role::Region, b"Playground").is_none());
}

#[test]
fn forges_and_accounts_offer_nothing_to_add_while_it_runs() {
    for address in [Address::Forges, Address::Accounts] {
        let mut root = root(address.clone());
        for ending in [false, true] {
            if ending {
                f::root_event(&mut root, Event::Act { action: Action::Pages(Act::LeavePlayground) });
            }
            let view = f::page_view(&root);
            for node in view.tree().nodes() {
                if node.role == Role::Link {
                    assert!(
                        ![
                            Some(b"+ Add a forge".as_slice()),
                            Some(b"Connect a forge"),
                            Some(b"Connect a model"),
                            Some(b"+ Connect a model")
                        ]
                        .contains(&node.name.as_deref())
                    );
                }
            }
            assert!(f::find(view.tree(), Role::Paragraph, NOTHING).is_some());
            let row = if address == Address::Forges {
                "Playground forge · in memory · acme/shop"
            } else {
                "Scripted model · the playground’s, until a model of yours connects · held by the deployment"
            };
            assert!(f::find(view.tree(), Role::Disclosure, row.as_bytes()).is_some());
        }
    }
}

#[test]
fn a_refused_reset_draws_try_again_bound_to_the_reset() {
    let mut root = root(Address::Forges);
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Act { action: Action::Pages(Act::LeavePlayground) },
        &mut out,
    );
    let mut attempt = None;
    for request in &out {
        if let temper_client_domain::Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Send { attempt: sent, .. } },
        } = request
        {
            assert!(attempt.is_none(), "one end");
            attempt = Some(*sent);
        }
    }
    let attempt = attempt.unwrap();
    let outcome = w::Outcome::EndRefused(
        w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
    );
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                    why: None,
                },
            },
        },
    );
    let mut view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, b"The playground was not left: not offered.").is_some());
    let retry = f::find(view.tree(), Role::Button, b"Try again").unwrap();
    assert_eq!(crate::act(&mut view, retry, skein_ui::Act::Press), Some(Action::Pages(Act::LeavePlayground)));
}
