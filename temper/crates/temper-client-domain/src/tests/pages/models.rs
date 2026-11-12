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
    me.providers = List::from_boxed(Box::new([c::ProviderOffer {
        provider: 17,
        name: c::DocumentText::complete(fixtures::acts::text(b"anthropic")),
        key_bytes: 4096,
        sign_in: true,
        key: true,
        programs: List::with_capacity(0),
        programs_cut: false,
        none: false,
        signed_in_plan: true,
        manage_usage: None,
    }]));
    let mut openai = me.providers.get(0).unwrap().clone();
    openai.provider = 18;
    openai.name = c::DocumentText::complete(fixtures::acts::text(b"openai"));
    me.providers = List::from_boxed(Box::new([me.providers.get(0).unwrap().clone(), openai]));
    me.deployment_role = Some(c::HeldRole::Owner);
    me.projects.get_mut(0).unwrap().models = List::from_boxed(Box::new([c::MeModel {
        new_until: None,
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
fn answer(root: &mut crate::Domain, outcome: Outcome) {
    let key = jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: root.pages().serial };
    crate::route::told(
        root,
        &fixtures::env(&fixtures::limits()),
        Told::Answered { key, outcome },
        &mut Queue::uncapped(),
    );
}
fn reading_fixture() -> (crate::Domain, skein_lib::Token) {
    let (mut root, _, watch) = super::accounts::opened(c::HeldRole::Policy { role: 1 });
    let c::WatchEvent::Delivered(c::Delivery::Accounts(mut doc)) = super::accounts::document(false) else {
        unreachable!()
    };
    doc.accounts = List::with_capacity(1);
    doc.accounts
        .push(c::AccountLine {
            facts: List::with_capacity(0),
            list: Some(c::AccountList { read: None, tried: None, failure: None, left_out: 0 }),
            number: 1,
            place: 1,
            uses: None,
            plan: false,
            provider: Some(17),
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(fixtures::acts::text(b"Own")),
            holder: c::AccountHolder::Party(fixtures::PARTY),
            sources: List::from_boxed(Box::new([c::AccountSource {
                name: None,
                place: 1,
                number: 1,
                kind: c::AccountKind::Key,
                state: c::AccountState::Granted,
                usable: true,
                expiring: false,
                attention: false,
                spent_until: None,
            }])),
            cut: false,
        })
        .unwrap();
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
    event(&mut root, Event::Went { place: Place::Address(Address::Models { project: fixtures::PROJECT }) });
    (root, watch)
}
fn moved(root: &mut crate::Domain, watch: skein_lib::Token) {
    let doc = pages::accounts::document(root.client()).unwrap().clone();
    deliver(root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))));
}
#[test]
fn a_press_asks_the_act_once() {
    for read in [false, true] {
        for watch_first in [false, true] {
            let (mut root, watch) = if read { reading_fixture() } else { opened() };
            let action = if read { Act::Read(17) } else { Act::Picker { number: 2, on: false } };
            let out = act(&mut root, action.clone());
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
            let expected = if read {
                c::Ask::ReadLists { provider: 17 }
            } else {
                c::Ask::ChangeModel(c::ChangeModel { act: c::ModelAct::Picker { number: 2, on: false } })
            };
            assert_eq!(asks.as_slice(), [&expected]);
            assert!(if read { root.pages().models().reading(17) } else { root.pages().models().pending(2) });
            assert!(act(&mut root, action.clone()).is_empty());
            if read && watch_first {
                moved(&mut root, watch);
            }
            let outcome = if read {
                c::Outcome::Account(c::AccountAnswer::Reading { count: 1 })
            } else {
                c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 1 })
            };
            answer(&mut root, Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(outcome))));
            if read {
                assert_eq!(root.pages().models().reading(17), !watch_first);
                if !watch_first {
                    assert!(act(&mut root, action.clone()).is_empty());
                    moved(&mut root, watch);
                }
                assert!(!root.pages().models().reading(17));
                assert!(!act(&mut root, action).is_empty());
            } else {
                assert!(!root.pages().models().pending(2));
                let mut me = root.pages().me(root.client()).unwrap().clone();
                assert!(me.projects.get(0).unwrap().models.get(0).unwrap().picker, "pending is not optimistic");
                me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().picker = false;
                deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))));
                assert!(
                    !root.pages().me(root.client()).unwrap().projects.get(0).unwrap().models.get(0).unwrap().picker
                );
            }
        }
    }
}
#[test]
fn a_refusal_stays_with_its_row() {
    for read in [false, true] {
        let (mut root, _) = if read { reading_fixture() } else { opened() };
        let action = if read { Act::Read(17) } else { Act::Picker { number: 2, on: false } };
        act(&mut root, action.clone());
        let outcome = Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NotOffered,
            at: None,
        }));
        answer(&mut root, outcome.clone());
        assert_eq!(
            if read { root.pages().models().read_refused(17) } else { root.pages().models().refused_at(2) },
            Some(&outcome)
        );
        assert_eq!(root.pages().models().refused_at(1), None);
        assert_eq!(root.pages().models().read_refused(18), None);
        assert!(!root.pages().models().pending(2));
        assert!(!root.pages().models().reading(17));
        act(&mut root, action);
        assert_eq!(
            if read { root.pages().models().read_refused(17) } else { root.pages().models().refused_at(2) },
            None
        );
        assert!(if read { root.pages().models().reading(17) } else { root.pages().models().pending(2) });
    }
}

