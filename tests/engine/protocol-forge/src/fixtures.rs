//! The component's small limits and configured forge.
use skein_forge_connection::Limits;
use skein_lib::Duration;
pub const WRITER: u64 = 2;
#[must_use]
pub fn limits() -> Limits {
    Limits {
        forges: 2,
        token_bytes: 64,
        key_bytes: u32::try_from(skein_tls::RS256_KEY_BYTES).expect("RS256 key bound fits u32"),
        installations: 8,
        log_origins: 1,
        log_poll: Duration::from_secs(5),
        log_catch_up: Duration::from_secs(86_400),
        log_pages: 10,
        delivery_bytes: 1_048_576,
        renew_before: Duration::from_secs(10 * 60),
        connections: 2,
        head_bytes: 4096,
        headers: 32,
        body_bytes_in: 1_048_576,
        depth: 16,
        chunk: 1024,
        string_bytes: 128,
        body_bytes: 1045,
        inner: 8,
        target_bytes: 1024,
        repositories: 2,
        users: 8,
        deadline: Duration::from_secs(10),
        idle: Duration::from_secs(5),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(8),
        hooks: 4,
        hook_head_bytes: 8192,
        hook_body_bytes: 1_048_576,
        hook_deadline: Duration::from_secs(10),
        hook_secret_bytes: 128,
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
    }
}

use core::net::{Ipv4Addr, SocketAddr};
use skein_forge_connection::{
    Settings, Shared,
    settings::{Forge, Kind, Transport},
};
use skein_lib::{List, Secret};
pub const FORGE: u16 = 1;
pub const TOKEN: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
#[must_use]
pub fn shared() -> Shared {
    Shared {
        calls: 2,
        rows: 8,
        answer_bytes: 65_536,
        op_bytes: 512,
        lifetime: Duration::from_secs(10),
        name_bytes: 128,
        repositories: 2,
        io: skein_io::Limits {
            sockets: 8,
            names: 0,
            refusals: 2,
            intake: 32_768,
            receive: 4096,
            output: 32_768,
            sends: 4,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(1),
            retry: Duration::from_millis(10),
        },
    }
}
#[must_use]
pub fn settings() -> Settings {
    let mut forges = List::with_capacity(2);
    forges
        .push(Forge {
            number: FORGE,
            kind: Kind::Forgejo,
            address: skein_io::Addr::Ip(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
            transport: Transport::Plaintext { name: None },
            log_origins: List::with_capacity(0),
            access: Some(skein_forge_connection::Access {
                credential: skein_forge::credential::Credential::Token(Secret::new(Box::from(TOKEN))),
                user: WRITER,
                webhook_secret: None,
            }),
        })
        .expect("one forge");
    Settings {
        forges,
        webhooks: None,
        word: Box::from(b"temper".as_slice()),
        user_agent: Box::from(b"temper/0.0.0".as_slice()),
    }
}
/// The token's writer is user 1; the scenario's repository is id 2.
#[must_use]
pub fn protocol_config() -> skein_fake_forge_protocol::Config {
    use skein_fake_forge_protocol::{Config, Faults, Repository};
    let mut tokens = List::with_capacity(1);
    tokens.push((Secret::new(Box::from(TOKEN)), 1)).expect("one token");
    let mut users = List::with_capacity(2);
    users.push((1, Box::from(b"engine".as_slice()))).expect("user");
    users.push((2, Box::from(b"person".as_slice()))).expect("user");
    let mut repositories = List::with_capacity(1);
    repositories.push(Repository { id: 2, full: Box::from(b"org/repo".as_slice()), protected: None }).expect("repo");
    Config {
        base: Box::from(b"http://127.0.0.1:3000".as_slice()),
        tokens,
        users,
        repositories,
        max_response_items: 2,
        default_paging_num: 30,
        hooks: None,
        faults: Faults { reset: 0, drop: 0, server_error: 0 },
    }
}

/// The connector world's writer, matching the fake's API token.
#[must_use]
pub fn writer_settings() -> Settings {
    let mut settings = settings();
    settings.forges.get_mut(0).expect("forge").access.as_mut().expect("access").user = 1;
    settings
}
/// Component bounds use exactly the top's client limits.
#[must_use]
pub fn protocol_shared() -> Shared {
    protocol_shared_for(&temper_engine_forge_world::LIMITS.client)
}
/// Component bounds for a world's explicit client limits.
#[must_use]
pub fn protocol_shared_for(client: &skein_forge_client::Limits) -> Shared {
    Shared {
        calls: client.calls,
        rows: client.rows,
        answer_bytes: client.answer_bytes,
        op_bytes: client.op_bytes,
        lifetime: client.lifetime,
        ..shared()
    }
}
#[must_use]
pub fn goal_view() -> temper_engine_domain_forge_issues::GoalView {
    use temper_engine_domain_forge_issues::{GoalView, Phase};
    GoalView {
        phase: Phase::Waiting,
        goal: 42,
        repository: 2,
        title: Box::from("Goal"),
        goal_text: Box::from("Plan"),
        plan: Box::new([]),
        milestones: Box::new([]),
        finished: None,
    }
}

/// Both byte-valued records' Debug and text-valued facts must hide the API token.
#[must_use]
pub fn token_absent(value: &impl core::fmt::Debug) -> bool {
    let text = format!("{value:?}");
    let bytes = format!("{TOKEN:?}");
    !text.as_bytes().windows(TOKEN.len()).any(|part| part == TOKEN) && !text.contains(&bytes[1..bytes.len() - 1])
}

/// The machine world's fake App and bot, supplied by temper's own fixture.
#[must_use]
pub fn github_settings() -> Settings {
    let mut settings = settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.kind = Kind::GitHub;
    forge.access = Some(skein_forge_connection::Access {
        credential: skein_forge::credential::Credential::App {
            id: 7,
            key: Secret::new(Box::from(skein_fake_forge_github::keys::KEYS[0])),
        },
        user: 9,
        webhook_secret: None,
    });
    settings
}

/// The joint's repository path and API/git writer for each dialect.
#[derive(Clone, Copy)]
pub struct Fixture {
    pub kind: Kind,
    pub owner: &'static [u8],
    pub name: &'static [u8],
    pub path: &'static [u8],
    pub writer: u64,
}
pub const FORGEJO: Fixture =
    Fixture { kind: Kind::Forgejo, owner: b"org", name: b"repo", path: b"org/repo", writer: 1 };
pub const GITHUB: Fixture =
    Fixture { kind: Kind::GitHub, owner: b"acme", name: b"widgets", path: b"acme/widgets", writer: 9 };
impl Fixture {
    #[must_use]
    pub fn settings(self) -> Settings {
        match self.kind {
            Kind::Forgejo => writer_settings(),
            Kind::GitHub => github_settings(),
        }
    }
}
