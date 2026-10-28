//! Picker host routing, with the same scripted HTTP face as the writing plans.
#![expect(clippy::disallowed_macros, reason = "test fixtures format only fake HTTP JSON")]
#![expect(clippy::disallowed_types, reason = "ordinary tests assemble fake JSON documents")]
#![expect(clippy::disallowed_methods, reason = "ordinary test assertions and fixture assembly")]

use super::lifecycle_tests::{accepted, app};
use super::*;

fn reached(service: &mut crate::Service) -> domain::ReachableAnswer {
    for _ in 0_u32..service.engine.as_ref().unwrap().events.len() {
        if let Some(root::Event::Forge(event)) = service.engine.as_mut().unwrap().events.pop()
            && let domain::Event::ReachableAnswered { answer, .. } = *event
        {
            return answer;
        }
    }
    panic!(
        "reachable terminal: runs {:?}, queued {:?}",
        service.engine.as_ref().unwrap().setup.runs,
        service.engine.as_ref().unwrap().events
    )
}
fn github_page(service: &mut crate::Service, page: u32, more: bool) -> domain::ReachableAnswer {
    public::reachable(service.engine.as_mut().unwrap(), 1, Token::new(90), Some(11), page);
    let mut face = Face::new();
    sent(&mut face, service);
    assert!(contains(&face.wire.received, b"POST /app/installations/11/access_tokens"));
    face.reply(service, br#"{"token":"ghs-fixture","expires_at":"2099-01-01T00:00:00Z"}"#);
    sent(&mut face, service);
    let path = alloc::format!("GET /installation/repositories?per_page=30&page={page}");
    assert!(contains(&face.wire.received, path.as_bytes()), "{}", String::from_utf8_lossy(&face.wire.received));
    if more {
        let mut rows = Vec::new();
        for id in 1_u32..=30 {
            rows.push(alloc::format!(
                r#"{{"id":{id},"owner":{{"id":10,"login":"acme"}},"name":"app","private":true}}"#
            ));
        }
        let body = alloc::format!(r#"{{"total_count":30,"repositories":[{}]}}"#, rows.join(","));
        face.reply(service, body.as_bytes());
    } else {
        face.reply(service, br#"{"total_count":31,"repositories":[{"id":31,"owner":{"id":10,"login":"acme"},"name":"last","private":false}]}"#);
    }
    for _ in 0_u32..4096 {
        face.turn(service);
    }
    reached(service)
}
#[test]
fn reachable_stages_github_pages_beside_a_writer_and_counts_both_against_runs() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    accepted(&mut service, &[11]);
    // A writing plan blocked on the parent's next action holds its independent slot.
    service
        .engine
        .as_mut()
        .unwrap()
        .setup
        .runs
        .insert(
            Token::new(900),
            Run {
                first: None,
                forge: 1,
                kind: Kind::Reconcile,
                internal: true,
                previous: domain::Connection::Connected { id: 9, login: b"bot".as_slice().into() },
            },
        )
        .unwrap();
    let domain::ReachableAnswer::Page(page) = github_page(&mut service, 1, true) else { panic!("page") };
    assert_eq!(page.rows.len(), 30);
    assert!(page.more);
    assert_eq!((page.rows[0].id, page.rows[0].owner_id, page.rows[0].private), (1, 10, true));
    assert!(service.engine.as_ref().unwrap().setup.runs.contains_key(&Token::new(900)));
    let domain::ReachableAnswer::Page(page) = github_page(&mut service, 2, false) else { panic!("page two") };
    assert_eq!(page.rows.len(), 1);
    assert!(!page.more);
    assert_eq!(page.rows[0].name.as_ref(), b"last");
    service.engine.as_mut().unwrap().setup.env.limits.runs = 1;
    public::reachable(service.engine.as_mut().unwrap(), 1, Token::new(91), Some(11), 1);
    assert_eq!(reached(&mut service), domain::ReachableAnswer::Refused(domain::ForgeRefusal::Busy));
}
#[test]
fn reachable_on_a_connected_forgejo_uses_its_api_token_and_filters_push_access() {
    let mut config = picker_config();
    config.limits.picker.page = 2;
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None, fixtures::forge_runtime()).unwrap();
    service.engine.as_mut().unwrap().store = fixtures::records_store();
    for _ in 0_u32..8 {
        crate::forge::records::advance(service.engine.as_mut().unwrap());
    }
    advance(service.engine.as_mut().unwrap());
    public::reachable(service.engine.as_mut().unwrap(), 1, Token::new(90), None, 1);
    let mut face = Face::new();
    sent(&mut face, &mut service);
    assert!(contains(&face.wire.received, b"GET /api/v1/settings/api"));
    face.reply(&mut service, br#"{"max_response_items":50,"default_paging_num":20}"#);
    sent(&mut face, &mut service);
    assert!(
        contains(&face.wire.received, b"GET /api/v1/user/repos?page=1&limit=2"),
        "{}",
        String::from_utf8_lossy(&face.wire.received)
    );
    face.reply(&mut service, br#"[{"id":41,"owner":{"id":10,"login":"acme"},"name":"app","private":true,"permissions":{"push":true}},{"id":42,"owner":{"id":10,"login":"acme"},"name":"readonly","private":false,"permissions":{"push":false}}]"#);
    let domain::ReachableAnswer::Page(page) = reached(&mut service) else { panic!("Forgejo page") };
    assert_eq!(page.rows.len(), 1);
    assert!(page.more);
    assert_eq!(page.rows[0].id, 41);
}

fn picker_client(service: &mut crate::Service, bearer: &[u8]) -> fixtures::http::Peer {
    use jig_client_protocol_wire::v19 as j;
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let engine = service.engine.as_mut().unwrap();
    let limits = &engine.clients.env.limits.wire;
    let request = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::Reachable(
                w::Reachable::new(&w::CEILINGS, w::ReachableParts { forge: 1, installation: None, page: 1 }).unwrap(),
            ),
        },
    )
    .unwrap();
    let family = j::FamilyRequest::new(
        limits,
        j::FamilyRequestParts { scope: j::Scope::Deployment, record: wire::requesting(&request) },
    )
    .unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(family.measure()).unwrap());
    family.encode(&mut bytes).unwrap();
    let request = j::Keyed::new(
        limits,
        j::KeyedParts {
            key: [90; 16],
            owner: j::Owner::Family(j::Family::new(limits, j::FamilyParts { number: wire::FORGE }).unwrap()),
            version: wire::CLIENT_VERSION,
            request: bytes.finish(),
            value: None,
        },
    )
    .unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(request.measure()).unwrap());
    request.encode(&mut bytes).unwrap();
    let mut client = fixtures::http::Peer::new(engine, b"POST", b"/jig/v1/request", &bytes.finish(), Some(bearer));
    client.pump(engine, true);
    client
}
fn picker_outcome(service: &crate::Service, client: &fixtures::http::Peer) -> temper_client_protocol_wire::v1::Outcome {
    use jig_client_protocol_wire::v19 as j;
    use temper_client_protocol_wire::v1 as w;
    let engine = service.engine.as_ref().unwrap();
    let at = skein_lib::bytes::find(client.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    let mut reader = skein_lib::Reader::new(&client.received()[at..]);
    let j::KeyedAnswer::FinalAnswer(answer) =
        j::KeyedAnswer::decode(&engine.clients.env.limits.wire, &mut reader).unwrap()
    else {
        panic!("final answer")
    };
    assert_eq!(
        answer.version(),
        temper_client_protocol_wire::CLIENT_VERSION,
        "core terminal: {:?}",
        j::Final::decode(&engine.clients.env.limits.wire, &mut skein_lib::Reader::new(answer.record()))
    );
    let answer = w::Answer::decode(&w::CEILINGS, &mut skein_lib::Reader::new(answer.record())).unwrap();
    answer.into_parts().outcome
}
#[test]
fn the_same_key_answers_the_same_page_without_a_run_and_is_unknown_after_store_restart() {
    use fixtures::http as peer;
    use jig_client_protocol_wire::v19 as j;
    use temper_client_protocol_wire::v1 as w;
    let mut service = local_service(picker_config(), local_store(false));
    restore_local(&mut service);
    let link = service.links().pop().unwrap();
    let engine = service.engine.as_mut().unwrap();
    let body = peer::link_body(engine, &link);
    let mut sign_in = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    sign_in.pump(engine, true);
    let j::SignInAnswer::SignedIn(answer) = sign_in.sign_in_answer(engine) else { panic!("owner sign in") };
    let bearer = peer::bearer(answer.token().as_ref().unwrap().expose());
    // This small store fixture gives the first link only its project role. Seed
    // its deployment role explicitly, as the root's public-forge fixtures do.
    let mut header = engine.domain.deployment();
    header.commits = header.commits.checked_add(1).unwrap();
    let number = header.commits;
    let mut store = core::mem::replace(&mut engine.store, fixtures::records_store());
    assert!(store.down(root::StoreRequest::Commit {
        number,
        writes: Box::new([
            root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(header)
            ))),
            root::Write::Save(root::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Roles {
                project: 0,
                holdings: Box::new([jig_domain_people::Holding { person: 1, role: jig_domain_people::Role::Owner }]),
            }))),
        ])
    }));
    let _kept = store.above.pop().unwrap();
    let mut service = local_service(picker_config(), store);
    restore_local(&mut service);
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Forge(Box::new(domain::Event::Connection {
        forge: 1,
        state: domain::Connection::Connected { id: 42, login: b"temper".as_slice().into() },
    })));
    let mut client = picker_client(&mut service, &bearer);
    assert!(
        !service.engine.as_ref().unwrap().setup.runs.is_empty(),
        "picker admitted: {:?}",
        picker_outcome(&service, &client)
    );
    let mut face = Face::new();
    sent(&mut face, &mut service);
    face.reply(&mut service, br#"{"max_response_items":50,"default_paging_num":20}"#);
    sent(&mut face, &mut service);
    face.reply(
        &mut service,
        br#"[{"id":41,"owner":{"id":10,"login":"acme"},"name":"app","private":true,"permissions":{"push":true}}]"#,
    );
    client.pump(service.engine.as_mut().unwrap(), true);
    let first = picker_outcome(&service, &client);
    let w::Outcome::Page(_) = &first else { panic!("page: {first:?}") };
    let again = picker_client(&mut service, &bearer);
    assert_eq!(picker_outcome(&service, &again), first);
    assert!(service.engine.as_ref().unwrap().setup.runs.is_empty());
    assert!(service.engine.as_ref().unwrap().setup.io.is_empty());
    let store = core::mem::replace(&mut service.engine.as_mut().unwrap().store, fixtures::records_store());
    let mut restarted = local_service(picker_config(), store);
    restore_local(&mut restarted);
    let client = picker_client(&mut restarted, &bearer);
    assert_eq!(picker_outcome(&restarted, &client), w::Outcome::Unknown);
    assert!(restarted.engine.as_ref().unwrap().setup.runs.is_empty());
}

