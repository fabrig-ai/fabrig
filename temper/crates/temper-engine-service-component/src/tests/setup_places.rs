//! Chosen and restored places cross the routing's port and file effects (protocol/setup/place.md, 2–6).
use super::*;
use crate::PlaceIo;
use temper_forge_setup::setup_file::SetupFile;

fn turn(engine: &mut crate::engine::Engine) -> Option<root::ApplicationAnswer> {
    let mut answer = None;
    while let Some(event) = engine.events.pop() {
        engine.step(event);
    }
    root::release(&mut engine.domain, &engine.env, &mut engine.requests);
    root::release(&mut engine.domain, &engine.env, &mut engine.requests);
    while let Some(request) = engine.requests.pop() {
        if let root::Request::Party(root::PartyRequest::Deliver(root::Delivery::Application { outcome, .. })) = request
        {
            answer = outcome;
            continue;
        }
        if let root::Request::Party(_) | root::Request::Bound { .. } = request {
            continue;
        }
        crate::route::request(engine, request);
    }
    engine.step(root::Event::Resume);
    engine.store.iterate();
    crate::engine::answer(engine);
    engine.domain.reclaim();
    answer
}
fn settle_answer(engine: &mut crate::engine::Engine) -> Option<root::ApplicationAnswer> {
    let mut answer = None;
    for _ in 0_u32..256 {
        let found = turn(engine);
        if found.is_some() {
            answer = found;
        }
    }
    answer
}
fn settle(engine: &mut crate::engine::Engine) {
    let _answer = settle_answer(engine);
}
fn ask_owned(service: &mut crate::Service) {
    sign_owner(service);
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Party(root::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: 1,
        key: [61; 16],
        digest: [61; 32],
        scope: people::ApplicationScope::Deployment,
        request: root::Application::Forge(domain::Ask::SetUp {
            forge: domain::Making::New(domain::ForgeAsk::Owned),
            first: domain::SetUpFirst::Nothing,
        }),
        carried: None,
    }));
    settle(engine);
}
fn choice(engine: &mut crate::engine::Engine) -> Token {
    let Some(PlaceIo::Choose { request }) = engine.take_place_io() else {
        panic!("routing chooses before making the record")
    };
    request
}
fn kept(engine: &mut crate::engine::Engine) -> domain::ForgeRecord {
    let rows = fixtures::load(&mut engine.store, root::Range::Forge);
    for row in rows {
        if let root::Record::Forge { row } = row
            && let domain::Stored::Forge(record) = *row
        {
            return record;
        }
    }
    panic!("chosen place is kept")
}
#[test]
fn owned_setup_chooses_then_keeps_one_place_and_registers_its_public_client() {
    let mut service = fresh_with_places(true, false);
    ask_owned(&mut service);
    let engine = service.engine.as_mut().unwrap();
    for port in [3000, 3001, 3002] {
        let request = choice(engine);
        engine.place_port(request, Some(port));
        assert!(engine.domain.forges().rows.is_empty(), "deployment listeners and fixed redirects are retried");
    }
    let request = choice(engine);
    engine.place_port(request, Some(4567));
    settle(engine);
    let record = kept(engine);
    assert_eq!(record.place.api, domain::Address { host: b"127.0.0.1".as_slice().into(), port: 4567, tls: None });
    assert_eq!(record.place.git.as_ref(), b"http://127.0.0.1:4567");
    assert!(record.place.web.is_none());
    let Some(PlaceIo::Write { forge, setup, .. }) = engine.take_place_io() else {
        panic!("the file is written before the group starts")
    };
    assert_eq!(forge, record.number);
    let SetupFile::Owned(owned) = setup.as_ref() else { panic!("owned file") };
    assert_eq!(owned.listen.port(), 4567);
    assert_eq!(owned.person.login.as_ref(), b"deployment-owner");
    assert_eq!(owned.person.email.as_ref(), b"deployment-owner@localhost");
    assert!(owned.fetch.forgejo && owned.fetch.runner && owned.public.is_none());
    assert_eq!(owned.sign_in.as_deref(), Some(b"forgejo".as_slice()));
    let runner = owned.runner.as_ref().unwrap();
    assert_eq!(runner.capacity, 1);
    assert_eq!(runner.labels.as_ref(), &[Box::from(b"temper-host:host".as_slice())]);
    assert_eq!(runner.path.as_ref(), b"/usr/local/bin:/usr/bin:/bin");
    assert!(engine.box_link.write.is_none(), "start waits for the file write");
    let kept_file = *setup;
    engine.place_file(forge, Ok(kept_file.clone()));
    settle(engine);
    assert_eq!(engine.box_link.write.take().unwrap().as_ref(), b"start 1\n");
    fixtures::listening(&mut service);
    assert_eq!(
        authorize(&mut service, 4567),
        jig_client_protocol_wire::AuthorizeAnswer::SignedOut,
        "a public runtime client is admitted without a secret"
    );
    let engine = service.engine.as_mut().unwrap();
    crate::places::forget(engine, forge);
    let jig_client_protocol_wire::AuthorizeAnswer::Refused(why) = authorize(&mut service, 4567) else {
        panic!("removed client is refused")
    };
    assert_eq!(why.why(), &jig_client_protocol_wire::AuthorizeWhy::UnknownClient);
    let mut restarted = fresh_with_places(true, false);
    fixtures::keep_forge(&mut restarted, record);
    let engine = restarted.engine.as_mut().unwrap();
    let Some(PlaceIo::Read { forge: restored, kind, .. }) = engine.take_place_io() else {
        panic!("restart reads a kept setup, never chooses another port")
    };
    assert_eq!(restored, forge);
    assert_eq!(kind, temper_forge_setup::machine::BoxKind::Owned);
    engine.place_file(restored, Ok(kept_file));
    settle(engine);
    assert!(engine.take_place_io().is_none());
    fixtures::listening(&mut restarted);
    assert_eq!(
        authorize(&mut restarted, 4567),
        jig_client_protocol_wire::AuthorizeAnswer::SignedOut,
        "startup re-adds the kept public client"
    );
}
#[expect(clippy::disallowed_macros, reason = "the protocol world formats its authorization query")]
fn authorize(service: &mut crate::Service, port: u16) -> jig_client_protocol_wire::AuthorizeAnswer {
    use jig_client_protocol_wire as wire;
    let limits = service.engine.as_ref().unwrap().clients.env.limits.wire;
    let query = format!(
        "response_type=code&client_id=forgejo&redirect_uri=http%3A%2F%2F127.0.0.1%3A{port}%2Fuser%2Foauth2%2Ftemper%2Fcallback&scope=openid&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256"
    );
    let record = wire::Authorize::new(&limits, wire::AuthorizeParts { query: query.into_bytes().into() }).unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut bytes).unwrap();
    let peer = crate::tests::provider::request(
        service,
        "POST",
        "/jig/v1/authorize",
        "Content-Type: application/octet-stream\r\nJig-Client: browser\r\n",
        &bytes.finish(),
    );
    let bytes = peer.received();
    assert!(bytes.starts_with(b"HTTP/1.1 200"), "provider response: {bytes:?}");
    let at = skein_lib::bytes::find(bytes, b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    wire::AuthorizeAnswer::decode(&limits, &mut skein_lib::Reader::new(&bytes[at..])).unwrap()
}
#[test]
fn eight_conflicting_choices_refuse_owned_setup_busy_without_a_record() {
    let mut service = fresh_with_places(true, false);
    ask_owned(&mut service);
    let engine = service.engine.as_mut().unwrap();
    for _ in 0_u32..8 {
        let request = choice(engine);
        engine.place_port(request, Some(3002));
    }
    let answer = settle_answer(engine);
    assert_eq!(answer, Some(root::ApplicationAnswer::SetUp(domain::SetUpAnswer::Refused(domain::SetUpRefusal::Busy))));
    assert!(engine.take_place_io().is_none());
    assert!(engine.domain.forges().rows.is_empty());
    let rows = fixtures::load(&mut engine.store, root::Range::Forge);
    for row in rows {
        if let root::Record::Forge { row } = row
            && let domain::Stored::Forge(_) = *row
        {
            panic!("busy made no forge record");
        }
    }
}
#[test]
fn playground_start_keeps_its_chosen_port_and_data_git_program() {
    let mut service = fresh_with_places(false, true);
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        request: Token::new(71),
        token: Token::new(72),
        retention: Duration::from_secs(60),
        ask: domain::Ask::StartPlayground,
        role: None,
        carried: None,
    })));
    settle(engine);
    let request = choice(engine);
    engine.place_port(request, Some(4568));
    settle(engine);
    let record = kept(engine);
    assert_eq!(record.mode, domain::Mode::Fake);
    assert_eq!(record.place.api.port, 4568);
    assert_eq!(record.place.git.as_ref(), b"http://127.0.0.1:4568");
    assert_eq!(record.git_program.as_deref(), Some(b"/data/demo/bin/git".as_slice()));
    let Some(PlaceIo::Write { forge, setup, .. }) = engine.take_place_io() else { panic!("fake setup is written") };
    let SetupFile::Fake(fake) = setup.as_ref() else { panic!("fake file") };
    assert_eq!(fake.listen.port(), 4568);
    assert_eq!(fake.git.as_ref(), b"/data/demo/bin/git");
    engine.place_file(forge, Ok(*setup));
    settle(engine);
    assert_eq!(engine.box_link.write.take().unwrap().as_ref(), b"start 1\n");
    assert!(engine.take_place_io().is_none());
}
