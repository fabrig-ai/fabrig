use skein_shell::keeper::thread_cpu;
pub(crate) use skein_shell::keeper::{open_signal_pipe, signal_current_thread};

mod tests {
    use skein_io::kernel::Error;
    use skein_scratch::Scratch;
    use skein_shell::keeper::{close_keeper_fd, descriptor_path};
    use skein_shell::{open_append, open_root};
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn the_thread_clock_grows_with_the_calling_threads_work() {
        let before = super::thread_cpu().expect("the calling thread has a CPU clock");
        let mut value = 1_u64;
        for _ in 0..100_000 {
            value = std::hint::black_box(value.wrapping_mul(3).wrapping_add(1));
        }
        std::hint::black_box(value);
        let after = super::thread_cpu().expect("the calling thread still has a CPU clock");
        assert!(after > before, "busy work consumes this thread's CPU");
        assert_eq!(crate::system::clock_nanos(libc::clockid_t::MAX), None, "an unavailable clock is not an assertion");
    }

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