fn picker_config() -> crate::settings::Engine {
    let mut config = fixtures::engine();
    fixtures::fill_files(&mut config);
    config
}

fn sent(face: &mut Face, service: &mut crate::Service) {
    for _ in 0_u32..1024 {
        face.turn(service);
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(
        contains(&face.wire.received, b"\r\n\r\n"),
        "request sent: component {}, loaded {}, runs {:?}, events {:?}, notes {:?}, reads {:?}, api {}",
        engine.setup.component.is_some(),
        engine.forge_records.done(),
        engine.setup.runs,
        engine.events,
        engine.notes,
        engine.setup.answers,
        engine.workers.forges[0].api_token.is_some()
    );
}

#[test]
fn qa_an_account_without_repositories_answers_no_rows_and_no_more() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    accepted(&mut service, &[11]);
    public::reachable(service.engine.as_mut().unwrap(), 1, Token::new(90), Some(11), 1);
    let mut face = Face::new();
    sent(&mut face, &mut service);
    face.reply(&mut service, br#"{"token":"ghs-fixture","expires_at":"2099-01-01T00:00:00Z"}"#);
    sent(&mut face, &mut service);
    face.reply(&mut service, br#"{"total_count":0,"repositories":[]}"#);
    let domain::ReachableAnswer::Page(page) = reached(&mut service) else { panic!("empty repository page") };
    assert!(page.rows.is_empty());
    assert!(!page.more);
}
