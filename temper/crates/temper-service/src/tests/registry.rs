//! Runtime registration at the root's service boundary (draft playground-reset.md, 3).
use super::fixtures;
use crate::{Note, route};
use skein_lib::{Secret, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge::{self as forge, Api, ApiTransport, Kind, Mode};
use temper_protocol_records::{self as records, Item};

fn record(number: u16, kind: Kind) -> forge::ForgeRecord {
    fixtures::record(
        number,
        kind,
        Mode::Connected,
        Api { host: b"127.0.0.1".as_slice().into(), port: 3003, transport: ApiTransport::Plaintext },
    )
}
fn register(engine: &mut crate::engine::Engine, record: forge::ForgeRecord) {
    route::request(engine, root::Request::Register(record));
}
fn call(engine: &mut crate::engine::Engine, forge: u16) -> skein_forge_connection::Answered {
    engine.forge.down(skein_forge_connection::Request::Call {
        call: Token::new(91),
        repository: skein_forge::api::Repository { forge, repository: 1 },
        op: skein_forge::api::Op::Read(skein_forge::api::Read::Settings),
    });
    for _ in 0..engine.forge.above.len() {
        if let Some(skein_forge_connection::Event::Answered(answer)) = engine.forge.above.pop() {
            return answer;
        }
    }
    panic!("terminal call answer")
}
#[test]
fn a_registered_forgejo_record_is_held_by_the_connection_and_read_by_the_documents() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().unwrap();
    register(engine, record(3, Kind::Forgejo));
    assert!(engine.forge.component.start(3).is_some());
    assert_eq!(call(engine, 3).result, Err(skein_forge::api::Error::NotConnected));
    let mut repository = fixtures::repository(7, b"acme", b"shop");
    repository.provider.forge = 3;
    let assignment = fixtures::assignment_with(repository);
    let items = crate::translate::items(
        &assignment,
        engine.workers.forges.as_slice(),
        &engine.values.tokens,
        &fixtures::wire_limits(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(items.repositories().get(0).unwrap().remote(), b"https://registered.example/acme/shop.git");
    let document = forge::Document::Landing(forge::LandingView {
        task: 61,
        project: 1,
        phase: forge::LandingPhase::Waiting,
        preparation: 0,
        stale_proofs: 0,
        turn: None,
        members: Box::new([forge::LandingMember {
            change: 51,
            forge: 3,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            branch: b"topic".as_slice().into(),
            landing: b"main".as_slice().into(),
            pull: Some(7),
            base: None,
            candidate: None,
            landed: None,
            gates: Box::new([]),
        }]),
    });
    let limits = &temper_client_protocol_wire::v1::CEILINGS;
    let bytes = crate::family::document(document, engine.workers.forges.as_slice(), limits).unwrap();
    let panel = temper_client_protocol_wire::decode_landing_panel(&bytes, limits).unwrap();
    assert_eq!(
        panel.members().get(0).unwrap().pull().as_ref().unwrap().address(),
        b"https://registered.example/acme/shop/pulls/7"
    );
}
#[test]
fn a_registered_record_needing_io_is_held_until_the_host_answers() {
    for tls in [false, true] {
        let mut service = fixtures::started();
        let engine = service.engine.as_mut().unwrap();
        let mut record = record(3, Kind::Forgejo);
        if tls {
            record.api.transport = ApiTransport::Tls {
                server_name: b"forge.example".as_slice().into(),
                trust: Some(b"/data/trust.pem".as_slice().into()),
            };
        } else {
            record.api.host = b"forge.example".as_slice().into();
        }
        let host = record.api.host.clone();
        register(engine, record);
        assert!(engine.forge.component.start(3).is_some());
        assert_eq!(engine.forge.unreachable(3), Some(host.clone()));
        assert_eq!(engine.resolves.len(), 1);
        for note in &engine.notes {
            assert_ne!(
                note,
                &Note::ForgeNotConnected { number: 3, why: crate::NotConnected::Unreachable { host: host.clone() } }
            );
        }
        for request in &engine.forge.io {
            if let skein_io::Request::Connect { addr, .. } = request {
                assert_ne!(*addr, skein_io::Addr::Ip(crate::forge::HELD_ADDRESS));
            }
        }
        route::request(engine, root::Request::Unregister { forge: 3 });
        assert_eq!(engine.forge.unreachable(3), None);
    }
}
#[test]
fn a_registered_github_record_has_its_tokens_entry_and_identity_room() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    register(engine, record(3, Kind::GitHub));
    assert!(engine.values.tokens.contains(3));
    assert!(engine.values.identities.newest(3).is_none());
    let value = engine.values.identities.newest(1).unwrap().1.clone();
    assert_eq!(engine.values.identities.put(3, value), 1);
    assert!(engine.values.identities.newest(1).is_some());
}
#[test]
fn an_update_replaces_the_mode_and_web_and_keeps_the_connection() {
    let mut service = fixtures::started();
    let engine = service.engine.as_mut().unwrap();
    let mut record = record(3, Kind::GitHub);
    register(engine, record.clone());
    engine.forge.down(skein_forge_connection::Request::Connect {
        forge: 3,
        access: skein_forge_connection::Access {
            credential: skein_forge::credential::Credential::App {
                id: 7,
                key: Secret::new(Box::from(skein_fake_forge_github::keys::KEYS[0])),
            },
            user: 9,
            webhook_secret: None,
        },
    });
    let start = engine.forge.component.start(3);
    record.mode = Mode::Managed;
    record.web = Some(b"https://web.example".as_slice().into());
    record.git = b"https://git.example".as_slice().into();
    record.git_program = None;
    record.read_only = true;
    record.bot.name = b"New bot".as_slice().into();
    register(engine, record.clone());
    let configured = engine.workers.forges.get(1).unwrap();
    assert_eq!(configured.mode, Mode::Managed);
    assert_eq!(configured.web, record.web);
    assert_eq!(configured.git, record.git);
    assert_eq!(configured.git_program, None);
    assert!(configured.app_read_only);
    assert_eq!(engine.records.get(&3), Some(&record));
    assert_eq!(engine.forge.component.start(3), start);
    assert_eq!(engine.forge.starts.len(), 1);
}
#[test]
fn an_update_changing_the_api_is_ignored_and_noted() {
    for change in [0_u8, 1, 2] {
        let mut service = fixtures::started();
        let engine = service.engine.as_mut().unwrap();
        let record = record(3, Kind::Forgejo);
        register(engine, record.clone());
        let mut changed = record.clone();
        match change {
            0 => changed.api.port = 3004,
            1 => changed.kind = Kind::GitHub,
            _ => {
                changed.api.transport =
                    ApiTransport::Tls { server_name: b"forge.example".as_slice().into(), trust: None }
            }
        }
        changed.mode = Mode::Managed;
        register(engine, changed);
        assert_eq!(engine.records.get(&3), Some(&record));
        assert_eq!(engine.notes.pop(), Some(Note::ForgeRecordChanged { number: 3 }));
    }
}
#[test]
fn unregister_drops_the_forge_from_every_table() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().unwrap();
    register(engine, record(3, Kind::GitHub));
    engine.values.identities.put(3, engine.values.identities.newest(1).unwrap().1.clone());
    engine.forge_records.rows.push(
        records::number(3, Item::User),
        Secret::new(records::encode(Item::User, &9_u64.to_le_bytes()).unwrap().into_boxed_slice()),
        4096,
    );
    crate::setup::lifecycle::test_unregister(engine, 3);
    assert!(engine.forge.component.start(3).is_none());
    assert!(engine.records.get(&3).is_none());
    assert_eq!(engine.workers.forges.len(), 1);
    assert!(engine.forge_records.rows.forge(3).is_none());
    assert!(engine.values.identities.newest(3).is_none());
    assert!(!engine.values.tokens.contains(3));
    assert_eq!(call(engine, 3).result, Err(skein_forge::api::Error::Unavailable));
    // A late row cannot recreate an unregistered forge's values.
    engine.forge_records.rows.push(records::number(3, Item::User), Secret::new(Box::new([])), 4096);
    assert!(engine.forge_records.rows.forge(3).is_none());
}
#[test]
fn records_restored_before_the_load_connect_a_forgejo_record_from_its_records() {
    let mut previous = fixtures::ready().engine.unwrap();
    let record = record(3, Kind::Forgejo);
    let configured = crate::forges::settings(&record);
    let origin = records::origin(skein_forge_connection::settings::Kind::Forgejo, &configured.api).encode();
    fixtures::commit(
        &mut previous.store,
        2,
        Box::new([root::Write::Save(root::Record::Forge { row: Box::new(forge::Stored::Forge(record)) })]),
    );
    let Some(jig_protocol_store::Event::Committed { .. }) = previous.store.above.pop() else {
        panic!("saved public forge record")
    };
    for (item, bytes) in
        [(Item::Origin, origin.as_ref()), (Item::User, &9_u64.to_le_bytes()), (Item::ApiToken, b"fixture-api-token")]
    {
        let number = records::number(3, item);
        let value = Secret::new(records::encode(item, bytes).unwrap().into_boxed_slice());
        assert!(previous.store.secrets(Token::new(123), fixtures::application_request(number, value)).is_none());
        let Some(jig_protocol_store::Event::Kept { .. }) = previous.store.above.pop() else {
            panic!("saved forge access record")
        };
    }
    let mut service = fixtures::restarting_from(previous);
    let mut registered_before_load = false;
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
        let engine = service.engine.as_ref().unwrap();
        if engine.records.contains_key(&3) && engine.store.forge_loads.is_empty() {
            registered_before_load = true;
        }
        if engine.forge_records.done() {
            break;
        }
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(registered_before_load, "root restoration registers the forge before secrets load");
    assert!(engine.forge_records.done());
    let access = fixtures::forge_start(engine, 3).unwrap();
    assert_eq!(access.user, 9);
    assert_eq!(
        access.credential,
        skein_forge::credential::Credential::Token(Secret::new(b"fixture-api-token".as_slice().into()))
    );
}
#[test]
fn the_document_forges_seed_the_registry_unchanged() {
    let mut settings = fixtures::document_engine();
    let mut second = fixtures::document_forges().into_iter().next().unwrap();
    second.number = 2;
    settings.forges = Box::new([settings.forges.into_iter().next().unwrap(), second]);
    fixtures::fill_files(&mut settings);
    let mut runtime = fixtures::loopback_runtime();
    let mut second = fixtures::loopback_runtime().into_boxed().into_iter().next().unwrap();
    second.number = 2;
    runtime = {
        let mut all = skein_lib::List::with_capacity(2);
        all.push(runtime.into_boxed().into_iter().next().unwrap()).unwrap();
        all.push(second).unwrap();
        all
    };
    let mut service = crate::Service::engine(settings, &fixtures::seeds(), None, runtime).unwrap();
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.workers.forges.len(), 2);
    for (index, number) in [(0, 1), (1, 2)] {
        let before = engine.workers.forges.get(index).unwrap();
        let git = before.git.clone();
        let api = before.api.clone();
        let token = before.api_token.clone();
        let identity = before.identity;
        register(
            engine,
            fixtures::record(
                number,
                Kind::GitHub,
                Mode::Managed,
                Api { host: b"different.example".as_slice().into(), port: 80, transport: ApiTransport::Plaintext },
            ),
        );
        let after = engine.workers.forges.get(index).unwrap();
        assert_eq!(after.number, number);
        assert_eq!(after.git, git);
        assert_eq!(after.api, api);
        assert_eq!(after.api_token, token);
        assert_eq!(after.identity, identity);
        assert_eq!(after.mode, Mode::Managed);
        assert_eq!(after.kind, crate::settings::Kind::Forgejo);
    }
    assert!(engine.records.is_empty());
    assert!(engine.notes.is_empty());
}
