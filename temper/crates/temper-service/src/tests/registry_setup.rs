//! Registered records use the document setup host and local lifecycle (temper/docs/design/protocol/setup.md, 5.1, 6.5).
use super::fixtures;
use crate::setup::{
    self,
    lifecycle::{Source, Trigger},
    registry_support as support,
    tests::{self as fixture, Face},
};
use crate::{route, settings};
use skein_forge_setup::{github::manifest::Target, host};
use skein_lib::{Secret, Time, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge::{self as domain, Api, ApiTransport, Kind, Mode};
use temper_protocol_records::{self as records, Item};

const OWNED: &[u8] = br#"{"bot":{"login":"temper","email":"temper@example.test"},"mode":{"owned":{"forgejo":"/bin/forgejo","runner":null,"listen":{"socket":"127.0.0.1:9"},"person":{"login":"owner","email":"owner@example.test"}}}}"#;
const FAKE: &[u8] = br#"{"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{"listen":{"socket":"127.0.0.1:9"},"git":"demo/bin/git","owner":"playground","repositories":[{"name":"widget","landing":"main"}]}}}"#;
const MANAGED: &[u8] = br#"{"bot":{"login":"temper","email":"temper@localhost"},"mode":"managed"}"#;

#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
fn record(kind: Kind, mode: Mode, setup: Option<&[u8]>) -> domain::ForgeRecord {
    let mut record = fixtures::record(
        1,
        kind,
        mode,
        Api { host: b"127.0.0.1".as_slice().into(), port: 9, transport: ApiTransport::Plaintext },
    );
    record.setup = match setup {
        Some(text) => Some(Box::from(text)),
        None => None,
    };
    record
}
fn config() -> settings::Engine {
    fixture::local_config(true, false).engine
}
fn service() -> crate::Service {
    let mut service = support::setup_service(config(), fixtures::ready().engine.unwrap().store);
    fixture::restore_local(&mut service);
    assert!(service.engine.as_ref().unwrap().setup.startup_restored());
    service
}
fn register(service: &mut crate::Service, record: domain::ForgeRecord) {
    let engine = service.engine.as_mut().unwrap();
    // The root keeps its record before publishing Register at this boundary.
    engine.step(root::Event::Forge(Box::new(domain::Event::Restore { record: domain::Stored::Forge(record.clone()) })));
    route::request(engine, root::Request::Register(record));
}
fn row(service: &mut crate::Service, item: Item, bytes: &[u8]) {
    service.engine.as_mut().unwrap().forge_records.rows.push(
        records::number(1, item),
        Secret::new(records::encode(item, bytes).unwrap().into_boxed_slice()),
        4096,
    );
}
fn start(service: &mut crate::Service, run: u64, plan: host::Plan) {
    fixture::start(service, run, plan);
}
fn advance(service: &mut crate::Service) {
    setup::advance(service.engine.as_mut().unwrap());
}
#[test]
fn a_registered_forgejo_without_setup_connects_as_its_recorded_bot() {
    let mut service = service();
    let mut record = record(Kind::Forgejo, Mode::Connected, None);
    record.bot.login = b"recorded-bot".as_slice().into();
    register(&mut service, record);
    row(&mut service, Item::User, &42_u64.to_le_bytes());
    let engine = service.engine.as_mut().unwrap();
    engine.events.clear();
    setup::lifecycle::connected(engine, 1);
    let Some(root::Event::Forge(event)) = engine.events.pop() else { panic!("connection event") };
    assert!(engine.events.is_empty());
    assert_eq!(
        event.as_ref(),
        &domain::Event::Connection {
            forge: 1,
            state: domain::Connection::Connected { id: 42, login: b"recorded-bot".as_slice().into() },
        }
    );
}
fn restored_forgejo() -> crate::Service {
    let mut service = service();
    let record = record(Kind::Forgejo, Mode::Connected, None);
    let origin =
        records::origin(skein_forge_connection::settings::Kind::Forgejo, &crate::forges::settings(&record).api)
            .encode();
    register(&mut service, record);
    row(&mut service, Item::Origin, &origin);
    row(&mut service, Item::ApiToken, b"test-api");
    row(&mut service, Item::User, &42_u64.to_le_bytes());
    service
}
#[test]
fn a_registered_forgejo_without_setup_connects_with_its_recorded_webhook_secret() {
    let mut service = restored_forgejo();
    row(&mut service, Item::WebhookSecret, b"test-webhook");
    let engine = service.engine.as_mut().unwrap();
    crate::forge::records::connect(engine, 1);
    let [(forge, access)] = engine.forge.starts.as_slice() else { panic!("one restored connection") };
    assert_eq!((*forge, access.user), (1, 42));
    assert_eq!(access.webhook_secret, Some(Secret::new(b"test-webhook".as_slice().into())));
}
#[test]
fn a_registered_forgejo_without_setup_publishes_its_recorded_git_identity() {
    let mut service = restored_forgejo();
    row(&mut service, Item::GitToken, b"test-git");
    let engine = service.engine.as_mut().unwrap();
    engine.events.clear();
    assert!(engine.values.identities.newest(1).is_none());
    crate::forge::records::connect(engine, 1);
    let Some(root::Event::Credential { number, generation, valid }) = engine.events.pop() else {
        panic!("recorded git credential")
    };
    assert_eq!((number, generation, valid), (1, 1, skein_lib::Duration::from_nanos(u64::MAX)));
    assert!(engine.events.is_empty(), "one credential publication");
    let (kept_generation, value) = engine.values.identities.newest(number).unwrap();
    assert_eq!(kept_generation, generation);
    let identity = temper_protocol_worker_wire::v1::Identity::decode(
        &engine.workers.encoding.limits,
        &mut skein_lib::Reader::new(value.expose()),
    )
    .unwrap();
    let record = engine.records.get(&1).unwrap();
    assert_eq!(identity.login(), record.bot.login.as_ref());
    assert_eq!(identity.name(), record.bot.name.as_ref());
    assert_eq!(identity.email(), record.bot.email.as_ref());
    let temper_protocol_worker_wire::v1::Kind::Forgejo(token) = identity.kind() else { panic!("Forgejo git token") };
    assert_eq!(token.token().expose(), b"test-git");
    let [(forge, access)] = engine.forge.starts.as_slice() else { panic!("one restored connection") };
    assert_eq!((*forge, access.user), (1, 42));
    assert!(access.webhook_secret.is_none());
}
#[test]
fn a_registered_owned_record_is_served_and_waits_for_its_group() {
    let mut hooks = config();
    hooks.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 0).into()) });
    let mut service = support::setup_service(hooks, fixtures::ready().engine.unwrap().store);
    fixture::restore_local(&mut service);
    let bound = ([127, 0, 0, 1], 41733).into();
    support::bound(&mut service, bound);
    let record = record(Kind::Forgejo, Mode::Owned, Some(OWNED));
    register(&mut service, record.clone());
    let Some(host::Kind::Forgejo(given)) = support::given(&service, 1) else { panic!("registered Forgejo") };
    assert_eq!(given.listener, Some(bound));
    assert_eq!(given.bot_login.as_ref(), b"temper");
    assert_eq!(given.bot_email.as_ref(), b"temper@example.test");
    assert_eq!(given.org_owner.as_deref(), Some(b"owner".as_slice()));
    assert_eq!(given.limits.repositories, 3);
    let local = support::local(&service, 1);
    assert_eq!(local.source, Source::Prepared);
    assert_eq!(local.trigger, Trigger::StartAsked);
    assert!(local.restored);
    assert!(!local.pending);
    assert_eq!(local.path.as_ref(), b"/data/forge-1/admin_token");
    assert_eq!(local.identity.number, 1);
    assert_eq!(local.identity.author.name, record.bot.name);
    assert_eq!(local.identity.author.email, record.bot.email);
    let engine = service.engine.as_mut().unwrap();
    let mut not_set_up = false;
    for event in &engine.events {
        if let root::Event::Forge(event) = event
            && let domain::Event::Connection { forge: 1, state: domain::Connection::NotSetUp } = event.as_ref()
        {
            not_set_up = true;
        }
    }
    assert!(not_set_up);
    engine.step(root::Event::Forge(Box::new(domain::Event::Connection {
        forge: 1,
        state: domain::Connection::NotSetUp,
    })));
    let mut offer = None;
    for forge in &engine.domain.forges().rows {
        if forge.number == 1 {
            offer = Some(forge.offer);
        }
    }
    assert_eq!(offer, Some(domain::Offer::Offered));
    for _ in 0_u32..4 {
        advance(&mut service);
    }
    assert_eq!(support::runs(&service), 0);
    assert!(service.take_handover_read().is_none());
    let mut updated = record;
    updated.bot.name = b"New author".as_slice().into();
    updated.bot.login = b"new-bot".as_slice().into();
    register(&mut service, updated.clone());
    let local = support::local(&service, 1);
    assert!(!local.pending);
    assert_eq!(local.bot_login, updated.bot.login);
    assert_eq!(local.identity.author.name, updated.bot.name);
    assert_eq!(support::runs(&service), 0);
    let mut connected = fixtures::record(
        3,
        Kind::Forgejo,
        Mode::Connected,
        Api { host: b"127.0.0.1".as_slice().into(), port: 3003, transport: ApiTransport::Plaintext },
    );
    connected.bot.login = b"another-bot".as_slice().into();
    connected.bot.email = b"another@example.test".as_slice().into();
    register(&mut service, connected.clone());
    let Some(host::Kind::Forgejo(given)) = support::given(&service, 3) else { panic!("connected Given") };
    assert_eq!(given.bot_login, connected.bot.login);
    assert_eq!(given.bot_email, connected.bot.email);
    registered_owned_setup(&mut service);
}

