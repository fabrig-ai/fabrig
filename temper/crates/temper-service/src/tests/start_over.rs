//! GitHub setup resumed or removed from durable App records
//! (temper/docs/design/draft/playground-reset.md, 6.4).
use crate::{engine::Engine, setup, tests::fixtures};
use alloc::boxed::Box;
use jig_protocol_store::{SecretOwner, Secrets};
use skein_forge_setup::{github::manifest::Target, host};
use skein_lib::{Duration, Secret, Time, Token, Wall};
use temper_engine_domain as root;
use temper_engine_domain_forge as domain;
use temper_protocol_records::{self as records, Item};

const FORGE: u16 = 1;
// The existing setup service fixtures drain a socket in 1024 single-loop passes.
const PASSES: u32 = 1024;
const RUN: Token = Token::new(80);

fn config() -> crate::settings::Engine {
    let mut config = fixtures::engine();
    config.forges[0].kind = crate::settings::Kind::GitHub;
    config.forges[0].api = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()));
    config.forges[0].web = Some(b"http://127.0.0.1:3000".as_slice().into());
    config.forges[0].api_token = None;
    config.forges[0].user = 0;
    config.records_only = Box::new([FORGE]);
    config.identities[0].token = Secret::new(Box::new([]));
    config.setup = Box::new([]);
    fixtures::fill_files(&mut config);
    fixtures::reseed(&mut config);
    config
}

fn open(mut store: crate::store::Store) -> crate::Service {
    store.above.clear();
    let mut runtime = fixtures::forge_runtime();
    runtime.get_mut(0).unwrap().address = ([127, 0, 0, 1], 3000).into();
    let mut service = crate::Service::engine(config(), &fixtures::seeds(), None, runtime, fixtures::shell()).unwrap();
    let engine = service.engine.as_mut().unwrap();
    engine.store = fixtures::reopened_store(store, &engine.env.limits);
    fixtures::restart(&mut service);
    service
}

fn host_turn(engine: &mut Engine) {
    engine.store.iterate();
    crate::engine::answer(engine);
    setup::advance(engine);
}

fn start(engine: &mut Engine, plan: host::Plan) {
    engine.setup.staged.insert(RUN, plan).unwrap();
    crate::route::request(engine, root::Request::Setup { run: RUN });
}

fn terminal(engine: &mut Engine) -> root::SetupEnd {
    for _ in 0..PASSES {
        host_turn(engine);
        for _ in 0..engine.events.len() {
            let event = engine.events.pop().unwrap();
            if let root::Event::Setup { run: RUN, end } = event {
                return end;
            }
            engine.events.push(event);
        }
    }
    panic!("setup plan terminal");
}

/// The setup host's scripted HTTP peer, on the shared public fake App key.
struct GitHub {
    owner: Option<Token>,
    wire: skein_world::stream::Wire,
}
impl GitHub {
    fn new() -> Self {
        Self { owner: None, wire: skein_world::stream::Wire::new(7) }
    }
    fn turn(&mut self, engine: &mut Engine) {
        while let Some(request) = engine.setup.io.pop() {
            match request {
                skein_io::Request::Connect { owner, .. } => {
                    self.owner = Some(owner);
                    self.wire = skein_world::stream::Wire::new(7);
                    engine.setup.up(skein_io::Event::Connecting { owner, socket: Token::new(123) });
                    engine.setup.up(skein_io::Event::Connected { owner });
                }
                skein_io::Request::Stream { down, .. } => self.wire.take(down),
                skein_io::Request::Close { .. } | skein_io::Request::Abort { .. } => {
                    engine.setup.up(skein_io::Event::Closed { owner: self.owner.take().unwrap() });
                }
                skein_io::Request::Listen { .. }
                | skein_io::Request::Bind { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. } => panic!("setup host socket operation"),
            }
        }
        if let Some(owner) = self.owner
            && let Some(up) = self.wire.answer()
        {
            engine.setup.up(skein_io::Event::Stream { owner, up });
        }
        host_turn(engine);
    }
    fn reply(&mut self, engine: &mut Engine, path: &[u8], body: &[u8]) {
        for _ in 0..PASSES {
            self.turn(engine);
        }
        assert!(skein_lib::bytes::find(&self.wire.received, path).is_some(), "expected GitHub route");
        let size = skein_lib::Decimal::of(u64::try_from(body.len()).unwrap());
        self.wire.write(&bytes(&[b"HTTP/1.1 200 OK\r\nContent-Length: ", size.as_bytes(), b"\r\n\r\n", body]));
        for _ in 0..PASSES {
            self.turn(engine);
        }
    }
}

