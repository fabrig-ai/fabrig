//! Restore the kept intent into the service's next Create (contracts, 2).
use super::*;
use crate::tests::fixtures;
use skein_lib::{Duration, Time, Wall};

fn settings() -> crate::settings::Engine {
    let mut config = fixtures::engine();
    config.forges[0].api = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 9).into()));
    config.setup = Box::new([fixtures::owned_setup(1)]);
    config.setup[0].entry.asked = true;
    config.setup[0].entry.orgs = Box::new([]);
    config.records_only = Box::new([1]);
    config.forges[0].api_token = None;
    config.forges[0].user = 0;
    config.identities[0].token = Secret::new(Box::new([]));
    fixtures::fill_files(&mut config);
    fixtures::reseed(&mut config);
    config
}
fn open(mut store: crate::store::Store) -> crate::Service {
    while store.above.pop().is_some() {}
    let config = settings();
    let io = config.jig.limits.io;
    let endpoints = crate::forge::prepare(&config, fixtures::forge_runtime()).unwrap();
    let mut service =
        crate::Service::prepared(config, &fixtures::seeds(), None, endpoints, None, io, Some(256), fixtures::shell())
            .unwrap();
    let limits = service.engine.as_ref().unwrap().env.limits;
    service.engine.as_mut().unwrap().store = fixtures::reopened_store(store, &limits);
    fixtures::restore_forges(&mut service);
    service.engine.as_mut().unwrap().setup.lifecycle.local_mut(1).unwrap().pending = false;
    for _ in 0_u32..4096 {
        fixtures::turn(&mut service);
        if service.engine.as_ref().unwrap().domain.ready() {
            return service;
        }
    }
    panic!("restart ready");
}
fn turn(engine: &mut crate::engine::Engine) {
    crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
}
#[test]
fn an_intent_restored_with_its_forge_is_made_at_the_next_connect() {
    let mut service = open(fixtures::records_store_with(&settings()));
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Forge(Box::new(domain::Event::Connection {
        forge: 1,
        state: domain::Connection::NotSetUp,
    })));
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        request: Token::new(91),
        token: Token::new(92),
        retention: Duration::from_secs(60),
        role: None,
        carried: None,
        ask: domain::Ask::SetUp {
            org: b"kept-org".as_slice().into(),
            repository: b"kept-repository".as_slice().into(),
            project: domain::SetUpProject::Existing(1),
            playground: false,
        },
    })));
    for _ in 0_u32..256 {
        turn(engine);
        if engine.box_link.write.is_some() {
            break;
        }
    }
    assert!(engine.box_link.write.is_some(), "the setup decision was committed before its start line");
    // Reopen only the durable maps, discarding both copies of the volatile intent.
    let store = core::mem::replace(&mut engine.store, fixtures::records_store());
    let mut restarted = open(store);
    let engine = restarted.engine.as_mut().unwrap();
    let first = engine.setup.lifecycle.local(1).unwrap().first.as_ref().expect("restored service intent");
    assert_eq!(first.org.as_ref(), b"kept-org");
    assert_eq!(first.repository.as_ref(), b"kept-repository");
    assert_eq!(engine.domain.forges().rows[0].first.as_ref().unwrap().state, domain::FirstState::Making);
    // At the next connect, Origin's kept answer advances Reconcile into Create.
    let run = next_run(&mut engine.setup.lifecycle);
    engine
        .setup
        .runs
        .insert(
            run,
            Run {
                first: None,
                previous: domain::Connection::NotConnected,
                forge: 1,
                kind: Kind::LocalReconcile,
                internal: true,
            },
        )
        .unwrap();
    kept(engine, Purpose::LocalOrigin(run));
    let create = engine.setup.runs.first().unwrap().1;
    assert_eq!(create.kind, Kind::LocalCreate);
    let first = create.first.as_ref().expect("Create holds its plan's names");
    assert_eq!(first.org.as_ref(), b"kept-org");
    assert_eq!(first.repository.as_ref(), b"kept-repository");
    assert!(engine.setup.lifecycle.local(1).unwrap().first.is_none());
    for event in &engine.setup.above {
        let host::Event::Stop { .. } = event else { continue };
        panic!("Create plan refused: {event:?}");
    }
}

#[test]
fn a_managed_connect_creates_the_restored_intent_with_its_runner_label() {
    let mut service = open(fixtures::records_store_with(&settings()));
    let engine = service.engine.as_mut().unwrap();
    let mut record = crate::forge::table::record(engine, 1).unwrap().clone();
    record.first = Some(domain::record::FirstIntent {
        org: b"managed-org".as_slice().into(),
        repository: b"managed-repository".as_slice().into(),
        project: domain::SetUpProject::Existing(1),
        label: b"docker".as_slice().into(),
    });
    restored_intent(engine, &record);
    let run = next_run(&mut engine.setup.lifecycle);
    engine
        .setup
        .runs
        .insert(
            run,
            Run {
                first: None,
                previous: domain::Connection::NotConnected,
                forge: 1,
                kind: Kind::AdminReconcile,
                internal: false,
            },
        )
        .unwrap();
    admin_create(engine, run);
    let create = engine.setup.runs.get(&run).unwrap();
    assert_eq!(create.kind, Kind::AdminCreate);
    let first = create.first.as_ref().expect("restored intent enters the managed Create plan");
    assert_eq!(first.org.as_ref(), b"managed-org");
    assert_eq!(first.repository.as_ref(), b"managed-repository");
    assert_eq!(first.label.as_ref(), b"docker");
    assert!(engine.setup.lifecycle.local(1).unwrap().first.is_none());
    for event in &engine.setup.above {
        let host::Event::Stop { .. } = event else { continue };
        panic!("Create plan refused: {event:?}");
    }
}

#[test]
fn a_file_backed_forge_with_a_kept_intent_uses_the_local_create_sequence() {
    let mut service = open(fixtures::records_store_with(&settings()));
    let engine = service.engine.as_mut().unwrap();
    let mut record = crate::forge::table::record(engine, 1).unwrap().clone();
    record.files.clone_from(&fixtures::engine().seed[0].files);
    assert!(record.files.is_some());
    let first = domain::record::FirstIntent {
        org: b"file-org".as_slice().into(),
        repository: b"file-repository".as_slice().into(),
        project: domain::SetUpProject::Existing(1),
        label: b"temper-host".as_slice().into(),
    };
    record.first = Some(first.clone());
    crate::forge::table::kept(engine, record.clone());
    assert_eq!(crate::forge::table::record(engine, 1), Some(&record), "retain the whole record with its file inputs");
    let local = engine.setup.lifecycle.local(1).expect("a kept intent needs the local lifecycle");
    let intent = local.first.as_ref().expect("first intent survives file inputs");
    assert_eq!(intent.org, first.org);
    assert_eq!(intent.repository, first.repository);
    assert_eq!(intent.label, first.label);
}
