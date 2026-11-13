#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "test scans captured diagnostics and inspects closed event variants"
)]
//! Connected Forgejo token admission and durable Check sequencing (temper/docs/design/protocol/forge/engine.md, 6).
use super::*;

#[test]
fn incomplete_and_admin_bundles_stop_before_any_plan() {
    for codes in [&[2_u16][..], &[1_u16, 2, 3][..]] {
        let mut service = routing_service(4, Duration::from_secs(60));
        let engine = service.engine.as_mut().unwrap();
        let mut rng = Rng::new(19);
        let mut bytes = [0_u8; 32];
        for byte in &mut bytes {
            *byte = b'a'.checked_add(u8::try_from(rng.below(26)).unwrap()).unwrap();
        }
        let mut items = List::with_capacity(3);
        for code in codes {
            items.push((*code, bytes.as_slice())).unwrap();
        }
        let value = records::encode_pasted(items.as_slice()).unwrap();
        let digest = skein_lib::sha256::digest(value.expose());
        engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
        assert!(engine.store.host.above.pop().is_some());
        public::tokens_connect(
            engine,
            1,
            Token::new(90),
            jig_domain_core::Carried { number: records::CARRIED, digest },
        );
        assert!(engine.setup.public.is_empty());
        assert!(engine.setup.staged.is_empty());
        assert!(engine.setup.runs.is_empty());
        assert!(engine.events.iter().any(|event| match event {
            root::Event::Forge(event) => match event.as_ref() {
                domain::Event::ConnectEnded { connected: false, why: Some(why), stop: None, .. } =>
                    why.as_ref() == b"the pasted values are not a Forgejo's tokens",
                _ => false,
            },
            _ => false,
        }));
    }
}

use super::create_tests::Forge;
use jig_protocol_store::{SecretOwner, Secrets};
use skein_forge_connection as connection;
use skein_lib::Reader;
use temper_client_protocol_wire::v1 as w;

