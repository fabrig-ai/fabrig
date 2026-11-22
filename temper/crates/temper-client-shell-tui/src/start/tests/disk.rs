use super::super::deployment::{self, Files};
use skein_fake_machine::{Facts, How, Is, Item, Machine, Opened, Refusal};
use skein_lib::Secret;
use std::{
    ffi::{OsStr, OsString},
    io,
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};
use temper_client_tui::Origin;

pub(super) const USER: u32 = 1000;

pub(super) struct Disk {
    machine: Machine,
    root: Opened,
}

impl Disk {
    pub(super) fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(USER);
        let root = machine.lay(
            &std::iter::once(Item::directory(b"tui").mode(0o700)).chain(items.iter().cloned()).collect::<Vec<_>>(),
        );
        Self { machine, root }
    }

    pub(super) fn prepare(&mut self, origin: &Origin) -> (PathBuf, bool) {
        deployment::prepare_with(Path::new("tui"), origin, USER, self).expect("origin directory")
    }

    fn open(&mut self, path: &Path, how: How) -> io::Result<Opened> {
        self.machine.open(self.root, path.as_os_str().as_encoded_bytes(), how).map_err(io_error)
    }

    fn facts(&mut self, path: &Path) -> io::Result<Facts> {
        let file = self.open(path, How::PathNoFollow)?;
        let facts = self.machine.stat(file);
        self.machine.close(file);
        Ok(facts)
    }

    pub(super) fn exists(&mut self, path: &Path) -> bool {
        self.facts(path).is_ok()
    }

    pub(super) fn is_file(&mut self, path: &Path) -> bool {
        self.facts(path).is_ok_and(|facts| facts.is == Is::File)
    }

    pub(super) fn read(&mut self, path: &Path) -> Vec<u8> {
        let file = self.open(path, How::Read).expect("read fixture file");
        let size = u32::try_from(self.machine.stat(file).size).expect("fixture size");
        let bytes = self.machine.read(file, 0, size).expect("fixture bytes");
        self.machine.close(file);
        bytes
    }

    pub(super) fn write(&mut self, path: &Path, bytes: &[u8]) {
        let file = self.open(path, How::Create { mode: 0o600 }).expect("fixture file");
        self.machine.write(file, 0, bytes).expect("fixture bytes");
        self.machine.close(file);
    }

    fn remove(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        let (parent, name) = parent_and_name(path);
        let dir = self.open(parent, How::Directory)?;
        let removed = self.machine.remove(dir, name.as_encoded_bytes(), directory).map_err(io_error);
        self.machine.close(dir);
        removed
    }
}

fn parent_and_name(path: &Path) -> (&Path, &OsStr) {
    (path.parent().expect("parent"), path.file_name().expect("entry name"))
}

fn io_error(why: Refusal) -> io::Error {
    if why == Refusal::NotFound { io::ErrorKind::NotFound.into() } else { io::Error::other(format!("{why:?}")) }
}

fn owned_private(facts: Facts, user: u32) -> bool {
    let group_and_other = facts.mode & 0o077;
    facts.owner == user && group_and_other == 0
}

impl Files for Disk {
    type Entries = std::vec::IntoIter<io::Result<(OsString, PathBuf)>>;

    fn check_entry(&mut self, path: &Path) -> io::Result<()> {
        self.facts(path).map(|_| ())
    }

    fn entries(&mut self, path: &Path) -> io::Result<Self::Entries> {
        let dir = self.open(path, How::Directory)?;
        let entries = self.machine.list(dir, 128, 4096).map_err(io_error);
        self.machine.close(dir);
        Ok(entries?
            .into_iter()
            .map(|(_, name)| {
                let name = OsString::from_vec(name.into_vec());
                let path = path.join(&name);
                Ok((name, path))
            })
            .collect::<Vec<_>>()
            .into_iter())
    }

    fn read_private(&mut self, path: &Path, max: u32, user: u32) -> Option<Secret> {
        let file = self.open(path, How::ReadNoFollow).ok()?;
        let facts = self.machine.stat(file);
        let bytes = if facts.is == Is::File && owned_private(facts, user) && facts.links == 1 {
            u32::try_from(facts.size)
                .ok()
                .filter(|len| *len <= max)
                .and_then(|len| self.machine.read(file, 0, len).ok())
        } else {
            None
        };
        self.machine.close(file);
        bytes.map(|bytes| Secret::new(bytes.into_boxed_slice()))
    }

    fn private_root(&mut self, path: &Path, user: u32) -> Result<(), String> {
        let facts = self.facts(path).map_err(|why| why.to_string())?;
        if facts.is == Is::Directory && owned_private(facts, user) {
            Ok(())
        } else {
            Err(format!("{} is not a private directory", path.display()))
        }
    }

    fn make_private(&mut self, path: &Path, user: u32) -> Result<(), String> {
        if !self.exists(path) {
            let (parent, name) = parent_and_name(path);
            let dir = self.open(parent, How::Directory).map_err(|why| why.to_string())?;
            let made = self.machine.make_directory(dir, name.as_encoded_bytes(), 0o700);
            self.machine.close(dir);
            made.map_err(|why| format!("{why:?}"))?;
        }
        self.private_root(path, user)
    }

    fn sync(&mut self, path: &Path) -> io::Result<()> {
        let file = self.open(path, How::Read)?;
        self.machine.sync(file);
        self.machine.close(file);
        Ok(())
    }

    fn rename(&mut self, source: &Path, target: &Path) -> io::Result<()> {
        let (from, source) = parent_and_name(source);
        let (to, target) = parent_and_name(target);
        let from = self.open(from, How::Directory)?;
        let renamed = self.open(to, How::Directory).and_then(|to| {
            let result = self.machine.rename(from, source.as_encoded_bytes(), to, target.as_encoded_bytes(), false);
            self.machine.close(to);
            result.map_err(io_error)
        });
        self.machine.close(from);
        renamed
    }

    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        self.remove(path, false)
    }

    fn remove_dir(&mut self, path: &Path) -> io::Result<()> {
        self.remove(path, true)
    }
}
