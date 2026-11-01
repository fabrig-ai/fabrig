use super::fixtures;
use jig_client::contract::{Delivery, DocumentText, Text, WatchEvent};
use skein_ui::{Act, Role, State};
use temper_client_domain::{Action, Address, ChatsFilter, Domain, Event, Place, pages};

fn root(address: Address) -> (Domain, skein_lib::Token) {
    let (mut root, watch, _, _, _) = fixtures::root_at_full(address, &fixtures::limits(), b"Shop");
    let mut me = root.pages().me(root.client()).expect("me delivered").clone();
    for index in 0..me.projects.len() {
        let row = me.projects.get_mut(index).expect("counted project");
        if row.project == 2 {
            row.name = DocumentText::complete(Text::new(Box::from(*b"widget"), 32).expect("name fits"));
        }
    }
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me))) },
            },
        },
    );
    (root, watch)
}

fn assert_page(root: &Domain, heading: &[u8], paragraph: &[u8]) {
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, root);
        let mut mains = 0_u32;
        for node in tree.nodes() {
            if node.role == Role::Main {
                mains = mains.checked_add(1).expect("node count fits");
            }
        }
        assert_eq!(mains, 1);
        assert!(fixtures::find(tree, Role::Heading { level: 1 }, heading).is_some());
        assert!(fixtures::find(tree, Role::Paragraph, paragraph).is_some());
    }
}

#[test]
fn the_settings_page_names_its_project() {
    for address in [Address::Chats { filter: ChatsFilter::All, start: false }, Address::ProjectSettings { project: 2 }]
    {
        let (mut root, _) = root(address.clone());
        let expected = if address == (Address::ProjectSettings { project: 2 }) { 2 } else { 1 };
        assert_eq!(root.pages().project(), Some(expected));
        fixtures::root_event(&mut root, Event::Went { place: Place::Address(Address::ProjectSettings { project: 2 }) });
        assert_eq!(root.pages().project(), Some(2));
        assert_page(&root, "widget · settings".as_bytes(), b"Project 2");
    }
}

#[test]
fn a_project_me_does_not_list_is_gone() {
    let (mut root, watch) = root(Address::ProjectSettings { project: 9 });
    assert_page(&root, b"Settings", b"This project is gone.");
    fixtures::root_event(&mut root, Event::Went { place: Place::Address(Address::ProjectSettings { project: 2 }) });
    assert_page(&root, "widget · settings".as_bytes(), b"Project 2");
    let mut me = root.pages().me(root.client()).expect("me delivered").clone();
    me.projects.clear();
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me))) },
            },
        },
    );
    assert_page(&root, b"Settings", b"This project is gone.");
    assert_eq!(root.pages().project(), None);
}

#[test]
fn the_navigation_links_the_current_projects_settings() {
    let (mut root, _) = root(Address::Chats { filter: ChatsFilter::All, start: false });
    fixtures::root_event(&mut root, Event::Act { action: Action::Pages(pages::Act::Project { project: 2 }) });
    for current in [false, true] {
        if current {
            fixtures::root_event(
                &mut root,
                Event::Went { place: Place::Address(Address::ProjectSettings { project: 2 }) },
            );
        }
        let mut view = fixtures::view();
        fixtures::build(&mut view, &root);
        let link = fixtures::find(view.tree(), Role::Link, b"Settings").expect("project settings link");
        for node in view.tree().nodes() {
            if node.id == link {
                assert_eq!(node.states.as_slice().contains(&State::Current), current);
            }
        }
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(Action::Pages(pages::Act::Go { address: Address::ProjectSettings { project: 2 } }))
        );
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/2/settings".as_slice()));
    }
}
