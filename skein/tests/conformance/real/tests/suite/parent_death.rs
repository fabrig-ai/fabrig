//! Parent-death exec against a real process tree, with an ordinary exec control.
#![cfg(target_os = "linux")]

use std::fs;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

use skein_io::kernel::{Exit, Fd, Signal};
use skein_lib::Duration;
use skein_scratch::Scratch;
use skein_shell::Clock;
use skein_world::Expired;

use super::given_listener::deadlines::Patience;

// The task's exit bound; startup uses the same bound so a broken helper fails promptly.
const PATIENCE: Duration = Duration::from_secs(5);
// Long enough to observe that ordinary exec survives, well below sleep's 30 seconds.
const SURVIVAL: Duration = Duration::from_millis(100);
const POLL: std::time::Duration = std::time::Duration::from_millis(1);

fn until(deadline: &Patience, mut ready: impl FnMut() -> bool) -> Result<(), Expired> {
    while !ready() {
        if deadline.passed() {
            return Err(deadline.expired());
        }
        std::thread::sleep(std::time::Duration::from_nanos(
            deadline.remaining().min(Duration::from_millis(1)).as_nanos(),
        ));
    }
    Ok(())
}

fn expired(expired: Expired, message: &str) -> ! {
    match expired {
        Expired::Starved(starved) => starved.pass(),
        Expired::Failed(figures) => panic!("{message}: {figures}"),
    }
}

struct Tree {
    parent: Child,
    parent_fd: Fd,
    child: Option<Fd>,
    scratch: Option<Scratch>,
    ended: bool,
}

impl Tree {
    fn start(mode: &str) -> Self {
        skein_shell::make_subreaper().expect("adopt and reap orphaned sleep");
        let scratch = Scratch::new("parent-death");
        let parent = Command::new(skein_scratch::paths::binary("process_fixture"))
            .args(["parent-death-parent", mode])
            .current_dir(scratch.path())
            .process_group(0)
            .stdin(Stdio::piped())
            .spawn()
            .expect("spawn parent fixture");
        let parent_fd = skein_shell::open_pidfd(parent.id()).expect("keep unreaped group leader");
        let mut tree = Self { parent, parent_fd, child: None, scratch: Some(scratch), ended: false };
        let mut pid = None;
        let mut deadline = Patience::new("the parent fixture reporting its child", PATIENCE);
        deadline.watch_process(tree.parent.id());
        let result = until(&deadline, || {
            assert!(
                skein_shell::poll_child(tree.parent_fd, false).expect("parent startup status").is_none(),
                "parent fixture exited before reporting its child"
            );
            pid = fs::read_to_string(tree.scratch.as_ref().expect("scratch").path().join("child-pid"))
                .ok()
                .and_then(|s| s.parse().ok());
            pid.is_some()
        });
        tree.check_wait(result, "process tree did not reach its expected state within 5 s");
        let pid = pid.expect("parent reported child PID");
        tree.child =
            Some(skein_shell::open_child_pidfd(pid, tree.parent.id()).expect("validate child identity before exec"));
        let mut deadline = Patience::new("the child executing sleep", PATIENCE);
        deadline.watch_process(pid);
        tree.parent.stdin.take().expect("parent input").write_all(b"x").expect("release exec child");
        let sleep = fs::canonicalize("/bin/sleep").expect("sleep executable");
        let result = until(&deadline, || {
            assert!(!tree.child_exited(), "child exited before executing sleep");
            fs::read_link(format!("/proc/{pid}/exe")).ok().as_ref() == Some(&sleep)
        });
        tree.check_wait(result, "process tree did not reach its expected state within 5 s");
        tree
    }

    fn kill_parent(&mut self) {
        let mut deadline = Patience::new("the killed parent fixture exiting", PATIENCE);
        deadline.watch_process(self.parent.id());
        self.parent.kill().expect("SIGKILL parent");
        // Retain the zombie leader until Drop has settled its owned group.
        let result =
            until(&deadline, || skein_shell::poll_child(self.parent_fd, false).expect("retain parent exit").is_some());
        self.check_wait(result, "process tree did not reach its expected state within 5 s");
    }

