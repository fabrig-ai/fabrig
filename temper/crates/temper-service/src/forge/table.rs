//! Service forge-table handoffs (temper/docs/design/draft/playground-reset.md, 3 and 4).
//! Document compatibility follows temper/docs/plan/playground-reset-r4/notes/contracts.md, D1.
use crate::engine::Engine;
use temper_engine_domain_forge::Forge;

pub(crate) fn kept(engine: &mut Engine, forge: Forge) {
    let number = forge.number;
    if record(engine, number) == Some(&forge) {
        return;
    }
    if record(engine, number).is_some() {
        unhold(engine, number);
    }
    if let Err(why) = hold(engine, forge) {
        let why = match why {
            NotKept::File | NotKept::Trust | NotKept::Bound => crate::NotConnected::Unreadable,
            NotKept::Unresolved => crate::NotConnected::Unreachable { host: Box::from(b"forge address".as_slice()) },
            NotKept::Unserved => not_held(b"Host::add: unserved"),
            NotKept::Started => not_held(b"Host::add: unavailable"),
            NotKept::Host(why) => not_held(host_reason(why)),
            NotKept::ComponentClosed => not_held(b"Component::add: Added::Closed"),
            NotKept::Component(why) => not_held(component_reason(why)),
        };
        engine.notes.push(crate::Note::ForgeNotConnected { number, why });
    }
}
fn not_held(reason: &[u8]) -> crate::NotConnected {
    crate::NotConnected::NotHeld { reason: Box::from(reason) }
}
fn component_reason(why: skein_forge_connection::Unusable) -> &'static [u8] {
    use skein_forge_connection::Unusable as U;
    match why {
        U::LogOrigins { .. } => b"Component::add: Added::Unusable LogOrigins",
        U::Origin { .. } => b"Component::add: Added::Unusable Origin",
        U::Zero => b"Component::add: Added::Unusable Zero",
        U::Word => b"Component::add: Added::Unusable Word",
        U::PlaintextName { .. } => b"Component::add: Added::Unusable PlaintextName",
        U::SecretBytes { .. } => b"Component::add: Added::Unusable SecretBytes",
        U::Secret { .. } => b"Component::add: Added::Unusable Secret",
        U::Listener => b"Component::add: Added::Unusable Listener",
        U::Connections { .. } => b"Component::add: Added::Unusable Connections",
        U::Deadline { .. } => b"Component::add: Added::Unusable Deadline",
        U::BodyBytes { .. } => b"Component::add: Added::Unusable BodyBytes",
        U::StringBytes { .. } => b"Component::add: Added::Unusable StringBytes",
        U::Repositories { .. } => b"Component::add: Added::Unusable Repositories",
        U::PageSize { .. } => b"Component::add: Added::Unusable PageSize",
        U::Documents(_) => b"Component::add: Added::Unusable Documents",
        U::Stack { .. } => b"Component::add: Added::Unusable Stack",
        U::Forges { .. } => b"Component::add: Added::Unusable Forges",
        U::Duplicate { .. } => b"Component::add: Added::Unusable Duplicate",
        U::Credential { .. } => b"Component::add: Added::Unusable Credential",
        U::Token { .. } => b"Component::add: Added::Unusable Token",
        U::Key { .. } => b"Component::add: Added::Unusable Key",
        U::AppDeadline { .. } => b"Component::add: Added::Unusable AppDeadline",
        U::RenewBefore { .. } => b"Component::add: Added::Unusable RenewBefore",
        U::KeyBytes { .. } => b"Component::add: Added::Unusable KeyBytes",
        U::GitHubDocuments(_) => b"Component::add: Added::Unusable GitHubDocuments",
        U::User { .. } => b"Component::add: Added::Unusable User",
        U::Address { .. } => b"Component::add: Added::Unusable Address",
        U::Request { .. } => b"Component::add: Added::Unusable Request",
    }
}
fn host_reason(why: skein_forge_setup::host::Unusable) -> &'static [u8] {
    use skein_forge_setup::host::Unusable as U;
    match why {
        U::Zero => b"Host::add: Unusable Zero",
        U::TokenBytes { .. } => b"Host::add: Unusable TokenBytes",
        U::Documents(_) => b"Host::add: Unusable Documents",
        U::GitHubDocuments(_) => b"Host::add: Unusable GitHubDocuments",
        U::GitHubMetadata { .. } => b"Host::add: Unusable GitHubMetadata",
        U::Stack { .. } => b"Host::add: Unusable Stack",
        U::Forges { .. } => b"Host::add: Unusable Forges",
        U::Duplicate { .. } => b"Host::add: Unusable Duplicate",
        U::Address { .. } => b"Host::add: Unusable Address",
        U::PlaintextName { .. } => b"Host::add: Unusable PlaintextName",
        U::Request { .. } => b"Host::add: Unusable Request",
        U::Given { .. } => b"Host::add: Unusable Given",
        U::GitHubGiven { .. } => b"Host::add: Unusable GitHubGiven",
        U::Reachable { .. } => b"Host::add: Unusable Reachable",
    }
}
#[expect(clippy::manual_map, reason = "record conversions use explicit matches")]
fn hold(engine: &mut Engine, record: Forge) -> Result<(), NotKept> {
    let number = record.number;
    let mut source = settings_forge(&record, &engine.forge.files)?;
    let identity = identity(&record, &engine.forge.files)?;
    let not_set_up = missing(&record, &engine.forge.files);
    let endpoint = endpoint(&record, engine.forge.shell)?;
    let host = match host_forge(engine, &record, &endpoint) {
        Ok(host) => Ok(host),
        Err(NotKept::Unserved) => Err(b"setup host serves plaintext only on loopback".as_slice()),
        Err(why) => return Err(why),
    };
    let held = match &record.api {
        r::Address::Host { name, .. } if endpoint.address == skein_io::Addr::Ip(super::HELD_ADDRESS) => {
            Some(name.clone())
        }
        r::Address::Host { .. } | r::Address::Socket(_) => None,
    };
    let handover = match &record.setup {
        Some(setup) => {
            let path = temper_forge_setup::place::Place { directory: setup.directory.clone(), user: Box::new([]) }
                .path(temper_forge_setup::place::File::AdminToken);
            match engine.forge.files.contents(&path) {
                Some(value) => Some(Secret::new(Box::from(value))),
                None => None,
            }
        }
        None => None,
    };
    if held.is_none()
        && record.files.is_some()
        && source.api_token.is_some()
        && let Some(setup) = &record.setup
        && match &setup.mode {
            r::SetupMode::Fake(_) => true,
            r::SetupMode::Owned(_) | r::SetupMode::Managed { .. } => false,
        }
        && handover.is_none()
    {
        return Err(NotKept::File);
    }
    match engine.forge.component.add(&engine.forge.env, endpoint, &mut engine.forge.above, &mut engine.forge.io) {
        Ok(()) => {}
        Err(skein_forge_connection::Added::Closed) => return Err(NotKept::ComponentClosed),
        Err(skein_forge_connection::Added::Unusable(why)) => return Err(NotKept::Component(why)),
    }
    if let Err(why) = engine.setup.add_forge(number, host, source.api_token.clone()) {
        engine.forge.component.remove(&engine.forge.env, number, &mut engine.forge.above, &mut engine.forge.io);
        return Err(match why {
            NotKept::Bound => {
                NotKept::Host(skein_forge_setup::host::Unusable::Forges { forges: engine.setup.env.limits.forges })
            }
            why @ (NotKept::Unresolved
            | NotKept::Trust
            | NotKept::File
            | NotKept::Unserved
            | NotKept::Started
            | NotKept::Host(_)
            | NotKept::ComponentClosed
            | NotKept::Component(_)) => why,
        });
    }
    if let Some(host) = &held {
        engine.forge.add_held(number, host.clone()).expect("component admitted forge");
    }
    engine.forge_records.rows.add(number).expect("component admitted forge");
    if source.kind == settings::Kind::GitHub {
        engine
            .values
            .tokens
            .add(number, &identity, engine.forge.env.limits.installations)
            .expect("component admitted forge");
    }
    if held.is_none() || record.files.is_none() || record.first.is_some() {
        // A kept first repository needs Create even with legacy file inputs.
        let local = if record.first.is_some() { Some(Forge { files: None, ..record.clone() }) } else { None };
        let local_record = match &local {
            Some(record) => record,
            None => &record,
        };
        engine
            .setup
            .hold_record(local_record, &source, identity, handover, engine.domain.ready())
            .expect("component admitted forge");
    }
    crate::setup::lifecycle::restored_intent(engine, &record);
    source.api_token = None;
    engine.workers.add_forge(source).expect("component admitted forge");
    engine.forge.records.insert(number, record).expect("component admitted forge");
    if not_set_up {
        crate::setup::lifecycle::state(engine, number, temper_engine_domain_forge::Connection::NotSetUp);
    }
    if engine.forge_records.past_connect() && super::records::load_one(engine, number) {
        super::records::connect_one(engine, number);
    }
    Ok(())
}
/// The whole connector record, retained for the box's group file.
pub(crate) fn record(engine: &Engine, forge: u16) -> Option<&Forge> {
    engine.forge.records.get(&forge)
}

