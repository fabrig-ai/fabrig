//! Files, locks, Unix sockets and build checksums (temper/docs/design/testing/qa/contracts.md, 1).
use crate::{Refusal, manifest, refused};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, DirBuilder, File, FileType, OpenOptions, Permissions, TryLockError},
    io::{self, Read, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    Replace,
    New,
    Append,
    OpenOrCreate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Default,
    Create(u32),
    Exact(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteOptions {
    pub way: WriteMode,
    pub mode: Mode,
    pub read: bool,
}
impl WriteOptions {
    #[must_use]
    pub const fn private(way: WriteMode) -> Self {
        Self { way, mode: Mode::Exact(0o600), read: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectoryMode {
    One,
    Parents,
    Private,
    /// Make just this directory, or set its exact mode when it already exists.
    PrivateOne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveMode {
    File,
    Tree,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveMode {
    Canonical,
    Absolute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockMode {
    Wait,
    Try,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathKind {
    File,
    Directory,
    Link,
    Socket,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub name: Box<[u8]>,
    pub kind: PathKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathFacts {
    pub kind: PathKind,
    pub size: u64,
    pub mode: u32,
    pub device: u64,
    pub inode: u64,
    pub links: u64,
}

pub trait FilesFace {
    type Lock;
    type Listener;
    type Connection: Read + Write;
    type Reader: Read;
    type Writer: Write;

    /// Read a file whole.
    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.open_read(path)?.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
    /// Open a file for reading as a stream.
    fn open_read(&mut self, _path: &Path) -> io::Result<Self::Reader> {
        unsupported()
    }
    /// Write a file: replace it, make it new, or append to it, with an exact mode
    /// set where asked (regardless of umask), and synced where asked.
    fn write(&mut self, _path: &Path, _bytes: &[u8], _options: WriteOptions, _sync: bool) -> io::Result<()> {
        unsupported()
    }
    /// Open a file with the write options, keeping the handle for the caller.
    fn open_write(&mut self, _path: &Path, _options: WriteOptions) -> io::Result<Self::Writer> {
        unsupported()
    }
    /// Rename a path.
    fn rename(&mut self, _source: &Path, _destination: &Path) -> io::Result<()> {
        unsupported()
    }
    /// Hard-link a file; the "crosses devices" answer is the caller's to read.
    fn link(&mut self, _source: &Path, _destination: &Path) -> io::Result<()> {
        unsupported()
    }
    /// Make a directory: one, with its parents, or private (missing parents made,
    /// the directory itself 0700 exactly, even where it was there).
    fn directory(&mut self, _path: &Path, _mode: DirectoryMode) -> io::Result<()> {
        unsupported()
    }
    /// Remove a file, a directory with what it holds, or whatever a path names,
    /// without following a link.
    fn remove(&mut self, _path: &Path, _mode: RemoveMode) -> io::Result<()> {
        unsupported()
    }
    /// List a directory: each entry's name as bytes and its kind.
    fn list(&mut self, _path: &Path) -> io::Result<Vec<FileEntry>> {
        unsupported()
    }
    /// A path's facts, following a link or not: its kind, size, permission bits,
    /// device, and the identity and link count that hard links share.
    fn facts(&mut self, _path: &Path, _follow: bool) -> io::Result<PathFacts> {
        unsupported()
    }
    /// Set a path's permission bits exactly.
    fn permissions(&mut self, _path: &Path, _mode: u32) -> io::Result<()> {
        unsupported()
    }
    /// Copy a file's bytes to a path.
    fn copy(&mut self, _source: &Path, _destination: &Path) -> io::Result<u64> {
        unsupported()
    }
    /// Resolve a path: canonical (following links; the path must exist) or
    /// absolute (lexical, from the working directory).
    fn resolve(&mut self, _path: &Path, _mode: ResolveMode) -> io::Result<PathBuf> {
        unsupported()
    }
    /// Lock a lock file, made 0600 where missing: waiting, or trying and answering
    /// `None` where it would wait; held until the lock's value is dropped.
    /// If `exact` is set, enforce 0600 even for a lock file that already exists.
    fn lock(&mut self, _path: &Path, _mode: LockMode, _exact: bool) -> io::Result<Option<Self::Lock>> {
        unsupported()
    }
    /// Bind a Unix socket: a stale entry replaced, the socket 0600, its accepts never waiting.
    fn bind(&mut self, _path: &Path) -> io::Result<Self::Listener> {
        unsupported()
    }
    /// Accept a Unix connection, or none yet; the connection reads and writes without
    /// waiting (`WouldBlock`) and closes when dropped.
    fn accept(&mut self, _listener: &Self::Listener) -> io::Result<Option<Self::Connection>> {
        unsupported()
    }
    /// Connect, or answer `None` for a path missing or refusing; the connection reads
    /// and writes without waiting (`WouldBlock`) and closes when dropped.
    fn connect(&mut self, _path: &Path) -> io::Result<Option<Self::Connection>> {
        unsupported()
    }
    /// Write the build files' SHA-256 lines over the face's streamed reads.
    fn hash(&mut self, directory: &Path, files: &[PathBuf]) -> Result<Vec<u8>, Refusal> {
        let mut answer = Vec::new();
        for path in files {
            let hex = self
                .open_read(&directory.join(path))
                .and_then(checksum)
                .map_err(|why| refused(format!("checksum: {}: {why}", path.display())))?;
            answer.extend_from_slice(&manifest::line(&hex, path));
        }
        Ok(answer)
    }
    /// Check the build files' SHA-256 lines over the face's reads, naming failures
    /// in manifest order.
    fn check(&mut self, directory: &Path, manifest: &Path) -> Result<(), Refusal> {
        let bytes = self.read(manifest).map_err(|why| refused(format!("{}: {why}", manifest.display())))?;
        let rows = manifest::rows(&bytes).map_err(|why| refused(format!("checksum check: {why}")))?;
        let mut failures = Vec::new();
        for (path, expected) in rows {
            match self.open_read(&directory.join(&path)).and_then(checksum) {
                Ok(hex) if hex == expected => {}
                Ok(_) => failures.push(format!("{}: FAILED", path.display())),
                Err(why) => failures.push(format!("{}: {why}", path.display())),
            }
        }
        if failures.is_empty() { Ok(()) } else { Err(refused(format!("checksum check: {}", failures.join("; ")))) }
    }
}

fn unsupported<T>() -> io::Result<T> {
    Err(io::ErrorKind::Unsupported.into())
}

fn checksum(mut file: impl Read) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut buffer = vec![0_u8; 65_536].into_boxed_slice();
    loop {
        let count = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(why) if why.kind() == io::ErrorKind::Interrupted => continue,
            Err(why) => return Err(why),
        };
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

// Keep the failing ancestor's path for the shared helper's PATH: WHY diagnostic.
pub(crate) fn private_directory<'a>(files: &mut impl FilesFace, path: &'a Path) -> Result<(), (&'a Path, io::Error)> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty())
        && files.facts(parent, true).is_err()
    {
        private_directory(files, parent)?;
    }
    files.directory(path, DirectoryMode::PrivateOne).map_err(|why| (path, why))
}

pub struct RealFiles;

impl FilesFace for RealFiles {
    type Lock = File;
    type Listener = UnixListener;
    type Connection = UnixStream;
    type Reader = File;
    type Writer = File;

    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }
    fn open_read(&mut self, path: &Path) -> io::Result<File> {
        File::open(path)
    }
    fn write(&mut self, path: &Path, bytes: &[u8], options: WriteOptions, sync: bool) -> io::Result<()> {
        let mut file = self.open_write(path, options)?;
        file.write_all(bytes)?;
        if sync {
            file.sync_all()?;
        }
        Ok(())
    }
    fn open_write(&mut self, path: &Path, options: WriteOptions) -> io::Result<File> {
        let mut open = OpenOptions::new();
        open.write(true).read(options.read);
        match options.way {
            WriteMode::Replace => open.create(true).truncate(true),
            WriteMode::New => open.create_new(true),
            WriteMode::Append => open.create(true).append(true),
            WriteMode::OpenOrCreate => open.create(true).truncate(false),
        };
        if let Mode::Create(mode) | Mode::Exact(mode) = options.mode {
            open.mode(mode);
        }
        let file = open.open(path)?;
        if let Mode::Exact(mode) = options.mode {
            file.set_permissions(Permissions::from_mode(mode))?;
        }
        Ok(file)
    }
    fn rename(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        fs::rename(source, destination)
    }
    fn link(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        fs::hard_link(source, destination)
    }
    fn directory(&mut self, path: &Path, mode: DirectoryMode) -> io::Result<()> {
        match mode {
            DirectoryMode::One => fs::create_dir(path),
            DirectoryMode::Parents => fs::create_dir_all(path),
            DirectoryMode::Private => private_directory(self, path).map_err(|(_, why)| why),
            DirectoryMode::PrivateOne => {
                match DirBuilder::new().mode(0o700).create(path) {
                    Ok(()) => {}
                    Err(why) if why.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => {}
                    Err(why) => return Err(why),
                }
                self.permissions(path, 0o700)
            }
        }
    }
    fn remove(&mut self, path: &Path, mode: RemoveMode) -> io::Result<()> {
        match mode {
            RemoveMode::File => fs::remove_file(path),
            RemoveMode::Tree => fs::remove_dir_all(path),
            RemoveMode::Any => match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path),
                Ok(_) => fs::remove_file(path),
                Err(why) => Err(why),
            },
        }
    }
    fn list(&mut self, path: &Path) -> io::Result<Vec<FileEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;
                Ok(FileEntry { name: entry.file_name().as_bytes().into(), kind: kind(entry.file_type()?) })
            })
            .collect()
    }
    fn facts(&mut self, path: &Path, follow: bool) -> io::Result<PathFacts> {
        let metadata = if follow { fs::metadata(path)? } else { fs::symlink_metadata(path)? };
        Ok(PathFacts {
            kind: kind(metadata.file_type()),
            size: metadata.len(),
            mode: metadata.mode() & 0o7777,
            device: metadata.dev(),
            inode: metadata.ino(),
            links: metadata.nlink(),
        })
    }
    fn permissions(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, Permissions::from_mode(mode))
    }
    fn copy(&mut self, source: &Path, destination: &Path) -> io::Result<u64> {
        fs::copy(source, destination)
    }
    fn resolve(&mut self, path: &Path, mode: ResolveMode) -> io::Result<PathBuf> {
        match mode {
            ResolveMode::Canonical => fs::canonicalize(path),
            ResolveMode::Absolute => std::path::absolute(path),
        }
    }
    fn lock(&mut self, path: &Path, mode: LockMode, exact: bool) -> io::Result<Option<File>> {
        let options = WriteOptions {
            way: WriteMode::OpenOrCreate,
            mode: if exact { Mode::Exact(0o600) } else { Mode::Create(0o600) },
            read: true,
        };
        let file = self.open_write(path, options)?;
        match mode {
            LockMode::Wait => file.lock()?,
            LockMode::Try => match file.try_lock() {
                Ok(()) => {}
                Err(TryLockError::WouldBlock) => return Ok(None),
                Err(TryLockError::Error(why)) => return Err(why),
            },
        }
        Ok(Some(file))
    }
    fn bind(&mut self, path: &Path) -> io::Result<UnixListener> {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(why) if why.kind() == io::ErrorKind::NotFound => {}
            Err(why) => return Err(why),
        }
        let listener = UnixListener::bind(path)?;
        let mut prepare = || {
            self.permissions(path, 0o600)?;
            listener.set_nonblocking(true)
        };
        if let Err(why) = prepare() {
            let _ = fs::remove_file(path);
            return Err(why);
        }
        Ok(listener)
    }
    fn accept(&mut self, listener: &UnixListener) -> io::Result<Option<UnixStream>> {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true)?;
                Ok(Some(stream))
            }
            Err(why) if why.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(why) => Err(why),
        }
    }
    fn connect(&mut self, path: &Path) -> io::Result<Option<UnixStream>> {
        match UnixStream::connect(path) {
            Ok(stream) => {
                stream.set_nonblocking(true)?;
                Ok(Some(stream))
            }
            Err(why) if matches!(why.kind(), io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused) => Ok(None),
            Err(why) => Err(why),
        }
    }
}

fn kind(file: FileType) -> PathKind {
    if file.is_file() {
        PathKind::File
    } else if file.is_dir() {
        PathKind::Directory
    } else if file.is_symlink() {
        PathKind::Link
    } else if file.is_socket() {
        PathKind::Socket
    } else {
        PathKind::Other
    }
}
