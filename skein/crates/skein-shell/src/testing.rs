//! Access to the shell's native adapters for its real-loop tests (runtime/shell.md, 8).

use std::ptr;

use skein_io::kernel::{Error, Fd, Signal};

use crate::{posix, system as native};

pub use crate::readiness::testing as readiness;
#[cfg(target_os = "macos")]
pub use native::files::testing::path_name;
pub use native::listener_calls::testing::{NO_ACCEPTCONN, option};

pub mod system {
    use std::net::SocketAddr;

    use skein_io::kernel::{Entry, Family, Fd};

    use crate::system as native;

    pub mod process {
        pub use crate::open_signals;
    }

    pub fn close(fd: Fd) -> Result<(), i32> {
        native::close(fd)
    }

    #[must_use]
    pub fn last_errno() -> i32 {
        native::last_errno()
    }

    #[must_use]
    pub fn clock_nanos(id: libc::clockid_t) -> Option<u64> {
        native::clock_nanos(id)
    }

    pub fn status_flags(fd: i32) -> Result<i32, i32> {
        native::status_flags(fd)
    }

    pub fn socket(family: Family) -> Result<Fd, i32> {
        native::socket(family)
    }

    pub fn bind(fd: Fd, addr: &SocketAddr) -> Result<(), i32> {
        native::bind(fd, addr)
    }

    pub fn bound_addr(fd: Fd) -> Result<SocketAddr, i32> {
        native::bound_addr(fd)
    }

    pub fn listen(fd: Fd, backlog: u32) -> Result<(), i32> {
        native::listen(fd, backlog)
    }

    #[cfg(target_os = "macos")]
    pub fn retry<T>(call: impl FnMut() -> Result<T, i32>) -> Result<T, i32> {
        native::retry(call)
    }

    #[cfg(target_os = "macos")]
    pub fn status(result: i32) -> Result<(), i32> {
        native::status(result)
    }

    pub struct Listings(native::Listings);

    impl Listings {
        pub fn list(&mut self, fd: Fd, entries: &mut [Entry], names: &mut [u8]) -> Result<u32, i32> {
            self.0.list(fd, entries, names)
        }

        pub fn forget(&mut self, fd: Fd) {
            self.0.forget(fd);
        }
    }

    impl Default for Listings {
        fn default() -> Self {
            Self(native::Listings::new())
        }
    }

    impl Listings {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }
    }
}

#[cfg(target_os = "linux")]
pub struct Ring(crate::linux::ring::Ring);

#[cfg(target_os = "linux")]
impl Ring {
    pub fn open(config: crate::Config) -> Result<Self, crate::OpenError> {
        crate::linux::ring::Ring::open(config).map(Self)
    }

    pub fn submit(&mut self, records: &mut skein_lib::Queue<skein_io::kernel::Submit>, wait: crate::Wait) {
        self.0.submit(records, wait);
    }

    pub fn reap(&mut self, out: &mut skein_lib::Queue<skein_io::kernel::Complete>) {
        self.0.reap(out);
    }
}

pub struct Kept {
    pub fd: Fd,
    pub modes: libc::termios,
    pub keys: crate::Keys,
}

impl Kept {
    fn kept(&self) -> crate::Kept {
        crate::Kept { fd: self.fd, modes: self.modes, keys: self.keys }
    }

    pub fn set_kept(&self) -> Result<(), Error> {
        self.kept().set_kept()
    }

    pub fn set_raw(&self) -> Result<(), Error> {
        self.kept().set_raw()
    }
}

#[cfg(target_os = "linux")]
pub fn close_bound() -> Result<i32, i32> {
    posix::parent_death::close_bound()
}

#[cfg(target_os = "linux")]
pub fn watch_parent(parent: libc::pid_t, signal: i32, until: i32) -> Result<(), i32> {
    posix::parent_death::watch_parent(parent, signal, until)
}

#[cfg(target_os = "linux")]
#[must_use]
pub fn parent_error(errno: i32) -> std::io::Error {
    posix::parent_death::parent_error(errno)
}

#[cfg(target_os = "linux")]
#[must_use]
pub fn signal_number(signal: Signal) -> i32 {
    posix::process::signal_number(signal)
}

#[cfg(target_os = "linux")]
pub fn nonblocking(fd: Fd) -> Result<(), i32> {
    posix::nonblocking(fd)
}

#[cfg(target_os = "macos")]
pub fn remove(dir: Fd, name: &[u8], directory: bool) -> Result<(), i32> {
    native::files::remove(dir, name, directory)
}

pub mod links {
    use std::fs;
    use std::os::unix::{
        ffi::OsStrExt,
        fs::{PermissionsExt, symlink},
    };
    use std::path::Path;

    #[must_use]
    pub fn pass_links(dir: i32, path: &[u8], owner: u32) -> Option<Box<[u8]>> {
        crate::posix::links::pass_links(dir, path, owner)
    }

    #[must_use]
    pub fn walk(path: &Path, owner: u32) -> Option<Box<[u8]>> {
        pass_links(libc::AT_FDCWD, path.as_os_str().as_bytes(), owner)
    }

    pub fn configure(path: &Path, mode: u32, parent: bool) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("explicit holding-directory mode");
        fs::create_dir(path.join("real")).expect("real directory");
        fs::set_permissions(path.join("real"), fs::Permissions::from_mode(0o755))
            .expect("explicit leaf-link directory mode");
        fs::write(path.join("real/f"), b"file").expect("real file");
        if parent {
            symlink("real", path.join("sys")).expect("parent link");
        }
    }
}

#[must_use]
#[expect(unsafe_code, reason = "the fixture reads the calling thread signal mask (runtime/shell.md, 6.3)")]
pub fn signal_mask() -> Vec<i32> {
    // SAFETY: sigset_t is C storage; zeroing also initializes any unused storage.
    let mut mask: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: a null input only reads this thread's mask into live storage.
    let read = unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, ptr::null(), ptr::from_mut(&mut mask)) };
    assert_eq!(read, 0, "read the calling thread's signal mask");
    let bits = size_of_val(&mask).checked_mul(8).expect("signal set bits fit");
    let mut members = Vec::new();
    // The set's storage bounds all signal numbers; unsupported numbers return -1.
    for signal in 1..=i32::try_from(bits).expect("signal numbers fit") {
        // SAFETY: mask is initialized; sigismember only reads it during this call.
        if unsafe { libc::sigismember(ptr::from_ref(&mask), signal) } == 1 {
            members.push(signal);
        }
    }
    members
}