pub(crate) fn removed(engine: &mut Engine, forge: u16) {
    unhold(engine, forge);
    crate::setup::erase_forge(engine, forge);
}
fn unhold(engine: &mut Engine, forge: u16) {
    engine.forge.component.remove(&engine.forge.env, forge, &mut engine.forge.above, &mut engine.forge.io);
    let mut callers = engine.setup.drop_forge(forge);
    while let Some(caller) = callers.pop() {
        crate::setup::public::removed(engine, caller);
    }
    engine.forge.records.remove(&forge);
    engine.forge.remove_held(forge);
    engine.forge_records.rows.remove(forge);
    engine.values.tokens.remove(forge);
    engine.workers.remove_forge(forge);
}

pub(crate) fn box_remove(engine: &mut Engine, forge: u16) {
    engine.box_link.limit_held(engine.env.limits.forge.forges);
    let mut out = skein_lib::Queue::with_capacity(2);
    engine.box_link.remove(forge, &mut out);
    crate::route::box_link::drain(engine, &mut out);
}

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
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum NotKept {
    Unresolved,
    Trust,
    File,
    Bound,
    Unserved,
    Started,
    Host(skein_forge_setup::host::Unusable),
    ComponentClosed,
    Component(skein_forge_connection::Unusable),
}
pub(super) fn address(record: &Forge) -> config::Address {
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

#[cfg(test)]
#[path = "../tests/start_over.rs"]
mod start_over;
