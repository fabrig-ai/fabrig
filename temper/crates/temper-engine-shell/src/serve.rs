//! The engine's start for serve (temper/docs/design/protocol/binaries.md, 1 and 2 and temper/docs/design/protocol/components.md).
use crate::{address, path, prefixed, setup};
use jig_protocol_config::{self as config, read};
use skein_io::kernel::{Exit, Fd};
use skein_lib::{List, Secret};
use temper_service::{self as service, settings::Seeds};
use temper_service_shell::{close, jig_directories, read_files_with_asked};
use temper_shell::directories::{Directories, EnvironmentEntry, unread};
mod accounts_runtime;
pub mod bundle;
pub mod github_logs;
mod google_runtime;
pub mod printed;
const CONFIG_BYTES: u32 = 1_048_576;
/// The worker's half of a start, supplied by a binary that holds one.
pub trait WorkerHalf {
    fn preflight(&self, named: &service::startup::Named) -> Result<service::settings::Agent, String>;
    fn alone(&self, directories: &Directories, bytes: &[u8], owned: &mut Vec<Fd>) -> Result<service::Service, String>;
    /// Call `before_root` once after agent preflight, before creating the worker root.
    fn start(
        &self,
        named: &service::startup::Named,
        directories: &Directories,
        owned: &mut Vec<Fd>,
        before_root: &mut dyn FnMut() -> Result<(), String>,
    ) -> Result<WorkerStart, String>;
    fn configure(
        &self,
        service: &mut service::Service,
        agent: &service::settings::Agent,
        named: &service::startup::Named,
        directories: &Directories,
    ) -> Result<(), String>;
}
pub struct WorkerStart {
    pub agent: service::settings::Agent,
    pub runtime: service::WorkerRuntime,
    pub program_login: Option<Box<[u8]>>,
}
/// Words needed by program-login on this deployment (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn program_environment(
    directories: &Directories,
    environment: &[EnvironmentEntry],
    working: &[u8],
    word: &[u8],
) -> Box<[u8]> {
    // temper/docs/design/client/forge/records.md: the answer's byte bound.
    const BOUND: usize = 49_216;
    let defaults: Vec<_> = environment
        .iter()
        .filter(|(name, _)| ![b"TEMPER_CONFIG".as_slice(), b"TEMPER_DATA", b"TEMPER_CACHE"].contains(&name.as_ref()))
        .cloned()
        .collect();
    let defaults = temper_shell::directories::resolve(&[], &defaults, working).ok().map(|(defaults, _)| defaults);
    let home = skein_shell::HOME.value(environment);
    let mut words = Vec::new();
    for (name, path, default) in [
        (b"TEMPER_CONFIG=".as_slice(), &directories.config, defaults.as_ref().map(|dirs| &dirs.config)),
        (b"TEMPER_DATA=", &directories.data, defaults.as_ref().map(|dirs| &dirs.data)),
        (b"TEMPER_CACHE=", &directories.cache, defaults.as_ref().map(|dirs| &dirs.cache)),
    ] {
        if Some(path) != default {
            words.extend_from_slice(name);
            words.extend_from_slice(&printed::path_word(path, home));
            words.push(b' ');
        }
    }
    let word = if word.len() <= BOUND.saturating_sub(words.len()) { word } else { b"temper" };
    if !words.is_empty() || word != b"temper" {
        words.extend_from_slice(word);
    }
    words.into_boxed_slice()
}

fn member(document: &skein_json::Document, at: u32, key: &[u8]) -> u32 {
    read::Object::new(document, at).expect("named checked object").member(document, key).expect("named checked member")
}
pub(crate) fn trust(file: Option<&[u8]>) -> Result<skein_tls::Config, String> {
    let source = file
        .map_or(skein_shell::Trust::Machine { ssl_cert_file: None }, |file| skein_shell::Trust::Der(path(file).into()));
    let roots = skein_shell::read_trust_roots(
        &source,
        &skein_shell::RootBounds { bundle_bytes: CONFIG_BYTES, certificates: 1024, certificate_bytes: 65_536 },
    )
    .map_err(|why| match why {
        skein_shell::RootsRefusal::NoBundle => "no trust bundle".into(),
        skein_shell::RootsRefusal::Unread { path, why } => format!("{}: {}", path.display(), unread(why)),
        skein_shell::RootsRefusal::TooMany { bound } => format!("past {bound} trust roots"),
        skein_shell::RootsRefusal::TooLong { index, bound } => format!("trust root {index}: past {bound} bytes"),
    })?;
    let (trust, _) = skein_tls::Config::from_der(&roots.certificates, &[]).map_err(|why| match why {
        skein_tls::Refusal::Roots => "no supported trust roots",
        skein_tls::Refusal::Alpn => "protocol names refused",
        skein_tls::Refusal::Provider => "TLS provider refused",
    })?;
    Ok(trust)
}
fn server_name(bytes: &[u8]) -> Result<skein_tls::Name, String> {
    skein_tls::Name::new(std::str::from_utf8(bytes).map_err(|why| why.to_string())?)
        .ok_or_else(|| "invalid server name".into())
}
fn accounts(engine: &config::Engine) -> Result<service::AccountsRuntime, String> {
    use std::net::ToSocketAddrs;
    accounts_runtime::resolve(engine, |host, port| (host, port).to_socket_addrs().ok()?.next())
}

