//! Service forge-table handoffs (temper/docs/design/draft/playground-reset.md, 3 and 4).
//! Document compatibility follows temper/docs/plan/playground-reset-r4/notes/contracts.md, D1.
#![cfg_attr(not(test), expect(dead_code, reason = "record handoffs are wired by fr-forge-added"))]
use crate::engine::Engine;
use temper_engine_domain_forge::Forge;

pub(crate) fn kept(_engine: &mut Engine, _forge: Forge) {}

pub(crate) fn removed(_engine: &mut Engine, _forge: u16) {}

pub(crate) fn box_remove(_engine: &mut Engine, _forge: u16) {}

use crate::settings;
use alloc::boxed::Box;
use jig_protocol_config as config;
use skein_lib::{List, Secret};
use temper_engine_domain_forge::record as r;
use temper_forge_setup::setup as s;

/// What the shell resolves at the start and for a forge kept later.
#[derive(Clone, Copy, Debug)]
#[expect(clippy::type_complexity, reason = "the shell contract names its two resolver signatures directly")]
pub struct Shell {
    pub address: fn(&config::Address) -> Result<core::net::SocketAddr, Box<[u8]>>,
    pub trust: fn(Option<&[u8]>) -> Result<skein_tls::Config, Box<[u8]>>,
}
impl Shell {
    /// A shell for callers that resolve no runtime endpoints.
    #[must_use]
    pub const fn refusing() -> Self {
        Self { address: refuse_address, trust: refuse_trust }
    }
}
fn refuse_address(_: &config::Address) -> Result<core::net::SocketAddr, Box<[u8]>> {
    Err(Box::from(b"address resolution refused".as_slice()))
}
fn refuse_trust(_: Option<&[u8]>) -> Result<skein_tls::Config, Box<[u8]>> {
    Err(Box::from(b"trust resolution refused".as_slice()))
}
/// A record could not supply a bounded service entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum NotKept {
    Unresolved,
    Trust,
    File,
    Bound,
    Unserved,
    Started,
}
fn address(record: &Forge) -> config::Address {
    match &record.api {
        r::Address::Socket(address) => config::Address::Addr(skein_io::Addr::Ip(*address)),
        r::Address::Host { name, port } => config::Address::Host { name: name.clone(), port: *port },
    }
}
fn contents(files: &settings::Files, path: &[u8], bound: u32) -> Result<Box<[u8]>, NotKept> {
    let bytes = files.contents(path).ok_or(NotKept::File)?;
    if bytes.len() > usize::try_from(bound).or(Err(NotKept::Bound))? {
        return Err(NotKept::Bound);
    }
    Ok(Box::from(bytes))
}
fn missing(record: &Forge, files: &settings::Files) -> bool {
    let Some(setup) = &record.setup else { return false };
    if !setup.asked {
        return false;
    }
    let Some(named) = &record.files else { return false };
    if files.contents(&named.api_token).is_none() {
        return true;
    }
    if let r::FileUser::File(name) = &named.user
        && files.contents(name).is_none()
    {
        return true;
    }
    if let Some(name) = &named.webhook_secret
        && files.contents(name).is_none()
    {
        return true;
    }
    if let Some(name) = &named.git_token
        && files.contents(name).is_none()
    {
        return true;
    }
    false
}
pub(crate) fn settings_forge(record: &Forge, files: &settings::Files) -> Result<settings::Forge, NotKept> {
    if !record.fits() {
        return Err(NotKept::Bound);
    }
    let missing = missing(record, files);
    let mut api_token = None;
    let mut webhook_secret = None;
    let mut user = 0;
    if let Some(named) = &record.files {
        if !missing {
            api_token = Some(config::secret(contents(files, &named.api_token, crate::startup::API_TOKEN_BYTES)?));
            if let Some(name) = &named.webhook_secret {
                webhook_secret = Some(config::secret(contents(files, name, crate::startup::API_TOKEN_BYTES)?));
            }
        }
        user = match &named.user {
            r::FileUser::Id(value) => *value,
            r::FileUser::File(name) if missing && files.contents(name).is_none() => 0,
            r::FileUser::File(name) => {
                let bytes = contents(files, name, crate::startup::application::USER_BYTES)?;
                let bytes = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
                if bytes.is_empty() || bytes.first() == Some(&b'0') {
                    return Err(NotKept::File);
                }
                let mut value = 0_u64;
                for digit in bytes {
                    if !digit.is_ascii_digit() {
                        return Err(NotKept::File);
                    }
                    value = value
                        .checked_mul(10)
                        .ok_or(NotKept::File)?
                        .checked_add(u64::from(digit.checked_sub(b'0').ok_or(NotKept::File)?))
                        .ok_or(NotKept::File)?;
                }
                value
            }
        };
    }
    Ok(settings::Forge {
        mode: temper_engine_domain_forge::Configured::from_record(record).mode,
        number: record.number,
        kind: match record.kind {
            temper_engine_domain_forge::Kind::Forgejo => settings::Kind::Forgejo,
            temper_engine_domain_forge::Kind::GitHub => settings::Kind::GitHub,
        },
        api: address(record),
        transport: match &record.transport {
            r::Transport::Plaintext => settings::Transport::Plaintext,
            r::Transport::Tls { server_name, trust } => settings::Transport::Tls {
                server_name: server_name.clone(),
                trust: match trust {
                    Some(name) => Some(contents(files, name, 1_048_576)?),
                    None => None,
                },
            },
        },
        git: record.git.clone(),
        git_program: record.git_program.clone(),
        web: record.web.clone(),
        app_read_only: record.read_only,
        api_token,
        webhook_secret,
        user,
        identity: record.identity.number,
    })
}
pub(crate) fn identity(record: &Forge, files: &settings::Files) -> Result<settings::Identity, NotKept> {
    if !record.fits() {
        return Err(NotKept::Bound);
    }
    let token = match &record.files {
        Some(named) if !missing(record, files) => match &named.git_token {
            Some(name) => {
                config::secret(contents(files, name, jig_protocol_worker_wire::CEILINGS.application_value_secret)?)
            }
            None => Secret::new(Box::new([])),
        },
        Some(_) | None => Secret::new(Box::new([])),
    };
    Ok(settings::Identity {
        number: record.identity.number,
        forge: record.number,
        token,
        login: record.identity.login.clone(),
        author: settings::Author { name: record.identity.name.clone(), email: record.identity.email.clone() },
    })
}
#[expect(clippy::disallowed_methods, reason = "the protocol translation validates a record's TLS server name")]
fn tls_name(bytes: &[u8]) -> Result<skein_tls::Name, NotKept> {
    skein_tls::Name::new(core::str::from_utf8(bytes).or(Err(NotKept::Trust))?).ok_or(NotKept::Trust)
}
pub(crate) fn endpoint(record: &Forge, shell: Shell) -> Result<skein_forge_connection::settings::Forge, NotKept> {
    use skein_forge_connection::settings as connection;
    if !record.fits() {
        return Err(NotKept::Bound);
    }
    let api = address(record);
    let address = match (shell.address)(&api) {
        Ok(value) => value,
        Err(_) => match api {
            config::Address::Host { .. } => super::HELD_ADDRESS,
            config::Address::Addr(_) => return Err(NotKept::Unresolved),
        },
    };
    let transport = match &record.transport {
        r::Transport::Plaintext => connection::Transport::Plaintext {
            name: match &record.api {
                r::Address::Host { name, .. } => Some(name.clone()),
                r::Address::Socket(_) => None,
            },
        },
        r::Transport::Tls { server_name, trust } => connection::Transport::Tls {
            server_name: tls_name(server_name)?,
            trust: (shell.trust)(trust.as_deref()).or(Err(NotKept::Trust))?,
        },
    };
    Ok(connection::Forge {
        number: record.number,
        kind: match record.kind {
            temper_engine_domain_forge::Kind::Forgejo => connection::Kind::Forgejo,
            temper_engine_domain_forge::Kind::GitHub => connection::Kind::GitHub,
        },
        address: skein_io::Addr::Ip(address),
        transport,
        access: None,
        log_origins: List::with_capacity(0),
    })
}
/// Build one setup host entry from its record and the machine's settings.
pub(crate) fn host_forge(
    engine: &Engine,
    record: &Forge,
    endpoint: &skein_forge_connection::settings::Forge,
) -> Result<skein_forge_setup::host::Forge, NotKept> {
    if record.number != endpoint.number {
        return Err(NotKept::Unresolved);
    }
    let source = settings_forge(record, &engine.forge.files)?;
    let entry = setup_entry(record);
    crate::setup::settings::one(
        engine.setup.context.as_ref().ok_or(NotKept::Bound)?,
        &source,
        entry.as_ref(),
        record.files.is_none(),
        endpoint,
    )
    .or(Err(NotKept::Bound))?
    .ok_or(NotKept::Unserved)
}
fn origin(value: &r::Origin) -> s::Origin {
    s::Origin { host: value.host.clone(), port: value.port }
}
fn account(value: &r::Account) -> s::Account {
    s::Account { login: value.login.clone(), email: value.email.clone() }
}
#[expect(clippy::manual_map, reason = "record translations use explicit matches")]
pub(crate) fn setup_entry(record: &Forge) -> Option<s::Entry> {
    let value = record.setup.as_ref()?;
    Some(s::Entry {
        forge: record.number,
        directory: value.directory.clone(),
        bot: account(&value.bot),
        orgs: value.orgs.clone(),
        asked: value.asked,
        mode: match &value.mode {
            r::SetupMode::Managed { admin_token } => s::Mode::Managed { admin_token: admin_token.clone() },
            r::SetupMode::Owned(value) => s::Mode::Owned(s::Owned {
                public: match &value.public {
                    Some(value) => Some(origin(value)),
                    None => None,
                },
                fetch: s::Fetch {
                    forgejo: value.fetch.forgejo,
                    runner: value.fetch.runner,
                    mirror: match &value.fetch.mirror {
                        Some(value) => {
                            Some(s::Mirror { forgejo: origin(&value.forgejo), runner: origin(&value.runner) })
                        }
                        None => None,
                    },
                },
                forgejo: value.forgejo.clone(),
                listen: value.listen,
                person: account(&value.person),
                runner: match &value.runner {
                    Some(value) => Some(s::Runner {
                        binary: value.binary.clone(),
                        labels: value.labels.clone(),
                        capacity: value.capacity,
                        path: value.path.clone(),
                    }),
                    None => None,
                },
            }),
            r::SetupMode::Fake(value) => {
                let mut repositories = List::with_capacity(u32::try_from(value.repositories.len()).ok()?);
                for repository in &value.repositories {
                    repositories
                        .push(s::Repository { name: repository.name.clone(), landing: repository.landing.clone() })
                        .ok()?;
                }
                s::Mode::Fake(s::Fake {
                    listen: value.listen,
                    git: value.git.clone(),
                    owner: value.owner.clone(),
                    repositories: repositories.into_boxed(),
                })
            }
        },
    })
}
