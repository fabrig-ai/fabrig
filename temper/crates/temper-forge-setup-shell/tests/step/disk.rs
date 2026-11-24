//! The file and lock faces over skein's fake machine (temper/docs/design/testing/shells.md, 3.1 and 3.2).
use skein_fake_machine::{Facts, How, Is, Item, Machine, Opened, Refusal};
use std::{
    cell::RefCell,
    fs::TryLockError,
    io::{self, ErrorKind},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    rc::Rc,
};
use temper_forge_setup_shell::{files, process};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    CreateNew(u32),
    Open,
    Write,
    Sync,
    Rename,
    NoFollow,
    RemoveFile,
    RemoveTree,
    CreateDirectory,
    SetMode(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Call {
    pub action: Action,
    pub path: PathBuf,
    pub to: Option<PathBuf>,
    pub result: Result<(), ErrorKind>,
}

pub struct Disk {
    root: Handle,
    pub calls: Vec<Call>,
    pub fail: Option<(Action, PathBuf)>,
}

#[derive(Debug)]
pub struct Handle {
    machine: Rc<RefCell<Machine>>,
    file: Opened,
    path: PathBuf,
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.machine.borrow_mut().close(self.file);
    }
}

impl Handle {
    pub fn bytes(&self, bound: u32) -> Vec<u8> {
        self.machine.borrow().read(self.file, 0, bound).expect("fixture file is readable")
    }

    pub fn stat(&self) -> Facts {
        self.machine.borrow().stat(self.file)
    }

    pub fn lock(&self) -> Result<(), TryLockError> {
        self.machine.borrow_mut().lock(self.file).map_err(|why| {
            if why == Refusal::Held { TryLockError::WouldBlock } else { TryLockError::Error(error(why)) }
        })
    }
}

impl Disk {
    pub fn new(items: &[Item]) -> Self {
        let mut machine = Machine::new();
        let file = machine.lay(items);
        let root = Handle { machine: Rc::new(RefCell::new(machine)), file, path: "/".into() };
        Self { root, calls: Vec::new(), fail: None }
    }

    pub fn open_at(&self, path: &Path, how: How) -> io::Result<Handle> {
        let relative = path.strip_prefix("/").expect("absolute fixture path");
        let name = if relative.as_os_str().is_empty() { b"." } else { relative.as_os_str().as_bytes() };
        let file = self.root.machine.borrow_mut().open(self.root.file, name, how).map_err(error)?;
        Ok(Handle { machine: self.root.machine.clone(), file, path: path.into() })
    }

    pub fn read(&self, path: &Path) -> Vec<u8> {
        self.open_at(path, How::Read).expect("fixture file").bytes(1 << 20)
    }

    pub fn entries(&self, path: &Path) -> usize {
        let directory = self.open_at(path, How::Directory).expect("fixture directory");
        self.root.machine.borrow_mut().list(directory.file, 100, 4096).expect("fixture entries").len()
    }

    pub fn open_lock(&self, path: &Path, mode: u32) -> io::Result<Handle> {
        match self.open_at(path, How::Write) {
            Err(why) if why.kind() == ErrorKind::NotFound => self.open_at(path, How::Create { mode }),
            result => result,
        }
    }

    pub fn lock(&self, path: &Path) -> Result<Handle, process::Refused> {
        process::lock_with(path, |path, mode| self.open_lock(path, mode), Handle::lock)
    }

    fn effect<T>(
        &mut self,
        action: Action,
        path: &Path,
        to: Option<&Path>,
        perform: impl FnOnce(&Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let result = if self.fail.as_ref().is_some_and(|(at, name)| *at == action && name == path) {
            self.fail = None;
            Err(ErrorKind::PermissionDenied.into())
        } else {
            perform(self)
        };
        self.calls.push(Call {
            action,
            path: path.into(),
            to: to.map(Path::to_path_buf),
            result: result.as_ref().map(|_| ()).map_err(io::Error::kind),
        });
        result
    }

    fn parent<'a>(&self, path: &'a Path) -> io::Result<(Handle, &'a [u8])> {
        let parent = self.open_at(path.parent().expect("fixture has a parent"), How::Directory)?;
        Ok((parent, path.file_name().expect("fixture entry has a name").as_bytes()))
    }

