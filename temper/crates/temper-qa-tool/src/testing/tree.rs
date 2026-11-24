//! The QA world's shared paths and their handles (temper/docs/design/testing/qa/contracts.md, 4).
use super::Connection;
use crate::{
    Refusal,
    faces::{
        DirectoryMode, FileEntry, FilesFace, LockMode, Mode, PathFacts, PathKind, RemoveMode, ResolveMode, WriteMode,
        WriteOptions,
    },
};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{self, ErrorKind, Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    rc::{Rc, Weak},
};

/// The face operation to fail, on the path as supplied by its caller (made absolute).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileOp {
    Read,
    OpenRead,
    Write,
    OpenWrite,
    Sync,
    Rename,
    Link,
    Directory,
    Remove,
    List,
    Facts,
    Permissions,
    Copy,
    Resolve,
    Lock,
    Bind,
    Accept,
    Connect,
}

#[derive(Debug, Default)]
struct Connections {
    pending: VecDeque<Connection>,
    scripted: VecDeque<Connection>,
}

#[derive(Debug)]
enum Contents {
    File(Vec<u8>),
    Directory,
    Link(PathBuf),
    Socket(Weak<RefCell<Connections>>),
}

impl Contents {
    fn kind(&self) -> PathKind {
        match self {
            Self::File(_) => PathKind::File,
            Self::Directory => PathKind::Directory,
            Self::Link(_) => PathKind::Link,
            Self::Socket(_) => PathKind::Socket,
        }
    }
}

#[derive(Debug)]
struct Inode {
    number: u64,
    device: u64,
    mode: u32,
    locked: bool,
    contents: Contents,
}
type Entry = Rc<RefCell<Inode>>;

struct State {
    entries: BTreeMap<PathBuf, Entry>,
    next: u64,
    file_mode: u32,
    directory_mode: u32,
    faults: BTreeMap<(PathBuf, FileOp), VecDeque<io::Error>>,
}

/// A handle on the tree; clones share files, faults, locks, listeners and default modes.
#[derive(Clone)]
pub struct Tree {
    state: Rc<RefCell<State>>,
    directory: PathBuf,
}

impl Default for Tree {
    fn default() -> Self {
        let root = Inode { number: 1, device: 1, mode: 0o755, locked: false, contents: Contents::Directory };
        Self {
            state: Rc::new(RefCell::new(State {
                entries: [(PathBuf::from("/"), Rc::new(RefCell::new(root)))].into(),
                next: 2,
                file_mode: 0o644,
                directory_mode: 0o755,
                faults: BTreeMap::new(),
            })),
            directory: "/".into(),
        }
    }
}

impl Tree {
    /// Use a different virtual working directory for this handle, not its clones.
    pub fn at(&self, directory: impl AsRef<Path>) -> io::Result<Self> {
        let directory = self.walk(directory.as_ref(), true, false)?;
        self.require_directory(&directory)?;
        Ok(Self { state: self.state.clone(), directory })
    }

    /// Set the modes for subsequent creations that don't ask for a mode.
    pub fn default_modes(&self, file: u32, directory: u32) {
        let mut state = self.state.borrow_mut();
        state.file_mode = file;
        state.directory_mode = directory;
    }

    /// Queue a failure for the next matching operation. A two-path operation checks both paths.
    pub fn inject(&self, path: impl AsRef<Path>, operation: FileOp, error: impl Into<io::Error>) {
        self.state
            .borrow_mut()
            .faults
            .entry((self.absolute(path.as_ref()), operation))
            .or_default()
            .push_back(error.into());
    }

    /// Inspect the entry itself, not a link's target, without consuming a scripted fault.
    pub fn kind(&self, path: impl AsRef<Path>) -> Option<PathKind> {
        self.walk(path.as_ref(), false, false)
            .ok()
            .and_then(|path| self.entry(&path).ok())
            .map(|e| e.borrow().contents.kind())
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.kind(path).is_some()
    }

    /// Inspect the entry's permission bits without consuming a scripted fault.
    pub fn mode(&self, path: impl AsRef<Path>) -> Option<u32> {
        self.walk(path.as_ref(), false, false).ok().and_then(|path| self.entry(&path).ok()).map(|e| e.borrow().mode)
    }