fn registered_owned_setup(service: &mut crate::Service) {
    let request = Token::new(91);
    let other = Token::new(92);
    for token in [request, other] {
        route::request(
            service.engine.as_mut().unwrap(),
            root::Request::SetUp {
                forge: 1,
                request: token,
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
                playground: false,
            },
        );
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.box_link.write.is_none(), "start waits for the group write");
    let Some(root::Event::Forge(event)) = engine.events.iter().last() else { panic!("second setup answered") };
    assert_eq!(
        **event,
        domain::Event::SetUpAnswered {
            forge: 1,
            request: other,
            answer: domain::SetUpAnswer::Refused(domain::SetUpRefusal::Busy),
        }
    );
    let write = service.take_group_write().expect("registered entry written before start");
    assert_eq!(write.forge, 1);
    assert_eq!(write.text, temper_forge_setup::setup::group_settings(OWNED, 1).unwrap());
    assert!(service.take_group_write().is_none(), "no duplicate group write");
    service.group_written(1, Ok(()));
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.box_link.write.as_deref(), Some(b"start 1\n".as_slice()));
    let Some(root::Event::Forge(event)) = engine.events.iter().last() else { panic!("first setup answered") };
    assert_eq!(**event, domain::Event::SetUpAnswered { forge: 1, request, answer: domain::SetUpAnswer::SettingUp });
}
#[test]
fn a_registered_owned_record_reconciles_after_its_group_started() {
    let mut service = service();
    register(&mut service, record(Kind::Forgejo, Mode::Owned, Some(OWNED)));
    service.engine.as_mut().unwrap().step(root::Event::Forge(Box::new(domain::Event::Restore {
        record: domain::Stored::Repository(fixture::local_repository()),
    })));
    row(&mut service, Item::ApiToken, b"test-api");
    row(&mut service, Item::GitToken, b"test-git");
    row(&mut service, Item::User, &42_u64.to_le_bytes());
    let read = support::handover(&mut service);
    assert_eq!(read.path.as_ref(), b"/data/forge-1/admin_token");
    service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
    advance(&mut service);
    let mut face = Face::new();
    let remove = fixture::takeover_done(&mut service, &mut face);
    service.handover_removed(1, remove.run, Ok(()));
    advance(&mut service);
    support::reconcile(&service, 1);
    fixture::finish_local(&mut service, &mut face);
    assert_eq!(support::runs(&service), 0);
    assert_eq!(service.engine.as_ref().unwrap().forge.starts.get(0).unwrap().1.user, 42);
}
#[test]
fn a_record_restored_with_records_reconciles_at_start() {
    let record = record(Kind::Forgejo, Mode::Owned, Some(OWNED));
    let mut store = fixture::local_store(false);
    fixtures::commit(
        &mut store,
        3,
        Box::new([root::Write::Save(root::Record::Forge { row: Box::new(domain::Stored::Forge(record.clone())) })]),
    );
    let Some(jig_protocol_store::Event::Committed { .. }) = store.above.pop() else { panic!("durable forge") };
    fixture::kept_record(&mut store, 1, Item::AdminToken, b"test-admin");
    let origin =
        records::origin(skein_forge_connection::settings::Kind::Forgejo, &crate::forges::settings(&record).api)
            .encode();
    fixture::kept_record(&mut store, 1, Item::Origin, &origin);
    let mut service = support::setup_service(config(), store);
    route::request(service.engine.as_mut().unwrap(), root::Request::Register(record));
    assert!(support::pending(&service));
    assert!(support::local(&service, 1).pending);
    assert!(!support::local(&service, 1).restored);
    fixture::restore_local(&mut service);
    support::reconcile(&service, 1);
    let mut face = Face::new();
    fixture::local_sent(&mut service, &mut face, b"GET /api/v1/user ");
    assert!(fixture::contains(&face.wire.received, b"test-admin"));
    assert!(service.take_handover_read().is_none());
}
#[test]
fn a_registered_fake_record_is_a_playground_and_waits_for_its_start() {
    let mut service = service();
    let mut record = record(Kind::Forgejo, Mode::Fake, Some(FAKE));
    record.playground = true;
    register(&mut service, record);
    let local = support::local(&service, 1);
    assert_eq!(local.source, Source::Prepared);
    assert_eq!(local.trigger, Trigger::StartDemo);
    assert_eq!(local.repositories, 1);
    assert!(!local.pending);
    for _ in 0_u32..4 {
        advance(&mut service);
    }
    assert_eq!(support::runs(&service), 0);
    assert!(service.take_handover_read().is_none());
    let mut not_set_up = false;
    for event in &service.engine.as_ref().unwrap().events {
        if let root::Event::Forge(event) = event
            && let domain::Event::Connection { forge: 1, state: domain::Connection::NotSetUp } = event.as_ref()
        {
            not_set_up = true;
        }
    }
    assert!(not_set_up);
    let _read = support::handover(&mut service);
    assert_eq!(support::local(&service, 1).trigger, Trigger::Asked);
}
#[test]
fn a_registered_managed_record_admin_connects() {
    let mut service = service();
    register(&mut service, record(Kind::Forgejo, Mode::Managed, Some(MANAGED)));
    let local = support::local(&service, 1);
    assert_eq!(local.source, Source::Managed);
    assert_eq!(local.trigger, Trigger::Start);
    let value = records::encode_pasted(1, b"test-admin").unwrap();
    let digest = skein_lib::sha256::digest(value.expose());
    let engine = service.engine.as_mut().unwrap();
    engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
    let Some(jig_protocol_store::Event::Carried { number, .. }) = engine.store.above.pop() else {
        panic!("carried admin")
    };
    route::request(
        engine,
        root::Request::Connect {
            forge: 1,
            request: Token::new(91),
            how: domain::How::ForgejoAdmin { first: None },
            carried: jig_domain_core::Carried { number, digest },
        },
    );
    let mut face = Face::new();
    fixture::local_sent(&mut service, &mut face, b"GET /api/v1/user ");
    face.reply(&mut service, br#"{"id":1,"login":"owner","is_admin":true}"#);
    fixture::local_sent(&mut service, &mut face, b"GET /api/v1/users/temper ");
    let Some(host::Kind::Forgejo(given)) = support::given(&service, 1) else { panic!("managed Given") };
    assert_eq!(given.bot_email.as_ref(), b"temper@localhost");
    assert_eq!(given.listener, None);
    assert!(service.take_handover_read().is_none());
}
#[test]
fn a_registered_github_record_runs_a_manifest() {
    let mut service = service();
    register(&mut service, record(Kind::GitHub, Mode::Managed, None));
    start(&mut service, 91, host::Plan::Manifest { forge: 1, target: Target::Personal });
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Manifest { state }, .. } = fixture::terminal(&mut service)
    else {
        panic!("registered manifest")
    };
    let page = service.setup_page(1, &state, Time::ZERO).unwrap();
    assert!(fixture::contains(page, b"/forges/1/github/redirect"));
    assert!(fixture::contains(page, b"https://github.com/settings/apps/new"));
}
#[test]
fn an_update_of_read_only_replaces_the_hosts_github_forge() {
    let mut service = service();
    let mut record = record(Kind::GitHub, Mode::Managed, None);
    register(&mut service, record.clone());
    row(&mut service, Item::AppSlug, b"fixture-app");
    row(&mut service, Item::AppId, &7_u64.to_le_bytes());
    row(&mut service, Item::AppKey, skein_fake_forge_github::keys::KEYS[0]);
    start(&mut service, 93, host::Plan::Reconcile { forge: 1, adopted: Box::new([]) });
    advance(&mut service);
    assert_eq!(support::runs(&service), 1);
    register(&mut service, record.clone());
    assert_eq!(support::runs(&service), 1);
    record.read_only = true;
    record.web = Some(b"https://web.example".as_slice().into());
    register(&mut service, record);
    let root::SetupEnd::Stopped { reason, .. } = fixture::terminal(&mut service) else {
        panic!("Remove ends the old host run")
    };
    assert!(reason.ends_with(b"Down"));
    let Some(host::Kind::GitHub(given)) = support::given(&service, 1) else { panic!("updated GitHub") };
    assert!(given.read_only);
    assert_eq!(given.web.as_ref(), b"https://web.example");
}
#[test]
fn unregister_ends_a_live_run_and_drops_the_forge_from_every_setup_table() {
    let mut service = service();
    register(&mut service, record(Kind::Forgejo, Mode::Owned, Some(OWNED)));
    start(
        &mut service,
        91,
        host::Plan::TakeOver { forge: 1, admin_token: Secret::new(b"test-admin".as_slice().into()) },
    );
    let mut face = Face::new();
    fixture::local_sent(&mut service, &mut face, b"GET /api/v1/user ");
    face.reply(&mut service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
    assert_eq!(support::runs(&service), 1);
    support::seed_tables(&mut service, 1);
    route::request(service.engine.as_mut().unwrap(), root::Request::Unregister { forge: 1 });
    support::removed(&service, 1);
    support::late_write(&mut service);
    advance(&mut service);
    assert_eq!(support::runs(&service), 0);
    assert!(service.engine.as_ref().unwrap().forge_records.rows.forge(1).is_none());
    assert!(support::given(&service, 1).is_none());
}
#[test]
fn a_held_record_is_unserved_unreachable() {
    let mut service = service();
    let mut record = record(Kind::Forgejo, Mode::Owned, Some(OWNED));
    record.api.host = b"held.example".as_slice().into();
    register(&mut service, record);
    assert!(support::given(&service, 1).is_none());
    assert_eq!(support::unserved(&service, 1), Some(b"unreachable".as_slice()));
    assert!(service.engine.as_ref().unwrap().setup.io.is_empty());
    let mut missing = false;
    while let Some(note) = service.notes().pop() {
        if note == (crate::Note::ForgeSetupStopped { number: 1, why: crate::SetupStopped::Missing }) {
            missing = true;
        }
    }
    assert!(missing);
    register(&mut service, record_for_refusal());
    assert!(support::given(&service, 3).is_some());
    support::host_capacity(&mut service, 1);
    let mut refused = record_for_refusal();
    refused.number = 4;
    register(&mut service, refused);
    assert!(support::given(&service, 4).is_none());
    assert_eq!(support::unserved(&service, 4), Some(b"setup host refused forge".as_slice()));
}

fn record_for_refusal() -> domain::ForgeRecord {
    let mut record = record(Kind::Forgejo, Mode::Connected, None);
    record.number = 3;
    record
}
