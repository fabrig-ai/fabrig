//! Terminal program login without deployment resources (protocol/credentials.md, 3; protocol/binaries.md, 5).
//! The caller prefixes a pre-exec failure with its program-login command. Smith's checked
//! configuration reader may first emit its own secret-free `smith:` diagnostics.
use skein_io::kernel::{Error, Fd};
use skein_shell::{Argument, Command, Reach, Set};
use std::{
    ffi::OsStr,
    io::IsTerminal,
    os::unix::{ffi::OsStrExt, process::CommandExt},
};
use temper_service_shell::jig_directories;
use temper_shell::directories::{self, EnvironmentEntry};

pub const PROGRAM_LOGIN: Command = Command {
    reach: Reach::Words(&["program-login"]),
    set: Set::Public,
    forms: &["[DIRECTORIES] NAME"],
    line: "Sign a program source in on this host, through the program's own sign-in.",
    arguments: &[
        Argument { shown: temper_shell::commands::DIRECTORIES.shown, line: temper_shell::commands::DIRECTORIES.line },
        Argument { shown: "NAME", line: "The program source's name: 1 to 32 of a to z, 0 to 9 and -." },
    ],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[],
    notes: "Checks the configuration and the agent's, makes the source's directory, then replaces itself by the program's sign-in, whose exit status is then its own. The engine need not run.",
};

/// The worker of a document that is not a worker alone, as the box's setup check reads it.
pub type Deployment =
    fn(&[u8], &jig_protocol_config::Directories) -> Result<temper_worker_service::startup::NamedWorker, String>;

fn source_name(name: &OsStr) -> Result<&str, String> {
    smith_host_domain::SourceName::new(name.as_bytes())
        .ok_or_else(|| smith_agent_document::program::Refusal::Name.to_string())?;
    Ok(name.to_str().expect("validated ASCII source name"))
}

pub fn command(
    directories: &directories::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
    deployment: Option<Deployment>,
) -> Result<std::process::Command, String> {
    let name = source_name(name)?;
    let bytes = directories.read()?;
    let worker = match temper_service_core::startup::nearest(&bytes) {
        Some(temper_service_core::startup::Shape::WorkerAlone) | None => {
            let part =
                temper_worker_service::startup::worker_part(&bytes, &jig_directories(directories)).map_err(|why| {
                    String::from_utf8_lossy(&temper_service_core::startup::problems::lines(&why)).into_owned()
                })?;
            part.named
        }
        Some(temper_service_core::startup::Shape::Standalone | temper_service_core::startup::Shape::EngineAlone) => {
            deployment.ok_or("temper.json holds no worker alone")?(&bytes, &jig_directories(directories))?
        }
    };
    let agent = super::agent::preflight_worker(&worker)?;
    let program = agent
        .programs
        .iter()
        .find(|program| program.adapter == temper_service_core::settings::Adapter::ClaudeCode)
        .ok_or("workers[0].agent.configuration: programs.claude_code: not configured")?;
    if super::agent::launcher(super::path(&program.directory)).as_deref() == Some(super::path(&program.path)) {
        temper_launcher::place(
            super::path(&program.path).parent().expect("launcher parent"),
            super::path(&directories.cache),
            super::path(&worker.program),
        )
        .map_err(|why| format!("programs.claude_code: {why}"))?;
    }
    let source = smith_agent_document::program::prepare(super::path(&program.directory), name)
        .map_err(|why| super::agent::program_directory_refusal(&why))?;
    Ok(smith_agent_document::program::sign_in(super::path(&program.path), &source, environment))
}

fn take_terminal(terminal: bool, take: impl FnOnce(Fd) -> Result<(), Error>) -> Result<(), String> {
    if terminal {
        take(Fd::new(0)).map_err(|why| format!("cannot take the terminal: {why:?}"))?;
    }
    Ok(())
}

pub fn run(
    directories: &directories::Directories,
    name: &OsStr,
    environment: &[EnvironmentEntry],
    deployment: Option<Deployment>,
) -> Result<u8, String> {
    take_terminal(std::io::stdin().is_terminal(), skein_shell::take_foreground)?;
    let environment = environment
        .iter()
        .map(|(name, value)| (OsStr::from_bytes(name).to_os_string(), OsStr::from_bytes(value).to_os_string()));
    let error = command(directories, name, environment, deployment)?.exec();
    Err(format!("exec: {error}"))
}

#[cfg(test)]
mod tests;
