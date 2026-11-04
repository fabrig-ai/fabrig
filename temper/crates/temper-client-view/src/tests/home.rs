use super::fixtures as f;
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, ChatsFilter};

const UNKNOWN: u32 = 99;

fn printed(view: &crate::View) -> Box<[u8]> {
    let mut print = skein_ui_text::Print::new(&view.limits().tree).expect("terminal bounds");
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    skein_lib::bytes::copy_of(print.text.as_slice())
}

fn main_nodes(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    let mut main = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Main {
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = tree.nodes().get(index..end);
        }
    }
    main.expect("main subtree")
}

fn main_find(tree: &skein_ui::Tree, role: Role, name: &[u8]) -> skein_ui::NodeId {
    let mut found = None;
    for node in main_nodes(tree) {
        if node.role == role && node.name.as_deref() == Some(name) {
            found = Some(node.id);
        }
    }
    found.expect("main control")
}

#[test]
fn project_pages_draw_their_heading_in_both_shells() {
    for (address, heading) in [
        (Address::Home { project: f::PROJECT }, b"Shop".as_slice()),
        (Address::Settings { project: f::PROJECT }, b"Settings for Shop".as_slice()),
    ] {
        let (root, _, _) = f::root_at(address, &f::domain_limits());
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let main = main_nodes(tree);
            assert!(main.first().expect("main").styles.as_slice().contains(&crate::look::MAIN));
            let _heading = main_find(tree, Role::Heading { level: 1 }, heading);
            assert!(skein_lib::bytes::find(&f::main_text(tree), b"not one of yours").is_none());
        }
        let view = f::page_view(&root);
        let printed = printed(&view);
        assert!(skein_lib::bytes::find(&printed, heading).is_some());
    }
}

#[test]
fn unlisted_projects_explain_and_link_to_chats() {
    for address in [Address::Home { project: UNKNOWN }, Address::Settings { project: UNKNOWN }] {
        let (root, _, _) = f::root_at(address, &f::domain_limits());
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let main = f::main_text(view.tree());
            assert!(skein_lib::bytes::find(&main, b"Project 99 is not one of yours.").is_some());
            let link = main_find(view.tree(), Role::Link, b"Chats");
            assert_eq!(
                crate::act(&mut view, link, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go {
                    address: Address::Chats { filter: ChatsFilter::All, start: false },
                }))
            );
        }
        let view = f::page_view(&root);
        assert!(skein_lib::bytes::find(&printed(&view), b"Project 99 is not one of yours.").is_some());
    }
}

fn settings_root() -> d::Domain {
    f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits()).0
}

#[test]
fn settings_lists_links_then_sign_out_in_both_shells() {
    let root = settings_root();
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let nodes = main_nodes(view.tree());
        assert_eq!(nodes[0].styles.as_slice(), &[crate::look::MAIN, crate::look::SETTINGS]);
        assert_eq!(crate::look::NAMES.style(crate::look::SETTINGS), Some(b"temper-settings".as_slice()));
        let mut at = 1;
        let mut list = None;
        for expected in [Role::Heading { level: 1 }, Role::List, Role::Button] {
            let node = &nodes[at];
            assert_eq!(node.role, expected);
            if expected == Role::List {
                list = Some(subtree(view.tree(), node.id));
            }
            at += usize::try_from(node.size).unwrap();
        }
        assert_eq!(at, nodes.len());
        let list = list.unwrap();
        let mut at = 1;
        for name in [b"Forges".as_slice(), b"Accounts", b"Adopt a repository"] {
            assert_eq!(list[at].role, Role::Item);
            assert_eq!(list[at + 1].role, Role::Link);
            assert_eq!(list[at + 1].name.as_deref(), Some(name));
            at += usize::try_from(list[at].size).unwrap();
        }
        assert_eq!(at, list.len());
        for (name, address, href) in [
            (b"Forges".as_slice(), Address::Forges, b"/forges".as_slice()),
            (b"Accounts", Address::Accounts, b"/accounts"),
            (b"Adopt a repository", Address::Adopt { project: f::PROJECT }, b"/projects/1/adopt"),
        ] {
            let link = main_find(view.tree(), Role::Link, name);
            assert!(!subtree(view.tree(), link)[0].states.as_slice().contains(&skein_ui::State::Current));
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(href));
            assert_eq!(crate::act(&mut view, link, Act::Press), Some(d::Action::Pages(d::pages::Act::Go { address })));
        }
        let button = main_find(view.tree(), Role::Button, b"Sign out");
        assert_eq!(crate::act(&mut view, button, Act::Press), Some(d::Action::Pages(d::pages::Act::SignOut)));
    }
}

