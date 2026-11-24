//! A laid filesystem for harness step tests (smith/docs/design/benchmarks/README.md, 13).
//! The machine owns the nodes, bytes and permissions. The tree resolves Unix
//! paths from its own `/`, scripts refusals, and records observable operations;
//! it never consults the host's filesystem or current directory.

use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsStr;
use std::io::{self, ErrorKind};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use skein_fake_machine::{How, Is, Item, Listed, Machine, Opened, Refusal};
use smith_bench::files::{Child, Facts, Files, Kind};

/// A face operation that can be observed or refused at a named path.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Operation {
    /// Inspect the named entry without following its final link.
    SymlinkMetadata,
    /// Inspect the target of the path.
    Metadata,
    /// Resolve a path to its physical spelling.
    Canonicalize,
    /// Open and list the named directory.
    ReadDir,
    /// Read a whole file's bytes.
    Read,
    /// Read a whole file as UTF-8.
    ReadToString,
    /// Read a prefix with a byte limit.
    ReadBounded,
    /// Read a prefix with a delimiter and byte limit.
    ReadUntil,
    /// Check whether the path has a target.
    TryExists,
    /// Hash a file's bytes.
    Sha256,
    /// Make a single directory.
    CreateDir,
    /// Make a directory and its missing parents.
    CreateDirAll,
    /// Make and write a file exclusively.
    CreateNew,
    /// Replace a file's bytes.
    Write,
    /// Set the target's permission bits.
    SetPermissions,
    /// Unlink a file or symbolic link.
    RemoveFile,
    /// Remove a tree without following its links.
    RemoveDirAll,
    /// Move an entry between names.
    Rename,
    /// Copy a file's bytes and mode.
    Copy,
}

/// One attempted write, including scripted refusals, in call order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Write {
    pub operation: Operation,
    /// Destination for operations with both a source and destination.
    pub path: PathBuf,
    /// Source for a rename or copy.
    pub source: Option<PathBuf>,
}

/// One read of a path the test marked never to be read, even if it failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Read {
    pub operation: Operation,
    pub path: PathBuf,
}

/// A benchmark filesystem laid on one fake machine, with a write and read log.
#[derive(Debug)]
pub struct Tree {
    disk: RefCell<Disk>,
    observations: RefCell<Observations>,
}

#[derive(Debug, Default)]
struct Observations {
    refusals: BTreeMap<(Operation, PathBuf), ErrorKind>,
    never_read: BTreeSet<PathBuf>,
    unexpected_reads: Vec<Read>,
    writes: Vec<Write>,
}

impl Tree {
    /// Lay items beneath `/` for the fake machine's default user.
    #[must_use]
    pub fn new(items: &[Item]) -> Self {
        Self::lay(Machine::new(), items)
    }

    /// Lay items beneath `/` with a particular effective user for permission checks.
    #[must_use]
    pub fn for_user(user: u32, items: &[Item]) -> Self {
        Self::lay(Machine::for_user(user), items)
    }

    fn lay(mut machine: Machine, items: &[Item]) -> Self {
        let root = machine.lay(items);
        Self { disk: RefCell::new(Disk { machine, root }), observations: RefCell::default() }
    }

