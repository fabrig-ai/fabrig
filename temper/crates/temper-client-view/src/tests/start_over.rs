//! The welcome starts a cut-short set up over (temper/docs/design/draft/playground-reset.md, 6.4).
use super::{connect_forge as cf, fixtures as f};
use crate::words::Word;
use jig_client::{Down, Up};
use skein_lib::{List, Queue, Token};
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, Choice, Step, forge};
use temper_client_protocol_wire::{self as wire, v1 as w};

const FORGE: u16 = 3;
const WELCOME: Address = Address::Welcome { step: Step::Forge { forge: None } };
fn failed() -> w::Connection {
    w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: None })
            .unwrap(),
    )
}
fn rows(kind: w::Kind, connection: w::Connection) -> List<w::ForgeRow> {
    let mut rows = List::with_capacity(2);
    let mode = if kind == w::Kind::Github { w::Mode::Managed } else { w::Mode::Owned };
    rows.push(cf::row(FORGE, kind, mode, connection, w::Offer::NotOffered)).unwrap();
    rows
}
fn assert_word(tree: &skein_ui::Tree, role: Role, word: Word) {
    assert!(f::find(tree, role, &f::words().say(word, &[])).is_some(), "{word:?}");
}
fn assert_style(tree: &skein_ui::Tree, id: skein_ui::NodeId, style: skein_ui::Style) {
    for node in tree.nodes() {
        if node.id == id {
            assert!(node.styles.as_slice().contains(&style));
            return;
        }
    }
    panic!("control exists");
}
fn assert_no_choices(tree: &skein_ui::Tree) {
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&crate::look::FLOW_CHOICES));
    }
}
fn start_over(root: &mut d::Domain) -> Token {
    let mut view = f::page_view(root);
    let button = f::find(view.tree(), Role::Button, b"Start over").expect("start over");
    let action = crate::act(&mut view, button, Act::Press).expect("bound control");
    assert_eq!(action, d::Action::Forge(forge::Act::StartOver { forge: FORGE }));
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Act { action },
        &mut out,
    );
    for request in &out {
        if let d::Request::Client {
            request:
                jig_client::Request::Down {
                    down: Down::Send { request: jig_client::Ask::Family { record, .. }, attempt, .. },
                },
        } = request
            && let w::Request::RemoveForge(value) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        {
            assert_eq!(value.forge(), FORGE);
            return *attempt;
        }
    }
    panic!("removal sent unconfirmed");
}
fn answer(root: &mut d::Domain, attempt: Token, outcome: w::Outcome) {
    f::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                },
            },
        },
    );
}
#[test]
fn the_forge_step_shows_a_forgejo_not_finished_with_start_over() {
    let root = f::forge_root(WELCOME, rows(w::Kind::Forgejo, failed()));
    let view = f::page_view(&root);
    let tree = view.tree();
    assert!(f::find(tree, Role::Region, "Forgejo on this machine · not finished".as_bytes()).is_some());
    assert_word(tree, Role::Paragraph, Word::ForgeNotFinishedDescription);
    let button = f::find(tree, Role::Button, b"Start over").expect("start over");
    assert_style(tree, button, crate::look::FLOW_PRIMARY);
    let expected_status = f::words().say(
        Word::ForgeStatus,
        &[
            skein_ui::phrases::Hole::Name(b"Forgejo on this machine".as_slice().into()),
            skein_ui::phrases::Hole::Name(forge::words::forge_connection(
                forge::connect::unfinished(root.forges()).unwrap(),
            )),
        ],
    );
    assert!(f::find(tree, Role::Paragraph, &expected_status).is_some());
    assert_no_choices(tree);
}
#[test]
fn github_not_finished_offers_go_on_before_start_over() {
    let mut root = f::forge_root(WELCOME, rows(w::Kind::Github, cf::connected()));
    let mut view = f::page_view(&root);
    let tree = view.tree();
    assert!(f::find(tree, Role::Region, "GitHub · not finished".as_bytes()).is_some());
    assert_word(tree, Role::Paragraph, Word::ForgeGitHubNotFinished);
    assert_word(tree, Role::Paragraph, Word::ForgeGitHubStartOver);
    let go_on = f::find(tree, Role::Link, b"Go on").expect("go on");
    assert_style(tree, go_on, crate::look::FLOW_PRIMARY);
    let settings = f::find(tree, Role::Link, &f::words().say(Word::ForgeAppSettings, &[])).expect("App settings");
    for node in tree.nodes() {
        if node.id == settings {
            assert_eq!(node.external.as_deref(), Some(b"https://github.com/settings/apps".as_slice()));
        }
    }
    let button = f::find(tree, Role::Button, b"Start over").expect("start over");
    assert_style(tree, button, crate::look::FLOW_SECONDARY);
    cf::words_in_order(
        tree,
        &["Go on", "Starting over removes this forge", "The App’s settings on GitHub", "Start over"],
    );
    assert_no_choices(tree);
    let action = crate::act(&mut view, go_on, Act::Press).expect("go on bound");
    let address = Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(FORGE)) } };
    assert_eq!(action, d::Action::Pages(d::pages::Act::Go { address: address.clone() }));
    assert_eq!(d::address::print(&address).as_ref(), b"/welcome?at=forge&forge=3");
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
    let view = f::page_view(&root);
    assert_word(view.tree(), Role::Button, Word::ForgeInstall);
    assert!(f::find(view.tree(), Role::Button, b"Start over").is_none());
}
#[test]
fn starting_over_then_removed_draws_the_choices_again() {
    let mut initial = List::with_capacity(2);
    initial
        .push(cf::row(2, w::Kind::Github, w::Mode::Managed, w::Connection::NotConnected, w::Offer::NotOffered))
        .unwrap();
    initial.push(cf::row(FORGE, w::Kind::Forgejo, w::Mode::Owned, failed(), w::Offer::NotOffered)).unwrap();
    let (mut root, watch) = f::forge_root_watch(WELCOME, initial);
    let attempt = start_over(&mut root);
    for accepted in [false, true] {
        if accepted {
            answer(&mut root, attempt, w::Outcome::Removing);
        }
        let view = f::page_view(&root);
        assert_word(view.tree(), Role::Status, Word::ForgeStartingOver);
        let card = f::find(view.tree(), Role::Region, "Forgejo on this machine · not finished".as_bytes()).unwrap();
        let mut remaining = 0;
        for node in view.tree().nodes() {
            if node.id == card {
                remaining = node.size;
            }
            if remaining > 0 {
                assert_ne!(node.role, Role::Button);
                assert_ne!(node.role, Role::Link);
                remaining -= 1;
            }
        }
    }
    let mut remaining = List::with_capacity(1);
    remaining
        .push(cf::row(2, w::Kind::Github, w::Mode::Managed, w::Connection::NotConnected, w::Offer::NotOffered))
        .unwrap();
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(
                            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: remaining }).unwrap(),
                        ),
                    },
                },
            },
        },
    );
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Button, b"Connect GitHub").is_some());
    assert!(f::find(view.tree(), Role::Region, "Forgejo on this machine · not finished".as_bytes()).is_none());
    assert!(forge::connect::unfinished(root.forges()).is_none());
}
#[test]
fn a_live_set_up_keeps_its_progress() {
    let connection = w::Connection::SettingUp(
        w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Prepare) }).unwrap(),
    );
    let root = f::forge_root(WELCOME, rows(w::Kind::Forgejo, connection));
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Region, "Forgejo on this machine · not finished".as_bytes()).is_none());
    assert!(f::find(view.tree(), Role::Button, b"Start over").is_none());
    cf::words_in_order(view.tree(), &["Forgejo on this machine", "preparing", "View"]);
    let mut view = f::page_view(&root);
    let link = f::find(view.tree(), Role::Link, b"View").expect("live setup link");
    assert_eq!(
        crate::act(&mut view, link, Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go {
            address: Address::ConnectForge { forge: Some(Choice::Forge(FORGE)) }
        }))
    );
}
#[test]
fn an_unfinished_forge_shows_before_another_live_one() {
    let mut rows = List::with_capacity(2);
    rows.push(cf::row(2, w::Kind::Github, w::Mode::Managed, cf::connected(), w::Offer::NotOffered)).unwrap();
    rows.push(cf::row(FORGE, w::Kind::Forgejo, w::Mode::Owned, w::Connection::Connecting, w::Offer::NotOffered))
        .unwrap();
    let root = f::forge_root(WELCOME, rows);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Region, "GitHub · not finished".as_bytes()).is_some());
    assert_word(view.tree(), Role::Link, Word::ForgeGoOn);
    assert_no_choices(view.tree());
}
#[test]
fn a_refused_start_over_shows_why_and_can_be_tried_again() {
    for why in [w::RemoveRefusal::NotOffered, w::RemoveRefusal::Role] {
        let mut root = f::forge_root(WELCOME, rows(w::Kind::Forgejo, failed()));
        let attempt = start_over(&mut root);
        let refusal = match why {
            w::RemoveRefusal::NotOffered => jig_client::contract::Refusal::NotOffered,
            w::RemoveRefusal::Role => jig_client::contract::Refusal::Role { held: None },
            w::RemoveRefusal::NoSuchForge => unreachable!(),
        };
        answer(
            &mut root,
            attempt,
            w::Outcome::RemoveRefused(w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why }).unwrap()),
        );
        let view = f::page_view(&root);
        let reason = f::words().say_word(jig_client::words::Word::refusal(&refusal), &[]);
        let expected = f::words().say(Word::ForgeRemoveRefused, &[skein_ui::phrases::Hole::Name(reason)]);
        assert!(f::find(view.tree(), Role::Status, &expected).is_some());
        assert_word(view.tree(), Role::Button, Word::ForgeStartOver);
        start_over(&mut root);
        assert_word(f::page_view(&root).tree(), Role::Status, Word::ForgeStartingOver);
    }
}

#[test]
fn go_on_follows_the_unfinished_row_after_a_finished_forge() {
    let mut listed = List::with_capacity(2);
    let mut finished = cf::row(2, w::Kind::Forgejo, w::Mode::Owned, cf::connected(), w::Offer::NotOffered).into_parts();
    finished.skein_bools[2] = true;
    listed.push(w::ForgeRow::new(&w::CEILINGS, finished).unwrap()).unwrap();
    listed.push(cf::row(FORGE, w::Kind::Github, w::Mode::Managed, cf::connected(), w::Offer::NotOffered)).unwrap();
    let mut root = f::forge_root(WELCOME, listed);
    let mut view = f::page_view(&root);
    let go_on = f::find(view.tree(), Role::Link, b"Go on").expect("go on");
    let Some(d::Action::Pages(d::pages::Act::Go { address })) = crate::act(&mut view, go_on, Act::Press) else {
        panic!("go on opens the row's step");
    };
    assert_eq!(address, Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(FORGE)) } });
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
    let view = f::page_view(&root);
    assert_word(view.tree(), Role::Button, Word::ForgeInstall);
    assert!(f::find(view.tree(), Role::Button, b"Start over").is_none());
}
