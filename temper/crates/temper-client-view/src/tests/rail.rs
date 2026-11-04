//! The project's frame (temper/docs/design/client/README.md, 5.1 and 5.3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests compare semantic subtrees and terminal text"
)]
use super::fixtures as f;
use crate::look;
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, ChatsFilter, Event, pages};

fn styled(tree: &Tree, style: Style) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).expect("styled node");
    let end = at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}
fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut out = Vec::new();
    let mut at = 1;
    while at < nodes.len() {
        out.push(&nodes[at]);
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    out
}

#[test]
fn the_frame_requires_me_and_a_project_and_yields_to_welcome_and_sign_in() {
    let root = f::root_in_project(None);
    assert_eq!(crate::frame::rail_project(&root, &f::words(), &crate::sign_in::Ways::NONE), Some(f::PROJECT));
    for root in f::pair(f::signed_out(), f::out_of_date()) {
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(view.tree().nodes().iter().any(|n| n.role == Role::Banner));
            assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::RAIL)));
        }
    }
    let view = f::built();
    assert!(view.tree().nodes().iter().any(|n| n.role == Role::Banner));
    let root = f::welcome_root(d::Step::Ready, false);
    assert_eq!(crate::frame::rail_project(&root, &f::words(), &crate::sign_in::Ways::NONE), None);
    assert!(f::page_view(&root).tree().nodes().iter().any(|n| n.role == Role::Banner));
    let root = f::no_project_root();
    assert!(root.pages().me(root.client()).is_some());
    assert_eq!(root.pages().project(), None);
    assert!(f::page_view(&root).tree().nodes().iter().any(|n| n.role == Role::Banner));
    let root = f::signed_out();
    let view = f::sign_in_view(
        &root,
        crate::words::Shell::Browser,
        crate::sign_in::Ways::parse("google", b"https://temper.example"),
    );
    assert!(view.tree().nodes().iter().any(|n| n.role == Role::Banner));
    let mut root = f::root_in_project(None);
    f::root_event(&mut root, Event::Went { place: d::Place::Authorize { query: Box::from(b"state=test".as_slice()) } });
    assert!(root.authorization().is_some());
    assert_eq!(crate::frame::rail_project(&root, &f::browser_words(), &crate::sign_in::Ways::NONE), None);
    assert!(f::browser_page_view(&root).tree().nodes().iter().any(|n| n.role == Role::Banner));
}

