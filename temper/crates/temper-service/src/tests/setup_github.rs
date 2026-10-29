//! GitHub service composition (temper/docs/design/protocol/setup.md, 17).
use super::*;
use jig_domain_people as people;
use jig_protocol_engine_clients as clients;
use skein_lib::{Reader, ReplyTo};
use temper_client_protocol_wire::{self as wire, v1 as w};

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
                        reply => panic!("unexpected family web reply: {reply:?}"),
                    }
                }
                root::Request::Party(_) => {}
                request => crate::route::request(engine, request),
            }
        }
        engine.step(root::Event::Resume);
        engine.store.iterate();
        crate::engine::answer(engine);
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