pub(super) struct Tokens {
    pub(super) api: Secret,
    pub(super) git: Secret,
    pub(super) hook: Secret,
}
impl Tokens {
    pub(super) fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        Self { api: random(&mut rng), git: random(&mut rng), hook: random(&mut rng) }
    }
    pub(super) fn bundle(&self, hook: bool) -> Secret {
        temper_client_domain::forge::forges::pasted_tokens(
            Some(self.api.expose()),
            Some(self.git.expose()),
            if hook { Some(self.hook.expose()) } else { None },
        )
        .unwrap()
    }
    pub(super) fn give(&self, forge: &mut Forge, login: &[u8], admin: bool, other_git_user: bool) -> u64 {
        let setup = forge.service.setup_mut();
        let user = match setup.find(login) {
            Some(account) => account.id,
            None => setup.account(login, admin, b""),
        };
        setup.token(user, b"api", &[b"read:user", b"write:repository", b"write:issue"], self.api.clone());
        let git_user = if other_git_user { setup.account(b"other-user", false, b"") } else { user };
        setup.token(git_user, b"git", &[b"read:user", b"write:repository"], self.git.clone());
        user
    }
    pub(super) fn scan(&self, engine: &crate::engine::Engine) {
        for secret in [&self.api, &self.git, &self.hook] {
            for note in &engine.notes {
                assert!(!contains(alloc::format!("{note:?}").as_bytes(), secret.expose()));
            }
            for event in &engine.events {
                assert!(!contains(alloc::format!("{event:?}").as_bytes(), secret.expose()));
            }
            for request in &engine.requests {
                assert!(!contains(alloc::format!("{request:?}").as_bytes(), secret.expose()));
            }
            for answer in &engine.setup.answers {
                assert!(!contains(alloc::format!("{answer:?}").as_bytes(), secret.expose()));
            }
            assert!(!contains(alloc::format!("{:?}", engine.domain.forges()).as_bytes(), secret.expose()));
        }
    }
}
fn random(rng: &mut Rng) -> Secret {
    let mut bytes = [0_u8; 32];
    for byte in &mut bytes {
        *byte = b'a'.checked_add(u8::try_from(rng.below(26)).unwrap()).unwrap();
    }
    Secret::new(Box::from(bytes))
}
fn empty_settings() -> crate::settings::Engine {
    let mut settings = fixtures::engine();
    fixtures::fill_files(&mut settings);
    settings
}
pub(super) fn owner() -> (crate::Service, Box<[u8]>) {
    use jig_client_protocol_wire::v21 as j;
    use jig_domain_people as people;
    let mut bootstrap = fixtures::ready();
    let mut header = bootstrap.engine.as_ref().unwrap().domain.deployment();
    header.commits = header.commits.checked_add(1).unwrap();
    let mut store = bootstrap.engine.take().unwrap().store;
    fixtures::commit(
        &mut store,
        header.commits,
        Box::new([
            root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(header),
            ))),
            root::Write::Erase(root::Key::Forge(domain::Key::Forge(1))),
            root::Write::Save(root::Record::Forge { row: Box::new(domain::Stored::ForgeNumber { last: 0 }) }),
            root::Write::Save(root::Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
                project: 0,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            }))),
        ]),
    );
    assert!(store.host.above.pop().is_some());
    let mut service =
        crate::Service::engine(empty_settings(), &fixtures::seeds(), None, List::with_capacity(0)).unwrap();
    service.engine.as_mut().unwrap().store = store;
    restore_local(&mut service);
    assert!(service.engine.as_ref().unwrap().domain.forges().rows.is_empty());
    let link = service.links().pop().unwrap();
    let engine = service.engine.as_mut().unwrap();
    let body = fixtures::http::link_body(engine, &link);
    let mut peer = fixtures::http::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine, true);
    let j::SignInAnswer::SignedIn(answer) = peer.sign_in_answer(engine) else { panic!("owner sign in") };
    (service, fixtures::http::bearer(answer.token().as_ref().unwrap().expose()))
}
fn making(new: bool) -> w::Making {
    if !new {
        return w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 1 }).unwrap());
    }
    let place = w::ManagedForge::new(
        &w::CEILINGS,
        w::ManagedForgeParts {
            host: b"127.0.0.1".as_slice().into(),
            port: 9,
            tls: None,
            git: b"http://127.0.0.1:9".as_slice().into(),
            web: None,
            git_program: None,
        },
    )
    .unwrap();
    w::Making::New(w::NewForge::new(&w::CEILINGS, w::NewForgeParts { forge: w::ForgeAsk::Connected(place) }).unwrap())
}
pub(super) fn http_connect(service: &mut crate::Service, bearer: &[u8], key: u8, new: bool, bundle: Secret) {
    let request = w::Request::Connect(
        w::Connect::new(
            &w::CEILINGS,
            w::ConnectParts {
                forge: making(new),
                how: w::How::ForgejoTokens(w::ForgejoTokens::new(&w::CEILINGS, w::ForgejoTokensParts {}).unwrap()),
            },
        )
        .unwrap(),
    );
    fixtures::forge_connecting(service, bearer, key, request, bundle);
}

