//! The new chat's session layout and start (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic subtrees and terminal text"
)]
use super::fixtures as f;

use crate::look;

use jig_client::contract as c;

use skein_lib::{Env, List, Queue, Time};

use skein_ui::{Act, Node, Role, State, Style, Tree};

use temper_client_domain::{self as d, Address, pages};

const ENTRY: &[u8] = b"What do you want done?";

const DRAFT: &[u8] = b"Make the change";

fn root() -> d::Domain {
    f::playground_root(Address::NewChat { draft: false }, f::PROJECT, true)
}

fn input(root: &mut d::Domain, act: pages::Act) {
    f::root_event(root, d::Event::Act { action: d::Action::Pages(act) });
}

fn named<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a Node {
    tree.nodes().iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).expect("named node")
}

fn styled(tree: &Tree, style: Style) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).expect("styled node");
    let end = at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}

fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut out = Vec::new();
    let mut at = 1_usize;
    while at < nodes.len() {
        out.push(&nodes[at]);
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    out
}

fn parts(tree: &Tree, style: Style) -> Vec<(Role, Vec<Style>)> {
    children(styled(tree, style)).into_iter().map(|n| (n.role, n.styles.as_slice().to_vec())).collect()
}

fn disabled(node: &Node) -> bool {
    node.states.as_slice().contains(&State::Disabled)
}

#[test]
fn session_parts_follow_the_requested_order_in_both_shells() {
    let root = root();
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        assert_eq!(skein_ui::outline::check(tree), None);
        assert_eq!(styled(tree, look::SESSION)[0].styles.as_slice(), [look::MAIN, look::SESSION]);
        for (style, expected) in [
            (look::SESSION, vec![(Role::Slot, vec![look::SESSION_TOP]), (Role::Region, vec![look::CONVO])]),
            (look::SESSION_TOP, vec![(Role::Navigation, vec![look::CRUMBS]), (Role::Slot, vec![look::SESSION_RIGHT])]),
            (look::SESSION_RIGHT, vec![(Role::Group, vec![look::SEG])]),
            (
                look::CONVO,
                vec![
                    (Role::Slot, vec![look::THREAD]),
                    (Role::Slot, vec![look::CHOICE]),
                    (Role::Form, vec![look::SESSION_FOOT]),
                ],
            ),
            (look::THREAD, vec![(Role::Slot, vec![look::STARTER])]),
            (look::SESSION_FOOT, vec![(Role::Slot, vec![look::SESSION_COMPOSER]), (Role::Slot, vec![look::FOOT_ROW])]),
            (look::SESSION_COMPOSER, vec![(Role::TextArea, vec![]), (Role::Button, vec![])]),
        ] {
            assert_eq!(parts(tree, style), expected, "{style:?}");
        }
        assert_eq!(styled(tree, look::SESSION_FOOT)[0].name.as_deref(), Some(ENTRY));
        assert_eq!(
            styled(tree, look::SESSION_HINT)[0].name.as_deref(),
            Some("Enter sends · Shift+Enter for a new line · Ctrl+\\ switches the draft".as_bytes())
        );
        assert_eq!(styled(tree, look::MODEL)[0].description.as_deref(), Some(b"Scripted model".as_slice()));
        assert!(f::find(tree, Role::Heading { level: 1 }, b"New chat").is_some());
        assert!(f::find(tree, Role::Button, b"Start chat").is_none());
        assert!(f::find(tree, Role::Button, b"Start").is_none());
        assert!(f::find(tree, Role::List, b"Your chats").is_none());
        let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
        skein_ui_text::render(tree, b"temper", None, &mut print);
        let text = String::from_utf8_lossy(print.text.as_slice());
        for word in ["Where:", "New chat", "Conversation:", "What do you want done?", "Send"] {
            assert!(text.contains(word), "{word}: {text}");
        }
    }
}

