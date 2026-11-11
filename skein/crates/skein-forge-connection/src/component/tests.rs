use super::*;
use crate::tests::fixtures;

#[test]
fn call_sizes_distinguish_real_rows_single_recovery_and_github() {
    let limits = Limits { body_bytes: 16_384, string_bytes: 512, inner: 32, ..fixtures::limits() };
    let shared = Shared { rows: 8, answer_bytes: 65_536, ..fixtures::shared() };
    for (kind, single, expected) in [
        (crate::settings::Kind::Forgejo, false, 8),
        (crate::settings::Kind::Forgejo, true, 1),
        (crate::settings::Kind::GitHub, false, 8),
        (crate::settings::Kind::GitHub, true, 1),
    ] {
        let op = api::Op::Read(api::Read::Items { since: Time::ZERO, page: 4, kind: None, single });
        assert_eq!(calls::page_size(&op, kind, 50, &limits, &shared), expected);
    }
    for (read, github) in [
        (api::Read::Reviews { number: 1, page: 1, skip: 0 }, 8),
        (api::Read::Remarks { number: 1, review: 7, page: 1, skip: 0 }, 8),
        (api::Read::PullFiles { number: 1, head: [0; 32], page: 1, skip: 0, sides: true }, 8),
    ] {
        let op = api::Op::Read(read);
        for (kind, expected) in [(crate::settings::Kind::Forgejo, 8), (crate::settings::Kind::GitHub, github)] {
            assert_eq!(calls::page_size(&op, kind, 50, &limits, &shared), expected);
        }
    }
    let op = api::Op::Read(api::Read::Item { number: 1, after: 0, since: Time::ZERO });
    for (kind, expected) in [(crate::settings::Kind::Forgejo, 1), (crate::settings::Kind::GitHub, 8)] {
        assert_eq!(calls::page_size(&op, kind, 50, &limits, &shared), expected);
        assert_eq!(calls::page_size(&op, kind, 3, &limits, &shared), expected.min(3));
    }
}

// forge/connection.md 6.1 reserves the listener's owner prefix.
const LISTENER_OWNER: u64 = 1_u64 << 59_u32;
const LAST_API_OWNER: u64 = LISTENER_OWNER - 1;
fn token_fixture() -> (Component, Env<Limits>, Queue<skein_io::Request>) {
    let limits = fixtures::limits();
    let c = Component::new(fixtures::settings(), &limits, &fixtures::shared()).expect("valid");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    (c, env, Queue::uncapped())
}
#[test]
#[should_panic(expected = "connection token prefix")]
fn api_tokens_stop_before_the_listener_range() {
    let (mut c, env, mut io) = token_fixture();
    c.common.next_token = LISTENER_OWNER;
    c.test_open_own(&env, &mut io);
}
#[test]
fn the_last_api_token_is_still_issued() {
    let (mut c, env, mut io) = token_fixture();
    c.common.next_token = LAST_API_OWNER;
    assert_eq!(c.test_open_own(&env, &mut io).raw(), LAST_API_OWNER);
}

#[test]
fn forgejo_starts_leave_the_github_reserved_slot_free() {
    // Three Forgejo starts contend for the two call slots; GitHub has its own third slot.
    const FORGEJO_FORGES: u32 = 3;
    const GITHUB_AT: u32 = FORGEJO_FORGES;
    const FORGES: u32 = FORGEJO_FORGES + 1;
    let limits = Limits { forges: FORGES, ..fixtures::limits() };
    let mut settings = fixtures::settings();
    let mut forges = List::with_capacity(FORGES);
    for number in 1..=FORGEJO_FORGES {
        let mut forge = fixtures::settings().forges.into_boxed().into_iter().next().expect("fixture forge");
        forge.number = u16::try_from(number).expect("forge number");
        forges.push(forge).expect("forge room");
    }
    let mut github = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("fixture forge");
    github.number = u16::try_from(FORGES).expect("forge number");
    forges.push(github).expect("forge room");
    settings.forges = forges;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut c = Component::new(settings, &limits, &fixtures::shared()).expect("fixture limits");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    c.begin(&env, &mut up, &mut io);
    assert_eq!(c.common.call_connections(), limits.connections);
    assert!(c.forges.get(GITHUB_AT).expect("forge").base.own.is_some());
    assert_eq!(c.common.connections.len(), 3);
}

use skein_forge::github::document;
use skein_lib::{Duration, Secret, Wall};