    /// Borrow the underlying machine for read-back outside the face and its logs.
    pub fn machine(&self) -> RefMut<'_, Machine> {
        RefMut::map(self.disk.borrow_mut(), |disk| &mut disk.machine)
    }

    /// The machine handle for `/`. Get it before borrowing the machine for read-back.
    #[must_use]
    pub fn root(&self) -> Opened {
        self.disk.borrow().root
    }

    /// Refuse an operation at this path with the given I/O error kind.
    pub fn refuse(&self, operation: Operation, path: impl Into<PathBuf>, kind: ErrorKind) {
        self.observations.borrow_mut().refusals.insert((operation, path.into()), kind);
    }

    /// Mark a path that must not be read, by name or as the target of a read through a link.
    pub fn never_read(&self, path: impl Into<PathBuf>) {
        self.observations.borrow_mut().never_read.insert(path.into());
    }

    /// Read the attempts on marked paths, in order.
    #[must_use]
    pub fn unexpected_reads(&self) -> Vec<Read> {
        self.observations.borrow().unexpected_reads.clone()
    }

    /// Read the write attempts in the order the face received them.
    #[must_use]
    pub fn writes(&self) -> Vec<Write> {
        self.observations.borrow().writes.clone()
    }

    fn check(&self, operation: Operation, path: &Path, read: bool) -> io::Result<()> {
        let mut observations = self.observations.borrow_mut();
        if read && observations.never_read.contains(path) {
            observations.unexpected_reads.push(Read { operation, path: path.into() });
        }
        match observations.refusals.get(&(operation, path.into())) {
            Some(kind) => Err((*kind).into()),
            None => Ok(()),
        }
    }

    fn resolve_read(&self, disk: &mut Disk, operation: Operation, path: &Path, follow: bool) -> io::Result<Resolved> {
        self.check(operation, path, true)?;
        let resolved = disk.resolve(path, follow, false)?;
        if resolved.path != path {
            self.check(operation, &resolved.path, true)?;
        }
        Ok(resolved)
    }

    fn inspect(&self, operation: Operation, path: &Path, follow: bool) -> io::Result<Resolved> {
        self.resolve_read(&mut self.disk.borrow_mut(), operation, path, follow)
    }

    fn bytes(&self, operation: Operation, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        let mut disk = self.disk.borrow_mut();
        let resolved = self.resolve_read(&mut disk, operation, path, true)?;
        disk.read(&resolved.path, limit)
    }

    fn writing(&self, operation: Operation, path: &Path, source: Option<&Path>) -> io::Result<()> {
        self.observations.borrow_mut().writes.push(Write {
            operation,
            path: path.into(),
            source: source.map(Path::to_path_buf),
        });
        self.check(operation, path, false)
    }
}

impl Files for Tree {
    fn symlink_metadata(&self, path: &Path) -> io::Result<Facts> {
        Ok(self.inspect(Operation::SymlinkMetadata, path, false)?.facts())
    }

