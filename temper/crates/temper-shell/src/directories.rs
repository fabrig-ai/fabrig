//! The binary's three directories (protocol/configuration.md, 12).
use skein_io::kernel::Error;
use skein_shell::{self as settings, Base, Fallback, Origin, Setting, Settings};
use std::{
    fmt,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
};

pub const SETTINGS: Settings = Settings {
    tool: "temper",
    table: &[
        Setting {
            name: "config",
            value: "CONFIG",
            about: "Configuration, including temper.json.",
            fallback: Fallback::Base(Base::Config),
        },
        Setting {
            name: "data",
            value: "DATA",
            about: "The deployment's data and secrets.",
            fallback: Fallback::Base(Base::Data),
        },
        Setting {
            name: "cache",
            value: "CACHE",
            about: "Binaries that can be fetched again.",
            fallback: Fallback::Base(Base::Cache),
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
type Argument = Box<[u8]>;
pub type EnvironmentEntry = (Argument, Argument);
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
impl Directories {
    #[must_use]
    pub fn jig(&self) -> jig_protocol_config::Directories {
        jig_protocol_config::Directories::given(&self.config, &self.data, &self.cache)
            .expect("resolved absolute bounded directories")
    }
    #[must_use]
    pub fn document(&self) -> PathBuf {
        super::path(&self.config).join("temper.json")
    }
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
    /// Missing documents are refused before data or cache is touched.
    pub fn read(&self) -> Result<Box<[u8]>, String> {
        let document = self.document();
        match std::fs::symlink_metadata(&document) {
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!("configuration {}: absent; temper init writes one", document.display()));
            }
            Err(why) => return Err(format!("configuration {}: {why}", document.display())),
            Ok(_) => {}
        }
        self.open(skein_shell::effective_user())?;
        skein_shell::read_file(&document, CONFIG_BYTES)
            .map_err(|why| format!("configuration {}: {}", document.display(), unread(why)))
    }
}
pub fn private_root(path: &Path, user: u32) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|why| why.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.uid() != user || metadata.mode() & 0o077 != 0
    {
        return Err("must be a private directory of this user's, not reached through a link".into());
    }
    Ok(())
}
pub fn make_private(path: &Path, user: u32) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .map_err(|why| format!("{}: {why}", path.display()))?,
        Err(why) => return Err(format!("{}: {why}", path.display())),
    }
    private_root(path, user).map_err(|why| format!("{}: {why}", path.display()))
}
const CONFIG_BYTES: u32 = 1 << 20;

fn errno(error: Error) -> i32 {
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
