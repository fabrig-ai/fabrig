//! Owned process cleanup and observations at the startup/process world's patience.
#[cfg(target_os = "linux")]
use skein_io::kernel::{Fd, Signal, Target};
use std::{
    process::{Child, Command},
    time::Duration,
};

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct Kept {
    pid: u32,
    parent: u32,
    fd: Fd,
}
#[cfg(target_os = "linux")]
impl Kept {
    fn wait(&self) {
        // A reaped pidfd reports POLLHUP; stream readiness treats that as an
        // error. In either case the pidfd-specific observation must prove exit.
        let _readiness = skein_shell::wait_readable(self.fd, super::WITHIN);
        assert!(
            skein_shell::pidfd_exited(self.fd).expect("pidfd exit observation"),
            "process {} exited within the world's patience",
            self.pid
        );
    }
}
#[cfg(target_os = "linux")]
impl Drop for Kept {
    fn drop(&mut self) {
        skein_shell::close_keeper_fd(self.fd);
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct Guard {
    pub child: Option<Child>,
    kept: Vec<Kept>,
    kill_on_drop: bool,
}
#[cfg(target_os = "linux")]
impl Guard {
    #[must_use]
    pub fn new(child: Child) -> Self {
        assert!(child.id() > 1, "owned child PID");
        Self { child: Some(child), kept: Vec::new(), kill_on_drop: true }
    }

    /// Retain identities in the current test's tree; its world owns cleanup.
    #[must_use]
    pub fn observing_test() -> Self {
        let mut guard = Self { child: None, kept: Vec::new(), kill_on_drop: false };
        let children = std::fs::read_to_string("/proc/thread-self/children").expect("test thread's children");
        for pid in children.split_whitespace().map(|pid| pid.parse().expect("child PID")) {
            let fd = match skein_shell::open_child_pidfd(pid, std::process::id()) {
                Ok(fd) => fd,
                Err(skein_io::kernel::Error::NotFound | skein_io::kernel::Error::Other(3)) => continue,
                Err(error) => panic!("test child identity: {error:?}"),
            };
            guard.kept.push(Kept { pid, parent: std::process::id(), fd });
            guard.descendants(pid);
        }
        guard
    }

    #[must_use]
    pub fn pids(&self) -> Vec<u32> {
        self.kept.iter().map(|kept| kept.pid).collect()
    }

    #[must_use]
    pub fn children(&self, parent: u32) -> Vec<u32> {
        self.kept.iter().filter(|kept| kept.parent == parent).map(|kept| kept.pid).collect()
    }

    /// The caller keeps this parent alive or unreaped throughout discovery.
    pub fn record(&mut self, pid: u32, parent: u32) {
        let fd = skein_shell::open_child_pidfd(pid, parent).expect("owned descendant identity");
        self.kept.push(Kept { pid, parent, fd });
    }

    pub fn signal(&self, pid: u32, signal: Signal) {
        let fd = self.kept.iter().find(|kept| kept.pid == pid).expect("recorded process").fd;
        skein_shell::signal_kept_child(fd, signal, Target::Child).expect("signal retained process");
    }

    pub fn wait_for(&self, pid: u32) {
        self.kept.iter().find(|kept| kept.pid == pid).expect("recorded process").wait();
    }

    pub fn wait_all(&self) {
        for kept in &self.kept {
            kept.wait();
        }
    }

    pub fn descendants(&mut self, pid: u32) {
        let fd = self.kept.iter().find(|kept| kept.pid == pid).expect("recorded parent").fd;
        self.descendants_of(pid, fd);
    }

    fn descendants_of(&mut self, pid: u32, parent: Fd) {
        if skein_shell::pidfd_exited(parent).expect("parent identity") {
            return;
        }
        if let Ok(children) = std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
            for child in children.split_whitespace().map(|child| child.parse().expect("child pid")) {
                let fd = match skein_shell::open_child_pidfd(child, pid) {
                    Ok(fd) => fd,
                    // A short-lived command (such as the PID file's mv) can
                    // exit between the children snapshot and pidfd_open.
                    Err(skein_io::kernel::Error::NotFound | skein_io::kernel::Error::Other(3)) => continue,
                    Err(error) => panic!("descendant identity: {error:?}"),
                };
                // Bind the parent observation to its retained identity too:
                // an exited parent's numeric PID might have been reused.
                if skein_shell::pidfd_exited(parent).expect("parent identity after discovery") {
                    skein_shell::close_keeper_fd(fd);
                    return;
                }
                self.kept.push(Kept { pid: child, parent: pid, fd });
                self.descendants_of(child, fd);
            }
        }
    }

    #[must_use]
    pub fn has_descendants(&self) -> bool {
        self.kept.len() > 1
    }
}
#[cfg(target_os = "linux")]
impl Drop for Guard {
    fn drop(&mut self) {
        // End every possible sender before releasing the parent's numeric PID.
        // An observing guard only closes identities; its world owns the children.
        if self.kill_on_drop {
            for kept in &self.kept {
                let _sent = skein_shell::signal_kept_child(kept.fd, Signal::Kill, Target::Child);
            }
            for kept in &self.kept {
                let _exited = skein_shell::wait_readable(kept.fd, super::WITHIN);
            }
        }
        self.kept.clear();
        if let Some(child) = &mut self.child {
            let _killed = child.kill();
            let _reaped = child.wait();
        }
    }
}

/// A Child that has not been reaped retains its PID until this signal is sent.
pub fn signal(child: &mut Child, signal: &str) {
    assert!(child.id() > 1, "owned child PID");
    if child.try_wait().expect("child status before signal").is_none() {
        let sent = Command::new("/usr/bin/kill")
            .args([&format!("-{signal}"), &child.id().to_string()])
            .status()
            .expect("signal owned child");
        assert!(sent.success(), "owned child accepted {signal}");
    }
}

/// The caller retains this child unreaped while observing its signal queues.
#[cfg(target_os = "linux")]
#[must_use]
pub fn term_consumed(child: &Child) -> bool {
    assert!(child.id() > 1, "owned child PID");
    let status = std::fs::read_to_string(format!("/proc/{}/status", child.id())).expect("owned child signal status");
    let mut queues = 0;
    for line in status.lines() {
        if let Some(mask) = line.strip_prefix("SigPnd:").or_else(|| line.strip_prefix("ShdPnd:")) {
            queues += 1;
            // Linux SIGTERM is signal 15; both thread and process queues matter.
            if u64::from_str_radix(mask.trim(), 16).expect("pending signal mask") & (1 << 14) != 0 {
                return false;
            }
        }
    }
    assert_eq!(queues, 2, "both pending signal queues observed");
    true
}

pub fn wait(mut ready: impl FnMut() -> bool) {
    let clock = skein_shell::Clock::new();
    let end = clock.now().now.saturating_add(super::WITHIN);
    loop {
        if ready() {
            return;
        }
        assert!(clock.now().now < end, "condition settled within the world's patience");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Numeric snapshots are observations only, never a capability to signal.
pub fn descendants(pid: u32, out: &mut Vec<u32>) {
    if let Ok(children) = std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
        for child in children.split_whitespace().map(|child| child.parse().expect("child PID")) {
            out.push(child);
            descendants(child, out);
        }
    }
}
