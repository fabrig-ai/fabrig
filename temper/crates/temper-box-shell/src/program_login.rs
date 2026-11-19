//! The box's deployment reader and printed program-login command (protocol/binaries.md, 5).
/// The box checks its document but needs no running engine for a program sign-in.
pub(crate) fn command(
    directories: &super::Directories,
    name: &std::ffi::OsStr,
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<std::process::Command, String> {
    use std::os::unix::ffi::OsStrExt;
    let environment: Vec<_> = environment.into_iter().collect();
    let own: Vec<super::EnvironmentEntry> =
        environment.iter().map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into())).collect();
    let read = temper_service_shell::configuration::read(directories, temper_config_shell::Shape::Standalone)?;
    read.prepare(|bytes| {
        let local =
            crate::serve::local_setting(&crate::path(&directories.data).join("local.setting"), &mut std::io::stderr());
        let places = local.as_ref().map_or(4, jig_client_protocol_wire::WorkerSetting::places);
        let mut part = temper_service::startup::box_worker(
            bytes,
            &temper_service_shell::jig_directories(directories),
            &temper_service::stock::Given::Stock,
            &temper_service::stock::agents(),
            places,
            &own,
        )
        .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())?
        .ok_or("workers: standalone takes one worker")?;
        if let Some(claude) = crate::fake_backends::configured_claude(directories)? {
            part.claude = Some(claude.as_os_str().as_bytes().into());
        }
        let executable = std::env::current_exe().map_err(|why| why.to_string())?;
        temper_worker_shell::program_login::from_machine(
            &part,
            directories,
            name.to_str().ok_or("source name must be ASCII")?,
            environment,
            &executable,
        )
    })
}
pub(crate) fn run(
    directories: &super::Directories,
    name: &std::ffi::OsStr,
    environment: &[super::EnvironmentEntry],
) -> Result<u8, String> {
    use std::os::unix::ffi::OsStrExt;
    let read = temper_service_shell::configuration::read(directories, temper_config_shell::Shape::Standalone)?;
    if read.prepare(|bytes| {
        temper_service::startup::shape(bytes, &temper_service_shell::jig_directories(directories))
            .map_err(|why| String::from_utf8_lossy(&temper_service::startup::lines(&why)).into_owned())
    })? == temper_service::startup::Shape::WorkerAlone
    {
        return temper_worker_shell::program_login::run(directories, name, environment, None);
    }
    let environment = environment.iter().map(|(name, value)| {
        (std::ffi::OsStr::from_bytes(name).to_os_string(), std::ffi::OsStr::from_bytes(value).to_os_string())
    });
    temper_worker_shell::program_login::run_command(command(directories, name, environment)?)
}
/// The box's POSIX command, without NAME (client/forge/records.md, *The printed command*).
pub(crate) fn line(directories: &super::Directories, binary: &[u8], home: Option<&[u8]>) -> Option<String> {
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