#[test]
fn rail_and_tabbar_keep_the_order_addresses_current_states_and_count() {
    let addresses = [
        Address::Home { project: f::PROJECT },
        Address::Inbox { filter: d::address::Filter::ALL },
        Address::Chats { filter: ChatsFilter::All, start: false },
        Address::Chat { task: 9 },
        Address::Task { task: 9, filter: d::PlanFilter::All },
        Address::Accounts,
        Address::ConnectModel { way: None },
        Address::Forges,
        Address::ConnectForge { forge: None },
        Address::Adopt { project: f::PROJECT },
        Address::Settings { project: f::PROJECT },
    ];
    for address in addresses {
        let (mut root, watch, _) = f::root_at(address.clone(), &f::domain_limits());
        for count in [0, 3] {
            let mut me = root.pages().me(root.client()).unwrap().clone();
            me.needs_total = count;
            f::deliver_me(&mut root, watch, me);
            for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
                assert_eq!(skein_ui::outline::check(view.tree()), None, "{address:?}");
                assert!(!view.tree().nodes().iter().any(|n| n.role == Role::Banner));
                let rail = styled(view.tree(), look::RAIL);
                assert_eq!(rail[0].role, Role::Navigation);
                assert_eq!(rail[0].name.as_deref(), Some(b"Shop".as_slice()));
                let parts = children(rail);
                assert_eq!(
                    parts.iter().map(|n| n.role).collect::<Vec<_>>(),
                    [Role::Link, Role::Disclosure, Role::Slot, Role::Link, Role::Slot]
                );
                assert_eq!(parts[0].name.as_deref(), Some(b"temper home".as_slice()));
                assert_eq!(parts[1].styles.as_slice(), [look::PROJECTS]);
                assert_eq!(parts[2].styles.as_slice(), [look::RAIL_NAV]);
                assert_eq!(parts[3].styles.as_slice(), [look::RAIL_NEW]);
                assert_eq!(parts[4].styles.as_slice(), [look::RAIL_FOOT]);
                let brand = parts[0].id;
                let new_chat = parts[3].id;
                assert_eq!(crate::href(&mut view, brand).as_deref(), Some(b"/projects/1".as_slice()));
                assert_eq!(crate::href(&mut view, new_chat).as_deref(), Some(b"/chats?at=start".as_slice()));
                let needs = if count == 0 { b"Needs you".as_slice() } else { b"Needs you 3" };
                let current = match address {
                    Address::Home { .. } => Some(b"Home".as_slice()),
                    Address::Inbox { .. } => Some(needs),
                    Address::Chats { .. } | Address::Chat { .. } => Some(b"Chats".as_slice()),
                    Address::Settings { .. } => Some(b"Settings".as_slice()),
                    Address::Task { .. }
                    | Address::Accounts
                    | Address::ConnectModel { .. }
                    | Address::Forges
                    | Address::ConnectForge { .. }
                    | Address::Adopt { .. }
                    | Address::Welcome { .. } => None,
                };
                let mut styles = vec![look::RAIL_NAV];
                let tab = view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR));
                if tab {
                    styles.push(look::TABBAR);
                    let nodes = view.tree().nodes();
                    let at = nodes.iter().position(|n| n.styles.as_slice().contains(&look::TABBAR)).unwrap();
                    let main = nodes.iter().position(|n| n.role == Role::Main).unwrap();
                    assert_eq!(at, main + usize::try_from(nodes[main].size).unwrap());
                    assert_eq!(nodes[at].role, Role::Navigation);
                    assert_eq!(nodes[at].name.as_deref(), Some(b"Sections".as_slice()));
                }
                for style in styles {
                    let links = children(styled(view.tree(), style));
                    let expected = if style == look::TABBAR {
                        vec![b"Home".as_slice(), needs, b"Chats", b"Settings"]
                    } else {
                        vec![b"Home".as_slice(), needs, b"Chats"]
                    };
                    assert_eq!(links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(), expected);
                    let ids = links
                        .iter()
                        .map(|n| (n.id, n.name.clone(), n.states.as_slice().contains(&State::Current)))
                        .collect::<Vec<_>>();
                    for ((id, name, marked), href) in
                        ids.into_iter().zip([b"/projects/1".as_slice(), b"/", b"/chats", b"/projects/1/settings"])
                    {
                        assert_eq!(marked, name.as_deref() == current && current.is_some());
                        assert_eq!(crate::href(&mut view, id).as_deref(), Some(href));
                    }
                }
                if let Address::Inbox { .. } = address {
                    assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Inbox").is_some());
                }
                assert_eq!(
                    f::find(view.tree(), Role::Button, b"Sign out").is_some(),
                    address == Address::Settings { project: f::PROJECT }
                );
            }
        }
    }
}

#[test]
fn the_foot_keeps_status_identity_and_lost_sign_out_and_places_behind_for_each_shell() {
    for state in [jig_client::Link::Live, jig_client::Link::Offline] {
        let (mut root, watch) = f::root_with_watch(None);
        if state == jig_client::Link::Offline {
            f::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
                    },
                },
            );
        }
        for browser in [false, true] {
            let view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let foot = children(styled(view.tree(), look::RAIL_FOOT));
            assert_eq!(foot[0].region, Some(crate::frame::link_region()));
            let expected = match state {
                jig_client::Link::Starting => b"Connecting".as_slice(),
                jig_client::Link::Offline => b"Offline",
                jig_client::Link::Live => b"Live",
                jig_client::Link::SignedOut => b"Signed out",
                jig_client::Link::OutOfDate => b"Out of date",
            };
            assert!(foot.iter().any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Settings")));
            assert_eq!(foot.last().unwrap().name.as_deref(), Some(b"Pat".as_slice()));
            let region = foot[0];
            let status = f::find(view.tree(), Role::Status, expected).unwrap();
            assert!(
                view.tree()
                    .nodes()
                    .iter()
                    .skip_while(|n| n.id != region.id)
                    .take(usize::try_from(region.size).unwrap())
                    .any(|n| n.id == status)
            );
            assert_eq!(view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)), browser);
        }
        let (mut root, _, page) = f::root_at(Address::Inbox { filter: d::address::Filter::ALL }, &f::domain_limits());
        f::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::WatchEnded {
                        watch: page.unwrap(),
                        end: jig_client::contract::WatchEnd::Dropped,
                    },
                },
            },
        );
        for browser in [false, true] {
            let view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let foot = children(styled(view.tree(), look::RAIL_FOOT));
            let region = foot[0];
            let behind = f::find(view.tree(), Role::Status, b"Behind").unwrap();
            let inside = view
                .tree()
                .nodes()
                .iter()
                .skip_while(|n| n.id != region.id)
                .take(usize::try_from(region.size).unwrap())
                .any(|n| n.id == behind);
            assert_eq!(inside, !browser);
            if browser {
                assert_eq!(foot[1].id, behind);
            }
        }
        f::root_event(&mut root, Event::Act { action: d::Action::Pages(pages::Act::SignOut) });
        f::root_event(&mut root, Event::Client { event: jig_client::Event::SignOutLost });
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(
                children(styled(view.tree(), look::RAIL_FOOT)).last().unwrap().name.as_deref(),
                Some(b"Sign out did not reach the engine.".as_slice())
            );
            assert!(f::find(view.tree(), Role::Button, b"Sign out").is_none());
        }
    }
}

