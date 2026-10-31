//! `Host`'s provided deadline and `Delegate`'s forwarding (shell.md, 12).

use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};

use crate::{Delegate, Host};

/// A host whose every answer is a field, and which counts its turns and drains.
struct Fixed {
    mechanism: Option<Time>,
    policy: Option<Time>,
    digest: Option<[u8; 32]>,
    exit: Option<Exit>,
    empty: bool,
    pending: bool,
    operations: u32,
    iterations: u32,
    drains: u32,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}

impl Fixed {
    fn new(mechanism: Option<Time>, policy: Option<Time>) -> Fixed {
        Fixed {
            mechanism,
            policy,
            digest: None,
            exit: None,
            empty: false,
            pending: false,
            operations: 1,
            iterations: 0,
            drains: 0,
            completions: Queue::with_capacity(1),
            submissions: Queue::with_capacity(1),
        }
    }
}

impl Host for Fixed {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.digest
    }
    fn drain(&mut self) {
        self.drains = self.drains.checked_add(1).expect("a few drains");
    }
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        self.iterations = self.iterations.checked_add(1).expect("a few turns");
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        self.pending
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.mechanism
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.policy
    }
    fn is_empty(&self) -> bool {
        self.empty
    }
    fn exit(&self) -> Option<Exit> {
        self.exit
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}

/// A world's enum of hosts, as a scenario writes it.
enum Process {
    Service(Box<Fixed>),
    Fake(Fixed),
}

impl Delegate for Process {
    fn host(&self) -> &dyn Host {
        match self {
            Process::Service(host) => host.as_ref(),
            Process::Fake(host) => host,
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Process::Service(host) => host.as_mut(),
            Process::Fake(host) => host,
        }
    }
}

fn at(nanos: u64) -> Time {
    Time::from_nanos(nanos)
}

#[test]
fn the_next_deadline_is_the_earlier_of_the_mechanism_and_the_policy_deadlines() {
    for (mechanism, policy, next) in [
        (None, None, None),
        (Some(at(5)), None, Some(at(5))),
        (None, Some(at(3)), Some(at(3))),
        (Some(at(5)), Some(at(3)), Some(at(3))),
        (Some(at(2)), Some(at(3)), Some(at(2))),
        (Some(at(4)), Some(at(4)), Some(at(4))),
    ] {
        assert_eq!(Fixed::new(mechanism, policy).next_deadline(), next, "{mechanism:?} and {policy:?}");
    }
}

#[test]
fn a_delegate_is_the_host_it_lends_in_every_method() {
    let mut service = Fixed::new(Some(at(7)), Some(at(9)));
    service.digest = Some([1; 32]);
    service.exit = Some(Exit::Code(3));
    service.empty = true;
    service.pending = true;
    service.operations = 12;
    let mut processes = [Process::Service(Box::new(service)), Process::Fake(Fixed::new(None, Some(at(2))))];
    for process in &mut processes {
        process.iterate(Time::ZERO, Wall::EPOCH);
        process.drain();
        process.completions().push(Complete {
            op: Token::new(1),
            kind: Op::Close { fd: Fd::new(-1) },
            result: Ok(Done::Nothing),
        });
        assert!(process.submissions().is_empty(), "the host's own submissions");
    }
    let [Process::Service(service), Process::Fake(fake)] = &processes else { panic!("a service, then a fake") };
    assert_eq!((service.iterations, service.drains, service.completions.len()), (1, 1, 1), "the service's turn");
    assert_eq!((fake.iterations, fake.drains, fake.completions.len()), (1, 1, 1), "the fake's turn");
    let [service, fake] = &processes;
    assert_eq!(service.state_digest(), Some([1; 32]), "the host's digest");
    assert!(service.work_pending(Time::ZERO), "the service's pending work");
    assert_eq!(
        (service.next_mechanism_deadline(), service.next_policy_deadline()),
        (Some(at(7)), Some(at(9))),
        "both deadlines"
    );
    assert_eq!(service.next_deadline(), Some(at(7)), "the earlier");
    assert!(service.is_empty(), "the service is empty");
    assert_eq!(service.exit(), Some(Exit::Code(3)), "the host's exit, not the empty host's default");
    assert_eq!(service.operations(), 12, "its ring");
    assert_eq!(fake.state_digest(), None, "no digest");
    assert!(!fake.work_pending(Time::ZERO), "the fake has none");
    assert_eq!(fake.next_deadline(), Some(at(2)), "its policy deadline alone");
    assert!(!fake.is_empty(), "the fake holds something");
    assert_eq!(fake.exit(), None, "not yet ended");
    assert_eq!(fake.operations(), 1, "its ring");
}
