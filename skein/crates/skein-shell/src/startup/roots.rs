//! Durable startup directories (runtime/shell-startup.md, section 1).

use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use skein_io::kernel::{Fd, PERMISSIONS};

trait DirectoryCalls {
    fn open_root(&mut self, path: &Path) -> Result<Fd, i32>;
    fn make_child(&mut self, parent: Fd, name: &CStr, mode: u32) -> Result<(), i32>;
    fn sync(&mut self, directory: Fd) -> Result<(), i32>;
    fn open_child(&mut self, parent: Fd, name: &CStr) -> Result<Fd, i32>;
    fn close(&mut self, directory: Fd);
}

struct Native;

impl DirectoryCalls for Native {
    fn open_root(&mut self, path: &Path) -> Result<Fd, i32> {
        crate::system::open_root(path)
    }

    fn make_child(&mut self, parent: Fd, name: &CStr, mode: u32) -> Result<(), i32> {
        crate::posix::files::make_child(parent, name, mode)
    }

    fn sync(&mut self, directory: Fd) -> Result<(), i32> {
        crate::system::files::sync(directory)
    }

    fn open_child(&mut self, parent: Fd, name: &CStr) -> Result<Fd, i32> {
        crate::posix::files::open_child(parent, name)
    }

    fn close(&mut self, directory: Fd) {
        let _closed = crate::posix::close(directory);
    }
}

/// Makes absent directories along `path` durable, then opens it as a root
/// (runtime/shell-startup.md, 1). Existing directories keep their modes; errors are errno.
pub fn make_root(path: &Path, mode: u32) -> Result<Fd, i32> {
    walk(path, mode, &mut Native)
}

fn walk(path: &Path, mode: u32, calls: &mut impl DirectoryCalls) -> Result<Fd, i32> {
    if mode & !PERMISSIONS != 0 {
        return Err(libc::EINVAL);
    }
    if path.as_os_str().is_empty() {
        return Err(libc::ENOENT);
    }
    let mut absent = Vec::new();
    let mut parent = None;
    for ancestor in path.ancestors() {
        let ancestor = if ancestor.as_os_str().is_empty() { Path::new(".") } else { ancestor };
        match calls.open_root(ancestor) {
            Ok(fd) => {
                parent = Some(fd);
                break;
            }
            Err(libc::ENOENT) => absent.push(ancestor),
            Err(error) => return Err(error),
        }
    }
    let mut parent = parent.ok_or(libc::ENOENT)?;
    for directory in absent.into_iter().rev() {
        let child = prepare_child(calls, parent, directory, mode);
        calls.close(parent);
        parent = child?;
    }
    Ok(parent)
}

fn prepare_child(calls: &mut impl DirectoryCalls, parent: Fd, path: &Path, mode: u32) -> Result<Fd, i32> {
    let name = path.components().next_back().ok_or(libc::EINVAL)?;
    let name = CString::new(name.as_os_str().as_bytes()).map_err(|_| libc::EINVAL)?;
    match calls.make_child(parent, &name, mode) {
        Ok(()) => calls.sync(parent)?,
        Err(libc::EEXIST) => {}
        Err(error) => return Err(error),
    }
    calls.open_child(parent, &name)
}

#[cfg(test)]
mod tests;
