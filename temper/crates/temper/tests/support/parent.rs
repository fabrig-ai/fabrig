//! Owned process cleanup and observations at the startup/process world's patience.
#[cfg(target_os = "linux")]
use skein_io::kernel::{Fd, Signal, Target};
use skein_world::{Deadline, Expired, Starved};
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
    fn wait(&self, owned: bool, what: &'static str) -> Option<Starved> {
        let mut deadline = Deadline::arm(what, super::WITHIN);
        deadline.watch_process(self.pid);
        let mut deferred = false;
        loop {
            if skein_shell::pidfd_exited(self.fd).expect("pidfd exit observation") {
                return None;
            }
            if !deferred && deadline.passed() {
                match deadline.expired() {
                    Expired::Starved(starved) if owned => return Some(starved),
                    Expired::Starved(_) => deferred = true,
                    Expired::Failed(figures) => {
                        panic!("process {} exited within the world's patience: {figures}", self.pid)
                    }
                }
            }
            // A reaped pidfd can report POLLHUP; only the identity-specific check proves exit.
            let _readiness =
                skein_shell::wait_readable(self.fd, if deferred { super::WITHIN } else { deadline.remaining() });
        }
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

    pub fn wait_for(&mut self, pid: u32) {
        let kept = self.kept.iter().find(|kept| kept.pid == pid).expect("recorded process");
        if let Some(starved) = kept.wait(self.kill_on_drop, "the retained child's exit") {
            self.end_owned();
            starved.pass();
        }
    }

    pub fn wait_all(&mut self) {
        for i in 0..self.kept.len() {
            self.wait_for(self.kept[i].pid);
        }
    }

    fn end_owned(&mut self) {
        assert!(self.kill_on_drop, "an observing guard leaves cleanup to its world");
        self.end_kept();
        if let Some(child) = &mut self.child {
            end_child(child);
        }
        self.child = None;
    }

    fn end_kept(&mut self) {
        for kept in &self.kept {
            let _sent = skein_shell::signal_kept_child(kept.fd, Signal::Kill, Target::Child);
        }
        for kept in &self.kept {
            let _starved = kept.wait(false, "the owned child's cleanup");
        }
        for kept in &self.kept {
            // A retained descendant may be reaped by its parent rather than by this test.
            let _reaped = skein_shell::poll_child(kept.fd, true);
        }
        self.kept.clear();
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
            self.end_owned();
        } else {
            self.kept.clear();
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

/// Wait with the explicit owner available for a starvation cleanup.
#[cfg(target_os = "linux")]
pub fn wait(guard: &mut Guard, mut ready: impl FnMut(&mut Guard) -> bool) {
    assert!(guard.kill_on_drop, "the condition wait has an owned process");
    let mut deadline = Deadline::arm("the owned parent's condition", super::WITHIN);
    if let Some(child) = &guard.child {
        deadline.watch_process(child.id());
    }
    loop {
        if ready(guard) {
            return;
        }
        if let Some(child) = &mut guard.child {
            assert!(
                child.try_wait().expect("condition's child status").is_none(),
                "owned child exited before the condition"
            );
        }
        if deadline.passed() {
            match deadline.expired() {
                Expired::Starved(starved) => {
                    guard.end_owned();
                    starved.pass();
                }
                Expired::Failed(figures) => panic!("condition settled within the world's patience: {figures}"),
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// End the passed child and every discovered descendant before releasing its identity.
pub fn end_child(child: &mut Child) {
    let running = child.try_wait().expect("owned child status before cleanup").is_none();
    #[cfg(target_os = "linux")]
    let mut tree = {
        let mut tree = Guard { child: None, kept: Vec::new(), kill_on_drop: false };
        if running {
            match skein_shell::open_child_pidfd(child.id(), std::process::id()) {
                Ok(fd) => {
                    tree.kept.push(Kept { pid: child.id(), parent: std::process::id(), fd });
                    tree.descendants(child.id());
                }
                Err(skein_io::kernel::Error::NotFound | skein_io::kernel::Error::Other(3)) => {}
                Err(error) => panic!("owned child's identity for cleanup: {error:?}"),
            }
        }
        for kept in &tree.kept {
            let _sent = skein_shell::signal_kept_child(kept.fd, Signal::Kill, Target::Child);
        }
        for kept in &tree.kept {
            let _starved = kept.wait(false, "the owned child's cleanup");
        }
        tree
    };
    if running {
        let _killed = child.kill();
    }
    child.wait().expect("owned child reaped");
    drop(child.stdin.take());
    drop(child.stdout.take());
    drop(child.stderr.take());
    #[cfg(target_os = "linux")]
    tree.end_kept();
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
