use super::fixtures as f;
use jig_client::contract as c;
use skein_ui::{Act, Role, State};
use temper_client_domain::{Action, Address, Domain, Event, Place, pages};

fn root(role: c::HeldRole, playground: bool) -> (Domain, skein_lib::Token) {
    let demo = if playground { Some((2, true)) } else { None };
    let (mut root, watch, _, _, _) =
        f::root_at_with_demo(Address::ProjectSettings { project: 2 }, &f::limits(), b"Shop", demo);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let row = me.projects.get_mut(1).unwrap();
    assert_eq!(row.project, 2);
    row.name = c::DocumentText::complete(c::Text::new(Box::from(*b"widget"), 32).unwrap());
    row.role = role;
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
    (root, watch)
}
fn disabled(tree: &skein_ui::Tree, name: &[u8]) -> bool {
    let id = f::find(tree, Role::Button, name).expect("delete button");
    for node in tree.nodes() {
        if node.id == id {
            return node.states.as_slice().contains(&State::Disabled);
        }
    }
    panic!("button node");
}
fn type_name(root: &mut Domain, view: &mut crate::View, name: &[u8]) {
    f::build(view, root);
    let field = f::find(view.tree(), Role::TextField, b"Type widget to confirm").unwrap();
    assert_eq!(crate::act(view, field, Act::Submit), None, "confirmation field does not submit");
    let action = crate::act(view, field, Act::Input { value: name.into() }).unwrap();
    assert_eq!(action, Action::Pages(pages::Act::DeleteWords { words: name.into() }));
    f::root_event(root, Event::Act { action });
}
fn press(root: &mut Domain, view: &mut crate::View) -> skein_lib::Token {
    f::build(view, root);
    let button = f::find(view.tree(), Role::Button, b"Delete widget").unwrap();
    let action = crate::act(view, button, Act::Press).unwrap();
    assert_eq!(action, Action::Pages(pages::Act::DeleteProject));
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Act { action },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
        {
            assert_eq!(*ask, c::Ask::RemoveProject { project: 2 });
            return attempt;
        }
    }
    panic!("delete sent");
}
fn answer(root: &mut Domain, attempt: skein_lib::Token, answer: c::Answer) {
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(answer)),
                },
            },
        },
    );
}

#[test]
fn an_owner_sees_delete_confirmed_by_name() {
    for words in [f::words(), f::browser_words()] {
        let (mut root, _) = root(c::HeldRole::Owner, false);
        let description = words.say(crate::words::Word::ProjectDeleteDescription, &[]);
        let mut view = f::view_of(words);
        let tree = f::build(&mut view, &root);
        assert!(f::find(tree, Role::Heading { level: 2 }, b"Delete project").is_some());
        assert!(f::find(tree, Role::Paragraph, &description).is_some());
        assert!(f::find(tree, Role::TextField, b"Type widget to confirm").is_some());
        assert!(disabled(tree, b"Delete widget"));
        for name in [b"widge".as_slice(), b"Widget", b"widget "] {
            type_name(&mut root, &mut view, name);
            assert!(disabled(f::build(&mut view, &root), b"Delete widget"));
        }
        type_name(&mut root, &mut view, b"widget");
        assert!(!disabled(f::build(&mut view, &root), b"Delete widget"));
    }
}

#[test]
fn a_member_sees_only_who_may_delete() {
    let (root, _) = root(c::HeldRole::Member, false);
    let mut view = f::view();
    let tree = f::build(&mut view, &root);
    assert!(f::find(tree, Role::Heading { level: 2 }, b"Delete project").is_some());
    assert!(f::find(tree, Role::Paragraph, "Only the project’s owners can delete it.".as_bytes()).is_some());
    assert!(f::find(tree, Role::TextField, b"Type widget to confirm").is_none());
    assert!(f::find(tree, Role::Button, b"Delete widget").is_none());
}

#[test]
fn the_playgrounds_project_has_no_delete_section() {
    let (root, _) = root(c::HeldRole::Owner, true);
    let mut view = f::view();
    let tree = f::build(&mut view, &root);
    assert!(f::find(tree, Role::Heading { level: 2 }, b"Delete project").is_none());
    assert!(f::find(tree, Role::TextField, b"Type widget to confirm").is_none());
    assert!(f::find(tree, Role::Button, b"Delete widget").is_none());
}

#[test]
fn deleting_and_deleted_and_refused_are_said() {
    for words in [f::words(), f::browser_words()] {
        let (mut root, watch) = root(c::HeldRole::Owner, false);
        let mut view = f::view_of(words);
        type_name(&mut root, &mut view, b"widget");
        let attempt = press(&mut root, &mut view);
        let tree = f::build(&mut view, &root);
        assert!(f::find(tree, Role::Status, "Deleting widget…".as_bytes()).is_some());
        assert!(f::find(tree, Role::Button, b"Delete widget").is_none());
        answer(&mut root, attempt, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
        let tree = f::build(&mut view, &root);
        assert!(f::find(tree, Role::Status, b"Not deleted: This act is not offered.").is_some());
        assert!(!disabled(tree, b"Delete widget"));
        let attempt = press(&mut root, &mut view);
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.clear();
        f::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
        );
        let tree = f::build(&mut view, &root);
        assert!(f::find(tree, Role::Status, "Deleting widget…".as_bytes()).is_some());
        assert!(f::find(tree, Role::Button, b"Delete widget").is_none());
        answer(&mut root, attempt, c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectRemoved { project: 2 })));
        assert!(f::find(f::build(&mut view, &root), Role::Status, b"widget is deleted.").is_some());
        f::root_event(
            &mut root,
            Event::Went {
                place: Place::Address(Address::Chats { filter: temper_client_domain::ChatsFilter::All, start: false }),
            },
        );
        assert!(root.pages().settings().is_none());
    }
}
