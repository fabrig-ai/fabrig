//! Complete typed startup admission for subprocess fixtures.
use std::path::Path;

/// Check startup and its in-memory construction without opening a store or running a service.
pub fn admission(directory: &Path, configuration: &temper_playground::documents::Configuration) -> Result<(), String> {
    use skein_lib::{List, Secret};
    use temper_service::{settings, startup};
    let directories = temper_service_shell::jig_directories(&super::configuration::directories(directory));
    let bytes = serde_json::to_vec(configuration).expect("fixture JSON");
    let text = |problems: startup::Problems| String::from_utf8_lossy(&startup::lines(&problems)).into_owned();
    let named = startup::named_with_limits(
        &bytes,
        &directories,
        &temper_service::stock::Given::Stock,
        &configuration.jig,
        &configuration.limits,
    )
    .map_err(text)?;
    let files =
        temper_service_shell::read_files_with_asked(named.files, named.asked_files, skein_shell::effective_user())?;
    let agent = temper_agent_fixtures::prepared(directory).agent;
    let seeds = settings::Seeds {
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
        link_secret: None,
        local_secret: None,
    };
    let mut checked = startup::checked_start_with_limits(
        &bytes,
        &directories,
        &files,
        Some(&agent),
        &seeds,
        List::with_capacity(0),
        None,
        &temper_service::stock::Given::Stock,
        None,
        &configuration.jig,
        &configuration.limits,
    )
    .map_err(text)?;
    let runtime = temper_service::WorkerRuntime {
        wires: None,
        root: skein_io::kernel::Fd::new(9),
        path: b"/worker".as_slice().into(),
        agent: temper_playground::documents::pinned_agent(),
    };
    let accounts = temper_engine_shell::serve::accounts(&checked.engine_mut().jig)?;
    startup::from_checked(
        checked,
        &files,
        Some(&agent),
        &seeds,
        Some(accounts),
        Some(runtime),
        None,
        None,
        Box::new([]),
    )
    .map_err(text)?;
    Ok(())
}
