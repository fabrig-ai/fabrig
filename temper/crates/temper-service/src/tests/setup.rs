use super::*;
use crate::tests::fixtures;
use skein_forge_setup::github::manifest::Target;
use skein_lib::{List, Rng};
use temper_engine_domain_forge as domain;
use temper_protocol_records as records;

fn service(states: u32, life: Duration) -> crate::Service {
    let mut service = routing_service(states, life);
    let ready = fixtures::ready().engine.expect("engine");
    service.engine.as_mut().expect("engine").env.limits = ready.env.limits;
    service.engine.as_mut().expect("engine").domain = ready.domain;
    service
}
// The page owners emit their measured sizes only in the usual figure write mode.
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "figure write mode names and writes the measured page sizes"
)]
pub(super) fn measure_page(name: &str, bytes: usize) {
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        let directory = skein_scratch::paths::tree().join("target/q-pages");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(format!("{name}.txt")), format!("{bytes}\n")).unwrap();
    }
}
// Token routing observes queued root inputs, so it needs no second service's restarted root.
pub(super) fn routing_service(states: u32, life: Duration) -> crate::Service {
    routing_service_at(states, life, None)
}
fn routing_service_at(
    states: u32,
    life: Duration,
    public: Option<crate::startup::application::Origin>,
) -> crate::Service {
    let config = github_config(states, life, public);
    let mut service = github_service(config);
    service.engine.as_mut().expect("engine").store = fixtures::records_store();
    load_routing(&mut service);
    service
}
fn github_config(
    states: u32,
    life: Duration,
    public: Option<crate::startup::application::Origin>,
) -> crate::settings::Engine {
    let mut config = fixtures::engine();
    config.clients.public = public;
    config.machine.github = github_record();
    config.limits.setup.states = states;
    config.limits.setup.state_life = life;
    fixtures::fill_files(&mut config);
    config
}
pub(crate) fn load_routing(service: &mut crate::Service) {
    fixtures::mark_store(&mut service.engine.as_mut().expect("engine").store);
    assert!(service.engine.as_ref().expect("engine").setup.component.is_none(), "no host before the rows load");
    for _ in 0_u32..8 {
        crate::forge::records::advance(service.engine.as_mut().expect("engine"));
    }
    advance(service.engine.as_mut().expect("engine"));
    assert!(service.engine.as_ref().expect("engine").setup.component.is_some());
    assert!(service.engine.as_ref().expect("engine").forge_records.done());
    for _ in 0..service.engine.as_mut().expect("engine").notes.len() {
        let _startup_note = service.engine.as_mut().expect("engine").notes.pop();
    }
    // These routing tests fake the listeners, leaving the host and forge as their real components.
    for _ in 0..service.engine.as_mut().expect("engine").workers.io.len() {
        let skein_io::Request::Listen { owner, addr } =
            service.engine.as_mut().expect("engine").workers.io.pop().unwrap()
        else {
            panic!("worker listener")
        };
        let engine = service.engine.as_mut().expect("engine");
        assert!(engine.workers.up(
            Time::ZERO,
            skein_io::Event::Listening { owner, listener: Token::new(78), addr },
            &mut engine.events
        ));
    }
}