    fn metadata(&self, path: &Path) -> io::Result<Facts> {
        Ok(self.inspect(Operation::Metadata, path, true)?.facts())
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(self.inspect(Operation::Canonicalize, path, true)?.path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Box<dyn Iterator<Item = io::Result<Child>> + '_>> {
        let mut disk = self.disk.borrow_mut();
        let directory = self.resolve_read(&mut disk, Operation::ReadDir, path, true)?;
        let children = disk.open(&directory.path, How::Directory, list)?;
        let directory = path.to_path_buf();
        Ok(Box::new(
            children
                .into_iter()
                .map(move |(kind, name)| Ok(Child::laid(directory.join(OsStr::from_bytes(&name)), entry_kind(kind)))),
        ))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.bytes(Operation::Read, path, u64::MAX)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        String::from_utf8(self.bytes(Operation::ReadToString, path, u64::MAX)?)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "stream did not contain valid UTF-8"))
    }

    fn read_bounded(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        self.bytes(Operation::ReadBounded, path, limit)
    }

    fn read_until(&self, path: &Path, delimiter: u8, limit: u64) -> io::Result<Vec<u8>> {
        let mut bytes = self.bytes(Operation::ReadUntil, path, limit)?;
        if let Some(end) = bytes.iter().position(|byte| *byte == delimiter) {
            bytes.truncate(end + 1);
        }
        Ok(bytes)
    }

    fn try_exists(&self, path: &Path) -> io::Result<bool> {
        match self.inspect(Operation::TryExists, path, true) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn sha256(&self, path: &Path) -> io::Result<String> {
        Ok(format!("{:x}", Sha256::digest(self.bytes(Operation::Sha256, path, u64::MAX)?)))
    }

    fn create_dir(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.writing(Operation::CreateDir, path, None)?;
        self.disk.borrow_mut().make_directory(path, mode)
    }

    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.writing(Operation::CreateDirAll, path, None)?;
        self.disk.borrow_mut().make_directories(path, mode)
    }

    fn create_new(&self, path: &Path, mode: u32, bytes: &[u8], sync: bool) -> io::Result<()> {
        self.writing(Operation::CreateNew, path, None)?;
        let mut disk = self.disk.borrow_mut();
        let resolved = disk.resolve(path, false, true)?;
        disk.open(&resolved.path, How::Create { mode: mode & 0o7777 }, |machine, file| {
            machine.write(file, 0, bytes).map_err(io_error)?;
            if sync {
                machine.sync(file);
            }
            Ok(())
        })
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.writing(Operation::Write, path, None)?;
        let mut disk = self.disk.borrow_mut();
        let resolved = disk.resolve(path, true, true)?;
        disk.replace(&resolved.path, bytes, 0o666, false)
    }

    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.writing(Operation::SetPermissions, path, None)?;
        let mut disk = self.disk.borrow_mut();
        let resolved = disk.resolve(path, true, false)?;
        disk.open(&resolved.path, How::PathNoFollow, |machine, file| {
            machine.set_permissions(file, mode).map_err(io_error)
        })
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.writing(Operation::RemoveFile, path, None)?;
        let mut disk = self.disk.borrow_mut();
        let resolved = disk.resolve(path, false, false)?;
        disk.remove(&resolved.path, false)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        self.writing(Operation::RemoveDirAll, path, None)?;
        let mut disk = self.disk.borrow_mut();
        // A terminal slash must not turn a link into permission to remove its target.
        let name = trim_slashes(path.as_os_str().as_bytes());
        let name = if name.is_empty() { path } else { Path::new(OsStr::from_bytes(name)) };
        let resolved = disk.resolve(name, false, false)?;
        match resolved.facts().kind {
            Kind::Link => disk.remove(&resolved.path, false),
            Kind::Directory => {
                if resolved.path == Path::new("/") {
                    return Err(io::Error::from_raw_os_error(libc::EBUSY));
                }
                disk.open(&resolved.path, How::Directory, remove_children)?;
                disk.remove(&resolved.path, true)
            }
            Kind::File | Kind::Other => Err(ErrorKind::NotADirectory.into()),
        }
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.writing(Operation::Rename, to, Some(from))?;
        self.check(Operation::Rename, from, false)?;
        self.disk.borrow_mut().rename(from, to)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
        self.writing(Operation::Copy, to, Some(from))?;
        let mut disk = self.disk.borrow_mut();
        let source = self.resolve_read(&mut disk, Operation::Copy, from, true)?;
        let facts = source.facts();
        if facts.kind != Kind::File {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "the source path is neither a regular file nor a symlink to a regular file",
            ));
        }
        let bytes = disk.read(&source.path, u64::MAX)?;
        let target = disk.resolve(to, true, true)?;
        let same = target.facts.is_some() && disk.same_node(&source.path, &target.path)?;
        let bytes = if same { &[] } else { bytes.as_slice() };
        disk.replace(&target.path, bytes, facts.mode, true)?;
        Ok(skein_lib::len_u64(bytes.len()))
    }
}

#[derive(Debug)]
struct Disk {
    machine: Machine,
    root: Opened,
}

struct Resolved {
    path: PathBuf,
    facts: Option<Facts>,
}

impl Resolved {
    fn facts(&self) -> Facts {
        self.facts.expect("resolved with an existing final entry required")
    }
}

impl Disk {
    fn open<T>(
        &mut self,
        path: &Path,
        how: How,
        use_file: impl FnOnce(&mut Machine, Opened) -> io::Result<T>,
    ) -> io::Result<T> {
        with_open(&mut self.machine, self.root, relative(path), how, use_file)
    }

