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

    /// Its earliest deadline, over every layer: override
    /// `next_mechanism_deadline` and inherit this minimum with the policy
    /// deadline, or override `next_deadline` itself.
    fn next_deadline(&self) -> Option<Time> {
        match (self.next_mechanism_deadline(), self.next_policy_deadline()) {
            (Some(mechanism), Some(policy)) => Some(mechanism.min(policy)),
            (Some(mechanism), None) => Some(mechanism),
            (None, Some(policy)) => Some(policy),
            (None, None) => None,
        }
    }

    /// Its earliest mechanism deadline: io's close, retry and file stall
    /// deadlines and any other layer's that is not policy (testing-strategy.md, 6).
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }

    /// Its earliest policy deadline, excluding io close, retry and file stalls
    /// (testing-strategy.md, section 6). Every host states it explicitly.
    fn next_policy_deadline(&self) -> Option<Time>;

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

/// A host made of hosts: a world's process enum over its kinds of process.
/// Each `Host` method goes to the variant `host` and `host_mut` name, every
/// provided one included (`exit`, `state_digest`, `drain`), so no dispatcher
/// forgets one (testing-strategy.md, 4: tests are ordinary Rust).
pub trait Dispatch {
    fn host(&self) -> &dyn Host;
    fn host_mut(&mut self) -> &mut dyn Host;
}

impl<D: Dispatch> Host for D {
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

    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
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

#[cfg(test)]
mod tests {
    use super::{Complete, Dispatch, Exit, Host, Queue, Submit, Time, Wall};

    struct Deadlines {
        mechanism: Option<Time>,
        policy: Option<Time>,
        completions: Queue<Complete>,
        submissions: Queue<Submit>,
        drained: bool,
    }

    impl Host for Deadlines {
        fn state_digest(&self) -> Option<[u8; 32]> {
            Some([7; 32])
        }
        fn drain(&mut self) {
            self.drained = true;
        }
        fn iterate(&mut self, _now: Time, _wall: Wall) {}
        fn completions(&mut self) -> &mut Queue<Complete> {
            &mut self.completions
        }
        fn submissions(&mut self) -> &mut Queue<Submit> {
            &mut self.submissions
        }
        fn work_pending(&self, _now: Time) -> bool {
            false
        }
        fn next_mechanism_deadline(&self) -> Option<Time> {
            self.mechanism
        }
        fn next_policy_deadline(&self) -> Option<Time> {
            self.policy
        }
        fn is_empty(&self) -> bool {
            true
        }
        fn exit(&self) -> Option<Exit> {
            Some(Exit::Code(17))
        }
        fn operations(&self) -> u32 {
            1
        }
    }

    struct Process(Deadlines);

    impl Dispatch for Process {
        fn host(&self) -> &dyn Host {
            &self.0
        }
        fn host_mut(&mut self) -> &mut dyn Host {
            &mut self.0
        }
    }

    #[test]
    fn a_hosts_deadline_is_the_earliest_mechanism_or_policy() {
        let early = Time::from_nanos(10);
        let late = Time::from_nanos(20);
        for (mechanism, policy, expected) in [
            (None, None, None),
            (Some(early), None, Some(early)),
            (None, Some(early), Some(early)),
            (Some(early), Some(late), Some(early)),
            (Some(late), Some(early), Some(early)),
            (Some(early), Some(early), Some(early)),
        ] {
            let host = Deadlines {
                mechanism,
                policy,
                completions: Queue::with_capacity(1),
                submissions: Queue::with_capacity(1),
                drained: false,
            };
            assert_eq!(host.next_deadline(), expected, "the earlier present deadline wakes the loop");
        }
    }

    #[test]
    fn dispatch_preserves_the_variants_digest_drain_and_failure_exit() {
        let mut process = Process(Deadlines {
            mechanism: None,
            policy: None,
            completions: Queue::with_capacity(1),
            submissions: Queue::with_capacity(1),
            drained: false,
        });
        assert_eq!(process.state_digest(), Some([7; 32]), "the variant supplies the replay state");
        assert_eq!(process.exit(), Some(Exit::Code(17)), "an empty variant retains its failure status");
        process.drain();
        assert!(process.0.drained, "the variant drains its shell diagnostics");
    }
}
