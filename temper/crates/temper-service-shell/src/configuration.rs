//! Binary-shaped configuration with the temporary legacy branch
//! (temper/docs/design/protocol/configuration/file.md, 6).
use std::{io::ErrorKind, os::unix::ffi::OsStrExt, path::Path};
use temper_config_shell::{Keys, Shape};
use temper_shell::directories::Directories;

pub struct Read {
    pub document: Box<[u8]>,
    pub keys: Option<Keys>,
}

/// The legacy branch lives here alone; a present TOML file always uses the table's reader.
pub fn read(directories: &Directories, shape: Shape) -> Result<Read, String> {
    let read = read_with_legacy(directories, shape, || directories.read())?;
    if read.keys.is_some() {
        directories.open(skein_shell::effective_user())?;
    }
    Ok(read)
}

/// Offline readers keep their legacy io and diagnostics, without opening data directories.
pub fn read_with_legacy(
    directories: &Directories,
    shape: Shape,
    legacy: impl FnOnce() -> Result<Box<[u8]>, String>,
) -> Result<Read, String> {
    let file = Path::new(std::ffi::OsStr::from_bytes(&directories.config)).join(temper_config_shell::FILE);
    if let Err(why) = std::fs::symlink_metadata(file)
        && why.kind() == ErrorKind::NotFound
    {
        return legacy().map(|document| Read { document, keys: None });
    }
    let temper_config_shell::Read { document, keys, .. } =
        temper_config_shell::read(directories, shape).map_err(|why| why.to_string())?;
    Ok(Read { document, keys: Some(keys) })
}

/// Program-login keeps its legacy single-line output, but file refusals are one per line.
#[must_use]
pub fn login_problems(command: &str, lines: &str) -> String {
    if lines.lines().any(|line| {
        line.starts_with(&format!("{}:", temper_config_shell::FILE))
            || line.starts_with(&format!("{}: moved:", temper_config_shell::FORMER))
    }) {
        crate::prefixed(command, lines)
    } else {
        crate::prefixed(command, &lines.lines().collect::<Vec<_>>().join("; "))
    }
}

impl Read {
    /// The shells already render member paths and the start's words. Translate at the
    /// output boundary, leaving lines outside the configuration and their endings intact.
    #[must_use]
    pub fn problems(&self, lines: &str) -> String {
        let Some(keys) = &self.keys else { return lines.into() };
        let mut rendered = String::new();
        for line in lines.split_inclusive('\n') {
            let (line, ending) = line.strip_suffix('\n').map_or((line, ""), |line| (line, "\n"));
            match line.split_once(": ") {
                Some((member, words)) => rendered.push_str(&keys.render_member(member, words)),
                None => rendered.push_str(line),
            }
            rendered.push_str(ending);
        }
        rendered
    }

    pub fn prepare<T>(&self, prepare: impl FnOnce(&[u8]) -> Result<T, String>) -> Result<T, String> {
        prepare(&self.document).map_err(|why| self.problems(&why))
    }
}