    fn child_exited(&self) -> bool {
        skein_shell::pidfd_exited(self.child.expect("registered child")).expect("poll sleep identity")
    }
}

impl Tree {
    fn check_wait(&mut self, result: Result<(), Expired>, message: &str) {
        if let Err(verdict) = result {
            self.end(matches!(&verdict, Expired::Starved(_)));
            expired(verdict, message);
        }
    }

    fn reap_adopted(&self) {
        // A private group and its retained leader identify orphaned children
        // even when startup ended before the fixture published their PID.
        let mut children = std::collections::BTreeSet::new();
        for task in fs::read_dir("/proc/self/task").expect("our threads") {
            let task = task.expect("our thread");
            if let Ok(pids) = fs::read_to_string(task.path().join("children")) {
                children.extend(pids.split_whitespace().map(|pid| pid.parse::<u32>().expect("an owned child PID")));
            }
        }
        for pid in children {
            if pid == self.parent.id() {
                continue;
            }
            // These are our unreaped children: no other owner reaps them,
            // so their numeric identities stay bound while the handle opens.
            let descriptor = skein_shell::open_pidfd(pid).expect("retain an adopted child");
            let stat = fs::read_to_string(format!("/proc/{pid}/stat")).expect("adopted child's identity");
            let mut fields = stat.rsplit_once(") ").expect("proc stat").1.split_whitespace();
            let _state = fields.next().expect("state field");
            let parent: u32 = fields.next().expect("parent field").parse().expect("parent PID");
            let group: u32 = fields.next().expect("group field").parse().expect("group PID");
            if parent == std::process::id() && group == self.parent.id() {
                skein_shell::abandon_binary(Some(descriptor), &[descriptor]);
            } else {
                skein_shell::close_keeper_fd(descriptor);
            }
        }
    }

    fn end(&mut self, classify: bool) {
        if self.ended {
            return;
        }
        self.ended = true;
        // Register a reported child even if startup timed out before consuming its PID.
        if self.child.is_none()
            && let Some(pid) = self
                .scratch
                .as_ref()
                .and_then(|scratch| fs::read_to_string(scratch.path().join("child-pid")).ok())
                .and_then(|pid| pid.parse().ok())
        {
            self.child = skein_shell::open_child_pidfd(pid, self.parent.id())
                .ok()
                .or_else(|| skein_shell::open_child_pidfd(pid, std::process::id()).ok());
        }
        let mut deadline = Patience::new("the killed parent exiting during cleanup", PATIENCE);
        deadline.watch_process(self.parent.id());
        // The private group also covers a child not yet reported during failed startup.
        let _killed = skein_shell::signal_kept_child(self.parent_fd, Signal::Kill, skein_io::kernel::Target::Group);
        let _killed = self.parent.kill();
        if let Some(child) = self.child {
            let _killed = skein_shell::signal_kept_child(child, Signal::Kill, skein_io::kernel::Target::Child);
        }
        let result = until(&deadline, || {
            skein_shell::poll_child(self.parent_fd, false).expect("observe retained parent").is_some()
        });
        // After an unanswered cleanup wait, finish the same owned teardown before exiting.
        while skein_shell::poll_child(self.parent_fd, false).expect("observe retained parent").is_none() {
            std::thread::sleep(POLL);
        }
        if let Some(child) = self.child.take() {
            skein_shell::abandon_binary(Some(child), &[child]);
        }
        self.reap_adopted();
        let _reaped = self.parent.wait();
        drop(self.parent.stdin.take());
        skein_shell::close_keeper_fd(self.parent_fd);
        drop(self.scratch.take());
        if classify && let Err(verdict) = result {
            expired(verdict, "process tree did not reach its expected state within 5 s");
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        self.end(!std::thread::panicking());
    }
}

#[test]
fn a_killed_parent_is_retained_until_its_group_is_settled() {
    let mut tree = Tree::start("plain");
    tree.kill_parent();
    assert_eq!(skein_shell::poll_child(tree.parent_fd, false), Ok(Some(Exit::Signal(9))));
    assert!(!tree.child_exited(), "plain exec remains for owned cleanup");
}

#[test]
fn a_parent_killed_after_exec_takes_its_child_with_it() {
    let mut tree = Tree::start("protected");
    let clock = Clock::new();
    let start = clock.now().now;
    let mut deadline = Patience::new("the parent-death child exiting after SIGKILL", PATIENCE);
    deadline.watch_process(skein_shell::child_id(tree.child.expect("child identity")).expect("the owned child's PID"));
    tree.kill_parent();
    let result = until(&deadline, || tree.child_exited());
    tree.check_wait(result, "parent-death signal must end sleep within 5 s");
    let elapsed = clock.now().now.saturating_since(start);
    eprintln!("parent SIGKILL to sleep exit: {} ns", elapsed.as_nanos());
    assert_eq!(
        skein_shell::poll_child(tree.child.expect("child identity"), false),
        Ok(Some(Exit::Signal(9))),
        "sleep died from the installed signal"
    );
}

#[test]
fn ordinary_exec_survives_its_parents_death() {
    let mut tree = Tree::start("plain");
    tree.kill_parent();
    let clock = Clock::new();
    let deadline = clock.now().now.saturating_add(SURVIVAL);
    while clock.now().now < deadline {
        assert!(!tree.child_exited(), "ordinary exec must leave sleep alive");
        std::thread::sleep(POLL);
    }
}

struct DirectChild(Child);

impl Drop for DirectChild {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _reaped = self.0.wait();
    }
}

