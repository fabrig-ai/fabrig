//! The machine world's GitHub peer (skein/docs/design/fake-forge.md, 10.6).
use skein_fake_forge_domain::{self as fake, api as a};
use skein_fake_forge_github::{
    app::App,
    fixture,
    inline::{self, Inline},
    installations::Installations,
    manifest::Manifest,
    service::{AppFacts, Config, Service},
};
use skein_forge::github;
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};

/// A wall past the fake's installation dates, advanced by the machine's clock.
pub const WALL_BASE: Wall = Wall::from_nanos(1_000_000_000_000_000);

/// The blob host is resolved to a loopback destination by this in-memory world.
pub const BLOB_ADDRESS: core::net::SocketAddr =
    core::net::SocketAddr::new(core::net::IpAddr::V4(core::net::Ipv4Addr::LOCALHOST), 3001);
pub const BLOB_ORIGIN: &[u8] = b"http://logs.example.test:3001";

struct LogScenario<'a> {
    third_origin: Option<&'a [u8]>,
    blob_redirect: Option<&'a [u8]>,
}

/// The GitHub service's state behind one machine's scripted peer.
pub struct Fake {
    inline: Inline,
    statuses: Vec<u16>,
}
impl Fake {
    /// Uses the GitHub fake's fixture App, accounts, repositories and installations.
    /// The App holds the grants setup asks for: its bot's permission is its
    /// installation's (forge-github.md, 4.1, 8.2).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::scenario(seed, AppFacts::Complete, true, b"https://hook.test/start")
    }

    /// The fixture App without grants or events: setup's checks of a lacking App
    /// (forge-setup.md, 5.6). Its bot's repository permission is none.
    #[must_use]
    pub fn bare(seed: u64) -> Self {
        Self::scenario(seed, AppFacts::Complete, false, b"https://hook.test/start")
    }

    /// Scenario-only App metadata and hook, using the dialect's public configuration.
    #[must_use]
    pub fn scenario(seed: u64, facts: AppFacts, complete: bool, hook: &[u8]) -> Self {
        Self::configured(seed, facts, complete, hook, None)
    }

    /// A failing CI job and the fake's signed-log scenarios.
    #[must_use]
    pub fn job_logs(seed: u64, third_origin: Option<&[u8]>, blob_redirect: Option<&[u8]>) -> Self {
        Self::configured(
            seed,
            AppFacts::Complete,
            true,
            b"https://hook.test/start",
            Some(LogScenario { third_origin, blob_redirect }),
        )
    }

    fn configured(seed: u64, facts: AppFacts, complete: bool, hook: &[u8], logs: Option<LogScenario<'_>>) -> Self {
        let has_logs = logs.is_some();
        let LogScenario { third_origin, blob_redirect } =
            logs.unwrap_or(LogScenario { third_origin: None, blob_redirect: None });
        let permissions = if complete { skein_fake_forge_github::app::wanted() } else { Box::from([]) };
        let events = if complete {
            github::setup::EVENTS.iter().map(|name| Box::from(*name)).collect::<Vec<_>>().into_boxed_slice()
        } else {
            Box::from([])
        };
        let config = Config {
            blob: Box::from(BLOB_ORIGIN),
            third_origin: third_origin.map(Box::from),
            blob_redirect: blob_redirect.map(Box::from),
            app: App { permissions, events, ..fixture::app() },
            app_facts: facts,
            hook: Box::from(hook),
            ..fixture::config(fixture::repositories())
        };
        let limits = fixture::limits();
        let mut service = Service::new(config, &limits, seed).expect("valid fixture config");
        fixture::install_three(&mut service);
        let (domain, config) = fixture_domain(seed, has_logs);
        Self { inline: Inline::new(service, domain, config, limits), statuses: Vec::new() }
    }

    pub(crate) fn into_service(self) -> Service {
        self.inline.service
    }

    /// Scenario access to the fake's installation store.
    pub fn installations_mut(&mut self) -> &mut Installations {
        self.inline.service.installations_mut()
    }

    /// Statuses of the fake's HTTP responses, in request order.
    #[must_use]
    pub fn statuses(&self) -> &[u16] {
        &self.statuses
    }

    /// Scenario releases use the fake's configured visibility delay and failed endpoint.
    pub fn release(&mut self, payload: github::delivery::Delivery, wall: Wall) -> u64 {
        let (id, post) = self.inline.service.release(payload, Some(1), wall).expect("fixture delivery fits");
        assert!(post.is_none(), "no hook endpoint");
        id
    }

    /// A redelivery retains its guid and gets a newer log id.
    pub fn redeliver(&mut self, id: u64, wall: Wall) -> u64 {
        let (id, post) = self.inline.service.redeliver(id, wall).expect("retained delivery");
        assert!(post.is_none(), "no hook endpoint");
        id
    }

    /// Later App requests are refused by the fake.
    pub fn revoke_key(&mut self) {
        self.inline.service.revoke_key();
    }

    /// Scenario inspection without issuing an API request.
    #[must_use]
    pub fn installation_ids(&mut self) -> Vec<u64> {
        self.inline.service.installations_mut().iter().map(|row| row.id).collect()
    }

    /// Answers one complete HTTP/1.1 request, keeping the connection open.
    pub fn serve(&mut self, request: &[u8], wall: Wall) -> Vec<u8> {
        let response = self.inline.serve(request, wall);
        self.statuses
            .push(core::str::from_utf8(&response[9..12]).expect("HTTP status digits").parse().expect("HTTP status"));
        response.into_vec()
    }

    /// Blob requests go through the fake's signature check and header-presence record.
    pub fn blob(&mut self, bytes: &[u8], wall: Wall) -> Vec<u8> {
        let response = self.inline.blob(bytes, wall);
        self.statuses
            .push(core::str::from_utf8(&response[9..12]).expect("HTTP status digits").parse().expect("HTTP status"));
        response.into_vec()
    }

    #[must_use]
    pub fn blob_records(&self) -> &Queue<skein_fake_forge_github::logs::Record> {
        self.inline.service.blob_records()
    }

    /// Answers one complete browser request without the API's headers or credentials.
    pub fn page(&mut self, request: &[u8], wall: Wall) -> Vec<u8> {
        assert!(
            !request.split(|byte| *byte == b'\n').take_while(|line| *line != b"\r").any(|line| {
                line.split(|byte| *byte == b':').next().is_some_and(|name| name.eq_ignore_ascii_case(b"Authorization"))
            }),
            "browser carries no API credential"
        );
        let response = self.inline.serve(request, wall);
        response.into_vec()
    }

    /// The fake's posted manifest, readable after conversion until reclamation.
    #[must_use]
    pub fn manifest(&self, state: &[u8]) -> Option<&Manifest> {
        self.inline.service.manifest(state)
    }
}

