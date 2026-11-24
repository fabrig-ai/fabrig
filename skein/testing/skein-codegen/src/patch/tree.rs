//! The branch's files, separate from the immutable base (skein/docs/design/data/codec/command.md, 8).

use alloc::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{fs, io};

use super::files::{KEPT, PATCHES, SCHEMA};
use super::{Error, Landed};

/// An owner's crate, on disk or in memory. Paths passed to it are relative to the crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    backing: Backing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Backing {
    Disk(PathBuf),
    Memory(BTreeMap<PathBuf, Vec<u8>>),
}

impl Tree {
    /// The owner's directory, read and written as the command goes.
    #[must_use]
    pub fn disk(owner: &Path) -> Self {
        Self { backing: Backing::Disk(owner.to_path_buf()) }
    }

    /// Files in memory, by path relative to the crate.
    #[must_use]
    pub const fn new(files: BTreeMap<PathBuf, Vec<u8>>) -> Self {
        Self { backing: Backing::Memory(files) }
    }

    /// The file's bytes, or `None` if it is absent.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] for a file that cannot be read.
    pub fn read(&self, path: &Path) -> Result<Option<Vec<u8>>, Error> {
        match self.read_file(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Error::io(&self.named(path), &error)),
        }
    }

    /// Every file under `schema/` and `patches/`.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] for a file or directory that cannot be read.
    pub fn files(&self) -> Result<BTreeMap<PathBuf, Vec<u8>>, Error> {
        self.collect(vec![PathBuf::from(PATCHES), PathBuf::from(SCHEMA)])
    }

    /// Writes a file, making its parent directories as needed.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] for a file or directory that cannot be written.
    pub fn write(&mut self, path: &Path, bytes: &[u8]) -> Result<(), Error> {
        if let Some(parent) = path.parent()
            && let Err(error) = self.make_directory(parent)
        {
            return Err(Error::io(&self.named(parent), &error));
        }
        match self.write_file(path, bytes) {
            Ok(()) => Ok(()),
            Err(error) => Err(Error::io(&self.named(path), &error)),
        }
    }

    /// Removes the file if present, pruning emptied golden directories.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] for a file or directory that cannot be removed.
    pub fn remove(&mut self, path: &Path) -> Result<(), Error> {
        match self.remove_file(path) {
            Ok(()) => {
                if let Some(parent) = path.parent() {
                    self.prune_goldens(parent)?;
                }
                Ok(())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(Error::io(&self.named(path), &error)),
        }
    }

    /// The owner's `patches/`, `schema/kept/` and `schema/wires`, as if landed.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] for a file or directory that cannot be read.
    pub fn landed(&self) -> Result<Landed, Error> {
        self.landed_files().map(Landed::new)
    }

    pub(super) fn landed_files(&self) -> Result<BTreeMap<PathBuf, Vec<u8>>, Error> {
        let mut files = self.collect(vec![PathBuf::from(PATCHES), PathBuf::from(KEPT)])?;
        let wires = PathBuf::from("schema/wires");
        if self.exists(&wires)? {
            let bytes = match self.read_file(&wires) {
                Ok(bytes) => bytes,
                Err(error) => return Err(Error::io(&wires, &error)),
            };
            files.insert(wires, bytes);
        }
        Ok(files)
    }

    fn collect(&self, mut directories: Vec<PathBuf>) -> Result<BTreeMap<PathBuf, Vec<u8>>, Error> {
        let mut files = BTreeMap::new();
        while let Some(directory) = directories.pop() {
            if !self.is_dir(&directory) {
                continue;
            }
            let mut entries = self.open_directory(&directory)?;
            while let Some(entry) = entries.next(&directory) {
                let path = match entry {
                    Ok(path) => path,
                    Err(error) => return Err(Error::io(&self.named(&directory), &error)),
                };
                if self.is_dir(&path) {
                    directories.push(path);
                } else {
                    match self.read_file(&path) {
                        Ok(bytes) => files.insert(path, bytes),
                        Err(error) => return Err(Error::io(&self.named(&path), &error)),
                    };
                }
            }
        }
        Ok(files)
    }

    /// Files directly in a directory, in path order; none if it is absent.
    pub(super) fn listed(&self, directory: &Path) -> Result<Vec<PathBuf>, Error> {
        let mut paths = Vec::new();
        if self.is_dir(directory) {
            let mut entries = self.open_directory(directory)?;
            while let Some(entry) = entries.next(directory) {
                let path = match entry {
                    Ok(path) => path,
                    Err(error) => return Err(Error::io(&self.named(directory), &error)),
                };
                if self.is_file(&path) {
                    paths.push(path);
                }
            }
            paths.sort();
        }
        Ok(paths)
    }

    pub(super) fn prune_goldens(&self, directory: &Path) -> Result<(), Error> {
        if let Backing::Disk(root) = &self.backing
            && directory.starts_with(Path::new(PATCHES).join("golden"))
            && root.join(directory).is_dir()
        {
            prune(root, directory)?;
        }
        Ok(())
    }

    /// For diagnostics that name the owner's directory on disk.
    pub(super) fn named(&self, path: &Path) -> PathBuf {
        match &self.backing {
            Backing::Disk(root) => root.join(path),
            Backing::Memory(_) => path.to_path_buf(),
        }
    }

    pub(super) fn exists(&self, path: &Path) -> Result<bool, Error> {
        let exists = match &self.backing {
            Backing::Disk(root) => root.join(path).try_exists(),
            Backing::Memory(files) => match not_beneath_file(files, path) {
                Ok(()) => Ok(files.contains_key(path) || directory_in(files, path)),
                Err(error) => Err(error),
            },
        };
        match exists {
            Ok(exists) => Ok(exists),
            Err(error) => Err(Error::io(path, &error)),
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        match &self.backing {
            Backing::Disk(root) => root.join(path).is_dir(),
            Backing::Memory(files) => directory_in(files, path),
        }
    }

    pub(super) fn is_file(&self, path: &Path) -> bool {
        match &self.backing {
            Backing::Disk(root) => root.join(path).is_file(),
            Backing::Memory(files) => files.contains_key(path),
        }
    }

    fn open_directory(&self, directory: &Path) -> Result<Entries, Error> {
        match self.entries(directory) {
            Ok(entries) => Ok(entries),
            Err(error) => Err(Error::io(&self.named(directory), &error)),
        }
    }

    pub(super) fn entries(&self, directory: &Path) -> io::Result<Entries> {
        match &self.backing {
            Backing::Disk(root) => Ok(Entries::Disk(fs::read_dir(root.join(directory))?)),
            Backing::Memory(files) => {
                not_beneath_file(files, directory)?;
                if files.contains_key(directory) {
                    return Err(io::ErrorKind::NotADirectory.into());
                }
                let mut paths = BTreeSet::new();
                for (path, _) in files.range(directory.to_path_buf()..) {
                    let Ok(relative) = path.strip_prefix(directory) else { break };
                    if let Some(child) = relative.components().next() {
                        paths.insert(directory.join(child));
                    }
                }
                if paths.is_empty() {
                    return Err(io::ErrorKind::NotFound.into());
                }
                Ok(Entries::Memory(paths.into_iter()))
            }
        }
    }

    pub(super) fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        match &self.backing {
            Backing::Disk(root) => fs::read(root.join(path)),
            Backing::Memory(files) => {
                not_beneath_file(files, path)?;
                if let Some(bytes) = files.get(path) {
                    Ok(bytes.clone())
                } else if directory_in(files, path) {
                    Err(io::ErrorKind::IsADirectory.into())
                } else {
                    Err(io::ErrorKind::NotFound.into())
                }
            }
        }
    }

    pub(super) fn read_text(&self, path: &Path) -> io::Result<String> {
        match &self.backing {
            Backing::Disk(root) => fs::read_to_string(root.join(path)),
            Backing::Memory(_) => match String::from_utf8(self.read_file(path)?) {
                Ok(text) => Ok(text),
                Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8")),
            },
        }
    }

    pub(super) fn make_directory(&self, path: &Path) -> io::Result<()> {
        match &self.backing {
            Backing::Disk(root) => fs::create_dir_all(root.join(path)),
            Backing::Memory(files) => {
                not_beneath_file(files, path)?;
                if files.contains_key(path) { Err(io::ErrorKind::AlreadyExists.into()) } else { Ok(()) }
            }
        }
    }

    pub(super) fn write_file(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        match &mut self.backing {
            Backing::Disk(root) => fs::write(root.join(path), bytes),
            Backing::Memory(files) => {
                not_beneath_file(files, path)?;
                if directory_in(files, path) {
                    return Err(io::ErrorKind::IsADirectory.into());
                }
                files.insert(path.to_owned(), bytes.to_vec());
                Ok(())
            }
        }
    }

    pub(super) fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        match &mut self.backing {
            Backing::Disk(root) => fs::remove_file(root.join(path)),
            Backing::Memory(files) => {
                not_beneath_file(files, path)?;
                if files.remove(path).is_some() {
                    Ok(())
                } else if directory_in(files, path) {
                    Err(io::ErrorKind::IsADirectory.into())
                } else {
                    Err(io::ErrorKind::NotFound.into())
                }
            }
        }
    }
}

