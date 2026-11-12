use super::fixtures;
use crate::{
    Address,
    pages::{self, Act, Event, Pages, Request},
};
use jig_client::{Key, Outcome, Told, contract as c};
use skein_lib::{List, Queue, Token};

struct Fixture {
    root: crate::Domain,
    pages: Pages,
    watch: Token,
    out: Queue<Request>,
}
fn fixture() -> Fixture {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let watch = fixtures::start(&mut root, List::uncapped());
    let mut f = Fixture { root, pages: Pages::new(), watch, out: Queue::uncapped() };
    update(&mut f, fixtures::chats::me());
    event(&mut f, Event::Went { address: Address::Settings { project: 1 } });
    while f.out.pop().is_some() {}
    f
}
fn event(f: &mut Fixture, event: Event) {
    pages::step(&mut f.pages, &fixtures::env(&fixtures::limits()), f.root.client(), event, &mut f.out);
}
fn act(f: &mut Fixture, act: Act) {
    event(f, Event::Act { act });
}
fn update(f: &mut Fixture, me: c::Me) {
    crate::step(
        &mut f.root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: f.watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut Queue::uncapped(),
    );
}
fn open(f: &mut Fixture) {
    act(f, Act::DeleteProject { project: 1 });
}
fn sent(f: &mut Fixture) -> Key {
    let Some(Request::Client(jig_client::Event::Send { key, name: None, request: jig_client::Ask::Jig(ask) })) =
        f.out.pop()
    else {
        panic!("one keyed jig send")
    };
    assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
    assert_eq!(*ask, jig_client::shapes::remove_project(f.pages.me(f.root.client()).unwrap(), 1).unwrap());
    assert!(f.out.is_empty());
    key
}
fn answer(f: &mut Fixture, key: Key, answer: c::Answer) {
    event(
        f,
        Event::Told { told: Told::Answered { key, outcome: Outcome::Answered(jig_client::Answered::Jig(answer)) } },
    );
}
#[test]
fn deletion_opens_only_for_the_settings_owner_and_cancel_sends_nothing() {
    for role in [c::HeldRole::Owner, c::HeldRole::Member, c::HeldRole::Policy { role: 1 }] {
        let mut f = fixture();
        let mut me = fixtures::chats::me();
        me.projects.get_mut(0).unwrap().role = role;
        update(&mut f, me);
        open(&mut f);
        assert_eq!(f.pages.deleting_project().is_some(), role == c::HeldRole::Owner);
        assert!(f.out.is_empty());
        act(&mut f, Act::Cancel);
        assert!(f.pages.deleting_project().is_none());
        assert!(f.out.is_empty());
    }
}
#[test]
fn deleting_sends_the_helper_ask_once_and_waits_for_removed_before_showing_all_home() {
    let mut f = fixture();
    open(&mut f);
    act(&mut f, Act::Confirm);
    let key = sent(&mut f);
    assert_eq!(f.pages.deleting_project().unwrap().sent, Some(key));
    act(&mut f, Act::Confirm);
    open(&mut f);
    assert!(f.out.is_empty());
    assert_eq!(f.pages.deleting_project().unwrap().sent, Some(key));
    answer(&mut f, key, c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: 1 })));
    assert!(f.pages.deleting_project().is_none());
    assert_eq!(f.out.pop(), Some(Request::Show { address: Address::AllHome }));
    assert!(f.out.is_empty());
}
#[test]
fn a_refused_deletion_keeps_the_sheet_and_retries_under_a_new_key() {
    let mut f = fixture();
    open(&mut f);
    act(&mut f, Act::Confirm);
    let key = sent(&mut f);
    let refused = c::Answer::Refused { refusal: c::Refusal::Role { held: Some(c::HeldRole::Member) }, at: None };
    answer(&mut f, key, refused.clone());
    let sheet = f.pages.deleting_project().unwrap();
    assert_eq!(sheet.sent, None);
    assert_eq!(sheet.failed, Some(Outcome::Answered(jig_client::Answered::Jig(refused))));
    assert!(f.out.is_empty());
    act(&mut f, Act::Confirm);
    assert_ne!(sent(&mut f), key);
    assert_eq!(f.pages.deleting_project().unwrap().failed, None);
}
#[test]
fn live_tasks_refusal_retries_a_new_request_and_joins_only_the_pending_send() {
    let mut f = fixture();
    open(&mut f);
    act(&mut f, Act::Confirm);
    let first = sent(&mut f);
    let mut tasks = List::with_capacity(1);
    tasks.push(12).unwrap();
    answer(&mut f, first, c::Answer::Refused { refusal: c::Refusal::LiveTasks { tasks, more: 0 }, at: None });
    assert!(f.pages.deleting_project().unwrap().failed.is_some());
    act(&mut f, Act::Confirm);
    let second = sent(&mut f);
    assert_ne!(second, first, "a kept refusal must not be replayed");
    assert_eq!(f.pages.deleting_project().unwrap().failed, None);
    act(&mut f, Act::Confirm);
    assert!(f.out.is_empty(), "pending presses join the send");
    answer(&mut f, first, c::Answer::Refused { refusal: c::Refusal::Role { held: None }, at: None });
    assert_eq!(f.pages.deleting_project().unwrap().sent, Some(second), "old answers cannot finish the new try");
    answer(&mut f, second, c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: 1 })));
    assert!(f.pages.deleting_project().is_none());
    assert_eq!(f.out.pop(), Some(Request::Show { address: Address::AllHome }));
}

#[test]
fn a_project_that_left_me_closes_without_sending() {
    let mut f = fixture();
    open(&mut f);
    assert!(f.pages.deleting_project().is_some());
    let mut me = fixtures::chats::me();
    me.projects = List::uncapped();
    update(&mut f, me);
    act(&mut f, Act::Confirm);
    assert!(f.pages.deleting_project().is_none());
    assert!(f.out.is_empty());
}