pub(super) struct Trace {
    pub(super) keeps: List<(Token, u64)>,
    login: bool,
    ended: bool,
    starts: Option<u32>,
    erase: bool,
}
impl Trace {
    pub(super) fn new() -> Self {
        Self { keeps: List::with_capacity(8), login: false, ended: false, starts: None, erase: false }
    }
    pub(super) fn scan(&mut self, engine: &crate::engine::Engine, tokens: &Tokens) {
        tokens.scan(engine);
        if self.starts.is_none() {
            self.starts = Some(engine.forge.starts.len());
        }
        if !engine.setup.erases.is_empty() {
            assert_eq!(engine.forge.starts.len(), self.starts.unwrap(), "Connect waits for Erased");
            assert_eq!(engine.workers.forges.bot_login(1), Some(b"checked-user".as_slice()));
            self.erase = true;
        }
        assert!(engine.setup.writes.len() <= 1, "keeps are serialized");
        for (owner, write) in &engine.setup.writes {
            if !self.keeps.iter().any(|(seen, _)| seen == owner) {
                if let Some((previous, number)) = self.keeps.iter().last() {
                    assert!(!engine.setup.writes.contains_key(previous));
                    assert!(engine.forge_records.rows.forge(1).unwrap().has(item(*number)));
                }
                self.keeps.push((*owner, write.number)).unwrap();
            }
        }
        for event in &engine.events {
            if let root::Event::Forge(event) = event {
                match event.as_ref() {
                    domain::Event::BotLogin { login, .. } => {
                        assert_eq!(login.as_ref(), b"checked-user");
                        self.login = true;
                    }
                    domain::Event::ConnectEnded { connected: true, .. } => {
                        assert!(self.login);
                        self.ended = true;
                    }
                    _ => {}
                }
            }
        }
        if !engine.forge.starts.is_empty() {
            assert!(engine.store.erases > 0, "Connect follows the carried erase");
            assert!(engine.values.identities.newest(1).is_some(), "identity precedes Connect");
        }
    }
}
fn item(number: u64) -> Item {
    match u16::try_from(number & 0xffff).unwrap() {
        2 => Item::ApiToken,
        3 => Item::GitToken,
        4 => Item::User,
        5 => Item::WebhookSecret,
        10 => Item::Origin,
        _ => panic!("unexpected Check write"),
    }
}
fn turn(service: &mut crate::Service, forge: &mut Forge, tokens: &Tokens, trace: &mut Trace, tick: u64) {
    let engine = service.engine.as_mut().unwrap();
    trace.scan(engine, tokens);
    while engine.notes.pop().is_some() {}
    while engine.setup.answers.pop().is_some() {}
    engine.env.now = Time::from_nanos(tick.checked_mul(1_000_000).unwrap());
    advance(engine);
    trace.scan(engine, tokens);
    engine.forge.iterate(engine.env.now, Wall::EPOCH);
    forge.turn_io(engine);
    forge.turn_server(engine);
    for _ in 0_u32..4 {
        trace.scan(engine, tokens);
        crate::engine::answer(engine);
        trace.scan(engine, tokens);
        // Keep the next completion across one service pass so every durable boundary is observed.
        let mut held = core::mem::replace(&mut engine.store.host.above, Queue::with_capacity(64));
        crate::engine::iterate(engine, engine.env.now, Wall::EPOCH);
        trace.scan(engine, tokens);
        while let Some(event) = held.pop() {
            engine.store.host.above.push(event);
        }
    }
    forge.service.reclaim();
}
fn is_connected(state: domain::Connection) -> bool {
    match state {
        domain::Connection::Connected { .. } => true,
        _ => false,
    }
}
pub(super) fn settle(service: &mut crate::Service, forge: &mut Forge, tokens: &Tokens, connected: bool) -> Trace {
    let mut trace = Trace::new();
    for tick in 1..20_000 {
        turn(service, forge, tokens, &mut trace, tick);
        let engine = service.engine.as_ref().unwrap();
        if engine.setup.public.is_empty() && engine.setup.runs.is_empty() && engine.store.erases > 0 {
            if connected {
                if !is_connected(engine.domain.forge_connection(1)) {
                    continue;
                }
            } else if engine.domain.forges().rows[0].ended.is_none() {
                continue;
            }
            return trace;
        }
    }
    panic!("tokens connect did not settle");
}
pub(super) fn erased(service: &mut crate::Service) {
    let engine = service.engine.as_mut().unwrap();
    assert!(match engine
        .store
        .secrets(Token::new(902), Secrets::Get { owner: SecretOwner::Application, number: records::CARRIED },)
    {
        Some(StoreEvent::Secret { value: None, .. }) => true,
        _ => false,
    });
    let Some(StoreEvent::SecretsLoaded { rows, .. }) = engine.store.secrets(
        Token::new(903),
        Secrets::Load { owner: SecretOwner::Application, after: Some(records::CARRIED.checked_sub(1).unwrap()) },
    ) else {
        panic!("carried rows")
    };
    assert!(rows.is_empty(), "every carried row was erased");
}
pub(super) fn connected(service: &crate::Service, user: u64, tokens: &Tokens, generation: u64) {
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(
        engine.domain.forge_connection(1),
        domain::Connection::Connected { id: user, login: b"checked-user".as_slice().into() }
    );
    let (number, access) = engine.forge.starts.iter().last().unwrap();
    assert_eq!((*number, access.user), (1, user));
    assert_eq!(access.credential, skein_forge::credential::Credential::Token(tokens.api.clone()));
    assert!(access.webhook_secret.is_none(), "a kept secret needs a configured listener");
    let (found, value) = engine.values.identities.newest(1).unwrap();
    assert_eq!(found, generation);
    let identity = temper_protocol_worker_wire::v1::Identity::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut Reader::new(value.expose()),
    )
    .unwrap();
    assert_eq!(identity.login(), b"checked-user");
    let temper_protocol_worker_wire::v1::Kind::Forgejo(auth) = identity.kind() else { panic!("Forgejo identity") };
    assert!(auth.token().expose() == tokens.git.expose());
    let rows = engine.forge_records.rows.forge(1).unwrap();
    assert!(rows.get(Item::ApiToken).unwrap() == tokens.api.expose());
    assert!(rows.get(Item::GitToken).unwrap() == tokens.git.expose());
}
#[test]
fn wire_tokens_keep_in_order_erase_connect_and_restart_as_the_checked_user() {
    let mut forge = Forge::new(false);
    let tokens = Tokens::new(21);
    let user = tokens.give(&mut forge, b"checked-user", false, false);
    let (mut service, bearer) = owner();
    http_connect(&mut service, &bearer, 1, true, tokens.bundle(true));
    let trace = settle(&mut service, &mut forge, &tokens, true);
    assert!(trace.login && trace.ended && trace.erase);
    let order: Box<[_]> = trace.keeps.into_boxed().into_iter().map(|(_, number)| item(number)).collect();
    assert_eq!(order.as_ref(), &[Item::User, Item::ApiToken, Item::GitToken, Item::WebhookSecret, Item::Origin]);
    connected(&service, user, &tokens, 1);
    erased(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let record = fixtures::load(&mut engine.store, root::Range::Forge)
        .into_iter()
        .find_map(|row| match row {
            root::Record::Forge { row } => match *row {
                domain::Stored::Forge(record) => Some(record),
                _ => None,
            },
            root::Record::Core(_) => None,
        })
        .unwrap();
    assert_eq!(record.mode, domain::Mode::Connected);
    assert_eq!(record.bot.login.as_ref(), b"checked-user");
    assert!(engine.forge_records.rows.forge(1).unwrap().get(Item::WebhookSecret).unwrap() == tokens.hook.expose());
    engine.forge.down(connection::Request::Close);
    for _ in 0_u32..16 {
        forge.turn_io(service.engine.as_mut().unwrap());
    }
    let store = service.engine.take().unwrap().store;
    let mut restarted =
        crate::Service::engine(empty_settings(), &fixtures::seeds(), None, List::with_capacity(0)).unwrap();
    restarted.engine.as_mut().unwrap().store = store;
    // Restore routes a records-based Connect before opening the clients' listener.
    let mut trace = Trace::new();
    for tick in 1..20_000 {
        turn(&mut restarted, &mut forge, &tokens, &mut trace, tick);
        if is_connected(restarted.engine.as_ref().unwrap().domain.forge_connection(1)) {
            break;
        }
    }
    connected(&restarted, user, &tokens, 1);
}

pub(super) fn stopped(service: &mut crate::Service, forge: &mut Forge, tokens: &Tokens) {
    // A refused replacement drops the component's access and closes its pooled connection.
    let engine = service.engine.as_mut().unwrap();
    let user = engine.workers.forges.get(1).unwrap().user;
    engine.forge.down(connection::Request::Connect {
        forge: 1,
        access: connection::Access {
            credential: skein_forge::credential::Credential::Token(Secret::new(Box::new([]))),
            user,
            webhook_secret: None,
        },
    });
    let mut trace = Trace::new();
    for tick in 1..64 {
        turn(service, forge, tokens, &mut trace, tick);
    }
    assert!(match service.engine.as_ref().unwrap().domain.forge_connection(1) {
        domain::Connection::Stopped { .. } => true,
        _ => false,
    });
}

fn record(service: &mut crate::Service) -> domain::ForgeRecord {
    fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge)
        .into_iter()
        .find_map(|row| match row {
            root::Record::Forge { row } => match *row {
                domain::Stored::Forge(record) => Some(record),
                _ => None,
            },
            root::Record::Core(_) => None,
        })
        .unwrap()
}
#[test]
fn stopped_forge_reconnects_only_as_its_kept_user_and_overwrites_each_pasted_row() {
    let mut forge = Forge::new(false);
    let old = Tokens::new(31);
    let user = old.give(&mut forge, b"checked-user", false, false);
    let (mut service, bearer) = owner();
    http_connect(&mut service, &bearer, 1, true, old.bundle(true));
    let _first = settle(&mut service, &mut forge, &old, true);
    connected(&service, user, &old, 1);
    stopped(&mut service, &mut forge, &old);
    let tokens = Tokens::new(32);
    assert_eq!(tokens.give(&mut forge, b"checked-user", false, false), user);
    http_connect(&mut service, &bearer, 2, false, tokens.bundle(true));
    let trace = settle(&mut service, &mut forge, &tokens, true);
    let order: Box<[_]> = trace.keeps.into_boxed().into_iter().map(|(_, number)| item(number)).collect();
    assert_eq!(order.as_ref(), &[Item::ApiToken, Item::GitToken, Item::WebhookSecret, Item::Origin]);
    connected(&service, user, &tokens, 2);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.store.erases, 2);
    assert_eq!(engine.forge.starts.len(), 3);
    assert!(engine.forge_records.rows.forge(1).unwrap().get(Item::WebhookSecret).unwrap() == tokens.hook.expose());
    erased(&mut service);
}
#[test]
fn another_users_tokens_keep_the_stopped_forge_and_its_records_unchanged() {
    let mut forge = Forge::new(false);
    let old = Tokens::new(41);
    let user = old.give(&mut forge, b"checked-user", false, false);
    let (mut service, bearer) = owner();
    http_connect(&mut service, &bearer, 1, true, old.bundle(false));
    let _first = settle(&mut service, &mut forge, &old, true);
    stopped(&mut service, &mut forge, &old);
    let before = service.engine.as_ref().unwrap().domain.forge_connection(1);
    let kept = record(&mut service);
    let tokens = Tokens::new(42);
    assert_ne!(tokens.give(&mut forge, b"another-user", false, false), user);
    http_connect(&mut service, &bearer, 2, false, tokens.bundle(true));
    let trace = settle(&mut service, &mut forge, &tokens, false);
    assert!(trace.keeps.is_empty());
    assert_eq!(record(&mut service), kept);
    let engine = service.engine.as_ref().unwrap();
    assert_eq!(engine.domain.forge_connection(1), before);
    let row = &engine.domain.forges().rows[0];
    assert_eq!(
        row.ended.as_deref(),
        Some(
            b"the tokens answer as another user than checked-user, this forge's: connect that user as a new forge"
                .as_slice()
        )
    );
    assert_eq!(row.ended_stop, None);
    assert_eq!(engine.forge.starts.len(), 2);
    let rows = engine.forge_records.rows.forge(1).unwrap();
    assert!(rows.get(Item::ApiToken).unwrap() == old.api.expose());
    assert!(rows.get(Item::GitToken).unwrap() == old.git.expose());
    assert!(!rows.has(Item::WebhookSecret));
    assert_eq!(engine.values.identities.newest(1).unwrap().0, 1);
    assert_eq!(engine.store.erases, 2);
    erased(&mut service);
}
enum Refusal {
    TwoUsers,
    Admin,
    Api,
    Git,
}
fn refusal(case: Refusal, words: &[u8]) {
    let mut forge = Forge::new(false);
    let tokens = Tokens::new(51);
    let (admin, other_user) = match case {
        Refusal::Admin => (true, false),
        Refusal::TwoUsers => (false, true),
        Refusal::Api | Refusal::Git => (false, false),
    };
    tokens.give(&mut forge, b"checked-user", admin, other_user);
    match case {
        Refusal::Api => {
            forge.service.setup_mut().revoke(tokens.api.expose());
        }
        Refusal::Git => {
            forge.service.setup_mut().revoke(tokens.git.expose());
        }
        Refusal::TwoUsers | Refusal::Admin => {}
    }
    let (mut service, bearer) = owner();
    http_connect(&mut service, &bearer, 1, true, tokens.bundle(true));
    let kept = record(&mut service);
    let trace = settle(&mut service, &mut forge, &tokens, false);
    assert!(trace.keeps.is_empty());
    assert_eq!(record(&mut service), kept);
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge_records.rows.forge(1).is_none());
    assert!(engine.forge.starts.is_empty());
    assert!(engine.values.identities.newest(1).is_none());
    assert_eq!(engine.domain.forge_connection(1), domain::Connection::NotConnected);
    assert_eq!(engine.domain.forges().rows[0].ended.as_deref(), Some(words));
    assert_eq!(engine.domain.forges().rows[0].ended_stop, None);
    erased(&mut service);
}
#[test]
fn two_users_tokens_stop_before_any_keep() {
    refusal(Refusal::TwoUsers, b"the git token answers as another user than the API token");
}
#[test]
fn site_admin_tokens_stop_before_any_keep() {
    refusal(Refusal::Admin, b"the API token is a site admin's: give the tokens of a user that is not one");
}
#[test]
fn a_refused_api_token_stops_before_any_keep() {
    refusal(Refusal::Api, b"Forgejo refused the API token");
}
#[test]
fn a_refused_git_token_stops_before_any_keep() {
    refusal(Refusal::Git, b"Forgejo refused the git token");
}

