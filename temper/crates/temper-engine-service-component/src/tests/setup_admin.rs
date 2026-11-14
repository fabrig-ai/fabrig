#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "test scans captured diagnostics and inspects closed event variants"
)]
//! A managed Forgejo's paste is held through Reconcile and Create, then kept.
use super::create_tests::Forge;
use super::*;
use crate::setup::lifecycle;
use jig_protocol_store::{SecretOwner, Secrets};

const ADMIN: &[u8] = b"admin-token-placeholder-0123";

fn config() -> fixtures::ConfiguredEngine {
    let mut config = local_config(true, false);
    let entry = &mut config.setup[0];
    entry.entry.mode = temper_forge_setup::setup::Mode::Managed { admin_token: None };
    entry.entry.directory = Box::new([]);
    entry.handover = None;
    config.identities[0].login = Box::new([]);
    config
}
fn store(forge: &Forge) -> crate::store::Store {
    let mut store = fixtures::records_store();
    for (item, bytes) in [(Item::ApiToken, b"test-api".as_slice()), (Item::GitToken, b"test-git")] {
        kept_record(&mut store, 1, item, bytes);
    }
    kept_record(&mut store, 1, Item::User, &forge.service.setup().find(b"temper").unwrap().id.to_le_bytes());
    store
}
fn forge(admin: bool, marked: bool, refuse_file: bool) -> Forge {
    let mut forge = Forge::new(refuse_file);
    let setup = forge.service.setup_mut();
    let user = setup.account(b"elsewhere-owner", admin, b"");
    setup.token(
        user,
        b"pasted",
        &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
        Secret::new(ADMIN.into()),
    );
    if !marked {
        setup.delete_account(b"temper");
        setup.account(b"temper", false, b"a person's account");
    }
    forge
}
fn service(forge: &Forge) -> crate::Service {
    let mut service = local_service(config(), store(forge));
    restore_local(&mut service);
    // Fresh managed records offer Connect instead of staging a handover.
    assert_eq!(service.engine.as_ref().unwrap().domain.forge_connection(1), domain::Connection::NotConnected);
    service
}
fn bundle(item: u16, bytes: &[u8]) -> Secret {
    let value = match item {
        1 => temper_client_domain::forge::forges::pasted_admin_token(bytes),
        _ => records::encode_pasted(&[(item, bytes)]).ok(),
    };
    match value {
        Some(value) => value,
        None => Secret::new(Box::from([0])),
    }
}
fn first(label: &[u8]) -> domain::FirstAsk {
    domain::FirstAsk { org: b"work".as_slice().into(), repository: b"hello".as_slice().into(), label: label.into() }
}
fn ask(service: &mut crate::Service, value: Secret, first: Option<domain::FirstAsk>) {
    let engine = service.engine.as_mut().unwrap();
    let digest = skein_lib::sha256::digest(value.expose());
    engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
    assert!(
        (match engine.store.host.above.pop() {
            Some(StoreEvent::Carried { number: found, .. }) => found == records::CARRIED,
            _ => false,
        })
    );
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        role: Some(jig_domain_people::Role::Owner),
        carried: Some(jig_domain_core::Carried { number: records::CARRIED, digest }),
        request: Token::new(90),
        token: Token::new(91),
        retention: Duration::from_secs(60),
        ask: domain::Ask::Connect { forge: domain::Making::Kept(1), how: domain::How::ForgejoAdmin { first } },
    })));
}
fn scan(engine: &crate::engine::Engine) {
    for note in &engine.notes {
        assert!(!contains(alloc::format!("{note:?}").as_bytes(), ADMIN));
    }
    for event in &engine.events {
        assert!(!contains(alloc::format!("{event:?}").as_bytes(), ADMIN));
    }
    for answer in &engine.setup.answers {
        assert!(!contains(alloc::format!("{answer:?}").as_bytes(), ADMIN));
    }
    assert!(!contains(alloc::format!("{:?}", engine.domain.forges()).as_bytes(), ADMIN));
    for (_, plan) in &engine.setup.staged {
        assert!(
            !(match plan {
                host::Plan::TakeOver { .. } => true,
                _ => false,
            })
        );
    }
    for (_, run) in &engine.setup.runs {
        assert_ne!(run.kind, Kind::LocalTakeOver);
    }
}
fn turn(service: &mut crate::Service, forge: &mut Forge, tick: u64) {
    let engine = service.engine.as_mut().unwrap();
    scan(engine);
    while engine.notes.pop().is_some() {}
    while engine.setup.answers.pop().is_some() {}
    engine.env.now = Time::from_nanos(tick.checked_mul(1_000_000).unwrap());
    advance(engine);
    engine.forge.iterate(engine.env.now, Wall::EPOCH);
    // This peer owns one socket: finish its exchange before serving the other component.
    let setup = match forge.socket {
        Some((_, setup)) => setup,
        None => !engine.setup.io.is_empty() || !engine.setup.runs.is_empty(),
    };
    let deferred =
        core::mem::replace(if setup { &mut engine.forge.io } else { &mut engine.setup.io }, Queue::uncapped());
    forge.turn_io(engine);
    let waiting = if setup { &mut engine.forge.io } else { &mut engine.setup.io };
    assert!(waiting.is_empty());
    *waiting = deferred;
    forge.turn_server(engine);
    for _ in 0_u32..4 {
        scan(engine);
        crate::engine::iterate(engine, engine.env.now, Wall::EPOCH);
    }
    forge.service.reclaim();
}
fn settle(service: &mut crate::Service, forge: &mut Forge) {
    for tick in 1..20_000 {
        turn(service, forge, tick);
        let engine = service.engine.as_ref().unwrap();
        if engine.setup.public.is_empty() && engine.setup.runs.is_empty() && engine.store.erases > 0 {
            scan(engine);
            return;
        }
    }
    panic!("admin connect did not settle: {:?}", service.engine.as_ref().unwrap().domain.forges());
}
fn admin_record(service: &crate::Service) -> Option<&[u8]> {
    service.engine.as_ref().unwrap().forge_records.rows.forge(1)?.get(Item::AdminToken).ok()
}
fn erased(service: &mut crate::Service, connected: bool) {
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.setup.public.is_empty());
    assert!(engine.setup.runs.is_empty());
    assert_eq!(engine.store.erases, 1);
    assert!(
        (match engine
            .store
            .secrets(Token::new(902), Secrets::Get { owner: SecretOwner::Application, number: records::CARRIED })
        {
            Some(StoreEvent::Secret { value: None, .. }) => true,
            _ => false,
        })
    );
    let Some(StoreEvent::SecretsLoaded { rows, .. }) =
        engine.store.secrets(Token::new(903), Secrets::Load { owner: SecretOwner::Application, after: None })
    else {
        panic!("store rows")
    };
    for row in rows {
        let jig_protocol_store::SecretValue::Application(value) = row.value else { panic!("application row") };
        if row.number == records::number(1, Item::AdminToken) && connected {
            assert!(contains(value.expose(), ADMIN));
        } else {
            assert!(!contains(value.expose(), ADMIN));
        }
    }
    if !connected {
        assert!(admin_record(service).is_none());
        assert_eq!(service.engine.as_ref().unwrap().domain.forge_connection(1), domain::Connection::NotConnected);
    }
}
// Keep the two complete service fixtures off the restart story's stack.
fn boxed_admin_service(forge: &Forge) -> Box<crate::Service> {
    Box::new(service(forge))
}
fn boxed_restarted_service(store: crate::store::Store) -> Box<crate::Service> {
    Box::new(local_service(config(), store))
}
#[test]
fn admin_paste_reconciles_creates_with_its_label_then_keeps_and_restarts_from_records() {
    let mut forge = forge(true, true, false);
    forge.service.setup_mut().org(b"work", b"elsewhere-owner");
    let mut service = boxed_admin_service(&forge);
    ask(&mut service, bundle(1, ADMIN), Some(first(b"docker_1.x-y")));
    let mut saw_create = false;
    for tick in 1..20_000 {
        turn(&mut service, &mut forge, tick);
        let engine = service.engine.as_ref().unwrap();
        for (_, run) in &engine.setup.runs {
            if run.kind == Kind::AdminCreate {
                saw_create = true;
                assert!(admin_record(&service).is_none(), "Reconcile alone cannot keep the admin token");
            }
        }
        if engine.setup.public.is_empty()
            && engine.store.erases > 0
            && (match engine.domain.forge_connection(1) {
                domain::Connection::Connected { .. } => true,
                _ => false,
            })
        {
            break;
        }
    }
    assert!(saw_create);
    assert_eq!(admin_record(&service), Some(ADMIN));
    erased(&mut service, true);
    let engine = service.engine.as_ref().unwrap();
    assert!(
        (match engine.domain.forge_connection(1) {
            domain::Connection::Connected { .. } => true,
            _ => false,
        })
    );
    assert_eq!(engine.domain.forges().rows[0].first.as_ref().unwrap().state, domain::FirstState::Made { id: 1 });
    assert!(engine.domain.forges().rows[0].admin);
    assert_eq!(engine.domain.forges().rows[0].first.as_ref().unwrap().label.as_ref(), b"docker_1.x-y");
    let head = forge.git.branch(b"work/hello", b"main").unwrap();
    let object = forge.git.object(head).unwrap();
    assert!(contains(object.tree.get(temper_forge_setup::STARTER_PATH).unwrap(), b"runs-on: docker_1.x-y\n"));
    assert!(forge.service.setup().repository_hooks(1).next().is_none());
    restart_admin_service(service, &mut forge);
}

