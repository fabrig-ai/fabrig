//! GitHub service composition (temper/docs/design/protocol/setup.md, 17).
use super::*;
use jig_domain_people as people;
use jig_protocol_engine_clients as clients;
use skein_lib::{Reader, ReplyTo};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[path = "setup_github_render.rs"]
mod render_tests;

const FORGE: u16 = 1;
const INSTALLATION: u64 = 22;
const REPLY: ReplyTo = ReplyTo::new(Token::new(91));
// Each scenario has two live links at most; leave room for the setup host's reads.
const STATES: u32 = 8;
const LIFE: Duration = Duration::from_secs(3600);
// The service fixtures use 256 passes to drain a root's store and child decisions.
const PASSES: u32 = 256;
// Adoption names forge/host/owner/repository/branch and two generated branch parts.
const ADOPTION_SEGMENTS: u32 = 7;
const LIVE: &[u8] = br#"[{"id":22,"account":{"id":122,"login":"account-fixture","type":"Organization"},"created_at":"1970-01-01T00:00:00Z","suspended_at":null}]"#;
const MINT: &[u8] = br#"{"token":"artificial-installation-token","expires_at":"1970-01-01T01:00:00Z"}"#;

fn owner_service() -> crate::Service {
    let mut bootstrap = fixtures::ready();
    let mut header = bootstrap.engine.as_ref().unwrap().domain.deployment();
    header.commits = header.commits.checked_add(1).unwrap();
    let mut store = bootstrap.engine.take().unwrap().store;
    assert!(store.down(root::StoreRequest::Commit {
        number: header.commits,
        writes: Box::new([
            root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(header),
            ))),
            root::Write::Save(root::Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
                project: 0,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            }))),
        ]),
    }));
    let _committed = store.above.pop().unwrap();
    let mut config = github_config(STATES, LIFE, None);
    config.jig.limits.core.authority.segments = ADOPTION_SEGMENTS;
    let mut service = local_service(config, store);
    load_routing(&mut service);
    restore_local(&mut service);
    service.engine.as_mut().unwrap().step(root::Event::Party(root::PartyInput::SignedIn {
        held: None,
        reply_to: REPLY,
        identity: people::Identity {
            key: people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    let _signed_in = pump(&mut service);
    service
}

fn pump(service: &mut crate::Service) -> Option<w::Outcome> {
    let engine = service.engine.as_mut().unwrap();
    let mut answer = None;
    for _ in 0..PASSES {
        if let Some(event) = engine.events.pop() {
            engine.step(event);
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        while let Some(request) = engine.requests.pop() {
            match request {
                root::Request::Party(root::PartyRequest::Deliver(root::Delivery::Application { to, outcome })) => {
                    assert_eq!(to, REPLY);
                    let bytes = crate::family::answer(outcome, &w::CEILINGS, &engine.clients.env.limits.wire).unwrap();
                    let record = w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
                    assert!(answer.is_none(), "one terminal per family call");
                    answer = Some(record.into_parts().outcome);
                }
                root::Request::Party(root::PartyRequest::Deliver(root::Delivery::WebReply { reply, .. })) => {
                    match reply {
                        people::Reply::SignedIn { .. } => {}
                        reply @ (people::Reply::Signer(_)
                        | people::Reply::SignedOut
                        | people::Reply::Outcome(_)
                        | people::Reply::Again(_)
                        | people::Reply::Full { .. }
                        | people::Reply::Refused(_)) => panic!("unexpected family web reply: {reply:?}"),
                    }
                }
                request @ root::Request::Party(root::PartyRequest::Deliver(root::Delivery::CarriedUnused {
                    ..
                })) => {
                    crate::route::request(engine, request);
                }
                root::Request::Party(_) => {}
                request @ (root::Request::Unsettled(_)
                | root::Request::Bound { .. }
                | root::Request::StopGroup { .. }
                | root::Request::SetUpGroup { .. }
                | root::Request::Return { .. }
                | root::Request::SetUp { .. }
                | root::Request::Connect { .. }
                | root::Request::Manifest { .. }
                | root::Request::Reachable { .. }
                | root::Request::Accept { .. }
                | root::Request::Install { .. }
                | root::Request::EraseCarried { .. }
                | root::Request::Setup { .. }
                | root::Request::Store(_)
                | root::Request::Worker(_)
                | root::Request::Forge { .. }
                | root::Request::Account(_)
                | root::Request::ToChild(_)
                | root::Request::Busy(_)
                | root::Request::Drained
                | root::Request::Stop) => crate::route::request(engine, request),
            }
        }
        engine.step(root::Event::Resume);
        engine.store.iterate();
        crate::engine::answer(engine);
        engine.clients.advance(&mut engine.events, &engine.workers.forges, engine.demo.as_ref(), &mut engine.accounts);
        engine.clients.secrets_step(&mut engine.store);
        advance(engine);
        engine.forge.iterate(engine.env.now, engine.env.wall);
        engine.forge.down(connection::Request::Offer);
        crate::engine::forge_pass(engine);
        engine.domain.reclaim();
    }
    answer
}

fn request(
    service: &mut crate::Service,
    key: u8,
    scope: people::ApplicationScope,
    request: w::Request,
) -> Option<w::Outcome> {
    let record = wire::requesting(&w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap());
    let engine = service.engine.as_mut().unwrap();
    let event = crate::family::to_root(
        clients::ToApplication::Request {
            reply_to: REPLY,
            sign_in: 1,
            key: [key; 16],
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            digest: skein_lib::sha256::digest(&record),
            record,
            carried: None,
            scope,
        },
        &engine.workers.forges,
        None,
        &w::CEILINGS,
    )
    .unwrap();
    engine.step(event);
    pump(service)
}

fn install(service: &mut crate::Service, key: u8) -> w::Outcome {
    request(
        service,
        key,
        people::ApplicationScope::Deployment,
        w::Request::Install(w::Install::new(&w::CEILINGS, w::InstallParts { forge: FORGE, back: None }).unwrap()),
    )
    .expect("install answer")
}

fn app_service() -> crate::Service {
    let mut service = owner_service();
    app(&mut service);
    show_connected(&mut service);
    service
}

fn show_connected(service: &mut crate::Service) {
    service.engine.as_mut().unwrap().step(root::Event::Forge(Box::new(domain::Event::Connection {
        forge: FORGE,
        state: domain::Connection::Connected { id: 9, login: b"app-fixture[bot]".as_slice().into() },
    })));
    let _settled = pump(service);
}

fn link_state(answer: &w::Outcome) -> &[u8] {
    let w::Outcome::Link(link) = answer else { panic!("install link: {answer:?}") };
    link.address().strip_prefix(b"http://127.0.0.1:3000/apps/app-fixture/installations/new?state=").unwrap()
}

fn accept_link(service: &mut crate::Service, answer: &w::Outcome) {
    installation(service, INSTALLATION, Change::Created);
    let mut peer = visit(service, b"GET", &setup_target(link_state(answer)));
    let mut face = Face::new();
    face.sent(service);
    face.reply(service, br#"{"id":22,"app_id":7,"account":{"id":122,"login":"account-fixture","type":"Organization"},"target_type":"Organization","created_at":"1970-01-01T00:00:00Z","suspended_at":null}"#);
    let _settled = pump(service);
    pump_visit(service, &mut peer);
    status(&peer, b"200");
}

fn connect_request(app: u64) -> w::Request {
    w::Request::Connect(
        w::Connect::new(
            &w::CEILINGS,
            w::ConnectParts {
                forge: FORGE,
                how: w::How::GithubApp(w::GitHubApp::new(&w::CEILINGS, w::GitHubAppParts { id: app }).unwrap()),
            },
        )
        .unwrap(),
    )
}

fn paste_success(service: &mut crate::Service, face: &mut Face) {
    for body in [
        br#"{"id":7,"slug":"app-fixture","permissions":{},"events":[]}"#.as_slice(),
        br#"{"id":9,"login":"app-fixture[bot]"}"#,
        br#"{"url":"https://app-fixture.invalid/","content_type":"json","insecure_ssl":"0"}"#,
    ] {
        face.sent(service);
        face.reply(service, body);
    }
    let _settled = pump(service);
}

fn http_owner() -> (crate::Service, Box<[u8]>) {
    use jig_client_protocol_wire::v19 as j;
    let mut service = owner_service();
    let link = service.links().pop().unwrap();
    let engine = service.engine.as_mut().unwrap();
    let body = fixtures::http::link_body(engine, &link);
    let mut peer = fixtures::http::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine, true);
    let j::SignInAnswer::SignedIn(answer) = peer.sign_in_answer(engine) else { panic!("owner sign in") };
    let bearer = fixtures::http::bearer(answer.token().as_ref().unwrap().expose());
    (service, bearer)
}

fn http_paste(service: &mut crate::Service, bearer: &[u8], key: u8, app: u64) -> w::Outcome {
    use jig_client_protocol_wire::v19 as j;
    let (_, bundle) = carried_pem();
    let engine = service.engine.as_mut().unwrap();
    let limits = &engine.clients.env.limits.wire;
    let record = wire::requesting(
        &w::Requesting::new(&w::CEILINGS, w::RequestingParts { request: connect_request(app) }).unwrap(),
    );
    let family = j::FamilyRequest::new(limits, j::FamilyRequestParts { scope: j::Scope::Deployment, record }).unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(family.measure()).unwrap());
    family.encode(&mut bytes).unwrap();
    let request = j::Keyed::new(
        limits,
        j::KeyedParts {
            key: [key; 16],
            owner: j::Owner::Family(j::Family::new(limits, j::FamilyParts { number: wire::FORGE }).unwrap()),
            version: wire::CLIENT_VERSION,
            request: bytes.finish(),
            value: Some(bundle),
        },
    )
    .unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(request.measure()).unwrap());
    request.encode(&mut bytes).unwrap();
    let mut peer = fixtures::http::Peer::new(engine, b"POST", b"/jig/v1/request", &bytes.finish(), Some(bearer));
    peer.pump(engine, true);
    let at = skein_lib::bytes::find(peer.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    let mut reader = Reader::new(&peer.received()[at..]);
    let j::KeyedAnswer::FinalAnswer(answer) =
        j::KeyedAnswer::decode(&engine.clients.env.limits.wire, &mut reader).unwrap()
    else {
        panic!("family terminal")
    };
    w::Answer::decode(&w::CEILINGS, &mut Reader::new(answer.record())).unwrap().into_parts().outcome
}

#[test]
fn replaying_install_keeps_its_link_after_one_acceptance() {
    let mut service = app_service();
    let first = install(&mut service, 1);
    assert_eq!(install(&mut service, 1), first);
    accept_link(&mut service, &first);
    assert_eq!(install(&mut service, 1), first);
    let peer = visit(&mut service, b"GET", &setup_target(link_state(&first)));
    status(&peer, b"410");
    assert_eq!(values(&service).as_ref(), [INSTALLATION]);
}

#[test]
fn independent_install_states_each_accept_the_same_set_member() {
    let mut service = app_service();
    let first = install(&mut service, 1);
    let second = install(&mut service, 2);
    assert_ne!(first, second);
    accept_link(&mut service, &second);
    accept_link(&mut service, &first);
    for answer in [&second, &first] {
        let peer = visit(&mut service, b"GET", &setup_target(link_state(answer)));
        status(&peer, b"410");
    }
    assert_eq!(values(&service).as_ref(), [INSTALLATION]);
    assert_eq!(service.engine.as_ref().unwrap().domain.forges().rows[0].installations.len(), 1);
}

#[test]
fn paste_replay_and_refused_replacement_keep_the_connection_and_write_counts() {
    // Paste keeps AppId, AppSlug, AppKey, User and Origin once; Accept keeps one set.
    const PASTE_WRITES: u64 = 5;
    const ACCEPT_WRITES: u64 = 1;
    // Each submitted paste carries a fresh row, erased even on replay or refusal.
    const CARRIED_ERASES: u32 = 4;
    let (mut service, bearer) = http_owner();
    let before = service.engine.as_ref().unwrap().setup.next_write;
    let first = http_paste(&mut service, &bearer, 1, 7);
    assert_eq!(first, w::Outcome::Connecting);
    paste_success(&mut service, &mut Face::new());
    show_connected(&mut service);
    assert_eq!(http_paste(&mut service, &bearer, 1, 7), first);
    let refused = http_paste(&mut service, &bearer, 2, u64::MAX);
    let w::Outcome::ConnectRefused(_) = refused else { panic!("replacement refused: {refused:?}") };
    assert_eq!(http_paste(&mut service, &bearer, 1, 7), first);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.setup.next_write.checked_sub(before).unwrap(), PASTE_WRITES);
    assert_eq!(engine.forge.starts.len(), 1);
    assert_eq!(engine.store.erases, CARRIED_ERASES);
    assert_eq!(
        engine.domain.forge_connection(FORGE),
        domain::Connection::Connected { id: 9, login: b"app-fixture[bot]".as_slice().into() }
    );
    assert_eq!(engine.forge_records.rows.forge(FORGE).unwrap().get(Item::AppId).unwrap(), 7_u64.to_le_bytes());
    let link = install(&mut service, 3);
    assert_eq!(install(&mut service, 3), link);
    accept_link(&mut service, &link);
    let peer = visit(&mut service, b"GET", &setup_target(link_state(&link)));
    status(&peer, b"410");
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.setup.next_write.checked_sub(before).unwrap(), PASTE_WRITES + ACCEPT_WRITES);
    assert_eq!(engine.store.erases, CARRIED_ERASES);
    for json in [false, true] {
        let printed = render_tests::status(&service, json);
        assert_eq!(printed.status, 0);
        assert!(contains(&printed.stdout, b"app-fixture"));
    }
}

fn accepted_token() -> crate::Service {
    let mut service = app_service();
    let link = install(&mut service, 1);
    accept_link(&mut service, &link);
    let engine = service.engine.as_mut().unwrap();
    engine.forge.above.push(connection::Event::GitToken {
        forge: FORGE,
        installation: INSTALLATION,
        account: connection::Account { id: 122, login: b"account-fixture".as_slice().into() },
        token: Secret::new(b"artificial-installation-token".as_slice().into()),
        expires: Wall::from_nanos(3_600_000_000_000),
    });
    engine.forge.above.push(connection::Event::Connected { forge: FORGE });
    crate::engine::forge_pass(engine);
    let _settled = pump(&mut service);
    service
}

#[test]
fn suspension_withdraws_the_accepted_installations_git_access() {
    let mut service = accepted_token();
    installation(&mut service, INSTALLATION, Change::Suspended);
    let _settled = pump(&mut service);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(crate::forge::tokens::account(&engine.values.tokens, FORGE, b"account-fixture"), None);
    let rows = engine.domain.forges().rows;
    assert!(rows[0].installations[0].1.suspended);
    assert!(!rows[0].installations[0].1.stray);
    for json in [false, true] {
        let printed = render_tests::status(&service, json);
        assert_eq!(printed.status, 0);
        assert!(contains(&printed.stdout, b"suspended"));
    }
}

#[test]
fn key_revocation_stops_the_forge_and_withdraws_git_access() {
    let mut service = accepted_token();
    let engine = service.engine.as_mut().unwrap();
    engine.forge.above.push(connection::Event::Stopped { forge: FORGE, why: connection::Stopped::KeyRefused });
    crate::engine::forge_pass(engine);
    let _settled = pump(&mut service);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(
        engine.domain.forge_connection(FORGE),
        domain::Connection::Stopped { reason: b"KeyRefused".as_slice().into() }
    );
    for json in [false, true] {
        let printed = render_tests::status(&service, json);
        assert_eq!(printed.status, 0);
        assert!(contains(&printed.stdout, b"KeyRefused"));
    }
    assert_eq!(crate::forge::tokens::account(&engine.values.tokens, FORGE, b"account-fixture"), None);
}

#[test]
fn a_successful_paste_retry_clears_the_rows_readable_failure() {
    const REASON: &[u8] = b"GitHub refused the App's id and key (401): check both are the same App's.";
    let (mut service, bearer) = http_owner();
    let _connecting = http_paste(&mut service, &bearer, 1, 7);
    let mut face = Face::new();
    face.sent(&mut service);
    face.reply_status(&mut service, b"401 Unauthorized", b"{}");
    let _failed = pump(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().domain.forges().rows[0].ended.as_deref(), Some(REASON));
    for json in [false, true] {
        let printed = render_tests::status(&service, json);
        assert_eq!(printed.status, 0);
        assert!(contains(&printed.stdout, REASON));
    }
    let _retry = http_paste(&mut service, &bearer, 2, 7);
    paste_success(&mut service, &mut Face::new());
    show_connected(&mut service);
    assert_eq!(service.engine.as_ref().unwrap().domain.forges().rows[0].ended, None);
    for json in [false, true] {
        let printed = render_tests::status(&service, json);
        assert_eq!(printed.status, 0);
        assert!(!contains(&printed.stdout, REASON));
        assert!(contains(&printed.stdout, b"app-fixture"));
    }
}

fn adopt_request(owner: &[u8]) -> w::Request {
    w::Request::Adopt(
        w::Adopt::new(
            &w::CEILINGS,
            w::AdoptParts {
                project: 1,
                home: false,
                forge: FORGE,
                owner: owner.into(),
                name: b"unadopted".as_slice().into(),
                prefix: b"temper".as_slice().into(),
                role: w::Role::Adopted,
                landing: b"main".as_slice().into(),
                ci: false,
                checks: List::with_capacity(0),
            },
        )
        .unwrap(),
    )
}

fn connected(service: &mut crate::Service, accepted_ids: &[u64], listing: &[u8]) -> Face {
    app(service);
    accepted(service, accepted_ids);
    let engine = service.engine.as_mut().unwrap();
    let origin = records::origin(connection::settings::Kind::GitHub, &engine.workers.forges[0].api);
    let access = records::access(
        connection::settings::Kind::GitHub,
        &origin,
        engine.forge_records.rows.forge(FORGE).unwrap(),
        &engine.forge_records.bounds,
        false,
    )
    .unwrap();
    engine.forge.down(connection::Request::Connect { forge: FORGE, access: access.access });
    engine.forge.down(connection::Request::Installations { forge: FORGE, accepted: access.accepted });
    let mut face = Face::new();
    for _ in 0..PASSES {
        connection_turn(service, &mut face);
    }
    connection_reply(service, &mut face, listing);
    if !accepted_ids.is_empty() {
        minted(service, &mut face);
        minted(service, &mut face);
        connection_reply(service, &mut face, br#"{"data":{"viewer":{"databaseId":9,"login":"app-fixture[bot]"}}}"#);
    }
    let _connected = pump(service);
    face
}

fn minted(service: &mut crate::Service, face: &mut Face) {
    let head = b"HTTP/1.1 201 Created\r\nContent-Length: ";
    let length = skein_lib::Decimal::of(u64::try_from(MINT.len()).unwrap());
    let mut bytes = skein_lib::Writer::new(
        head.len()
            .checked_add(length.as_bytes().len())
            .unwrap()
            .checked_add(4)
            .unwrap()
            .checked_add(MINT.len())
            .unwrap(),
    );
    bytes.put(head).unwrap();
    bytes.put(length.as_bytes()).unwrap();
    bytes.put(b"\r\n\r\n").unwrap();
    bytes.put(MINT).unwrap();
    face.wire.write(&bytes.finish());
    for _ in 0..PASSES {
        connection_turn(service, face);
    }
}

#[test]
fn unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap() {
    for (name, ids, listing, owner) in [
        ("unaccepted", &[][..], b"[]".as_slice(), b"account-fixture".as_slice()),
        ("other account", &[INSTALLATION][..], LIVE, b"other-account".as_slice()),
    ] {
        let mut service = owner_service();
        let _connection = connected(&mut service, ids, listing);
        let answer = request(&mut service, 1, people::ApplicationScope::Project(1), adopt_request(owner));
        let Some(w::Outcome::ForgeRefused(refusal)) = answer else {
            let engine = service.engine.as_ref().unwrap();
            panic!(
                "{name}: {answer:?}, blocked {}, pending {:?}, events {}, ready {}, connection {:?}",
                engine.blocked,
                engine.pending,
                engine.events.len(),
                engine.domain.ready(),
                engine.domain.forge_connection(FORGE)
            )
        };
        // Known gap, temper-07: NotInstalled projects to NoAccess on the family wire.
        assert_eq!(refusal.reason(), &w::ForgeReason::NoAccess, "{name}");
        assert!(
            service.engine.as_ref().unwrap().domain.forges().rows[0].repositories.is_empty(),
            "{name}: no adoption"
        );
    }
}
