//! The binary's three directories (protocol/configuration/directories.md).
use skein_io::kernel::{Error, Kind, Stat};
use skein_shell::{self as settings, Base, Fallback, Origin, Setting, Settings};
use std::{fmt, io, os::unix::fs::DirBuilderExt, path::Path};

pub const SETTINGS: Settings = Settings {
    tool: "temper",
    about: "The deployment's three directories.",
    table: &[
        Setting {
            name: "config",
            value: "CONFIG",
            about: "Configuration files.",
            fallback: Fallback::Base(Base::Config),
            set: skein_shell::Set::Public,
            twin: Some("TEMPER_CONFIG"),
            notes: "",
        },
        Setting {
            name: "data",
            value: "DATA",
            about: "The deployment's data and secrets.",
            fallback: Fallback::Base(Base::Data),
            set: skein_shell::Set::Public,
            twin: Some("TEMPER_DATA"),
            notes: "",
        },
        Setting {
            name: "cache",
            value: "CACHE",
            about: "Binaries that can be fetched again.",
            fallback: Fallback::Base(Base::Cache),
            set: skein_shell::Set::Public,
            twin: Some("TEMPER_CACHE"),
            notes: "",
        },
    ],
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directories {
    pub config: Box<[u8]>,
    pub data: Box<[u8]>,
    pub cache: Box<[u8]>,
    pub origins: [Origin; 3],
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Settings(settings::Refusal),
    Bound(&'static str),
}
impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(why) => why.fmt(out),
            Self::Bound(name) => write!(out, "--{name} DIR: absolute path is past 4096 bytes"),
        }
    }
}
pub type Argument = Box<[u8]>;
pub type EnvironmentEntry = (Argument, Argument);
#[expect(
    clippy::result_large_err,
    reason = "skein/docs/design/runtime/shell-startup.md, 1.5 preserves the refused setting and its documentation"
)]
pub fn resolve(
    arguments: &[Argument],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Directories, Box<[Argument]>), Refusal> {
    let resolved = settings::resolve(&SETTINGS, arguments, environment).map_err(Refusal::Settings)?;
    let mut paths = [Vec::new(), Vec::new(), Vec::new()];
    for (row, (setting, absolute)) in SETTINGS.table.iter().zip(&mut paths).enumerate() {
        let value = &resolved.value(row).bytes;
        if !value.starts_with(b"/") {
            absolute.extend_from_slice(working);
            if !absolute.ends_with(b"/") {
                absolute.push(b'/');
            }
        }
        absolute.extend_from_slice(value);
        if absolute.len() > 4096 {
            return Err(Refusal::Bound(setting.name));
        }
    }
    let [config, data, cache] = paths.map(Vec::into_boxed_slice);
    Ok((
        Directories {
            config,
            data,
            cache,
            origins: [resolved.value(0).origin, resolved.value(1).origin, resolved.value(2).origin],
        },
        resolved.rest().into(),
    ))
}
/// The directory paths resolved for any binary.
pub type Paths = Directories;

impl Directories {
    #[must_use]
    pub fn flags(&self) -> Box<[Argument]> {
        vec![
            Box::from(b"--config".as_slice()),
            self.config.clone(),
            Box::from(b"--data".as_slice()),
            self.data.clone(),
            Box::from(b"--cache".as_slice()),
            self.cache.clone(),
        ]
        .into_boxed_slice()
    }
    pub fn open(&self, user: u32) -> Result<(), String> {
        for directory in [&self.data, &self.cache] {
            make_private(super::path(directory), user)?;
        }
        Ok(())
    }
}
pub fn private_root(path: &Path, user: u32) -> Result<(), String> {
    private_root_with(user, skein_shell::symlink_stat(path))
}

/// Check a root using metadata from its name, not from a link target.
pub fn private_root_with(user: u32, metadata: std::io::Result<Stat>) -> Result<(), String> {
    let metadata = metadata.map_err(|why| why.to_string())?;
    if metadata.kind != Kind::Directory || metadata.owner != user || metadata.mode & 0o077 != 0 {
        return Err("must be a private directory of this user's, not reached through a link".into());
    }
    Ok(())
}
pub fn make_private(path: &Path, user: u32) -> Result<(), String> {
    make_private_with(path, user, &mut Native)
}

/// Facts and effects for preparing a private root (temper/docs/design/testing/shells.md, 3.11).
pub(crate) trait PrivateDirectories {
    fn symlink_stat(&mut self, path: &Path) -> io::Result<Stat>;
    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()>;
}

struct Native;

impl PrivateDirectories for Native {
    fn symlink_stat(&mut self, path: &Path) -> io::Result<Stat> {
        skein_shell::symlink_stat(path)
    }

    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        std::fs::DirBuilder::new().recursive(true).mode(mode).create(path)
    }
}

/// Check the root at its final name before and after preparing it.
pub(crate) fn make_private_with(path: &Path, user: u32, disk: &mut impl PrivateDirectories) -> Result<(), String> {
    let named = |why| format!("{}: {why}", path.display());
    match disk.symlink_stat(path) {
        Ok(_) => {}
        Err(why) if why.kind() == io::ErrorKind::NotFound => {
            disk.create_directories(path, 0o700).map_err(|why| named(why.to_string()))?;
        }
        Err(why) => return Err(named(why.to_string())),
    }
    private_root_with(user, disk.symlink_stat(path)).map_err(named)
}

#[must_use]
pub fn errno(error: Error) -> i32 {
    match error {
        Error::Refused => 111,
        Error::Busy => 11,
        Error::Reset => 104,
        Error::BrokenPipe => 32,
        Error::NotConnected => 107,
        Error::AddressInUse => 98,
        Error::AddressNotAvailable => 99,
        Error::Unreachable => 101,
        Error::TimedOut => 110,
        Error::TooManyOpenFiles => 24,
        Error::NoBufferSpace => 12,
        Error::Cancelled => 125,
        Error::TooLate | Error::NotFound => 2,
        Error::Exists => 17,
        Error::NotADirectory => 20,
        Error::IsADirectory => 21,
        Error::NotEmpty => 39,
        Error::Permission => 13,
        Error::NoSpace => 28,
        Error::ReadOnly => 30,
        Error::TooManyLinks => 40,
        Error::NameTooLong => 36,
        Error::Escape => 18,
        Error::NotAFile => 6,
        Error::InvalidArgument => 22,
        Error::Reach(_) => unreachable!("a reach error answers a spawn, never a file or directory operation"),
        Error::Other(number) => number,
    }
}
#[must_use]
pub fn unread(why: skein_shell::Unread) -> String {
    match why {
        skein_shell::Unread::Absent => "absent".into(),
        skein_shell::Unread::NotAFile => "not a file".into(),
        skein_shell::Unread::TooLarge { max } => format!("past {max} bytes"),
        skein_shell::Unread::Unsafe(_) => "readable by others or reached through a link".into(),
        skein_shell::Unread::Kernel(error) => format!("errno {}", errno(error)),
    }
}
#[cfg(test)]
pub mod tests;