    /// Assign a device to an existing directory and its contents; new entries inherit their parent's device.
    pub fn set_device(&self, path: impl AsRef<Path>, device: u64) -> io::Result<()> {
        let path = self.walk(path.as_ref(), true, false)?;
        self.require_directory(&path)?;
        let state = self.state.borrow();
        let inside: BTreeSet<_> =
            state.entries.iter().filter(|(p, _)| p.starts_with(&path)).map(|(_, e)| e.borrow().number).collect();
        if state.entries.iter().any(|(p, e)| {
            let entry = e.borrow();
            !p.starts_with(&path) && entry.device != device && inside.contains(&entry.number)
        }) {
            return Err(ErrorKind::CrossesDevices.into());
        }
        for (p, entry) in &state.entries {
            if p.starts_with(&path) {
                entry.borrow_mut().device = device;
            }
        }
        Ok(())
    }

    pub fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.walk(path.as_ref(), false, true)?;
        self.insert(&path, Contents::Link(target.as_ref().into()), 0o777).map(|_| ())
    }

    /// Keep this value in a test to hold the lock as a separate process would.
    pub fn hold_lock(&self, path: impl AsRef<Path>) -> io::Result<Lock> {
        self.clone().lock(path.as_ref(), LockMode::Wait, false).map(|lock| lock.expect("waiting lock"))
    }

    /// Bind a test peer to a path in the world.
    pub fn socket(&self, path: impl AsRef<Path>) -> io::Result<Listener> {
        self.clone().bind(path.as_ref())
    }

    /// Get a handle to a live listener without rebinding its socket.
    pub fn bound_listener(&self, path: impl AsRef<Path>) -> io::Result<Listener> {
        let path = self.walk(path.as_ref(), true, false)?;
        let entry = self.entry(&path)?;
        let Contents::Socket(connections) = &entry.borrow().contents else {
            return Err(ErrorKind::ConnectionRefused.into());
        };
        let connections = connections.upgrade().ok_or(ErrorKind::ConnectionRefused)?;
        Ok(Listener { path, connections })
    }

    fn absolute(&self, path: &Path) -> PathBuf {
        let mut absolute = PathBuf::from("/");
        for part in self.directory.join(path).components() {
            match part {
                Component::RootDir | Component::CurDir => (),
                Component::ParentDir => {
                    absolute.pop();
                }
                Component::Normal(name) => absolute.push(name),
                Component::Prefix(_) => unreachable!("Unix paths"),
            }
        }
        absolute
    }

    fn fault(&self, path: &Path, operation: FileOp) -> io::Result<()> {
        let mut state = self.state.borrow_mut();
        let key = (self.absolute(path), operation);
        let Some(queue) = state.faults.get_mut(&key) else { return Ok(()) };
        let error = queue.pop_front().expect("queued fault");
        if queue.is_empty() {
            state.faults.remove(&key);
        }
        Err(error)
    }

    fn entry(&self, path: &Path) -> io::Result<Entry> {
        self.state.borrow().entries.get(path).cloned().ok_or_else(|| ErrorKind::NotFound.into())
    }

    fn require_directory(&self, path: &Path) -> io::Result<()> {
        if self.entry(path)?.borrow().contents.kind() == PathKind::Directory {
            Ok(())
        } else {
            Err(ErrorKind::NotADirectory.into())
        }
    }

    // Resolve components in order: `..` after a symlink is relative to its target.
    // Only the last component may be missing when the operation can create it.
    fn walk(&self, path: &Path, follow: bool, missing: bool) -> io::Result<PathBuf> {
        if path.as_os_str().is_empty() {
            return Err(ErrorKind::NotFound.into());
        }
        let mut remaining = self.directory.join(path);
        let mut resolved = PathBuf::from("/");
        for _ in 0..40 {
            let mut parts = remaining.components().peekable();
            let mut target = None;
            while let Some(part) = parts.next() {
                match part {
                    Component::RootDir => resolved = "/".into(),
                    Component::CurDir => (),
                    Component::ParentDir => {
                        self.require_directory(&resolved)?;
                        resolved.pop();
                    }
                    Component::Normal(name) => {
                        self.require_directory(&resolved)?;
                        let child = resolved.join(name);
                        let last = parts.peek().is_none();
                        let entry = match self.entry(&child) {
                            Ok(entry) => entry,
                            Err(why) if why.kind() == ErrorKind::NotFound && missing && last => return Ok(child),
                            Err(why) => return Err(why),
                        };
                        if let Contents::Link(link) = &entry.borrow().contents
                            && (follow || !last)
                        {
                            if link.as_os_str().is_empty() {
                                return Err(ErrorKind::NotFound.into());
                            }
                            target = Some(link.join(parts.collect::<PathBuf>()));
                            break;
                        }
                        resolved = child;
                    }
                    Component::Prefix(_) => unreachable!("Unix paths"),
                }
            }
            let Some(target) = target else { return Ok(resolved) };
            remaining = target;
        }
        Err(io::Error::other("too many symbolic links"))
    }

    fn parent_device(&self, path: &Path) -> io::Result<u64> {
        let parent = path.parent().ok_or(ErrorKind::PermissionDenied)?;
        self.require_directory(parent)?;
        Ok(self.entry(parent)?.borrow().device)
    }

    fn insert(&self, path: &Path, contents: Contents, mode: u32) -> io::Result<Entry> {
        if self.entry(path).is_ok() {
            return Err(ErrorKind::AlreadyExists.into());
        }
        let device = self.parent_device(path)?;
        let mut state = self.state.borrow_mut();
        let entry = Rc::new(RefCell::new(Inode { number: state.next, device, mode, locked: false, contents }));
        state.next += 1;
        state.entries.insert(path.into(), entry.clone());
        Ok(entry)
    }

    fn open_file(&self, path: &Path, options: WriteOptions) -> io::Result<FileHandle> {
        let opened_as = self.absolute(path);
        let path = self.walk(path, options.way != WriteMode::New, true)?;
        let entry = match self.entry(&path) {
            Ok(_) if options.way == WriteMode::New => return Err(ErrorKind::AlreadyExists.into()),
            Ok(entry) => entry,
            Err(_) => {
                let mode = match options.mode {
                    Mode::Default => self.state.borrow().file_mode,
                    Mode::Exact(mode) | Mode::Create(mode) => mode,
                };
                self.insert(&path, Contents::File(Vec::new()), mode)?
            }
        };
        {
            let mut inode = entry.borrow_mut();
            let Contents::File(bytes) = &mut inode.contents else {
                return Err(not_file(&inode.contents));
            };
            if options.way == WriteMode::Replace {
                bytes.clear();
            }
            if let Mode::Exact(mode) = options.mode {
                inode.mode = mode;
            }
        }
        Ok(FileHandle {
            entry,
            opened_as,
            offset: 0,
            append: options.way == WriteMode::Append,
            readable: options.read,
            writable: true,
        })
    }

    fn reader(&self, path: &Path) -> io::Result<FileHandle> {
        let entry = self.entry(&self.walk(path, true, false)?)?;
        if !matches!(entry.borrow().contents, Contents::File(_)) {
            return Err(not_file(&entry.borrow().contents));
        }
        Ok(FileHandle {
            entry,
            opened_as: self.absolute(path),
            offset: 0,
            append: false,
            readable: true,
            writable: false,
        })
    }

    fn entry_facts(&self, entry: &Entry) -> PathFacts {
        let inode = entry.borrow();
        let links = self.state.borrow().entries.values().filter(|other| Rc::ptr_eq(other, entry)).count();
        let size = match &inode.contents {
            Contents::File(bytes) => bytes.len(),
            Contents::Link(path) => path.as_os_str().as_bytes().len(),
            Contents::Directory | Contents::Socket(_) => 0,
        };
        PathFacts {
            kind: inode.contents.kind(),
            size: size as u64,
            mode: inode.mode,
            device: inode.device,
            inode: inode.number,
            links: links as u64,
        }
    }
}

