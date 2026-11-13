#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "routing tests inspect captured events and diagnostics"
)]
//! Replacement access and durable records (temper/docs/design/protocol/forge/engine.md, 2 and 6).
use super::create_tests::Forge;
use super::tokens_tests::{self as tokens, Tokens};
use super::*;
use temper_client_protocol_wire::v1 as w;

const FORGE: u16 = 1;
const REQUEST: Token = Token::new(90);
// Replace keeps the three pasted tokens/secret items and Origin.
const TOKEN_WRITES: u32 = 4;
// Paste keeps AppId, AppSlug, AppKey, User and Origin.
const APP_WRITES: u64 = 5;

fn carried(service: &mut crate::Service, value: Secret) -> jig_domain_core::Carried {
    let digest = skein_lib::sha256::digest(value.expose());
    let engine = service.engine.as_mut().unwrap();
    engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
    let Some(StoreEvent::Carried { number, .. }) = engine.store.host.above.pop() else { panic!("carried value") };
    jig_domain_core::Carried { number, digest }
}
fn replace(service: &mut crate::Service, value: Secret) -> u64 {
    let carried = carried(service, value);
    let number = carried.number;
    crate::route::request(
        service.engine.as_mut().unwrap(),
        root::Request::Replace { forge: FORGE, request: REQUEST, carried },
    );
    number
}
fn connected_forgejo() -> (crate::Service, Box<[u8]>, Forge, Tokens, u64) {
    let mut forge = Forge::new(false);
    let old = Tokens::new(71);
    let user = old.give(&mut forge, b"checked-user", false, false);
    let (mut service, bearer) = tokens::owner();
    tokens::http_connect(&mut service, &bearer, 1, true, old.bundle(true));
    let _first = tokens::settle(&mut service, &mut forge, &old, true);
    (service, bearer, forge, old, user)
}
fn http_replace(service: &mut crate::Service, bearer: &[u8], value: Secret) {
    let request = w::Request::Replace(w::Replace::new(&w::CEILINGS, w::ReplaceParts { forge: FORGE }).unwrap());
    fixtures::forge_connecting(service, bearer, 2, request, value);
}
fn observe_replace(trace: &mut tokens::Trace, engine: &crate::engine::Engine, new: &Tokens) {
    trace.scan(engine, new);
    if !engine.setup.erases.is_empty() {
        assert_eq!(engine.values.identities.newest(u32::from(FORGE)).unwrap().0, 1);
    }
    if engine.forge.starts.len() == 2 {
        let (generation, value) = engine.values.identities.newest(u32::from(FORGE)).unwrap();
        assert_eq!(generation, 2, "the next identity is present when Connect is handed down");
        let identity = temper_protocol_worker_wire::v1::Identity::decode(
            &temper_protocol_worker_wire::v1::CEILINGS,
            &mut skein_lib::Reader::new(value.expose()),
        )
        .unwrap();
        let temper_protocol_worker_wire::v1::Kind::Forgejo(auth) = identity.kind() else { panic!("Forgejo identity") };
        assert!(auth.token().expose() == new.git.expose());
    }
}
// The live connection stays open while Check uses its own socket. Both faces hold
// the same fixture accounts and issued credentials.
fn replace_settle(
    service: &mut crate::Service,
    forge: &mut Forge,
    check: &mut Forge,
    new: &Tokens,
    success: bool,
) -> tokens::Trace {
    let mut trace = tokens::Trace::new();
    for tick in 1_u64..20_000 {
        let engine = service.engine.as_mut().unwrap();
        observe_replace(&mut trace, engine, new);
        while engine.notes.pop().is_some() {}
        while engine.setup.answers.pop().is_some() {}
        engine.env.now = Time::from_nanos(tick.checked_mul(1_000_000).unwrap());
        advance(engine);
        observe_replace(&mut trace, engine, new);
        engine.forge.iterate(engine.env.now, Wall::EPOCH);
        let live = core::mem::replace(&mut engine.forge.io, Queue::uncapped());
        check.turn_io(engine);
        engine.forge.io = live;
        let setup = core::mem::replace(&mut engine.setup.io, Queue::uncapped());
        forge.turn_io(engine);
        engine.setup.io = setup;
        check.turn_server(engine);
        forge.turn_server(engine);
        for _ in 0_u32..4 {
            observe_replace(&mut trace, engine, new);
            crate::engine::answer(engine);
            observe_replace(&mut trace, engine, new);
            let mut held = core::mem::replace(&mut engine.store.host.above, Queue::uncapped());
            crate::engine::iterate(engine, engine.env.now, Wall::EPOCH);
            observe_replace(&mut trace, engine, new);
            while let Some(event) = held.pop() {
                engine.store.host.above.push(event);
            }
        }
        check.service.reclaim();
        forge.service.reclaim();
        if engine.setup.public.is_empty() && engine.setup.runs.is_empty() && engine.setup.erases.is_empty() {
            if success {
                if engine.forge.starts.len() != 2
                    || engine.domain.forge_connection(FORGE)
                        != (domain::Connection::Connected {
                            id: engine.workers.forges.get(FORGE).unwrap().user,
                            login: b"checked-user".as_slice().into(),
                        })
                {
                    continue;
                }
            } else if engine.domain.forges().rows[0].ended.is_none() {
                continue;
            }
            return trace;
        }
    }
    panic!("replacement did not settle");
}
fn checking(old: &Tokens, new: &Tokens, login: &[u8]) -> Forge {
    let mut check = Forge::new(false);
    old.give(&mut check, b"checked-user", false, false);
    new.give(&mut check, login, false, false);
    check
}
#[test]
fn git_token_replacement_checks_kept_api_and_connects_again() {
    let (mut service, bearer, mut forge, old, user) = connected_forgejo();
    let mut new = Tokens::new(72);
    new.api = old.api.clone();
    new.give(&mut forge, b"checked-user", false, false);
    let bundle = records::encode_pasted(&[(Item::GitToken.code(), new.git.expose())]).unwrap();
    http_replace(&mut service, &bearer, bundle);
    let mut check = checking(&old, &new, b"checked-user");
    let trace = replace_settle(&mut service, &mut forge, &mut check, &new, true);
    let order: Box<[_]> = trace.keeps.into_boxed().into_iter().map(|(_, number)| number).collect();
    assert_eq!(order.as_ref(), &[records::number(FORGE, Item::GitToken), records::number(FORGE, Item::Origin)]);
    tokens::connected(&service, user, &new, 2);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.forge.starts.len(), 2);
    assert!(engine.setup.pastes.is_empty() && engine.setup.erases.is_empty());
    old.scan(engine);
    tokens::erased(&mut service);
}
#[test]
fn stopped_forge_replacement_checks_both_tokens_and_reconnects() {
    let (mut service, bearer, mut forge, old, user) = connected_forgejo();
    tokens::stopped(&mut service, &mut forge, &old);
    let new = Tokens::new(73);
    new.give(&mut forge, b"checked-user", false, false);
    http_replace(&mut service, &bearer, new.bundle(true));
    let trace = tokens::settle(&mut service, &mut forge, &new, true);
    assert_eq!(trace.keeps.len(), TOKEN_WRITES);
    tokens::connected(&service, user, &new, 2);
    tokens::erased(&mut service);
}
#[test]
fn other_users_replacement_preserves_connected_access_and_records() {
    let (mut service, bearer, mut forge, old, _) = connected_forgejo();
    let before = service.engine.as_ref().unwrap().domain.forge_connection(FORGE);
    let new = Tokens::new(74);
    new.give(&mut forge, b"another-user", false, false);
    http_replace(&mut service, &bearer, new.bundle(true));
    let mut check = checking(&old, &new, b"another-user");
    let trace = replace_settle(&mut service, &mut forge, &mut check, &new, false);
    assert!(trace.keeps.is_empty());
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.domain.forge_connection(FORGE), before);
    assert_eq!(
        engine.domain.forges().rows[0].ended.as_deref(),
        Some(
            b"the tokens answer as another user than checked-user, this forge's: connect that user as a new forge"
                .as_slice()
        )
    );
    let rows = engine.forge_records.rows.forge(FORGE).unwrap();
    assert!(rows.get(Item::ApiToken).unwrap() == old.api.expose());
    assert!(rows.get(Item::GitToken).unwrap() == old.git.expose());
    assert!(rows.get(Item::WebhookSecret).unwrap() == old.hook.expose());
    assert_eq!(engine.forge.starts.len(), 1);
    assert_eq!(engine.values.identities.newest(FORGE.into()).unwrap().0, 1);
    tokens::erased(&mut service);
}
fn stopped_before_plan(service: &crate::Service, why: &[u8]) {
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.setup.public.is_empty() && engine.setup.staged.is_empty() && engine.setup.runs.is_empty());
    assert!(
        engine.events.iter().any(|event| match event {
            root::Event::Forge(event) => match event.as_ref() {
                domain::Event::ConnectEnded { connected: false, why: Some(found), .. } => found.as_ref() == why,
                _ => false,
            },
            _ => false,
        }),
        "events: {:?}",
        engine.events
    );
}
#[test]
fn admin_and_secret_only_replacements_stop_before_any_plan() {
    let (mut service, _, _, old, _) = connected_forgejo();
    for items in [&[(1, old.api.expose()), (2, old.api.expose())][..], &[(5, old.hook.expose())][..]] {
        let number = replace(&mut service, records::encode_pasted(items).unwrap());
        stopped_before_plan(&service, b"the pasted values are not a Forgejo's tokens");
        public::erase(service.engine.as_mut().unwrap(), number);
        crate::engine::answer(service.engine.as_mut().unwrap());
    }
}