fn act(root: &mut crate::Domain, act: Act) -> Queue<crate::Request> {
    event(root, Event::Act { action: Action::Pages(pages::Act::Models(act)) })
}
#[test]
fn add_asks_once_with_the_provider_and_the_name() {
    let (mut root, _) = opened();
    let me = root.pages().me(root.client()).unwrap();
    assert_eq!(root.pages().models().provider(me, Some(17)), Some(18));
    assert_eq!(root.pages().models().provider(me, None), Some(17));
    act(&mut root, Act::Provider(18));
    act(&mut root, Act::Name(b"claude-haiku-5-5".as_slice().into()));
    let out = act(&mut root, Act::Add);
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
        &[&c::Ask::ChangeModel(c::ChangeModel {
            act: c::ModelAct::Add { provider: 18, name: fixtures::acts::text(b"claude-haiku-5-5") }
        })]
    );
    assert!(root.pages().models().add_pending());
    assert!(act(&mut root, Act::Add).is_empty());
    answer(
        &mut root,
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::SettingsChanged(
            c::OutcomeSettingsChanged { revision: 1 },
        )))),
    );
    assert!(!root.pages().models().add_pending());
    assert_eq!((root.pages().models().name()).len(), 0);
    assert_eq!(root.pages().models().added(), Some(18));
}
#[test]
fn a_name_out_of_shape_keeps_add_off() {
    let (mut root, _) = opened();
    for name in [b"".as_slice(), b"space name", b"slash/name", b"new\nline", "modèle".as_bytes(), &[0], &[b'a'; 129]] {
        act(&mut root, Act::Name(name.into()));
        let me = root.pages().me(root.client()).unwrap();
        assert!(!root.pages().models().can_add(me, fixtures::PROJECT, None), "{name:?}");
        assert!(act(&mut root, Act::Add).is_empty());
    }
}
#[test]
fn name_shape_takes_the_shapes_a_provider_names_a_model_by() {
    // These rows also check jig-domain-core's settings::catalogue::valid_name.
    for name in [b"claude-opus-5-5".as_slice(), b"openai:gpt_6.1", b"A0", b"-._:", &[b'a'; 128]] {
        assert!(pages::models::name_shape(name), "{name:?}");
    }
    for name in [b"".as_slice(), b"space name", b"slash/name", b"new\nline", "modèle".as_bytes(), &[0], &[b'a'; 129]] {
        assert!(!pages::models::name_shape(name), "{name:?}");
    }
}
#[test]
fn a_name_the_provider_holds_says_so() {
    let (mut root, _) = opened();
    act(&mut root, Act::Name(b"claude-sonnet-5-5".as_slice().into()));
    let me = root.pages().me(root.client()).unwrap();
    assert!(root.pages().models().duplicate(me, fixtures::PROJECT, None));
    assert!(act(&mut root, Act::Add).is_empty());
}
#[test]
fn a_member_cannot_add() {
    let (mut root, watch) = opened();
    act(&mut root, Act::Name(b"claude-haiku-5-5".as_slice().into()));
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.deployment_role = Some(c::HeldRole::Policy { role: 1 });
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))));
    assert!(act(&mut root, Act::Add).is_empty());
}
#[test]
fn an_add_refusal_stays_under_the_form() {
    let (mut root, _) = opened();
    act(&mut root, Act::Name(b"claude-haiku-5-5".as_slice().into()));
    act(&mut root, Act::Add);
    let outcome =
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }));
    answer(&mut root, outcome.clone());
    assert_eq!(root.pages().models().add_refused(), Some(&outcome));
    assert_eq!(root.pages().models().refused_at(2), None);
    assert_eq!(root.pages().models().name(), b"claude-haiku-5-5");
    act(&mut root, Act::Add);
    assert_eq!(root.pages().models().add_refused(), None);
}
