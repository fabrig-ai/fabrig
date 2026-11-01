//! Record-driven service membership (temper/docs/design/domain/root.md, 9.1; contracts, D1).
use super::*;
use crate::engine::Engine;
use jig_protocol_store::{Event, SecretOwner, SecretValue, Secrets};
use skein_lib::{Time, Token};
use temper_protocol_records::{self as records, Item};

fn service(settings: settings::Engine) -> crate::Service {
    let endpoints = document_endpoints(&settings);
    let limits = crate::limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    let models = config::core(&settings.jig, [7; 16], 7).settings.models;
    let store = fixtures::memory_store_for(&limits, &settings.jig.limits.store, &models);
    let io = settings.jig.limits.io;
    let mut service =
        crate::Service::prepared(settings, &fixtures::seeds(), None, endpoints, None, io, None, shell()).unwrap();
    service.engine.as_mut().unwrap().store = store;
    service
}
fn secret(engine: &mut Engine, forge: u16, item: Item, value: &[u8]) {
    let request = Secrets::Keep {
        owner: SecretOwner::Application,
        number: records::number(forge, item),
        value: SecretValue::Application(Secret::new(records::encode(item, value).unwrap().into_boxed_slice())),
    };
    assert!(engine.store.secrets(Token::new(123), request).is_none());
    let Some(Event::Kept { .. }) = engine.store.above.pop() else { panic!("fixture secret kept") };
}
fn forgejo(engine: &mut Engine, forge: &Forge) {
    secret(engine, forge.number, Item::ApiToken, b"runtime-api");
    secret(engine, forge.number, Item::GitToken, b"runtime-git");
    secret(engine, forge.number, Item::User, &9_u64.to_le_bytes());
    let origin = records::origin(connection::Kind::Forgejo, &table::settings_forge(forge, &empty_files()).unwrap().api);
    secret(engine, forge.number, Item::Origin, &origin.encode());
}
#[test]
fn the_template_holds_its_forges_only_once_kept() {
    let mut settings = template();
    settings.jig = fixtures::engine().jig;
    fixtures::fill_files(&mut settings);
    let records = settings.seed.clone();
    let mut service = service(settings);
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.workers.forges.is_empty());
    for number in [1, 2, 3] {
        assert!(table::record(engine, number).is_none());
    }
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.workers.forges.len(), 3);
    for record in records {
        assert_eq!(table::record(engine, record.number), Some(&record));
        table::kept(engine, record);
    }
    assert_eq!(engine.workers.forges.len(), 3);
}
#[test]
fn a_forge_kept_after_the_start_connects_from_its_records() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let mut record = fixtures::engine().seed[0].clone();
    record.number = 2;
    record.files = None;
    record.setup = None;
    record.api = r::Address::Socket(([127, 0, 0, 1], 3001).into());
    forgejo(engine, &record);
    let before = engine.forge.starts.len();
    table::kept(engine, record.clone());
    assert_eq!(table::record(engine, 2), Some(&record));
    assert_eq!(engine.forge.starts.len(), before + 1);
    let (number, access) = engine.forge.starts.get(before).unwrap();
    assert_eq!((*number, access.user), (2, 9));
    assert_eq!(
        access.credential,
        skein_forge::credential::Credential::Token(Secret::new(b"runtime-api".as_slice().into()))
    );
    table::kept(engine, record);
    assert_eq!(engine.forge.starts.len(), before + 1);
}
#[test]
fn a_kept_github_forge_takes_its_installations() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let mut record = fixtures::engine().seed[0].clone();
    record.number = 2;
    record.files = None;
    record.kind = temper_engine_domain_forge::Kind::GitHub;
    record.identity.number = 2;
    record.api = r::Address::Socket(([127, 0, 0, 1], 3001).into());
    for (item, value) in [
        (Item::AppId, 7_u64.to_le_bytes().as_slice()),
        (Item::AppSlug, b"fixture-app".as_slice()),
        (Item::AppKey, skein_fake_forge_github::keys::KEYS[0]),
        (Item::Accepted, 1_u64.to_le_bytes().as_slice()),
    ] {
        secret(engine, 2, item, value);
    }
    secret(engine, 2, Item::User, &9_u64.to_le_bytes());
    let origin =
        records::origin(connection::Kind::GitHub, &table::settings_forge(&record, &empty_files()).unwrap().api);
    secret(engine, 2, Item::Origin, &origin.encode());
    table::kept(engine, record.clone());
    assert_eq!(table::record(engine, 2), Some(&record));
    assert_eq!(engine.forge.accepted.get(0), Some(&(2, Box::from([1_u64]))));
    forge::tokens::connected(engine, 2);
    assert!(engine.values.identities.newest(2).is_some());
}
#[test]
fn a_removed_forge_leaves_the_component_the_host_and_every_table() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let mut record = fixtures::engine().seed[0].clone();
    record.number = 2;
    record.files = None;
    record.setup = None;
    record.api = r::Address::Socket(([127, 0, 0, 1], 3001).into());
    forgejo(engine, &record);
    table::kept(engine, record.clone());
    crate::setup::public::reachable(engine, 2, Token::new(999), None, 1);
    table::removed(engine, 2);
    crate::setup::advance(engine);
    assert!(table::record(engine, 2).is_none());
    assert!(engine.workers.forges.iter().all(|forge| forge.number != 2));
    assert!(engine.forge_records.rows.forge(2).is_none());
    assert!(engine.forge.unreachable(2).is_none());
    let value = engine.store.secrets(
        Token::new(123),
        Secrets::Get { owner: SecretOwner::Application, number: records::number(2, Item::ApiToken) },
    );
    let Some(Event::Secret { value: Some(_), .. }) = value else { panic!("removed forge secrets retained") };
    table::kept(engine, record.clone());
    assert_eq!(table::record(engine, 2), Some(&record));
}
#[test]
fn an_unresolvable_record_is_noted_and_not_held() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.forge.shell = Shell::refusing();
    let mut record = fixtures::engine().seed[0].clone();
    record.number = 2;
    record.files = None;
    record.api = r::Address::Socket(([127, 0, 0, 1], 3001).into());
    table::kept(engine, record);
    assert!(table::record(engine, 2).is_none());
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::ForgeNotConnected {
            number: 2,
            why: crate::NotConnected::Unreachable { host: b"forge address".as_slice().into() },
        })
    );
}
#[test]
fn a_document_naming_no_forge_serves_its_clients() {
    let input = fixtures::with(&fixtures::document(), b"application.forges", b"[]");
    let input = fixtures::with(&input, b"application.identities", b"[]");
    let mut service = service(parsed(&input, &fixtures::files()));
    let settings = fixtures::engine();
    let limits = crate::limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    service.engine.as_mut().unwrap().store = fixtures::memory_store_for(
        &limits,
        &settings.jig.limits.store,
        &config::core(&settings.jig, [7; 16], 7).settings.models,
    );
    fixtures::restart(&mut service);
    assert!(service.engine.as_ref().unwrap().workers.forges.is_empty());
    assert!(service.engine.as_ref().unwrap().clients.opened());
    assert_eq!(service.end(), None);
    assert!(!service.work_pending(Time::from_nanos(1)));
    let engine = service.engine.as_mut().unwrap();
    let mut client = fixtures::http::Peer::new(engine, b"GET", b"/jig/v1/stream", &[], None);
    client.pump(engine, true);
    assert!(client.received().starts_with(b"HTTP/1.1 401 "));
}