fn app_bundle() -> (Secret, Secret) {
    let pem = Secret::new(skein_fake_forge_github::keys::pem(skein_fake_forge_github::keys::KEYS[1]).unwrap());
    let bundle = temper_client_domain::forge::forges::pasted_app_key(pem.expose()).unwrap();
    (pem, bundle)
}
fn no_key_in_diagnostics(service: &crate::Service, pem: &Secret) {
    let engine = service.engine.as_ref().unwrap();
    for secret in [pem.expose(), skein_fake_forge_github::keys::KEYS[1]] {
        for event in &engine.events {
            assert!(!contains(alloc::format!("{event:?}").as_bytes(), secret));
        }
        for note in &engine.notes {
            assert!(!contains(alloc::format!("{note:?}").as_bytes(), secret));
        }
        for request in &engine.requests {
            assert!(!contains(alloc::format!("{request:?}").as_bytes(), secret));
        }
    }
}
fn reconcile_app(service: &mut crate::Service) -> Token {
    let mut face = Face::new();
    let mut github = skein_forge_kit::github::Fake::new(7);
    let engine = service.engine.as_mut().unwrap();
    advance(engine);
    let run = *engine.setup.runs.first().expect("Connected starts reconcile").0;
    assert_eq!(engine.setup.runs.get(&run).unwrap().kind, Kind::Reconcile);
    for _ in 0_u32..16 {
        if service.engine.as_ref().unwrap().setup.runs.is_empty() {
            break;
        }
        lifecycle_tests::github_reply(service, &mut face, &mut github);
        crate::engine::iterate(service.engine.as_mut().unwrap(), Time::ZERO, Wall::EPOCH);
    }
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.setup.runs.is_empty());
    for _ in 0_u32..8 {
        advance(engine);
        assert!(engine.setup.runs.is_empty(), "one reconcile per Connected");
    }
    run
}
fn connected_github() -> (Box<crate::Service>, fixtures::ForgeFace, Token) {
    let store = fixtures::ready().engine.unwrap().store;
    let mut service = Box::new(local_service(github_config(8, Duration::from_secs(3600), None), store));
    load_routing(&mut service);
    restore_local(&mut service);
    lifecycle_tests::app(&mut service);
    lifecycle::connect(service.engine.as_mut().unwrap(), FORGE);
    let mut connection = fixtures::ForgeFace::new();
    connection.connected(&mut service);
    assert!(contains(connection.sent(), b"GET /app/installations"));
    connection.answer(&mut service, b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n[]");
    let run = reconcile_app(&mut service);
    (service, connection, run)
}
fn finish_paste(service: &mut crate::Service, pem: &Secret, success: bool) -> Option<Box<[u8]>> {
    for _ in 0_u32..32 {
        no_key_in_diagnostics(service, pem);
        let engine = service.engine.as_mut().unwrap();
        crate::engine::answer(engine);
        advance(engine);
        if engine.setup.public.is_empty() {
            break;
        }
    }
    no_key_in_diagnostics(service, pem);
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.setup.public.is_empty() && engine.setup.runs.is_empty());
    let mut why = None;
    let mut ended = 0_u32;
    while let Some(event) = engine.events.pop() {
        if let root::Event::Forge(event) = event
            && let domain::Event::ConnectEnded { connected, why: reason, .. } = *event
        {
            assert_eq!(connected, success);
            why = reason;
            ended = ended.checked_add(1).unwrap();
        }
    }
    assert_eq!(ended, 1);
    why
}
#[test]
fn github_replacement_keeps_key_connects_again_then_reconciles_once() {
    let (mut service, mut connection, first_reconcile) = connected_github();
    let (pem, bundle) = app_bundle();
    let before = service.engine.as_ref().unwrap().setup.next_write;
    let number = replace(&mut service, bundle);
    let engine = service.engine.as_ref().unwrap();
    let run = *engine.setup.public.first().unwrap().0;
    assert_eq!(engine.setup.runs.get(&run).unwrap().kind, Kind::Paste);
    let mut face = Face::new();
    for body in [
        br#"{"id":7,"slug":"app-fixture","permissions":{},"events":[]}"#.as_slice(),
        br#"{"id":9,"login":"app-fixture[bot]"}"#,
        br#"{"url":"https://app-fixture.invalid/","content_type":"json","insecure_ssl":"0"}"#,
    ] {
        face.sent(&mut service);
        face.reply(&mut service, body);
        no_key_in_diagnostics(&service, &pem);
    }
    assert!(finish_paste(&mut service, &pem, true).is_none());
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.setup.next_write.checked_sub(before).unwrap(), APP_WRITES);
    assert!(
        engine.forge_records.rows.forge(FORGE).unwrap().get(Item::AppKey).unwrap()
            == skein_fake_forge_github::keys::KEYS[1]
    );
    assert_eq!(engine.forge.starts.len(), 1, "Connect waits for carried erase");
    public::erase(service.engine.as_mut().unwrap(), number);
    crate::engine::answer(service.engine.as_mut().unwrap());
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.forge.starts.len(), 2);
    let (_, access) = engine.forge.starts.iter().last().unwrap();
    let skein_forge::credential::Credential::App { id, key } = &access.credential else { panic!("App access") };
    assert_eq!(*id, 7);
    assert!(key.expose() == skein_fake_forge_github::keys::KEYS[1]);
    connection.connected(&mut service);
    connection.answer(&mut service, b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n[]");
    let second_reconcile = reconcile_app(&mut service);
    assert_ne!(first_reconcile, second_reconcile);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(
        engine.domain.forge_connection(FORGE),
        domain::Connection::Connected { id: 9, login: b"app-fixture[bot]".as_slice().into() }
    );
    assert!(engine.setup.pastes.is_empty() && engine.setup.erases.is_empty());
}
#[test]
fn another_apps_key_stops_before_writes_and_preserves_connection() {
    let (mut service, _connection, _) = connected_github();
    let (pem, bundle) = app_bundle();
    let before = service.engine.as_ref().unwrap().setup.next_write;
    let number = replace(&mut service, bundle);
    let mut face = Face::new();
    face.sent(&mut service);
    face.reply(&mut service, br#"{"id":8,"slug":"another-app","permissions":{},"events":[]}"#);
    let why = finish_paste(&mut service, &pem, false).unwrap();
    assert!(contains(&why, b"OtherApp"));
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.setup.next_write, before);
    assert!(
        engine.forge_records.rows.forge(FORGE).unwrap().get(Item::AppKey).unwrap()
            == skein_fake_forge_github::keys::KEYS[0]
    );
    assert_eq!(
        engine.domain.forge_connection(FORGE),
        domain::Connection::Connected { id: 9, login: b"app-fixture[bot]".as_slice().into() }
    );
    public::erase(service.engine.as_mut().unwrap(), number);
    crate::engine::answer(service.engine.as_mut().unwrap());
    assert_eq!(service.engine.as_ref().unwrap().forge.starts.len(), 1);
}

