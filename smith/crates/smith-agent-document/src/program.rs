//! A program source's sign-in on the person's terminal (smith/docs/design/protocol/programs.md,
//! 4.1 and 4.2): its private directories and its command. Keeps no login;
//! `prepare` checks directories and `sign_in` leaves execution to the caller.

use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt},
    },
    path::{Path, PathBuf},
    process::Stdio,
};

#[expect(
    clippy::disallowed_types,
    reason = "The caller owns terminal sign-in, outside hosted process I/O (protocol/programs.md, 4.1)."
)]
use std::process::Command;

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

/// What makes an existing directory unsafe (`skein/docs/design/runtime/io/files.md`, 3).
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

#[derive(Clone, Copy)]
enum Kind {
    Directory,
    Link,
    Other,
}

#[derive(Clone, Copy)]
struct Facts {
    kind: Kind,
    owner: u32,
    mode: u32,
}

fn inspect(path: &Path) -> std::io::Result<Facts> {
    let metadata = fs::symlink_metadata(path)?;
    let kind = if metadata.file_type().is_symlink() {
        Kind::Link
    } else if metadata.is_dir() {
        Kind::Directory
    } else {
        Kind::Other
    };
    Ok(Facts { kind, owner: metadata.uid(), mode: metadata.mode() })
}

/// Check an existing directory's kind, owner and mode; `Ok(false)` means absent.
pub fn private_directory(path: &Path) -> Result<bool, Refusal> {
    private_directory_with(path, skein_shell::effective_user, &mut inspect)
}

fn private_directory_with(
    path: &Path,
    effective_user: fn() -> u32,
    inspect: &mut impl FnMut(&Path) -> std::io::Result<Facts>,
) -> Result<bool, Refusal> {
    let inspected: PathBuf = path.components().collect();
    let facts = match inspect(&inspected) {
        Ok(facts) => facts,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(Refusal::Io { path: path.into(), error }),
    };
    let found = match facts.kind {
        Kind::Link => Some(Unsafe::Link),
        Kind::Other => Some(Unsafe::NotADirectory),
        Kind::Directory if facts.owner != effective_user() => Some(Unsafe::Owner { found: facts.owner }),
        Kind::Directory if facts.mode & 0o077 != 0 => Some(Unsafe::Mode(facts.mode & 0o077)),
        Kind::Directory => None,
    };
    match found {
        Some(found) => Err(Refusal::Unsafe { path: path.into(), found }),
        None => Ok(true),
    }
}

/// Make and recheck the private programs and source directories, refusing names first.
pub fn prepare(directory: &Path, source: &str) -> Result<PathBuf, Refusal> {
    prepare_with(directory, source, skein_shell::effective_user, inspect, make_directory)
}

fn prepare_with(
    directory: &Path,
    source: &str,
    effective_user: fn() -> u32,
    mut inspect: impl FnMut(&Path) -> std::io::Result<Facts>,
    mut make_directory: impl FnMut(&Path, bool, u32) -> std::io::Result<()>,
) -> Result<PathBuf, Refusal> {
    let programs = Programs { directory: directory.as_os_str().as_bytes().into(), claude: Box::default() };
    let configuration = match configuration_directory(&programs, source.as_bytes()) {
        Ok(path) => PathBuf::from(OsStr::from_bytes(&path)),
        Err(Error::Name) => return Err(Refusal::Name),
        Err(Error::Directory | Error::Environment) => return Err(Refusal::Relative(directory.into())),
    };
    let mut check = |path: &Path| private_directory_with(path, effective_user, &mut inspect);
    let programs_present = check(directory)?;
    let source_present = check(&configuration)?;
    if !programs_present {
        make(directory, true, &mut make_directory)?;
    }
    require_directory(directory, check(directory)?)?;
    if !source_present {
        make(&configuration, false, &mut make_directory)?;
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

fn make_directory(path: &Path, parents: bool, mode: u32) -> std::io::Result<()> {
    fs::DirBuilder::new().recursive(parents).mode(mode).create(path)
}

fn make(
    path: &Path,
    parents: bool,
    make_directory: &mut impl FnMut(&Path, bool, u32) -> std::io::Result<()>,
) -> Result<(), Refusal> {
    match make_directory(path, parents, 0o700) {
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