/// The fake App's public test key.
#[must_use]
pub fn app_key() -> &'static [u8] {
    fixture::key()
}

/// Signs an App JWT and prefixes its whole Authorization value.
#[must_use]
pub fn bearer(key: &[u8], app: u64, issued: Wall, expires: Wall) -> Vec<u8> {
    let input = github::jwt::signing_input(app, issued, expires).expect("fixture JWT window");
    let signature = skein_tls::sign_rs256(key, &input).expect("test key signs");
    let token = github::jwt::token(&input, &signature).expect("fixture JWT");
    let mut header = b"Bearer ".to_vec();
    header.extend_from_slice(token.expose());
    header
}

fn fixture_domain(seed: u64, logs: bool) -> (fake::Domain, fake::Config) {
    let mut config = crate::fixtures::fake_config();
    config.limits.repositories = 3;
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    config.skew = fake::Skew::None;
    let mut domain = fake::Domain::new(&config, seed);
    for (id, name) in [(2, b"acme/widgets".as_slice()), (4, b"acme/tools"), (6, b"acme/absent")] {
        let first = fake::repository(
            &mut domain,
            &config,
            a::Setup {
                default: Box::from(if id == 4 { b"queue".as_slice() } else { b"main".as_slice() }),
                checks: fixture_checks(logs && id == 2),
                protection: (id == 4).then(|| a::Protection {
                    linear: false,
                    code_owners: false,
                    bypassing_users: Box::from([]),
                    queue: true,
                    branch: Box::from(b"queue".as_slice()),
                    contexts: Box::from([]),
                    approvals: 0,
                    dismiss_stale: false,
                }),
                ..fixture::setup(id, name)
            },
        );
        fake::grant(&mut domain, name, 9, a::Permission::Write);
        if id == 4 {
            merge_fixture(&mut domain, &config, first);
        }
        fake::settings(
            &mut domain,
            name,
            a::Settings { default: Box::from(b"main".as_slice()), merge: true, squash: true, rebase: false },
        );
        if id == 2 {
            widget_branches(&mut domain, &config, first);
            if logs {
                log_head(&mut domain, &config);
            }
        }
    }
    (domain, config)
}

