//! Every host method crosses the delegate boundary, including provided
//! methods (skein/docs/plan/component-kit/notes/decisions.md, 6.1).

use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};

use crate::{Delegate, Host};

struct Probe {
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    turn: Option<(Time, Wall)>,
    drained: bool,
}

impl Host for Probe {
    fn state_digest(&self) -> Option<[u8; 32]> {
        Some([7; 32])
    }

    fn drain(&mut self) {
        self.drained = true;
    }

    fn iterate(&mut self, now: Time, wall: Wall) {
        self.turn = Some((now, wall));
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, now: Time) -> bool {
        self.turn.is_some_and(|(last, _wall)| last == now)
    }

    fn next_deadline(&self) -> Option<Time> {
        Some(Time::from_nanos(30))
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        Some(Time::from_nanos(10))
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        Some(Time::from_nanos(20))
    }

    fn is_empty(&self) -> bool {
        self.drained
    }

    fn exit(&self) -> Option<Exit> {
        self.drained.then_some(Exit::Code(1))
    }

    fn operations(&self) -> u32 {
        17
    }
}

struct Forwarded(Probe);

impl Delegate for Forwarded {
    fn host(&self) -> &dyn Host {
        &self.0
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        &mut self.0
    }
}

#[test]
fn delegation_preserves_every_host_method() {
    let mut host =
        Forwarded(Probe { completions: Queue::uncapped(), submissions: Queue::uncapped(), turn: None, drained: false });
    let now = Time::from_nanos(40);
    let wall = Wall::from_nanos(50);
    assert_eq!(host.state_digest(), Some([7; 32]), "the selected host supplies the digest");
    assert_eq!(host.next_deadline(), Some(Time::from_nanos(30)), "an overridden deadline is preserved");
    assert_eq!(host.next_mechanism_deadline(), Some(Time::from_nanos(10)), "the mechanism deadline is forwarded");
    assert_eq!(host.next_policy_deadline(), Some(Time::from_nanos(20)), "the policy deadline is forwarded");
    assert_eq!(host.operations(), 17, "the selected host prices the ring");
    let completions = std::ptr::from_mut(&mut host.0.completions);
    assert_eq!(std::ptr::from_mut(host.completions()), completions, "completions reach the selected host's queue");
    let submissions = std::ptr::from_mut(&mut host.0.submissions);
    assert_eq!(std::ptr::from_mut(host.submissions()), submissions, "submissions come from the selected host's queue");
    assert!(!host.work_pending(now), "the selected host has not iterated");
    host.iterate(now, wall);
    assert_eq!(host.0.turn, Some((now, wall)), "both clocks reach the selected host");
    assert!(host.work_pending(now), "the selected host's pending work is visible");
    assert!(!host.work_pending(Time::ZERO), "the query time reaches the selected host");
    assert!(!host.is_empty(), "the selected host has diagnostics to drain");
    assert_eq!(host.exit(), None, "the selected host has no terminal exit yet");
    host.drain();
    assert!(host.0.drained, "the selected host drains diagnostics");
    assert!(host.is_empty(), "the selected host is empty after draining");
    assert_eq!(host.exit(), Some(Exit::Code(1)), "the selected host's failure exit survives forwarding");
}