fn converted() -> crate::Service {
    let mut service = open(fixtures::records_store_with(&config()));
    let engine = service.engine.as_mut().unwrap();
    start(engine, host::Plan::Manifest { forge: FORGE, target: Target::Personal });
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Manifest { state }, .. } = terminal(engine) else {
        panic!("manifest state")
    };
    start(engine, host::Plan::Convert { forge: FORGE, code: Secret::new(b"fixture-code".as_slice().into()), state });
    let key = skein_fake_forge_github::keys::KEYS[0];
    let length =
        skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, u32::try_from(key.len()).unwrap())
            .unwrap();
    let mut encoded = skein_lib::Writer::new(usize::try_from(length).unwrap());
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, key, &mut encoded).unwrap();
    let body = bytes(&[br#"{"id":7,"slug":"app-fixture","owner":{"id":1,"login":"owner-fixture"},"pem":"-----BEGIN RSA PRIVATE KEY-----\n"#, &encoded.finish(), br#"\n-----END RSA PRIVATE KEY-----\n","client_id":"fixture","client_secret":"unused","webhook_secret":"unused"}"#]);
    let mut github = GitHub::new();
    github.reply(engine, b"/app-manifests/", &body);
    github.reply(engine, b"GET /app ", br#"{"id":7,"slug":"app-fixture"}"#);
    github.reply(engine, b"/users/", br#"{"id":9,"login":"app-fixture[bot]"}"#);
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Install { .. }, .. } = terminal(engine) else {
        panic!("converted App ready to install")
    };
    for item in [Item::AppId, Item::AppSlug, Item::AppKey, Item::User, Item::Origin] {
        assert!(engine.forge_records.rows.forge(FORGE).unwrap().get(item).is_ok(), "converted record kept");
    }
    assert!(engine.forge.accepted.get(0).unwrap().1.is_empty(), "cut before installation");
    service
}

fn restarted() -> crate::Service {
    let previous = converted().engine.unwrap();
    open(previous.store)
}

#[test]
fn github_goes_on_from_its_app_records_after_a_restart() {
    let mut service = restarted();
    let engine = service.engine.as_mut().unwrap();
    let (number, access) = engine.forge.starts.get(0).expect("restart connected from records");
    assert_eq!((*number, access.user), (FORGE, 9));
    let skein_forge::credential::Credential::App { id, .. } = &access.credential else { panic!("App access") };
    assert_eq!(*id, 7);
    assert!(engine.forge.accepted.get(0).unwrap().1.is_empty());
    engine.step(root::Event::Forge(Box::new(domain::Event::Connection {
        forge: FORGE,
        state: domain::Connection::Connected { id: 9, login: b"app-fixture[bot]".as_slice().into() },
    })));
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        request: Token::new(91),
        token: Token::new(92),
        retention: Duration::from_secs(60),
        role: Some(jig_domain_people::Role::Owner),
        carried: None,
        ask: domain::Ask::Install { forge: FORGE, back: None },
    })));
    let mut links = 0_u32;
    for _ in 0..PASSES {
        host_turn(engine);
        for _ in 0..engine.events.len() {
            let event = engine.events.pop().unwrap();
            if let root::Event::Forge(event) = &event
                && let domain::Event::InstallAnswered { request, answer: domain::InstallAnswer::Link(link), .. } =
                    event.as_ref()
            {
                assert_eq!(*request, Token::new(91));
                assert!(link.starts_with(b"http://127.0.0.1:3000/apps/app-fixture/installations/new?state="));
                links += 1;
            } else {
                engine.step(event);
            }
        }
        crate::engine::release(engine);
    }
    assert_eq!(links, 1, "Go on answered with its installation link");
}

#[test]
fn a_removed_github_forge_keeps_no_record() {
    let mut service = restarted();
    let engine = service.engine.as_mut().unwrap();
    engine.step(root::Event::Forge(Box::new(domain::Event::Application {
        request: Token::new(91),
        token: Token::new(92),
        retention: Duration::from_secs(60),
        role: Some(jig_domain_people::Role::Owner),
        carried: None,
        ask: domain::Ask::RemoveForge { forge: FORGE },
    })));
    for _ in 0..PASSES {
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
    }
    assert!(engine.domain.forges().rows.is_empty());
    assert!(super::record(engine, FORGE).is_none());
    assert!(engine.forge_records.rows.forge(FORGE).is_none());
    for item in crate::forge::records::ITEMS {
        let Some(jig_protocol_store::Event::Secret { value: None, .. }) = engine.store.secrets(
            Token::new(123),
            Secrets::Get { owner: SecretOwner::Application, number: records::number(FORGE, item) },
        ) else {
            panic!("App record erased")
        };
    }
    let mut restarted = open(service.engine.take().unwrap().store);
    let engine = restarted.engine.as_mut().unwrap();
    assert!(engine.domain.forges().rows.is_empty());
    assert!(engine.forge.starts.is_empty(), "removed forge is not seeded or connected again");
}

fn bytes(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).unwrap();
    }
    let mut writer = skein_lib::Writer::new(size);
    for part in parts {
        writer.put(part).unwrap();
    }
    writer.finish()
}
