//! Host deadlines and process forwarding (shell.md, 12), and referee defaults
//! (testing-strategy.md, 7).

use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_world::{Forward, Host, Referee};
use skein_world_tests::hosted::Script;

struct Deadlines {
    script: Script,
    mechanism: Option<Time>,
    policy: Option<Time>,
}

impl Host for Deadlines {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.script.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.script.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.script.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.script.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.mechanism
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.policy
    }
    fn is_empty(&self) -> bool {
        self.script.is_empty()
    }
    fn operations(&self) -> u32 {
        self.script.operations()
    }
}

#[test]
fn the_provided_deadline_selects_the_earliest_present_layer() {
    let early = Some(Time::from_nanos(2));
    let late = Some(Time::from_nanos(5));
    for (mechanism, policy, expected) in [
        (None, None, None),
        (early, None, early),
        (None, late, late),
        (early, late, early),
        (late, early, early),
        (early, early, early),
    ] {
        let host = Deadlines { script: Script::new(&[]), mechanism, policy };
        assert_eq!(host.next_deadline(), expected, "mechanism {mechanism:?}, policy {policy:?}");
    }
    // Script has no mechanism deadline and keeps its existing deadline override.
    assert_eq!(Script::new(&[]).next_mechanism_deadline(), None);
}

struct Legacy {
    script: Script,
}

impl Host for Legacy {
    fn state_digest(&self) -> Option<[u8; 32]> {
        Some([7; 32])
    }
    fn drain(&mut self) {
        self.script.drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.script.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.script.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.script.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.script.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        Some(Time::from_nanos(5))
    }
    fn next_deadline(&self) -> Option<Time> {
        // An existing override may differ from the provided merge.
        Some(Time::from_nanos(9))
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        Some(Time::from_nanos(2))
    }
    fn is_empty(&self) -> bool {
        self.script.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.script.exit()
    }
    fn operations(&self) -> u32 {
        self.script.operations()
    }
}

struct Process(Legacy);

impl Forward for Process {
    fn host(&self) -> &dyn Host {
        &self.0
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        &mut self.0
    }
}

#[test]
fn forwarding_preserves_the_hosts_overrides_and_terminal_status() {
    let mut script = Script::new(&[]);
    script.terminal = Some(Exit::Code(7));
    let mut process = Process(Legacy { script });
    assert_eq!(process.next_deadline(), Some(Time::from_nanos(9)), "preserve the override");
    assert_eq!(process.next_mechanism_deadline(), Some(Time::from_nanos(5)));
    assert_eq!(process.next_policy_deadline(), Some(Time::from_nanos(2)));
    assert_eq!(process.state_digest(), Some([7; 32]));
    process.iterate(Time::ZERO, Wall::EPOCH);
    process.drain();
    assert_eq!(process.0.script.iterations, 1);
    assert_eq!(process.0.script.drains, 1);
    assert!(process.is_empty());
    assert_eq!(process.exit(), Some(Exit::Code(7)), "forward the hosts failure exit");
}

struct Quiet;

impl Referee<()> for Quiet {
    fn observe(&mut self, _now: Time, _procs: &[()]) {}
    fn passed(&self) -> bool {
        true
    }
}

#[test]
fn a_quiet_referee_injects_nothing_and_keeps_no_deadline() {
    let mut referee = Quiet;
    referee.act(Time::ZERO, &mut [()]);
    assert_eq!(referee.next_deadline(), None);
    assert_eq!(referee.overdue(Time::from_nanos(u64::MAX)), None);
    assert!(referee.passed());
}
