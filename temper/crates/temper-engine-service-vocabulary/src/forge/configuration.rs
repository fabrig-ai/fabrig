//! Forge configuration shared by the startup and component (protocol/services.md, 4.2).
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, Segment, What};
use skein_lib::{Duration, List};
use temper_engine_domain_forge as forge;

/// The user agent every forge request carries; a test pins it.
pub const USER_AGENT: &str = "temper/0.0.0";
const WORD: &[u8] = b"temper";
/// A forge's endpoint and TLS transport, supplied by the shell; an unresolved host uses loopback port zero.
#[derive(Debug)]
pub struct ForgeRuntime {
    pub number: u16,
    pub address: core::net::SocketAddr,
    pub transport: skein_forge_connection::settings::Transport,
    /// The kind's default job-log origins, resolved by the shell before serving.
    pub log_origins: List<skein_forge_connection::settings::LogOrigin>,
}
/// Pair the configuration and shell endpoints by forge number.
pub fn settings(
    forges: &[settings::Forge],
    runtime: List<ForgeRuntime>,
    webhooks: Option<skein_forge_connection::hooks::Webhooks>,
    found: &mut Problems,
) -> Option<skein_forge_connection::settings::Settings> {
    let mut endpoints = List::with_capacity(runtime.len());
    for endpoint in runtime.into_boxed() {
        endpoints.push(Some(endpoint)).expect("runtime capacity");
    }
    let mut configured = List::with_capacity(u32::try_from(forges.len()).expect("configured forges"));
    for (index, forge) in forges.iter().enumerate() {
        let mut endpoint = None;
        for at in 0..endpoints.len() {
            let candidate = endpoints.get_mut(at).expect("runtime entry");
            if let Some(value) = candidate.as_ref()
                && value.number == forge.number
            {
                endpoint = candidate.take();
                break;
            }
        }
        let path = forge_path(u32::try_from(index).expect("forge index")).member(b"api");
        let Some(endpoint) = endpoint else {
            found.push(path, What::Refers);
            continue;
        };
        let name = match &forge.api {
            jig_protocol_config::Address::Host { name, .. } => Some(name.clone()),
            jig_protocol_config::Address::Addr(skein_io::Addr::Ip(_)) => None,
            jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => {
                found.push(path, What::Plaintext);
                continue;
            }
        };
        let transport = match endpoint.transport {
            skein_forge_connection::settings::Transport::Plaintext { .. } => {
                skein_forge_connection::settings::Transport::Plaintext { name }
            }
            transport @ skein_forge_connection::settings::Transport::Tls { .. } => transport,
        };
        configured
            .push(skein_forge_connection::settings::Forge {
                number: forge.number,
                kind: match forge.kind {
                    settings::Kind::Forgejo => skein_forge_connection::settings::Kind::Forgejo,
                    settings::Kind::GitHub => skein_forge_connection::settings::Kind::GitHub,
                },
                address: skein_io::Addr::Ip(endpoint.address),
                transport,
                access: None,
                log_origins: forge.log_origins.clone().unwrap_or(endpoint.log_origins),
            })
            .expect("configured forge capacity");
    }
    for (index, endpoint) in endpoints.iter().enumerate() {
        if endpoint.is_some() {
            found.push(forge_path(u32::try_from(index).expect("runtime index")).member(b"api"), What::Refers);
        }
    }
    if found.is_empty() {
        Some(skein_forge_connection::settings::Settings {
            forges: configured,
            webhooks,
            word: Box::from(WORD),
            user_agent: Box::from(USER_AGENT.as_bytes()),
        })
    } else {
        None
    }
}
fn forge_path(index: u32) -> Path {
    Path::root().member(b"application").member(b"forges").child(Segment::Index(index))
}
/// Limits shared with the domain client and io.
#[must_use]
pub fn shared_limits(limits: &limits::Limits, io: &skein_io::Limits) -> skein_forge_connection::Shared {
    skein_forge_connection::Shared {
        calls: limits.forge.client.calls,
        rows: limits.forge.client.rows,
        answer_bytes: limits.forge.client.answer_bytes,
        op_bytes: limits.forge.client.op_bytes,
        lifetime: limits.forge.client.lifetime,
        name_bytes: limits.forge.name_bytes,
        repositories: limits.forge.repositories,
        io: *io,
    }
}
/// The call deadline follows the lifetime, capped for GitHub
/// (temper/docs/design/protocol/configuration/limits.md, "The forge client's lifetime and the forge call's deadline").
#[must_use]
pub(crate) fn deadline(lifetime: Duration) -> Duration {
    lifetime.min(skein_forge_connection::limits::APP_DEADLINE)
}