/// Resolve the machine's GitHub host once at start (temper/docs/design/protocol/configuration/engine.md).
fn github_runtime(
    binary: &str,
    machine: Option<&temper_forge_setup::machine::GitHubMachine>,
    roots: &skein_tls::Config,
    resolve: fn(&config::Address) -> Result<std::net::SocketAddr, String>,
) -> Result<service::ForgeRuntime, String> {
    let default = config::Address::Host { name: Box::from(b"api.github.com".as_slice()), port: 443 };
    let api = machine.map_or(&default, |machine| &machine.api);
    let address = resolve(api).unwrap_or(service::forge::HELD_ADDRESS);
    let transport = if machine.is_none_or(|machine| machine.tls) {
        let name = match api {
            config::Address::Host { name, .. } => server_name(name)?,
            config::Address::Addr(address) => {
                server_name(address.socket().ok_or("requires a socket address")?.ip().to_string().as_bytes())?
            }
        };
        skein_forge_connection::settings::Transport::Tls { server_name: name, trust: roots.clone() }
    } else {
        skein_forge_connection::settings::Transport::Plaintext { name: None }
    };
    let log_origins =
        if machine.is_none() { github_logs::resolve_for(binary, None, resolve)? } else { List::with_capacity(0) };
    Ok(service::ForgeRuntime { number: 0, address, transport, log_origins })
}
pub fn prepare(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<service::Service, String> {
    prepare_with_store(binary, environment, working, worker, directories, bytes, owned).map(|(service, _)| service)
}
pub fn prepare_with_store(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<(service::Service, Box<[u8]>), String> {
    prepare_with_limits(
        binary,
        environment,
        working,
        worker,
        directories,
        bytes,
        owned,
        false,
        None,
        None,
        &service::stock::Given::Stock,
    )
    .map(|(service, store, _)| (service, store))
}
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "startup supplies the binary and worker half beside the resolved process facts"
)]
fn prepare_with_limits(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
    under_box: bool,
    link_secret: Option<Secret>,
    local_secret: Option<Secret>,
    given: &service::stock::Given,
) -> Result<(service::Service, Box<[u8]>, u32), String> {
    if worker.is_none()
        && matches!(
            service::startup::nearest(bytes),
            Some(service::startup::Shape::WorkerAlone | service::startup::Shape::Standalone)
        )
    {
        return Err(format!("temper.json names a worker, and {binary} runs none"));
    }
    if service::startup::shape_with(bytes, &jig_directories(directories), given)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?
        == service::startup::Shape::WorkerAlone
    {
        return worker
            .expect("worker alone requires its half")
            .alone(directories, bytes, owned)
            .map(|service| (service, Box::new([]) as Box<[u8]>, 0));
    }
    let setup::Deployment { mut named, checked, limits, .. } =
        setup::check_with(bytes, &jig_directories(directories), given)?;
    if worker.is_none() && named.worker.is_some() {
        return Err(format!("temper.json names a worker, and {binary} runs none"));
    }
    let document =
        config::document(bytes, &config::DocumentLimits { bytes: CONFIG_BYTES, tokens: CONFIG_BYTES, depth: 32 })
            .expect("named checked document");
    let engine = temper_service::startup::engine_with(
        &document,
        member(&document, 0, b"engine"),
        &jig_directories(directories),
        given,
    )
    .expect("named checked engine");
    let roots = trust(None)?;
    let github = github_runtime(binary, named.machine.github.as_ref(), &roots, address)?;
    let forges = List::with_capacity(0);
    let user = skein_shell::effective_user();
    let mut inputs = None;
    let mut names = core::mem::take(&mut named.files);
    let mut asked = core::mem::take(&mut named.asked_files);
    // Agent preflight precedes these checks; none may create the worker root.
    let mut before_root = || {
        let files = read_files_with_asked(core::mem::take(&mut names), core::mem::take(&mut asked), user)?;
        let google = google_runtime::resolve(named.sign_in.as_ref())?;
        let mut bundle = match named.bundle.as_deref() {
            Some(configured) => Some(bundle::read(configured, named.bundle_limits)?),
            None => bundle::own(named.bundle_limits)?,
        };
        if let Some(bundle) = &mut bundle {
            bundle.fill_sign_in(named.sign_in.is_some()).ok_or_else(|| {
                format!(
                    "application.clients.bundle: {}: the page has no </head> for its sign-in metadata",
                    named.bundle.as_deref().map_or("embedded".into(), |name| String::from_utf8_lossy(name))
                )
            })?;
        }
        inputs = Some((files, google, bundle));
        Ok(())
    };
    let start = if named.worker.is_some() {
        Some(worker.expect("named worker requires its half").start(&named, directories, owned, &mut before_root)?)
    } else {
        before_root()?;
        None
    };
    let (files, google, bundle) = inputs.expect("engine inputs checked before the worker root");
    let (agent, runtime, program_login) = match start {
        Some(WorkerStart { agent, runtime, program_login }) => (Some(agent), Some(runtime), program_login),
        None => (None, None, None),
    };
    let accounts = accounts(&engine)?;
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let crypto = skein_shell::crypto_seed().map_err(|errno| format!("cryptographic seed: errno {errno}"))?;
    let drawn = skein_shell::crypto_seed().map_err(|errno| format!("deployment id: errno {errno}"))?;
    let mut deployment = [0_u8; 16];
    deployment.copy_from_slice(drawn.expose().get(..16).expect("32-byte seed"));
    // Drawn at every start; the service keeps it at the first and restores
    // the kept one at every later start (jig's configuration.md 7).
    let drawn = skein_shell::crypto_seed().map_err(|errno| format!("installation id: errno {errno}"))?;
    let mut installation = [0_u8; 16];
    installation.copy_from_slice(drawn.expose().get(..16).expect("32-byte seed"));
    let mut service = service::startup::with_setup_given(
        bytes,
        &jig_directories(directories),
        &files,
        agent.as_ref(),
        &Seeds { deployment, installation, seed, crypto, link_secret, local_secret },
        Some(accounts),
        runtime,
        forges,
        Some(&checked),
        if under_box { Some(limits.line_bytes) } else { None },
        program_login,
        given,
        program_environment(directories, environment, working, &printed::running_word(environment)),
    )
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    service.configure_forge_runtime(github, roots);
    service.configure_google_endpoints(google);
    if let Some(agent) = &agent {
        worker.expect("local agent requires its half").configure(&mut service, agent, &named, directories)?;
    }
    let root = skein_shell::make_root(path(&named.deployment), 0o700)
        .map_err(|errno| format!("engine.deployment.directory: errno {errno}"))?;
    owned.push(root);
    service.adopt_store(root, user).map_err(|_| "store refused its deployment root")?;
    if let Some(bundle) = bundle {
        service.adopt_bundle(bundle);
    }
    Ok((service, named.deployment, limits.line_bytes))
}
/// The kernel operations a standalone start keeps in flight at most (its ring's size),
/// derived without retaining its descriptors.
pub fn operations_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    operations_in_mode(binary, environment, working, worker, directories, config, false)
}
/// The same for a boxed start.
pub fn boxed_operations_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    operations_in_mode(binary, environment, working, worker, directories, config, true)
}
fn operations_in_mode(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    config: &[u8],
    boxed: bool,
) -> Result<u32, String> {
    directories.open(skein_shell::effective_user())?;
    let mut owned = Vec::new();
    let result = prepare_with_limits(
        binary,
        environment,
        working,
        worker,
        directories,
        config,
        &mut owned,
        boxed,
        None,
        None,
        &service::stock::Given::Stock,
    )
    .map(|(service, _, _)| service.operations());
    close(owned);
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
pub fn prepare_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
) -> Result<service::Service, String> {
    let bytes = directories.read().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result = prepare(binary, environment, working, worker, directories, &bytes, &mut owned);
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
/// Prepare an in-process world's service with the supplied defaults.
pub fn prepare_in_with(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    given: &service::stock::Given,
) -> Result<service::Service, String> {
    let bytes = directories.read().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result = prepare_with_limits(
        binary,
        environment,
        working,
        worker,
        directories,
        &bytes,
        &mut owned,
        false,
        None,
        None,
        given,
    )
    .map(|(service, _, _)| service);
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
/// The same for an in-process world's service, with the supplied defaults.
pub fn operations_in_with(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    worker: Option<&dyn WorkerHalf>,
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
) -> Result<u32, String> {
    directories.open(skein_shell::effective_user())?;
    let mut owned = Vec::new();
    let result = prepare_with_limits(
        binary,
        environment,
        working,
        worker,
        directories,
        config,
        &mut owned,
        false,
        None,
        None,
        given,
    )
    .map(|(service, _, _)| service.operations());
    close(owned);
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
pub fn run(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    boxed: bool,
    worker: Option<&dyn WorkerHalf>,
) -> Result<Exit, String> {
    temper_service_shell::run(binary, directories, environment, boxed, |bytes, owned, secrets| {
        prepare_with_limits(
            binary,
            environment,
            working,
            worker,
            directories,
            bytes,
            owned,
            boxed,
            secrets.link,
            secrets.local,
            &service::stock::Given::Stock,
        )
        .map(|(service, store, line_bytes)| temper_service_shell::Prepared { service, store, line_bytes })
    })
}

#[cfg(test)]
#[path = "serve/tests/program_environment.rs"]
mod program_environment;

#[cfg(test)]
mod tests;
