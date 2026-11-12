//! No fixture outlives its test (skein/docs/design/testing/README.md, 5).
#![cfg(target_os = "linux")]

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::os::fd::AsRawFd;
use std::os::unix::{fs::OpenOptionsExt, process::CommandExt};
use std::process::{Child, Command, Stdio};

use skein_io::kernel::{Fd, Signal, Target};
use skein_lib::Duration;
use skein_shell::keeper;
use skein_world::Expired;

use super::given_listener::deadlines::Patience;

const PATIENCE: Duration = Duration::from_secs(5);
const POLL: std::time::Duration = std::time::Duration::from_millis(1);

fn until(deadline: &Patience, mut ready: impl FnMut() -> bool) -> Result<(), Expired> {
    while !ready() {
        if deadline.passed() {
            return Err(deadline.expired());
        }
        std::thread::sleep(POLL);
    }
    Ok(())
}

fn expired(verdict: Expired, message: &str) -> ! {
    match verdict {
        Expired::Starved(starved) => starved.pass(),
        Expired::Failed(figures) => panic!("{message}: {figures}"),
    }
}

struct Process {
    name: String,
    pid: u32,
    fd: Fd,
    must_end: bool,
}

struct Tree {
    owner: Child,
    reaped: bool,
    stderr: Option<File>,
    pending: String,
    processes: Vec<Process>,
    subreaper: bool,
    ended: bool,
}

impl Tree {
    fn start(mode: &str) -> Self {
        let subreaper = keeper::subreaper().expect("read adoption setting");
        keeper::make_subreaper().expect("adopt the fixture's orphans for cleanup");
        let owner = Command::new("/bin/sh")
            .args(["-c", "\"$0\" \"$1\" & printf 'leader:%s\\n' \"$!\" >&2; wait"])
            .arg(skein_scratch::paths::binary("process_fixture"))
            .arg(mode)
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start an owner standing in for a test");
        let mut tree = Self {
            owner,
            reaped: false,
            stderr: None,
            pending: String::new(),
            processes: Vec::new(),
            subreaper,
            ended: false,
        };
        let stderr = tree.owner.stderr.take().expect("owner stderr");
        // Open a separate nonblocking description so startup has a patience,
        // without another thread or changing the writers' pipe flags.
        tree.stderr = Some(
            OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(format!("/proc/self/fd/{}", stderr.as_raw_fd()))
                .expect("nonblocking fixture reports"),
        );
        drop(stderr);
        let mut deadline = Patience::new("the owned fixture reporting readiness", PATIENCE);
        deadline.watch_process(tree.owner.id());
        let result = until(&deadline, || {
            tree.read_reports(mode);
            let reported = match mode {
                "never" => 1,
                "supervise-grandchild" => 3,
                _ => 2,
            };
            if tree.processes.len() != reported {
                return false;
            }
            // never has no readiness line: observe it parked in the fixture,
            // rather than killing its owner before it has even entered main.
            mode != "never"
                || tree.processes.iter().any(|process| {
                    fs::read_link(format!("/proc/{}/exe", process.pid)).ok()
                        == fs::canonicalize(skein_scratch::paths::binary("process_fixture")).ok()
                        && fs::read_to_string(format!("/proc/{}/stat", process.pid)).ok().is_some_and(|stat| {
                            stat.rsplit_once(") ").is_some_and(|(_, fields)| fields.starts_with("S "))
                        })
                })
        });
        if let Err(verdict) = result {
            tree.end();
            expired(verdict, "fixture did not report readiness within 5 s");
        }
        tree
    }

