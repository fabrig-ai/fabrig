//! Record-driven forge runtimes (temper/docs/design/draft/playground-reset.md, 3).
use super::fixtures;
use crate::{Service, forge};
use skein_lib::{List, Secret, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as domain;
use temper_protocol_records::{self as records, Item};

const NUMBER: u16 = 77;
const PORT: u16 = 3076;

fn record(host: &[u8]) -> domain::ForgeRecord {
    domain::ForgeRecord {
        number: NUMBER,
        kind: domain::Kind::Forgejo,
        mode: domain::Mode::Connected,
        state: domain::ForgeState::Kept,
        place: domain::Place {
            api: domain::Address { host: Box::from(host), port: PORT, tls: None },
            git: Box::from(b"http://forge.test:3076".as_slice()),
            web: Some(Box::from(b"http://forge.test:3076".as_slice())),
        },
        read_only: false,
        bot: domain::Bot {
            login: Box::from(b"writer".as_slice()),
            name: Box::from(b"Writer".as_slice()),
            email: Box::from(b"writer@example.test".as_slice()),
        },
        asked: false,
        first: None,
    }
}

#[expect(clippy::disallowed_methods, reason = "ordinary fixtures parse their supplied host")]
fn service(host: &[u8]) -> Service {
    let mut settings = fixtures::engine();
    fixtures::fill_files(&mut settings);
    let mut service = Service::engine(settings, &fixtures::seeds(), None, List::with_capacity(0)).unwrap();
    let mut store = fixtures::records_store();
    fixtures::commit(&mut store, 1, Box::new([root::Write::Erase(root::Key::Forge(domain::Key::Forge(1)))]));
    assert!(store.above.pop().is_some());
    service.engine.as_mut().unwrap().store = store;
    let record = record(host);
    let api = match core::str::from_utf8(host).unwrap().parse::<core::net::IpAddr>() {
        Ok(ip) => jig_protocol_config::Address::Addr(skein_io::Addr::Ip(core::net::SocketAddr::new(ip, PORT))),
        Err(_) => jig_protocol_config::Address::Host { name: Box::from(host), port: PORT },
    };
    let origin = records::origin(skein_forge_connection::settings::Kind::Forgejo, &api).encode();
    for (item, bytes) in [
        (Item::Origin, origin),
        (Item::User, Box::from(42_u64.to_le_bytes())),
        (Item::ApiToken, Box::from(b"api-credential".as_slice())),
        (Item::GitToken, Box::from(b"git-credential".as_slice())),
    ] {
        let value = Secret::new(records::encode(item, &bytes).unwrap().into_boxed_slice());
        let engine = service.engine.as_mut().unwrap();
        assert!(
            engine
                .store
                .secrets(Token::new(123), fixtures::application_request(records::number(NUMBER, item), value))
                .is_none()
        );
        assert!(engine.store.above.pop().is_some());
    }
    let engine = service.engine.as_mut().unwrap();
    fixtures::commit(
        &mut engine.store,
        2,
        Box::new([root::Write::Save(root::Record::Forge { row: Box::new(domain::Stored::Forge(record)) })]),
    );
    assert!(engine.store.above.pop().is_some());
    service
}

fn load(service: &mut Service) {
    let mut face = fixtures::ForgeFace::new();
    face.select(NUMBER);
    for _ in 0_u32..256 {
        face.turn(service);
    }
    assert_eq!(service.end(), None);
    assert!(service.engine.as_ref().unwrap().forge_records.done());
}

#[test]
fn a_kept_record_adds_its_forge_and_connects_from_its_secrets() {
    let mut service = service(b"127.0.0.1");
    load(&mut service);
    let engine = service.engine.as_ref().unwrap();
    let kept = engine.workers.forges.get(NUMBER).unwrap();
    assert_eq!((kept.number, kept.identity, kept.user), (NUMBER, u32::from(NUMBER), 42));
    assert_eq!(kept.mode, domain::Mode::Connected);
    assert!(kept.api_token.is_none());
    assert_eq!(kept.git.as_ref(), b"http://forge.test:3076");
    let (number, access) = engine.forge.starts.get(0).unwrap();
    assert_eq!((*number, access.user), (NUMBER, 42));
    assert_eq!(
        access.credential,
        skein_forge::credential::Credential::Token(Secret::new(Box::from(b"api-credential".as_slice())))
    );
}

#[test]
fn a_record_by_ip_literal_is_reached_at_once() {
    for host in [b"127.0.0.1".as_slice(), b"::1".as_slice()] {
        let mut service = service(host);
        assert!(service.unresolved().is_empty());
        assert!(service.engine.as_ref().unwrap().forge.unreachable(NUMBER).is_none());
        load(&mut service);
        assert_eq!(service.engine.as_ref().unwrap().forge.starts.len(), 1);
    }
}

#[test]
fn a_record_by_host_name_is_held_until_resolved_at_start() {
    let mut service = service(b"forge.test");
    load(&mut service);
    assert_eq!(service.unresolved().as_ref(), &[(NUMBER, Box::from(b"forge.test".as_slice()), PORT)]);
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge.starts.is_empty());
    assert_eq!(
        engine.notes.iter().next(),
        Some(&crate::Note::ForgeNotConnected {
            number: NUMBER,
            why: crate::NotConnected::Unreachable { host: Box::from(b"forge.test".as_slice()) },
        })
    );
    assert!(engine.clients.starting.is_some());
    service.resolved(NUMBER, Some(([127, 0, 0, 1], PORT).into()));
    assert!(service.unresolved().is_empty());
    assert!(service.engine.as_ref().unwrap().forge.unreachable(NUMBER).is_none());
    assert_eq!(service.engine.as_ref().unwrap().forge.starts.len(), 1);
    load(&mut service);
    assert!(service.engine.as_ref().unwrap().forge.serving.is_some());
}

