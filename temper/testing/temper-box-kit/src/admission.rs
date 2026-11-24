//! Checked startup and construction over composed fixture inputs.
use skein_lib::{List, Secret};
use temper_playground::documents::Configuration;
use temper_service::{settings, startup, stock::Given};

/// Admit the composed document, typed limits, named files and agent without reading any of them from disk.
pub fn admission_over(
    document: &[u8],
    configuration: &Configuration,
    files: &settings::Files,
    agent: &settings::Agent,
    given: &Given,
) -> Result<(), String> {
    startup_over(document, configuration, files, Some(agent), given).map(|_| ())
}

/// Construct a checked fixture with or without a local worker.
pub fn startup_over(
    document: &[u8],
    configuration: &Configuration,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    given: &Given,
) -> Result<temper_service::Service, String> {
    let seeds = seeds();
    let checked = startup::checked_start_with_limits(
        document,
        &jig_protocol_config::Directories::working(),
        files,
        agent,
        &seeds,
        List::with_capacity(0),
        None,
        given,
        None,
        &configuration.jig,
        &configuration.limits,
    )
    .map_err(|why| text(&why))?;
    construct_over(checked, files, agent, &seeds, given)
}

fn text(problems: &startup::Problems) -> String {
    String::from_utf8_lossy(&startup::lines(problems)).into_owned()
}

#[must_use]
pub fn seeds() -> settings::Seeds {
    settings::Seeds {
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
        link_secret: None,
        local_secret: None,
    }
}

/// An unopened worker root and a fixed program path for checked fixture construction.
#[must_use]
pub fn worker_runtime() -> temper_service::WorkerRuntime {
    temper_service::WorkerRuntime {
        wires: None,
        root: skein_io::kernel::Fd::new(9),
        path: b"/worker".as_slice().into(),
        agent: temper_playground::documents::pinned_agent(),
    }
}

pub fn construct_over(
    checked: startup::CheckedStart,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    given: &Given,
) -> Result<temper_service::Service, String> {
    startup::from_checked(
        checked,
        files,
        agent,
        seeds,
        Some(accounts(given)),
        agent.map(|_| worker_runtime()),
        None,
        None,
        Box::new([]),
    )
    .map_err(|why| text(&why))
}

#[must_use]
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
