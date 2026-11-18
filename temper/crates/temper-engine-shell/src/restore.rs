//! Restore over a stopped deployment (temper/docs/design/protocol/checkpoints.md, 2.5).
use crate::{path, prefixed, stopped};
use jig_protocol_store::{CheckpointName, CheckpointRefusal, Event, RestoreRefusal, Restored};
use temper_engine_service as service;
use temper_shell::directories::{self, Directories, EnvironmentEntry};

#[cfg(test)]
mod tests;

pub fn configure(
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Directories, CheckpointName), String> {
    let (directories, rest) = directories::resolve(arguments, environment, working).map_err(|why| why.to_string())?;
    let [name] = rest.as_ref() else { return Err("invalid arguments".into()) };
    let name = CheckpointName::parse(name).ok_or_else(|| "invalid arguments".to_string())?;
    Ok((directories, name))
}

pub fn run(
    binary: &str,
    directories: &Directories,
    name: CheckpointName,
    build_name: &'static str,
) -> Result<u8, String> {
    let (mut service, deployment) =
        stopped::prepare(binary, directories).map_err(|why| prefixed(&format!("{binary} restore"), &why))?;
    service.engine_component().build_name(build_name);
    run_prepared(binary, directories, name, service, &deployment)
}

/// The box prepares its standalone engine without constructing a worker.
pub fn run_prepared(
    binary: &str,
    directories: &Directories,
    name: CheckpointName,
    mut service: service::Service,
    deployment: &[u8],
) -> Result<u8, String> {
    let mut owned = Vec::new();
    let result = (|| {
        let directory = skein_shell::open_root(path(deployment)).map_err(|errno| {
            format!("{}: no readable deployment directory: errno {errno}", path(deployment).display())
        })?;
        owned.push(directory);
        let drawn = skein_shell::crypto_seed().map_err(|errno| format!("restore draw: errno {errno}"))?;
        let mut draw = [0; 16];
        draw.copy_from_slice(drawn.expose().get(..16).expect("32-byte draw"));
        service
            .engine_component()
            .restore_store(directory, skein_shell::effective_user(), name, draw)
            .expect("new restore adopts root");
        stopped::drive(binary, directories, service, deployment, report, true)
    })();
    if result.is_err() {
        temper_service_shell::close(owned);
    }
    result.map_err(|why| prefixed(&format!("{binary} restore"), &why))
}

fn report(service: &service::Service, binary: &str, _: &Directories, deployment: &[u8]) -> Option<(String, u8)> {
    let report = service.upgrade_report()?;
    let name = report.checkpoint?;
    report.events.iter().find_map(|event| {
        if let Event::Restored { restored, .. } = event {
            Some(lines(binary, name, Ok(restored), deployment))
        } else if let Event::RestoreRefused { why, .. } = event {
            Some(lines(binary, name, Err(why), deployment))
        } else {
            None
        }
    })
}

/// One terminal's exact line and exit, shared by both deployment binaries.
#[must_use]
pub fn lines(
    binary: &str,
    name: CheckpointName,
    terminal: Result<&Restored, &RestoreRefusal>,
    deployment: &[u8],
) -> (String, u8) {
    let prefix = format!("{binary} restore");
    match terminal {
        Ok(restored) => {
            let name = restored.name.bytes();
            let safety = restored.safety.bytes();
            let safety = String::from_utf8_lossy(&safety);
            (
                format!(
                    "{prefix}: restored {} as generation {}: {} files linked, {} copied ({} bytes) in {} ms; the state before it is checkpoint {safety}, and restore {safety} brings it back\n",
                    String::from_utf8_lossy(&name),
                    restored.generation,
                    restored.linked,
                    restored.copied,
                    restored.bytes,
                    restored.took / 1_000_000,
                ),
                0,
            )
        }
        Err(RestoreRefusal::Busy) => (
            format!(
                "{prefix}: {} is in use by another process; stop the engine before restoring\n",
                path(deployment).display()
            ),
            1,
        ),
        Err(RestoreRefusal::Source { why: CheckpointRefusal::InUse }) => {
            (format!("{prefix}: {} is in use\n", String::from_utf8_lossy(&name.bytes())), 1)
        }
        Err(RestoreRefusal::Source { why: CheckpointRefusal::Missing }) => {
            (format!("{prefix}: no checkpoint {}\n", String::from_utf8_lossy(&name.bytes())), 1)
        }
        Err(RestoreRefusal::Open { why }) => {
            crate::lines::store_refusal(binary, service::refused_start(why, deployment))
        }
        Err(why) => (format!("{prefix}: refused: {}\n", why.words()), 1),
    }
}
