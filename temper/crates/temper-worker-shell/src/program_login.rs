//! Terminal program login without deployment resources (protocol/credentials-llm-accounts.md, 1; protocol/binaries.md, 5).
//! The caller prefixes a pre-exec failure with its program-login command. Smith's checked
//! configuration reader may first emit its own secret-free `smith:` diagnostics.
use skein_io::kernel::{Error, Fd};
use skein_shell::{Argument, Command, Reach, Set};
use std::{
    ffi::{OsStr, OsString},
    io::IsTerminal,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
};
use temper_config_shell::Shape;
use temper_service_shell::configuration::{self, Read};
use temper_service_shell::jig_directories;
use temper_shell::directories::{self, EnvironmentEntry};
use temper_shell::lookup::is_executable;
use temper_worker_service::startup::WorkerPart;

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

fn source_name(name: &OsStr) -> Result<&str, String> {
    smith_host_domain::SourceName::new(name.as_bytes())
        .ok_or_else(|| smith_agent_document::program::Refusal::Name.to_string())?;
    Ok(name.to_str().expect("validated ASCII source name"))
}

pub fn command(
    directories: &directories::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
) -> Result<std::process::Command, String> {
    command_with(
        directories,
        name,
        environment,
        |shape| {
            let read = configuration::read(directories, shape)?;
            let programs = read.programs()?;
            Ok((read, programs))
        },
        |part, name, environment| {
            let executable = std::env::current_exe().map_err(|why| why.to_string())?;
            let agent = executable.parent().ok_or("worker executable has no directory")?.join("temper-agent");
            from_machine(part, directories, name, environment, &agent)
        },
    )
}

pub(crate) fn command_with(
    directories: &directories::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
    read_configuration: impl FnOnce(Shape) -> Result<(Read, temper_config_shell::Programs), String>,
    from_part: impl FnOnce(&WorkerPart, &str, Vec<(OsString, OsString)>) -> Result<std::process::Command, String>,
) -> Result<std::process::Command, String> {
    let name = source_name(name)?;
    let environment: Vec<_> = environment.into_iter().collect();
    let (read, programs) = read_configuration(Shape::WorkerAlone)?;
    read.prepare(|bytes| {
        let own: Vec<EnvironmentEntry> =
            environment.iter().map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into())).collect();
        let mut part = temper_worker_service::startup::worker_part(bytes, &jig_directories(directories), None, &own)
            .map_err(|why| {
                String::from_utf8_lossy(&temper_service_core::startup::problems::lines(&why)).into_owned()
            })?;
        if let Some(claude) = &programs.claude {
            part.claude = Some(claude.as_os_str().as_bytes().into());
        }
        from_part(&part, name, environment)
    })
}

/// A box and a worker alone use their worker entry, never a retained agent document.
pub fn from_machine(
    part: &WorkerPart,
    directories: &directories::Directories,
    name: &str,
    environment: Vec<(OsString, OsString)>,
    agent: &Path,
) -> Result<std::process::Command, String> {
    from_machine_with(part, directories, name, environment, is_executable, |programs, program, name| {
        if super::agent::launcher(programs).as_deref() == Some(program) {
            super::worker::executable_file(agent, "the derived agent program", is_executable)?;
            temper_launcher::place(program.parent().expect("launcher parent"), super::path(&directories.cache), agent)
                .map_err(|why| format!("programs.claude_code: {why}"))?;
        }
        smith_agent_document::program::prepare(programs, name)
            .map_err(|why| super::agent::program_directory_refusal(&why))
    })
}

pub(crate) fn from_machine_with(
    part: &WorkerPart,
    directories: &directories::Directories,
    name: &str,
    environment: Vec<(OsString, OsString)>,
    is_executable: impl Fn(&Path) -> bool,
    prepare_source: impl FnOnce(&Path, &Path, &str) -> Result<PathBuf, String>,
) -> Result<std::process::Command, String> {
    let name = source_name(OsStr::new(name))?;
    let captured: Vec<_> =
        environment.iter().map(|(name, value)| (Box::from(name.as_bytes()), Box::from(value.as_bytes()))).collect();
    let program = match part.claude.as_deref() {
        Some(program) => {
            super::worker::executable_file(super::path(program), "application.workers[0].claude", &is_executable)?;
            super::path(program).to_path_buf()
        }
        None => super::worker::find_claude_with(&captured, is_executable).unwrap_or_else(|| {
            super::agent::launcher(&super::path(&directories.data).join("programs")).expect("programs launcher")
        }),
    };
    let programs = super::path(&directories.data).join("programs");
    let source = prepare_source(&programs, &program, name)?;
    Ok(smith_agent_document::program::sign_in(&program, &source, environment))
}
/// Take the terminal immediately before replacing the caller with its prepared command.
pub fn run_command(mut command: std::process::Command) -> Result<u8, String> {
    take_terminal(std::io::stdin().is_terminal(), skein_shell::take_foreground)?;
    Err(format!("exec: {}", command.exec()))
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
) -> Result<u8, String> {
    take_terminal(std::io::stdin().is_terminal(), skein_shell::take_foreground)?;
    let environment = environment
        .iter()
        .map(|(name, value)| (OsStr::from_bytes(name).to_os_string(), OsStr::from_bytes(value).to_os_string()));
    let error = command(directories, name, environment)?.exec();
    Err(format!("exec: {error}"))
}

#[cfg(test)]
mod tests;
