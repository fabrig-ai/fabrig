use super::{demo, fixtures as f};
use crate::{
    Address, ChatsFilter, Domain, Event, Place,
    pages::{self, Act, Request, project::Deletion},
};
use jig_client::{Told, contract as c};
use skein_lib::{List, Queue};

fn opened(role: c::HeldRole) -> Domain {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let watch = f::start(&mut root, List::with_capacity(0));
    let mut me = f::chats::me();
    let project = me.projects.get_mut(0).unwrap();
    project.project = 2;
    project.name = c::DocumentText::complete(f::acts::text(b"widget"));
    project.role = role;
    crate::step(
        &mut root,
        &f::env(&f::limits()),
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
        &mut Queue::uncapped(),
    );
    crate::step(
        &mut root,
        &f::env(&f::limits()),
        Event::Went { place: Place::Address(Address::ProjectSettings { project: 2 }) },
        &mut Queue::uncapped(),
    );
    root
}

fn event(root: &mut Domain, event: pages::Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    pages::step(&mut root.pages, &f::env(&f::limits()), &root.client, event, &mut out);
    out
}
fn act(root: &mut Domain, act: Act) -> Queue<Request> {
    event(root, pages::Event::Act { act })
}
fn typed(root: &mut Domain, words: &[u8]) {
    assert!(act(root, Act::DeleteWords { words: words.into() }).is_empty());
}
fn sent(root: &mut Domain) -> jig_client::Key {
    let mut out = act(root, Act::DeleteProject);
    let Some(Request::Client(jig_client::Event::Send { key, name, request })) = out.pop() else {
        panic!("delete sent")
    };
    assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
    assert_eq!(name, None);
    assert_eq!(request, jig_client::Ask::Jig(Box::new(c::Ask::RemoveProject { project: 2 })));
    assert!(out.is_empty());
    key
}

#[test]
fn delete_sends_remove_project_only_once_the_name_is_typed() {
    let mut root = opened(c::HeldRole::Owner);
    for words in [b"".as_slice(), b"widge", b"Widget", b"widget ", b"widget\n"] {
        typed(&mut root, words);
        assert!(act(&mut root, Act::DeleteProject).is_empty());
    }
    typed(&mut root, b"widget");
    let key = sent(&mut root);
    assert_eq!(root.pages().settings().unwrap().deleting, Some(key));
    assert!(act(&mut root, Act::DeleteProject).is_empty());
    typed(&mut root, b"other");
    assert_eq!(&*root.pages().settings().unwrap().typed, b"widget");
}

#[test]
fn a_member_who_is_no_owner_cannot_delete() {
    let mut root = opened(c::HeldRole::Member);
    typed(&mut root, b"widget");
    assert!(act(&mut root, Act::DeleteProject).is_empty());
}

#[test]
fn the_playgrounds_project_offers_no_delete() {
    let mut root = opened(c::HeldRole::Owner);
    let mut parts = temper_client_protocol_wire::decode_playground_answer(
        &demo::answer(b"Change"),
        &temper_client_protocol_wire::v1::CEILINGS,
    )
    .unwrap()
    .into_parts();
    parts.project = Some(2);
    let answer = temper_client_protocol_wire::playground_answer(
        &temper_client_protocol_wire::v1::PlaygroundAnswer::new(&temper_client_protocol_wire::v1::CEILINGS, parts)
            .unwrap(),
    );
    demo::deliver_demo_read(&mut root, jig_client::Readout::Family { family: f::FORGE, record: Some(answer) });
    assert_eq!(root.demo().unwrap().project, Some(2));
    typed(&mut root, b"widget");
    assert!(act(&mut root, Act::DeleteProject).is_empty());
}

#[test]
fn removed_opens_chats_and_refused_says_why() {
    for (answer, deletion, show) in [
        (c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectRemoved { project: 2 })), Deletion::Deleted, true),
        (
            c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None },
            Deletion::Refused(c::Refusal::NotOffered),
            false,
        ),
    ] {
        let mut root = opened(c::HeldRole::Owner);
        typed(&mut root, b"widget");
        let key = sent(&mut root);
        let told = Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) };
        let mut out = Queue::uncapped();
        crate::route::told(&mut root, &f::env(&f::limits()), told, &mut out);
        assert_eq!(root.pages().settings().unwrap().outcome, Some(deletion));
        assert_eq!(root.pages().settings().unwrap().deleting, None);
        assert_eq!(
            out.pop(),
            if show {
                Some(crate::Request::Show {
                    address: Address::Chats { filter: ChatsFilter::All, start: false },
                    replace: false,
                })
            } else {
                None
            }
        );
        assert!(out.is_empty());
        if !show {
            assert_ne!(sent(&mut root), key, "retry uses a fresh key");
        }
    }
}

#[test]
fn unknown_and_lost_and_unrelated_answers_claim_nothing() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Lost,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
            c::OwnerOutcome::ProjectRemoved { project: 3 },
        )))),
    ] {
        let mut root = opened(c::HeldRole::Owner);
        typed(&mut root, b"widget");
        let key = sent(&mut root);
        let mut other = key;
        other.number = other.number.checked_add(1).unwrap();
        assert!(
            event(&mut root, pages::Event::Told { told: Told::Answered { key: other, outcome: outcome.clone() } })
                .is_empty()
        );
        assert_eq!(root.pages().settings().unwrap().deleting, Some(key));
        assert!(event(&mut root, pages::Event::Told { told: Told::Answered { key, outcome } }).is_empty());
        assert_eq!(root.pages().settings().unwrap().outcome, Some(Deletion::Unknown));
        assert_ne!(sent(&mut root), key);
    }
}

#[test]
fn leaving_drops_the_typed_name() {
    let mut root = opened(c::HeldRole::Owner);
    typed(&mut root, b"widget");
    event(&mut root, pages::Event::Went { address: Address::ProjectSettings { project: 2 } });
    assert_eq!(&*root.pages().settings().unwrap().typed, b"widget");
    event(&mut root, pages::Event::Went { address: Address::Chats { filter: ChatsFilter::All, start: false } });
    assert!(root.pages().settings().is_none());
    event(&mut root, pages::Event::Went { address: Address::ProjectSettings { project: 2 } });
    assert!(root.pages().settings().unwrap().typed.is_empty());
    typed(&mut root, b"widget");
    event(&mut root, pages::Event::Left);
    assert!(root.pages().settings().is_none());
}