pub(crate) fn start(service: &mut crate::Service, run: u64, plan: host::Plan) {
    let run = Token::new(run);
    service.engine.as_mut().expect("engine").setup.staged.insert(run, plan).unwrap();
    crate::route::request(service.engine.as_mut().expect("engine"), root::Request::Setup { run });
}
pub(crate) fn terminal(service: &mut crate::Service) -> root::SetupEnd {
    for _ in 0_u32..4 {
        advance(service.engine.as_mut().expect("engine"));
    }
    for _ in 0..service.engine.as_mut().expect("engine").events.len() {
        if let Some(root::Event::Setup { end, .. }) = service.engine.as_mut().expect("engine").events.pop() {
            return end;
        }
    }
    panic!("setup terminal")
}
fn manifest(service: &mut crate::Service, run: u64) -> Box<[u8]> {
    start(service, run, host::Plan::Manifest { forge: 1, target: Target::Personal });
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Manifest { state }, .. } = terminal(service) else {
        panic!("manifest")
    };
    state
}
fn row(service: &mut crate::Service, item: Item, bytes: &[u8]) {
    service.engine.as_mut().expect("engine").forge_records.rows.push(
        records::number(1, item),
        Secret::new(match records::encode(item, bytes) {
            Ok(value) => value.into_boxed_slice(),
            Err(records::Unreadable) => Box::from([0]),
        }),
        4096,
    );
}
#[test]
fn the_host_starts_after_loading_with_temper_urls_and_the_configured_limits() {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    let page = service.setup_page(1, &state, Time::ZERO).unwrap();
    for expected in [
        b"temper-".as_slice(),
        b"/forges/1/github/redirect",
        b"/forges/1/github/setup",
        b"http://127.0.0.1:3000/settings/apps/new",
    ] {
        assert!(contains(page, expected));
    }
    assert_eq!(service.engine.as_ref().expect("engine").setup.env.limits.runs, 8);
    assert_eq!(service.engine.as_ref().expect("engine").setup.env.limits.states, 8);
    assert!(service.engine.as_ref().expect("engine").setup.io.is_empty(), "manifest is local");
}
#[test]
#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
fn reads_use_the_retained_row_past_its_version_and_absence_answers_none() {
    for present in [false, true] {
        let mut service = service(8, Duration::from_secs(3600));
        if present {
            row(&mut service, Item::AppSlug, b"app-fixture");
        }
        start(&mut service, 1, host::Plan::Install { forge: 1 });
        let end = terminal(&mut service);
        let (_, host::Answer::Read(value)) = service.engine.as_mut().expect("engine").setup.answers.pop().unwrap()
        else {
            panic!("read answer")
        };
        assert_eq!(
            match &value {
                Some(value) => Some(value.expose().len()),
                None => None,
            },
            if present { Some(11) } else { None }
        );
        match end {
            root::SetupEnd::Done { outcome: root::SetupOutcome::Install { url, .. }, .. } => {
                assert!(url.starts_with(b"http://127.0.0.1:3000/apps/app-fixture/"));
            }
            root::SetupEnd::Stopped { reason, .. } => assert!(reason.ends_with(b"Missing")),
            root::SetupEnd::Done { .. } => panic!("install terminal"),
        }
        assert!(service.engine.as_ref().expect("engine").setup.io.is_empty());
    }
}
#[test]
fn pages_expire_on_their_done_clock_and_reload_does_not_consume_them() {
    let mut service = service(2, Duration::from_secs(3));
    let state = manifest(&mut service, 1);
    for seconds in [0, 2, 2] {
        assert!(service.setup_page(1, &state, Time::ZERO.saturating_add(Duration::from_secs(seconds))).is_some());
    }
    assert!(service.setup_page(2, &state, Time::ZERO).is_none());
    assert!(service.setup_page(1, &state, Time::ZERO.saturating_add(Duration::from_secs(3))).is_none());
    service.engine.as_mut().expect("engine").env.now = Time::ZERO.saturating_add(Duration::from_secs(3));
    advance(service.engine.as_mut().expect("engine"));
    assert!(service.engine.as_ref().expect("engine").setup.pages.is_empty());
    assert_eq!(service.engine.as_ref().expect("engine").setup.next_deadline(), None);
}
#[test]
fn the_oldest_page_is_dropped_at_capacity() {
    let mut service = service(1, Duration::from_secs(3600));
    let first = manifest(&mut service, 1);
    let second = manifest(&mut service, 2);
    assert!(service.setup_page(1, &first, Time::ZERO).is_none());
    assert!(service.setup_page(1, &second, Time::ZERO).is_some());
}
#[test]
fn an_install_can_evict_a_manifests_state_while_its_page_lives() {
    let mut service = service(1, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    row(&mut service, Item::AppSlug, b"app-fixture");
    start(&mut service, 2, host::Plan::Install { forge: 1 });
    assert!(let_done(terminal(&mut service)));
    assert!(service.setup_page(1, &state, Time::ZERO).is_some());
    // The slug above only made an Install-purpose state; no App remains in this host-state fixture.
    service.engine.as_mut().expect("engine").forge_records.rows =
        crate::forge::records::Loader::new(&fixtures::engine().limits).rows;
    start(&mut service, 3, host::Plan::Convert { forge: 1, code: code(), state });
    let root::SetupEnd::Stopped { forge, reason } = terminal(&mut service) else { panic!("evicted state refused") };
    assert_eq!(forge, 1);
    assert!(reason.ends_with(b"Unknown"));
    assert!(service.engine.as_ref().expect("engine").setup.io.is_empty(), "evicted state sends no conversion");
}
fn let_done(end: root::SetupEnd) -> bool {
    match end {
        root::SetupEnd::Done { .. } => true,
        root::SetupEnd::Stopped { .. } => false,
    }
}
fn code() -> Secret {
    Secret::new(Box::from(skein_lib::Decimal::of(Rng::new(805).next_u64()).as_bytes()))
}

pub(crate) struct Face {
    owner: Option<Token>,
    hold_done: bool,
    done: Option<host::Event>,
    pub(crate) wire: skein_world::stream::Wire,
}
impl Face {
    pub(crate) fn new() -> Self {
        Self { owner: None, hold_done: false, done: None, wire: skein_world::stream::Wire::new(7) }
    }
    pub(crate) fn turn(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..service.engine.as_mut().expect("engine").setup.io.len() {
            match service.engine.as_mut().expect("engine").setup.io.pop().unwrap() {
                skein_io::Request::Connect { owner, .. } => {
                    self.owner = Some(owner);
                    self.wire = skein_world::stream::Wire::new(7);
                    service
                        .engine
                        .as_mut()
                        .expect("engine")
                        .setup
                        .up(skein_io::Event::Connecting { owner, socket: Token::new(123) });
                    service.engine.as_mut().expect("engine").setup.up(skein_io::Event::Connected { owner });
                }
                skein_io::Request::Stream { down, .. } => self.wire.take(down),
                skein_io::Request::Close { .. } | skein_io::Request::Abort { .. } => {
                    service
                        .engine
                        .as_mut()
                        .expect("engine")
                        .setup
                        .up(skein_io::Event::Closed { owner: self.owner.take().unwrap() });
                }
                skein_io::Request::Listen { .. }
                | skein_io::Request::Bind { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. } => panic!("setup socket request"),
            }
        }
        if let Some(owner) = self.owner
            && let Some(up) = self.wire.answer()
        {
            service.engine.as_mut().expect("engine").setup.up(skein_io::Event::Stream { owner, up });
        }
        if self.hold_done {
            let engine = service.engine.as_mut().unwrap();
            for _ in 0..engine.setup.above.len() {
                let event = engine.setup.above.pop().unwrap();
                match event {
                    done @ host::Event::Done { .. } => self.done = Some(done),
                    other @ (host::Event::Need { .. } | host::Event::Stop { .. } | host::Event::Closed) => {
                        engine.setup.above.push(other);
                    }
                }
            }
        }
        advance(service.engine.as_mut().expect("engine"));
    }
    pub(crate) fn reply(&mut self, service: &mut crate::Service, body: &[u8]) {
        self.reply_status(service, b"200 OK", body);
    }
    fn reply_status(&mut self, service: &mut crate::Service, status: &[u8], body: &[u8]) {
        let length = skein_lib::Decimal::of(u64::try_from(body.len()).unwrap());
        let head = b"HTTP/1.1 ";
        let content = b"\r\nContent-Length: ";
        let mut writer = skein_lib::Writer::new(
            head.len()
                .checked_add(status.len())
                .unwrap()
                .checked_add(content.len())
                .unwrap()
                .checked_add(length.as_bytes().len())
                .unwrap()
                .checked_add(4)
                .unwrap()
                .checked_add(body.len())
                .unwrap(),
        );
        writer.put(head).unwrap();
        writer.put(status).unwrap();
        writer.put(content).unwrap();
        writer.put(length.as_bytes()).unwrap();
        writer.put(b"\r\n\r\n").unwrap();
        writer.put(body).unwrap();
        self.wire.write(&writer.finish());
        for _ in 0_u32..1024 {
            self.turn(service);
        }
    }
    pub(crate) fn sent(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..1024 {
            self.turn(service);
        }
        assert!(contains(&self.wire.received, b"\r\n\r\n"), "request sent");
    }
}
fn writing() -> (crate::Service, Face) {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    start(&mut service, 2, host::Plan::Convert { forge: 1, code: code(), state });
    let face = conversion_replies(&mut service);
    (service, face)
}
fn conversion_replies(service: &mut crate::Service) -> Face {
    let mut face = Face::new();
    face.sent(service);
    let der = skein_fake_forge_github::keys::KEYS[0];
    let encoded =
        skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, u32::try_from(der.len()).unwrap())
            .unwrap();
    let mut base64 = skein_lib::Writer::new(usize::try_from(encoded).unwrap());
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, der, &mut base64).unwrap();
    let prefix = br#"{"id":7,"slug":"app-fixture","owner":{"id":1,"login":"owner-fixture"},"pem":"-----BEGIN RSA PRIVATE KEY-----\n"#;
    let suffix = br#"\n-----END RSA PRIVATE KEY-----\n","client_id":"fixture","client_secret":"unused","webhook_secret":"unused"}"#;
    let mut body = skein_lib::Writer::new(
        prefix.len().checked_add(usize::try_from(encoded).unwrap()).unwrap().checked_add(suffix.len()).unwrap(),
    );
    body.put(prefix).unwrap();
    body.put(&base64.finish()).unwrap();
    body.put(suffix).unwrap();
    face.reply(service, &body.finish());
    face.sent(service);
    face.reply(service, br#"{"id":7,"slug":"app-fixture"}"#);
    face.sent(service);
    face.reply(service, br#"{"id":9,"login":"app-fixture[bot]"}"#);
    if service.engine.as_mut().expect("engine").setup.writes.is_empty() {
        if let Some(root::Event::Setup { end, .. }) = service.engine.as_mut().expect("engine").events.pop() {
            panic!("conversion terminal: {end:?}");
        }
        panic!(
            "conversion pending: owner {}, incoming {}, outgoing {}",
            face.owner.is_some(),
            service.engine.as_mut().expect("engine").setup.above.len(),
            service.engine.as_mut().expect("engine").setup.io.len()
        );
    }
    face
}
#[test]
fn written_is_answered_only_after_kept_and_the_routing_rows_follow_each_keep() {
    let (mut service, mut face) = writing();
    for expected in [Item::AppId, Item::AppSlug, Item::AppKey, Item::User] {
        assert!(service.engine.as_ref().expect("engine").setup.answers.is_empty(), "Written must wait for Kept");
        assert!(
            (match service.engine.as_ref().expect("engine").forge_records.rows.forge(1) {
                Some(rows) => rows.get(expected).ok(),
                None => None,
            })
            .is_none(),
            "unkept row is invisible"
        );
        let owner = *service.engine.as_mut().expect("engine").setup.writes.first().unwrap().0;
        let write = service.engine.as_mut().expect("engine").setup.writes.get(&owner).unwrap();
        assert_eq!(write.number, records::number(1, expected));
        assert_eq!(write.value.expose().get(..2), Some(records::VERSION.to_be_bytes().as_slice()));
        answer(service.engine.as_mut().expect("engine"), StoreEvent::Kept { owner });
        assert_eq!(
            service.engine.as_mut().expect("engine").setup.answers.pop(),
            Some((Token::new(2), host::Answer::Written))
        );
        service.engine.as_mut().expect("engine").forge_records.rows.forge(1).unwrap().get(expected).unwrap();
        face.turn(&mut service);
    }
    advance(service.engine.as_mut().expect("engine"));
    let owner = *service.engine.as_mut().expect("engine").setup.writes.first().unwrap().0;
    assert_eq!(
        service.engine.as_ref().expect("engine").setup.writes.get(&owner).unwrap().number,
        records::number(1, Item::Origin)
    );
    answer(service.engine.as_mut().expect("engine"), StoreEvent::Kept { owner });
    assert!(let_done(terminal(&mut service)));
}
#[test]
fn a_failed_keep_stops_the_engine_and_never_answers_written() {
    let (mut service, _face) = writing();
    let owner = *service.engine.as_mut().expect("engine").setup.writes.first().unwrap().0;
    let Some(StoreEvent::Kept { owner: kept }) = service.engine.as_mut().expect("engine").store.above.pop() else {
        panic!("pending keep terminal")
    };
    assert_eq!(kept, owner);
    service.engine.as_mut().expect("engine").store.above.push(StoreEvent::Corrupt { owner });
    crate::engine::iterate(service.engine.as_mut().expect("engine"), Time::ZERO, Wall::EPOCH);
    assert!(service.end().is_some());
    assert!(service.engine.as_ref().expect("engine").setup.answers.is_empty());
}
#[test]
fn draining_closes_the_host_and_waits_for_closed() {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    start(&mut service, 2, host::Plan::Convert { forge: 1, code: code(), state });
    let mut face = Face::new();
    face.sent(&mut service);
    crate::stop::shutdown(service.engine.as_mut().expect("engine"));
    crate::stop::drained(service.engine.as_mut().expect("engine"));
    advance(service.engine.as_mut().expect("engine"));
    assert!(!service.engine.as_ref().expect("engine").setup.closed, "Closed waits for the live socket");
    face.turn(&mut service);
    assert!(service.engine.as_ref().expect("engine").setup.closed);
    assert!(service.engine.as_ref().expect("engine").setup.pages.is_empty());
    assert_eq!(service.engine.as_ref().expect("engine").setup.next_deadline(), None);
}
fn tls_forges() -> (crate::settings::Engine, List<crate::ForgeRuntime>) {
    // Skein's fixed TLS-world root, carried here without another test dependency.
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
    let mut config = github_config(8, Duration::from_secs(3600), None);
    config.machine.github.api.transport =
        domain::ApiTransport::Tls { server_name: b"first.test".as_slice().into(), trust: None };
    let mut runtime = List::with_capacity(2);
    for (number, name, address) in [(1, "first.test", [192, 0, 2, 7]), (2, "second.test", [192, 0, 2, 8])] {
        let (trust, _) = skein_tls::Config::from_der(core::slice::from_ref(&root), &[name.as_bytes()]).unwrap();
        runtime
            .push(crate::ForgeRuntime {
                number,
                address: (address, 443).into(),
                transport: skein_forge_connection::settings::Transport::Tls {
                    server_name: skein_tls::Name::new(name).unwrap(),
                    trust,
                },
            })
            .unwrap();
    }
    (config, runtime)
}
#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "compare each prepared public trust configuration without exposing credentials"
)]
fn each_github_tls_entry_keeps_its_prepared_name_trust_and_address() {
    let (config, runtime) = tls_forges();
    let service = resolved_githubs(config, runtime);
    let engine = service.engine.as_ref().unwrap();
    let (settings, _) = engine.setup.pending.as_ref().unwrap();
    assert_eq!(engine.setup.served.len(), 2);
    assert_eq!(settings.forges.len(), 2);
    assert_eq!(settings.user_agent.as_ref(), crate::forge::USER_AGENT.as_bytes());
    for number in [1, 2] {
        let endpoint = engine.forge.endpoint(number).unwrap();
        let mut entry = None;
        for candidate in &settings.forges {
            if candidate.number() == endpoint.number {
                entry = Some(candidate);
                break;
            }
        }
        let entry = entry.unwrap();
        assert_eq!(entry.address, skein_io::Addr::Ip(endpoint.address));
        let host::Transport::Tls { server_name, trust } = &entry.transport else { panic!("TLS entry") };
        let skein_forge_connection::settings::Transport::Tls { server_name: name, trust: prepared } =
            &endpoint.transport
        else {
            panic!("prepared TLS")
        };
        assert_eq!(server_name, name);
        assert_eq!(alloc::format!("{trust:?}"), alloc::format!("{prepared:?}"));
    }
}
#[test]
fn admitted_tls_starts_after_rows_and_serves_a_local_manifest() {
    let (config, runtime) = configured(8, 8, true);
    let mut service = resolved_githubs(config, runtime);
    assert!(service.engine.as_ref().expect("engine").setup.component.is_none(), "no host before the rows load");
    let (settings, _) = service.engine.as_ref().expect("engine").setup.pending.as_ref().unwrap();
    assert_eq!(settings.forges.len(), 2);
    for entry in &settings.forges {
        let host::Transport::Tls { .. } = &entry.transport else { panic!("admitted TLS") };
        assert!(service.engine.as_ref().expect("engine").setup.served.contains_key(&entry.number()));
    }
    service.engine.as_mut().expect("engine").store = fixtures::records_store();
    for _ in 0_u32..8 {
        crate::forge::records::advance(service.engine.as_mut().expect("engine"));
    }
    advance(service.engine.as_mut().expect("engine"));
    assert!(service.engine.as_ref().expect("engine").forge_records.done());
    assert!(service.engine.as_ref().expect("engine").setup.component.is_some());
    for _ in 0..service.engine.as_mut().expect("engine").notes.len() {
        let _startup_note = service.engine.as_mut().expect("engine").notes.pop();
    }
    let state = manifest(&mut service, 1);
    assert!(service.setup_page(1, &state, Time::ZERO).is_some());
    assert!(service.end().is_none());
    assert!(service.engine.as_ref().expect("engine").setup.io.is_empty(), "manifest is local");
}
#[test]
fn the_host_names_the_metadata_limits_and_rejects_bad_start_limits() {
    let mut service = service(8, Duration::from_secs(3600));
    let setup = &mut service.engine.as_mut().expect("engine").setup;
    assert_eq!(setup.env.limits.inner, 32);
    let mut found = jig_protocol_config::Problems::empty();
    settings::problems(host::Unusable::GitHubMetadata { inner: 1, string_bytes: 1 }, &mut found);
    for (index, name) in [b"forge_component.inner".as_slice(), b"forge_component.string_bytes"].into_iter().enumerate()
    {
        assert_eq!(found.problems.get(u32::try_from(index).unwrap()).unwrap().path, settings::limit_path(name));
    }
}