struct StepFixture {
    component: Component,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<skein_io::Request>,
}
impl StepFixture {
    fn new(settings: Settings) -> Self {
        let limits = fixtures::limits();
        Self {
            component: Component::new(settings, &limits, &fixtures::shared()).expect("fixture"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            up: Queue::uncapped(),
            io: Queue::uncapped(),
        }
    }
    fn send(&mut self, request: Request) {
        self.component.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn local(&mut self) {
        self.component.test_local_step(&self.env, &mut self.up);
    }
    fn minted(&mut self, expires: u64, permissions: document::Levels) {
        self.component.test_minted(
            Ok(document::Minted {
                token: Secret::new(Box::from(b"artificial-token".as_slice())),
                expires: Wall::from_nanos(Duration::from_secs(expires).as_nanos()),
                permissions,
            }),
            &self.env,
            &mut self.up,
        );
        self.local();
    }
    fn tokens(&mut self) -> u32 {
        let mut count = 0_u32;
        while let Some(event) = self.up.pop() {
            if let Event::GitToken { .. } = event {
                count = count.checked_add(1).expect("fixture token count");
            }
        }
        count
    }
    fn connected(&mut self) {
        let owner = self.component.forges.get(0).expect("forge").base.own.expect("own connection");
        self.component.up(
            &self.env,
            skein_io::Event::Connecting { owner, socket: Token::new(100) },
            &mut self.up,
            &mut self.io,
        );
        self.component.up(&self.env, skein_io::Event::Connected { owner }, &mut self.up, &mut self.io);
    }
    fn forgejo_ready() -> Self {
        let mut h = Self::new(fixtures::settings());
        h.component.forges.get_mut(0).expect("forge").base.start = Start::Ready { cap: 2 };
        let owner = h.component.test_open_own(&h.env, &mut h.io);
        let id = *h.component.common.connections.get(&owner).expect("connection");
        h.component.common.connection_slab.get_mut(id).expect("connection").link.connecting(Token::new(100), &mut h.io);
        h.component.common.connection_slab.get_mut(id).expect("connection").finish(
            &h.env,
            skein_http::client::Reuse::Keep,
            false,
        );
        h
    }
}
fn github_ready() -> StepFixture {
    let mut h = StepFixture::new(fixtures::github_settings());
    let gh = h.component.test_github_mut();
    gh.accept(&[1]);
    gh.begin_read(h.env.limits.start_backoff);
    h.component.test_paged(
        Box::from([document::Installation {
            id: 1,
            account: 22,
            login: Box::from(b"acme".as_slice()),
            kind: document::Account::Organization,
            created: Wall::from_nanos(1_000_000_000),
            suspended: false,
        }]),
        false,
        &h.env,
        &mut h.up,
    );
    h.local();
    h.minted(3600, document::Levels::NONE);
    h.minted(4200, document::Levels::NONE);
    h.component.test_identified(
        document::Viewer { id: 9, login: Box::from(b"test-app[bot]".as_slice()) },
        &h.env,
        &mut h.up,
    );
    h.local();
    h.env.now = h.env.now.saturating_add(h.env.limits.idle);
    h.env.wall = Wall::from_nanos(h.env.now.as_nanos());
    h
}
fn renewing() -> StepFixture {
    let mut h = github_ready();
    h.env.now = Time::from_nanos(Duration::from_secs(3540).as_nanos());
    h.env.wall = Wall::from_nanos(h.env.now.as_nanos());
    h.component.test_github_mut().fire(h.env.now);
    h.local();
    h
}
fn renewed(h: &mut StepFixture) {
    h.minted(10800, document::Levels::NONE);
}
fn contents_renewing() -> StepFixture {
    let mut h = renewing();
    renewed(&mut h);
    h.env.now = Time::from_nanos(Duration::from_secs(4140).as_nanos());
    h.env.wall = Wall::from_nanos(h.env.now.as_nanos());
    h.component.test_github_mut().fire(h.env.now);
    h.local();
    h
}

fn refuse(h: &mut StepFixture, revision: u64) {
    let ForgeEntry { base, dialect } = h.component.forges.get_mut(0).expect("forge");
    match dialect {
        Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
        Dialect::GitHub(gh) => h.component.common.github_token_refused(base, gh, 1, revision, &h.env),
    }
}
fn revision(h: &StepFixture) -> u64 {
    h.component.test_github().rows.get(&1).expect("row").token_revision
}
#[test]
fn call_401_coalesces_one_renewal() {
    let mut h = github_ready();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(row.full.pending && row.contents.pending);
    h.local();

    renewed(&mut h);
    let gh = h.component.test_github();
    assert_eq!(gh.mint, Some((1, skein_forge::github::request::Scope::Contents)));
    let row = gh.rows.get(&1).expect("row");
    assert!(!row.full.pending && !row.contents.pending);
    renewed(&mut h);
    h.local();
    assert_eq!(h.tokens(), 2);
    match h.component.test_github().work {
        crate::installations::Work::Idle => {}
        crate::installations::Work::Minting { .. }
        | crate::installations::Work::Identity { .. }
        | crate::installations::Work::Waiting { .. } => unreachable!("coalesced scopes settled"),
    }
}
#[test]
fn call_401_coalesces_an_already_running_full_renewal() {
    let mut h = renewing();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    assert!(!h.component.test_github().rows.get(&1).expect("row").full.pending);
    renewed(&mut h);
    let gh = h.component.test_github();
    assert_eq!(gh.mint, Some((1, skein_forge::github::request::Scope::Contents)));
    assert!(!gh.rows.get(&1).expect("row").full.pending);
}
#[test]
fn stale_call_401_cannot_renew_a_replaced_full() {
    let mut h = renewing();
    let old = revision(&h);
    renewed(&mut h);
    refuse(&mut h, old);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(!row.full.pending && !row.contents.pending);
    assert_eq!(row.refused_revision, None);
}
#[test]
fn call_401_cannot_revive_gone() {
    let mut h = github_ready();
    let old = revision(&h);
    h.send(Request::Installations { forge: fixtures::FORGE, accepted: Box::from([]) });
    refuse(&mut h, old);
    let row = h.component.test_github().rows.get(&1).expect("Gone row");
    assert!(!row.full.pending && !row.contents.pending);
    assert_eq!(row.refused_revision, None);
}
#[test]
fn call_401_cannot_revive_a_replaced_connect() {
    let mut h = github_ready();
    let old = revision(&h);
    h.send(Request::Connect {
        forge: fixtures::FORGE,
        access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 7, 9),
    });
    refuse(&mut h, old);
    let row = h.component.test_github().rows.get(&1).expect("replacement row");
    assert!(!row.full.pending && !row.contents.pending);
    assert_eq!(row.refused_revision, None);
}
#[test]
fn call_401_during_backoff_keeps_one_retry() {
    let mut h = renewing();
    h.component.test_github_mut().status = Some(503);
    h.component.test_minted(Err(api::Error::Unavailable), &h.env, &mut h.up);
    let until = h.component.test_github().deadline();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    assert_eq!(h.component.test_github().deadline(), until);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(!row.full.pending && row.contents.pending);
}

#[test]
fn call_401_coalesces_an_already_running_contents_renewal() {
    let mut h = contents_renewing();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(row.full.pending);
    assert!(!row.contents.pending);
    renewed(&mut h);
    renewed(&mut h);
    h.local();
    assert_eq!(h.tokens(), 2);
    assert!(h.component.test_github().mint.is_none());
}

#[test]
fn renewal_local_signing_refusal_stops_invalid_key() {
    let mut h = github_ready();
    h.component.forges.get_mut(0).expect("forge").base.forge.access.as_mut().expect("access").credential =
        skein_forge::credential::Credential::App { id: 7, key: Secret::new(Box::from([0; 64])) };
    h.component.test_github_mut().jwt.forget();
    let revision = revision(&h);
    refuse(&mut h, revision);
    h.local();

    h.local();
    h.component.drive(&h.env, &mut h.up, &mut h.io);
    h.connected();
    let mut stopped = None;
    let mut connected = 0_u32;
    while let Some(event) = h.up.pop() {
        if let Event::Stopped { forge, why } = event {
            stopped = Some((forge, why));
        }
        if let Event::Connected { .. } = event {
            connected = connected.checked_add(1).expect("fixture connection count");
        }
    }
    assert_eq!(stopped, Some((fixtures::FORGE, crate::Stopped::Invalid { value: crate::Value::Key })));
    assert_eq!(connected, 1);
}

#[test]
fn a_key_that_parses_and_will_not_sign_stops_before_any_request() {
    let mut key = skein_lib::bytes::copy_of(skein_fake_forge_github::keys::KEYS.first().expect("public test key"));
    let at = skein_lib::bytes::find(&key, &[2, 3, 1, 0, 1]).expect("public exponent");
    *key.get_mut(at.checked_add(4).expect("exponent byte offset")).expect("exponent byte") = 3;
    let access = fixtures::app_access(&key, 7, 9);
    let mut h = StepFixture::new(fixtures::github_settings());
    assert_eq!(limits::access(&h.env.limits, crate::settings::Kind::GitHub, false, &access), Ok(()));
    h.send(Request::Connect { forge: fixtures::FORGE, access });
    h.component.drive(&h.env, &mut h.up, &mut h.io);
    h.connected();
    h.component.drive(&h.env, &mut h.up, &mut h.io);
    let mut stopped = None;
    while let Some(event) = h.up.pop() {
        if let Event::Stopped { forge, why } = event {
            stopped = Some((forge, why));
        }
    }
    assert_eq!(stopped, Some((fixtures::FORGE, crate::Stopped::Invalid { value: crate::Value::Key })));
    while let Some(request) = h.io.pop() {
        if let skein_io::Request::Stream { down: skein_lib::stream::Down::Send(_), .. } = request {
            panic!("invalid key wrote to the peer");
        }
    }
}

#[test]
fn an_unheld_failure_does_not_wake_a_closing_connection() {
    let mut h = StepFixture::forgejo_ready();
    h.component.test_waiting(
        Token::new(1),
        Waiting::new(
            fixtures::REPO,
            api::Op::Read(api::Read::Branch { branch: Box::from(fixtures::BRANCH) }),
            h.env.now.saturating_add(h.env.limits.deadline),
        ),
    );
    h.component.drive_calls(&h.env, &mut h.up, &mut h.io);
    let id = *h.component.common.calls.get(&Token::new(1)).expect("call");
    h.component.common.finish(Token::new(1), id, Ok(api::Answer::Commit(fixtures::BRANCH_HEAD)), &mut h.up);
    let Some(Event::Answered(_)) = h.up.pop() else { panic!("terminal answer") };
    let owner = *h.component.common.connections.iter().next_back().expect("idle call connection").0;
    assert!(!h.component.common.holders.contains_key(&owner));
    h.component.up(&h.env, skein_io::Event::Failed { owner, error: skein_io::Error::Reset }, &mut h.up, &mut h.io);
    assert_ne!(h.component.next_deadline(), Some(h.env.now));
}

impl Component {
    fn test_waiting(&mut self, call: Token, waiting: Waiting) {
        let id = self.common.waiting.insert(waiting).expect("fixture call room");
        let _old = self.common.calls.insert(call, id).expect("fixture call index");
    }
}

#[test]
fn a_call_of_the_other_dialect_is_unavailable() {
    const CALL: Token = Token::new(1);
    for (name, settings, side) in [
        (
            "GitHub call under Forgejo",
            fixtures::settings(),
            Side::GitHub(Box::new(GitHubCall {
                exchange: connection::github::Exchange::new(),
                plan: None,
                route: None,
                discovery: None,
                discovered: None,
            })),
        ),
        (
            "Forgejo call under GitHub",
            fixtures::github_settings(),
            Side::Forgejo(Box::new(ForgejoCall {
                exchange: connection::forgejo::Exchange::new(),
                plan: Plan::new(fixtures::REPO, api::Op::Read(api::Read::Settings)),
            })),
        ),
    ] {
        let limits = fixtures::limits();
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut c = Component::new(settings, &limits, &fixtures::shared()).expect("fixture limits");
        c.forges.get_mut(0).expect("forge").base.start = Start::Ready { cap: 2 };
        let mut waiting =
            Waiting::new(fixtures::REPO, api::Op::Read(api::Read::Settings), env.now.saturating_add(limits.deadline));
        waiting.side = side;
        c.test_waiting(CALL, waiting);
        let mut up = Queue::uncapped();
        let mut io = Queue::uncapped();
        c.drive_calls(&env, &mut up, &mut io);
        let Some(Event::Answered(answer)) = up.pop() else {
            panic!("{name}: terminal answer");
        };
        assert_eq!(
            answer,
            Answered { call: CALL, cost: 0, free: 0, rate: None, result: Err(api::Error::Unavailable) },
            "{name}",
        );
        c.drive_calls(&env, &mut up, &mut io);
        assert!(up.is_empty(), "{name}: only one answer");
        assert!(io.is_empty(), "{name}: no exchange begun");
    }
}

impl Component {
    fn test_open_own(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) -> Token {
        let ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        let role = match dialect {
            Dialect::Forgejo { .. } => Use::Own,
            Dialect::GitHub(_) => Use::Reserved,
        };
        self.common.open_own(base, role, env, io)
    }
}

impl Dialect {
    fn test_exchange_fresh(&self) -> bool {
        match self {
            Self::Forgejo { exchange, .. } => exchange.test_fresh(),
            Self::GitHub(gh) => gh.exchange.test_fresh(),
        }
    }
}

#[test]
fn an_own_exchange_goes_with_its_connection() {
    const CALL: Token = Token::new(1);
    #[derive(Clone, Copy, Debug)]
    enum End {
        Loss,
        Connect,
        Shed,
    }
    for end in [End::Loss, End::Connect, End::Shed] {
        for (name, settings) in [("Forgejo", fixtures::settings()), ("GitHub", fixtures::github_settings())] {
            let mut h = StepFixture::new(settings);
            h.component.begin(&h.env, &mut h.up, &mut h.io);
            h.connected();
            let entry = h.component.forges.get(0).expect("forge");
            let owner = entry.base.own.expect("own connection");
            assert!(!entry.dialect.test_exchange_fresh(), "{name} {end:?}: held before");
            match end {
                End::Loss => h.component.own_closed(owner, false, &h.env, &mut h.up),
                End::Connect => {
                    let access = entry.base.forge.access.clone().expect("fixture access");
                    h.component.connect(&h.env, fixtures::FORGE, access, &mut h.up, &mut h.io);
                }
                End::Shed => h.component.shed(),
            }
            assert!(h.component.forges.get(0).expect("forge").dialect.test_exchange_fresh(), "{name} {end:?}");
        }
    }
    // GitHub's reserved connection cannot be assigned to a call; Forgejo's idle own can.
    let mut h = StepFixture::forgejo_ready();
    let entry = h.component.forges.get_mut(0).expect("forge");
    let owner = entry.base.own.expect("idle own connection");
    match &mut entry.dialect {
        Dialect::Forgejo { exchange, .. } => exchange.test_hold_heard(),
        Dialect::GitHub(gh) => gh.exchange.test_hold_heard(),
    }
    assert!(!entry.dialect.test_exchange_fresh(), "assign: held before");
    h.component.test_waiting(
        CALL,
        Waiting::new(
            fixtures::REPO,
            api::Op::Read(api::Read::Settings),
            h.env.now.saturating_add(h.env.limits.deadline),
        ),
    );
    h.component.drive_calls(&h.env, &mut h.up, &mut h.io);
    let entry = h.component.forges.get(0).expect("forge");
    assert_eq!(entry.base.own, None, "assign: ownership taken");
    assert_eq!(h.component.common.holders.get(&owner), Some(&CALL), "assign: call holds connection");
    assert!(entry.dialect.test_exchange_fresh(), "assign");

    let mut github = github_ready();
    github.send(Request::Call { call: CALL, repository: fixtures::REPO, op: api::Op::Read(api::Read::Settings) });
    // A queued terminal answer precedes the transport ending. The retired call stays observable until reclaim.
    for (name, mut h) in [("Forgejo", h), ("GitHub", github)] {
        let id = *h.component.common.calls.get(&CALL).expect("active call");
        let waiting = h.component.common.waiting.get_mut(id).expect("call");
        let owner = waiting.connection.expect("call connection");
        match &mut waiting.side {
            Side::Forgejo(call) => call.exchange.test_hold_heard(),
            Side::GitHub(call) => call.exchange.test_hold_heard(),
            Side::Unbegun => unreachable!("active exchange"),
        }
        let connection = *h.component.common.connections.get(&owner).expect("connection");
        h.component.common.connection_slab.get_mut(connection).expect("connection").failed(connection::Failure::Seen);
        h.component.drive_calls(&h.env, &mut h.up, &mut h.io);
        let fresh = match &h.component.common.waiting.get(id).expect("retired call before reclaim").side {
            Side::Forgejo(call) => call.exchange.test_fresh(),
            Side::GitHub(call) => call.exchange.test_fresh(),
            Side::Unbegun => unreachable!("retired exchange"),
        };
        assert!(fresh, "{name}: closed call exchange releases before reclaim");
    }
}
