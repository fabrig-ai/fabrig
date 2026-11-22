use std::{
    collections::BTreeMap,
    ffi::OsStr,
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use skein_fake_machine::{Facts, How, Is, Item, Machine, Opened, Refusal};

use super::super::{
    Options, Plan,
    options::{self, FileFacts, PreflightDisk},
};
use crate::EnvironmentEntry;

const USER: u32 = 1000;
pub(super) const ROOT: &str = "/scratch";
pub(super) const BOX_PROGRAMS: [&str; 3] = ["temper-box", "fake-claude", "temper"];

pub(super) struct Disk {
    machine: Machine,
    root: Opened,
    links: BTreeMap<PathBuf, PathBuf>,
}

impl Disk {
    pub(super) fn new(dirs: &[&str]) -> Self {
        let mut machine = Machine::for_user(USER);
        let root = machine.lay(&[Item::directory(b"scratch").mode(0o700)]);
        let mut disk = Self { machine, root, links: BTreeMap::new() };
        for dir in dirs {
            disk.directory(&Path::new(ROOT).join(dir));
        }
        disk
    }

    pub(super) fn prepare(&mut self, options: Options, env: &[EnvironmentEntry]) -> Result<Plan, String> {
        options::prepare_with(options, env, self)
    }

    pub(super) fn directory(&mut self, path: &Path) {
        self.make_session_dir(path).expect("fixture directory");
    }

    pub(super) fn programs(&mut self, names: &[&str]) {
        for name in names {
            self.program(&Path::new(ROOT).join("bin").join(name), 0o700);
        }
    }

    pub(super) fn program(&mut self, path: &Path, mode: u32) {
        self.write_mode(path, b"not executed", mode).expect("program fixture");
    }

    pub(super) fn write(&mut self, path: &Path, bytes: &[u8]) {
        self.write_mode(path, bytes, 0o600).expect("fixture file");
    }

    pub(super) fn read(&mut self, path: &Path) -> Vec<u8> {
        let file = self.open(path, How::Read).expect("fixture file");
        let len = u32::try_from(self.machine.stat(file).size).expect("fixture length");
        let bytes = self.machine.read(file, 0, len).expect("fixture bytes");
        self.machine.close(file);
        bytes
    }

    pub(super) fn read_link(&mut self, path: &Path) -> &Path {
        assert_eq!(self.stat(path, How::PathNoFollow).expect("opener link").is, Is::Link);
        self.links.get(path).expect("opener target")
    }

    fn open(&mut self, path: &Path, how: How) -> io::Result<Opened> {
        let path = path.strip_prefix("/").expect("absolute fixture path");
        let name = if path.as_os_str().is_empty() { b"." } else { path.as_os_str().as_bytes() };
        self.machine.open(self.root, name, how).map_err(io_error)
    }

    fn stat(&mut self, path: &Path, how: How) -> io::Result<Facts> {
        let entry = self.open(path, how)?;
        let facts = self.machine.stat(entry);
        self.machine.close(entry);
        Ok(facts)
    }

    fn parent<'a>(&mut self, path: &'a Path) -> io::Result<(Opened, &'a OsStr)> {
        let name = path.file_name().expect("fixture name");
        let dir = self.open(path.parent().expect("fixture parent"), How::Directory)?;
        Ok((dir, name))
    }

    fn remove(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        let (dir, name) = self.parent(path)?;
        let removed = self.machine.remove(dir, name.as_bytes(), directory);
        self.machine.close(dir);
        removed.map_err(io_error)?;
        self.links.remove(path);
        Ok(())
    }

    fn write_mode(&mut self, path: &Path, bytes: &[u8], mode: u32) -> io::Result<()> {
        let file = self.open(path, How::Create { mode })?;
        let written = self.machine.write(file, 0, bytes);
        self.machine.close(file);
        written.map_err(io_error)
    }
}

fn io_error(why: Refusal) -> io::Error {
    match why {
        Refusal::NotFound => io::ErrorKind::NotFound.into(),
        other => io::Error::other(format!("{other:?}")),
    }
}

impl PreflightDisk for Disk {
    fn canonicalize(&mut self, path: &Path) -> io::Result<PathBuf> {
        let path = Path::new("/").join(path);
        self.stat(&path, How::Read)?;
        Ok(path)
    }

    fn facts(&mut self, path: &Path, follow: bool) -> io::Result<FileFacts> {
        let stat = self.stat(path, if follow { How::Read } else { How::PathNoFollow })?;
        Ok(FileFacts {
            directory: stat.is == Is::Directory,
            file: stat.is == Is::File,
            owner: stat.owner,
            mode: stat.mode,
        })
    }

    fn current_dir(&mut self) -> io::Result<PathBuf> {
        Ok(PathBuf::from("/"))
    }

    fn effective_user(&self) -> u32 {
        USER
    }

    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()> {
        let dir = self.open(path, How::Directory)?;
        let entries = self.machine.list(dir, 128, 4096);
        self.machine.close(dir);
        for (kind, name) in entries.map_err(io_error)? {
            let child = path.join(OsStr::from_bytes(&name));
            if kind == Is::Directory {
                self.remove_dir_all(&child)?;
            } else {
                self.remove_file(&child)?;
            }
        }
        self.remove(path, true)
    }

    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        self.remove(path, false)
    }

    fn make_session_dir(&mut self, path: &Path) -> io::Result<()> {
        let (dir, name) = self.parent(path)?;
        let made = self.machine.make_directory(dir, name.as_bytes(), 0o700);
        self.machine.close(dir);
        made.map_err(io_error)
    }

    fn link(&mut self, target: &Path, path: &Path) -> io::Result<()> {
        let (dir, name) = self.parent(path)?;
        let source = self.machine.lay(&[Item::link(b"link", target.as_os_str().as_bytes())]);
        let moved = self.machine.rename(source, b"link", dir, name.as_bytes(), true);
        self.machine.close(source);
        self.machine.close(dir);
        moved.map_err(io_error)?;
        // The fake machine exposes link kind through stat. Keep the target for readlink.
        self.links.insert(path.into(), target.into());
        Ok(())
    }

    fn make_session_file(&mut self, path: &Path) -> io::Result<()> {
        self.write_mode(path, b"", 0o600)
    }
}