pub(crate) fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    for part in bytes.windows(needle.len()) {
        if part == needle {
            return true;
        }
    }
    false
}
#[test]
fn every_host_unusable_arm_has_an_operator_path() {
    use host::Stacked::{HttpOverTls, TlsOverIo};
    use host::{Stacked, Unusable};
    for why in [
        Unusable::Zero,
        Unusable::TokenBytes { token_bytes: 1 },
        Unusable::Documents(skein_forge::forgejo::limits::Problem::Depth),
        Unusable::GitHubDocuments(skein_forge::github::limits::Problem::Depth),
        Unusable::GitHubMetadata { inner: 1, string_bytes: 1 },
        Unusable::Stack { stacked: Stacked::HttpOverIo, demand: 2, cap: 1 },
        Unusable::Stack { stacked: HttpOverTls, demand: 2, cap: 1 },
        Unusable::Stack { stacked: TlsOverIo, demand: 2, cap: 1 },
        Unusable::Stack { stacked: Stacked::JsonOverHttp, demand: 2, cap: 1 },
        Unusable::Forges { forges: 1 },
        Unusable::Duplicate { forge: 1 },
        Unusable::Address { forge: 1 },
        Unusable::PlaintextName { forge: 1 },
        Unusable::Request { forge: 1 },
        Unusable::Given {
            forge: 1,
            refused: skein_forge_setup::given::Refused {
                place: skein_forge_setup::given::Place::Deadline,
                other: None,
            },
        },
        Unusable::GitHubGiven {
            forge: 1,
            refused: skein_forge_setup::github::given::Refused {
                place: skein_forge_setup::github::given::Place::Backoff,
                other: Some(skein_forge_setup::github::given::Place::BackoffMax),
            },
        },
        Unusable::Reachable { forge: 1, reachable: 2, inner: 1 },
    ] {
        let mut found = jig_protocol_config::Problems::empty();
        settings::problems(why, &mut found);
        assert!(!found.is_empty());
        let Unusable::Stack { stacked, .. } = why else { continue };
        let expected: &[&[u8]] = match stacked {
            HttpOverTls => &[b"forge_component.chunk", b"forge_component.tls.read", b"forge_component.tls.send"],
            TlsOverIo => &[b"forge_component.tls.read", b"forge_component.tls.send", b"io.intake", b"io.output"],
            Stacked::HttpOverIo | Stacked::JsonOverHttp => continue,
        };
        assert_eq!(found.problems.len(), u32::try_from(expected.len()).unwrap());
        for (problem, name) in found.problems.iter().zip(expected) {
            assert_eq!(problem.path, settings::limit_path(name));
        }
    }
}
#[test]
fn invalid_host_limits_stop_after_load_and_setup_overrides_are_checked() {
    let mut service = service(8, Duration::from_secs(3600));
    let mut config = github_config(8, Duration::from_secs(3600), None);
    config.limits.forge_component.inner = 13;
    config.limits.picker.page = 1;
    service.engine.as_mut().expect("engine").setup = prepare(&config, &fixtures::seeds().crypto, false).unwrap();
    let record = config.machine.github.clone();
    register(service.engine.as_mut().unwrap(), &record);
    advance(service.engine.as_mut().expect("engine"));
    assert!(service.end().is_none());
    assert!(service.engine.as_ref().unwrap().setup.unserved.contains_key(&1));
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    for name in [b"setup.states".as_slice(), b"setup.runs", b"setup.state_life_ms", b"setup.orgs"] {
        assert_eq!(crate::limits::apply(&mut own, name, 0), Err(crate::limits::Refused::Range));
    }
    assert_eq!(crate::limits::apply(&mut own, b"setup.runs", 3), Ok(()));
    assert_eq!(own.setup.runs, 3);
}