    fn remove(&self, path: &Path, tree: bool) -> io::Result<()> {
        let (parent, name) = self.parent(path)?;
        remove(&mut self.root.machine.borrow_mut(), parent.file, name, tree).map_err(error)
    }
}

impl files::Disk for Disk {
    type File = Handle;

    fn create_new(&mut self, path: &Path, mode: u32) -> io::Result<Handle> {
        self.effect(Action::CreateNew(mode), path, None, |disk| disk.open_at(path, How::Create { mode }))
    }
    fn open(&mut self, path: &Path) -> io::Result<Handle> {
        self.effect(Action::Open, path, None, |disk| disk.open_at(path, How::Read))
    }
    fn write_all(&mut self, file: &mut Handle, bytes: &[u8]) -> io::Result<()> {
        self.effect(Action::Write, &file.path, None, |_| {
            file.machine.borrow_mut().write(file.file, 0, bytes).map_err(error)
        })
    }
    fn sync(&mut self, file: &Handle) -> io::Result<()> {
        self.effect(Action::Sync, &file.path, None, |_| {
            file.machine.borrow_mut().sync(file.file);
            Ok(())
        })
    }
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        self.effect(Action::Rename, from, Some(to), |disk| {
            let (src, from) = disk.parent(from)?;
            let (dst, to) = disk.parent(to)?;
            let result = disk.root.machine.borrow_mut().rename(src.file, from, dst.file, to, false);
            result.map_err(error)
        })
    }
    fn is_directory(&mut self, path: &Path) -> io::Result<bool> {
        self.effect(Action::NoFollow, path, None, |disk| {
            Ok(disk.open_at(path, How::PathNoFollow)?.stat().is == Is::Directory)
        })
    }
    fn remove_file(&mut self, path: &Path) -> io::Result<()> {
        self.effect(Action::RemoveFile, path, None, |disk| disk.remove(path, false))
    }
    fn remove_dir_all(&mut self, path: &Path) -> io::Result<()> {
        self.effect(Action::RemoveTree, path, None, |disk| disk.remove(path, true))
    }
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        self.effect(Action::CreateDirectory, path, None, |disk| {
            // The forge lays out directories parent first.
            let (parent, name) = disk.parent(path)?;
            let made = disk.root.machine.borrow_mut().make_directory(parent.file, name, 0o777);
            match made {
                Err(Refusal::Exists) => disk.open_at(path, How::Directory).map(drop),
                result => result.map_err(error),
            }
        })
    }
    fn set_mode(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        // The fake machine has no chmod; the check observes the requested mode.
        self.effect(Action::SetMode(mode), path, None, |disk| disk.open_at(path, How::Read).map(drop))
    }
}

fn remove(machine: &mut Machine, parent: Opened, name: &[u8], tree: bool) -> Result<(), Refusal> {
    if tree {
        let directory = machine.open(parent, name, How::DirectoryNoFollow)?;
        let result = (|| {
            loop {
                let entries = machine.list(directory, 32, 4096)?;
                if entries.is_empty() {
                    return Ok(());
                }
                for (kind, name) in entries {
                    remove(machine, directory, &name, kind == Is::Directory)?;
                }
            }
        })();
        machine.close(directory);
        result?;
    }
    machine.remove(parent, name, tree)
}

fn error(why: Refusal) -> io::Error {
    match why {
        Refusal::NotFound => ErrorKind::NotFound,
        Refusal::Exists => ErrorKind::AlreadyExists,
        Refusal::NotADirectory => ErrorKind::NotADirectory,
        Refusal::IsADirectory => ErrorKind::IsADirectory,
        Refusal::NotEmpty => ErrorKind::DirectoryNotEmpty,
        Refusal::Permission => ErrorKind::PermissionDenied,
        Refusal::NoSpace => ErrorKind::StorageFull,
        Refusal::Held
        | Refusal::Loop
        | Refusal::NameTooLong
        | Refusal::Escape
        | Refusal::NotAFile
        | Refusal::Beneath => {
            panic!("unexpected fixture file refusal: {why:?}")
        }
    }
    .into()
}