pub(super) enum Entries {
    Disk(fs::ReadDir),
    Memory(alloc::collections::btree_set::IntoIter<PathBuf>),
}

impl Entries {
    pub(super) fn next(&mut self, directory: &Path) -> Option<io::Result<PathBuf>> {
        match self {
            Self::Disk(entries) => Some(match entries.next()? {
                Ok(entry) => Ok(directory.join(entry.file_name())),
                Err(error) => Err(error),
            }),
            Self::Memory(entries) => entries.next().map(Ok),
        }
    }
}

fn directory_in(files: &BTreeMap<PathBuf, Vec<u8>>, path: &Path) -> bool {
    for (file, _) in files.range(path.to_path_buf()..) {
        if !file.starts_with(path) {
            break;
        }
        if file != path {
            return true;
        }
    }
    false
}

fn not_beneath_file(files: &BTreeMap<PathBuf, Vec<u8>>, path: &Path) -> io::Result<()> {
    for parent in path.ancestors().skip(1) {
        if files.contains_key(parent) {
            return Err(io::ErrorKind::NotADirectory.into());
        }
    }
    Ok(())
}

fn prune(root: &Path, directory: &Path) -> Result<(), Error> {
    let full = root.join(directory);
    let mut entries = match fs::read_dir(&full) {
        Ok(entries) => entries,
        Err(error) => return Err(Error::io(&full, &error)),
    };
    if entries.next().is_none() {
        drop(entries);
        if let Err(error) = fs::remove_dir(&full) {
            return Err(Error::io(&full, &error));
        }
        if let Some(parent) = directory.parent()
            && parent != Path::new(PATCHES)
        {
            prune(root, parent)?;
        }
    }
    Ok(())
}