#[test]
fn stopped_connection_text_has_a_fixed_bound() {
    use skein_forge_connection::{StartRead, Stopped, Value};
    let mut reasons = List::with_capacity(13);
    for value in [Value::Credential, Value::Token, Value::Key, Value::User, Value::Secret] {
        reasons.push(Stopped::Invalid { value }).unwrap();
    }
    for read in [StartRead::User, StartRead::ApiSettings, StartRead::Installations, StartRead::Mint, StartRead::Viewer]
    {
        reasons.push(Stopped::Refused { read, status: u16::MAX }).unwrap();
    }
    reasons.push(Stopped::TokenRefused { installation: u64::MAX, status: u16::MAX }).unwrap();
    reasons.push(Stopped::OtherUser { user: u64::MAX }).unwrap();
    reasons.push(Stopped::KeyRefused).unwrap();
    for why in &reasons {
        assert!(
            u64::try_from(lifecycle::stopped_text(*why).len()).unwrap()
                <= temper_engine_domain_forge::STOPPED_TEXT_BYTES
        );
    }
}
fn configured(runs: u32, states: u32, tls: bool) -> (crate::settings::Engine, List<crate::ForgeRuntime>) {
    let (mut config, runtime) = if tls {
        tls_forges()
    } else {
        (github_config(states, Duration::from_secs(3600), None), fixtures::forge_runtime())
    };
    config.limits.setup.runs = runs;
    config.limits.setup.states = states;
    fixtures::fill_files(&mut config);
    (config, runtime)
}

#[path = "setup_reachable.rs"]
mod reachable_tests;

#[path = "setup_lifecycle.rs"]
mod lifecycle_tests;

#[path = "setup_adoption.rs"]
mod adoption_tests;

#[path = "git_tokens.rs"]
mod git_tokens;

