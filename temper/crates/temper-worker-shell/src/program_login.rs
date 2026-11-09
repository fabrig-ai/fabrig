//! Terminal program login without deployment resources (protocol/credentials.md, 3; protocol/README.md, 4.5).
//! The caller prefixes a pre-exec failure with its program-login command. Smith's checked
//! configuration reader may first emit its own secret-free `smith:` diagnostics.
use std::{
    ffi::OsStr,
    os::unix::{ffi::OsStrExt, process::CommandExt},
};
use temper_service_shell::jig_directories;
use temper_shell::{
    commands::{Argument, DIRECTORIES, Row, Who},
    directories,
};

pub const PROGRAM_LOGIN: Row = Row {
    words: &["program-login"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES] NAME"],
    line: "Sign a program source in on this host, through the program's own sign-in.",
    arguments: &[
        DIRECTORIES,
        Argument { shown: "NAME", line: "The program source's name: 1 to 32 of a to z, 0 to 9 and -." },
    ],
    settings: &[&directories::SETTINGS],
    notes: "Checks the configuration and the agent's, makes the source's directory, then replaces itself by the program's sign-in, whose exit status is then its own. The engine need not run.",
};

/// The worker of a document that is not a worker alone, as the box's setup check reads it.
pub type Deployment =
    fn(&[u8], &jig_protocol_config::Directories) -> Result<temper_service::startup::NamedWorker, String>;

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
    let worker = match temper_service::startup::shape(&bytes, &jig_directories(directories))
        .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?
    {
        temper_service::startup::Shape::WorkerAlone => {
            let part = temper_service::startup::worker_part(&bytes, &jig_directories(directories))
                .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?;
            part.named
        }
        temper_service::startup::Shape::Standalone | temper_service::startup::Shape::EngineAlone => {
            deployment.ok_or("temper.json holds no worker alone")?(&bytes, &jig_directories(directories))?
        }
    };
    let agent = super::agent::preflight_worker(&worker)?;
    let program = agent
        .programs
        .iter()
        .find(|program| program.adapter == temper_service::settings::Adapter::ClaudeCode)
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

#[expect(
    clippy::disallowed_methods,
    reason = "cli C6: the process environment configures program sign-in; temper-worker's and temper-box's tables will pass it"
)]
pub fn run(directories: &directories::Directories, name: &OsStr, deployment: Option<Deployment>) -> Result<u8, String> {
    let error = command(directories, name, std::env::vars_os(), deployment)?.exec();
    Err(format!("exec: {error}"))
}

#[cfg(test)]
mod tests;