#[test]
fn a_host_refusal_rolls_back_component_admission() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    let mut record = fixtures::engine().seed[0].clone();
    record.number = 2;
    let bound = engine.setup.env.limits.forges;
    engine.setup.env.limits.forges = 0;
    table::kept(engine, record.clone());
    assert!(table::record(engine, 2).is_none());
    assert!(engine.workers.forges.iter().all(|forge| forge.number != 2));
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::ForgeNotConnected {
            number: 2,
            why: crate::NotConnected::NotHeld { reason: b"Host::add: Unusable Forges".as_slice().into() },
        })
    );
    engine.setup.env.limits.forges = bound;
    table::kept(engine, record.clone());
    assert_eq!(table::record(engine, 2), Some(&record));
}
#[test]
fn a_closed_component_refuses_a_kept_forge_without_holding_it() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    engine.forge.down(skein_forge_connection::Request::Close);
    let mut record = fixtures::engine().seed[0].clone();
    record.number = 2;
    table::kept(engine, record);
    assert!(table::record(engine, 2).is_none());
    assert!(engine.workers.forges.iter().all(|forge| forge.number != 2));
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::ForgeNotConnected {
            number: 2,
            why: crate::NotConnected::NotHeld { reason: b"Component::add: Added::Closed".as_slice().into() },
        })
    );
}
