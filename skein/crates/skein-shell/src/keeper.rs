//! What a test harness alone needs of the processes it keeps (runtime/readiness.md, 5).
//! A service has no use for this module: its calls are at the crate's root.

pub use crate::group::{child_id, group_has_other_members, group_live, process_live, signal_group};
pub use crate::process::{
    HostedPipes, abandon_binary, close_keeper_fd, hosted_pipes, open_child_pidfd, open_signal_pipe, poll_child,
    signal_current_thread, signal_kept_child, start_binary, write_service_signal,
};
pub use crate::system::wait_readable;

use skein_io::kernel::{Error, Fd};

/// The path an open descriptor names now, for a harness handing a held root to a path-based caller.
pub fn descriptor_path(fd: Fd) -> Result<Box<[u8]>, Error> {
    crate::system::descriptor_path(fd).map_err(crate::posix::process::map_error)
}

#[cfg(target_os = "linux")]
pub use crate::process::{
    make_subreaper, open_cgroup, open_pidfd, pidfd_exited, set_subreaper, start_binary_in_cgroup, subreaper,
    wait_cgroup_change,
};

#[cfg(test)]
mod tests {
    use super::{Error, close_keeper_fd, descriptor_path};
    use crate::{open_append, open_root};
    use skein_scratch::Scratch;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn a_directory_descriptor_names_its_canonical_path() {
        let scratch = Scratch::new("descriptor-directory");
        let directory = open_root(scratch.path()).expect("open scratch directory");
        let canonical = scratch.path().canonicalize().expect("canonical scratch directory");
        assert_eq!(
            descriptor_path(directory).expect("directory path").as_ref(),
            canonical.as_os_str().as_bytes(),
            "a held directory names its filesystem path"
        );
        close_keeper_fd(directory);
    }

    #[test]
    fn a_file_descriptor_names_its_path() {
        let scratch = Scratch::new("descriptor-file");
        let path = scratch.path().join("file");
        let root = open_root(scratch.path()).expect("open scratch directory");
        let file = open_append(root, b"file", 0o600).expect("open scratch file");
        let canonical = path.canonicalize().expect("canonical scratch file");
        assert_eq!(
            descriptor_path(file).expect("file path").as_ref(),
            canonical.as_os_str().as_bytes(),
            "a held file names its filesystem path"
        );
        close_keeper_fd(file);
        close_keeper_fd(root);
    }

    #[test]
    fn a_closed_descriptor_has_the_bad_descriptor_error() {
        let scratch = Scratch::new("descriptor-closed");
        let fd = open_root(scratch.path()).expect("open scratch directory");
        close_keeper_fd(fd);
        assert_eq!(descriptor_path(fd), Err(Error::Other(libc::EBADF)), "a closed descriptor has no path");
    }
}
