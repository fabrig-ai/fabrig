//! Fake peer settings (temper/docs/design/playground.md, 4).
use crate::documents;
use skein_lib::Duration;
mod llm;
/// Synthetic admin token.
pub const ADMIN_TOKEN: &[u8] = b"restart-admin-token";
/// Synthetic api token.
pub const API_TOKEN: &[u8] = b"0000000000000000000000000000000000000002";
/// Synthetic git token.
pub const GIT_TOKEN: &[u8] = b"0000000000000000000000000000000000000003";
const TOKENS: u32 = 5;
/// The managed forge's table.
#[must_use]
pub fn table() -> skein_fake_forge_protocol::Config {
    let mut table = documents::hosted_git::table(b"acme/app", None);
    table.users.get_mut(0).expect("bot user").1 = b"temper".as_slice().into();
    let mut tokens = skein_lib::List::with_capacity(TOKENS);
    for token in table.tokens.into_boxed() {
        tokens.push(token).expect("fixture tokens");
    }
    for (token, user) in [(ADMIN_TOKEN, 2), (API_TOKEN, 1), (GIT_TOKEN, 1)] {
        tokens.push((skein_lib::Secret::new(token.into()), user)).expect("restart tokens");
    }
    table.tokens = tokens;
    table
}
/// Seed the managed forge's worker, collaborator and administrator.
pub fn setup_peer(peer: &mut skein_fake_peers::forge::Peer) {
    let setup = peer.setup_mut();
    let worker = setup.account(b"temper", false, b"temper:bot");
    assert_eq!(worker, documents::WORKER_USER);
    setup
        .grant(documents::REPOSITORY, b"temper", skein_forge::forgejo::request::Grant::Write)
        .expect("fixture collaborator");
    let admin = setup.account(b"stranger", true, b"Restart site admin");
    setup.token(
        admin,
        b"admin",
        &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
        skein_lib::Secret::new(ADMIN_TOKEN.into()),
    );
}
/// The composition's default Forgejo peer, sharing `forge` with the git transport.
#[must_use]
pub fn forge_peer(forge: &temper_fake_git::Typed, seed: u64) -> skein_fake_peers::forge::Peer {
    let mut peer = skein_fake_peers::forge::Peer::new(
        documents::FORGE,
        peer_limits(),
        forge_table(),
        forge_limits(),
        forge.forge.clone(),
        seed,
    )
    .expect("forge peer");
    setup_peer(&mut peer);
    peer
}
/// The composition's forge API table.
#[must_use]
pub fn forge_table() -> skein_fake_forge_protocol::Config {
    table()
}
/// The forge's peer limits.
#[must_use]
pub fn peer_limits() -> skein_fake_peers::Limits {
    skein_fake_peers::Limits {
        io: skein_io::Limits {
            sockets: 16,
            names: 0,
            refusals: 8,
            intake: 65_536,
            receive: 1024,
            output: 65_536,
            sends: 8,
            accepts: 4,
            backlog: 8,
            close_timeout: Duration::from_secs(1),
            retry: Duration::from_millis(1),
        },
        connections: 4,
        queue: 64,
        plaintext: 32_768,
        ciphertext: 32_768,
        observations: STORY_OBSERVATIONS,
        observation_bytes: STORY_OBSERVATION_BYTES,
    }
}
/// The forge's HTTP and document limits.
#[must_use]
pub fn forge_limits() -> skein_fake_forge_protocol::Limits {
    skein_fake_forge_protocol::Limits {
        calls: 4,
        connections: 4,
        raw_bytes: 4096,
        http: skein_http::server::Limits {
            head: 4096,
            headers: 16,
            body: 25_165_824,
            read: 1024,
            response: 4096,
            send: 1024,
        },
        documents: skein_forge::forgejo::limits::Limits {
            json: skein_json::tokenizer::Limits {
                chunk: 1024,
                string: 25_165_824,
                number: 32,
                depth: 16,
                length: 25_165_824,
            },
            writer: skein_json::writer::Limits { depth: 16, length: 25_165_824 },
            string_bytes: 512,
            body_bytes: 25_165_824,
            inner: 256,
            target_bytes: 4096,
        },
    }
}
const STORY_OBSERVATIONS: u32 = 1_024;
const STORY_OBSERVATION_BYTES: u32 = 2_097_152;
/// The scripted provider with caller-owned scripts and seed.
#[must_use]
pub fn scripted_provider(scripts: Box<[skein_fake_llm_domain::api::Script]>, seed: u64) -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_protocol::{documents, provider};
    let config = llm_config();
    let protocol = llm_limits();
    skein_fake_peers::llm::Peer::new(
        (std::net::Ipv4Addr::LOCALHOST, 443).into(),
        skein_fake_peers::Transport::Plaintext,
        llm_peer_limits(),
        provider::Config {
            echo: skein_llm::openai::Echo::NONE,
            usage_fields: documents::UsageFields::ALL,
            provider: documents::Provider::OpenAi,
            path: skein_llm::Endpoint::openai().target,
            headers: Box::new([]),
            without_credential: false,
        },
        skein_llm::Credential::key(crate::documents::KEY.into()),
        protocol,
        skein_fake_llm_domain::Domain::try_scripted(&config, seed, scripts).expect("bounded provider scripts"),
        config,
    )
    .expect("configured provider")
}
pub use llm::{llm_config, llm_limits, llm_peer_limits};