#[test]
fn settings_carries_both_forge_and_account_marks() {
    use jig_client::contract::AccountState;
    use temper_client_protocol_wire::v1 as w;
    for (connection, account, style, forge_words, account_words) in [
        (
            w::Connection::Connecting,
            AccountState::Later { until: None },
            look::NAV_LIMITED,
            b"a forge is setting up".as_slice(),
            b"an account is limited".as_slice(),
        ),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            AccountState::SignInAgain,
            look::NAV_NEEDS,
            b"a forge needs you".as_slice(),
            b"an account needs you".as_slice(),
        ),
    ] {
        let (mut root, watch) = f::forge_root_accounts(
            Address::Settings { project: f::PROJECT },
            super::connect_forge::rows(connection, w::Offer::Offered),
        );
        let mut accounts = super::accounts::document(1);
        accounts.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = account;
        super::accounts::deliver(&mut root, watch, accounts);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let foot = styled(view.tree(), look::RAIL_FOOT);
            let link = children(foot).into_iter().find(|n| n.role == Role::Link).unwrap();
            assert_eq!(link.name.as_deref(), Some(b"Settings".as_slice()));
            assert!(link.states.as_slice().contains(&State::Current));
            let marks = foot.iter().filter(|n| n.styles.as_slice().contains(&style)).collect::<Vec<_>>();
            assert_eq!(marks.len(), 2);
            assert_eq!(marks[0].name.as_deref(), Some(forge_words));
            assert_eq!(marks[1].name.as_deref(), Some(account_words));
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
            let text = String::from_utf8_lossy(print.text.as_slice());
            let expected = format!(
                "Settings · {} · {}",
                core::str::from_utf8(forge_words).unwrap(),
                core::str::from_utf8(account_words).unwrap()
            );
            if !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)) {
                assert!(text.contains(&expected), "{text}");
            }
        }
    }
}

#[test]
fn the_playground_setup_link_leads_the_foot_and_leaves() {
    let mut root = f::playground_root(Address::Forges, f::PROJECT, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let foot = children(styled(view.tree(), look::RAIL_FOOT));
        assert_eq!(foot[0].name.as_deref(), Some(b"Set up temper".as_slice()));
        let id = foot[0].id;
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Forge(d::forge::Act::EndPlayground)));
    }
    f::root_event(&mut root, Event::Act { action: d::Action::Forge(d::forge::Act::EndPlayground) });
    assert!(f::find(f::page_view(&root).tree(), Role::Link, b"Set up temper").is_none());
}

#[test]
fn the_terminal_numbers_the_rail_and_keeps_the_first_line() {
    let (mut root, watch) = f::root_with_watch(None);
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    let view = f::page_view(&root);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
    let text = String::from_utf8_lossy(print.text.as_slice());
    assert_eq!(text.lines().next(), Some("== Inbox == Offline; Behind"));
    for line in ["[6] Home", "[7] Needs you (current)", "[8] Chats", "[9] + New chat", "[10] Settings"] {
        assert!(text.lines().any(|s| s.trim() == line), "{line}: {text}");
    }
    assert!(!text.contains("Sections:"));
    assert!(!text.contains("Sign out"));
}