/// The small profile's component room, derived from the forge where they agree.
#[must_use]
pub fn small(forge: &forge::Limits) -> skein_forge_connection::Limits {
    skein_forge_connection::Limits {
        forges: forge.forges,
        token_bytes: limits::FORGE_TOKEN_BYTES,
        key_bytes: u32::try_from(skein_tls::RS256_KEY_BYTES).expect("RS256 key bound fits u32"),
        installations: limits::FORGE_INSTALLATIONS,
        log_origins: 20,
        log_poll: Duration::from_secs(5),
        log_catch_up: Duration::from_secs(86_400),
        log_pages: 10,
        delivery_bytes: 1_048_576,
        renew_before: Duration::from_secs(10 * 60),
        connections: forge.client.calls,
        kept_bytes: u64::from(forge.client.calls)
            .checked_mul(u64::from(forge.client.answer_bytes))
            .expect("two u32 limits fit u64"),
        head_bytes: 16_384,
        headers: 64,
        body_bytes_in: 16_777_216,
        depth: 16,
        chunk: 4096,
        string_bytes: 512.max(forge.name_bytes),
        body_bytes: 16_384.max(skein_forge::marker::length(WORD, forge.client.op_bytes).unwrap_or(u32::MAX)),
        inner: 32,
        target_bytes: 4096,
        repositories: forge.repositories,
        users: 256,
        deadline: deadline(forge.client.lifetime),
        idle: Duration::from_secs(4),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(60),
        hooks: 4,
        hook_head_bytes: 8192,
        hook_body_bytes: 1_048_576,
        hook_deadline: Duration::from_secs(10),
        hook_secret_bytes: 128,
        tls: skein_tls::client::Limits { read: 16_384, send: 16_384, records: skein_tls::client::MAX_RECORD },
    }
}
/// Report every member involved in the component's refusal.
pub fn problems(unusable: skein_forge_connection::Unusable, found: &mut Problems) {
    use skein_forge_connection::limits::{Stacked, Unusable};
    let names: &[&[u8]] = match unusable {
        Unusable::KeptBytes { .. } => {
            &[b"forge_component.kept_bytes", b"forge.client.calls", b"forge.client.answer_bytes"]
        }
        Unusable::Connections { .. } => &[b"forge_component.connections", b"forge.client.calls"],
        Unusable::Deadline { .. } => &[b"forge_component.deadline_ms", b"forge.client.lifetime_ms"],
        Unusable::BodyBytes { .. } => &[b"forge_component.body_bytes", b"forge.client.op_bytes"],
        Unusable::StringBytes { .. } => &[b"forge_component.string_bytes", b"forge.name_bytes"],
        Unusable::Repositories { .. } => &[b"forge_component.repositories", b"forge.repositories"],
        Unusable::PageSize { .. } => &[
            b"forge_component.body_bytes",
            b"forge_component.string_bytes",
            b"forge_component.inner",
            b"forge.client.answer_bytes",
        ],
        Unusable::KeyBytes { .. } => &[b"forge_component.key_bytes"],
        Unusable::AppDeadline { .. } => &[b"forge_component.deadline_ms"],
        Unusable::RenewBefore { .. } => &[b"forge_component.renew_before_ms", b"forge_component.deadline_ms"],
        Unusable::Documents(_) | Unusable::GitHubDocuments(_) => {
            &[b"forge_component.depth", b"forge_component.body_bytes_in", b"forge_component.chunk"]
        }
        Unusable::Stack { stacked, .. } => match stacked {
            Stacked::HttpOverTls => {
                &[b"forge_component.chunk", b"forge_component.tls.read", b"forge_component.tls.send"]
            }
            Stacked::HttpOverIo => &[b"forge_component.chunk", b"io.intake", b"io.output"],
            Stacked::TlsOverIo => {
                &[b"forge_component.tls.read", b"forge_component.tls.send", b"io.intake", b"io.output"]
            }
            Stacked::JsonOverHttp | Stacked::JsonOverServer => &[b"forge_component.chunk"],
            Stacked::ServerOverIo => &[b"forge_component.hook_head_bytes", b"io.intake", b"io.output"],
        },
        Unusable::SecretBytes { .. } | Unusable::Secret { .. } => &[b"forge_component.hook_secret_bytes"],
        Unusable::Listener => {
            found.push(Path::root().member(b"application").member(b"webhooks").member(b"listener"), What::Plaintext);
            return;
        }
        Unusable::Forges { .. } => &[b"forge_component.forges"],
        Unusable::Request { .. } => &[
            b"forge_component.target_bytes",
            b"forge_component.token_bytes",
            b"forge_component.tls.send",
            b"io.output",
        ],
        Unusable::Zero | Unusable::Word => &[b"forge_component"],
        Unusable::Origin { forge, .. } => {
            found.push(forge_path(u32::from(forge)).member(b"log_origins"), What::Bound);
            return;
        }
        Unusable::LogOrigins { forge, .. } => {
            found.push(forge_path(u32::from(forge)).member(b"log_origins"), What::Bound);
            &[b"forge_component.log_origins"]
        }
        Unusable::Duplicate { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"number"), What::Unique);
            return;
        }
        Unusable::Token { .. } | Unusable::Credential { .. } | Unusable::Key { .. } | Unusable::User { .. } => {
            unreachable!(
                "temper's prepared settings carry no forge access: a forge's access is its records' (credentials.md, 5)"
            )
        }
        Unusable::PlaintextName { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"api"), What::Bound);
            return;
        }
        Unusable::Address { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"api"), What::Plaintext);
            return;
        }
    };
    for name in names {
        found
            .push(Path::root().member(b"application").member(b"limits").member(b"overrides").member(name), What::Bound);
    }
}
#[must_use]
pub fn indexed(
    unusable: skein_forge_connection::Unusable,
    forges: &[settings::Forge],
) -> skein_forge_connection::Unusable {
    let mut unusable = unusable;
    if let skein_forge_connection::Unusable::Duplicate { forge }
    | skein_forge_connection::Unusable::Secret { forge }
    | skein_forge_connection::Unusable::Key { forge }
    | skein_forge_connection::Unusable::Token { forge }
    | skein_forge_connection::Unusable::Credential { forge }
    | skein_forge_connection::Unusable::User { forge }
    | skein_forge_connection::Unusable::PlaintextName { forge }
    | skein_forge_connection::Unusable::Request { forge }
    | skein_forge_connection::Unusable::Origin { forge, .. }
    | skein_forge_connection::Unusable::LogOrigins { forge, .. }
    | skein_forge_connection::Unusable::Address { forge } = &mut unusable
    {
        for (index, configured) in forges.iter().enumerate() {
            if configured.number == *forge {
                *forge = u16::try_from(index).expect("bounded forge index");
                break;
            }
        }
    }
    unusable
}

pub fn prepare(
    config: &settings::Engine,
    _runtime: List<ForgeRuntime>,
) -> Result<skein_forge_connection::settings::Settings, Problems> {
    let mut found = Problems::empty();
    let prepared = settings(&[], List::with_capacity(0), config.webhooks.clone(), &mut found).ok_or(found)?;
    if let Err(why) = skein_forge_connection::limits::check(
        &config.limits.forge_component,
        &shared_limits(&config.limits, &config.jig.limits.io),
        &prepared,
    ) {
        let mut found = Problems::empty();
        problems(indexed(why, &[]), &mut found);
        return Err(found);
    }
    Ok(prepared)
}

#[must_use]
pub fn config(
    settings: &settings::Engine,
    deployment: [u8; 16],
) -> (temper_engine_domain_forge::Config, Box<[forge::GitIdentity]>) {
    (
        temper_engine_domain_forge::Config {
            push: match &settings.push {
                Some(push) => Some(push.services.clone()),
                None => None,
            },
            namespace: Box::new(deployment),
            writers: Box::new([]),
        },
        Box::new([]),
    )
}
