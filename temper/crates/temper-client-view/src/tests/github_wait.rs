//! GitHub's note uses the sign-in's drawing and waits on the forge's watch.
#![expect(clippy::disallowed_methods, reason = "test assertions inspect semantic trees")]
use super::{connect_forge as kit, fixtures as f};
use crate::{View, look};
use skein_lib::{List, Queue, Token};
use skein_ui::{Act, Role, Run, Tree};
use temper_client_domain::{self as d, Address, Choice, Step};
use temper_client_protocol_wire::v1 as w;
const LINK: &[u8] = b"http://127.0.0.1:4100/forges/3/github/manifest?state=S";
const PATH: &[u8] = b"/forges/3/github/manifest?state=S";
const CONNECT_BODY: &str =
    "Make temper’s App there, then choose the account and the repositories it may use. This page moves on by itself.";
const INSTALL_BODY: &str =
    "Choose the account and the repositories temper’s App may use there. This page moves on by itself.";
fn welcome() -> Address {
    Address::Welcome { step: Step::Forge { forge: None } }
}
fn press(root: &mut d::Domain, view: &View, label: &[u8]) -> Queue<d::Request> {
    let id = kit::node(view.tree(), Role::Button, label);
    kit::drive(root, crate::event(view, id, Act::Press).unwrap())
}
fn answered(root: &mut d::Domain, attempt: Token, address: &[u8]) {
    kit::answer(root, attempt, w::Outcome::Link(w::Link { address: address.into() }));
}
fn started(address: Address, rows: List<w::ForgeRow>, button: &[u8], refused: bool, link: &[u8]) -> d::Domain {
    let mut root = f::forge_root(address, rows);
    let view = f::browser_page_view(&root);
    let attempt = kit::attempt(press(&mut root, &view, button));
    if refused {
        kit::drive(&mut root, d::Event::TabRefused);
    }
    answered(&mut root, attempt, link);
    root
}
fn styled(tree: &Tree, style: skein_ui::Style) -> usize {
    tree.nodes().iter().position(|node| node.styles.as_slice().contains(&style)).expect("note's style")
}
fn no_paste(tree: &Tree) {
    assert!(!f::text(tree).windows(b"Signed in, but".len()).any(|part| part == b"Signed in, but"));
    assert!(
        !tree.nodes().iter().any(|node| node.name.as_deref().is_some_and(|name| name.starts_with(b"Signed in, but")))
    );
    assert!(f::find(tree, Role::TextField, "Address of the page that couldn’t be reached".as_bytes()).is_none());
}
fn strong_status(tree: &Tree, title: &[u8]) {
    let id = kit::node(tree, Role::Status, title);
    assert!(tree.nodes().iter().any(|node| kit::parent(tree, node.id) == Some(id)
        && node.text.iter().any(|run| run.run == Run::Strong && run.text.as_ref() == title)));
}
#[test]
fn welcome_waits_above_its_foot_and_reopens_github_on_its_origin_without_sending() {
    let mut root = f::forge_root(welcome(), List::with_capacity(0));
    let mut view = f::browser_page_view(&root);
    let before = f::node_text(view.tree(), styled(view.tree(), look::WELCOME_FOOT));
    let attempt = kit::attempt(press(&mut root, &view, b"Connect GitHub"));
    answered(&mut root, attempt, LINK);
    f::build(&mut view, &root);
    let tree = view.tree();
    strong_status(tree, b"Waiting for GitHub, in the new tab");
    let away = styled(tree, look::WELCOME_AWAY);
    let spin = styled(tree, look::WELCOME_SPIN);
    let foot = styled(tree, look::WELCOME_FOOT);
    assert!(away < spin && spin < foot);
    assert_eq!(f::node_text(tree, foot), before);
    let body = kit::node(tree, Role::Paragraph, CONNECT_BODY.as_bytes());
    assert!(tree.find(body).unwrap().styles.as_slice().contains(&look::FLOW_MUTED));
    let again = kit::node(tree, Role::Link, b"Open it again");
    assert!(tree.find(again).unwrap().styles.as_slice().contains(&look::WELCOME_AGAIN));
    assert!(f::text(tree).windows("Open it again ›".len()).any(|part| part == "Open it again ›".as_bytes()));
    assert!(f::find(tree, Role::Button, b"Connect GitHub").is_none());
    no_paste(tree);
    assert_eq!(crate::href(&mut view, again).as_deref(), Some(PATH));
    let mut out = kit::drive(&mut root, crate::event(&view, again, Act::Press).unwrap());
    assert_eq!(out.pop(), Some(d::Request::Tab { tab: d::Tab::Open { opening: d::Opening::GitHub } }));
    assert_eq!(out.pop(), Some(d::Request::Tab { tab: d::Tab::PointOnOrigin { to: LINK.into() } }));
    assert!(out.is_empty(), "reopening sends no connect or install");
    assert_eq!(root.pages().address(), &welcome());
}
#[test]
fn settings_and_row_wait_in_connection_form_and_installations_use_their_own_words() {
    for (address, rows, label, welcome, install) in [
        (Address::ConnectForge { forge: None }, List::with_capacity(0), b"Connect GitHub".as_slice(), false, false),
        (Address::Forges, kit::github_rows(true), b"Install on another account", false, true),
        (Address::Forges, kit::github_rows(true), b"Choose its repositories on GitHub", false, true),
        (
            Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(kit::GITHUB)) } },
            kit::github_rows(false),
            b"Install on GitHub",
            true,
            true,
        ),
    ] {
        let link = if install { b"https://github.example/install".as_slice() } else { LINK };
        let root = started(address, rows, label, false, link);
        let mut view = f::browser_page_view(&root);
        let tree = view.tree();
        strong_status(tree, b"Waiting for GitHub, in the new tab");
        assert!(
            f::text(tree)
                .windows(if install { INSTALL_BODY.len() } else { CONNECT_BODY.len() })
                .any(|part| part == if install { INSTALL_BODY.as_bytes() } else { CONNECT_BODY.as_bytes() })
        );
        let again = kit::node(tree, Role::Link, b"Open it again");
        assert!(tree.find(again).unwrap().styles.as_slice().contains(&if welcome {
            look::WELCOME_AGAIN
        } else {
            look::CONNECT_SECONDARY
        }));
        if !welcome {
            styled(tree, look::CONNECT_WAITING);
            styled(tree, look::CONNECT_ACTS);
        }
        assert!(f::find(tree, Role::Button, label).is_none());
        no_paste(tree);
        assert_eq!(crate::href(&mut view, again).as_deref(), Some(if install { link } else { PATH }));
    }
}
#[test]
fn refused_tabs_terminal_and_bad_manifest_paths_show_the_own_link_form() {
    for (refused, terminal, link) in [
        (true, false, LINK),
        (false, true, LINK),
        (false, false, b"http://127.0.0.1:4100".as_slice()),
        (false, false, b"http://127.0.0.1:4100//another.example/path"),
        (false, false, b"http://127.0.0.1:4100/path\\not-allowed"),
    ] {
        let root = started(welcome(), List::with_capacity(0), b"Connect GitHub", refused, link);
        let mut view = if terminal { f::page_view(&root) } else { f::browser_page_view(&root) };
        let tree = view.tree();
        strong_status(tree, b"Waiting for GitHub");
        let open = tree
            .nodes()
            .iter()
            .enumerate()
            .find(|(index, node)| {
                node.role == Role::Link && f::node_text(tree, *index).as_ref() == "Open GitHub ›".as_bytes()
            })
            .unwrap_or_else(|| panic!("own GitHub link: {link:?}, terminal={terminal}"))
            .1
            .id;
        if !terminal {
            assert!(tree.find(open).unwrap().styles.as_slice().contains(&look::FLOW_PRIMARY));
        }
        assert!(f::find(tree, Role::Status, b"Waiting for GitHub, in the new tab").is_none());
        if terminal {
            assert!(!tree.nodes().iter().any(|node| node.styles.as_slice().contains(&look::WELCOME_SPIN)));
        }
        no_paste(tree);
        let path = d::forge::github::origin_path(link);
        assert_eq!(crate::href(&mut view, open).as_deref(), Some(if terminal { link } else { path.unwrap_or(link) }));
    }
}
fn app_rows(app: Option<w::AppWay>) -> List<w::ForgeRow> {
    let mut rows = kit::github_rows(false);
    for row in rows.iter_mut() {
        row.app = app.clone();
        row.ended = None;
    }
    rows
}
fn control(view: &View, role: Role, text: &[u8]) -> skein_ui::NodeId {
    view.tree()
        .nodes()
        .iter()
        .enumerate()
        .find(|(index, node)| node.role == role && f::node_text(view.tree(), *index).as_ref() == text)
        .expect("the note's control")
        .1
        .id
}
#[test]
fn the_made_apps_reopen_is_an_install_in_both_forms_and_shells() {
    for welcome_form in [true, false] {
        for (refused, terminal) in [(false, false), (true, false), (false, true)] {
            let address = if welcome_form { welcome() } else { Address::ConnectForge { forge: None } };
            let (mut root, watch) = f::forge_root_watch(address, List::with_capacity(0));
            let mut view = if terminal { f::page_view(&root) } else { f::browser_page_view(&root) };
            let attempt = kit::attempt(press(&mut root, &view, b"Connect GitHub"));
            if refused {
                kit::drive(&mut root, d::Event::TabRefused);
            }
            answered(&mut root, attempt, LINK);
            kit::deliver(&mut root, watch, app_rows(None));
            f::build(&mut view, &root);
            let own = refused || terminal;
            let text = if own { "Open GitHub ›" } else { "Open it again ›" }.as_bytes();
            let name = if own { text } else { b"Open it again" };
            let title = if own { b"Waiting for GitHub".as_slice() } else { b"Waiting for GitHub, in the new tab" };
            let link = control(&view, Role::Link, text);
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(if terminal { LINK } else { PATH }));
            strong_status(view.tree(), title);
            kit::words_in_order(view.tree(), &[CONNECT_BODY]);

            let out = kit::deliver(&mut root, watch, app_rows(Some(w::AppWay::Manifest)));
            for request in &out {
                if let d::Request::Tab { .. } = request {
                    panic!("the App appearing sends no departure");
                }
            }
            f::build(&mut view, &root);
            strong_status(view.tree(), title);
            kit::words_in_order(view.tree(), &[CONNECT_BODY]);
            let again = control(&view, Role::Button, text);
            assert_eq!(view.tree().find(again).unwrap().name.as_deref(), Some(name));
            if !terminal {
                let style = if welcome_form {
                    if refused { look::FLOW_PRIMARY } else { look::WELCOME_AGAIN }
                } else if refused {
                    look::CONNECT_PRIMARY
                } else {
                    look::CONNECT_SECONDARY
                };
                assert!(view.tree().find(again).unwrap().styles.as_slice().contains(&style));
            }
            assert!(f::find(view.tree(), Role::Link, name).is_none());
            assert_eq!(crate::href(&mut view, again), None);
            assert_eq!(
                crate::act(&mut view, again, Act::Press),
                Some(d::Action::Forge(d::forge::Act::Install { forge: kit::GITHUB }))
            );
            no_paste(view.tree());
            let mut out = press(&mut root, &view, name);
            assert_eq!(out.pop(), Some(d::Request::Tab { tab: d::Tab::Open { opening: d::Opening::GitHub } }));
            let attempt = kit::attempt(out);
            f::build(&mut view, &root);
            kit::words_in_order(view.tree(), &[INSTALL_BODY]);
            assert!(!f::text(view.tree()).windows(CONNECT_BODY.len()).any(|part| part == CONNECT_BODY.as_bytes()));
            answered(&mut root, attempt, b"https://github.example/install");
            f::build(&mut view, &root);
            kit::words_in_order(view.tree(), &[INSTALL_BODY]);
            let text = if terminal { "Open GitHub ›" } else { "Open it again ›" }.as_bytes();
            let link = control(&view, Role::Link, text);
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"https://github.example/install".as_slice()));
        }
    }
}
#[test]
fn github_connecting_shows_the_last_connects_line() {
    let root = f::forge_root(
        Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(kit::GITHUB)) } },
        kit::github_rows(false),
    );
    let view = f::browser_page_view(&root);
    assert!(
        f::text(view.tree())
            .windows(b"The last connect stopped: fixture stopped".len())
            .any(|part| part == b"The last connect stopped: fixture stopped")
    );
    kit::node(view.tree(), Role::Button, b"Install on GitHub");
}
#[test]
fn github_cut_short_shows_the_last_connects_line_under_its_words() {
    let root = f::forge_root(welcome(), kit::github_rows(false));
    let view = f::browser_page_view(&root);
    kit::words_in_order(
        view.tree(),
        &[
            "FORGE · not finished",
            "The GitHub App stays yours",
            "The last connect stopped: fixture stopped",
            "Start over",
        ],
    );
}
