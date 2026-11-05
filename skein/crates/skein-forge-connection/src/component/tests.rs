use super::*;
use crate::tests::fixtures;

// forge-connection.md 6.1 reserves the listener's owner prefix.
const LISTENER_OWNER: u64 = 1_u64 << 59_u32;
const LAST_API_OWNER: u64 = LISTENER_OWNER - 1;
fn token_fixture() -> (Component, Env<Limits>, Queue<skein_io::Request>) {
    let limits = fixtures::limits();
    let c = Component::new(fixtures::settings(), &limits, &fixtures::shared()).expect("valid");
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
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
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut c = Component::new(settings, &limits, &fixtures::shared()).expect("fixture limits");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    c.begin(&env, &mut up, &mut io);
    assert_eq!(c.common.call_connections(), limits.connections);
    assert!(c.forges.get(GITHUB_AT).expect("forge").base.own.is_some());
    assert_eq!(c.common.connections.len(), 3);
}

use crate::tests::github_renewal::{ready as github_ready, renewed_contents, renewed_full, renewing};

fn refuse(h: &mut fixtures::Harness, revision: u64) {
    let ForgeEntry { base, dialect } = h.component.forges.get_mut(0).expect("forge");
    match dialect {
        Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
        Dialect::GitHub(gh) => h.component.common.github_token_refused(base, gh, 1, revision, &h.env),
    }
}
fn revision(h: &fixtures::Harness) -> u64 {
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
    h.run(skein_lib::Duration::ZERO);
    h.connected(fixtures::FORGE);
    renewed_full(&mut h);
    let gh = h.component.test_github();
    assert_eq!(gh.mint, Some((1, skein_forge::github::request::Scope::Contents)));
    let row = gh.rows.get(&1).expect("row");
    assert!(!row.full.pending && !row.contents.pending);
    renewed_contents(&mut h);
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.git_tokens.as_ref().expect("token history").len(), 2);
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
    renewed_full(&mut h);
    let gh = h.component.test_github();
    assert_eq!(gh.mint, Some((1, skein_forge::github::request::Scope::Contents)));
    assert!(!gh.rows.get(&1).expect("row").full.pending);
}
#[test]
fn stale_call_401_cannot_renew_a_replaced_full() {
    let mut h = renewing();
    let old = revision(&h);
    renewed_full(&mut h);
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
    h.answer(fixtures::FORGE, &fixtures::status(503));
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
    let mut h = crate::tests::github_renewal::contents_renewing();
    let revision = revision(&h);
    refuse(&mut h, revision);
    refuse(&mut h, revision);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert!(row.full.pending);
    assert!(!row.contents.pending);
    renewed_contents(&mut h);
    renewed_full(&mut h);
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.git_tokens.as_ref().expect("token history").len(), 2);
    assert!(h.component.test_github().mint.is_none());
}

#[test]
fn renewal_local_signing_refusal_stops_invalid_key() {
    let mut h = github_ready();
    h.component.forges.get_mut(0).expect("forge").base.forge.access.as_mut().expect("access").credential =
        skein_forge::credential::Credential::App { id: 7, key: skein_lib::Secret::new(Box::from([0; 64])) };
    h.component.test_github_mut().jwt.forget();
    let revision = revision(&h);
    refuse(&mut h, revision);
    h.run(skein_lib::Duration::ZERO);
    h.connected(fixtures::FORGE);
    h.run(skein_lib::Duration::ZERO);
    assert_eq!(h.stopped(), Some((fixtures::FORGE, crate::Stopped::Invalid { value: crate::Value::Key })));
    assert_eq!(h.connected_count, 1);
}

#[test]
fn an_unheld_failure_does_not_wake_a_closing_connection() {
    let mut h = fixtures::Harness::ready();
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Branch { branch: Box::from(fixtures::BRANCH) }));
    h.run(skein_lib::Duration::ZERO);
    h.connected(fixtures::FORGE);
    fixtures::branch_answers(&mut h, fixtures::FORGE);
    assert!(h.answered(1).is_some());
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
        let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
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
    pub(crate) fn test_calls_ready(&mut self) {
        self.forges.get_mut(0).expect("forge").base.start = Start::Ready { cap: skein_forge::github::pin::PAGE_CAP };
        self.forges.get_mut(0).expect("forge").base.forge.access =
            Some(fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
        let gh = self.test_github_mut();
        gh.connected = true;
        gh.identity_checked = true;
        gh.baseline = false;
    }
    pub(crate) fn test_github(&self) -> &crate::installations::GitHub {
        &self.test_github_forge().state
    }
    pub(crate) fn test_github_mut(&mut self) -> &mut crate::installations::GitHub {
        &mut self.test_github_forge_mut().state
    }
    pub(crate) fn test_local(&mut self, env: &Env<Limits>, up: &mut Queue<Event>) {
        let before = up.len();
        let ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        match dialect {
            Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            Dialect::GitHub(gh) => {
                for _ in 0..64_u32 {
                    if !self.common.github_local(base, gh, env, up) || up.len() != before {
                        break;
                    }
                }
            }
        }
    }
}

impl Component {
    pub(crate) fn test_github_handoffs(&self) -> u32 {
        let mut count = 0_u32;
        for entry in &self.forges {
            match &entry.dialect {
                Dialect::GitHub(gh) => {
                    if gh.exchange.test_github_handoff() {
                        count = count.saturating_add(1);
                    }
                }
                Dialect::Forgejo { .. } => {}
            }
        }
        for (_, id) in &self.common.calls {
            match &self.common.waiting.get(*id).expect("indexed call").side {
                Side::GitHub(call) => {
                    if call.exchange.test_github_handoff() {
                        count = count.saturating_add(1);
                    }
                }
                Side::Unbegun | Side::Forgejo(_) => {}
            }
        }
        count
    }
}

impl Component {
    fn test_github_forge(&self) -> &GitHubForge {
        match &self.forges.get(0).expect("forge").dialect {
            Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            Dialect::GitHub(gh) => gh,
        }
    }
    fn test_github_forge_mut(&mut self) -> &mut GitHubForge {
        match &mut self.forges.get_mut(0).expect("forge").dialect {
            Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            Dialect::GitHub(gh) => gh,
        }
    }
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
            let mut h = fixtures::Harness::new(settings);
            h.connected(fixtures::FORGE);
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
    let mut h = fixtures::Harness::ready();
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

    let mut github = fixtures::github_calls();
    github.call(CALL.raw(), fixtures::REPO, api::Op::Read(api::Read::Settings));
    github.run(skein_lib::Duration::ZERO);
    github.connected(fixtures::FORGE);
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

impl Component {
    pub(crate) fn test_last_connection(&mut self) -> &mut Connection {
        let (_, id) = self.common.connections.last().expect("fixture connection");
        self.common.connection_slab.get_mut(*id).expect("indexed connection")
    }
    pub(crate) fn test_open_call(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>) {
        let forge = &self.forges.get(0).expect("fixture forge").base.forge;
        self.common.open_call_origin(forge, None, env, io);
    }
    pub(crate) fn test_start_user(&mut self, env: &Env<Limits>, up: &mut Queue<Event>) {
        let base = &mut self.forges.get_mut(0).expect("fixture forge").base;
        self.common.start_heard(base, StartHeard::User { id: fixtures::WRITER }, env, up);
    }
}
