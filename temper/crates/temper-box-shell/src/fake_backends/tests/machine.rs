//! Source and effective startup files under one fake root.

use super::*;
use crate::fake::disk::Metadata;
use skein_fake_machine::{Facts, How, Is, Item, Listed, Machine, Opened, Refusal};
use skein_io::kernel::Stat;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

const USER: u32 = 1000;

pub(super) fn io_error(why: Refusal) -> std::io::Error {
    std::io::Error::from_raw_os_error(crate::errno(skein_fake_machine::kernel_error(why)))
}

pub(super) struct Tree {
    machine: Rc<RefCell<Machine>>,
    root: Opened,
}

impl Tree {
    pub(super) fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(USER);
        let root = machine.lay(items);
        Self { machine: Rc::new(RefCell::new(machine)), root }
    }

    fn relative(path: &Path) -> &[u8] {
        let path = path.strip_prefix("/").expect("absolute fixture path").as_os_str().as_bytes();
        if path.is_empty() { b"." } else { path }
    }

    pub(super) fn metadata(&self, path: &Path) -> Result<Facts, Refusal> {
        let mut machine = self.machine.borrow_mut();
        let handle = machine.open(self.root, Self::relative(path), How::PathNoFollow)?;
        let facts = machine.stat(handle);
        machine.close(handle);
        Ok(facts)
    }

    pub(super) fn exists(&self, path: &Path) -> bool {
        match self.metadata(path) {
            Ok(_) => true,
            Err(Refusal::NotFound) => false,
            Err(why) => panic!("{}: {why:?}", path.display()),
        }
    }

    pub(super) fn public_path(&self, path: &Path) -> Result<(), String> {
        public_path_with(path, |path| {
            let facts = self.metadata(path).map_err(io_error)?;
            Ok(PublicMetadata {
                linked: facts.is == Is::Link,
                file: facts.is == Is::File,
                links: u64::from(facts.links),
            })
        })
    }

    pub(super) fn private_document(&self, path: &Path) -> Result<(), String> {
        let facts = self.metadata(path).map_err(|why| format!("{}: {why:?}", path.display()))?;
        private_document_from(path, facts.owner, USER, facts.mode)
    }

    fn empty(&self, path: &Path) -> Result<bool, String> {
        match self.list(path, 1) {
            Ok(entries) => Ok(entries.is_empty()),
            Err(Refusal::NotFound) => Ok(true),
            Err(why) => Err(format!("{}: {why:?}", path.display())),
        }
    }

    pub(super) fn list(&self, path: &Path, max: u32) -> Result<Vec<Listed>, Refusal> {
        let mut machine = self.machine.borrow_mut();
        let dir = machine.open(self.root, Self::relative(path), How::Directory)?;
        let listed = machine.list(dir, max, 4096);
        machine.close(dir);
        listed
    }

    pub(super) fn read_bounded(&self, path: &Path, max: u32) -> Result<Box<[u8]>, skein_shell::Unread> {
        use skein_shell::Unread;
        let unread = |why| {
            if why == Refusal::NotFound {
                Unread::Absent
            } else if why == Refusal::NotAFile {
                Unread::NotAFile
            } else {
                Unread::Kernel(skein_fake_machine::kernel_error(why))
            }
        };
        let mut machine = self.machine.borrow_mut();
        let handle = machine.open(self.root, Self::relative(path), How::Read).map_err(unread)?;
        let facts = machine.stat(handle);
        let result = if facts.is != Is::File {
            Err(Unread::NotAFile)
        } else if facts.size > u64::from(max) {
            Err(Unread::TooLarge { max })
        } else {
            machine.read(handle, 0, max).map(Vec::into_boxed_slice).map_err(unread)
        };
        machine.close(handle);
        result
    }

    pub(super) fn read(&self, path: &Path) -> Box<[u8]> {
        self.read_bounded(path, crate::CONFIG_BYTES).expect("fixture file")
    }

    pub(super) fn create(&self, path: &Path, bytes: &[u8]) {
        self.create_exclusive(path, bytes).expect("create fixture");
    }

    pub(super) fn create_exclusive(&self, path: &Path, bytes: &[u8]) -> Result<(), Refusal> {
        let mut machine = self.machine.borrow_mut();
        let handle = machine.open(self.root, Self::relative(path), How::Create { mode: 0o600 })?;
        let result = machine.write(handle, 0, bytes);
        machine.close(handle);
        result
    }

    pub(super) fn remove(&self, path: &Path) {
        self.with_parent(path, |machine, directory, name| machine.remove(directory, name, false))
            .expect("remove fixture");
    }

    fn with_parent<T>(
        &self,
        path: &Path,
        apply: impl FnOnce(&mut Machine, Opened, &[u8]) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let mut machine = self.machine.borrow_mut();
        let parent = path.parent().expect("fixture parent");
        let directory = machine.open(self.root, Self::relative(parent), How::Directory)?;
        let name = path.file_name().expect("fixture name").as_bytes();
        let result = apply(&mut machine, directory, name);
        machine.close(directory);
        result
    }

    fn config_facts(&self, dirs: &Directories) -> temper_config_shell::Facts {
        temper_config_shell::Facts {
            login: "fixture".into(),
            hostname: "localhost".into(),
            provider_key: self.exists(&crate::path(&dirs.data).join("provider.key")),
        }
    }

    fn read_config(&self, dirs: &Directories) -> Result<temper_config_shell::Read, String> {
        temper_config_shell::read_with(
            Shape::Standalone,
            || {
                self.metadata(&crate::path(&dirs.config).join(temper_config_shell::FORMER))
                    .map(|_| ())
                    .map_err(io_error)
            },
            |bound| self.read_bounded(&config_path(dirs), bound),
            || Ok(self.config_facts(dirs)),
        )
        .map_err(|why| why.to_string())
    }

    pub(super) fn parse(&self, dirs: &Directories, text: &[u8]) -> Result<temper_config_shell::Read, String> {
        temper_config_shell::parse(text, Shape::Standalone, &self.config_facts(dirs)).map_err(|why| why.to_string())
    }

    pub(super) fn source(&self, dirs: &Directories) -> Result<Source, String> {
        let path = config_path(dirs);
        Source::read_with(
            dirs,
            || self.empty(crate::path(&dirs.config)),
            || self.read_config(dirs),
            || {
                self.public_path(&path)?;
                Ok(self.read(&path))
            },
        )
    }
}

