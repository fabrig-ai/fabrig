//! No-follow stat and mkdir at the names in a scripted disk.
use skein_fake_machine::{How, Is, Item, Machine, Opened, Refusal};
use skein_io::kernel::{Kind, Stat};
use std::{cell::RefCell, io, os::unix::ffi::OsStrExt, path::Path};

pub(crate) const USER: u32 = 1000;

pub(crate) struct Disk {
    machine: RefCell<Machine>,
    root: Opened,
}

impl Disk {
    pub(crate) fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(USER);
        let root = machine.lay(items);
        Self { machine: RefCell::new(machine), root }
    }

    pub(crate) fn stat(&self, path: &Path) -> io::Result<Stat> {
        let mut machine = self.machine.borrow_mut();
        let handle = machine.open(self.root, relative(path), How::PathNoFollow).map_err(io_error)?;
        let facts = machine.stat(handle);
        machine.close(handle);
        let kind = match facts.is {
            Is::File => Kind::File,
            Is::Directory => Kind::Directory,
            Is::Link => Kind::Symlink,
            Is::Device | Is::Fifo => Kind::Other,
        };
        Ok(Stat { kind, size: facts.size, mode: facts.mode, owner: facts.owner, links: facts.links })
    }

    pub(crate) fn make(&self, path: &Path, parents: bool, mode: u32) -> io::Result<()> {
        let mut machine = self.machine.borrow_mut();
        let parent = relative(path.parent().expect("fixture directory parent"));
        let directory = if parents {
            assert!(parent.is_empty(), "only the programs name uses recursive creation");
            self.root
        } else {
            machine.open(self.root, parent, How::Directory).map_err(io_error)?
        };
        let result = machine.make_directory(directory, path.file_name().expect("fixture leaf").as_bytes(), mode);
        if !parents {
            machine.close(directory);
        }
        result.map_err(io_error)
    }

    pub(crate) fn assert_absent(&self, path: &Path) {
        assert_eq!(self.stat(path).unwrap_err().kind(), io::ErrorKind::NotFound, "{} stays absent", path.display());
    }
}

fn relative(path: &Path) -> &[u8] {
    path.strip_prefix("/fixture").expect("fixture path").as_os_str().as_bytes()
}

fn io_error(why: Refusal) -> io::Error {
    io::Error::from_raw_os_error(temper_shell::directories::errno(skein_fake_machine::kernel_error(why)))
}
