//! A program source's sign-in on the person's terminal (smith/docs/design/protocol/programs.md,
//! 4.1 and 4.2): its private directories and its command. Keeps no login;
//! `prepare` checks directories and `sign_in` leaves execution to the caller.

use std::{
    ffi::{OsStr, OsString},
    os::unix::{ffi::OsStrExt, fs::DirBuilderExt},
    path::{Path, PathBuf},
    process::Stdio,
};

#[expect(
    clippy::disallowed_types,
    reason = "The caller owns terminal sign-in, outside hosted process I/O (protocol/programs.md, 4.1)."
)]
use std::process::Command;

use skein_io::kernel::{Kind, Stat};
use smith_protocol_program::{ANTHROPIC_DIRECTORY, Error, Programs, SIGN_IN, configuration_directory, keeps};

/// Why a source's directories were refused before its program ran.
#[derive(Debug)]
pub enum Refusal {
    /// The source's name is outside 1 to 32 of `a-z`, `0-9` and `-`.
    Name,
    /// The programs directory is not absolute or contains NUL.
    Relative(PathBuf),
    /// A directory exists but is not private: see `Unsafe`.
    Unsafe { path: PathBuf, found: Unsafe },
    /// A directory could not be read or made.
    Io { path: PathBuf, error: std::io::Error },
}

/// What makes an existing directory unsafe (`skein/docs/design/runtime/io/files.md`, 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsafe {
    /// A symbolic link, not a directory reached directly.
    Link,
    /// Not a directory.
    NotADirectory,
    /// Another user's.
    Owner { found: u32 },
    /// Open to group or others: the mode's bits past `0o700`.
    Mode(u32),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name => formatter.write_str("source name must be 1 to 32 of a-z, 0-9 and -"),
            Self::Relative(path) => {
                write!(formatter, "{}: programs directory must be absolute without NUL", path.display())
            }
            Self::Unsafe { path, found } => {
                write!(formatter, "{}: ", path.display())?;
                match found {
                    Unsafe::Link => formatter.write_str("symbolic link"),
                    Unsafe::NotADirectory => formatter.write_str("not a directory"),
                    Unsafe::Owner { found } => write!(formatter, "owned by user {found}"),
                    Unsafe::Mode(mode) => write!(formatter, "nonprivate mode bits {mode:#o}"),
                }
            }
            Self::Io { path, error } => write!(formatter, "{}: {error}", path.display()),
        }
    }
}

/// Check an existing directory's kind, owner and mode; `Ok(false)` means absent.
pub fn private_directory(path: &Path) -> Result<bool, Refusal> {
    private_directory_with(path, skein_shell::effective_user(), inspect_at(path, skein_shell::symlink_stat))
}

fn inspect_at(path: &Path, inspect: impl FnOnce(&Path) -> std::io::Result<Stat>) -> std::io::Result<Stat> {
    let inspected: PathBuf = path.components().collect();
    inspect(&inspected)
}

/// Check supplied no-follow metadata; a not-found result means the name is absent.
pub fn private_directory_with(path: &Path, user: u32, metadata: std::io::Result<Stat>) -> Result<bool, Refusal> {
    let metadata = match metadata {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(Refusal::Io { path: path.into(), error }),
    };
    let found = if metadata.kind == Kind::Symlink {
        Some(Unsafe::Link)
    } else if metadata.kind != Kind::Directory {
        Some(Unsafe::NotADirectory)
    } else if metadata.owner != user {
        Some(Unsafe::Owner { found: metadata.owner })
    } else if metadata.mode & 0o077 != 0 {
        Some(Unsafe::Mode(metadata.mode & 0o077))
    } else {
        None
    };
    match found {
        Some(found) => Err(Refusal::Unsafe { path: path.into(), found }),
        None => Ok(true),
    }
}

/// Make and recheck the private programs and source directories, refusing names first.
pub fn prepare(directory: &Path, source: &str) -> Result<PathBuf, Refusal> {
    prepare_with(directory, source, skein_shell::effective_user(), skein_shell::symlink_stat, |path, parents, mode| {
        std::fs::DirBuilder::new().recursive(parents).mode(mode).create(path)
    })
}

/// Check both names before making either directory, then recheck each at its name.
pub fn prepare_with(
    directory: &Path,
    source: &str,
    user: u32,
    mut inspect: impl FnMut(&Path) -> std::io::Result<Stat>,
    mut create: impl FnMut(&Path, bool, u32) -> std::io::Result<()>,
) -> Result<PathBuf, Refusal> {
    let programs = Programs { directory: directory.as_os_str().as_bytes().into(), claude: Box::default() };
    let configuration = match configuration_directory(&programs, source.as_bytes()) {
        Ok(path) => PathBuf::from(OsStr::from_bytes(&path)),
        Err(Error::Name) => return Err(Refusal::Name),
        Err(Error::Directory | Error::Environment) => return Err(Refusal::Relative(directory.into())),
    };
    let mut check = |path: &Path| private_directory_with(path, user, inspect_at(path, &mut inspect));
    let programs_present = check(directory)?;
    let source_present = check(&configuration)?;
    if !programs_present {
        made(directory, create(directory, true, 0o700))?;
    }
    require_directory(directory, check(directory)?)?;
    if !source_present {
        made(&configuration, create(&configuration, false, 0o700))?;
    }
    require_directory(&configuration, check(&configuration)?)?;
    Ok(configuration)
}

fn require_directory(path: &Path, present: bool) -> Result<(), Refusal> {
    if present {
        Ok(())
    } else {
        Err(Refusal::Io { path: path.into(), error: std::io::Error::from(std::io::ErrorKind::NotFound) })
    }
}

fn made(path: &Path, result: std::io::Result<()>) -> Result<(), Refusal> {
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(Refusal::Io { path: path.into(), error }),
    }
}

/// Build `<program> auth login` in the filtered terminal environment with inherited streams.
#[must_use]
#[expect(
    clippy::disallowed_types,
    reason = "The caller execs or waits on terminal sign-in (protocol/programs.md, 4.1)."
)]
pub fn sign_in(program: &Path, configuration: &Path, base: impl IntoIterator<Item = (OsString, OsString)>) -> Command {
    let mut command = Command::new(program);
    command.env_clear();
    for (name, value) in base {
        if keeps(name.as_bytes()) {
            command.env(name, value);
        }
    }
    for argument in SIGN_IN {
        command.arg(OsStr::from_bytes(argument));
    }
    command
        .env("CLAUDE_CONFIG_DIR", configuration)
        .env("ANTHROPIC_CONFIG_DIR", configuration.join(ANTHROPIC_DIRECTORY))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
}

#[cfg(test)]
mod tests;