#[test]
fn a_forge_family_row_of_any_number_is_kept_by_the_loader() {
    let settings = fixtures::engine();
    let mut loader = forge::records::Loader::new(&settings.limits);
    for number in [0, NUMBER, u16::MAX] {
        loader.loaded(
            Box::new([jig_protocol_store::SecretRow {
                owner: jig_protocol_store::SecretOwner::Application,
                number: records::number(number, Item::User),
                value: jig_protocol_store::SecretValue::Application(Secret::new(
                    records::encode(Item::User, &42_u64.to_le_bytes()).unwrap().into_boxed_slice(),
                )),
            }]),
            None,
        );
        let rows = loader.rows.forge(number).unwrap();
        assert_eq!(rows.get(Item::User).unwrap(), 42_u64.to_le_bytes());
    }
}

#[test]
fn identities_and_tokens_take_a_forge_kept_at_runtime() {
    let mut service = service(b"127.0.0.1");
    load(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let (generation, value) = engine.values.identities.newest(u32::from(NUMBER)).unwrap();
    assert_eq!(generation, 1);
    let mut reader = skein_lib::Reader::new(value.expose());
    let identity =
        temper_protocol_worker_wire::v1::Identity::decode(&engine.workers.encoding.limits, &mut reader).unwrap();
    assert_eq!(identity.login(), b"writer");
    let temper_protocol_worker_wire::v1::Kind::Forgejo(token) = identity.kind() else { panic!("Forgejo identity") };
    assert_eq!(token.token().expose(), b"git-credential");
    let mut worker = crate::identities::IdentityTable::new(1);
    worker.granted(u32::from(NUMBER), 1, None, fixtures::NOW).unwrap();
    assert!(worker.granted(u32::from(NUMBER) + 1, 1, None, fixtures::NOW).is_err());
    let mut github = record(b"127.0.0.1");
    github.number = NUMBER + 1;
    github.kind = domain::Kind::GitHub;
    let number = github.number;
    let origin =
        records::Origin { kind: skein_forge_connection::settings::Kind::GitHub, api: "127.0.0.1:3076".into() }.encode();
    for (item, bytes) in [
        (Item::Origin, origin),
        (Item::User, Box::from(9_u64.to_le_bytes())),
        (Item::AppId, Box::from(7_u64.to_le_bytes())),
        (Item::AppSlug, Box::from(b"fixture-app".as_slice())),
        (Item::AppKey, Box::from(skein_fake_forge_github::keys::KEYS[0])),
    ] {
        engine.forge_records.rows.push(
            records::number(number, item),
            Secret::new(records::encode(item, &bytes).unwrap().into_boxed_slice()),
            4096,
        );
    }
    crate::route::request(engine, root::Request::ForgeKept(github));
    forge::tokens::token(
        engine,
        number,
        7,
        skein_forge_connection::Account { id: 9, login: Box::from(b"account".as_slice()) },
        Secret::new(Box::from(b"installation-token".as_slice())),
        skein_lib::Wall::from_nanos(u64::MAX),
    );
    forge::tokens::connected(engine, number);
    assert_eq!(forge::tokens::account(&engine.values.tokens, number, b"account"), Some(9));
    assert!(engine.values.identities.newest(u32::from(number)).is_some());
}

#[test]
fn a_failed_resolution_keeps_the_forge_held_and_allows_startup_to_finish() {
    let mut service = service(b"forge.test");
    load(&mut service);
    service.resolved(NUMBER, None);
    load(&mut service);
    assert!(service.unresolved().is_empty());
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge.serving.is_some());
    assert!(engine.forge.starts.is_empty());
    assert!(engine.forge.unreachable(NUMBER).is_some());
    service.resolved(NUMBER, Some(([127, 0, 0, 1], PORT).into()));
    assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
}

#[test]
fn a_host_kept_after_serving_stays_held_until_the_next_start() {
    let mut service = service(b"127.0.0.1");
    load(&mut service);
    let mut later = record(b"later.test");
    later.number = NUMBER + 1;
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::ForgeKept(later));
    assert!(service.unresolved().is_empty());
    let before = service.engine.as_ref().unwrap().forge.starts.len();
    service.resolved(NUMBER + 1, Some(([127, 0, 0, 1], PORT).into()));
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge.unreachable(NUMBER + 1).is_some());
    assert_eq!(engine.forge.starts.len(), before);
}