fn not_file(contents: &Contents) -> io::Error {
    match contents.kind() {
        PathKind::Directory => ErrorKind::IsADirectory.into(),
        PathKind::File | PathKind::Link | PathKind::Socket | PathKind::Other => ErrorKind::InvalidInput.into(),
    }
}

#[derive(Debug)]
pub struct FileHandle {
    entry: Entry,
    opened_as: PathBuf,
    offset: usize,
    append: bool,
    readable: bool,
    writable: bool,
}

impl Read for FileHandle {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if !self.readable {
            return Err(ErrorKind::PermissionDenied.into());
        }
        let inode = self.entry.borrow();
        let Contents::File(file) = &inode.contents else { unreachable!("opened file") };
        let mut remaining = file.get(self.offset..).unwrap_or_default();
        let count = remaining.read(bytes)?;
        self.offset += count;
        Ok(count)
    }
}

impl Write for FileHandle {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.writable {
            return Err(ErrorKind::PermissionDenied.into());
        }
        let mut inode = self.entry.borrow_mut();
        let Contents::File(file) = &mut inode.contents else { unreachable!("opened file") };
        let start = if self.append { file.len() } else { self.offset };
        let end = start + bytes.len();
        if end > file.len() {
            file.resize(end, 0);
        }
        file[start..end].copy_from_slice(bytes);
        self.offset = end;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Lock {
    entry: Entry,
    acquired: Cell<bool>,
}

impl Drop for Lock {
    fn drop(&mut self) {
        if self.acquired.get() {
            self.entry.borrow_mut().locked = false;
        }
    }
}

#[derive(Debug)]
pub struct Listener {
    path: PathBuf,
    connections: Rc<RefCell<Connections>>,
}

impl Listener {
    /// Serve the next connect with a test peer instead of queuing it for `accept`.
    /// The peer can send an answer before the caller connects and read its request afterwards.
    pub fn script_connection(&self) -> Connection {
        let (client, peer) = Connection::pair();
        self.connections.borrow_mut().scripted.push_back(client);
        peer
    }
}

impl FilesFace for Tree {
    type Lock = Lock;
    type Listener = Listener;
    type Connection = Connection;
    type Reader = FileHandle;
    type Writer = FileHandle;

    fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
        self.fault(path, FileOp::Read)?;
        let mut bytes = Vec::new();
        self.reader(path)?.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn open_read(&mut self, path: &Path) -> io::Result<FileHandle> {
        self.fault(path, FileOp::OpenRead)?;
        self.reader(path)
    }

    fn write(&mut self, path: &Path, bytes: &[u8], options: WriteOptions, sync: bool) -> io::Result<()> {
        self.fault(path, FileOp::Write)?;
        self.open_file(path, options)?.write_all(bytes)?;
        if sync {
            self.fault(path, FileOp::Sync)?;
        }
        Ok(())
    }

    fn open_write(&mut self, path: &Path, options: WriteOptions) -> io::Result<FileHandle> {
        self.fault(path, FileOp::OpenWrite)?;
        self.open_file(path, options)
    }

    fn reader_facts(&mut self, file: &FileHandle) -> io::Result<PathFacts> {
        Ok(self.entry_facts(&file.entry))
    }

    fn writer_permissions(&mut self, file: &FileHandle, mode: u32) -> io::Result<()> {
        file.entry.borrow_mut().mode = mode;
        Ok(())
    }

    fn sync_writer(&mut self, file: &FileHandle) -> io::Result<()> {
        self.fault(&file.opened_as, FileOp::Sync)
    }

    fn open_log_tail(&mut self, path: &Path) -> Result<FileHandle, Refusal> {
        let mut log = crate::private::file(path, self, WriteMode::Append, true)?;
        log.offset = match &log.entry.borrow().contents {
            Contents::File(bytes) => bytes.len(),
            _ => unreachable!("opened file"),
        };
        Ok(log)
    }