// Keep extraction of the boxed engine's store off the first construction's frame.
fn restart_admin_service(mut service: Box<crate::Service>, forge: &mut Forge) {
    service.engine.as_mut().unwrap().forge.down(skein_forge_connection::Request::Close);
    for _ in 0_u32..16 {
        forge.turn_io(service.engine.as_mut().unwrap());
        service.engine.as_mut().unwrap().forge.iterate(Time::ZERO, Wall::EPOCH);
    }
    let store = service.engine.take().unwrap().store;
    let mut restarted = boxed_restarted_service(store);
    restore_local(&mut restarted);
    for tick in 1..20_000 {
        turn(&mut restarted, forge, tick);
        let engine = restarted.engine.as_ref().unwrap();
        if match engine.domain.forge_connection(1) {
            domain::Connection::Connected { .. } => true,
            _ => false,
        } {
            break;
        }
    }
    let engine = restarted.engine.as_ref().unwrap();
    assert!(
        (match engine.domain.forge_connection(1) {
            domain::Connection::Connected { .. } => true,
            _ => false,
        })
    );
    assert!(engine.setup.public.is_empty());
    assert!(engine.domain.forges().rows[0].first.is_none(), "restart never repeats Create");
}
#[test]
fn admin_connect_without_first_keeps_after_reconcile_and_connects() {
    let mut forge = forge(true, true, false);
    let mut service = service(&forge);
    ask(&mut service, bundle(1, ADMIN), None);
    settle(&mut service, &mut forge);
    assert_eq!(admin_record(&service), Some(ADMIN));
    erased(&mut service, true);
    assert!(forge.service.setup().organization(b"work").is_none());
}
#[test]
fn bad_admin_bundle_erases_without_keeping_a_token() {
    for value in [bundle(11, ADMIN), bundle(1, b""), Secret::new(ADMIN.into())] {
        let mut forge = forge(true, true, false);
        let mut service = service(&forge);
        ask(&mut service, value, None);
        settle(&mut service, &mut forge);
        erased(&mut service, false);
    }
}
#[test]
fn refused_admin_token_erases_without_keeping_it() {
    let mut forge = forge(true, true, false);
    let mut service = service(&forge);
    forge.service.setup_mut().revoke(ADMIN);
    ask(&mut service, bundle(1, ADMIN), None);
    settle(&mut service, &mut forge);
    erased(&mut service, false);
    assert_eq!(
        service.engine.as_ref().unwrap().domain.forges().rows[0].ended_stop,
        Some(domain::AdminStop::Refused { status: 401, stage: domain::SetupStage::Admin })
    );
    assert!(contains(
        service.engine.as_ref().unwrap().domain.forges().rows[0].ended.as_ref().unwrap(),
        b"Refused(401)"
    ));
}
#[test]
fn not_admin_erases_without_keeping_the_token() {
    let mut forge = forge(false, true, false);
    let mut service = service(&forge);
    ask(&mut service, bundle(1, ADMIN), None);
    settle(&mut service, &mut forge);
    erased(&mut service, false);
    assert_eq!(service.engine.as_ref().unwrap().domain.forges().rows[0].ended_stop, Some(domain::AdminStop::NotAdmin));
    assert!(contains(service.engine.as_ref().unwrap().domain.forges().rows[0].ended.as_ref().unwrap(), b"NotAdmin"));
}
#[test]
fn unmarked_bot_erases_without_keeping_the_token() {
    let mut forge = forge(true, false, false);
    let mut service = service(&forge);
    ask(&mut service, bundle(1, ADMIN), None);
    settle(&mut service, &mut forge);
    erased(&mut service, false);
    assert_eq!(service.engine.as_ref().unwrap().domain.forges().rows[0].ended_stop, Some(domain::AdminStop::Unmarked));
    assert!(contains(service.engine.as_ref().unwrap().domain.forges().rows[0].ended.as_ref().unwrap(), b"Unmarked"));
}
#[test]
fn create_file_stop_marks_first_failed_and_erases_without_keeping_the_token() {
    let mut forge = forge(true, true, true);
    forge.service.setup_mut().org(b"work", b"elsewhere-owner");
    let mut service = service(&forge);
    ask(&mut service, bundle(1, ADMIN), Some(first(b"docker")));
    settle(&mut service, &mut forge);
    erased(&mut service, false);
    assert_eq!(
        service.engine.as_ref().unwrap().domain.forges().rows[0].ended_stop,
        Some(domain::AdminStop::Refused { status: 413, stage: domain::SetupStage::File })
    );
    assert_eq!(
        service.engine.as_ref().unwrap().domain.forges().rows[0].first.as_ref().unwrap().state,
        domain::FirstState::Failed { step: domain::CreateStep::File }
    );
}
#[test]
fn restart_without_an_admin_record_offers_connect_and_sweeps_a_crashed_carry() {
    let forge = forge(true, true, false);
    let mut store = store(&forge);
    store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, bundle(1, ADMIN)));
    assert!(
        (match store.host.above.pop() {
            Some(StoreEvent::Carried { number: found, .. }) => found == records::CARRIED,
            _ => false,
        })
    );
    let mut service = local_service(config(), store);
    restore_local(&mut service);
    erased(&mut service, false);
}
#[test]
fn both_admin_plans_read_the_paste_even_with_different_records_and_adoptions() {
    let forge = forge(true, true, false);
    let mut service = service(&forge);
    let engine = service.engine.as_mut().unwrap();
    row_placeholder(engine);
    engine
        .setup
        .lifecycle
        .adoptions
        .insert(
            1,
            lifecycle::Adoption {
                next: core::num::NonZeroU64::new(1).unwrap(),
                repositories: 1,
                files: Some(lifecycle::AdoptionFiles {
                    admin_token: Secret::new(b"different-adoption-token".as_slice().into()),
                    access: skein_forge_connection::Access {
                        credential: skein_forge::credential::Credential::Token(Secret::new(
                            b"different-api-token".as_slice().into(),
                        )),
                        user: 3,
                        webhook_secret: None,
                    },
                }),
            },
        )
        .unwrap();
    let carried = {
        let value = bundle(1, ADMIN);
        let digest = skein_lib::sha256::digest(value.expose());
        engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
        assert!(
            (match engine.store.host.above.pop() {
                Some(StoreEvent::Carried { number: found, .. }) => found == records::CARRIED,
                _ => false,
            })
        );
        jig_domain_core::Carried { number: records::CARRIED, digest }
    };
    public::admin_connect(engine, 1, Token::new(90), Some(first(b"docker")), carried);
    let run = *engine.setup.runs.first().unwrap().0;
    for kind in [Kind::AdminReconcile, Kind::AdminCreate] {
        engine.setup.runs.get_mut(&run).unwrap().kind = kind;
        let value = read_item(engine, run, 1, skein_forge_setup::item::Item::AdminToken).unwrap();
        assert_eq!(value.expose(), ADMIN);
        assert!(read_item(engine, run, 1, skein_forge_setup::item::Item::Pasted).is_none());
    }
}
fn row_placeholder(engine: &mut crate::engine::Engine) {
    engine.forge_records.rows.push(
        records::number(1, Item::AdminToken),
        Secret::new(records::encode(Item::AdminToken, b"different-record-token").unwrap().into_boxed_slice()),
        4096,
    );
}
#[test]
fn runner_labels_are_checked_again_before_yaml_is_made() {
    for label in [b"".as_slice(), b"x\nruns-on: hostile", b"x: y", &[b'a'; 65]] {
        assert!(temper_forge_setup::starter_workflow(label).is_none());
    }
    for label in [b"docker".as_slice(), b"._-", &[b'a'; 64]] {
        assert!(temper_forge_setup::starter_workflow(label).is_some());
    }
    let mut forge = forge(true, true, false);
    let mut service = service(&forge);
    ask(&mut service, bundle(1, ADMIN), Some(first(b"bad\nlabel")));
    settle(&mut service, &mut forge);
    erased(&mut service, false);
    assert!(forge.service.setup().organization(b"work").is_none());
}

