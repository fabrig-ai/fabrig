//! Startup paths beneath one fake root (skein/docs/design/testing.md, 4).

use crate::{How, Machine, Opened, kernel_error};
use skein_io::kernel::Error;
use skein_lib::Queue;
use skein_sim::{Answer, Call, Handle};

/// A filesystem and its base directory, serving a simulated world's startup and file calls.
#[derive(Debug)]
pub struct Rooted {
    pub files: Machine,
    pub root: Opened,
}

impl skein_sim_world::Machine for Rooted {
    fn open_root(&mut self, path: &[u8]) -> Result<Handle, Error> {
        self.files.open(self.root, path, How::Directory).map(|opened| Handle::new(opened.raw())).map_err(kernel_error)
    }

    fn close_root(&mut self, root: Handle) {
        self.files.close(Opened::new(root.raw()));
    }

    fn open_append(&mut self, root: Handle, path: &[u8], mode: u32) -> Result<Handle, Error> {
        self.files
            .open(Opened::new(root.raw()), path, How::Append { mode })
            .map(|opened| Handle::new(opened.raw()))
            .map_err(kernel_error)
    }

    fn step(&mut self, call: Call, answers: &mut Queue<Answer>) {
        crate::step(&mut self.files, call, answers);
    }
}

#[cfg(test)]
mod tests {
    use super::Rooted;
    use crate::{How, Item, Machine, Opened};
    use skein_io::kernel::Error;
    use skein_sim_world::Machine as WorldMachine;

    #[test]
    fn startup_handles_are_independent_appendable_and_rollback_releases_them() {
        let mut files = Machine::new();
        let root = files.lay(&[Item::directory(b"work"), Item::file(b"file", b"data")]);
        let mut rooted = Rooted { files, root };
        let first = rooted.open_root(b"work").expect("directory");
        let second = rooted.open_root(b"work").expect("fresh directory");
        assert_ne!(first, second);
        assert_eq!(rooted.open_root(b"missing"), Err(Error::NotFound));
        assert_eq!(rooted.open_root(b"file"), Err(Error::NotADirectory));
        assert_eq!(rooted.open_root(b"../escape"), Err(Error::Escape));
        let append = rooted.open_append(first, b"log", 0o600).expect("append file");
        rooted.files.append(Opened::new(append.raw()), b"record").expect("write");
        rooted.close_append(append);
        let read = rooted.files.open(Opened::new(second.raw()), b"log", How::Read).expect("read log");
        assert_eq!(rooted.files.read(read, 0, 100).expect("read"), b"record");
        rooted.files.close(read);
        rooted.close_root(first);
        rooted.close_root(second);
        assert_eq!(rooted.files.open_handles(), 1, "only the base remains");
    }
}