#[test]
fn a_name_resolved_at_start_serves_a_later_record_on_its_own_port() {
    let mut service = service(b"forge.test");
    load(&mut service);
    service.resolved(NUMBER, Some(([127, 0, 0, 1], PORT).into()));
    load(&mut service);
    let mut later = record(b"forge.test");
    later.number = NUMBER + 1;
    later.place.api.port = PORT + 1;
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::ForgeKept(later));
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.workers.forges.get(NUMBER + 1).is_some());
    assert!(engine.forge.unreachable(NUMBER + 1).is_none());
    assert!(service.unresolved().is_empty());
}

#[test]
fn an_add_refused_by_the_component_drops_the_runtime_and_names_its_host() {
    let mut service = service(b"127.0.0.1");
    let engine = service.engine.as_mut().unwrap();
    engine.forge.env.limits.forges = 1;
    let mut later = record(b"full.test");
    later.number = NUMBER + 1;
    crate::route::request(engine, root::Request::ForgeKept(later));
    assert!(engine.workers.forges.get(NUMBER).is_some());
    assert!(engine.workers.forges.get(NUMBER + 1).is_none());
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::ForgeNotConnected {
            number: NUMBER + 1,
            why: crate::NotConnected::Unreachable { host: Box::from(b"full.test".as_slice()) },
        })
    );
}

fn roots() -> skein_tls::Config {
    let root = skein_lib::base64::decode(
        skein_lib::base64::Alphabet::Standard,
        concat!(
            "MIIBmDCCAT+gAwIBAgIUHvatnWfGKnu6ssb8pgtoqmWxTcQwCgYIKoZIzj0EAwIwGjEYMBYGA1UEAwwPc2tlaW4gdGVzdCByb290",
            "MB4XDTI2MDEwMTAwMDAwMFoXDTM2MDEwMTAwMDAwMFowGjEYMBYGA1UEAwwPc2tlaW4gdGVzdCByb290MFkwEwYHKoZIzj0CAQYI",
            "KoZIzj0DAQcDQgAElnuG+BWgtzb++8N0S1weL+2KuE3Fb9nx992dyCDQ6GJdAwludLQPNm9HNipn55h9kfFjKtdy/de0tsxuavfE",
            "UqNjMGEwHQYDVR0OBBYEFPmq8/JFoN7ZJ8taLdr3/hphqvWjMB8GA1UdIwQYMBaAFPmq8/JFoN7ZJ8taLdr3/hphqvWjMA8GA1Ud",
            "EwEB/wQFMAMBAf8wDgYDVR0PAQH/BAQDAgEGMAoGCCqGSM49BAMCA0cAMEQCIEtpmEfeTnDuXGC/OGm3Q7fM97DmGIwA3AUsfwTn",
            "AaLNAiAdXjbm1OYLV5vW4t68UIk4Kzxdk5qEue0bPNUZ9oISMw==",
        )
        .as_bytes(),
        412,
    )
    .unwrap();
    skein_tls::Config::from_der(core::slice::from_ref(&root), &[]).unwrap().0
}

#[test]
fn a_kept_github_uses_the_machine_endpoint_and_the_records_tls_name() {
    let mut service = service(b"127.0.0.1");
    let roots = roots();
    service.configure_forge_runtime(
        crate::ForgeRuntime {
            number: 0,
            address: ([192, 0, 2, 9], 443).into(),
            transport: skein_forge_connection::settings::Transport::Tls {
                server_name: skein_tls::Name::new("api.github.test").unwrap(),
                trust: roots.clone(),
            },
        },
        roots,
    );
    let mut github = record(b"api.github.test");
    github.number = NUMBER + 1;
    github.kind = domain::Kind::GitHub;
    github.place.api.port = 8443;
    github.place.api.tls = Some(Box::from(b"certificate.github.test".as_slice()));
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::ForgeKept(github));
    let engine = service.engine.as_ref().unwrap();
    let configured = engine.workers.forges.get(NUMBER + 1).unwrap();
    let endpoint = crate::forges::endpoint(engine, configured).unwrap();
    assert_eq!(endpoint.address, skein_io::Addr::Ip(([192, 0, 2, 9], 8443).into()));
    let skein_forge_connection::settings::Transport::Tls { server_name, .. } = endpoint.transport else {
        panic!("TLS endpoint")
    };
    let skein_tls::Server::Dns(name) = server_name.server() else { panic!("DNS name") };
    assert_eq!(name, "certificate.github.test");
    assert!(service.unresolved().is_empty());
}

#[test]
fn plaintext_outside_loopback_is_refused_when_the_record_is_kept() {
    let mut service = service(b"127.0.0.1");
    let mut remote = record(b"192.0.2.7");
    remote.number = NUMBER + 1;
    let pending = service.engine.as_ref().unwrap().forge.io.len();
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::ForgeKept(remote));
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.workers.forges.get(NUMBER + 1).is_none());
    assert!(engine.forge.component.start(NUMBER + 1).is_none());
    assert_eq!(engine.forge.io.len(), pending);
}
