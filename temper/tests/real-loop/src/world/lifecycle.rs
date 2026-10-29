//! Judge decisions over host snapshots (temper/docs/design/domain/testing.md, 2.4).
use crate::calls::Caller;

#[derive(Default)]
pub(super) struct Lifecycle {
    pub(super) stopping: bool,
    pub(super) engine_stopping: bool,
    pub(super) finished: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Hosts {
    pub(super) service_empty: bool,
    pub(super) workers_empty: bool,
    pub(super) children_settled: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Shutdown {
    pub(super) workers: bool,
    pub(super) engine: bool,
    pub(super) peers: bool,
}

impl Lifecycle {
    pub(super) fn release(caller: &mut Caller, links: u32, workers_ready: bool) {
        if links != 0 && workers_ready {
            caller.released = true;
        }
    }

    pub(super) fn shutdown(&mut self, caller_empty: bool, hosts: Hosts) -> Shutdown {
        let mut shutdown = Shutdown::default();
        if caller_empty && hosts.children_settled && !self.stopping {
            shutdown.workers = true;
            self.stopping = true;
        }
        if self.stopping && hosts.workers_empty && !self.engine_stopping {
            shutdown.engine = true;
            self.engine_stopping = true;
        }
        if hosts.service_empty && caller_empty && !self.finished {
            shutdown.peers = true;
            self.finished = true;
        }
        shutdown
    }

    pub(super) fn passed(&self, failed: bool) -> bool {
        self.finished && !failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{calls, fixtures};
    use skein_io::kernel::{Fd, Op};
    use skein_lib::{Time, Wall};
    use skein_shell::Host;
    use std::path::PathBuf;

    #[test]
    fn ready_links_release_the_caller_and_admit_its_expected_story_once() {
        let mut caller = Caller::new(vec![calls::sign_in(calls::D1)], PathBuf::new(), Fd::new(3));
        let mut releases = 0;
        for (links, workers_ready, admitted) in
            [(0, false, false), (0, true, false), (1, false, false), (1, true, true), (2, true, true), (0, false, true)]
        {
            let before = caller.released;
            Lifecycle::release(&mut caller, links, workers_ready);
            releases += usize::from(!before && caller.released);
            assert_eq!(caller.released, admitted);
            caller.iterate(Time::ZERO, Wall::from_nanos(0));
            assert_eq!(caller.calls.len(), usize::from(admitted));
        }
        assert_eq!(releases, 1, "a later link or readiness loss never readmits the caller");
        let submission = caller.submissions().pop().expect("admitted sign-in");
        let Op::Spawn { spawn } = submission.kind else { panic!("story admission spawns ctl") };
        assert_eq!(spawn.program.as_ref(), fixtures::CTL);
        let arguments: Vec<_> = spawn.args.iter().map(AsRef::as_ref).collect();
        assert_eq!(arguments, [b"clients/client-1".as_slice(), b"sign-in", b"link"]);
        assert_eq!(spawn.root, Fd::new(3));
        assert_eq!(caller.calls[0].tries, 1);
        assert!(caller.submissions().is_empty(), "no second command was admitted");
    }

    #[test]
    fn shutdown_waits_for_the_caller_children_and_workers_and_requests_finish_once() {
        let mut lifecycle = Lifecycle::default();
        for (caller_empty, children_settled, workers_empty, service_empty, expected) in [
            (false, false, false, false, Shutdown::default()),
            (false, true, true, true, Shutdown::default()),
            (true, false, false, false, Shutdown::default()),
            (true, true, false, false, Shutdown { workers: true, engine: false, peers: false }),
            (true, true, false, false, Shutdown::default()),
            (true, true, true, false, Shutdown { workers: false, engine: true, peers: false }),
            (true, true, true, false, Shutdown::default()),
            (false, true, true, true, Shutdown::default()),
            (true, true, true, true, Shutdown { workers: false, engine: false, peers: true }),
            (true, true, true, true, Shutdown::default()),
        ] {
            let was_finished = lifecycle.finished;
            let shutdown = lifecycle.shutdown(caller_empty, Hosts { service_empty, workers_empty, children_settled });
            assert_eq!(shutdown, expected);
            assert_eq!(lifecycle.finished, was_finished || expected.peers);
            assert_eq!(lifecycle.passed(false), lifecycle.finished);
        }
        assert!(lifecycle.stopping && lifecycle.engine_stopping && lifecycle.finished);
    }

    #[test]
    fn passing_requires_finished_without_failure() {
        for (finished, failed, passed) in
            [(false, false, false), (false, true, false), (true, false, true), (true, true, false)]
        {
            let lifecycle = Lifecycle { finished, ..Lifecycle::default() };
            assert_eq!(lifecycle.passed(failed), passed);
        }
    }
}