fn widget_branches(domain: &mut fake::Domain, config: &fake::Config, first: u64) {
    for branch in [b"alpha".as_slice(), b"feature/one"] {
        fixture_call(
            domain,
            config,
            b"acme/widgets",
            a::Op::Write(a::Write::CreateBranch { branch: Box::from(branch), commit: first }),
        );
    }
}

fn fixture_checks(logs: bool) -> a::Checks {
    let mut checks = crate::joint::checks(if logs { 0 } else { 1000 });
    if logs {
        checks.cue = Some(a::Cue::PullFile {
            path: Box::from(b"changes/31.md".as_slice()),
            no_pull: a::Check::Failed,
            step: Box::from(b"changes".as_slice()),
            log: Box::from(b"Missing changes/31.md with the line: opened 2026-11-01T14:07:25Z".as_slice()),
        });
    } else {
        checks.contexts = Box::from([]);
    }
    checks
}

fn log_head(domain: &mut fake::Domain, config: &fake::Config) {
    let commit = fake::commit(
        domain,
        config,
        1,
        Box::new([a::File { path: Box::from(b"widget".as_slice()), content: Box::from(b"repair".as_slice()) }]),
        b"another failed head",
    )
    .expect("fixture commit")
    .expect("changed fixture");
    fixture_call(
        domain,
        config,
        b"acme/widgets",
        a::Op::Git(a::Git::Push { branch: Box::from(b"alpha".as_slice()), commit, expected: None }),
    );
}

// Tools has ordinary and queued pulls; widgets keeps the read fixtures' branches.
fn merge_fixture(domain: &mut fake::Domain, config: &fake::Config, first: u64) {
    let repository = b"acme/tools";
    fake::grant(domain, repository, 31, a::Permission::Read);
    let head = fake::commit(
        domain,
        config,
        first,
        Box::from([a::File { path: Box::from(b"change".as_slice()), content: Box::from(b"feature".as_slice()) }]),
        b"feature",
    )
    .expect("fixture commit")
    .expect("changed tree");
    for op in [
        a::Op::Write(a::Write::CreateBranch { branch: Box::from(b"main".as_slice()), commit: first }),
        a::Op::Git(a::Git::Push { branch: Box::from(b"feature".as_slice()), commit: head, expected: None }),
    ] {
        fixture_call(domain, config, repository, op);
    }
    for base in [b"main".as_slice(), b"queue"] {
        fixture_call(
            domain,
            config,
            repository,
            a::Op::Write(a::Write::OpenPull {
                title: Box::from(b"feature".as_slice()),
                body: Box::from(b"pull body".as_slice()),
                head: Box::from(b"feature".as_slice()),
                base: Box::from(base),
                opens_empty: false,
            }),
        );
    }
}
fn fixture_call(domain: &mut fake::Domain, config: &fake::Config, repository: &[u8], op: a::Op) {
    inline::call(
        domain,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *config },
        fake::Event::Call { reply_to: ReplyTo::new(Token::new(77)), user: 9, repository: Box::from(repository), op },
    )
    .expect("fixture call");
}
