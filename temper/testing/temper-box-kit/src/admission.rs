//! Complete typed startup admission for subprocess fixtures.
use std::path::Path;

use skein_lib::{List, Secret};
use temper_playground::documents::Configuration;
use temper_service::{settings, startup, stock::Given};

/// Check startup and its in-memory construction for a fixture laid by a native story.
pub fn admission(directory: &Path, configuration: &Configuration) -> Result<(), String> {
    let directories = temper_service_shell::jig_directories(&super::configuration::directories(directory));
    let bytes = serde_json::to_vec(configuration).expect("fixture JSON");
    let named =
        startup::named_with_limits(&bytes, &directories, &Given::Stock, &configuration.jig, &configuration.limits)
            .map_err(|why| text(&why))?;
    let files =
        temper_service_shell::read_files_with_asked(named.files, named.asked_files, skein_shell::effective_user())?;
    let agent = temper_agent_fixtures::prepared(directory).agent;
    let seeds = seeds();
    let mut checked = checked(&bytes, &directories, configuration, &files, &agent, &Given::Stock, &seeds)?;
    let accounts = temper_engine_shell::serve::accounts(&checked.engine_mut().jig)?;
    construct(checked, &files, &agent, &seeds, accounts)
}

/// Admit the composed document, typed limits, named files and agent without reading any of them from disk.
pub fn admission_over(
    document: &[u8],
    configuration: &Configuration,
    files: &settings::Files,
    agent: &settings::Agent,
    given: &Given,
) -> Result<(), String> {
    let seeds = seeds();
    let checked =
        checked(document, &jig_protocol_config::Directories::working(), configuration, files, agent, given, &seeds)?;
    construct(checked, files, agent, &seeds, accounts(given))
}

fn text(problems: &startup::Problems) -> String {
    String::from_utf8_lossy(&startup::lines(problems)).into_owned()
}

fn seeds() -> settings::Seeds {
    settings::Seeds {
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
        link_secret: None,
        local_secret: None,
    }
}

fn checked(
    document: &[u8],
    directories: &jig_protocol_config::Directories,
    configuration: &Configuration,
    files: &settings::Files,
    agent: &settings::Agent,
    given: &Given,
    seeds: &settings::Seeds,
) -> Result<startup::CheckedStart, String> {
    startup::checked_start_with_limits(
        document,
        directories,
        files,
        Some(agent),
        seeds,
        List::with_capacity(0),
        None,
        given,
        None,
        &configuration.jig,
        &configuration.limits,
    )
    .map_err(|why| text(&why))
}

fn construct(
    checked: startup::CheckedStart,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: temper_service::AccountsRuntime,
) -> Result<(), String> {
    let runtime = temper_service::WorkerRuntime {
        wires: None,
        root: skein_io::kernel::Fd::new(9),
        path: b"/worker".as_slice().into(),
        agent: temper_playground::documents::pinned_agent(),
    };
    startup::from_checked(checked, files, Some(agent), seeds, Some(accounts), Some(runtime), None, None, Box::new([]))
        .map_err(|why| text(&why))?;
    Ok(())
}

pub(crate) fn accounts(given: &temper_service::stock::Given) -> temper_service::AccountsRuntime {
    use skein_oauth_accounts::{Endpoint, Origin, Transport};
    let mut runtime = temper_service::AccountsRuntime { providers: skein_lib::List::with_capacity(16), trust: None };
    for value in given.running() {
        let Some(sign_in) = &value.provider.sign_in else { continue };
        let endpoint = || {
            let transport = match &sign_in.endpoint_transport {
                jig_protocol_config::Transport::Plaintext => Transport::Plaintext,
                jig_protocol_config::Transport::Tls { server_name, .. } => {
                    let (trust, _) =
                        skein_tls::Config::from_der(&[skein_tls_kit::pki::ROOT.into()], &[]).expect("fixture trust");
                    Transport::Tls {
                        server_name: skein_tls::Name::new(std::str::from_utf8(server_name).expect("TLS name"))
                            .expect("configured TLS name"),
                        trust,
                    }
                }
            };
            Endpoint { address: "127.0.0.1:9".parse().expect("unopened stand-in"), transport }
        };
        let mut origins = skein_lib::List::with_capacity(1);
        origins.push(Origin { uri: sign_in.issuer.clone(), endpoint: endpoint() }).expect("one shared issuer origin");
        runtime.providers.push((value.provider.number, endpoint(), origins)).expect("bounded providers");
    }
    runtime
}