pub(crate) fn local_config(records: bool, fake: bool) -> crate::settings::Engine {
    let mut config = fixtures::engine();
    assert!(records, "file-era starts are refused");
    let mut record = fixtures::forge_record(1);
    record.mode = if fake { domain::Mode::Fake } else { domain::Mode::Owned };
    record.api.port = 9;
    record.bot.login = b"temper".as_slice().into();
    record.setup = Some(if fake {
        br#"{"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{"listen":{"socket":"127.0.0.1:9"},"git":"/data/git","owner":"demo","repositories":[{"name":"widget","landing":"main"}]}}}"#.as_slice().into()
    } else {
        fixtures::owned_body().into()
    });
    config.machine.owned = Some(record);
    config.limits.forge.client.repositories = 3;
    config.limits.forge.output = 96;
    fixtures::fill_files(&mut config);
    config
}
pub(crate) fn kept_record(store: &mut crate::store::Store, forge: u16, item: Item, bytes: &[u8]) {
    let owner = Token::new(901);
    assert!(
        store
            .secrets(
                owner,
                fixtures::application_request(
                    records::number(forge, item),
                    Secret::new(match records::encode(item, bytes) {
                        Ok(value) => value.into_boxed_slice(),
                        Err(records::Unreadable) => Box::from([0]),
                    })
                )
            )
            .is_none()
    );
    let answer = store.above.pop();
    let Some(StoreEvent::Kept { owner: found }) = answer else {
        panic!("durable seed: {answer:?}, marked: {}", store.component.mark_durable());
    };
    assert_eq!(found, owner);
}
pub(crate) fn local_store(token: bool) -> crate::store::Store {
    use temper_engine_domain_forge::Stored;
    // Persist the real first-start settings before constructing a restart fixture.
    let mut bootstrap = fixtures::ready();
    let mut header = bootstrap.engine.as_ref().unwrap().domain.deployment();
    header.commits = header.commits.checked_add(1).unwrap();
    let mut store = bootstrap.engine.take().unwrap().store;
    if token {
        kept_record(&mut store, 1, Item::AdminToken, b"test-admin");
    }
    for (item, value) in
        [(Item::ApiToken, b"test-api".as_slice()), (Item::GitToken, b"test-git"), (Item::User, &42_u64.to_le_bytes())]
    {
        kept_record(&mut store, 1, item, value);
    }
    let row = local_repository();
    assert!(store.down(root::StoreRequest::Commit {
        number: header.commits,
        writes: Box::new([
            root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(header),
            ))),
            root::Write::Save(root::Record::Forge { row: Box::new(Stored::Repository(row)) }),
        ]),
    }));
    let Some(StoreEvent::Committed { number, .. }) = store.above.pop() else {
        panic!("durable repository");
    };
    assert_eq!(number, header.commits);
    store
}
pub(crate) fn local_repository() -> domain::Repository {
    use temper_engine_domain_forge::{Kinds, Protection, Repository, Role};
    Repository {
        project: 1,
        home: true,
        provider: skein_forge::api::Repository { forge: 1, repository: 41 },
        host: b"forge.example".as_slice().into(),
        owner: b"demo".as_slice().into(),
        name: b"widget".as_slice().into(),
        prefix: b"temper/".as_slice().into(),
        role: Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: b"main".as_slice().into(),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}
#[expect(clippy::manual_map, reason = "the test fixture uses explicit matches")]
pub(crate) fn local_service(config: crate::settings::Engine, store: crate::store::Store) -> crate::Service {
    let mut record = match &config.machine.owned {
        Some(record) => record.clone(),
        None => config.machine.github.clone(),
    };
    record.number = 1;
    let handover = match config.files.contents(b"/fixture/handover") {
        Some(bytes) => Some(Secret::new(bytes.into())),
        None => None,
    };
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None).unwrap();
    fixtures::register_forges(&mut service, &[record]);
    if let Some(handover) = handover {
        service.engine.as_mut().unwrap().setup.lifecycle.local_mut(1).unwrap().handover = Some(handover);
    }
    service.engine.as_mut().unwrap().store = store;
    service
}
pub(crate) fn restore_local(service: &mut crate::Service) {
    // The generic kernel refuses unhandled network connects. Hold local work
    // until service admission opens, then give its real setup streams to Face.
    // domain/root.md, 9.1 holds effects through the durable mark; the service pass
    // publishes the held sign-in link after observing decisions open.
    let pending = match service.engine.as_mut().unwrap().setup.lifecycle.local_mut(1) {
        Some(local) => {
            let pending = local.pending;
            local.pending = false;
            Some(pending)
        }
        None => None,
    };
    for _ in 0_u32..4096 {
        fixtures::turn(service);
        let engine = service.engine.as_mut().unwrap();
        if engine.domain.ready() && engine.clients.starting.is_none() {
            if let Some(pending) = pending {
                engine.setup.lifecycle.local_mut(1).unwrap().pending = pending;
            }
            crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
            advance(engine);
            return;
        }
    }
    let engine = service.engine.as_ref().unwrap();
    panic!(
        "root restore completes: end={:?}, step={:?}, settings={:?}, header={:?}",
        service.end(),
        engine.domain.restart_failure(),
        engine.domain.restore_failure(),
        engine.domain.deployment(),
    );
}
pub(crate) fn local_sent(service: &mut crate::Service, face: &mut Face, path: &[u8]) {
    face.sent(service);
    assert!(contains(&face.wire.received, path), "wanted path {path:?}");
}
fn finish_fake_adoption(service: &mut crate::Service, face: &mut Face) {
    if !service.engine.as_ref().unwrap().setup.lifecycle.adoptions.contains_key(&1) {
        return;
    }
    face.hold_done = false;
    local_sent(service, face, b"GET /api/v1/user ");
    face.reply(service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
    adoption_tests::record_repository(service, face, 1, adoption_tests::WRITE_PERMISSION);
}
fn reconcile_to_repository(service: &mut crate::Service, face: &mut Face) {
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
    local_sent(service, face, b"GET /api/v1/repositories/41 ");
    face.reply(service, br#"{"id":41,"owner":{"id":1,"login":"demo"},"name":"widget","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":true,"allow_rebase":true,"html_url":"http://127.0.0.1:9/demo/widget"}"#);
    local_sent(service, face, b"/collaborators/temper/permission ");
}
#[test]
fn kept_local_records_wait_for_restore_done_and_origin_before_connecting_as_the_recorded_bot() {
    for fake in [false] {
        let mut service = local_service(local_config(true, fake), local_store(true));
        let engine = service.engine.as_mut().unwrap();
        for _ in 0_u32..8 {
            crate::forge::records::advance(engine);
        }
        advance(engine);
        assert!(engine.setup.runs.is_empty(), "no Reconcile before root restore");
        assert!(engine.forge.starts.is_empty());
        assert!(engine.domain.adopted_ids(1).is_empty());
        restore_local(&mut service);
        assert_eq!(service.engine.as_ref().unwrap().domain.adopted_ids(1).as_ref(), &[41]);
        let engine = service.engine.as_mut().unwrap();
        let before = (engine.setup.io.len(), engine.setup.runs.len(), engine.setup.writes.len());
        crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
        assert_eq!(
            (engine.setup.io.len(), engine.setup.runs.len(), engine.setup.writes.len()),
            before,
            "a second restore iterate changes no local setup work"
        );
        // A completed walk is never repeated, including for an already stopped local start.
        engine.setup.lifecycle.local_mut(1).unwrap().restored = false;
        lifecycle::restored(engine);
        assert!(!engine.setup.lifecycle.local(1).unwrap().restored);
        engine.setup.lifecycle.local_mut(1).unwrap().restored = true;
        let mut face = Face::new();
        reconcile_to_repository(&mut service, &mut face);
        assert!(service.engine.as_ref().unwrap().forge.starts.is_empty(), "held Reconcile has no access");
        face.hold_done = true;
        face.reply(&mut service, br#"{"permission":"write","user":{"id":42,"login":"temper"}}"#);
        assert!(face.done.is_some(), "the actual host completed Reconcile");
        assert!(service.engine.as_ref().unwrap().setup.writes.is_empty(), "no Origin before Done");
        assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
        service.engine.as_mut().unwrap().setup.above.push(face.done.take().unwrap());
        advance(service.engine.as_mut().unwrap());
        let engine = service.engine.as_mut().unwrap();
        assert_eq!(engine.setup.writes.len(), 1);
        let write = engine.setup.writes.first().unwrap().1;
        assert_eq!(write.number, records::number(1, Item::Origin), "never writes AdminToken");
        assert!(engine.forge.starts.is_empty(), "Origin Kept is held");
        assert!(engine.values.identities.newest(1).is_none());
        let Some(StoreEvent::Kept { owner }) = engine.store.above.pop() else {
            panic!("durable Origin");
        };
        answer(engine, StoreEvent::Kept { owner });
        if fake {
            finish_fake_adoption(&mut service, &mut face);
        }
        let engine = service.engine.as_mut().unwrap();
        assert_eq!(engine.forge.starts.get(0).unwrap().1.user, 42);
        assert!(engine.forge.accepted.is_empty(), "Forgejo has no installation set");
        let (generation, value) = engine.values.identities.newest(1).unwrap();
        assert_eq!(generation, 1);
        let value = fixtures::decoded_identity(value);
        assert_eq!(value.login(), b"temper");
        assert_eq!(
            match value.kind() {
                temper_protocol_worker_wire::v1::Kind::Forgejo(token) => token.token(),
                temper_protocol_worker_wire::v1::Kind::Github(_) => panic!("Forgejo identity"),
            },
            &Secret::new(b"test-git".as_slice().into())
        );
        let mut credentials = 0_u32;
        for event in &engine.events {
            if let root::Event::Credential { number, generation, .. } = event {
                assert_eq!((*number, *generation), (1, 1));
                credentials = credentials.checked_add(1).unwrap();
            }
        }
        assert_eq!(credentials, 1);
        lifecycle::connected(engine, 1);
        let mut connected = false;
        for event in &engine.events {
            if let root::Event::Forge(value) = event
                && let temper_engine_domain_forge::Event::Connection {
                    forge: 1,
                    state: temper_engine_domain_forge::Connection::Connected { id, login },
                } = value.as_ref()
            {
                assert_eq!(*id, 42);
                assert_eq!(login.as_ref(), b"temper");
                connected = true;
            }
        }
        assert!(connected);
        assert!(engine.notes.is_empty(), "no credentials in Notes");
        no_setup_events(engine);
        advance(engine);
        assert!(engine.setup.runs.is_empty(), "Forgejo does not reconcile from Connected");
        // A second start uses exactly the durable store, including its AdminToken and Origin.
        let mut store = core::mem::replace(&mut engine.store, fixtures::records_store());
        for _ in 0..store.above.len() {
            let _terminal = store.above.pop();
        }
        let mut restarted = local_service(local_config(true, fake), store);
        restore_local(&mut restarted);
        let mut face = Face::new();
        local_sent(&mut restarted, &mut face, b"GET /api/v1/user ");
        assert!(contains(&face.wire.received, b"test-admin"));
    }
}

#[test]
fn local_given_uses_the_repository_capacity_no_orgs_and_configured_listener() {
    let mut config = local_config(true, false);
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    let service = local_service(config, local_store(true));
    let engine = service.engine.as_ref().unwrap();
    let host::Kind::Forgejo(given) = registry_support::given(&service, 1).unwrap() else { panic!("Forgejo given") };
    assert_eq!(given.limits.repositories, engine.setup.statics.limits.forge.client.repositories);
    assert_eq!(given.limits.repositories, 3);
    assert!(given.orgs.is_empty());
    assert_eq!(given.org_owner.as_deref(), Some(b"owner".as_slice()));
    assert_eq!(given.listener, Some(([127, 0, 0, 1], 7101).into()));
    assert_eq!(engine.setup.served.len(), 1);
}
fn local_with_other(config: crate::settings::Engine, store: crate::store::Store) -> crate::Service {
    let mut service = local_service(config, store);
    let mut other = github_record();
    other.number = 2;
    fixtures::register_forges(&mut service, &[other]);

    service
}
fn other_progresses(service: &mut crate::Service) {
    while service.notes().pop().is_some() {}
    start(service, 77, host::Plan::Manifest { forge: 2, target: Target::Personal });
    let root::SetupEnd::Done { forge: 2, .. } = terminal(service) else {
        panic!("other forge progresses");
    };
    for (_, write) in &service.engine.as_ref().unwrap().setup.writes {
        assert_ne!(write.number, records::number(1, Item::AdminToken));
    }
}
#[test]
fn no_kept_admin_token_names_preparation_and_leaves_the_other_forge_served() {
    let mut service = local_with_other(local_config(true, false), local_store(false));
    restore_local(&mut service);
    let mut prepared = false;
    for note in service.notes().iter() {
        if let crate::Note::ForgeSetupStopped { number: 1, why: crate::SetupStopped::NotPrepared } = note {
            prepared = true;
        }
    }
    assert!(prepared);
    assert!(service.engine.as_ref().unwrap().setup.io.is_empty());
    assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
    other_progresses(&mut service);
}
#[test]
fn local_stops_missing_busy_and_origin_mismatch_leave_the_other_forge_served() {
    for fault in [0_u8, 1, 2, 3, 4, 5] {
        let mut store = local_store(true);
        if fault == 3 || fault == 4 {
            let origin = records::origin(
                if fault == 3 {
                    skein_forge_connection::settings::Kind::GitHub
                } else {
                    skein_forge_connection::settings::Kind::Forgejo
                },
                &jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 10).into())),
            )
            .encode();
            kept_record(&mut store, 1, Item::Origin, &origin);
        }
        if fault == 5 {
            kept_record(&mut store, 1, Item::AdminToken, b"");
        }
        let mut service = local_with_other(local_config(true, false), store);
        // Hold root restore while setting the host's actual admission conditions.
        for _ in 0_u32..8 {
            crate::forge::records::advance(service.engine.as_mut().unwrap());
        }
        advance(service.engine.as_mut().unwrap());
        if fault == 1 {
            service.engine.as_mut().unwrap().setup.served.remove(&1);
        }
        if fault == 2 {
            service
                .engine
                .as_mut()
                .unwrap()
                .setup
                .runs
                .insert(
                    Token::new(76),
                    Run {
                        first: None,
                        forge: 1,
                        kind: Kind::Adopt,
                        internal: true,
                        previous: temper_engine_domain_forge::Connection::NotConnected,
                    },
                )
                .unwrap();
        }
        restore_local(&mut service);
        if fault == 0 {
            let mut face = Face::new();
            local_sent(&mut service, &mut face, b"GET /api/v1/user ");
            face.wire.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
            for _ in 0_u32..1024 {
                face.turn(&mut service);
            }
        }
        let expected = match fault {
            0 => Some(crate::SetupStopped::Stopped {
                plan: crate::SetupPlan::Reconcile,
                step: crate::SetupStep { stage: reconcile::Stage::Admin, why: reconcile::Why::Refused(403) },
            }),
            1 => Some(crate::SetupStopped::Missing),
            2 => Some(crate::SetupStopped::Busy),
            _ => None,
        };
        let engine = service.engine.as_ref().unwrap();
        let mut named = false;
        for note in &engine.notes {
            named |= match (note, expected.as_ref()) {
                (crate::Note::ForgeSetupStopped { number: 1, why }, Some(expected)) => why == expected,
                (crate::Note::ForgeNotConnected { number: 1, why: records::NotConnected::OtherForge { .. } }, None) => {
                    fault != 5
                }
                (crate::Note::ForgeNotConnected { number: 1, why: records::NotConnected::Unreadable }, None) => {
                    fault == 5
                }
                _ => false,
            };
        }
        assert!(named);
        if fault != 0 {
            assert!(engine.setup.io.is_empty(), "refused before any request");
        }
        assert!(engine.forge.starts.is_empty());
        assert!(engine.setup.writes.is_empty());
        no_setup_events(engine);
        other_progresses(&mut service);
    }
}

