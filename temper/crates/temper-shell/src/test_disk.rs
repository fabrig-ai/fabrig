//! Startup file facts from skein's fake machine (temper/docs/design/testing/shells.md, 3.11–3.12).
use crate::directories::{PrivateDirectories, errno};
use skein_fake_machine::{How, Is, Item, Machine, Opened, Refusal};
use skein_io::kernel::{Kind, Stat};
use std::{io, os::unix::ffi::OsStrExt, path::Path};

pub const USER: u32 = 1000;

pub struct Disk {
    pub machine: Machine,
    root: Opened,
}

impl Disk {
    pub fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(USER);
        let root = machine.lay(items);
        Self { machine, root }
    }

    pub fn open(&mut self, path: &Path, how: How) -> Result<Opened, Refusal> {
        let name = path.strip_prefix("/").expect("absolute fixture path").as_os_str().as_bytes();
        self.machine.open(self.root, if name.is_empty() { b"." } else { name }, how)
    }

    pub fn at_parent<T>(
        &mut self,
        path: &Path,
        effect: impl FnOnce(&mut Machine, Opened, &[u8]) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let parent = self.open(path.parent().expect("fixture parent"), How::Directory)?;
        let result = effect(&mut self.machine, parent, name(path));
        self.machine.close(parent);
        result
    }
}

impl PrivateDirectories for Disk {
    fn symlink_stat(&mut self, path: &Path) -> io::Result<Stat> {
        // The native stat follows links in ancestors, but not at the final name.
        self.at_parent(path, |machine, parent, name| {
            let entry = machine.open(parent, name, How::PathNoFollow)?;
            let facts = machine.stat(entry);
            machine.close(entry);
            Ok(Stat {
                kind: match facts.is {
                    Is::File => Kind::File,
                    Is::Directory => Kind::Directory,
                    Is::Link => Kind::Symlink,
                    Is::Fifo | Is::Device => Kind::Other,
                },
                size: facts.size,
                mode: facts.mode,
                owner: facts.owner,
                links: facts.links,
            })
        })
        .map_err(io_error)
    }

    fn create_directories(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        let parent = path.parent().expect("fixture parent");
        match self.open(parent, How::Directory) {
            Ok(opened) => self.machine.close(opened),
            Err(Refusal::NotFound) => self.create_directories(parent, mode)?,
            Err(why) => return Err(io_error(why)),
        }
        self.at_parent(path, |machine, parent, name| machine.make_directory(parent, name, mode)).map_err(io_error)
    }
}

pub fn name(path: &Path) -> &[u8] {
    path.file_name().expect("fixture entry name").as_bytes()
}

pub fn io_error(why: Refusal) -> io::Error {
    io::Error::from_raw_os_error(errno(skein_fake_machine::kernel_error(why)))
}