    fn read_reports(&mut self, mode: &str) {
        let mut bytes = [0_u8; 1024];
        match self.stderr.as_mut().expect("report pipe").read(&mut bytes) {
            Ok(0) => panic!("fixture report pipe closed during startup: {}", self.pending),
            Ok(n) => self.pending.push_str(std::str::from_utf8(&bytes[..n]).expect("ASCII fixture reports")),
            Err(error) if error.kind() == ErrorKind::WouldBlock || error.kind() == ErrorKind::Interrupted => {}
            Err(error) => panic!("read fixture reports: {error}"),
        }
        while let Some((line, rest)) = self.pending.split_once('\n') {
            let (name, pid) = line.split_once(':').expect("named fixture PID");
            assert!(["leader", "descendant", "grandchild"].contains(&name), "unexpected report: {line}");
            assert!(!self.processes.iter().any(|process| process.name == name), "duplicate report: {line}");
            let pid = pid.parse().expect("numeric fixture PID");
            self.processes.push(Process {
                name: name.to_owned(),
                pid,
                fd: keeper::open_pidfd(pid).expect("retain reported process identity"),
                must_end: name == "leader" || !["fork-live", "supervise-ignore"].contains(&mode),
            });
            self.pending = rest.to_owned();
        }
    }

    fn kill_owner(&mut self) {
        // Kill only the owner's PID, never its private group: this is what a
        // runner does when the test cannot run cleanup.
        self.owner.kill().expect("SIGKILL owner alone");
        self.owner.wait().expect("reap killed owner");
        self.reaped = true;
    }

    fn survivors(&self) -> Vec<String> {
        self.processes
            .iter()
            .filter(|process| process.must_end && !keeper::pidfd_exited(process.fd).expect("poll fixture identity"))
            .map(|process| format!("{}:{}", process.name, process.pid))
            .collect()
    }

    fn end(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;
        if !self.reaped {
            let _killed = self.owner.kill();
            self.owner.wait().expect("reap owner during cleanup");
            self.reaped = true;
        }
        // End every retained survivor through its pidfd before reporting a
        // failure. Bounded sleeps are owned for cleanup but not exit assertions.
        for process in &self.processes {
            let _killed = keeper::signal_kept_child(process.fd, Signal::Kill, Target::Child);
        }
        // This test owns no other children. Adoption also covers startup
        // failures before a detached descendant's report could be consumed.
        // Re-scan after reaping: killing an adopted leader can orphan its child.
        loop {
            let mut children = std::collections::BTreeSet::new();
            for task in fs::read_dir("/proc/self/task").expect("test threads") {
                if let Ok(pids) = fs::read_to_string(task.expect("test thread").path().join("children")) {
                    children.extend(pids.split_whitespace().map(|pid| pid.parse::<u32>().expect("owned child PID")));
                }
            }
            if children.is_empty() {
                break;
            }
            for pid in children {
                let fd = keeper::open_pidfd(pid).expect("retain unreaped adopted child");
                let _killed = keeper::signal_kept_child(fd, Signal::Kill, Target::Child);
                while keeper::poll_child(fd, true).expect("reap adopted fixture process").is_none() {
                    std::thread::sleep(POLL);
                }
                keeper::close_keeper_fd(fd);
            }
        }
        for process in self.processes.drain(..) {
            keeper::close_keeper_fd(process.fd);
        }
        drop(self.stderr.take());
        keeper::set_subreaper(self.subreaper).expect("restore adoption setting");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        self.end();
    }
}

fn owner_killed(mode: &str) {
    let mut tree = Tree::start(mode);
    let mut deadline = Patience::new("the fixture tree ending after its owner was killed", PATIENCE);
    for process in &tree.processes {
        deadline.watch_process(process.pid);
    }
    tree.kill_owner();
    let result = until(&deadline, || tree.survivors().is_empty());
    let survivors = tree.survivors();
    tree.end();
    if let Err(verdict) = result {
        expired(
            verdict,
            &format!("{mode} outlived its killed owner after 5 s: {} (killed during cleanup)", survivors.join(", ")),
        );
    }
}

#[test]
fn never_ends_with_its_killed_owner() {
    owner_killed("never");
}

#[test]
fn supervise_term_ends_with_its_killed_owner() {
    owner_killed("supervise-term");
}

#[test]
fn supervise_grandchild_ends_with_its_killed_owner() {
    owner_killed("supervise-grandchild");
}

#[test]
fn supervise_ignore_leader_ends_with_its_killed_owner() {
    owner_killed("supervise-ignore");
}

#[test]
fn fork_live_leader_ends_with_its_killed_owner() {
    owner_killed("fork-live");
}