/// Effects of a preparation, over the same tree as the source/retained fixtures above.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Effect {
    Directory(PathBuf, u32),
    Open(PathBuf, u32, bool),
    Mode(PathBuf, u32),
    Write(PathBuf),
    Sync(PathBuf),
    Rename(PathBuf, PathBuf),
    Remove(PathBuf),
}
impl Effect {
    pub(super) fn under(&self, root: &Path) -> bool {
        match self {
            Self::Directory(path, _)
            | Self::Open(path, _, _)
            | Self::Mode(path, _)
            | Self::Write(path)
            | Self::Sync(path)
            | Self::Remove(path) => path.starts_with(root),
            Self::Rename(from, to) => from.starts_with(root) || to.starts_with(root),
        }
    }
}

pub(super) struct Sandbox {
    pub tree: Tree,
    pub effects: RefCell<Vec<Effect>>,
    pub binds: RefCell<Vec<u16>>,
    pub init_ports: RefCell<Vec<[u16; 2]>>,
    pub program_reads: RefCell<Vec<PathBuf>>,
    pub binary_reads: Cell<u32>,
    next_port: Cell<u16>,
    // The fake machine holds bytes, links and entries. Supplement its stat with
    // a named node number and the mode asked by chmod, which it doesn't model.
    identities: RefCell<BTreeMap<PathBuf, u64>>,
    next_identity: Cell<u64>,
    modes: RefCell<BTreeMap<PathBuf, u32>>,
}
pub(super) const ADAPTER_PATH: &str = "/programs/fake-claude";
pub(super) const SECRET: &[u8] = b"synthetic-sandbox-worker-secret";
impl Sandbox {
    pub(super) fn new(items: &[Item]) -> Self {
        Self {
            tree: Tree::new(items),
            effects: RefCell::default(),
            binds: RefCell::default(),
            init_ports: RefCell::default(),
            program_reads: RefCell::default(),
            binary_reads: Cell::new(0),
            next_port: Cell::new(9010),
            identities: RefCell::default(),
            next_identity: Cell::new(1),
            modes: RefCell::default(),
        }
    }
    fn mark(&self, effect: Effect) {
        self.effects.borrow_mut().push(effect);
    }
    fn identity(&self, path: &Path) -> u64 {
        *self.identities.borrow_mut().entry(path.into()).or_insert_with(|| {
            let id = self.next_identity.get();
            self.next_identity.set(id.checked_add(1).expect("fixture node identity"));
            id
        })
    }
    fn mode_at(&self, path: &Path, mode: u32) {
        self.mark(Effect::Mode(path.into(), mode));
        self.modes.borrow_mut().insert(path.into(), mode);
    }
    fn mkdirs(&self, path: &Path) -> Result<(), String> {
        if !self.tree.exists(path) {
            self.mkdirs(path.parent().expect("absolute fixture path"))?;
            self.mark(Effect::Directory(path.into(), 0o700));
            self.tree
                .with_parent(path, |machine, parent, name| machine.make_directory(parent, name, 0o700))
                .map_err(|why| format!("{}: {why:?}", path.display()))?;
        }
        Ok(())
    }
    fn init_file(&self, path: &Path, bytes: &[u8], mode: u32) -> Result<(), String> {
        let mut file = self.open_write(path, mode, false).map_err(|why| why.to_string())?;
        self.write(&mut file, bytes).map_err(|why| why.to_string())?;
        self.sync(&file).map_err(|why| why.to_string())
    }
}

