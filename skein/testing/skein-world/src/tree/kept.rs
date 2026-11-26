//! A walk of descendants kept below a live parent (skein/docs/design/examples.md, 6).
//! The leader is borrowed; the walk owns the exit watches it opens below it.

use alloc::collections::BTreeMap;

use skein_io::kernel::{Error, Fd, Signal, Target};
use skein_lib::Duration;
use skein_shell::keeper;

use super::NO_PROCESS;

// The adoption needed to reap this fixture is Linux's (examples.md, 6).
#[cfg(test)]
#[cfg(target_os = "linux")]
pub(super) mod tests;

#[derive(Debug)]
pub(super) struct KeptProcesses {
    members: BTreeMap<u32, Fd>,
}

impl KeptProcesses {
    pub(super) fn new() -> Self {
        Self { members: BTreeMap::new() }
    }

    pub(super) fn discover(
        &mut self,
        leader: Option<(u32, Fd)>,
        mut list: impl FnMut(u32) -> Result<Box<[u32]>, Error>,
    ) -> Result<(), String> {
        self.prune()?;
        let mut parents: Vec<_> = self.members.iter().map(|(&pid, &fd)| (pid, fd)).collect();
        parents.extend(leader);
        while let Some((parent, handle)) = parents.pop() {
            let is_leader = leader == Some((parent, handle));
            if !parent_live(handle, is_leader)? {
                continue;
            }
            let children = match list(parent) {
                Ok(children) => children,
                Err(NO_PROCESS | Error::NotFound | Error::InvalidArgument) => continue,
                Err(error) => return Err(format!("listing children of {parent}: {error:?}")),
            };
            for pid in children {
                if self.members.contains_key(&pid) {
                    continue;
                }
                let child = match keeper::open_child_pidfd(pid, parent) {
                    Ok(child) => child,
                    Err(NO_PROCESS | Error::NotFound | Error::InvalidArgument) => continue,
                    Err(error) => return Err(format!("keeping process {pid} below {parent}: {error:?}")),
                };
                // Check the parent again after binding: a numeric listing
                // or parent check that raced its exit is not our evidence.
                match parent_live(handle, is_leader) {
                    Ok(true) => {
                        self.members.insert(pid, child);
                        parents.push((pid, child));
                    }
                    otherwise => {
                        keeper::close_keeper_fd(child);
                        otherwise?;
                        break;
                    }
                }
            }
        }
        self.prune()
    }

    pub(super) fn prune(&mut self) -> Result<(), String> {
        let mut failure = Ok(());
        self.members.retain(|&pid, handle| match keeper::pidfd_exited(*handle) {
            Ok(false) => true,
            Ok(true) => {
                keeper::close_keeper_fd(*handle);
                false
            }
            Err(error) => {
                failure = Err(format!("polling kept process {pid}: {error:?}"));
                true
            }
        });
        failure
    }

    pub(super) fn live(&self) -> Vec<u32> {
        self.members.keys().copied().collect()
    }

    pub(super) fn signal(&self, signal: Signal) -> Result<Vec<u32>, String> {
        let mut delivered = Vec::new();
        let mut failure = Ok(());
        for (&pid, &handle) in &self.members {
            match keeper::pidfd_exited(handle) {
                Ok(true) => continue,
                Ok(false) => {}
                Err(error) => {
                    failure = Err(format!("polling kept process {pid}: {error:?}"));
                    continue;
                }
            }
            match keeper::signal_kept_child(handle, signal, Target::Child) {
                Ok(()) => delivered.push(pid),
                Err(NO_PROCESS) => {}
                Err(error) => failure = Err(format!("signalling kept process {pid}: {error:?}")),
            }
        }
        failure.map(|()| delivered)
    }

    pub(super) fn finish_killing(&mut self) -> Result<(), String> {
        let clock = skein_shell::Clock::new();
        let until = clock.now().now.saturating_add(Duration::from_secs(2));
        loop {
            self.prune()?;
            if self.members.is_empty() {
                return Ok(());
            }
            if clock.now().now >= until {
                return Err(format!("left running: {:?}", self.live()));
            }
            let _delivered = self.signal(Signal::Kill)?;
            std::thread::sleep(core::time::Duration::from_millis(10));
        }
    }
}

impl Drop for KeptProcesses {
    fn drop(&mut self) {
        for &handle in self.members.values() {
            keeper::close_keeper_fd(handle);
        }
    }
}

fn parent_live(handle: Fd, leader: bool) -> Result<bool, String> {
    let live = if leader {
        keeper::poll_child(handle, false).map(|exit| exit.is_none())
    } else {
        keeper::pidfd_exited(handle).map(|exit| !exit)
    };
    live.map_err(|error| format!("polling parent {handle:?}: {error:?}"))
}