    fn rename(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        self.fault(source, FileOp::Rename)?;
        if self.absolute(source) != self.absolute(destination) {
            self.fault(destination, FileOp::Rename)?;
        }
        let source = self.walk(source, false, false)?;
        let destination = self.walk(destination, false, true)?;
        let from = self.entry(&source)?;
        if source == destination {
            return Ok(());
        }
        if destination.starts_with(&source) {
            return Err(ErrorKind::InvalidInput.into());
        }
        if self.parent_device(&source)? != self.parent_device(&destination)? {
            return Err(ErrorKind::CrossesDevices.into());
        }
        let mut state = self.state.borrow_mut();
        let directory = from.borrow().contents.kind() == PathKind::Directory;
        if let Some(to) = state.entries.get(&destination) {
            if Rc::ptr_eq(&from, to) {
                return Ok(());
            }
            match (directory, to.borrow().contents.kind() == PathKind::Directory) {
                (true, false) => return Err(ErrorKind::NotADirectory.into()),
                (false, true) => return Err(ErrorKind::IsADirectory.into()),
                (true, true) if state.entries.keys().any(|p| p.parent() == Some(&destination)) => {
                    return Err(ErrorKind::DirectoryNotEmpty.into());
                }
                (true, true) | (false, false) => (),
            }
        }
        let paths: Vec<_> = state.entries.keys().filter(|p| p.starts_with(&source)).cloned().collect();
        for path in paths {
            let relative = path.strip_prefix(&source).expect("renamed path");
            let moved = if relative.as_os_str().is_empty() { destination.clone() } else { destination.join(relative) };
            let entry = state.entries.remove(&path).expect("renamed entry");
            state.entries.insert(moved, entry);
        }
        Ok(())
    }

    fn link(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        self.fault(source, FileOp::Link)?;
        if self.absolute(source) != self.absolute(destination) {
            self.fault(destination, FileOp::Link)?;
        }
        let entry = self.entry(&self.walk(source, false, false)?)?;
        let destination = self.walk(destination, false, true)?;
        if entry.borrow().contents.kind() == PathKind::Directory {
            return Err(ErrorKind::PermissionDenied.into());
        }
        if entry.borrow().device != self.parent_device(&destination)? {
            return Err(ErrorKind::CrossesDevices.into());
        }
        if self.entry(&destination).is_ok() {
            return Err(ErrorKind::AlreadyExists.into());
        }
        self.state.borrow_mut().entries.insert(destination, entry);
        Ok(())
    }

    fn directory(&mut self, path: &Path, mode: DirectoryMode) -> io::Result<()> {
        self.fault(path, FileOp::Directory)?;
        if mode == DirectoryMode::Private {
            return crate::faces::private_directory(self, path).map_err(|(_, why)| why);
        }
        if mode == DirectoryMode::Parents
            && let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
            && self.walk(parent, true, false).is_err()
        {
            self.directory(parent, mode)?;
        }
        let resolved = self.walk(path, false, true)?;
        if self.entry(&resolved).is_ok() {
            if mode == DirectoryMode::One
                || !self.walk(path, true, false).is_ok_and(|path| self.require_directory(&path).is_ok())
            {
                return Err(ErrorKind::AlreadyExists.into());
            }
        } else {
            let bits = if mode == DirectoryMode::PrivateOne { 0o700 } else { self.state.borrow().directory_mode };
            self.insert(&resolved, Contents::Directory, bits)?;
        }
        if mode == DirectoryMode::PrivateOne {
            self.permissions(path, 0o700)?;
        }
        Ok(())
    }

    fn remove(&mut self, path: &Path, mode: RemoveMode) -> io::Result<()> {
        self.fault(path, FileOp::Remove)?;
        let path = self.walk(path, false, false)?;
        if path == Path::new("/") {
            return Err(ErrorKind::PermissionDenied.into());
        }
        let kind = self.entry(&path)?.borrow().contents.kind();
        match (mode, kind) {
            (RemoveMode::File, PathKind::Directory) => return Err(ErrorKind::IsADirectory.into()),
            (RemoveMode::Tree, PathKind::File | PathKind::Socket | PathKind::Other) => {
                return Err(ErrorKind::NotADirectory.into());
            }
            (RemoveMode::Any | RemoveMode::Tree | RemoveMode::File, _) => (),
        }
        self.state.borrow_mut().entries.retain(|p, _| !p.starts_with(&path));
        Ok(())
    }

    fn list(&mut self, path: &Path) -> io::Result<Vec<FileEntry>> {
        self.fault(path, FileOp::List)?;
        let path = self.walk(path, true, false)?;
        self.require_directory(&path)?;
        Ok(self
            .state
            .borrow()
            .entries
            .iter()
            .filter(|(p, _)| p.parent() == Some(&path))
            .map(|(p, e)| FileEntry {
                name: p.file_name().expect("entry name").as_bytes().into(),
                kind: e.borrow().contents.kind(),
            })
            .collect())
    }

