//! Translate directory operations to the fake machine, with one open root per tree.
use super::{Kind, Metadata};
use skein_fake_machine::{How, Is, Listed, Machine, Opened, Refusal};
use std::io::{self, ErrorKind};

pub struct MachineDirectory {
    pub machine: Machine,
    pub root: Opened,
    path: String,
}

#[derive(Clone, Copy)]
pub(super) enum Change<'a> {
    Replace(&'a [u8]),
    Create(&'a [u8]),
    Append(&'a [u8]),
    Truncate(u64),
}

fn error(refusal: Refusal) -> io::Error {
    let kind = match refusal {
        Refusal::NotFound => ErrorKind::NotFound,
        Refusal::Exists => ErrorKind::AlreadyExists,
        Refusal::NotADirectory => ErrorKind::NotADirectory,
        Refusal::IsADirectory => ErrorKind::IsADirectory,
        Refusal::NotEmpty => ErrorKind::DirectoryNotEmpty,
        Refusal::Permission => ErrorKind::PermissionDenied,
        Refusal::NoSpace => ErrorKind::StorageFull,
        Refusal::Held => ErrorKind::WouldBlock,
        Refusal::Loop | Refusal::NameTooLong | Refusal::Escape | Refusal::NotAFile | Refusal::Beneath => {
            return io::Error::other(format!("fake directory: {refusal:?}"));
        }
    };
    kind.into()
}

impl MachineDirectory {
    pub(super) fn new(machine: Machine, root: Opened, path: &str) -> Self {
        Self { machine, root, path: path.into() }
    }

    fn at(&self, name: &str) -> String {
        match (self.path.is_empty(), name.is_empty()) {
            (true, true) => ".".into(),
            (true, false) => name.into(),
            (false, true) => self.path.clone(),
            (false, false) => format!("{}/{name}", self.path),
        }
    }

    pub(super) fn metadata(&mut self, name: &str) -> io::Result<Metadata> {
        let file = self.machine.open(self.root, self.at(name).as_bytes(), How::PathNoFollow).map_err(error)?;
        let facts = self.machine.stat(file);
        self.machine.close(file);
        let kind = match facts.is {
            Is::Directory => Kind::Directory,
            Is::File => Kind::File,
            Is::Link | Is::Fifo | Is::Device => Kind::Other,
        };
        Ok(Metadata { kind, mode: facts.mode, size: facts.size })
    }

    pub(super) fn exists(&mut self, name: &str) -> bool {
        match self.machine.open(self.root, self.at(name).as_bytes(), How::Read) {
            Ok(file) => {
                self.machine.close(file);
                true
            }
            Err(_) => false,
        }
    }

    pub(super) fn read(&mut self, name: &str, limit: u32) -> io::Result<Vec<u8>> {
        let file = self.machine.open(self.root, self.at(name).as_bytes(), How::Read).map_err(error)?;
        let bytes = self.machine.read(file, 0, limit);
        self.machine.close(file);
        bytes.map_err(error)
    }

    pub(super) fn change(&mut self, name: &str, change: Change<'_>) -> io::Result<()> {
        let path = self.at(name);
        let path = path.as_bytes();
        let create = How::Create { mode: 0o666 };
        let file = match change {
            Change::Create(_) => self.machine.open(self.root, path, create),
            Change::Replace(_) => match self.machine.open(self.root, path, How::Write) {
                Err(Refusal::NotFound) => self.machine.open(self.root, path, create),
                opened => opened,
            },
            Change::Append(_) | Change::Truncate(_) => self.machine.open(self.root, path, How::Write),
        }
        .map_err(error)?;
        let result = match change {
            Change::Replace(bytes) => match self.machine.truncate(file, 0) {
                Ok(()) => self.machine.write(file, 0, bytes),
                Err(refusal) => Err(refusal),
            },
            Change::Create(bytes) => self.machine.write(file, 0, bytes),
            Change::Append(bytes) => self.machine.append(file, bytes),
            Change::Truncate(size) => self.machine.truncate(file, size),
        };
        if result.is_ok() && !matches!(change, Change::Replace(_)) {
            self.machine.sync(file);
        }
        self.machine.close(file);
        result.map_err(error)
    }

    pub(super) fn sync(&mut self, name: &str) -> io::Result<()> {
        let file = self.machine.open(self.root, self.at(name).as_bytes(), How::Read).map_err(error)?;
        self.machine.sync(file);
        self.machine.close(file);
        Ok(())
    }

    pub(super) fn create_dir_all(&mut self, name: &str) -> io::Result<()> {
        self.mkdirs(self.at(name).as_bytes()).map_err(error)
    }

    fn mkdirs(&mut self, path: &[u8]) -> Result<(), Refusal> {
        match self.machine.open(self.root, path, How::Directory) {
            Ok(dir) => {
                self.machine.close(dir);
                return Ok(());
            }
            Err(Refusal::NotFound) => {}
            Err(refusal) => return Err(refusal),
        }
        let (parent, name) = parent_and_name(path);
        self.mkdirs(parent)?;
        let dir = self.machine.open(self.root, parent, How::Directory)?;
        let result = self.machine.make_directory(dir, name, 0o777);
        self.machine.close(dir);
        result
    }

    pub(super) fn remove_dir_all(&mut self, name: &str) -> io::Result<()> {
        let path = self.at(name);
        let (parent, name) = parent_and_name(path.as_bytes());
        let dir = self.machine.open(self.root, parent, How::Directory).map_err(error)?;
        let result = remove_tree(&mut self.machine, dir, name);
        self.machine.close(dir);
        result.map_err(error)
    }

    pub fn entries(&mut self, name: &str) -> io::Result<Vec<Listed>> {
        let dir = self.machine.open(self.root, self.at(name).as_bytes(), How::Directory).map_err(error)?;
        let mut entries = Vec::new();
        let result = loop {
            match self.machine.list(dir, 32, 8192) {
                Ok(batch) if batch.is_empty() => break Ok(entries),
                Ok(batch) => entries.extend(batch),
                Err(refusal) => break Err(error(refusal)),
            }
        };
        self.machine.close(dir);
        result
    }
}

fn parent_and_name(path: &[u8]) -> (&[u8], &[u8]) {
    for (index, &byte) in path.iter().enumerate().rev() {
        if byte == b'/' {
            return (&path[..index], &path[index + 1..]);
        }
    }
    (b".", path)
}

fn remove_tree(machine: &mut Machine, parent: Opened, name: &[u8]) -> Result<(), Refusal> {
    let dir = match machine.open(parent, name, How::DirectoryNoFollow) {
        Ok(dir) => dir,
        Err(Refusal::Loop) => return machine.remove(parent, name, false),
        Err(refusal) => return Err(refusal),
    };
    let result = remove_entries(machine, dir);
    machine.close(dir);
    result?;
    machine.remove(parent, name, true)
}

fn remove_entries(machine: &mut Machine, dir: Opened) -> Result<(), Refusal> {
    loop {
        let entries = machine.list(dir, 32, 8192)?;
        if entries.is_empty() {
            return Ok(());
        }
        for (kind, name) in entries {
            if kind == Is::Directory {
                remove_tree(machine, dir, &name)?;
            } else {
                machine.remove(dir, &name, false)?;
            }
        }
    }
}
