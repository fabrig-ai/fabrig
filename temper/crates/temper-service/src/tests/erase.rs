use crate::tests::fixtures;
use jig_protocol_store::{Event, SecretOwner, Secrets};
use skein_lib::Token;
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_protocol_records::{ITEMS, number};

fn service() -> crate::Service {
    let mut service = fixtures::ready();
    service.engine.as_mut().unwrap().store = fixtures::records_store();
    service
}
fn erase(service: &mut crate::Service) {
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::EraseSecrets { forge: 1 });
}
fn complete(service: &mut crate::Service, number: u16, answers: u32) {
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.events.is_empty(), "no answer before durable terminals");
    for index in 0_u32..10 {
        let event = engine.store.above.pop().expect("erase terminal");
        let Event::Erased { .. } = &event else { panic!("durable erase terminal") };
        crate::setup::answer(engine, event);
        if index < 9 {
            assert!(engine.events.is_empty());
        }
    }
    for _ in 0..answers {
        let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("erase answer") };
        assert_eq!(*event, forge::Event::SecretsErased { forge: number });
    }
    assert!(engine.events.is_empty());
}
#[test]
fn erase_secrets_waits_for_each_durable_terminal_before_submitting_the_next_item() {
    let mut service = service();
    let engine = service.engine.as_mut().unwrap();
    crate::setup::tests::kept_record(&mut engine.store, 1, ITEMS[0], b"test-value");
    let before = engine.store.erases;
    erase(&mut service);
    let engine = service.engine.as_mut().unwrap();
    for index in 0_u32..10 {
        assert_eq!(engine.store.erases - before, index + 1, "one erase in flight, including absent items");
        assert_eq!(engine.store.above.len(), 1, "one unconsumed durable terminal");
        assert!(engine.events.is_empty(), "no completion until every item is durable");
        let event = engine.store.above.pop().expect("erase terminal");
        let Event::Erased { .. } = &event else { panic!("durable erase terminal") };
        crate::setup::answer(engine, event);
    }
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("erase answer") };
    assert_eq!(*event, forge::Event::SecretsErased { forge: 1 });
    assert!(engine.events.is_empty());
    assert!(engine.store.above.is_empty());
}
#[test]
fn erase_secrets_erases_each_item_and_answers_once_durable() {
    let mut service = service();
    let engine = service.engine.as_mut().unwrap();
    for item in ITEMS {
        crate::setup::tests::kept_record(&mut engine.store, 1, item, b"test-value");
    }
    crate::setup::tests::kept_record(&mut engine.store, 2, ITEMS[0], b"other-forge");
    let before = engine.store.erases;
    erase(&mut service);
    complete(&mut service, 1, 1);
    assert_eq!(service.engine.as_ref().unwrap().store.erases - before, 10);
    let engine = service.engine.as_mut().unwrap();
    for item in ITEMS {
        let Some(Event::Secret { value: None, .. }) = engine
            .store
            .secrets(Token::new(99), Secrets::Get { owner: SecretOwner::Application, number: number(1, item) })
        else {
            panic!("forge record erased")
        };
    }
    let Some(Event::Secret { value: Some(_), .. }) = engine
        .store
        .secrets(Token::new(99), Secrets::Get { owner: SecretOwner::Application, number: number(2, ITEMS[0]) })
    else {
        panic!("other forge record retained")
    };
}
#[test]
fn erase_secrets_of_a_forge_with_no_records_answers_erased() {
    let mut service = service();
    erase(&mut service);
    complete(&mut service, 1, 1);
    assert!(service.engine.as_ref().unwrap().store.above.is_empty());
}
#[test]
fn two_erases_of_one_forge_answer_each() {
    let mut service = service();
    let before = service.engine.as_ref().unwrap().store.erases;
    erase(&mut service);
    erase(&mut service);
    complete(&mut service, 1, 2);
    assert_eq!(service.engine.as_ref().unwrap().store.erases - before, 10);
    assert!(service.engine.as_ref().unwrap().store.above.is_empty());
}
#[test]
fn erasing_two_forges_shares_one_durable_flight_and_answers_each_forge() {
    let mut service = service();
    let before = service.engine.as_ref().unwrap().store.erases;
    erase(&mut service);
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::EraseSecrets { forge: 2 });
    assert_eq!(service.engine.as_ref().unwrap().store.erases - before, 1);
    assert_eq!(service.engine.as_ref().unwrap().store.above.len(), 1);
    complete(&mut service, 1, 1);
    assert_eq!(service.engine.as_ref().unwrap().store.erases - before, 11);
    assert_eq!(service.engine.as_ref().unwrap().store.above.len(), 1);
    complete(&mut service, 2, 1);
    assert_eq!(service.engine.as_ref().unwrap().store.erases - before, 20);
    assert!(service.engine.as_ref().unwrap().store.above.is_empty());
}
#[test]
fn a_failed_erase_stops_the_service_without_answering_erased() {
    let mut service = service();
    erase(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let Some(Event::Erased { owner }) = engine.store.above.pop() else { panic!("erase terminal") };
    crate::setup::answer(engine, Event::Corrupt { owner });
    assert!(engine.end.is_some());
    assert!(engine.events.is_empty());
}