    fn facts(&mut self, path: &Path, follow: bool) -> io::Result<PathFacts> {
        self.fault(path, FileOp::Facts)?;
        let entry = self.entry(&self.walk(path, follow, false)?)?;
        Ok(self.entry_facts(&entry))
    }

    fn permissions(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        self.fault(path, FileOp::Permissions)?;
        self.entry(&self.walk(path, true, false)?)?.borrow_mut().mode = mode;
        Ok(())
    }

    fn copy(&mut self, source: &Path, destination: &Path) -> io::Result<u64> {
        self.fault(source, FileOp::Copy)?;
        if self.absolute(source) != self.absolute(destination) {
            self.fault(destination, FileOp::Copy)?;
        }
        let mut from = self.reader(source)?;
        let mode = Mode::Exact(from.entry.borrow().mode);
        let mut to = self.open_file(destination, WriteOptions { way: WriteMode::Replace, mode, read: false })?;
        io::copy(&mut from, &mut to)
    }

    fn resolve(&mut self, path: &Path, mode: ResolveMode) -> io::Result<PathBuf> {
        self.fault(path, FileOp::Resolve)?;
        match mode {
            ResolveMode::Canonical => self.walk(path, true, false),
            ResolveMode::Absolute => Ok(self.absolute(path)),
        }
    }

    fn lock(&mut self, path: &Path, mode: LockMode, exact: bool) -> io::Result<Option<Lock>> {
        let lock = self.open_lock(path, exact)?;
        if self.try_lock(&lock)? {
            Ok(Some(lock))
        } else {
            match mode {
                // There is no other thread to release a blocking lock in this world.
                LockMode::Wait => Err(ErrorKind::WouldBlock.into()),
                LockMode::Try => Ok(None),
            }
        }
    }

    fn open_lock(&mut self, path: &Path, exact: bool) -> io::Result<Lock> {
        self.fault(path, FileOp::Lock)?;
        let options = WriteOptions {
            way: WriteMode::OpenOrCreate,
            mode: if exact { Mode::Exact(0o600) } else { Mode::Create(0o600) },
            read: true,
        };
        let file = self.open_file(path, options)?;
        Ok(Lock { entry: file.entry, acquired: Cell::new(false) })
    }

    fn try_lock(&mut self, lock: &Lock) -> io::Result<bool> {
        let mut inode = lock.entry.borrow_mut();
        if inode.locked && !lock.acquired.get() {
            return Ok(false);
        }
        inode.locked = true;
        lock.acquired.set(true);
        Ok(true)
    }

    fn bind(&mut self, path: &Path) -> io::Result<Listener> {
        self.fault(path, FileOp::Bind)?;
        let path = self.walk(path, false, true)?;
        if self.kind(&path) == Some(PathKind::Directory) {
            return Err(ErrorKind::IsADirectory.into());
        }
        self.state.borrow_mut().entries.remove(&path);
        let connections = Rc::new(RefCell::new(Connections::default()));
        self.insert(&path, Contents::Socket(Rc::downgrade(&connections)), 0o600)?;
        Ok(Listener { path, connections })
    }

    fn accept(&mut self, listener: &Listener) -> io::Result<Option<Connection>> {
        self.fault(&listener.path, FileOp::Accept)?;
        Ok(listener.connections.borrow_mut().pending.pop_front())
    }

    fn connect(&mut self, path: &Path) -> io::Result<Option<Connection>> {
        let entry = match self
            .fault(path, FileOp::Connect)
            .and_then(|()| self.walk(path, true, false))
            .and_then(|p| self.entry(&p))
        {
            Ok(entry) => entry,
            Err(why) if matches!(why.kind(), ErrorKind::NotFound | ErrorKind::ConnectionRefused) => return Ok(None),
            Err(why) => return Err(why),
        };
        let Contents::Socket(connections) = &entry.borrow().contents else { return Ok(None) };
        let Some(connections) = connections.upgrade() else { return Ok(None) };
        let mut connections = connections.borrow_mut();
        if let Some(client) = connections.scripted.pop_front() {
            return Ok(Some(client));
        }
        let (client, server) = Connection::pair();
        connections.pending.push_back(server);
        Ok(Some(client))
    }
}