fn no_setup_events(engine: &crate::engine::Engine) {
    for event in &engine.events {
        if let root::Event::Setup { .. } = event {
            panic!("admin and setup stop text never enter root events");
        }
    }
}

fn told_local(engine: &crate::engine::Engine, expected: &domain::Connection) -> bool {
    for event in &engine.events {
        if let root::Event::Forge(event) = event
            && let domain::Event::Connection { forge: 1, state } = event.as_ref()
            && state == expected
        {
            return true;
        }
    }
    false
}
fn with_handover(token: &[u8]) -> crate::settings::Engine {
    let mut config = local_config(true, false);
    inject_handover(&mut config, Secret::new(token.into()));
    config
}
fn takeover_to_kept(service: &mut crate::Service, face: &mut Face) {
    local_sent(service, face, b"GET /api/v1/user ");
    face.reply(service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.setup.writes.len(), 1);
    assert_eq!(engine.setup.writes.first().unwrap().1.number, records::number(1, Item::AdminToken));
    assert!(service.take_handover_removal().is_none(), "no removal before Kept");
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.setup.answers.is_empty(), "Written waits for Kept");
    let Some(StoreEvent::Kept { owner }) = engine.store.above.pop() else { panic!("durable AdminToken") };
    answer(engine, StoreEvent::Kept { owner });
    let Some((_, host::Answer::Written)) = engine.setup.answers.pop() else { panic!("Kept answers Written") };
    assert!(service.take_handover_removal().is_none(), "Kept alone does not remove");
    local_sent(service, face, b"GET /api/v1/settings/api ");
    face.reply(service, br#"{"max_response_items":50,"default_paging_num":10}"#);
    local_sent(service, face, b"GET /api/v1/admin/users/temper-admin/tokens");
}
pub(crate) fn takeover_done(service: &mut crate::Service, face: &mut Face) -> crate::HandoverRemoval {
    takeover_to_kept(service, face);
    face.hold_done = true;
    face.reply(service, b"[]");
    assert!(face.done.is_some());
    assert!(service.take_handover_removal().is_none(), "held Done blocks removal");
    service.engine.as_mut().unwrap().setup.above.push(face.done.take().unwrap());
    advance(service.engine.as_mut().unwrap());
    let remove = service.take_handover_removal().expect("Done requests removal");
    assert!(service.take_handover_removal().is_none(), "one correlated removal");
    assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
    remove
}
pub(crate) fn box_started(service: &mut crate::Service) -> crate::HandoverRead {
    while service.notes().pop().is_some() {}
    let engine = service.engine.as_mut().unwrap();
    let mut out = Queue::with_capacity(3);
    engine.box_link = crate::route::box_link::Link::new(128, true, 4);
    engine.box_link.ask(1, Token::new(456), &mut out);
    crate::route::box_link::drain(engine, &mut out);
    engine.box_link.write = None;
    engine.box_link.read(b"started 1\n", &mut out);
    crate::route::box_link::drain(engine, &mut out);
    assert!(told_local(engine, &domain::Connection::Connecting));
    service.take_handover_read().expect("box preparation requests the handover")
}
pub(crate) fn finish_local(service: &mut crate::Service, face: &mut Face) {
    face.hold_done = false;
    reconcile_to_repository(service, face);
    face.reply(service, br#"{"permission":"write","user":{"id":42,"login":"temper"}}"#);
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.forge.starts.is_empty());
    let Some(StoreEvent::Kept { owner }) = engine.store.above.pop() else { panic!("durable Origin") };
    answer(engine, StoreEvent::Kept { owner });
    finish_fake_adoption(service, face);
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.forge.starts.get(0).unwrap().1.user, 42);
    lifecycle::connected(engine, 1);
}
#[test]
fn takeover_waits_for_kept_done_and_correlated_removal_before_restored_reconcile() {
    let mut service = local_with_other(with_handover(b"test-admin"), local_store(false));
    restore_local(&mut service);
    let mut face = Face::new();
    let remove = takeover_done(&mut service, &mut face);
    other_progresses(&mut service);
    service.handover_removed(2, remove.run, Ok(()));
    advance(service.engine.as_mut().unwrap());
    service.handover_removed(1, Token::new(0), Ok(()));
    advance(service.engine.as_mut().unwrap());
    let lifecycle::FileStep::Removing { .. } = service.engine.as_ref().unwrap().setup.lifecycle.local(1).unwrap().file
    else {
        panic!("unrelated removal answers change nothing")
    };
    service.handover_removed(1, remove.run, Ok(()));
    advance(service.engine.as_mut().unwrap());
    finish_local(&mut service, &mut face);
    let engine = service.engine.as_mut().unwrap();
    assert_eq!(engine.values.identities.newest(1).unwrap().0, 1);
    engine.forge.starts.clear();
    engine.forge_records.rows.push(
        records::number(1, Item::GitToken),
        Secret::new(records::encode(Item::GitToken, b"replacement-git").unwrap().into_boxed_slice()),
        engine.setup.value_bytes,
    );
    lifecycle::connect_local(engine, 1);
    assert_eq!(engine.values.identities.newest(1).unwrap().0, 2, "every local connect puts the current git token");
    let identity = fixtures::decoded_identity(engine.values.identities.newest(1).unwrap().1);
    let temper_protocol_worker_wire::v1::Kind::Forgejo(token) = identity.kind() else { panic!("local git identity") };
    assert!(token.token().expose() == b"replacement-git", "the next generation serves the replacement");
    for generation in [1, 2] {
        let mut count = 0_u32;
        for event in &engine.events {
            if let root::Event::Credential { number: 1, generation: found, .. } = event
                && *found == generation
            {
                count = count.checked_add(1).unwrap();
            }
        }
        assert_eq!(count, 1);
    }
}
#[test]
fn restart_always_takes_a_present_handover_even_when_the_record_differs() {
    for next in [Some(b"test-admin".as_slice()), Some(b"different-admin".as_slice()), None] {
        let mut service = local_service(with_handover(b"test-admin"), local_store(false));
        restore_local(&mut service);
        let mut face = Face::new();
        let remove = takeover_done(&mut service, &mut face);
        service.handover_removed(1, remove.run, Err(21_i32));
        advance(service.engine.as_mut().unwrap());
        let engine = service.engine.as_mut().unwrap();
        assert!(engine.forge.starts.is_empty());
        assert!(engine.setup.runs.is_empty(), "failed removal blocks reconcile");
        let store = core::mem::replace(&mut engine.store, fixtures::records_store());
        let config = match next {
            Some(value) => with_handover(value),
            None => local_config(true, false),
        };
        let mut restarted = local_service(config, store);
        restore_local(&mut restarted);
        let engine = restarted.engine.as_ref().unwrap();
        let kind = engine.setup.runs.first().unwrap().1.kind;
        assert_eq!(kind, if next.is_some() { Kind::LocalTakeOver } else { Kind::LocalReconcile });
        if let Some(value) = next {
            let mut face = Face::new();
            let remove = takeover_done(&mut restarted, &mut face);
            let engine = restarted.engine.as_ref().unwrap();
            assert!(
                engine.forge_records.rows.forge(1).unwrap().get(Item::AdminToken).unwrap() == value,
                "the present handover replaces the record"
            );
            restarted.handover_removed(1, remove.run, Ok(()));
            advance(restarted.engine.as_mut().unwrap());
            assert_eq!(restarted.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalReconcile);
        }
    }
}
#[test]
fn an_asked_forge_takes_its_handover_at_started_and_connects() {
    let config = local_config(true, false);
    let mut service = local_service(config, local_store(false));
    restore_local(&mut service);
    assert!(!service.engine.as_ref().unwrap().setup.lifecycle.local(1).unwrap().pending);
    let engine = service.engine.as_ref().unwrap();
    assert!(
        told_local(engine, &domain::Connection::NotSetUp)
            || engine.domain.forge_connection(1) == domain::Connection::NotSetUp,
        "the stopped local start preserves the client's setup offer"
    );
    let read = box_started(&mut service);
    assert!(service.take_handover_read().is_none());
    service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
    advance(service.engine.as_mut().unwrap());
    let mut face = Face::new();
    let remove = takeover_done(&mut service, &mut face);
    service.handover_removed(1, remove.run, Ok(()));
    advance(service.engine.as_mut().unwrap());
    finish_local(&mut service, &mut face);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.values.identities.newest(1).unwrap().0, 1);
    assert!(told_local(engine, &domain::Connection::Connected { id: 42, login: b"temper".as_slice().into() }));
}
#[test]
fn a_takeover_that_stops_at_started_fails_naming_its_step_and_keeps_the_file() {
    let mut service = local_with_other(local_config(true, false), local_store(false));
    restore_local(&mut service);
    let read = box_started(&mut service);
    service.handover_read(1, read.run, Ok(Some(Secret::new(Box::new([])))));
    advance(service.engine.as_mut().unwrap());
    advance(service.engine.as_mut().unwrap());
    assert!(service.take_handover_removal().is_none());
    let engine = service.engine.as_ref().unwrap();
    assert!(told_local(
        engine,
        &domain::Connection::SetUpFailed { why: b"taking the handover: Admin: Missing".as_slice().into() }
    ));
    assert!(engine.forge.starts.is_empty());
    other_progresses(&mut service);
}
#[test]
fn a_failed_removal_at_started_blocks_reconcile_and_the_next_ask_takes_it_over_again() {
    let mut service = local_service(local_config(true, false), local_store(false));
    restore_local(&mut service);
    for attempt in 0_u32..2 {
        let read = box_started(&mut service);
        service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
        advance(service.engine.as_mut().unwrap());
        let mut face = Face::new();
        let remove = takeover_done(&mut service, &mut face);
        service.handover_removed(1, remove.run, if attempt == 1 { Ok(()) } else { Err(21_i32) });
        advance(service.engine.as_mut().unwrap());
        let engine = service.engine.as_ref().unwrap();
        assert!(engine.forge.starts.is_empty());
        if attempt == 0 {
            assert!(engine.setup.runs.is_empty());
            assert!(told_local(
                engine,
                &domain::Connection::SetUpFailed {
                    why: b"removing the handover: forge 1's admin_token: errno 21".as_slice().into()
                }
            ));
        } else {
            assert_eq!(engine.setup.runs.first().unwrap().1.kind, Kind::LocalReconcile);
        }
    }
}
#[test]
fn takeover_stops_missing_and_busy_keep_the_handover_and_isolate_the_forge() {
    for fault in 0_u8..4 {
        let config = if fault == 1 { with_handover(b"") } else { with_handover(b"test-admin") };
        let mut service = local_with_other(config, local_store(false));
        for _ in 0_u32..8 {
            crate::forge::records::advance(service.engine.as_mut().unwrap());
        }
        advance(service.engine.as_mut().unwrap());
        if fault == 2 {
            service.engine.as_mut().unwrap().setup.served.remove(&1);
        }
        if fault == 3 {
            service
                .engine
                .as_mut()
                .unwrap()
                .setup
                .runs
                .insert(
                    Token::new(76),
                    Run {
                        first: None,
                        forge: 1,
                        kind: Kind::Adopt,
                        internal: true,
                        previous: temper_engine_domain_forge::Connection::NotConnected,
                    },
                )
                .unwrap();
        }
        restore_local(&mut service);
        if fault == 0 {
            let mut face = Face::new();
            local_sent(&mut service, &mut face, b"GET /api/v1/user ");
            face.wire.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
            for _ in 0_u32..1024 {
                face.turn(&mut service);
            }
        }
        let expected = match fault {
            0 | 1 => crate::SetupStopped::Stopped {
                plan: crate::SetupPlan::TakeOver,
                step: crate::SetupStep {
                    stage: reconcile::Stage::Admin,
                    why: if fault == 0 { reconcile::Why::Refused(403) } else { reconcile::Why::Missing },
                },
            },
            2 => crate::SetupStopped::Missing,
            3 => crate::SetupStopped::Busy,
            _ => unreachable!(),
        };
        let engine = service.engine.as_ref().unwrap();
        let mut named = false;
        for note in &engine.notes {
            if let crate::Note::ForgeSetupStopped { number: 1, why } = note {
                named |= *why == expected;
            }
        }
        assert!(named, "fault {fault} has its typed failure");
        assert!(engine.forge.starts.is_empty());
        assert!(engine.setup.lifecycle.local(1).unwrap().handover.is_none(), "stops discard the staged secret");
        assert!(service.take_handover_removal().is_none(), "stops retain the file");
        other_progresses(&mut service);
    }
}
#[test]
fn unreadable_or_missing_handover_at_started_is_a_named_retryable_failure() {
    for answer in [Ok(None), Err(crate::HandoverUnread::Unsafe(skein_io::file::Unsafe::Link))] {
        let mut service = local_service(local_config(true, false), local_store(false));
        restore_local(&mut service);
        let read = box_started(&mut service);
        service.handover_read(1, read.run, answer);
        advance(service.engine.as_mut().unwrap());
        let engine = service.engine.as_ref().unwrap();
        let mut failed = false;
        for event in &engine.events {
            if let root::Event::Forge(event) = event
                && let domain::Event::Connection { forge: 1, state: domain::Connection::SetUpFailed { why } } =
                    event.as_ref()
            {
                failed |= why.as_ref() == b"the box's preparation left no handover"
                    || why.starts_with(b"the handover is unreadable: forge 1's admin_token: ");
            }
        }
        assert!(failed);
        assert!(engine.setup.runs.is_empty());
        assert!(service.take_handover_removal().is_none());
        let read = box_started(&mut service);
        service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
        advance(service.engine.as_mut().unwrap());
        assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalTakeOver);
    }
}
#[test]
fn closing_discards_pending_file_correlations_and_late_answers() {
    for removal in [false, true] {
        let mut service = local_service(local_config(true, false), local_store(false));
        restore_local(&mut service);
        let read = box_started(&mut service);
        let run = if removal {
            service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
            advance(service.engine.as_mut().unwrap());
            takeover_done(&mut service, &mut Face::new()).run
        } else {
            read.run
        };
        service.engine.as_mut().unwrap().setup.close();
        service.handover_read(1, run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
        advance(service.engine.as_mut().unwrap());
        service.handover_removed(1, run, Ok(()));
        advance(service.engine.as_mut().unwrap());
        assert!(service.take_handover_read().is_none());
        assert!(service.take_handover_removal().is_none());
        assert!(service.engine.as_ref().unwrap().forge.starts.is_empty());
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the diagnostic protocol test checks UTF-8 after truncation")]
fn setup_reasons_fit_the_wire_at_a_character_boundary() {
    let mut text = skein_lib::Writer::new(257);
    text.put(&[b'x'; 255]).unwrap();
    text.put("é".as_bytes()).unwrap();
    for (text, length) in [
        (Box::from([b'x'; 255].as_slice()), 255),
        (Box::from([b'x'; 256].as_slice()), 256),
        (Box::from([b'x'; 257].as_slice()), 256),
        (text.finish(), 255),
    ] {
        let reason = lifecycle::bounded_reason(text);
        assert_eq!(reason.len(), length);
        core::str::from_utf8(&reason).unwrap();
    }
}

#[test]
fn file_answers_wait_for_output_room_and_ignore_other_correlations() {
    let mut service = local_service(local_config(true, false), local_store(false));
    restore_local(&mut service);
    let read = box_started(&mut service);
    service.handover_read(2, read.run, Ok(None));
    service.handover_read(1, Token::new(0), Ok(None));
    let lifecycle::FileStep::Reading { sent: true, .. } =
        service.engine.as_ref().unwrap().setup.lifecycle.local(1).unwrap().file
    else {
        panic!("unrelated read answers change nothing")
    };
    let engine = service.engine.as_mut().unwrap();
    while engine.notes.room() > 0 {
        engine.notes.push(crate::Note::ForgeSetupStopped { number: 2, why: crate::SetupStopped::Busy });
    }
    service.handover_read(1, read.run, Ok(Some(Secret::new(b"test-admin".as_slice().into()))));
    advance(service.engine.as_mut().unwrap());
    assert!(service.engine.as_ref().unwrap().setup.runs.is_empty(), "read waits for output room");
    while service.notes().pop().is_some() {}
    advance(service.engine.as_mut().unwrap());
    let remove = takeover_done(&mut service, &mut Face::new());
    let engine = service.engine.as_mut().unwrap();
    while engine.notes.room() > 0 {
        engine.notes.push(crate::Note::ForgeSetupStopped { number: 2, why: crate::SetupStopped::Busy });
    }
    service.handover_removed(1, remove.run, Ok(()));
    advance(service.engine.as_mut().unwrap());
    assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalRemoval);
    while service.notes().pop().is_some() {}
    advance(service.engine.as_mut().unwrap());
    assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalReconcile);
}

#[test]
fn a_missing_file_at_started_reconciles_the_kept_token_and_names_a_stop() {
    let mut service = local_with_other(local_config(true, false), local_store(true));
    restore_local(&mut service);
    let mut face = Face::new();
    for asked in [false, true] {
        if asked {
            let read = box_started(&mut service);
            service.handover_read(1, read.run, Ok(None));
            advance(service.engine.as_mut().unwrap());
        }
        assert_eq!(service.engine.as_ref().unwrap().setup.runs.first().unwrap().1.kind, Kind::LocalReconcile);
        assert!(service.take_handover_removal().is_none());
        local_sent(&mut service, &mut face, b"GET /api/v1/user ");
        face.wire.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
        for _ in 0_u32..1024 {
            face.turn(&mut service);
        }
    }
    assert!(told_local(
        service.engine.as_ref().unwrap(),
        &domain::Connection::SetUpFailed { why: b"reconciling: Admin: Refused(403)".as_slice().into() }
    ));
    other_progresses(&mut service);
}

#[path = "setup_bound_webhook.rs"]
mod bound_webhook;

#[path = "setup_admin.rs"]
mod admin_tests;

#[path = "setup_create.rs"]
mod create_tests;

fn demo_config() -> crate::settings::Engine {
    local_config(true, false)
}
#[test]
fn an_end_during_handover_or_reconcile_cannot_continue_setup() {
    for during_reconcile in [false, true] {
        let mut config = demo_config();
        inject_handover(&mut config, Secret::new(b"test-admin".as_slice().into()));
        let mut service = local_service(config, local_store(true));
        restore_local(&mut service);
        let engine = service.engine.as_mut().unwrap();
        let run = *engine.setup.runs.first().unwrap().0;
        if during_reconcile {
            engine.setup.runs.get_mut(&run).unwrap().kind = Kind::LocalReconcile;
        }
        let mut record = engine.records.get(&1).unwrap().clone();
        record.state = domain::ForgeState::Removing;
        engine.step(root::Event::Forge(Box::new(domain::Event::Restore { record: domain::Stored::Forge(record) })));
        lifecycle::done(engine, run, reconcile::Outcome::Nothing);
        assert!(engine.setup.runs.is_empty());
        assert!(engine.setup.writes.is_empty());
        assert!(engine.forge.starts.is_empty());
        assert!(service.take_handover_removal().is_none());
    }
}
#[test]
fn github_app_urls_use_the_clients_public_origin() {
    let public = crate::startup::application::Origin {
        scheme: crate::startup::application::Scheme::Https,
        host: Box::from(b"temper.example.com".as_slice()),
        port: Some(8443),
        name: Box::from(b"temper.example.com:8443".as_slice()),
    };
    let mut service = routing_service_at(8, Duration::from_secs(3600), Some(public));
    let state = manifest(&mut service, 1);
    let page = service.setup_page(1, &state, Time::ZERO).unwrap();
    assert!(contains(page, b"https://temper.example.com:8443/forges/1/github/redirect"));
    assert!(contains(page, b"https://temper.example.com:8443/forges/1/github/setup"));
}

#[test]
fn setup_reconciliation_waits_for_the_mark() {
    // The shared service restart fixture's existing iteration horizon.
    const PASSES: u32 = 256;
    let mut previous = fixtures::ready().engine.unwrap();
    kept_record(&mut previous.store, 1, Item::AdminToken, b"test-admin");
    let mut service = fixtures::restarting_from(previous);
    let config = local_config(true, false);
    let setup = prepare(&config, &fixtures::seeds().crypto, false).unwrap();
    service.engine.as_mut().unwrap().setup = setup;
    let engine = service.engine.as_mut().unwrap();
    engine.workers.forges.get_mut(0).unwrap().api = crate::forges::settings(config.machine.owned.as_ref().unwrap()).api;
    register(engine, config.machine.owned.as_ref().unwrap());
    service.engine.as_mut().unwrap().store.hold_mark = true;
    for _ in 0..PASSES {
        fixtures::turn(&mut service);
    }
    let engine = service.engine.as_mut().unwrap();
    assert!(
        engine.store.held_mark.is_some(),
        "end {:?}, room {}, restart {:?}, ready {}",
        engine.end,
        engine.room(),
        engine.domain.restart_failure(),
        engine.domain.ready()
    );
    assert!(!engine.setup.lifecycle.startup_restored());
    assert!(engine.setup.component.is_none());
    assert!(engine.setup.runs.is_empty());
    assert!(engine.setup.io.is_empty());
    let mark = engine.store.held_mark.take().unwrap();
    engine.store.hold_mark = false;
    assert!(engine.store.down(mark));
    for _ in 0..PASSES {
        crate::engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    assert!(engine.store.component.mark_durable());
    assert!(engine.setup.lifecycle.startup_restored());
    assert!(engine.setup.component.is_some());
    assert_eq!(engine.setup.runs.len(), 1, "one configured reconciliation");
    for _ in 0..PASSES {
        crate::engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    assert_eq!(engine.setup.runs.len(), 1, "no second reconciliation while its file answer is held");
}

fn github_record() -> domain::ForgeRecord {
    let mut record = fixtures::forge_record(1);
    record.kind = domain::Kind::GitHub;
    record.bot.name = Box::new([]);
    record.bot.email = Box::new([]);
    record.api.port = 3000;
    record.web = Some(b"http://127.0.0.1:3000".as_slice().into());
    record
}
fn github_service(config: crate::settings::Engine) -> crate::Service {
    let mut record = config.machine.github.clone();
    record.number = 1;
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None).unwrap();
    fixtures::register_forges(&mut service, &[record]);
    service
}
fn inject_handover(config: &mut crate::settings::Engine, value: Secret) {
    let mut entries = List::with_capacity(config.files.entries.len().checked_add(1).unwrap());
    for (path, bytes) in core::mem::replace(&mut config.files.entries, List::with_capacity(0)).into_boxed() {
        if path.as_ref() != b"/fixture/handover" {
            entries.push((path, bytes)).unwrap();
        }
    }
    entries.push((b"/fixture/handover".as_slice().into(), value)).unwrap();
    config.files.entries = entries;
}

fn resolved_githubs(config: crate::settings::Engine, runtime: List<crate::ForgeRuntime>) -> crate::Service {
    let mut record = config.machine.github.clone();
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None).unwrap();
    for endpoint in runtime.into_boxed() {
        record.number = endpoint.number;
        fixtures::register_forges(&mut service, &[record.clone()]);
        service.resolved(record.number, Ok(endpoint));
    }
    service
}