#[test]
fn github_replacement_requires_a_readable_app_id() {
    let (_, bundle) = app_bundle();
    for id in [None, Some(b"bad-id".as_slice()), Some(0_u64.to_le_bytes().as_slice())] {
        let mut service = routing_service(4, Duration::from_secs(60));
        if let Some(id) = id {
            row(&mut service, Item::AppId, id);
        }
        let before = service.engine.as_ref().unwrap().setup.next_write;
        let _number = replace(&mut service, bundle.clone());
        stopped_before_plan(&service, b"its records are unreadable");
        assert_eq!(service.engine.as_ref().unwrap().setup.next_write, before);
    }
}
#[test]
fn github_replacement_requires_one_pasted_key() {
    let tokens = Tokens::new(76);
    for items in [&[(2, tokens.api.expose())][..], &[(11, tokens.api.expose()), (3, tokens.git.expose())][..]] {
        let mut service = routing_service(4, Duration::from_secs(60));
        row(&mut service, Item::AppId, &7_u64.to_le_bytes());
        let before = service.engine.as_ref().unwrap().setup.next_write;
        let _number = replace(&mut service, records::encode_pasted(items).unwrap());
        stopped_before_plan(&service, b"the pasted value is not a key");
        assert_eq!(service.engine.as_ref().unwrap().setup.next_write, before);
    }
}
#[test]
fn changed_or_missing_carried_rows_stop_before_any_plan() {
    let (_, bundle) = app_bundle();
    for (present, changed) in [(false, false), (true, true)] {
        let mut service = routing_service(4, Duration::from_secs(60));
        let mut carried = if present {
            carried(&mut service, bundle.clone())
        } else {
            jig_domain_core::Carried { number: records::CARRIED, digest: [0; 32] }
        };
        if changed {
            carried.digest[0] ^= 1;
        }
        crate::route::request(
            service.engine.as_mut().unwrap(),
            root::Request::Replace { forge: FORGE, request: REQUEST, carried },
        );
        stopped_before_plan(&service, b"the pasted value changed");
    }
}
#[test]
#[expect(clippy::match_like_matches_macro, reason = "the step subset uses explicit matches")]
fn full_replacement_answers_busy_without_reading_the_carried_row() {
    let mut service = routing_service(4, Duration::from_secs(60));
    service.engine.as_mut().unwrap().setup.env.limits.runs = 0;
    let (_, bundle) = app_bundle();
    let carried = carried(&mut service, bundle);
    let engine = service.engine.as_mut().unwrap();
    let before = engine.store.secret_requests;
    crate::route::request(engine, root::Request::Replace { forge: FORGE, request: REQUEST, carried });
    assert_eq!(engine.store.secret_requests, before);
    assert!(engine.setup.public.is_empty() && engine.setup.staged.is_empty());
    assert!(engine.events.iter().any(|event| match event {
        root::Event::Forge(event) => match event.as_ref() {
            domain::Event::ConnectAnswered {
                answer: domain::ConnectAnswer::Refused(domain::ForgeRefusal::Busy),
                ..
            } => true,
            _ => false,
        },
        _ => false,
    }));
}
