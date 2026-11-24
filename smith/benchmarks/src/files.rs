//! The harness's filesystem face: native calls in a run, a laid tree in a step
//! test (smith/docs/design/benchmarks/README.md, 13). It keeps no paths or
//! contents; callers decide what to admit, bound, sort and publish.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// The kind of node found by a filesystem lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// A directory containing named entries.
    Directory,
    /// A regular file containing bytes.
    File,
    /// A symbolic link containing a target path.
    Link,
    /// A special entry, such as a FIFO, device or socket.
    Other,
}

impl From<fs::FileType> for Kind {
    fn from(kind: fs::FileType) -> Self {
        if kind.is_dir() {
            Self::Directory
        } else if kind.is_file() {
            Self::File
        } else if kind.is_symlink() {
            Self::Link
        } else {
            Self::Other
        }
    }
}

/// Facts returned by an entry's metadata lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Facts {
    pub kind: Kind,
    /// Permission bits (`0o7777`), not the file type.
    pub mode: u32,
    /// Length in bytes.
    pub size: u64,
    /// Owning user ID.
    pub owner: u32,
}

impl From<fs::Metadata> for Facts {
    fn from(metadata: fs::Metadata) -> Self {
        Self {
            kind: metadata.file_type().into(),
            mode: metadata.permissions().mode() & 0o7777,
            size: metadata.len(),
            owner: metadata.uid(),
        }
    }
}

/// A directory child whose no-follow kind need not be asked for until admitted by name.
#[derive(Debug)]
pub struct Child {
    entry: Entry,
}

#[derive(Debug)]
enum Entry {
    Native(fs::DirEntry),
    Laid { path: PathBuf, kind: Kind },
}

impl Child {
    /// Supply the named child and its kind from a test tree.
    #[must_use]
    pub fn laid(path: PathBuf, kind: Kind) -> Self {
        Self { entry: Entry::Laid { path, kind } }
    }

    /// Return the child's path with the spelling of the directory that was listed.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        match &self.entry {
            Entry::Native(entry) => entry.path(),
            Entry::Laid { path, .. } => path.clone(),
        }
    }

    /// Ask for the child's kind without following links.
    pub fn file_type(&self) -> io::Result<Kind> {
        match &self.entry {
            Entry::Native(entry) => entry.file_type().map(Kind::from),
            Entry::Laid { kind, .. } => Ok(*kind),
        }
    }
}

/// File operations used by native harness runs and supplied by offline tests.
pub trait Files {
    /// Inspect an entry itself, not a final link's target.
    fn symlink_metadata(&self, path: &Path) -> io::Result<Facts>;

    /// Inspect an entry, following any links.
    fn metadata(&self, path: &Path) -> io::Result<Facts>;

    /// Resolve an existing path to its absolute physical path.
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;

    /// Yield children in filesystem order, deferring each kind and iteration error.
    fn read_dir(&self, path: &Path) -> io::Result<Box<dyn Iterator<Item = io::Result<Child>> + '_>>;

    /// Read the whole file as bytes.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Read the whole file as UTF-8, refusing invalid text.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    /// Read at most `limit` bytes from the start; pass N + 1 to detect excess over N.
    fn read_bounded(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>>;

    /// Read at most `limit` bytes from the start, stopping after the first `delimiter` (included).
    fn read_until(&self, path: &Path, delimiter: u8, limit: u64) -> io::Result<Vec<u8>>;

    /// Check whether a path has a target, distinguishing absence from failed lookups.
    fn try_exists(&self, path: &Path) -> io::Result<bool>;

    /// Stream an admitted regular file's bytes to a lowercase hexadecimal SHA-256.
    fn sha256(&self, path: &Path) -> io::Result<String>;

    /// Create one directory with the requested permission bits, subject to umask.
    fn create_dir(&self, path: &Path, mode: u32) -> io::Result<()>;

    /// Create a directory and missing parents with the requested bits, subject to umask.
    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()>;

    /// Create a file exclusively with the requested bits, write all bytes, and sync if requested.
    fn create_new(&self, path: &Path, mode: u32, bytes: &[u8], sync: bool) -> io::Result<()>;

    /// Replace a file's bytes, creating it with the default file mode if absent.
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;

    /// Set the target's permission bits.
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()>;

    /// Remove a file or link without following its final component.
    fn remove_file(&self, path: &Path) -> io::Result<()>;

    /// Remove a directory tree without following its links.
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;

    /// Move one entry to another name, replacing a compatible destination.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;

    /// Copy file bytes and permissions, returning the number of bytes copied.
    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64>;
}

/// The live filesystem, making the harness's native calls directly.
#[derive(Clone, Copy, Debug)]
pub struct Native;

impl Files for Native {
    fn symlink_metadata(&self, path: &Path) -> io::Result<Facts> {
        fs::symlink_metadata(path).map(Facts::from)
    }

    fn metadata(&self, path: &Path) -> io::Result<Facts> {
        fs::metadata(path).map(Facts::from)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        path.canonicalize()
    }

    fn read_dir(&self, path: &Path) -> io::Result<Box<dyn Iterator<Item = io::Result<Child>> + '_>> {
        Ok(Box::new(fs::read_dir(path)?.map(|entry| entry.map(|entry| Child { entry: Entry::Native(entry) }))))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn read_bounded(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        fs::File::open(path)?.take(limit).read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn read_until(&self, path: &Path, delimiter: u8, limit: u64) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        BufReader::new(fs::File::open(path)?.take(limit)).read_until(delimiter, &mut bytes)?;
        Ok(bytes)
    }

    fn try_exists(&self, path: &Path) -> io::Result<bool> {
        path.try_exists()
    }

    fn sha256(&self, path: &Path) -> io::Result<String> {
        let mut file = fs::File::open(path)?;
        let mut digest = Sha256::new();
        let mut bytes = [0; 8192];
        loop {
            let read = file.read(&mut bytes)?;
            if read == 0 {
                break;
            }
            digest.update(&bytes[..read]);
        }
        Ok(format!("{:x}", digest.finalize()))
    }

    fn create_dir(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::DirBuilder::new().mode(mode).create(path)
    }

    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::DirBuilder::new().recursive(true).mode(mode).create(path)
    }

    fn create_new(&self, path: &Path, mode: u32, bytes: &[u8], sync: bool) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(mode).open(path)?;
        file.write_all(bytes)?;
        if sync {
            file.sync_all()?;
        }
        Ok(())
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::write(path, bytes)
    }

    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
        fs::copy(from, to)
    }
}