#[test]
fn malformed_wire_bundles_erase_without_staging_a_plan() {
    for codes in [&[2_u16][..], &[1_u16, 2, 3][..]] {
        let tokens = Tokens::new(61);
        let mut items = List::with_capacity(3);
        for code in codes {
            items.push((*code, tokens.api.expose())).unwrap();
        }
        let bundle = records::encode_pasted(items.as_slice()).unwrap();
        let (mut service, bearer) = owner();
        http_connect(&mut service, &bearer, 1, true, bundle);
        let mut forge = Forge::new(false);
        let trace = settle(&mut service, &mut forge, &tokens, false);
        assert!(trace.keeps.is_empty());
        let engine = service.engine.as_ref().unwrap();
        assert!(engine.setup.staged.is_empty() && engine.setup.runs.is_empty());
        assert!(engine.setup.io.is_empty());
        assert!(engine.forge_records.rows.forge(1).is_none());
        assert_eq!(
            engine.domain.forges().rows[0].ended.as_deref(),
            Some(b"the pasted values are not a Forgejo's tokens".as_slice())
        );
        erased(&mut service);
    }
}

#[test]
fn changed_digest_and_other_bundle_version_stop_before_any_plan() {
    let tokens = Tokens::new(71);
    let mut other_version: Box<[u8]> = Box::from(tokens.bundle(false).expose());
    other_version[..2].copy_from_slice(&2_u16.to_be_bytes());
    for (value, changed, words) in [
        (tokens.bundle(false), true, b"the pasted value changed".as_slice()),
        (Secret::new(other_version), false, records::PastedError::Version.message()),
        (Secret::new(Box::from([0_u8])), false, b"the pasted values are not a Forgejo's tokens".as_slice()),
    ] {
        let mut service = routing_service(4, Duration::from_secs(60));
        let engine = service.engine.as_mut().unwrap();
        let mut digest = skein_lib::sha256::digest(value.expose());
        if changed {
            digest[0] ^= 1;
        }
        engine.store.secrets(Token::new(901), fixtures::application_request(records::CARRIED, value));
        assert!(engine.store.host.above.pop().is_some());
        public::tokens_connect(
            engine,
            1,
            Token::new(90),
            jig_domain_core::Carried { number: records::CARRIED, digest },
        );
        assert!(engine.setup.public.is_empty() && engine.setup.runs.is_empty() && engine.setup.staged.is_empty());
        assert!(engine.setup.writes.is_empty());
        assert!(engine.events.iter().any(|event| match event {
            root::Event::Forge(event) => match event.as_ref() {
                domain::Event::ConnectEnded { connected: false, why: Some(why), stop: None, .. } =>
                    why.as_ref() == words,
                _ => false,
            },
            _ => false,
        }));
    }
}

#[test]
fn a_login_rewrite_preserves_the_endpoint_but_an_author_change_replaces_it() {
    let mut forge = Forge::new(false);
    let tokens = Tokens::new(81);
    tokens.give(&mut forge, b"checked-user", false, false);
    let (mut service, bearer) = owner();
    http_connect(&mut service, &bearer, 1, true, tokens.bundle(false));
    let _trace = settle(&mut service, &mut forge, &tokens, true);
    let mut kept = record(&mut service);
    kept.bot.login = b"renamed-user".as_slice().into();
    let engine = service.engine.as_mut().unwrap();
    crate::forges::record(engine, kept.clone());
    assert_eq!(engine.workers.forges.bot_login(1), Some(b"renamed-user".as_slice()));
    assert_eq!(engine.forge.starts.len(), 1);
    assert_eq!(engine.values.identities.newest(1).unwrap().0, 1);
    kept.bot.name = b"New author".as_slice().into();
    crate::forges::record(engine, kept);
    assert_eq!(engine.forge.starts.len(), 2);
    assert_eq!(engine.values.identities.newest(1).unwrap().0, 2);
    tokens.scan(engine);
}
