//! Forge removal's service boundary (temper/docs/design/protocol/forge.md, 2.3).
use crate::tests::fixtures;
use crate::{Service, forge::gone};
use jig_protocol_store::{Event, SecretOwner, Secrets};
use skein_lib::{Secret, Token};
use temper_engine_domain as root;
use temper_protocol_records::{self as records, Item};

const ITEMS: [Item; 10] = [
    Item::AdminToken,
    Item::ApiToken,
    Item::GitToken,
    Item::User,
    Item::WebhookSecret,
    Item::AppId,
    Item::AppSlug,
    Item::AppKey,
    Item::Accepted,
    Item::Origin,
];

fn service() -> Service {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.store = fixtures::marked_store();
    engine.events = skein_lib::Queue::uncapped();
    for forge in [1, 99] {
        for item in ITEMS {
            let number = records::number(forge, item);
            let value = Secret::new(Box::from(b"synthetic secret record".as_slice()));
            assert!(
                engine.store.secrets(Token::new(123), fixtures::application_request(number, value.clone())).is_none()
            );
            let Some(Event::Kept { .. }) = engine.store.above.pop() else { panic!("secret kept") };
            engine.forge_records.rows.push(number, value, 1024);
        }
    }
    service
}

fn request(service: &mut Service, forge: u16) {
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::ForgeGone { forge });
}

fn complete(service: &mut Service) {
    let engine = service.engine.as_mut().unwrap();
    for _ in 0_u32..10 {
        let event = engine.store.above.pop().expect("durable erase");
        let Event::Erased { owner } = event else { panic!("durable erase") };
        gone::answer(engine, owner);
    }
    assert!(engine.store.above.is_empty());
}

fn answered(service: &mut Service, forge: u16) {
    let engine = service.engine.as_mut().unwrap();
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("forge answer") };
    let temper_engine_domain_forge::Event::SecretsErased { forge: found } = *event else { panic!("secrets erased") };
    assert_eq!(found, forge);
    assert!(engine.events.is_empty());
}

fn absent(store: &mut crate::store::Store, forge: u16) {
    for item in ITEMS {
        let answer = store.secrets(
            Token::new(124),
            Secrets::Get { owner: SecretOwner::Application, number: records::number(forge, item) },
        );
        let Some(Event::Secret { value: None, .. }) = answer else { panic!("secret absent") };
    }
}

#[test]
fn forge_gone_removes_the_forge_from_both_components() {
    let mut service = service();
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.workers.forges.get(1).is_some());
    assert!(engine.setup.served.contains_key(&1));
    assert!(engine.forge.component.start(1).is_some());
    let identity = engine.workers.forges.get(1).unwrap().identity;
    engine.values.identities.put(identity, Secret::new(Box::from(b"old identity".as_slice())));
    engine.values.identities.put(identity, Secret::new(Box::from(b"new identity".as_slice())));
    request(&mut service, 1);
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.workers.forges.get(1).is_none());
    assert!(engine.forge.component.start(1).is_none());
    assert!(!engine.setup.served.contains_key(&1));
    assert!(engine.forge_records.rows.forge(1).is_none());
    assert!(engine.forge_records.rows.forge(99).is_some());
    assert!(engine.values.identities.newest(identity).is_none());
    assert!(crate::forge::tokens::account(&engine.values.tokens, 1, b"writer").is_none());
}

#[test]
fn forge_gone_erases_every_secret_item_then_answers() {
    let mut service = service();
    request(&mut service, 1);
    request(&mut service, 1);
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.events.is_empty(), "durability is withheld");
    for _ in 0_u32..9 {
        let Some(Event::Erased { owner }) = engine.store.above.pop() else { panic!("durable erase") };
        gone::answer(engine, owner);
        assert!(engine.events.is_empty(), "all erases must commit");
    }
    let Some(Event::Erased { owner }) = engine.store.above.pop() else { panic!("final erase") };
    gone::answer(engine, owner);
    assert_eq!(engine.store.erases, 10);
    absent(&mut engine.store, 1);
    for item in ITEMS {
        let Some(Event::Secret { value: Some(_), .. }) = engine.store.secrets(
            Token::new(125),
            Secrets::Get { owner: SecretOwner::Application, number: records::number(99, item) },
        ) else {
            panic!("other forge's secret retained")
        };
    }
    answered(&mut service, 1);
}

#[test]
fn forge_gone_again_erases_nothing_and_answers() {
    let mut service = service();
    // Forge 99 has records and no runtime.
    assert!(service.engine.as_ref().unwrap().workers.forges.get(99).is_none());
    request(&mut service, 99);
    complete(&mut service);
    answered(&mut service, 99);
    request(&mut service, 99);
    assert_eq!(service.engine.as_ref().unwrap().store.erases, 10);
    assert!(service.engine.as_ref().unwrap().store.above.is_empty());
    answered(&mut service, 99);
}

#[test]
fn a_restart_after_gone_finds_no_secret_of_the_forge() {
    let mut service = service();
    request(&mut service, 1);
    complete(&mut service);
    let previous = service.engine.take().unwrap();
    let mut store = fixtures::reopened_store(previous.store, &previous.env.limits);
    fixtures::mark_store(&mut store);
    absent(&mut store, 1);
}

#[test]
fn a_failed_erase_stops_without_answering() {
    let mut service = service();
    request(&mut service, 1);
    let engine = service.engine.as_mut().unwrap();
    let Some(Event::Erased { owner }) = engine.store.above.pop() else { panic!("erase") };
    engine.store.above.push(Event::Corrupt { owner });
    crate::engine::answer(engine);
    assert!(engine.end.is_some());
    assert!(engine.events.is_empty());
}

#[test]
fn erase_completions_follow_the_service_store_route() {
    let mut service = service();
    request(&mut service, 99);
    for _ in 0_u32..32 {
        fixtures::turn(&mut service);
    }
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.end.is_none());
    assert_eq!(engine.forge.erasing.get(&99), Some(&None));
    assert_eq!(engine.store.erases, 10);
    absent(&mut engine.store, 99);
}

#[test]
fn a_forge_with_no_runtime_or_records_is_erased_and_answered() {
    let mut service = service();
    request(&mut service, 100);
    complete(&mut service);
    answered(&mut service, 100);
    request(&mut service, 100);
    assert_eq!(service.engine.as_ref().unwrap().store.erases, 10);
    answered(&mut service, 100);
}

#[test]
fn a_restored_removal_waits_for_the_durable_start_mark() {
    let mut service = service();
    let engine = service.engine.as_mut().unwrap();
    engine.store = fixtures::memory_store();
    request(&mut service, 1);
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.store.erases, 0);
    assert!(engine.store.above.is_empty());
    fixtures::mark_store(&mut engine.store);
    gone::advance(engine);
    complete(&mut service);
    answered(&mut service, 1);
    assert!(service.engine.as_ref().unwrap().forge.erase_waiting.is_empty());
}
