//! The box's deployment reader and printed program-login command (protocol/binaries.md, 5).
pub(crate) fn deployment(
    bytes: &[u8],
    directories: &jig_protocol_config::Directories,
) -> Result<temper_service::startup::NamedWorker, String> {
    let super::setup::Deployment { named, .. } = super::setup::check(bytes, directories)?;
    named.worker.ok_or_else(|| "workers: standalone takes one worker".into())
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