#[test]
fn admin_create_makes_a_missing_organization_owned_by_the_verified_admin() {
    let mut forge = forge(true, true, false);
    let mut service = service(&forge);
    ask(&mut service, bundle(1, ADMIN), Some(first(b"docker")));
    settle(&mut service, &mut forge);
    assert_eq!(admin_record(&service), Some(ADMIN));
    assert_eq!(forge.service.setup().organization(b"work").unwrap().owner.as_ref(), b"elsewhere-owner");
}

fn disk_turn(store: &mut crate::store::Store, kernel: &mut skein_shell::Kernel) {
    store.iterate();
    let wait = if kernel.in_flight() > 0 || !store.submitted.is_empty() {
        skein_shell::Wait::Forever
    } else {
        skein_shell::Wait::No
    };
    kernel.submit(&mut store.submitted, wait);
    let mut completions = Queue::with_capacity(64);
    kernel.reap(&mut completions);
    while let Some(completion) = completions.pop() {
        store.complete(completion);
    }
}
#[test]
fn admin_record_files_and_directories_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = skein_scratch::Scratch::new_in(&std::env::current_dir().unwrap(), "admin-record-modes");
    std::fs::set_permissions(scratch.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let (root, _, profile) = fixtures::store_limits();
    let limits = crate::store::limits(&root, &root.forge, &profile).unwrap();
    let mut store = crate::store::Store::new(&root, &limits, &[], fixtures::seeds().installation);
    let fd = skein_shell::open_root(scratch.path()).unwrap();
    store.adopt(fd, skein_shell::effective_user()).unwrap();
    let mut kernel =
        skein_shell::Kernel::open_on(skein_shell::Config { operations: 64 }, skein_shell::Backend::Readiness).unwrap();
    let mut opened = false;
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.host.above.pop() {
            assert!(
                (match event {
                    StoreEvent::Opened { .. } => true,
                    _ => false,
                }),
                "{event:?}"
            );
            opened = true;
            break;
        }
    }
    assert!(opened);
    // This permissions fixture does not consume the census (jig protocol/store/README.md, 6.3).
    let Some(StoreEvent::Census { .. }) = store.host.above.pop() else { panic!("opened census") };
    fixtures::commit(&mut store, 1, Box::new([root::Write::Wires]));
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.host.above.pop() {
            let StoreEvent::Committed { .. } = event else { panic!("{event:?}") };
            break;
        }
    }
    assert!(store.host.mark_durable());
    store.secrets(
        Token::new(901),
        fixtures::application_request(
            records::number(1, Item::AdminToken),
            Secret::new(records::encode(Item::AdminToken, ADMIN).unwrap().into_boxed_slice()),
        ),
    );
    let mut kept = false;
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if let Some(event) = store.host.above.pop() {
            assert!(
                (match event {
                    StoreEvent::Kept { .. } => true,
                    _ => false,
                }),
                "{event:?}"
            );
            kept = true;
            break;
        }
    }
    assert!(kept);
    let mut paths = Queue::with_capacity(64);
    paths.push(scratch.path().join("generations/1/secrets"));
    let mut files = 0_u32;
    while let Some(path) = paths.pop() {
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(!metadata.is_symlink());
        if metadata.is_dir() {
            assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
            for entry in std::fs::read_dir(&path).unwrap() {
                paths.push(entry.unwrap().path());
            }
        } else {
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
            files = files.checked_add(1).unwrap();
        }
    }
    assert!(files > 0);
    store.close();
    for _ in 0_u32..4096 {
        disk_turn(&mut store, &mut kernel);
        if store.closed() {
            break;
        }
    }
    assert!(store.closed());
}

