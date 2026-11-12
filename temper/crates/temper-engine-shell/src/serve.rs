//! The engine's start for serve (temper/docs/design/protocol/binaries.md, 1 and 2 and temper/docs/design/protocol/components.md).
use crate::{address, path, prefixed, setup};
use jig_protocol_config::{self as config, read};
use skein_io::kernel::{Exit, Fd};
use skein_lib::{List, Secret};
use temper_engine_service::{self as service, settings::Seeds};
use temper_service_shell::{close, jig_directories, read_files_with_asked};
use temper_shell::directories::{Directories, EnvironmentEntry, unread};
mod accounts_runtime;
pub mod bundle;
pub mod github_logs;
mod google_runtime;
pub mod printed;
const CONFIG_BYTES: u32 = 1_048_576;
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
pub fn trust(file: Option<&[u8]>) -> Result<skein_tls::Config, String> {
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
pub fn accounts(engine: &config::Engine) -> Result<service::AccountsRuntime, String> {
    use std::net::ToSocketAddrs;
    accounts_runtime::resolve(engine, |host, port| (host, port).to_socket_addrs().ok()?.next())
}

/// Resolve the machine's GitHub host once at start (temper/docs/design/protocol/configuration/engine.md).
pub fn github_runtime(
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
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<service::Service, String> {
    prepare_with_store(binary, environment, working, directories, bytes, owned).map(|(service, _)| service)
}
pub fn prepare_with_store(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<(service::Service, Box<[u8]>), String> {
    prepare_given(binary, environment, working, directories, bytes, owned, &service::stock::Given::Stock)
}
fn prepare_given(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
    given: &service::stock::Given,
) -> Result<(service::Service, Box<[u8]>), String> {
    if matches!(
        service::startup::nearest(bytes),
        Some(service::startup::Shape::WorkerAlone | service::startup::Shape::Standalone)
    ) {
        return Err(format!("temper.json names a worker, and {binary} runs none"));
    }
    service::startup::shape_with(bytes, &jig_directories(directories), given)
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let setup::Deployment { mut named, checked, .. } = setup::check_with(bytes, &jig_directories(directories), given)?;
    let engine = settings(bytes, directories, given);
    let Runtimes { roots, github } = runtimes(binary, &named)?;
    let user = skein_shell::effective_user();
    let Inputs { files, google, bundle } = before_root(&mut named, user)?;
    let accounts = accounts(&engine)?;
    let seeds = seeds(None, None)?;
    let mut service = service::startup::with_setup_given(
        bytes,
        &jig_directories(directories),
        &files,
        &seeds,
        Some(accounts),
        List::with_capacity(0),
        Some(&checked),
        given,
        program_environment(directories, environment, working, &printed::running_word(environment)),
    )
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    service.configure_forge_runtime(github, roots);
    service.configure_google_endpoints(google);
    adopt(&mut service, &named.deployment, bundle, user, owned)?;
    Ok((service, named.deployment))
}
/// The kernel operations an engine alone keeps in flight at most, without retaining its descriptors.
pub fn operations_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    operations_in_with(binary, environment, working, directories, config, &service::stock::Given::Stock)
}
pub fn prepare_in(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
) -> Result<service::Service, String> {
    let bytes = directories.read().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result = prepare(binary, environment, working, directories, &bytes, &mut owned);
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
    directories: &Directories,
    given: &service::stock::Given,
) -> Result<service::Service, String> {
    let bytes = directories.read().map_err(|why| prefixed(&format!("{binary} serve"), &why))?;
    let mut owned = Vec::new();
    let result =
        prepare_given(binary, environment, working, directories, &bytes, &mut owned, given).map(|(service, _)| service);
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
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
) -> Result<u32, String> {
    directories.open(skein_shell::effective_user())?;
    let mut owned = Vec::new();
    let result = prepare_given(binary, environment, working, directories, config, &mut owned, given)
        .map(|(service, _)| service.operations());
    close(owned);
    result.map_err(|why| prefixed(&format!("{binary} serve"), &why))
}
pub fn run(
    binary: &str,
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
) -> Result<Exit, String> {
    temper_service_shell::run(
        binary,
        directories,
        environment,
        crate::served::SERVED,
        |_| Ok(()),
        |bytes, owned, ()| {
            prepare_with_store(binary, environment, working, directories, bytes, owned)
                .map(|(service, store)| temper_service_shell::Prepared { service, store, adoption: () })
        },
        |_, _, ()| Ok(()),
    )
}

/// The engine settings after the setup check has named the document.
#[must_use]
pub fn settings(bytes: &[u8], directories: &Directories, given: &service::stock::Given) -> config::Engine {
    let document =
        config::document(bytes, &config::DocumentLimits { bytes: CONFIG_BYTES, tokens: CONFIG_BYTES, depth: 32 })
            .expect("named checked document");
    temper_engine_service::startup::engine_with(
        &document,
        member(&document, 0, b"engine"),
        &jig_directories(directories),
        given,
    )
    .expect("named checked engine")
}
/// Forge runtimes checked before any agent preflight.
pub struct Runtimes {
    pub roots: skein_tls::Config,
    pub github: service::ForgeRuntime,
}
pub fn runtimes(binary: &str, named: &service::startup::Named) -> Result<Runtimes, String> {
    let roots = trust(None)?;
    let github = github_runtime(binary, named.machine.github.as_ref(), &roots, address)?;
    Ok(Runtimes { roots, github })
}
/// Engine inputs checked after agent preflight and before the worker root.
pub struct Inputs {
    pub files: service::settings::Files,
    pub google: service::clients::fetch::Runtime,
    pub bundle: Option<service::clients::bundle::Bundle>,
}
pub fn before_root(named: &mut service::startup::Named, user: u32) -> Result<Inputs, String> {
    let required = core::mem::take(&mut named.files);
    let asked = core::mem::take(&mut named.asked_files);
    let files = read_files_with_asked(required, asked, user)?;
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
    Ok(Inputs { files, google, bundle })
}
pub fn seeds(link_secret: Option<Secret>, local_secret: Option<Secret>) -> Result<Seeds, String> {
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
    Ok(Seeds { deployment, installation, seed, crypto, link_secret, local_secret })
}
/// Adopt the engine's store and page after the caller has configured its service.
pub fn adopt(
    service: &mut service::Service,
    deployment: &[u8],
    bundle: Option<service::clients::bundle::Bundle>,
    user: u32,
    owned: &mut Vec<Fd>,
) -> Result<(), String> {
    let root = skein_shell::make_root(path(deployment), 0o700)
        .map_err(|errno| format!("engine.deployment.directory: errno {errno}"))?;
    owned.push(root);
    service.adopt_store(root, user).map_err(|_| "store refused its deployment root")?;
    if let Some(bundle) = bundle {
        service.adopt_bundle(bundle);
    }
    Ok(())
}

#[cfg(test)]
#[path = "serve/tests/program_environment.rs"]
mod program_environment;

#[cfg(test)]
mod tests;
