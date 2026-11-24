//! The output and fixture directory face (skein/docs/design/testing/fakes/github.md, 2).
mod machine;

use crate::Result;
use skein_fake_machine::{Machine, Opened};
use skein_forge_conformance::write::{self, Secrets};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use machine::Change;
pub use machine::MachineDirectory;

/// Names are relative to the directory; the empty name names the directory itself.
pub enum Directory {
    Host(PathBuf),
    Fake(Box<MachineDirectory>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Directory,
    File,
    Other,
}

pub(crate) struct Metadata {
    pub kind: Kind,
    pub mode: u32,
    pub size: u64,
}

fn at(dir: &Path, name: &str) -> PathBuf {
    // Don't add a trailing slash when checking the directory itself: that would follow a link.
    if name.is_empty() { dir.to_owned() } else { dir.join(name) }
}

impl Directory {
    #[must_use]
    pub fn fake(machine: Machine, root: Opened, path: &str) -> Self {
        Self::Fake(Box::new(MachineDirectory::new(machine, root, path)))
    }

    pub(crate) fn host_path(&self) -> Option<&Path> {
        match self {
            Self::Host(path) => Some(path),
            Self::Fake(_) => None,
        }
    }

    pub(crate) fn create_dir_all(&mut self, name: &str) -> Result<()> {
        match self {
            Self::Host(dir) => fs::create_dir_all(at(dir, name))?,
            Self::Fake(dir) => dir.create_dir_all(name)?,
        }
        Ok(())
    }

    fn remove_dir_all(&mut self, name: &str) -> Result<()> {
        match self {
            Self::Host(dir) => fs::remove_dir_all(at(dir, name))?,
            Self::Fake(dir) => dir.remove_dir_all(name)?,
        }
        Ok(())
    }

    pub fn exists(&mut self, name: &str) -> bool {
        match self {
            Self::Host(dir) => at(dir, name).exists(),
            Self::Fake(dir) => dir.exists(name),
        }
    }

    pub(crate) fn metadata(&mut self, name: &str) -> Result<Metadata> {
        Ok(match self {
            Self::Host(dir) => {
                let meta = fs::symlink_metadata(at(dir, name))?;
                let kind = if meta.is_dir() {
                    Kind::Directory
                } else if meta.is_file() {
                    Kind::File
                } else {
                    Kind::Other
                };
                Metadata { kind, mode: meta.permissions().mode() & 0o7777, size: meta.len() }
            }
            Self::Fake(dir) => dir.metadata(name)?,
        })
    }

    pub fn read_bounded(&mut self, name: &str, limit: u64) -> Result<Vec<u8>> {
        match self {
            Self::Host(dir) => {
                let mut bytes = Vec::new();
                File::open(at(dir, name))?.take(limit).read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            Self::Fake(dir) => Ok(dir.read(name, u32::try_from(limit)?)?),
        }
    }

    pub fn read_text(&mut self, name: &str) -> std::io::Result<String> {
        match self {
            Self::Host(dir) => fs::read_to_string(at(dir, name)),
            Self::Fake(dir) => match String::from_utf8(dir.read(name, u32::MAX)?) {
                Ok(text) => Ok(text),
                Err(error) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, error.utf8_error())),
            },
        }
    }

    pub fn write(&mut self, name: &str, bytes: &[u8]) -> Result<()> {
        match self {
            Self::Host(dir) => fs::write(at(dir, name), bytes)?,
            Self::Fake(dir) => dir.change(name, Change::Replace(bytes))?,
        }
        Ok(())
    }

    /// Exclusively create a file in this directory, syncing the file and then the directory.
    pub(crate) fn create_synced(&mut self, name: &str, bytes: &[u8]) -> Result<()> {
        match self {
            Self::Host(dir) => {
                let mut file = OpenOptions::new().write(true).create_new(true).open(at(dir, name))?;
                file.write_all(bytes)?;
                file.sync_all()?;
                File::open(dir)?.sync_all()?;
            }
            Self::Fake(dir) => {
                dir.change(name, Change::Create(bytes))?;
                dir.sync("")?;
            }
        }
        Ok(())
    }

    pub(crate) fn append_synced(&mut self, name: &str, line: &str) -> Result<()> {
        match self {
            Self::Host(dir) => {
                let mut file = OpenOptions::new().append(true).open(at(dir, name))?;
                writeln!(file, "{line}")?;
                file.sync_all()?;
            }
            Self::Fake(dir) => dir.change(name, Change::Append(format!("{line}\n").as_bytes()))?,
        }
        Ok(())
    }

    pub(crate) fn truncate_synced(&mut self, name: &str, size: u64) -> Result<()> {
        match self {
            Self::Host(dir) => {
                let file = OpenOptions::new().write(true).open(at(dir, name))?;
                file.set_len(size)?;
                file.sync_all()?;
            }
            Self::Fake(dir) => dir.change(name, Change::Truncate(size))?,
        }
        Ok(())
    }

    pub(crate) fn sync(&mut self, name: &str) -> Result<()> {
        match self {
            Self::Host(dir) => File::open(at(dir, name))?.sync_all()?,
            Self::Fake(dir) => dir.sync(name)?,
        }
        Ok(())
    }
}

/// Scan both streams before replacing the case directory or writing any bytes.
pub fn transcript(out: &mut Directory, case: &str, request: &[u8], response: &[u8], secrets: &Secrets) -> Result<()> {
    write::check(case, request, response, secrets)?;
    let dir = format!("transcripts/{case}");
    if out.exists(&dir) {
        out.remove_dir_all(&dir)?;
    }
    out.create_dir_all(&dir)?;
    out.write(&format!("{dir}/request"), request)?;
    out.write(&format!("{dir}/response"), response)
}