fn pending(service: &mut crate::Service) -> Token {
    ask(service, bundle(1, ADMIN), None);
    for _ in 0_u32..128 {
        let engine = service.engine.as_mut().unwrap();
        scan(engine);
        crate::engine::iterate(engine, Time::ZERO, Wall::EPOCH);
        if let Some((run, _)) = engine.setup.public.first() {
            return *run;
        }
    }
    panic!("admin run admitted");
}
#[test]
fn busy_admin_connect_erases_its_unused_paste() {
    let forge = forge(true, true, false);
    let mut service = service(&forge);
    let _run = pending(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let value = bundle(1, ADMIN);
    let digest = skein_lib::sha256::digest(value.expose());
    let number = records::CARRIED.checked_add(1).unwrap();
    engine.store.secrets(Token::new(901), fixtures::application_request(number, value));
    assert!(
        (match engine.store.host.above.pop() {
            Some(StoreEvent::Carried { number: found, .. }) => found == number,
            _ => false,
        })
    );
    public::admin_connect(engine, 1, Token::new(92), None, jig_domain_core::Carried { number, digest });
    assert_eq!(engine.store.erases, 1);
    assert!(
        (match engine.store.secrets(Token::new(902), Secrets::Get { owner: SecretOwner::Application, number }) {
            Some(StoreEvent::Secret { value: None, .. }) => true,
            _ => false,
        })
    );
    assert!(engine.events.iter().any(|event| match event {
        root::Event::Forge(event)
            if match event.as_ref() {
                domain::Event::ConnectAnswered {
                    answer: domain::ConnectAnswer::Refused(domain::ForgeRefusal::Busy),
                    ..
                } => true,
                _ => false,
            } =>
            true,
        _ => false,
    }));
    assert!(admin_record(&service).is_none());
}
#[test]
fn a_failed_keep_ends_the_admin_caller_and_erases_before_the_engine_stops() {
    let forge = forge(true, true, false);
    let mut service = service(&forge);
    let run = pending(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let origin =
        forge_records::Origin { kind: skein_forge_connection::settings::Kind::Forgejo, api: "127.0.0.1:3000".into() }
            .encode();
    lifecycle::keep(engine, 1, Item::Origin, &origin, Purpose::Host(run));
    let Some(StoreEvent::Kept { owner }) = engine.store.host.above.pop() else { panic!("pending keep") };
    answer(engine, StoreEvent::Corrupt { owner });
    assert!(engine.end.is_some());
    assert!(engine.setup.public.is_empty());
    assert!(engine.setup.runs.is_empty());
    assert!(engine.setup.writes.is_empty());
    assert_eq!(engine.store.erases, 1);
    scan(engine);
    assert!(engine.events.iter().any(|event| match event {
        root::Event::Forge(event)
            if match event.as_ref() {
                domain::Event::ConnectEnded { connected: false, .. } => true,
                _ => false,
            } =>
            true,
        _ => false,
    }));
    assert!(
        (match engine
            .store
            .secrets(Token::new(902), Secrets::Get { owner: SecretOwner::Application, number: records::CARRIED })
        {
            Some(StoreEvent::Secret { value: None, .. }) => true,
            _ => false,
        })
    );
    crate::engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    assert!(admin_record(&service).is_none());
}
#[test]
fn an_ended_forge_finishes_the_admin_run_without_keeping_the_token() {
    let forge = forge(true, true, false);
    let mut service = service(&forge);
    let run = pending(&mut service);
    let engine = service.engine.as_mut().unwrap();
    end_domain(engine);
    // Replace the fake host's pending read with its terminal, so no stale Need follows Done.
    while engine.setup.above.pop().is_some() {}
    engine.setup.above.push(host::Event::Done { run, outcome: reconcile::Outcome::Nothing });
    advance(engine);
    assert!(engine.setup.public.is_empty());
    assert!(engine.setup.runs.is_empty());
    scan(engine);
    // Run the real root's end and erase decision; the fake host still has no HTTP reply.
    for _ in 0_u32..64 {
        crate::engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    erased(&mut service, false);
}
#[test]
fn a_github_app_paste_cannot_decode_an_admin_bundle() {
    let mut service = routing_service(4, Duration::from_secs(60));
    let engine = service.engine.as_mut().unwrap();
    let value = bundle(1, ADMIN);
    let digest = skein_lib::sha256::digest(value.expose());
    engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
    assert!(
        (match engine.store.host.above.pop() {
            Some(StoreEvent::Carried { number: found, .. }) => found == records::CARRIED,
            _ => false,
        })
    );
    public::connect(
        engine,
        1,
        Token::new(90),
        core::num::NonZeroU64::new(7).unwrap(),
        jig_domain_core::Carried { number: records::CARRIED, digest },
    );
    assert!(engine.setup.public.is_empty());
    assert!(engine.setup.runs.is_empty());
    scan(engine);
    assert!(engine.events.iter().any(|event| match event {
        root::Event::Forge(event)
            if match event.as_ref() {
                domain::Event::ConnectEnded { connected: false, why: Some(why), .. }
                    if why.as_ref() == b"the pasted value is not a key" =>
                    true,
                _ => false,
            } =>
            true,
        _ => false,
    }));
}

#[test]
fn reconcile_and_create_use_the_paste_instead_of_an_older_admin_record() {
    let mut forge = forge(true, true, false);
    forge.service.setup_mut().org(b"work", b"elsewhere-owner");
    let mut service = service(&forge);
    row_placeholder(service.engine.as_mut().unwrap());
    ask(&mut service, bundle(1, ADMIN), Some(first(b"docker")));
    settle(&mut service, &mut forge);
    assert_eq!(admin_record(&service), Some(ADMIN));
    assert_eq!(
        service.engine.as_ref().unwrap().domain.forges().rows[0].first.as_ref().unwrap().state,
        domain::FirstState::Made { id: 1 }
    );
    erased(&mut service, true);
}

#[test]
fn a_managed_forge_on_this_host_is_served_the_listener() {
    let forge = forge(true, true, false);
    let mut config = config();
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    let service = local_service(config, store(&forge));
    let engine = service.engine.as_ref().unwrap();
    let (settings, _) = engine.setup.pending.as_ref().unwrap();
    let host::Kind::Forgejo(given) = &settings.forges.get(0).unwrap().kind else { panic!("Forgejo") };
    assert_eq!(given.listener, Some(([127, 0, 0, 1], 7101).into()));
    assert!(given.orgs.is_empty());
    assert!(given.org_owner.is_none());
    assert!(!engine.setup.waiting_for_webhooks);
    assert!(engine.setup.lifecycle.local(1).unwrap().webhooks);
}

#[test]
fn admin_scope_refusals_reach_the_row_at_the_failed_stage() {
    for (scopes, missing_bot, stage) in [
        (b"read:issue".as_slice(), false, domain::SetupStage::Admin),
        (b"read:user".as_slice(), true, domain::SetupStage::Bot),
    ] {
        let mut forge = forge(true, true, false);
        let setup = forge.service.setup_mut();
        setup.revoke(ADMIN);
        let user = setup.find(b"elsewhere-owner").unwrap().id;
        setup.token(user, b"limited", &[scopes], Secret::new(ADMIN.into()));
        let mut service = service(&forge);
        if missing_bot {
            forge.service.setup_mut().delete_account(b"temper");
        }
        ask(&mut service, bundle(1, ADMIN), None);
        settle(&mut service, &mut forge);
        erased(&mut service, false);
        assert_eq!(
            service.engine.as_ref().unwrap().domain.forges().rows[0].ended_stop,
            Some(domain::AdminStop::Refused { status: 403, stage })
        );
    }
}

#[test]
fn admin_connect_refuses_a_first_organization_named_as_the_bot() {
    let mut forge = forge(true, true, false);
    let mut service = service(&forge);
    let mut first = first(b"docker");
    first.org = b"TeMpEr".as_slice().into();
    ask(&mut service, bundle(1, ADMIN), Some(first));
    // An admitted connect would be Connecting; admission's CarriedUnused goes to jig, not the store.
    assert_eq!(service.engine.as_ref().unwrap().domain.forge_connection(1), domain::Connection::NotConnected);
    for tick in 1..64 {
        turn(&mut service, &mut forge, tick);
        let engine = service.engine.as_ref().unwrap();
        assert!(engine.setup.staged.is_empty() && engine.setup.public.is_empty() && engine.setup.runs.is_empty());
    }
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.domain.forge_connection(1), domain::Connection::NotConnected);
    assert!(engine.domain.forges().rows[0].first.is_none());
    assert!(forge.service.setup().organization(b"TeMpEr").is_none());
    assert!(admin_record(&service).is_none());
}

#[test]
fn setup_refuses_a_carried_value_of_another_kind() {
    for kind in [jig_protocol_store::CarriedKind::Key, jig_protocol_store::CarriedKind::Moved] {
        let mut service = routing_service(4, Duration::from_secs(60));
        let engine = service.engine.as_mut().unwrap();
        let value = bundle(11, ADMIN);
        let digest = skein_lib::sha256::digest(value.expose());
        engine.store.secrets(Token::new(901), Secrets::Carry { kind, value });
        let Some(StoreEvent::Carried { number, .. }) = engine.store.host.above.pop() else { panic!("carried seed") };
        public::connect(
            engine,
            1,
            Token::new(90),
            core::num::NonZeroU64::new(7).unwrap(),
            jig_domain_core::Carried { number, digest },
        );
        assert!(engine.setup.public.is_empty());
        assert!(engine.setup.runs.is_empty());
        assert!(engine.events.iter().any(|event| match event {
            root::Event::Forge(event) => match event.as_ref() {
                domain::Event::ConnectEnded { connected: false, why: Some(why), .. } =>
                    why.as_ref() == b"the pasted value changed",
                _ => false,
            },
            _ => false,
        }));
    }
}

// A private repository outside every bot team; API and git share its fixture id.
#[expect(clippy::too_many_lines, reason = "one fixture seeds shared API and git state before its connected service")]
pub(super) fn access_host(owned: bool) -> (crate::Service, Forge) {
    use skein_fake_forge_domain as git;
    use skein_fake_forge_forgejo as fake;
    let mut forge = Forge::new(false);
    let limits = skein_forge_kit::fixtures::fake_limits();
    let mut repositories = List::with_capacity(1);
    repositories.push(fake::Repository { id: 1, full: b"other/closed".as_slice().into(), protected: None }).unwrap();
    forge.service = fake::Service::new(
        fake::Config {
            hooks: None,
            base: b"http://127.0.0.1:9".as_slice().into(),
            tokens: List::with_capacity(0),
            users: List::with_capacity(0),
            repositories,
            max_response_items: 32,
            default_paging_num: 30,
            sources: Box::new([]),
            faults: fake::Faults { reset: 0, drop: 0, server_error: 0 },
        },
        &limits,
        7,
    )
    .unwrap();
    let setup = forge.service.setup_mut();
    let admin = setup.account(b"elsewhere-owner", true, b"");
    setup.token(
        admin,
        b"admin",
        &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
        Secret::new(ADMIN.into()),
    );
    let bot = setup.account(b"temper", false, b"temper:bot");
    for (name, token) in [(b"temper-api".as_slice(), b"test-api".as_slice()), (b"temper-git", b"test-git")] {
        setup.token(bot, name, &[b"read:user", b"write:repository"], Secret::new(token.into()));
    }
    setup.org(b"other", b"elsewhere-owner");
    setup.private(1).unwrap();
    git::repository(
        &mut forge.git,
        &forge.config,
        git::api::Setup {
            id: 1,
            name: b"other/closed".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: git::api::Checks {
                cues: Box::new([]),
                contexts: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 0,
                reruns: 0,
                cue: None,
            },
            workflows: Box::new([]),
            protection: None,
            hooked: false,
        },
    );
    let mut config = if owned { local_config(true, false) } else { config() };
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    config.engine.jig.limits.core.authority.segments = 8;
    config.engine.jig.limits.core.people.amendment_bytes = 256;
    let mut store = store(&forge);
    if owned {
        kept_record(&mut store, 1, Item::AdminToken, ADMIN);
        kept_record(&mut store, 1, Item::WebhookSecret, b"hook-secret");
    }
    let mut service = local_service(config, store);
    service.engine.as_mut().unwrap().setup.waiting_for_webhooks = false;
    restore_local(&mut service);
    if !owned {
        ask(&mut service, bundle(1, ADMIN), None);
    }
    for tick in 1..20_000 {
        turn(&mut service, &mut forge, tick);
        if match service.engine.as_ref().unwrap().domain.forge_connection(1) {
            domain::Connection::Connected { .. } => true,
            _ => false,
        } {
            break;
        }
    }
    assert!(match service.engine.as_ref().unwrap().domain.forge_connection(1) {
        domain::Connection::Connected { .. } => true,
        _ => false,
    });
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Party(root::PartyInput::SignedIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(Token::new(100)),
        identity: jig_domain_people::Identity {
            key: jig_domain_people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    for tick in 20_000..20_032 {
        turn(&mut service, &mut forge, tick);
    }
    (service, forge)
}
pub(super) fn adopt_private(service: &mut crate::Service, key: u8) {
    service.engine.as_mut().unwrap().step(root::Event::Party(root::PartyInput::Application {
        scope: jig_domain_people::ApplicationScope::Project(1),
        reply_to: skein_lib::ReplyTo::new(Token::new(u64::from(key))),
        sign_in: 1,
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: root::Application::Forge(domain::Ask::Adopt(domain::Adoption {
            project: 1,
            home: false,
            forge: 1,
            host: b"forge.example".as_slice().into(),
            owner: b"other".as_slice().into(),
            name: b"closed".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: domain::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    }));
}
pub(super) fn finish_access(service: &mut crate::Service, forge: &mut Forge) {
    for tick in 20_032..40_000 {
        turn(service, forge, tick);
        let rows = service.engine.as_ref().unwrap().domain.forges();
        if rows.rows[0].repositories.iter().any(|row| row.name.as_ref() == b"closed") {
            assert!(service.engine.as_ref().unwrap().setup.public.is_empty());
            for tick in 40_000..40_016 {
                turn(service, forge, tick);
            }
            let records = fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge);
            assert!(records.iter().any(|record| match record {
                root::Record::Forge { row } => match row.as_ref() {
                    domain::Stored::Repository(repository) =>
                        repository.name.as_ref() == b"closed" && repository.kinds.push,
                    _ => false,
                },
                root::Record::Core(_) => false,
            }));
            return;
        }
    }
    let records = fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge);
    panic!(
        "private adoption did not finish: {records:?}; collaborator: {:?}; tally: {:?}",
        forge.service.setup().collaborator(1, b"temper"),
        forge.service.tally()
    );
}

#[test]
fn a_managed_adoption_on_this_host_grants_write_and_makes_one_hook_at_the_listener() {
    let (mut service, mut forge) = access_host(false);
    assert!(forge.service.setup().collaborator(1, b"temper").is_none());
    adopt_private(&mut service, 70);
    finish_access(&mut service, &mut forge);
    assert_eq!(forge.service.setup().collaborator(1, b"temper"), Some(skein_forge::forgejo::request::Grant::Write));
    let mut hooks = forge.service.setup().repository_hooks(1);
    assert_eq!(hooks.next().expect("adoption hook").url.as_ref(), b"http://127.0.0.1:7101/hooks/1");
    assert!(hooks.next().is_none(), "one hook");
}
#[test]
fn an_adoption_without_admin_repository_scope_refuses_and_names_its_step() {
    let (mut service, mut forge) = access_host(false);
    let setup = forge.service.setup_mut();
    setup.revoke(ADMIN);
    let user = setup.find(b"elsewhere-owner").unwrap().id;
    setup.token(user, b"limited", &[b"read:user"], Secret::new(ADMIN.into()));
    let engine = service.engine.as_mut().unwrap();
    public::adopt_access(engine, 1, Token::new(70), b"other".as_slice().into(), b"closed".as_slice().into());
    let mut refused = false;
    let mut named = false;
    for tick in 20_032..40_000 {
        let engine = service.engine.as_mut().unwrap();
        for event in &engine.events {
            if let root::Event::Forge(event) = event
                && let domain::Event::AccessAnswered { request, answer, .. } = event.as_ref()
                && *request == Token::new(70)
            {
                assert_eq!(*answer, Err(domain::AdoptionRefusal::NoAccess));
                refused = true;
            }
        }
        for note in &engine.notes {
            if let crate::Note::ForgeSetupStopped { number: 1, why } = note {
                assert_eq!(
                    *why,
                    crate::SetupStopped::Stopped {
                        plan: crate::SetupPlan::Adopt,
                        step: crate::SetupStep {
                            stage: reconcile::Stage::Repository,
                            why: reconcile::Why::Refused(403)
                        }
                    }
                );
                named = true;
            }
        }
        if refused && named {
            break;
        }
        // Inspect before the root consumes the host's terminal event.
        advance(engine);
        engine.forge.iterate(engine.env.now, Wall::EPOCH);
        forge.turn_io(engine);
        forge.turn_server(engine);
        engine.env.now = Time::from_nanos(tick * 1_000_000);
    }
    assert!(refused && named);
    assert!(service.engine.as_ref().unwrap().setup.runs.is_empty());
    assert!(forge.service.setup().collaborator(1, b"temper").is_none());
}
#[test]
fn a_second_access_is_in_flight_and_can_retry_after_the_first_plan() {
    let (mut service, mut forge) = access_host(false);
    let engine = service.engine.as_mut().unwrap();
    for request in [70, 71] {
        public::adopt_access(engine, 1, Token::new(request), b"other".as_slice().into(), b"closed".as_slice().into());
    }
    assert!(access_answered(engine, Some(Token::new(71)), domain::AdoptionRefusal::InFlight));
    for tick in 20_032..40_000 {
        turn(&mut service, &mut forge, tick);
        if service.engine.as_ref().unwrap().setup.runs.is_empty() {
            break;
        }
    }
    adopt_private(&mut service, 72);
    finish_access(&mut service, &mut forge);
}

#[test]
fn a_managed_start_stop_names_repository_on_the_row_and_offers_connect() {
    let forge = forge(true, true, false);
    let mut service = service(&forge);
    let engine = service.engine.as_mut().unwrap();
    let run = Token::new(70);
    engine
        .setup
        .runs
        .insert(
            run,
            Run {
                making: None,
                words: None,
                first: None,
                previous: domain::Connection::NotConnected,
                forge: 1,
                kind: Kind::LocalReconcile,
                internal: true,
            },
        )
        .unwrap();
    lifecycle::stopped(
        engine,
        run,
        reconcile::Failure {
            forge: 1,
            step: reconcile::Stage::Repository,
            why: reconcile::Why::Refused(403),
            subject: b"other/closed".as_slice().into(),
        },
    );
    for _ in 0_u32..16 {
        crate::engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(
        engine.domain.forge_connection(1),
        domain::Connection::SetUpFailed { why: b"reconciling: Repository: Refused(403)".as_slice().into() }
    );
    assert!(engine.domain.forges().rows[0].admin, "Connect remains offered");
    ask(&mut service, bundle(1, ADMIN), None);
    assert_eq!(
        service.engine.as_ref().unwrap().domain.forge_connection(1),
        domain::Connection::Connecting,
        "Connect is admitted"
    );
}

#[test]
fn each_access_stop_answers_by_its_step_and_names_adopt() {
    use domain::AdoptionRefusal as Refusal;
    use reconcile::{Stage, Why};
    let forge = forge(true, true, false);
    let cases = [
        (Stage::Admin, Why::Missing, Refusal::NoAccess),
        (Stage::Admin, Why::Down, Refusal::NoAccess),
        (Stage::Repository, Why::Missing, Refusal::NotFound),
        (Stage::Repository, Why::Refused(404), Refusal::NoAccess),
        (Stage::Repository, Why::Down, Refusal::Unreachable),
        (Stage::Repository, Why::TimedOut, Refusal::Unreachable),
        (Stage::State, Why::Busy, Refusal::InFlight),
        (Stage::Repository, Why::Unknown, Refusal::InvalidAnswer),
        (Stage::Bot, Why::Missing, Refusal::InvalidAnswer),
        (Stage::Bot, Why::Refused(403), Refusal::InvalidAnswer),
    ];
    for (step, why, expected) in cases {
        let mut service = service(&forge);
        let engine = service.engine.as_mut().unwrap();
        while engine.notes.pop().is_some() {}
        public::adopt_access(engine, 1, Token::new(70), b"other".as_slice().into(), b"closed".as_slice().into());
        let run = *engine.setup.runs.first().unwrap().0;
        lifecycle::stopped(
            engine,
            run,
            reconcile::Failure { forge: 1, step, why, subject: b"other/closed".as_slice().into() },
        );
        assert!(engine.setup.runs.is_empty());
        assert!(engine.setup.public.is_empty());
        let mut answered = 0_u32;
        while let Some(event) = engine.events.pop() {
            if let root::Event::Forge(event) = event
                && let domain::Event::AccessAnswered { answer, .. } = *event
            {
                assert_eq!(answer, Err(expected), "{step:?} {why:?}");
                answered += 1;
            }
        }
        assert_eq!(answered, 1);
        assert!(engine.notes.iter().any(|note| match note {
            crate::Note::ForgeSetupStopped {
                number: 1,
                why: crate::SetupStopped::Stopped { plan: crate::SetupPlan::Adopt, step: found },
            } => found.stage == step && found.why == why,
            _ => false,
        }));
    }
}

#[test]
fn access_local_ends_and_full_entrances_answer_without_leaving_a_caller() {
    let forge = forge(true, true, false);
    let mut service = service(&forge);
    for (reason, expected) in [
        (b"setup host: busy".as_slice(), domain::AdoptionRefusal::InFlight),
        (b"forge ended", domain::AdoptionRefusal::Unreachable),
    ] {
        let engine = service.engine.as_mut().unwrap();
        public::adopt_access(engine, 1, Token::new(70), b"other".as_slice().into(), b"closed".as_slice().into());
        let run = *engine.setup.runs.first().unwrap().0;
        lifecycle::refused(engine, run, 1, reason.into());
        assert!(engine.setup.runs.is_empty() && engine.setup.public.is_empty());
        assert!(access_answered(engine, None, expected));
        while engine.events.pop().is_some() {}
    }
    for public_full in [false, true] {
        let engine = service.engine.as_mut().unwrap();
        if public_full {
            for number in 0..engine.setup.public.capacity() {
                engine
                    .setup
                    .public
                    .insert(
                        Token::new(u64::from(number)),
                        public::Caller::Reachable { forge: 1, request: Token::new(0) },
                    )
                    .unwrap();
            }
        } else {
            engine.setup.env.limits.runs = 0;
        }
        public::adopt_access(engine, 1, Token::new(70), b"other".as_slice().into(), b"closed".as_slice().into());
        assert!(access_answered(engine, None, domain::AdoptionRefusal::InFlight));
        while engine.events.pop().is_some() {}
        engine.setup.env.limits.runs = 4;
        engine.setup.public = Map::with_capacity(engine.setup.public.capacity());
    }
}

fn access_answered(engine: &crate::engine::Engine, request: Option<Token>, expected: domain::AdoptionRefusal) -> bool {
    engine.events.iter().any(|event| match event {
        root::Event::Forge(event) => match event.as_ref() {
            domain::Event::AccessAnswered { request: found, answer, .. } => {
                (request.is_none() || request == Some(*found)) && *answer == Err(expected)
            }
            _ => false,
        },
        _ => false,
    })
}

#[test]
fn a_secret_drawn_at_a_first_adoption_reaches_the_connection() {
    let (mut service, mut forge) = access_host(false);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(
        engine.forge_records.rows.forge(1).unwrap().get(Item::WebhookSecret),
        Err(forge_records::NotConnected::Incomplete)
    );
    assert!(engine.forge.starts.iter().last().unwrap().1.webhook_secret.is_none());
    adopt_private(&mut service, 70);
    finish_access(&mut service, &mut forge);
    let engine = service.engine.as_ref().unwrap();
    let secret = engine.forge_records.rows.forge(1).unwrap().get(Item::WebhookSecret).unwrap();
    assert_eq!(
        engine
            .forge
            .starts
            .iter()
            .last()
            .unwrap()
            .1
            .webhook_secret
            .as_ref()
            .expect("connection receives the drawn secret")
            .expose(),
        secret
    );
    assert!(match engine.domain.forge_connection(1) {
        domain::Connection::Connected { .. } => true,
        _ => false,
    });
}

#[test]
fn a_managed_forge_whose_hook_delivers_nothing_stays_connected_and_keeps_reading() {
    let (mut service, mut forge) = access_host(false);
    adopt_private(&mut service, 70);
    finish_access(&mut service, &mut forge);
    assert_eq!(forge.service.setup().repository_hooks(1).count(), 1);
    for tick in 40_016_u64..40_272 {
        turn(&mut service, &mut forge, tick.checked_mul(10_000).unwrap());
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(match engine.domain.forge_connection(1) {
        domain::Connection::Connected { .. } => true,
        _ => false,
    });
    let rows = engine.domain.forges();
    assert!(rows.rows[0].ended.is_none());
    assert!(rows.rows[0].ended_stop.is_none());
    for line in &forge.requests {
        assert!(!(contains(line, b"/hooks/") && contains(line, b"/tasks")));
    }
}

#[test]
fn a_managed_forge_elsewhere_is_served_no_listener() {
    let forge = forge(true, true, false);
    let mut config = config();
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    let mut service = local_service(config, store(&forge));
    let roots = roots();
    service.configure_forge_runtime(
        crate::ForgeRuntime {
            log_origins: List::with_capacity(0),
            number: 0,
            address: ([192, 0, 2, 9], 443).into(),
            transport: skein_forge_connection::settings::Transport::Tls {
                server_name: skein_tls::Name::new("forge.example").unwrap(),
                trust: roots.clone(),
            },
        },
        roots,
    );
    let mut remote = fixtures::forge_record(2);
    remote.mode = domain::Mode::Managed;
    remote.place.api.host = b"192.0.2.9".as_slice().into();
    remote.place.api.port = 443;
    remote.place.api.tls = Some(b"forge.example".as_slice().into());
    crate::route::request(service.engine.as_mut().unwrap(), root::Request::ForgeKept(remote));
    let engine = service.engine.as_ref().unwrap();
    let host::Kind::Forgejo(given) = &engine.setup.served.get(&2).unwrap().kind else { panic!("Forgejo") };
    assert!(given.listener.is_none());
    assert!(!engine.setup.lifecycle.local(2).unwrap().webhooks);
    assert!(engine.forge.io.is_empty() && engine.setup.io.is_empty(), "no exchange with the documentation address");
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
