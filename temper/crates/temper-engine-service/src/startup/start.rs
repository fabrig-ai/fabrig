//! The engine's start joins checked startup settings to the component (protocol/services.md, 4.1).
use crate::startup::{CheckedStart, Problems, checked_start_with, lines};
use crate::{AccountsRuntime, Service, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_lib::List;

/// Check serve's configuration, building only the engine for an upgrade tool (protocol/binaries.md).
pub fn restoring_with(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    given: &crate::stock::Given,
) -> Result<Service, Problems> {
    let CheckedStart { engine, io, forges, .. } =
        checked_start_with(document, directories, files, agent, seeds, List::with_capacity(0), None, given)?;
    restoring_prepared(engine, seeds, accounts, forges, io)
}

/// Restore an engine whose document and runtime facts were checked by the composing service.
pub fn restoring_prepared(
    engine: settings::Engine,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    forges: skein_forge_connection::Settings,
    io: skein_io::Limits,
) -> Result<Service, Problems> {
    match Service::prepared(engine, seeds, accounts, forges, io, None, None) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

/// Start an engine alone with the checked runtime facts.
#[expect(clippy::too_many_arguments, reason = "the engine start takes its runtime facts beside the document")]
pub fn with_setup_given(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
    given: &crate::stock::Given,
    program_environment: Box<[u8]>,
) -> Result<Service, Problems> {
    let CheckedStart { mut engine, io, forges, not_set_up, warnings } =
        checked_start_with(document, directories, files, None, seeds, forges, setup, given)?;
    engine.program_environment = program_environment;

    match Service::prepared(engine, seeds, accounts, forges, io, None, None) {
        Ok(mut service) => {
            for warning in &warnings.problems {
                let mut found = Problems::empty();
                found.push(warning.path.clone(), warning.what);
                let line = lines(&found);
                service.notes().push(crate::Note::StartupWarning {
                    line: line.strip_suffix(b"\n").expect("one warning line").into(),
                });
            }
            for number in &not_set_up {
                crate::setup::lifecycle::state(
                    service.engine.as_mut().expect("engine startup"),
                    *number,
                    temper_engine_domain_forge::Connection::NotSetUp,
                );
            }
            Ok(service)
        }
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}
