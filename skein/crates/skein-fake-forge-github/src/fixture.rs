//! Shared scenario values (skein/docs/design/testing/fakes/github.md, 1.6).
use crate::{
    app::App,
    installations::{self, Kind, Reach},
    service::{self, Account, Config, Limits, Row, Service},
};
use alloc::boxed::Box;
use skein_fake_forge_domain::{self as fake, api as a};
use skein_forge::github;
use skein_json::{tokenizer, writer};
use skein_lib::{Duration, List, Secret, Wall, bytes};

pub const APP: u64 = 7;
pub const BOT: u64 = 9;
pub const OWNER: u64 = 22;
pub const PERSON: u64 = 31;
pub const PATH: &[u8] = b"acme/widgets";
#[must_use]
pub fn key() -> &'static [u8] {
    crate::keys::KEYS.first().expect("first test key")
}
#[must_use]
pub fn app() -> App {
    App {
        id: APP,
        slug: bytes::copy_of(b"test-app"),
        owner: PERSON,
        permissions: Box::new([]),
        events: Box::new([]),
        key: key(),
        webhook: Secret::new(bytes::copy_of(b"test-webhook")),
    }
}
#[must_use]
pub fn accounts() -> List<Account> {
    let mut rows = List::with_capacity(2);
    for (login, id, organization) in [(b"acme".as_slice(), OWNER, true), (b"alice", PERSON, false)] {
        rows.push(Account { login: bytes::copy_of(login), id, organization }).expect("fixture accounts");
    }
    rows
}
#[must_use]
pub fn repositories() -> List<Row> {
    let mut rows = List::with_capacity(3);
    for (id, name, organization, private) in
        [(4, b"tools".as_slice(), false, false), (2, b"widgets", true, true), (6, b"absent", true, false)]
    {
        rows.push(row(id, name, organization, private)).expect("fixture repositories");
    }
    rows
}
#[must_use]
pub fn widgets(id: u32) -> List<Row> {
    let mut rows = List::with_capacity(3);
    rows.push(row(id, b"widgets", true, true)).expect("fixture repository");
    rows
}
fn row(id: u32, name: &[u8], organization: bool, private: bool) -> Row {
    Row { id, owner: bytes::copy_of(b"acme"), owner_id: OWNER, name: bytes::copy_of(name), organization, private }
}
#[must_use]
pub fn config(repositories: List<Row>) -> Config {
    Config {
        rate_quota: 5000,
        rate_window: Duration::from_secs(3600),
        token_life: installations::TOKEN_LIFE,
        deliveries: crate::delivery::Config::standard(),
        base: bytes::copy_of(b"http://127.0.0.1:3000"),
        blob: bytes::copy_of(b"http://logs.example.test"),
        third_origin: None,
        blob_redirect: None,
        app: app(),
        app_facts: service::AppFacts::Complete,
        shown: service::Shown::Ruleset,
        ruleset_id: 17,
        pull_requests_only: false,
        required_signatures: false,
        tokens: List::with_capacity(0),
        faults: crate::Faults { reset: 0, drop: 0, server_error: 0 },
        bot: BOT,
        hook: bytes::copy_of(b"https://hook.test/start"),
        repositories,
        accounts: accounts(),
        person: bytes::copy_of(b"alice"),
    }
}
#[must_use]
pub fn limits() -> Limits {
    Limits {
        deliveries: crate::delivery::Limits { entries: 64, payload_bytes: 32768 },
        http: skein_http::server::Limits {
            head: 4096,
            headers: 16,
            body: 4096,
            read: 1024,
            response: 16384,
            send: 1024,
        },
        github: github::limits::Limits {
            json: tokenizer::Limits { depth: 10, string: 4096, number: 32, chunk: 128, length: 8192 },
            writer: writer::Limits { depth: 10, length: 8192 },
            string_bytes: 256,
            body_bytes: 4096,
            inner: u32::try_from(github::setup::EVENTS.len().max(github::setup::PERMISSIONS.len()))
                .expect("manifest lists fit"),
            target_bytes: 1024,
        },
        installations: installations::Limits { installations: 3, tokens: 12, listed: 2 },
        pages: crate::pages::Limits { states: 3, manifest_bytes: 2048 },
        page: 2,
        body: 4096,
        response: 8192,
        blob_records: 16,
        git_records: 16,
    }
}
#[must_use]
pub fn domain_config() -> fake::Config {
    fake::Config {
        limits: fake::Limits {
            repositories: 2,
            labels: 1,
            items: 1,
            comments: 1,
            dependencies: 1,
            reviews: 1,
            commits: 8,
            files: 1,
            statuses: 1,
            contexts: 1,
            body_bytes: 128,
            calls: 2,
            hooks: 4,
            ..fake::Limits::TEST
        },
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        late_min: Duration::ZERO,
        late_max: Duration::ZERO,
        land_min: Duration::ZERO,
        land_max: Duration::ZERO,
        skew: fake::Skew::None,
        ..fake::Config::test()
    }
}

pub fn install_one(service: &mut Service, at: Wall) {
    assert_eq!(
        service.installations_mut().install(OWNER, b"acme", Kind::Organization, Reach::All, at),
        Ok(1),
        "first installation"
    );
}
pub fn install_three(service: &mut Service) {
    let mut listed = List::with_capacity(2);
    for id in [2, 4] {
        listed.push(id).expect("fixture reach");
    }
    let store = service.installations_mut();
    assert_eq!(
        store.install(OWNER, b"acme", Kind::Organization, Reach::Listed(listed), Wall::from_nanos(1_000_000_000_000)),
        Ok(1),
        "first installation"
    );
    assert_eq!(
        store.install(23, b"example", Kind::User, Reach::All, Wall::from_nanos(1_001_000_000_000)),
        Ok(2),
        "second installation"
    );
    assert_eq!(
        store.install(24, b"other", Kind::User, Reach::All, Wall::from_nanos(1_002_000_000_000)),
        Ok(3),
        "third installation"
    );
    assert!(store.suspend(3, true), "third installation exists");
}
#[must_use]
pub fn setup(id: u32, name: &[u8]) -> a::Setup {
    a::Setup {
        id,
        name: bytes::copy_of(name),
        default: bytes::copy_of(b"main"),
        workflows: Box::new([]),
        tree: Box::new([]),
        labels: Box::new([]),
        protection: None,
        hooked: false,
        checks: a::Checks {
            cues: Box::new([]),
            contexts: Box::new([]),
            latency_min: Duration::ZERO,
            latency_max: Duration::ZERO,
            silent: 0,
            passes: 1000,
            reruns: 0,
            cue: None,
        },
    }
}
