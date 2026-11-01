use super::*;
use temper_engine_domain_forge as domain;

fn configured(hooks: bool) -> crate::settings::Engine {
    let mut config = local_config(true, true);
    config.machine.owned.as_mut().unwrap().setup = Some(br#"{"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{"listen":{"socket":"127.0.0.1:9"},"git":"/data/git","owner":"demo","repositories":[{"name":"widget","landing":"main"},{"name":"other","landing":"main"}]}}}"#.as_slice().into());
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    if !hooks {
        config.webhooks = None;
    }
    config.limits.forge.repositories = 3;
    config
}
fn record_answers(service: &mut crate::Service) {
    let engine = service.engine.as_mut().unwrap();
    while let Some(event) = engine.store.above.pop() {
        answer(engine, event);
    }
    while engine.setup.answers.pop().is_some() {}
    advance(engine);
}
fn reconcile_bot(service: &mut crate::Service, face: &mut Face) {
    local_sent(service, face, b"GET /api/v1/user ");
    assert!(contains(&face.wire.received, b"test-admin"));
    face.reply(service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
    local_sent(service, face, b"GET /api/v1/users/temper ");
    face.reply(service, br#"{"id":42,"login":"temper","full_name":"temper:bot","is_admin":false}"#);
    for token in [b"test-api".as_slice(), b"test-git"] {
        local_sent(service, face, b"GET /api/v1/user ");
        assert!(contains(&face.wire.received, token));
        face.reply(service, br#"{"id":42,"login":"temper"}"#);
    }
}
#[expect(clippy::disallowed_macros, reason = "the protocol peer formats its repository fixture")]
pub(super) fn record_repository(service: &mut crate::Service, face: &mut Face, id: u64, permission: &[u8]) {
    local_sent(service, face, format!("GET /api/v1/repositories/{id} ").as_bytes());
    face.reply(service, format!(r#"{{"id":{id},"owner":{{"id":1,"login":"demo"}},"name":"widget","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":true,"allow_rebase":true,"html_url":"http://127.0.0.1:9/demo/widget"}}"#).as_bytes());
    local_sent(service, face, b"/collaborators/temper/permission ");
    assert!(contains(&face.wire.received, b"test-admin"));
    face.reply(service, permission);
}
fn record_hook(service: &mut crate::Service, face: &mut Face, secret: &[u8]) {
    local_sent(service, face, b"GET /api/v1/settings/api ");
    face.reply(service, br#"{"max_response_items":50,"default_paging_num":30}"#);
    local_sent(service, face, b"/hooks?page=1&limit=32 ");
    face.reply(service, b"[]");
    local_sent(service, face, b"POST /api/v1/repos/demo/widget/hooks ");
    assert!(contains(&face.wire.received, b"http://127.0.0.1:7101/hooks/1"));
    assert!(contains(&face.wire.received, secret), "the hook uses the record's secret");
    assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
    face.reply(
        service,
        br#"{"id":1,"config":{"url":"http://127.0.0.1:7101/hooks/1"},"events":["push"],"active":true}"#,
    );
}
pub(super) const WRITE_PERMISSION: &[u8] = br#"{"permission":"write","user":{"id":42,"login":"temper"}}"#;
fn records_reconciled(hooks: bool, handover: bool) -> (crate::Service, Face, Box<[u8]>) {
    let mut config = configured(hooks);
    if handover {
        inject_handover(&mut config, Secret::new(b"test-admin".as_slice().into()));
    }
    let mut service = local_service(config, local_store(true));
    restore_local(&mut service);
    let mut face = Face::new();
    if handover {
        assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalTakeOver);
        let remove = takeover_done(&mut service, &mut face);
        face.hold_done = false;
        service.handover_removed(1, remove.run, Ok(()));
        advance(service.engine.as_mut().unwrap());
    }
    assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalReconcile);
    reconcile_bot(&mut service, &mut face);
    record_repository(&mut service, &mut face, 41, WRITE_PERMISSION);
    let secret = if hooks {
        let engine = service.engine.as_ref().unwrap();
        assert_eq!(engine.setup.writes.first().unwrap().1.number, records::number(1, Item::WebhookSecret));
        record_answers(&mut service);
        let secret: Box<[u8]> = service
            .engine
            .as_ref()
            .unwrap()
            .forge_records
            .rows
            .forge(1)
            .unwrap()
            .get(Item::WebhookSecret)
            .unwrap()
            .into();
        record_hook(&mut service, &mut face, &secret);
        secret
    } else {
        Box::new([])
    };
    assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
    record_answers(&mut service);
    (service, face, secret)
}
#[test]
fn records_fake_takes_over_reconciles_adopts_in_order_then_connects_from_records() {
    for hooks in [false, true] {
        let (mut service, mut face, secret) = records_reconciled(hooks, true);
        for id in [1, 2] {
            assert_eq!(service.engine.as_ref().unwrap().setup.lifecycle.adoptions.get(&1).unwrap().next.get(), id);
            assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::Adopt);
            assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
            local_sent(&mut service, &mut face, b"GET /api/v1/user ");
            assert!(contains(&face.wire.received, b"test-admin"));
            face.reply(&mut service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
            face.hold_done = true;
            record_repository(&mut service, &mut face, id, WRITE_PERMISSION);
            if hooks {
                record_hook(&mut service, &mut face, &secret);
            }
            assert!(face.done.is_some());
            assert!(service.engine.as_ref().unwrap().forge.starts.is_empty(), "no Connect before the last Adopt Done");
            service.engine.as_mut().unwrap().setup.above.push(face.done.take().unwrap());
            face.hold_done = false;
            advance(service.engine.as_mut().unwrap());
        }
        let engine = service.engine.as_ref().unwrap();
        assert!(engine.setup.lifecycle.adoptions.is_empty());
        let (_, access) = engine.forge.starts.get(0).expect("Connect after every Adopt");
        assert_eq!(access.user, 42);
        assert_eq!(
            access.credential,
            skein_forge::credential::Credential::Token(Secret::new(b"test-api".as_slice().into()))
        );
        assert_eq!(access.webhook_secret, if hooks { Some(Secret::new(secret)) } else { None });
        assert_eq!(engine.forge_records.rows.forge(1).unwrap().get(Item::AdminToken).unwrap(), b"test-admin");
        assert!(service.take_handover_removal().is_none());
    }
}
#[test]
fn records_fake_stops_at_adopt_without_connecting_or_starting_the_next_repository() {
    let (mut service, mut face, _) = records_reconciled(false, false);
    local_sent(&mut service, &mut face, b"GET /api/v1/user ");
    face.reply(&mut service, br#"{"id":1,"login":"person","is_admin":false}"#);
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.setup.lifecycle.adoptions.is_empty());
    assert!(engine.setup.runs.is_empty());
    assert!(engine.forge.starts.is_empty());
    assert!(engine.setup.io.is_empty());
    let mut stopped = false;
    for note in &engine.notes {
        if let crate::Note::ForgeSetupStopped { number: 1, why } = note {
            assert_eq!(
                why,
                &crate::SetupStopped::Stopped {
                    plan: crate::SetupPlan::Adopt,
                    step: crate::SetupStep { stage: reconcile::Stage::Admin, why: reconcile::Why::NotAdmin },
                }
            );
            stopped = true;
        }
    }
    assert!(stopped);
    let mut not_connected = false;
    for event in &engine.events {
        if let root::Event::Forge(value) = event
            && let domain::Event::Connection { forge: 1, state: domain::Connection::NotConnected } = value.as_ref()
        {
            not_connected = true;
        }
    }
    assert!(not_connected);
}
#[test]
fn records_fake_restart_reconciles_restored_repositories_and_adopts_only_the_missing_one() {
    for adopted in [1_u64, 2] {
        let mut store = local_store(true);
        let mut writes = List::with_capacity(2);
        for id in 1..=adopted {
            let mut repository = local_repository();
            repository.provider.repository = u32::try_from(id).unwrap();
            repository.name = if id == 1 { b"seeded-1".as_slice().into() } else { b"seeded-2".as_slice().into() };
            repository.project = u32::try_from(id + 1).unwrap();
            writes
                .push(root::Write::Save(root::Record::Forge { row: Box::new(domain::Stored::Repository(repository)) }))
                .unwrap();
        }
        fixtures::commit(&mut store, 2, writes.into_boxed());
        assert!(store.above.pop().is_some());
        let mut service = local_service(configured(false), store);
        restore_local(&mut service);
        assert_eq!(service.engine.as_ref().unwrap().domain.adopted_ids(1).len(), usize::try_from(adopted + 1).unwrap());
        let mut face = Face::new();
        reconcile_bot(&mut service, &mut face);
        for id in 1..=adopted {
            record_repository(&mut service, &mut face, id, WRITE_PERMISSION);
        }
        record_repository(&mut service, &mut face, 41, WRITE_PERMISSION);
        record_answers(&mut service);
        if adopted == 1 {
            local_sent(&mut service, &mut face, b"GET /api/v1/user ");
            assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::Adopt);
            face.reply(&mut service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
            record_repository(&mut service, &mut face, 2, WRITE_PERMISSION);
        }
        assert_eq!(service.engine.as_ref().unwrap().forge.starts.len(), 1);
        assert!(service.engine.as_ref().unwrap().setup.lifecycle.adoptions.is_empty());
        assert!(service.engine.as_ref().unwrap().setup.runs.is_empty());
    }
}
#[test]
fn records_fake_given_uses_the_webhooks_listener_without_a_file_secret() {
    for hooks in [false, true] {
        let config = configured(hooks);
        let service = local_service(config, local_store(true));
        let host::Kind::Forgejo(given) = service.engine.as_ref().unwrap().setup.served.get(&1).unwrap() else {
            panic!("fake Given")
        };
        assert_eq!(given.listener, if hooks { Some(([127, 0, 0, 1], 7101).into()) } else { None });
    }
}