#[test]
fn a_wrong_parent_is_refused_before_exec() {
    let scratch = Scratch::new("wrong-parent");
    let stderr = fs::File::create(scratch.path().join("stderr")).expect("capture refusal");
    let mut child = DirectChild(
        Command::new(skein_scratch::paths::binary("process_fixture"))
            .args(["parent-death-child", "--parent", "4294967295", "protected"])
            .current_dir(scratch.path())
            .stdin(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .expect("spawn child with impossible parent PID"),
    );
    child.0.stdin.take().expect("child input").write_all(b"x").expect("release exec child");
    let mut status = None;
    let mut deadline = Patience::new("the refused exec child exiting", PATIENCE);
    deadline.watch_process(child.0.id());
    let result = until(&deadline, || {
        status = child.0.try_wait().expect("poll refused child");
        status.is_some()
    });
    if let Err(verdict) = result {
        drop(child);
        drop(scratch);
        expired(verdict, "process tree did not reach its expected state within 5 s");
    }
    assert!(!status.expect("child exited").success(), "wrong parent must fail");
    assert!(
        fs::read_to_string(scratch.path().join("stderr"))
            .expect("refusal message")
            .contains("parent-death exec refused: parent changed before exec"),
        "the child must report the parent mismatch instead of executing sleep"
    );
}

#[test]
fn cleanup_reaps_a_child_without_a_reported_pid() {
    let mut tree = Tree::start("plain");
    let child = tree.child.take().expect("registered child");
    let keeper = skein_shell::open_child_pidfd(skein_shell::child_id(child).expect("child PID"), tree.parent.id())
        .expect("retain the child's identity for its reap check");
    skein_shell::close_keeper_fd(child);
    fs::remove_file(tree.scratch.as_ref().expect("scratch").path().join("child-pid")).expect("remove the PID report");
    drop(tree);
    let reaped = skein_shell::poll_child(keeper, false);
    skein_shell::close_keeper_fd(keeper);
    assert_eq!(
        reaped,
        Err(skein_io::kernel::Error::Other(libc::ECHILD)),
        "cleanup reaps the adopted child without its PID report"
    );
}
