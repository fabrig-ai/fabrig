use super::{
    accounts::{deliver, event},
    fixtures,
};
use crate::{
    Action, Address, Event, Place,
    pages::{self, models::Act},
};
use jig_client::{Outcome, Told, contract as c};
use skein_lib::{List, Queue};

fn opened() -> (crate::Domain, skein_lib::Token) {
    let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let watch = fixtures::start(&mut root, List::with_capacity(0));
    let mut me = fixtures::chats::me();
    me.deployment_role = Some(c::HeldRole::Owner);
    me.projects.get_mut(0).unwrap().models = List::from_boxed(Box::new([c::MeModel {
        number: 2,
        source: c::ModelSource::Stock,
        estimated: false,
        name: fixtures::acts::text(b"claude-sonnet-5-5"),
        shown: None,
        provider: Some(17),
        window: 1024,
        output: 128,
        efforts: List::with_capacity(0),
        effort: None,
        accounts: List::with_capacity(0),
        picker: true,
        description: None,
        prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
    }]));
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))));
    event(&mut root, Event::Went { place: Place::Address(Address::Models { project: fixtures::PROJECT }) });
    (root, watch)
}
fn press(root: &mut crate::Domain) -> Queue<crate::Request> {
    event(root, Event::Act { action: Action::Pages(pages::Act::Models(Act::Picker { number: 2, on: false })) })
}
fn answer(root: &mut crate::Domain, outcome: Outcome) {
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: root.pages().serial };
    crate::route::told(
        root,
        &fixtures::env(&fixtures::limits()),
        Told::Answered { key, outcome },
        &mut Queue::uncapped(),
    );
}
#[test]
fn a_press_asks_the_act_once() {
    let (mut root, watch) = opened();
    let out = press(&mut root);
    let mut asks = List::uncapped();
    for request in &out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
        {
            asks.push(ask.as_ref()).unwrap();
        }
    }
    assert_eq!(
        asks.as_slice(),
        &[&c::Ask::ChangeModel(c::ChangeModel { act: c::ModelAct::Picker { number: 2, on: false } })]
    );
    assert!(root.pages().models().pending(2));
    assert!(!root.pages().models().pending(1));
    assert!(press(&mut root).is_empty());
    let mut me = root.pages().me(root.client()).unwrap().clone();
    assert!(me.projects.get(0).unwrap().models.get(0).unwrap().picker, "pending is not optimistic");
    answer(
        &mut root,
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
            c::OutcomeSettingsChanged { revision: 1 },
        )))),
    );
    assert!(!root.pages().models().pending(2));
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().picker = false;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))));
    assert!(!root.pages().me(root.client()).unwrap().projects.get(0).unwrap().models.get(0).unwrap().picker);
}
#[test]
fn a_refusal_stays_with_its_row() {
    let (mut root, _) = opened();
    press(&mut root);
    let outcome =
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }));
    answer(&mut root, outcome.clone());
    assert_eq!(root.pages().models().refused_at(2), Some(&outcome));
    assert_eq!(root.pages().models().refused_at(1), None);
    assert!(!root.pages().models().pending(2));
    press(&mut root);
    assert_eq!(root.pages().models().refused_at(2), None);
    assert!(root.pages().models().pending(2));
}