#[test]
fn settings_links_carry_the_forge_and_account_marks_in_both_shells() {
    use jig_client::contract::AccountState;
    use temper_client_protocol_wire::v1 as w;
    for (connection, account, style, forge_words, account_words) in [
        (
            w::Connection::Connecting,
            AccountState::Later { until: Some(f::NOW) },
            crate::look::NAV_LIMITED,
            b"a forge is setting up".as_slice(),
            b"an account is limited".as_slice(),
        ),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            AccountState::SignInAgain,
            crate::look::NAV_NEEDS,
            b"a forge needs you",
            b"an account needs you",
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
            for (name, expected) in [(b"Forges".as_slice(), forge_words), (b"Accounts", account_words)] {
                let link = main_find(view.tree(), Role::Link, name);
                let mut marks = 0_u32;
                for node in subtree(view.tree(), link) {
                    if node.styles.as_slice().contains(&style) {
                        marks += 1;
                        assert_eq!(node.role, Role::Text);
                        assert_eq!(node.name.as_deref(), Some(expected));
                        assert_eq!(f::nodes_text(core::slice::from_ref(node)).as_ref(), expected);
                    }
                }
                assert_eq!(marks, 1, "one mark on each settings link");
            }
        }
        let text = printed(&f::page_view(&root));
        for expected in [forge_words, account_words] {
            assert!(skein_lib::bytes::find(&text, expected).is_some());
        }
    }
}

#[test]
fn settings_terminal_numbers_each_link_and_sign_out() {
    let root = settings_root();
    let mut view = f::page_view(&root);
    for (role, name, action) in [
        (Role::Link, b"Forges".as_slice(), d::pages::Act::Go { address: Address::Forges }),
        (Role::Link, b"Accounts", d::pages::Act::Go { address: Address::Accounts }),
        (Role::Link, b"Adopt a repository", d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } }),
        (Role::Button, b"Sign out", d::pages::Act::SignOut),
    ] {
        let control = main_find(view.tree(), role, name);
        let (node, act) = f::terminal_command(view.tree(), &view.limits().tree, control, name, b"Settings for Shop");
        assert_eq!(crate::act(&mut view, node, act), Some(d::Action::Pages(action)));
    }
}

#[test]
fn settings_lost_sign_out_alert_offers_sign_out_again() {
    let mut root = settings_root();
    f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SignOut) });
    f::root_event(&mut root, d::Event::Client { event: jig_client::Event::SignOutLost });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let _alert = main_find(view.tree(), Role::Alert, b"Sign out did not reach the engine.");
        let button = main_find(view.tree(), Role::Button, b"Sign out");
        assert_eq!(crate::act(&mut view, button, Act::Press), Some(d::Action::Pages(d::pages::Act::SignOut)));
        assert!(skein_lib::bytes::find(&printed(&view), b"Sign out did not reach the engine.").is_some());
    }
}

fn subtree(tree: &skein_ui::Tree, id: skein_ui::NodeId) -> &[skein_ui::Node] {
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.id == id {
            let end = at.checked_add(usize::try_from(node.size).expect("subtree size")).expect("subtree end");
            return &tree.nodes()[at..end];
        }
    }
    panic!("subtree root exists")
}
