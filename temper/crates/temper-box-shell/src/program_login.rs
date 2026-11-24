//! The box's deployment reader and printed program-login command (protocol/binaries.md, 5).
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};
use temper_service::startup::WorkerPart;
use temper_service_shell::configuration::{self, Read};

/// The box checks its document but needs no running engine for a program sign-in.
pub(crate) fn command(
    directories: &super::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
) -> Result<Command, String> {
    command_with(
        directories,
        name,
        environment,
        || configuration::read(directories, temper_config_shell::Shape::Standalone),
        temper_shell::lookup::is_executable,
        || {
            let copy = crate::path(&directories.data).join("local.setting");
            crate::serve::local_setting(&copy, &mut std::io::stderr()).as_ref().map_or(4, |local| local.places)
        },
        || crate::fake_backends::configured_claude(directories),
        std::env::current_exe,
        |part, name, environment, executable| {
            temper_worker_shell::program_login::from_machine(part, directories, name, environment, executable)
        },
    )
}

#[expect(clippy::too_many_arguments, reason = "the program login step takes each startup lookup at its call site")]
fn command_with(
    directories: &super::Directories,
    name: &OsStr,
    environment: impl IntoIterator<Item = (OsString, OsString)>,
    read_configuration: impl FnOnce() -> Result<Read, String>,
    is_executable: impl Fn(&Path) -> bool,
    places: impl FnOnce() -> u32,
    configured_claude: impl FnOnce() -> Result<Option<PathBuf>, String>,
    current_exe: impl FnOnce() -> std::io::Result<PathBuf>,
    from_part: impl FnOnce(&WorkerPart, &str, Vec<(OsString, OsString)>, &Path) -> Result<Command, String>,
) -> Result<Command, String> {
    let environment: Vec<_> = environment.into_iter().collect();
    let own: Vec<super::EnvironmentEntry> =
        environment.iter().map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into())).collect();
    let read = read_configuration()?;
    let programs = read.programs_with(is_executable)?;
    read.prepare(|bytes| {
        let mut part = temper_service::startup::box_worker(
            bytes,
            &temper_service_shell::jig_directories(directories),
            &temper_service::stock::Given::Stock,
            &temper_service::stock::agents(),
            places(),
            &own,
        )
        .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?
        .ok_or("workers: standalone takes one worker")?;
        let claude = match programs.claude {
            Some(claude) => Some(claude),
            None => configured_claude()?,
        };
        if let Some(claude) = claude {
            part.claude = Some(claude.as_os_str().as_bytes().into());
        }
        let executable = current_exe().map_err(|why| why.to_string())?;
        from_part(&part, name.to_str().ok_or("source name must be ASCII")?, environment, &executable)
    })
}

pub(crate) fn run(
    directories: &super::Directories,
    name: &OsStr,
    environment: &[super::EnvironmentEntry],
) -> Result<u8, String> {
    let environment = environment
        .iter()
        .map(|(name, value)| (OsStr::from_bytes(name).to_os_string(), OsStr::from_bytes(value).to_os_string()));
    run_with(|| command(directories, name, environment), temper_worker_shell::program_login::run_command)
}

fn run_with(
    prepare: impl FnOnce() -> Result<Command, String>,
    run: impl FnOnce(Command) -> Result<u8, String>,
) -> Result<u8, String> {
    prepare().and_then(run).map_err(|why| configuration::login_problems("temper-box program-login", &why))
}

/// The box's POSIX command, without NAME (client/forge/records.md, *The printed command*).
#[must_use]
pub fn line(directories: &super::Directories, binary: &[u8], home: Option<&[u8]>) -> Option<String> {
    use std::fmt::Write;
    let mut command = format!("{} program-login", std::str::from_utf8(binary).ok()?);
    for (flag, path) in [("config", &directories.config), ("data", &directories.data), ("cache", &directories.cache)] {
        std::str::from_utf8(path).ok()?;
        let word = super::printed::path_word(path, home);
        write!(command, " --{flag} {}", std::str::from_utf8(&word).ok()?).expect("string");
    }
    if command.len() > 8192 {
        return None;
    }
    Some(command)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod path_tests;