#[test]
fn meta_line_uses_the_home_repository_or_the_project_name() {
    use temper_client_protocol_wire::v1 as w;
    for adopted in [false, true] {
        let mut parts = super::connect_forge::row(
            2,
            w::Kind::Forgejo,
            w::Mode::Owned,
            super::connect_forge::connected(),
            w::Offer::NotOffered,
        )
        .into_parts();
        parts.repositories = List::with_capacity(1);
        if adopted {
            parts
                .repositories
                .push(
                    w::Adoption::new(
                        &w::CEILINGS,
                        w::AdoptionParts {
                            owner: b"owner".as_slice().into(),
                            name: b"repo".as_slice().into(),
                            project: f::PROJECT,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let mut rows = List::with_capacity(1);
        rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        let root = f::forge_root(Address::NewChat { draft: false }, rows);
        let expected = if adopted {
            "Say what you want; the agent works in owner/repo."
        } else {
            "Say what you want. Shop has no repository yet, so the agent can talk and plan but not change code."
        };
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(styled(view.tree(), look::META_LINE)[0].name.as_deref(), Some(expected.as_bytes()));
        }
    }
}

#[test]
fn path_links_lead_through_chats_with_or_without_a_project() {
    for project in [None, Some(f::PROJECT)] {
        let mut root = if project.is_some() { f::root_in_project(project) } else { f::no_project_root() };
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::NewChat { draft: false }) });
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            if project.is_none() {
                assert!(f::find(tree, Role::Heading { level: 1 }, b"No project yet").is_some());
                assert!(f::find(tree, Role::Form, b"New project").is_some());
                assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::TRAIL)));
                continue;
            }
            let trail = children(styled(tree, look::TRAIL));
            assert_eq!(trail.len(), 1);
            let mut links = vec![(styled(tree, look::BACK)[0].id, Address::Chats { filter: d::ChatsFilter::All })];
            assert_eq!(styled(tree, look::BACK)[0].name.as_deref(), Some(b"Chats".as_slice()));
            assert_eq!(trail.last().unwrap().name.as_deref(), Some(b"Chats".as_slice()));
            links.push((trail.last().unwrap().id, Address::Chats { filter: d::ChatsFilter::All }));
            for (node, address) in links {
                assert_eq!(crate::href(&mut view, node).as_deref(), Some(d::address::print(&address).as_ref()));
                assert_eq!(crate::act(&mut view, node, Act::Press), Some(d::Action::Pages(pages::Act::Go { address })));
            }
        }
    }
    // With no selected project the frame offers project creation; the session's
    // own path still leads through Chats when drawn without a project.
    for words in [f::words(), f::browser_words()] {
        let root = f::no_project_root();
        let mut b = skein_ui::Builder::new(&f::tree_limits());
        let mut bindings = crate::frame::Bindings::new(f::tree_limits().bindings);
        crate::pages::session::top(
            &mut b,
            &root,
            &words,
            &mut bindings,
            None,
            crate::pages::session::Path::Chats,
            crate::pages::session::Heading::Words(b"New chat"),
        );
        b.close();
        let tree = b.finish();
        let trail = children(styled(&tree, look::TRAIL));
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].name.as_deref(), Some(b"Chats".as_slice()));
    }
}

#[test]
fn opening_focuses_the_entry_except_while_renaming() {
    let (mut root, watch, _) = f::root_at(Address::NewChat { draft: false }, &f::domain_limits());
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().role = c::HeldRole::Owner;
    f::deliver_me(&mut root, watch, me);
    for renaming in [false, true, false] {
        if renaming {
            input(&mut root, pages::Act::Switcher(pages::switcher::Act::Rename));
            assert!(root.pages().switcher().renaming().is_some());
        } else {
            input(&mut root, pages::Act::Switcher(pages::switcher::Act::Cancel));
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let entry = named(view.tree(), Role::TextArea, ENTRY);
            assert_eq!(entry.focus, if renaming { None } else { Some(0) });
            assert_eq!(entry.placeholder.as_deref(), Some(ENTRY));
        }
    }
}

#[test]
fn empty_not_ready_and_pending_disable_send_but_keep_the_draft_editable() {
    for (model, words, pending) in [(true, b"".as_slice(), false), (false, DRAFT, false), (true, DRAFT, true)] {
        let mut root = if model {
            root()
        } else {
            let mut root = super::choice::no_model_root(false);
            f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::NewChat { draft: false }) });
            root
        };
        input(&mut root, pages::Act::ChatWords { words: words.into() });
        if pending {
            input(&mut root, pages::Act::StartChat);
            assert!(root.pages().chats().unwrap().start.is_some());
        }
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let send = named(view.tree(), Role::Button, b"Send");
            assert!(disabled(send));
            let id = send.id;
            assert_eq!(crate::act(&mut view, id, Act::Press), None);
            let entry = named(view.tree(), Role::TextArea, ENTRY);
            assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), words);
            assert_eq!(entry.states.as_slice().contains(&State::SubmitsOnEnter), model && !pending);
            let id = entry.id;
            if !model || pending {
                assert_eq!(crate::act(&mut view, id, Act::Submit), None);
            }
            assert_eq!(
                crate::act(&mut view, id, Act::Input { value: DRAFT.into() }),
                Some(d::Action::Pages(pages::Act::ChatWords { words: DRAFT.into() }))
            );
            if !model {
                let nodes = view.tree().nodes();
                let line = nodes
                    .iter()
                    .enumerate()
                    .position(|(at, n)| {
                        n.styles.as_slice().contains(&look::ACCOUNT_LINE)
                            && f::node_text(view.tree(), at).as_ref() == b"No model yet \xc2\xb7 Connect a model"
                    })
                    .expect("no model line");
                let entry = nodes.iter().position(|n| n.id == id).unwrap();
                assert!(line < entry);
                let connect = named(view.tree(), Role::Link, b"Connect a model").id;
                assert_eq!(
                    crate::act(&mut view, connect, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go { address: Address::ConnectModel { way: None } }))
                );
            }
        }
    }
}

