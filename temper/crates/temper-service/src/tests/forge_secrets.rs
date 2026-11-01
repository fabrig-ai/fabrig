//! Forge removal and restart cleanup (temper/docs/design/draft/playground-reset.md, 4).
use crate::{engine::Engine, forge, setup, tests::fixtures};
use alloc::boxed::Box;
use jig_protocol_store::{Event, SecretOwner, SecretValue, Secrets};
use skein_lib::{Secret, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as domain;
use temper_protocol_records::{self as records, Item};

const UNKEPT: u16 = 77;

fn seed(engine: &mut Engine, forge: u16) {
    for item in forge::records::ITEMS {
        let request = Secrets::Keep {
            owner: SecretOwner::Application,
            number: records::number(forge, item),
            value: SecretValue::Application(Secret::new(Box::from([2, 255]))),
        };
        assert!(engine.store.secrets(Token::new(123), request).is_none());
        let Some(Event::Kept { owner }) = engine.store.above.pop() else { panic!("fixture secret kept") };
        assert_eq!(owner, Token::new(123));
    }
}
fn absent(engine: &mut Engine, forge: u16) {
    for item in forge::records::ITEMS {
        let found = engine.store.secrets(
            Token::new(124),
            Secrets::Get { owner: SecretOwner::Application, number: records::number(forge, item) },
        );
        let Some(Event::Secret { value: None, .. }) = found else { panic!("credential erased") };
    }
}
fn erase_answers(engine: &mut Engine) {
    for _ in 0..forge::records::ITEMS.len() {
        let Some(event @ Event::Erased { .. }) = engine.store.above.pop() else { panic!("one erase in flight") };
        setup::answer(engine, event);
    }
    assert!(!setup::erasing_records(engine));
}
#[test]
fn a_removed_forge_has_every_credential_record_erased() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    seed(engine, 1);
    seed(engine, UNKEPT);
    let before = engine.store.erases;
    crate::route::request(engine, root::Request::ForgeRemoved { forge: 1 });
    assert_eq!(engine.store.erases, before + 1, "one credential write at a time");
    erase_answers(engine);
    assert_eq!(engine.store.erases, before + 10);
    absent(engine, 1);
    let Some(Event::Secret { value: Some(_), .. }) = engine.store.secrets(
        Token::new(124),
        Secrets::Get { owner: SecretOwner::Application, number: records::number(UNKEPT, Item::AppKey) },
    ) else {
        panic!("another forge is untouched")
    };
    // Repeating removal, including every absent item, is successful.
    forge::table::removed(engine, 1);
    erase_answers(engine);
    absent(engine, 1);
}
#[test]
fn a_removed_forges_public_connect_is_answered_not_connected() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let value = records::encode_pasted(1, b"fixture-admin").unwrap();
    let digest = skein_lib::sha256::digest(value.expose());
    engine.store.secrets(Token::new(123), fixtures::application_request(records::CARRIED, value));
    let Some(Event::Carried { number, .. }) = engine.store.above.pop() else { panic!("paste carried") };
    let request = Token::new(999);
    setup::public::admin_connect(engine, 1, request, None, jig_domain_core::Carried { number, digest });
    engine.events.clear();
    forge::table::removed(engine, 1);
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("public answer") };
    assert_eq!(
        *event,
        domain::Event::ConnectAnswered {
            forge: 1,
            request,
            answer: domain::ConnectAnswer::Refused(domain::ForgeRefusal::NotConnected),
        }
    );
    setup::advance(engine);
    assert!(engine.events.is_empty(), "no second terminal from queued host output");
    assert!(engine.setup.staged.is_empty());
    let Some(Event::Secret { value: None, .. }) =
        engine.store.secrets(Token::new(124), Secrets::Get { owner: SecretOwner::Application, number })
    else {
        panic!("carried paste erased")
    };
}
#[test]
fn a_removed_forge_is_never_connected_again() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let before = engine.forge.starts.len();
    forge::table::removed(engine, 1);
    assert!(forge::ended(engine, 1));
    assert!(forge::ended(engine, UNKEPT));
    forge::connect(
        engine,
        1,
        Ok(forge::Start::Adopted(skein_forge_connection::Access {
            credential: skein_forge::credential::Credential::Token(Secret::new(b"fixture-api".as_slice().into())),
            user: 9,
            webhook_secret: None,
        })),
    );
    forge::records::connect_one(engine, 1);
    setup::public::manifest(engine, 1, Token::new(998), None);
    assert_eq!(engine.forge.starts.len(), before);
    assert!(engine.setup.staged.is_empty());
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("missing forge refused") };
    assert_eq!(
        *event,
        domain::Event::ConnectAnswered {
            forge: 1,
            request: Token::new(998),
            answer: domain::ConnectAnswer::Refused(domain::ForgeRefusal::NotConnected),
        }
    );
}
#[test]
fn leftover_records_of_an_unkept_forge_are_erased_at_the_next_start() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    seed(engine, UNKEPT);
    forge::table::removed(engine, UNKEPT);
    // The first erase is durable; its completion and the remaining work die with the process.
    let Some(Event::Erased { .. }) = engine.store.above.pop() else { panic!("first erase durable") };
    let previous = service.engine.take().unwrap();
    let mut restarted = fixtures::restarting_from(previous);
    fixtures::restart(&mut restarted);
    let engine = restarted.engine.as_mut().unwrap();
    assert!(engine.forge_records.done());
    absent(engine, UNKEPT);
    assert!(forge::table::record(engine, 1).is_some(), "the connector's kept forge survives");
    for note in &engine.notes {
        assert_ne!(*note, crate::Note::ForgeUnconfigured { number: UNKEPT });
    }
}

#[test]
fn a_removed_forges_other_public_requests_are_answered_not_connected() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    forge::table::removed(engine, 1);
    let request = Token::new(999);
    setup::public::install(engine, 1, request, None);
    setup::public::accept(engine, 1, request, core::num::NonZeroU64::new(7).unwrap());
    setup::public::reachable(engine, 1, request, None, 1);
    for expected in [
        domain::Event::InstallAnswered {
            forge: 1,
            request,
            answer: domain::InstallAnswer::Refused(domain::ForgeRefusal::NotConnected),
            stopped: None,
        },
        domain::Event::AcceptAnswered {
            forge: 1,
            request,
            answer: domain::AcceptAnswer::Refused(domain::ForgeRefusal::NotConnected),
            stopped: None,
        },
        domain::Event::ReachableAnswered {
            forge: 1,
            request,
            answer: domain::ReachableAnswer::Refused(domain::ForgeRefusal::NotConnected),
        },
    ] {
        let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("public answer") };
        assert_eq!(*event, expected);
    }
    assert!(engine.events.is_empty());
    assert!(engine.setup.staged.is_empty());
}