    fn resolve(&mut self, path: &Path, follow: bool, allow_missing: bool) -> io::Result<Resolved> {
        let bytes = path.as_os_str().as_bytes();
        if bytes.is_empty() {
            return Err(io_error(Refusal::NotFound));
        }
        if bytes.contains(&0) {
            return Err(ErrorKind::InvalidInput.into());
        }
        let mut todo = components(bytes);
        let mut resolved = PathBuf::from("/");
        let mut links = 0;
        while let Some(part) = todo.pop_front() {
            if part == b"." || part == b".." {
                self.open(&resolved.join("."), How::PathNoFollow, |_, _| Ok(()))?;
                if part == b".." {
                    resolved.pop();
                }
                continue;
            }
            resolved.push(OsStr::from_bytes(&part));
            let last = todo.is_empty();
            let found = self.open(&resolved, How::PathNoFollow, |machine, handle| {
                let facts = machine.stat(handle);
                let target = if !last || follow { machine.link_target(handle) } else { None };
                Ok((facts, target))
            });
            let (facts, target) = match found {
                Ok(found) => found,
                Err(error) if last && allow_missing && error.kind() == ErrorKind::NotFound => {
                    return Ok(Resolved { path: resolved, facts: None });
                }
                Err(error) => return Err(error),
            };
            if let Some(target) = target {
                links += 1;
                if links > 40 {
                    return Err(io_error(Refusal::Loop));
                }
                if target.is_empty() {
                    return Err(io_error(Refusal::NotFound));
                }
                if target.starts_with(b"/") {
                    resolved = PathBuf::from("/");
                } else {
                    resolved.pop();
                }
                let mut expanded = components(&target);
                expanded.append(&mut todo);
                todo = expanded;
            } else if last {
                return Ok(Resolved { path: resolved, facts: Some(file_facts(facts)) });
            } else if facts.is != Is::Directory {
                return Err(io_error(Refusal::NotADirectory));
            }
        }
        let facts = self.open(&resolved, How::PathNoFollow, |machine, handle| Ok(file_facts(machine.stat(handle))))?;
        Ok(Resolved { path: resolved, facts: Some(facts) })
    }

    fn read(&mut self, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        self.open(path, How::Read, |machine, file| {
            if limit == 0 {
                return Ok(Vec::new());
            }
            machine.read(file, 0, u32::try_from(limit).unwrap_or(u32::MAX)).map_err(io_error)
        })
    }

    fn replace(&mut self, path: &Path, bytes: &[u8], mode: u32, copy_mode: bool) -> io::Result<()> {
        // The machine's append open requires write permission but not read permission.
        self.open(path, How::Append { mode }, |machine, file| {
            machine.truncate(file, 0).map_err(io_error)?;
            if copy_mode {
                machine.set_permissions(file, mode).map_err(io_error)?;
            }
            machine.write(file, 0, bytes).map_err(io_error)
        })
    }

    fn same_node(&mut self, source: &Path, target: &Path) -> io::Result<bool> {
        let source = self.open(source, How::PathNoFollow, |machine, handle| Ok(machine.node(handle)))?;
        self.open(target, How::PathNoFollow, |machine, handle| Ok(machine.node(handle) == source))
    }

    fn make_directory(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        if path.as_os_str().is_empty() {
            return Err(io_error(Refusal::NotFound));
        }
        let Some((parent, name)) = parent_and_name(path) else {
            return Err(io_error(Refusal::Exists));
        };
        let parent = self.resolve(parent, true, false)?.path;
        if name == b"." || name == b".." {
            self.resolve(path, true, false)?;
            return Err(io_error(Refusal::Exists));
        }
        self.open(&parent, How::PathNoFollow, |machine, dir| {
            machine.make_directory(dir, name, mode & 0o777).map_err(io_error)?;
            if mode & 0o7000 != 0 {
                with_open(machine, dir, name, How::PathNoFollow, |machine, handle| {
                    let masked = machine.stat(handle).mode | (mode & 0o7000);
                    machine.set_permissions(handle, masked).map_err(io_error)
                })?;
            }
            Ok(())
        })
    }

    fn make_directories(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        if path.as_os_str().is_empty() {
            return Ok(());
        }
        let mut result = self.make_directory(path, mode);
        if result.as_ref().is_err_and(|error| error.kind() == ErrorKind::NotFound) {
            let (parent, _) = parent_and_name(path).ok_or_else(|| io_error(Refusal::NotFound))?;
            self.make_directories(parent, mode)?;
            result = self.make_directory(path, mode);
        }
        if result.is_err() && self.resolve(path, true, false).is_ok_and(|entry| entry.facts().kind == Kind::Directory) {
            Ok(())
        } else {
            result
        }
    }