#[test]
fn input_enter_and_send_emit_the_chat_acts() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = named(view.tree(), Role::TextArea, ENTRY).id;
        assert_eq!(
            crate::act(&mut view, entry, Act::Input { value: b"Changed".as_slice().into() }),
            Some(d::Action::Pages(pages::Act::ChatWords { words: b"Changed".as_slice().into() }))
        );
        assert_eq!(crate::act(&mut view, entry, Act::Submit), Some(d::Action::Pages(pages::Act::StartChat)));
        let send = named(view.tree(), Role::Button, b"Send");
        assert!(!disabled(send));
        let id = send.id;
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::StartChat)));
    }
}

fn start(root: &mut d::Domain) -> skein_lib::Token {
    input(root, pages::Act::ChatWords { words: DRAFT.into() });
    let env = Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() };
    let mut out = Queue::uncapped();
    for event in [
        d::Event::Act { action: d::Action::Pages(pages::Act::StartChat) },
        d::Event::Client { event: jig_client::Event::Stored },
    ] {
        d::step(root, &env, event, &mut out);
    }
    for request in &out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return *attempt;
        }
    }
    panic!("start sent");
}

fn answer(root: &mut d::Domain, attempt: skein_lib::Token, answer: c::Answer) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(answer)),
                },
            },
        },
        &mut out,
    );
    out
}

#[test]
fn refused_start_keeps_the_words_and_feedback_under_the_composer() {
    let mut root = root();
    let attempt = start(&mut root);
    answer(&mut root, attempt, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    assert!(root.pages().chats().unwrap().start.is_none());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(named(view.tree(), Role::TextArea, ENTRY).value.as_ref().unwrap().text.as_ref(), DRAFT);
        assert!(!disabled(named(view.tree(), Role::Button, b"Send")));
        assert_eq!(children(styled(view.tree(), look::SESSION_FOOT)).last().unwrap().role, Role::Status);
        assert!(String::from_utf8_lossy(&f::main_text(view.tree())).contains("not offered"));
    }
}

#[test]
fn only_the_starts_address_focuses_its_composer_including_the_playground() {
    for playground in [false, true] {
        for start in [false, true] {
            let address =
                if start { Address::NewChat { draft: false } } else { Address::Chats { filter: d::ChatsFilter::All } };
            let root = if playground {
                f::playground_root(address, f::PROJECT, true)
            } else {
                f::root_at(address, &f::domain_limits()).0
            };
            let view = f::page_view(&root);
            let mut focused = 0_usize;
            for node in view.tree().nodes() {
                if node.focus.is_some() {
                    assert_eq!(node.role, Role::TextArea);
                    assert_eq!(node.name.as_deref(), Some(b"What do you want done?".as_slice()));
                    focused = focused.checked_add(1).expect("focus requests fit");
                }
            }
            assert_eq!(focused, usize::from(start), "playground={playground}, start={start}");
        }
    }
}

#[test]
fn the_placeholder_and_the_meta_line() {
    let root = f::playground_root(Address::NewChat { draft: true }, f::PROJECT, true);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        assert_eq!(skein_ui::outline::check(tree), None);
        let field = named(tree, Role::TextArea, b"What should this draft hold?");
        assert_eq!(field.placeholder.as_deref(), Some(b"What should this draft hold?".as_slice()));
        assert!(tree.nodes().iter().any(|node| node.styles.as_slice().contains(&look::MODE_SPLIT)));
        assert!(
            tree.nodes().iter().any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"untitled.md"))
        );
        assert!(tree.nodes().iter().any(|node| {
            node.text.iter().any(|run| run.text.as_ref() == b"A chat that writes a draft. Say what it should hold.")
        }));
    }
}

#[test]
fn the_bar_offers_the_modes_when_the_setting_offers_them() {
    let mut root = super::document::offered(None, false);
    for draft in [false, true] {
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::NewChat { draft }) });
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(f::find(view.tree(), Role::Group, b"The draft").is_some());
            assert!(f::find(view.tree(), Role::Button, b"+ Draft").is_none());
            assert!(!view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&look::STATUS_LINE)));
            for (name, target, pressed) in
                [(b"Chat".as_slice(), false, !draft), (b"Split", true, draft), (b"Draft", true, false)]
            {
                let button = named(view.tree(), Role::Button, name);
                assert_eq!(button.states.as_slice().contains(&State::Pressed), pressed);
                let id = button.id;
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go { address: Address::NewChat { draft: target } }))
                );
            }
        }
    }
    let plain = f::root_with_ways(
        Address::NewChat { draft: false },
        Some(d::forge::ways::Ways { drafts: false, ..d::forge::ways::Ways::STOCK }),
    );
    assert!(f::find(f::browser_page_view(&plain).tree(), Role::Group, b"The draft").is_none());
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Ask { project: f::PROJECT }) });
    assert!(f::find(f::browser_page_view(&root).tree(), Role::Group, b"The draft").is_none());
}
