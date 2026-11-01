//! Host answers for held record APIs (temper/docs/design/protocol/setup.md, 8.3; protocol/forge.md, 2.2).
use super::fixtures;
use crate::setup::registry_support as support;
use crate::{ForgeRuntime, NotConnected, Note, Service, route};
use skein_forge_connection::settings::Transport;
use skein_lib::{Secret, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge::{self as domain, Api, ApiTransport, Kind, Mode};
use temper_protocol_records::{self as records, Item};

const FORGE: u16 = 3;
const HOST: &[u8] = b"held.example";
const PORT: u16 = 3003;
// The restore fixture's service loop settles within the same bound as fixtures::restart.
const TURNS: u32 = 256;
// The service fixture's application-record value bound.
const VALUE_BYTES: u32 = 4096;
// Reconcile's initial reads settle within setup's terminal fixture's four advances.
const RECONCILE_STEPS: u32 = 4;

fn record(host: &[u8], transport: ApiTransport) -> domain::ForgeRecord {
    fixtures::record(FORGE, Kind::Forgejo, Mode::Connected, Api { host: host.into(), port: PORT, transport })
}
fn held() -> domain::ForgeRecord {
    record(HOST, ApiTransport::Plaintext)
}
fn register(service: &mut Service, record: domain::ForgeRecord) {
    route::request(service.engine.as_mut().unwrap(), root::Request::Register(record));
}
fn answer() -> ForgeRuntime {
    ForgeRuntime {
        number: FORGE,
        address: ([127, 0, 0, 1], PORT).into(),
        transport: Transport::Plaintext { name: Some(HOST.into()) },
    }
}
fn rows(service: &mut Service, record: &domain::ForgeRecord) {
    let origin =
        records::origin(skein_forge_connection::settings::Kind::Forgejo, &crate::forges::settings(record).api).encode();
    for (item, bytes) in
        [(Item::Origin, origin.as_ref()), (Item::User, &9_u64.to_le_bytes()), (Item::ApiToken, b"test-api")]
    {
        service.engine.as_mut().unwrap().forge_records.rows.push(
            records::number(FORGE, item),
            Secret::new(records::encode(item, bytes).unwrap().into_boxed_slice()),
            VALUE_BYTES,
        );
    }
}
fn unreachable_notes(service: &mut Service) -> u32 {
    let mut count = 0_u32;
    while let Some(note) = service.notes().pop() {
        if let Note::ForgeNotConnected { number, why: NotConnected::Unreachable { host } } = note {
            assert_eq!(number, FORGE);
            assert_eq!(host.as_ref(), HOST);
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
#[test]
fn a_held_record_is_asked_once_and_joins_both_layers_when_resolved() {
    let mut service = fixtures::ready();
    register(&mut service, held());
    rows(&mut service, &held());
    let resolve = service.take_resolve().unwrap();
    assert_eq!((resolve.forge, resolve.host.as_ref(), resolve.port), (FORGE, HOST, PORT));
    assert!(resolve.tls.is_none());
    assert!(service.take_resolve().is_none());
    register(&mut service, held());
    assert!(service.take_resolve().is_none(), "an outstanding request is asked once");
    service.resolved(FORGE, Ok(answer()));
    assert!(support::given(&service, FORGE).is_some());
    assert_eq!(support::unserved(&service, FORGE), None);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.forge.unreachable(FORGE), None);
    let (number, access) = engine.forge.starts.get(0).unwrap();
    assert_eq!((*number, access.user), (FORGE, 9));
    assert_eq!(
        access.credential,
        skein_forge::credential::Credential::Token(Secret::new(b"test-api".as_slice().into()))
    );
    let mut connected = false;
    for request in &engine.forge.io {
        if let skein_io::Request::Connect { addr, .. } = request {
            assert_eq!(*addr, skein_io::Addr::Ip(answer().address));
            connected = true;
        }
    }
    assert!(connected);
    service.resolved(FORGE, Ok(answer()));
    register(&mut service, held());
    assert_eq!(service.engine.as_ref().unwrap().forge.starts.len(), 1);
    assert!(support::given(&service, FORGE).is_some(), "updates retain the resolved endpoint");
    assert!(service.take_resolve().is_none());
    assert_eq!(unreachable_notes(&mut service), 0);
}
#[test]
fn a_refused_resolve_keeps_it_held_and_names_it_once() {
    let mut service = fixtures::started();
    register(&mut service, held());
    service.take_resolve().unwrap();
    service.resolved(FORGE, Err(b"lookup refused".as_slice().into()));
    service.resolved(FORGE, Err(b"duplicate answer".as_slice().into()));
    for _ in 0..TURNS {
        fixtures::turn(&mut service);
        if service.engine.as_ref().unwrap().forge_records.done() {
            break;
        }
    }
    assert!(service.engine.as_ref().unwrap().forge_records.done());
    assert_eq!(service.engine.as_ref().unwrap().forge.unreachable(FORGE).as_deref(), Some(HOST));
    assert!(support::given(&service, FORGE).is_none());
    assert!(service.take_resolve().is_none());
    assert_eq!(unreachable_notes(&mut service), 1);
    register(&mut service, held());
    assert_eq!(service.take_resolve().unwrap().forge, FORGE, "an update retries a refusal");
    service.resolved(FORGE, Err(b"refused again".as_slice().into()));
    assert_eq!(unreachable_notes(&mut service), 1);
}
#[test]
fn an_answer_for_a_forge_not_held_is_ignored() {
    let mut service = fixtures::ready();
    service.resolved(FORGE, Ok(answer()));
    service.resolved(FORGE, Err(b"late".as_slice().into()));
    assert!(service.engine.as_ref().unwrap().forge.component.start(FORGE).is_none());
    register(&mut service, record(b"127.0.0.1", ApiTransport::Plaintext));
    service.resolved(FORGE, Ok(answer()));
    service.resolved(FORGE, Err(b"late".as_slice().into()));
    assert!(service.engine.as_ref().unwrap().forge.endpoint(FORGE).is_none());
    assert_eq!(unreachable_notes(&mut service), 0);
    route::request(service.engine.as_mut().unwrap(), root::Request::Unregister { forge: FORGE });
    register(&mut service, held());
    route::request(service.engine.as_mut().unwrap(), root::Request::Unregister { forge: FORGE });
    assert!(service.take_resolve().is_none(), "unregister drops queued requests");
    service.resolved(FORGE, Ok(answer()));
    assert!(service.engine.as_ref().unwrap().forge.component.start(FORGE).is_none());
}
#[test]
fn the_loader_waits_for_an_outstanding_resolve_then_connects_from_records() {
    let mut previous = fixtures::ready().engine.unwrap();
    let record = held();
    fixtures::commit(
        &mut previous.store,
        2,
        Box::new([root::Write::Save(root::Record::Forge { row: Box::new(domain::Stored::Forge(record.clone())) })]),
    );
    let Some(jig_protocol_store::Event::Committed { .. }) = previous.store.above.pop() else { panic!("durable forge") };
    let origin =
        records::origin(skein_forge_connection::settings::Kind::Forgejo, &crate::forges::settings(&record).api)
            .encode();
    for (item, bytes) in
        [(Item::Origin, origin.as_ref()), (Item::User, &9_u64.to_le_bytes()), (Item::ApiToken, b"test-api")]
    {
        let value = Secret::new(records::encode(item, bytes).unwrap().into_boxed_slice());
        let request = fixtures::application_request(records::number(FORGE, item), value);
        assert!(previous.store.secrets(Token::new(123), request).is_none());
        let Some(jig_protocol_store::Event::Kept { .. }) = previous.store.above.pop() else { panic!("durable access") };
    }
    let mut service = fixtures::restarting_from(previous);
    let mut requested = false;
    for _ in 0..TURNS {
        fixtures::turn(&mut service);
        if service.take_resolve().is_some() {
            requested = true;
            break;
        }
    }
    assert!(requested);
    for _ in 0..TURNS {
        fixtures::turn(&mut service);
    }
    assert!(!service.engine.as_ref().unwrap().forge_records.done());
    assert!(fixtures::forge_start(service.engine.as_ref().unwrap(), FORGE).is_none());
    assert!(service.take_resolve().is_none());
    service.resolved(FORGE, Ok(answer()));
    assert!(
        fixtures::forge_start(service.engine.as_ref().unwrap(), FORGE).is_none(),
        "the loader owns the initial connect"
    );
    for _ in 0..TURNS {
        fixtures::turn(&mut service);
        if service.engine.as_ref().unwrap().forge_records.done() {
            break;
        }
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge_records.done());
    assert_eq!(fixtures::forge_start(engine, FORGE).unwrap().user, 9);
    assert_eq!(unreachable_notes(&mut service), 0);
}
#[test]
fn an_ip_literal_with_tls_is_asked_and_a_plaintext_one_is_not() {
    let mut service = fixtures::started();
    let tls = ApiTransport::Tls {
        server_name: b"forge.example".as_slice().into(),
        trust: Some(b"/data/trust.der".as_slice().into()),
    };
    register(&mut service, record(b"127.0.0.1", tls));
    let resolve = service.take_resolve().unwrap();
    assert_eq!(resolve.host.as_ref(), b"127.0.0.1");
    let tls = resolve.tls.unwrap();
    assert_eq!(tls.server_name.as_ref(), b"forge.example");
    assert_eq!(tls.trust.as_deref(), Some(b"/data/trust.der".as_slice()));
    assert!(service.take_resolve().is_none());
    let mut plaintext = record(b"127.0.0.1", ApiTransport::Plaintext);
    plaintext.number = 4;
    register(&mut service, plaintext);
    assert!(service.take_resolve().is_none());
    assert_eq!(service.engine.as_ref().unwrap().forge.unreachable(4), None);
}

#[test]
fn a_local_reconcile_waits_for_resolution_and_uses_the_resolved_endpoint() {
    let mut service = fixtures::ready();
    let mut record = held();
    record.mode = Mode::Managed;
    record.setup = Some(br#"{"bot":{"login":"temper","email":"temper@localhost"},"mode":"managed"}"#.as_slice().into());
    register(&mut service, record.clone());
    rows(&mut service, &record);
    service.engine.as_mut().unwrap().forge_records.rows.push(
        records::number(FORGE, Item::AdminToken),
        Secret::new(records::encode(Item::AdminToken, b"test-admin").unwrap().into_boxed_slice()),
        VALUE_BYTES,
    );
    service.take_resolve().unwrap();
    crate::setup::advance(service.engine.as_mut().unwrap());
    assert_eq!(support::runs(&service), 0);
    service.resolved(FORGE, Ok(answer()));
    assert!(support::local(&service, FORGE).pending);
    // Reconcile reads its staged records before it opens the setup stream.
    for _ in 0..RECONCILE_STEPS {
        crate::setup::advance(service.engine.as_mut().unwrap());
    }
    support::reconcile(&service, FORGE);
    let mut connected = false;
    for request in &service.engine.as_ref().unwrap().setup.io {
        if let skein_io::Request::Connect { addr, .. } = request {
            assert_eq!(*addr, skein_io::Addr::Ip(answer().address));
            connected = true;
        }
    }
    assert!(connected, "the setup host uses the resolved address");
}
