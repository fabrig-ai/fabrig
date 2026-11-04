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