pub(super) struct File {
    machine: Rc<RefCell<Machine>>,
    handle: Opened,
    path: PathBuf,
}
impl Drop for File {
    fn drop(&mut self) {
        self.machine.borrow_mut().close(self.handle);
    }
}

impl Files for Sandbox {
    type File = File;
    fn user(&self) -> u32 {
        USER
    }
    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let facts = self.tree.metadata(path).map_err(io_error)?;
        let kind = match facts.is {
            Is::File => Kind::File,
            Is::Directory => Kind::Directory,
            Is::Link => Kind::Symlink,
            Is::Fifo | Is::Device => Kind::Other,
        };
        let mode = self.modes.borrow().get(path).copied().unwrap_or(facts.mode);
        Ok(Metadata {
            stat: Stat { kind, mode, size: facts.size, owner: facts.owner, links: facts.links },
            identity: (1, self.identity(path)),
        })
    }
    fn exists(&self, path: &Path) -> bool {
        self.tree.exists(path)
    }
    fn public_path(&self, path: &Path) -> Result<(), String> {
        self.tree.public_path(path)
    }
    fn read(&self, path: &Path, bound: u32, private: bool) -> Result<Box<[u8]>, skein_shell::Unread> {
        if private {
            let stat = self.metadata(path).expect("peer checks precede its read").stat;
            if let Some(why) = skein_io::file::Unsafe::private_file(stat, USER) {
                return Err(skein_shell::Unread::Unsafe(why));
            }
        }
        self.tree.read_bounded(path, bound)
    }
    fn open_write(&self, path: &Path, mode: u32, replace: bool) -> io::Result<File> {
        self.mark(Effect::Open(path.into(), mode, replace));
        let how = if replace { How::Write } else { How::Create { mode } };
        let handle =
            self.tree.machine.borrow_mut().open(self.tree.root, Tree::relative(path), how).map_err(io_error)?;
        let file = File { machine: Rc::clone(&self.tree.machine), handle, path: path.into() };
        if replace {
            self.tree.machine.borrow_mut().truncate(handle, 0).map_err(io_error)?;
        }
        Ok(file)
    }
    fn mode(&self, file: &File, mode: u32) -> io::Result<()> {
        self.mode_at(&file.path, mode);
        Ok(())
    }
    fn write(&self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        self.mark(Effect::Write(file.path.clone()));
        self.tree.machine.borrow_mut().write(file.handle, 0, bytes).map_err(io_error)
    }
    fn sync(&self, file: &File) -> io::Result<()> {
        self.mark(Effect::Sync(file.path.clone()));
        self.tree.machine.borrow_mut().sync(file.handle);
        Ok(())
    }
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.mark(Effect::Rename(from.into(), to.into()));
        assert_eq!(from.parent(), to.parent(), "staging is beside the document");
        self.tree
            .with_parent(from, |machine, parent, from| {
                machine.rename(parent, from, parent, to.file_name().expect("document name").as_bytes(), false)
            })
            .map_err(io_error)?;
        let id = self.identity(from);
        let mut identities = self.identities.borrow_mut();
        identities.remove(from);
        identities.insert(to.into(), id);
        let mut modes = self.modes.borrow_mut();
        let mode = modes.remove(from);
        modes.remove(to);
        if let Some(mode) = mode {
            modes.insert(to.into(), mode);
        }
        Ok(())
    }
    fn remove(&self, path: &Path) -> io::Result<()> {
        self.mark(Effect::Remove(path.into()));
        self.tree
            .with_parent(path, |machine, directory, name| machine.remove(directory, name, false))
            .map_err(io_error)?;
        self.identities.borrow_mut().remove(path);
        self.modes.borrow_mut().remove(path);
        Ok(())
    }
}

