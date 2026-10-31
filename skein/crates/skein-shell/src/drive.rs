//! A service loop and its adapter (shell.md, sections 12 and 13).
//! The host keeps every service entity and queue; the loop knows only its
//! settlement, work and deadlines, never its domain state.

use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};

/// A process a world hosts: a service, or a fake, each a step machine with a
/// loop of its own (programming-model.md, section 2). The harness reaps its
/// completions into [`Host::completions`], calls [`Host::iterate`], and
/// submits [`Host::submissions`], as its shell would.
pub trait Host {
    /// A digest of held state for replay (lib.md, 5.1); only worlds read it.
    fn state_digest(&self) -> Option<[u8; 32]> {
        None
    }

    /// Drain shell diagnostics once, after iteration and before submission.
    fn drain(&mut self) {}

    /// One turn of its loop, between its reap and its submit: both passes
    /// and the reclaim point, at `now` and `wall`.
    fn iterate(&mut self, now: Time, wall: Wall);

    fn completions(&mut self) -> &mut Queue<Complete>;

    fn submissions(&mut self) -> &mut Queue<Submit>;

    /// Whether `iterate` has work at `now` without the kernel.
    fn work_pending(&self, now: Time) -> bool;

    /// Its earliest mechanism deadline: io's own, its close and retry
    /// deadlines and the stall deadlines owners state for files (io.md, 2).
    /// Every host states it explicitly.
    fn next_mechanism_deadline(&self) -> Option<Time>;

    /// Its earliest policy deadline, excluding io close, retry and file stalls
    /// (testing-strategy.md, section 6). Every host states it explicitly: it is
    /// the one the teardown invariant watches.
    fn next_policy_deadline(&self) -> Option<Time>;

    /// Its earliest deadline, over every layer: the earlier of its mechanism
    /// and policy deadlines.
    fn next_deadline(&self) -> Option<Time> {
        match (self.next_mechanism_deadline(), self.next_policy_deadline()) {
            (Some(mechanism), Some(policy)) => Some(mechanism.min(policy)),
            (Some(deadline), None) | (None, Some(deadline)) => Some(deadline),
            (None, None) => None,
        }
    }

    /// Whether it holds nothing: every slab empty, nothing in flight, every
    /// queue empty (testing-strategy.md, 6).
    fn is_empty(&self) -> bool;

    /// A hosted child's terminal exit, after it closes its resources. Hosts
    /// with a failure status override the successful empty-host default.
    fn exit(&self) -> Option<Exit> {
        if self.is_empty() { Some(Exit::Code(0)) } else { None }
    }

    /// The most operations it has in flight at once: the size of its ring.
    fn operations(&self) -> u32;
}

/// A process that is one of several hosts, such as a world's enum of its
/// processes: it lends the host it holds, and is that host in every method,
/// while its owner still matches on what it holds (shell.md, 12).
pub trait Delegate {
    fn host(&self) -> &dyn Host;

    fn host_mut(&mut self) -> &mut dyn Host;
}

impl<D: Delegate> Host for D {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host().state_digest()
    }

    fn drain(&mut self) {
        self.host_mut().drain();
    }

    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }

    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }

    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }

    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }

    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }

    fn operations(&self) -> u32 {
        self.host().operations()
    }
}

/// Runs a service until its entities and kernel operations have settled.
/// Its exit belongs to the host, after its final drain (shell.md, 13).
#[cfg(unix)]
pub fn drive(kernel: &mut crate::Kernel, clock: &crate::Clock, host: &mut impl Host) -> Exit {
    loop {
        kernel.reap(host.completions());
        let crate::Now { now, wall } = clock.now();
        host.iterate(now, wall);
        host.drain();
        if host.is_empty() {
            assert!(host.completions().is_empty(), "an empty host has no completions");
            assert!(host.submissions().is_empty(), "an empty host has no submissions");
            assert_eq!(kernel.in_flight(), 0, "an empty host has no kernel operation in flight");
            return host.exit().expect("an empty host has a terminal exit");
        }
        let wait = if host.work_pending(now) {
            crate::Wait::No
        } else {
            match host.next_deadline() {
                Some(deadline) => crate::Wait::Until(deadline),
                None => crate::Wait::Forever,
            }
        };
        kernel.submit(host.submissions(), wait);
    }
}
