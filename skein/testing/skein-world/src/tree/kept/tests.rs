//! The kept-process walk against Linux's pidfds (skein/docs/design/examples.md, 6).

use core::time::Duration as Sleep;
use skein_io::kernel::{Fd, Signal, Spawn, Target};
use skein_lib::Duration;
use skein_shell::{Clock, keeper};
use std::path::Path;

use super::KeptProcesses;

const PATIENCE: Duration = Duration::from_secs(2);

struct Family {
    subreaper: bool,
    leader: (u32, Fd),
    descendants: Vec<Fd>,
}

impl Family {
    fn start() -> Self {
        let subreaper = keeper::subreaper().expect("read subreaper setting");
        keeper::make_subreaper().expect("adopt only to reap the test fixture");
        let root = skein_shell::open_root(Path::new("/tmp")).expect("fixture directory");
        let mut spawn = Spawn {
            root,
            reach: None,
            dir: Box::new([]),
            program: Box::from(b"/bin/sh".as_slice()),
            args: [b"-c".as_slice(), b"/usr/bin/setsid /bin/sh -c '/bin/sleep 10 & wait' & wait"]
                .into_iter()
                .map(Box::from)
                .collect(),
            env: Box::new([]),
            pipes: Box::new([]),
        };
        let started = keeper::start_binary(&mut spawn, false);
        keeper::close_keeper_fd(root);
        let (handle, _) = started.expect("start the shell leader");
        let leader = (keeper::child_id(handle).expect("leader pid"), handle);
        Self { subreaper, leader, descendants: Vec::new() }
    }

    fn find_child(&mut self, parent: u32) -> u32 {
        let clock = Clock::new();
        let until = clock.now().now.saturating_add(PATIENCE);
        loop {
            if let Some(&pid) = keeper::process_children(parent).expect("list the live parent's children").first() {
                self.descendants.push(keeper::open_child_pidfd(pid, parent).expect("keep a separate reap handle"));
                return pid;
            }
            assert!(clock.now().now < until, "did not find a child below {parent}");
            std::thread::sleep(Sleep::from_millis(10));
        }
    }
}

impl Drop for Family {
    fn drop(&mut self) {
        // Only the fixture owner waits. Its second set of handles is not
        // shared with the walk, which must do no reaping of its own.
        keeper::abandon_binary(Some(self.leader.1), &[self.leader.1]);
        for handle in self.descendants.drain(..) {
            let _killed = keeper::signal_kept_child(handle, Signal::Kill, Target::Child);
            let _exited = keeper::pidfd_exited(handle).is_ok_and(|exited| exited)
                || keeper::wait_readable(handle, PATIENCE) == Ok(true);
            let _reaped = keeper::poll_child(handle, true);
            keeper::close_keeper_fd(handle);
        }
        keeper::set_subreaper(self.subreaper).expect("restore the test process's subreaper setting");
    }
}

fn reap(handle: Fd) {
    assert_eq!(keeper::wait_readable(handle, PATIENCE), Ok(true), "exit is posted before the reap");
    assert!(
        keeper::poll_child(handle, true).expect("reap the fixture process").is_some(),
        "the owner reaps the posted exit"
    );
}

pub(in crate::tree) fn check_kept_family() {
    let mut family = Family::start();
    let (leader, handle) = family.leader;
    let mut walk = KeptProcesses::new();
    let child = family.find_child(leader);
    let grandchild = family.find_child(child);
    walk.discover(Some(family.leader), keeper::process_children).expect("poll the walk below the live leader");
    let mut descendants = vec![child, grandchild];
    descendants.sort_unstable();
    assert_eq!(walk.live(), descendants, "the leader is borrowed and the two descendants are kept");

    keeper::signal_kept_child(handle, Signal::Kill, Target::Child).expect("kill only the leader");
    reap(handle);
    let mut visited = Vec::new();
    walk.discover(None, |pid| {
        visited.push(pid);
        keeper::process_children(pid)
    })
    .expect("poll with no live leader");
    visited.sort_unstable();
    assert_eq!(visited, descendants, "only the two live kept processes are listed");
    assert_eq!(walk.live(), descendants, "both descendants survive the leader's reap");

    let handles: Vec<_> = walk.members.values().copied().collect();
    assert_eq!(walk.signal(Signal::Kill), Ok(descendants), "the walk signals each live kept process");
    for &handle in &family.descendants {
        reap(handle);
    }
    assert!(handles.iter().all(|&fd| keeper::pidfd_exited(fd) == Ok(true)));
    visited.clear();
    walk.discover(None, |pid| {
        visited.push(pid);
        keeper::process_children(pid)
    })
    .expect("poll after both kept exits posted");
    assert!(visited.is_empty(), "did not list children of exited processes");
    assert!(walk.live().is_empty(), "kept processes are pruned after exit");
    assert!(handles.iter().all(|&fd| keeper::pidfd_exited(fd) == Err(skein_io::kernel::Error::InvalidArgument)));
    walk.finish_killing().expect("the tree is settled");
}
