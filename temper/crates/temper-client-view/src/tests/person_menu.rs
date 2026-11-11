//! The rail's person and sign out (temper/docs/design/client/pages/README.md and temper/docs/design/client/view.md, 3).
#![expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "ordinary tests inspect semantic subtrees")]
use super::fixtures as f;
use crate::look;
use skein_ui::{Act, Node, Role, Tree};
use temper_client_domain::{self as d, Address, pages};

fn root() -> d::Domain {
    f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits()).0
}
fn styled(tree: &Tree, style: skein_ui::Style) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
    let count = usize::try_from(tree.nodes().get(at).unwrap().size).unwrap();
    tree.nodes().get(at..at.checked_add(count).unwrap()).unwrap()
}
fn named<'a>(nodes: &'a [Node], role: Role, name: &[u8]) -> &'a Node {
    nodes.iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).unwrap()
}
#[test]
fn the_rails_person_is_a_menu_in_a_browser() {
    let view = f::browser_page_view(&root());
    let foot = styled(view.tree(), look::RAIL_FOOT);
    let menu = named(foot, Role::Menu, b"Pat");
    let person = styled(view.tree(), look::RAIL_PERSON);
    let slot = person.first().unwrap();
    assert_eq!(slot.role, Role::Slot);
    assert_eq!(slot.size, menu.size.checked_add(1).unwrap());
    assert_eq!(person.get(1).unwrap().id, menu.id);
    assert_eq!(
        person.iter().filter(|n| n.role == Role::MenuItem).map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
        [b"Accounts".as_slice(), b"What temper remembers about you", b"Telling you", b"Sign out"]
    );
    assert!(!foot.iter().any(|n| n.role == Role::Status && n.name.as_deref() == Some(b"Pat")));
}
#[test]
fn the_person_menus_items_go_to_accounts_and_sign_out() {
    let mut view = f::browser_page_view(&root());
    for (name, act) in [
        (b"Accounts".as_slice(), pages::Act::Go { address: Address::Accounts }),
        (b"What temper remembers about you", pages::Act::Go { address: Address::Remembered }),
        (b"Telling you", pages::Act::Go { address: Address::Told }),
        (b"Sign out", pages::Act::SignOut),
    ] {
        let id = f::find(view.tree(), Role::MenuItem, name).unwrap();
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(act)));
    }
}
#[test]
fn the_terminal_keeps_the_persons_line_and_has_no_menu() {
    let view = f::page_view(&root());
    let foot = styled(view.tree(), look::RAIL_FOOT);
    assert_eq!(foot.iter().rev().find(|n| n.name.is_some()).unwrap().id, named(foot, Role::Status, b"Pat").id);
    assert!(f::find(view.tree(), Role::Menu, b"Pat").is_none());
    assert!(f::find(view.tree(), Role::MenuItem, b"Sign out").is_none());
}
#[test]
fn a_lost_sign_out_keeps_its_line_after_the_menu_which_still_offers_it() {
    let mut root = root();
    f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(pages::Act::SignOut) });
    f::root_event(&mut root, d::Event::Client { event: jig_client::Event::SignOutLost });
    let mut view = f::browser_page_view(&root);
    let foot = styled(view.tree(), look::RAIL_FOOT);
    let menu = named(foot, Role::Menu, b"Pat");
    let after =
        foot.iter().position(|n| n.id == menu.id).unwrap().checked_add(usize::try_from(menu.size).unwrap()).unwrap();
    assert_eq!(foot.get(after).unwrap().role, Role::Alert);
    assert_eq!(
        foot.iter().rev().find(|n| n.name.is_some()).unwrap().name.as_deref(),
        Some(b"Sign out did not reach the engine.".as_slice())
    );
    let id = f::find(view.tree(), Role::MenuItem, b"Sign out").unwrap();
    assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::SignOut)));
}
#[test]
fn the_accounts_tabs_sign_out_is_marked_for_the_sheet() {
    let root = f::root_at(Address::Accounts, &f::domain_limits()).0;
    for mut view in [f::browser_page_view(&root), f::page_view(&root)] {
        let form = named(view.tree().nodes(), Role::Form, b"Sign out");
        assert!(form.styles.as_slice().contains(&look::SETTINGS_SETTING));
        assert!(form.styles.as_slice().contains(&look::SETTINGS_SIGN_OUT));
        let row = styled(view.tree(), look::SETTINGS_SIGN_OUT);
        let link = named(row, Role::Link, b"What temper remembers about you").id;
        let told = named(row, Role::Link, b"Telling you").id;
        assert_eq!(crate::act(&mut view, told, Act::Press), Some(d::Action::Pages(pages::Act::Go { address: Address::Told })));
        let row = styled(view.tree(), look::SETTINGS_SIGN_OUT);
        let sign_out = named(row, Role::Button, b"Sign out").id;
        assert!(row.iter().position(|n| n.id == link).unwrap() < row.iter().position(|n| n.id == sign_out).unwrap());
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Remembered }))
        );
    }
}
#[test]
fn a_project_less_accounts_page_keeps_the_top_bars_sign_out() {
    let mut root = f::no_project_root();
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    let mut view = f::browser_page_view(&root);
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::RAIL)));
    let banner = styled(view.tree(), look::TOP_END);
    let id = named(banner, Role::Button, b"Sign out").id;
    assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::SignOut)));
    assert!(f::find(view.tree(), Role::MenuItem, b"Sign out").is_none());
    assert!(f::find(view.tree(), Role::Menu, b"Pat").is_none());
}