impl Host for Sandbox {
    type Lock = ();
    type Listeners = [NonZeroU16; 2];
    type Peer = SocketAddr;
    fn binary(&self) -> Result<PathBuf, String> {
        self.binary_reads.set(self.binary_reads.get().checked_add(1).expect("binary reads"));
        Ok("/programs/temper".into())
    }
    fn source(&self, dirs: &Directories) -> Result<Source, String> {
        self.tree.source(dirs)
    }
    fn config_facts(&self, dirs: &Directories) -> Result<temper_config_shell::Facts, temper_config_shell::Refusal> {
        Ok(self.tree.config_facts(dirs))
    }
    fn read_config(&self, dirs: &Directories) -> Result<temper_config_shell::Read, String> {
        self.tree.read_config(dirs)
    }
    fn read_bundle(&self, path: &Path, limits: temper_service::limits::Clients) -> Result<(), String> {
        // The engine's directory reader is below this supplied bundle verdict.
        let page = self.read(&path.join("index.html"), limits.bundle_bytes, false).map_err(crate::unread)?;
        assert_eq!(page.as_ref(), PAGE);
        Ok(())
    }
    fn program_metadata(&self, path: &Path) -> Option<(bool, u32)> {
        self.program_reads.borrow_mut().push(path.into());
        self.metadata(path).ok().map(|metadata| (metadata.stat.kind == Kind::File, metadata.stat.mode))
    }
    fn make_private(&self, path: &Path, user: u32) -> Result<(), String> {
        self.mkdirs(path)?;
        private_root_with(self, path, user)
    }
    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.mode_at(path, mode);
        Ok(())
    }
    fn lock(&self, root: &Path) -> Result<(), String> {
        let path = root.join("lock");
        if !self.exists(&path) {
            drop(self.open_write(&path, 0o600, false).map_err(|why| why.to_string())?);
        }
        Ok(())
    }
    fn init(
        &self,
        dirs: &Directories,
        binary: &Path,
        preferred: [u16; 2],
        adapter: &Path,
    ) -> Result<Initialized<Self::Listeners>, String> {
        assert_eq!(binary, Path::new("/programs/temper"));
        assert_eq!(adapter, Path::new(ADAPTER_PATH));
        self.init_ports.borrow_mut().push(preferred);
        let config = crate::path(&dirs.config);
        let data = crate::path(&dirs.data);
        for path in [config, data, crate::path(&dirs.cache)] {
            self.mkdirs(path)?;
        }
        let [clients, webhooks] = preferred
            .into_iter()
            .zip([8100, 8374])
            .map(|(port, fallback)| {
                NonZeroU16::new(if port == 0 { fallback } else { port }).expect("supplied init port")
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("two init reservations");
        let document = config.join(FILE);
        let secret = data.join("worker.secret");
        self.init_file(&secret, SECRET, 0o600)?;
        let file = crate::init::configuration(clients, webhooks);
        self.init_file(&document, &temper_config_shell::render(&file, Shape::Standalone), 0o644)?;
        Ok(Initialized {
            paths: vec![secret, document],
            ports: vec![clients, webhooks],
            listeners: [clients, webhooks],
        })
    }
    fn bind(&self, port: u16) -> io::Result<SocketAddr> {
        self.binds.borrow_mut().push(port);
        let port = if port == 0 {
            let port = self.next_port.get();
            self.next_port.set(port.checked_add(1).expect("fixture port"));
            port
        } else {
            port
        };
        Ok(([127, 0, 0, 1], port).into())
    }
    fn address(&self, peer: &SocketAddr) -> io::Result<SocketAddr> {
        Ok(*peer)
    }
}
