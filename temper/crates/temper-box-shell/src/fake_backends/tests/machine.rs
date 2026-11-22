//! Source and effective startup files under one fake root.

use super::*;
use skein_fake_machine::{Facts, How, Is, Item, Listed, Machine, Opened, Refusal};
use std::cell::RefCell;

const USER: u32 = 1000;

pub(super) fn io_error(why: Refusal) -> std::io::Error {
    std::io::Error::from_raw_os_error(crate::errno(skein_fake_machine::kernel_error(why)))
}

pub(super) struct Tree {
    machine: RefCell<Machine>,
    root: Opened,
}

impl Tree {
    pub(super) fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(USER);
        let root = machine.lay(items);
        Self { machine: RefCell::new(machine), root }
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
        let mut machine = self.machine.borrow_mut();
        let parent = path.parent().expect("fixture parent");
        let directory = machine.open(self.root, Self::relative(parent), How::Directory).expect("open fixture parent");
        let name = path.file_name().expect("fixture name").as_bytes();
        let removed = machine.remove(directory, name, false);
        machine.close(directory);
        removed.expect("remove fixture");
    }

    pub(super) fn parse(&self, dirs: &Directories, text: &[u8]) -> Result<temper_config_shell::Read, String> {
        let facts = temper_config_shell::Facts {
            login: "fixture".into(),
            hostname: "localhost".into(),
            provider_key: self.exists(&crate::path(&dirs.data).join("provider.key")),
        };
        temper_config_shell::parse(text, Shape::Standalone, &facts).map_err(|why| why.to_string())
    }

    pub(super) fn source(&self, dirs: &Directories) -> Result<Source, String> {
        let path = config_path(dirs);
        Source::read_with(
            dirs,
            || self.empty(crate::path(&dirs.config)),
            || self.parse(dirs, &self.read(&path)),
            || {
                self.public_path(&path)?;
                Ok(self.read(&path))
            },
        )
    }
}
