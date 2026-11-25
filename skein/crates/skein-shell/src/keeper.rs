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

/// CPU consumed by the calling thread, or `None` when its clock is unavailable
/// (skein/docs/design/testing/service-tiers.md, 1).
#[must_use]
pub fn thread_cpu() -> Option<skein_lib::Duration> {
    crate::system::clock_nanos(libc::CLOCK_THREAD_CPUTIME_ID).map(skein_lib::Duration::from_nanos)
}
