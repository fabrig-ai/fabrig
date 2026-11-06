//! Terminal program login without deployment resources (protocol/credentials.md, 3).
//! A pre-exec failure adds one `temper program-login:` line. Smith's checked
//! configuration reader may first emit its own secret-free `smith:` diagnostics.
use std::{
    ffi::OsStr,
    os::unix::{ffi::OsStrExt, process::CommandExt},
};

fn source_name(name: &OsStr) -> Result<&str, String> {
    smith_host_domain::SourceName::new(name.as_bytes())
        .ok_or_else(|| smith_agent_shell::program::Refusal::Name.to_string())?;
    Ok(name.to_str().expect("validated ASCII source name"))
}

fn command(
    directories: &super::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<std::process::Command, String> {
    let name = source_name(name)?;
    let bytes = directories.read()?;
    let agent = match temper_service::startup::shape(&bytes, &directories.jig())
        .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?
    {
        temper_service::startup::Shape::WorkerAlone => {
            let part = temper_service::startup::worker_part(&bytes, &directories.jig())
                .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?;
            super::agent::preflight_worker(&part.named)?
        }
        temper_service::startup::Shape::Standalone | temper_service::startup::Shape::EngineAlone => {
            let super::setup::Deployment { named, .. } = super::setup::check(&bytes, &directories.jig())?;
            super::agent::preflight(&named)?
        }
    };
    let program = agent
        .programs
        .iter()
        .find(|program| program.adapter == temper_service::settings::Adapter::ClaudeCode)
        .ok_or("workers[0].agent.configuration: programs.claude_code: not configured")?;
    if super::agent::launcher(super::path(&program.directory)).as_deref() == Some(super::path(&program.path)) {
        super::claude_code::place(
            super::path(&program.path).parent().expect("launcher parent"),
            super::path(&directories.cache),
        )
        .map_err(|why| format!("programs.claude_code: {why}"))?;
    }
    let source = smith_agent_shell::program::prepare(super::path(&program.directory), name)
        .map_err(|why| super::agent::program_directory_refusal(&why))?;
    Ok(smith_agent_shell::program::sign_in(super::path(&program.path), &source, environment))
}

pub(crate) fn run(directories: &super::Directories, name: &OsStr) -> Result<u8, String> {
    let error = command(directories, name, std::env::vars_os())?.exec();
    Err(format!("exec: {error}"))
}

/// The box's POSIX command, without NAME (client/README.md, 4.6, *The printed command*).
pub(crate) fn line(directories: &super::Directories, binary: &std::path::Path) -> Option<String> {
    use std::fmt::Write;
    let mut command = format!("'{}' program-login", binary.to_str()?.replace('\'', "'\\''"));
    for (flag, path) in [("config", &directories.config), ("data", &directories.data), ("cache", &directories.cache)] {
        let path = std::str::from_utf8(path).ok()?;
        write!(command, " --{flag} '{}'", path.replace('\'', "'\\''")).expect("string");
    }
    if command.len() > 8192 {
        return None;
    }
    Some(command)
}

#[cfg(test)]
mod tests;