    fn remove(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        let (parent, name) = parent_and_name(path).ok_or_else(|| io::Error::from_raw_os_error(libc::EBUSY))?;
        self.open(parent, How::PathNoFollow, |machine, dir| machine.remove(dir, name, directory).map_err(io_error))
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let (from_parent, from_name) = parent_and_name(from).ok_or(ErrorKind::InvalidInput)?;
        let (to_parent, to_name) = parent_and_name(to).ok_or(ErrorKind::InvalidInput)?;
        if [from_name, to_name].iter().any(|name| *name == b"." || *name == b"..") {
            return Err(ErrorKind::InvalidInput.into());
        }
        let from_parent = self.resolve(from_parent, true, false)?.path;
        let to_parent = self.resolve(to_parent, true, false)?.path;
        let root = self.root;
        self.open(&from_parent, How::PathNoFollow, |machine, from_dir| {
            with_open(machine, root, relative(&to_parent), How::PathNoFollow, |machine, to_dir| {
                machine.rename(from_dir, from_name, to_dir, to_name, false).map_err(io_error)
            })
        })
    }
}

fn components(path: &[u8]) -> VecDeque<Vec<u8>> {
    let mut parts: VecDeque<_> =
        path.split(|&byte| byte == b'/').filter(|part| !part.is_empty()).map(<[u8]>::to_vec).collect();
    if path.ends_with(b"/") {
        parts.push_back(b".".to_vec());
    }
    parts
}

fn relative(path: &Path) -> &[u8] {
    match path.as_os_str().as_bytes().strip_prefix(b"/").expect("a resolved absolute path") {
        [] => b".",
        name => name,
    }
}

// Keep dot and parent components in the input rather than normalizing the name
// used for a mkdir or rename before the machine sees its parent.
fn parent_and_name(path: &Path) -> Option<(&Path, &[u8])> {
    let path = trim_slashes(path.as_os_str().as_bytes());
    if path.is_empty() {
        return None;
    }
    let start = path.iter().rposition(|&byte| byte == b'/').map_or(0, |last| last + 1);
    let (parent, name) = path.split_at(start);
    let parent = if parent.is_empty() { b"/" } else { parent };
    Some((Path::new(OsStr::from_bytes(parent)), name))
}

fn trim_slashes(path: &[u8]) -> &[u8] {
    &path[..path.iter().rposition(|&byte| byte != b'/').map_or(0, |last| last + 1)]
}

fn with_open<T>(
    machine: &mut Machine,
    base: Opened,
    path: &[u8],
    how: How,
    use_file: impl FnOnce(&mut Machine, Opened) -> io::Result<T>,
) -> io::Result<T> {
    let file = machine.open(base, path, how).map_err(io_error)?;
    let result = use_file(machine, file);
    machine.close(file);
    result
}

fn list(machine: &mut Machine, dir: Opened) -> io::Result<Vec<Listed>> {
    machine.list(dir, u32::MAX, u32::MAX).map_err(io_error)
}

fn remove_children(machine: &mut Machine, dir: Opened) -> io::Result<()> {
    for (kind, name) in list(machine, dir)? {
        let directory = kind == Is::Directory;
        if directory {
            with_open(machine, dir, &name, How::DirectoryNoFollow, remove_children)?;
        }
        machine.remove(dir, &name, directory).map_err(io_error)?;
    }
    Ok(())
}

fn file_facts(facts: skein_fake_machine::Facts) -> Facts {
    Facts { kind: entry_kind(facts.is), mode: facts.mode & 0o7777, size: facts.size, owner: facts.owner }
}

fn entry_kind(kind: Is) -> Kind {
    match kind {
        Is::Directory => Kind::Directory,
        Is::File => Kind::File,
        Is::Link => Kind::Link,
        Is::Fifo | Is::Device => Kind::Other,
    }
}

fn io_error(refusal: Refusal) -> io::Error {
    io::Error::from_raw_os_error(match refusal {
        Refusal::Held => libc::EWOULDBLOCK,
        Refusal::NotFound => libc::ENOENT,
        Refusal::Exists => libc::EEXIST,
        Refusal::NotADirectory => libc::ENOTDIR,
        Refusal::IsADirectory => libc::EISDIR,
        Refusal::NotEmpty => libc::ENOTEMPTY,
        Refusal::Permission => libc::EACCES,
        Refusal::NoSpace => libc::ENOSPC,
        // Rust doesn't expose a stable kind constructor for ELOOP. Keep the
        // host errno so `kind()` agrees with native without naming that variant.
        Refusal::Loop => libc::ELOOP,
        Refusal::NameTooLong => libc::ENAMETOOLONG,
        Refusal::Escape => libc::EXDEV,
        Refusal::NotAFile | Refusal::Beneath => libc::EINVAL,
    })
}
