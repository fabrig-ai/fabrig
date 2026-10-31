//! Check cancellation crosses the run and machine boundaries once.
//! Contract: smith/docs/design/domain/run.md, sections 8.1 and 10.

use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use smith_domain_run::facts::FactKind;
use smith_domain_run::outcome::{Change, Declared, Field};
use smith_domain_run::{self as run, Ask, End, Event, Exit, Request, Returned};
use smith_run_world::{Settings, host};

const CALL: Token = Token::new(7);
const PEER: Token = Token::new(100);
const DEADLINE: Duration = Duration::from_secs(60);

struct PendingCheck {
    domain: run::Domain,
    env: Env<run::Limits>,
    facts: Vec<run::facts::Fact>,
    checks_ended: u32,
    conversation: Token,
    owner: Token,
}

impl PendingCheck {
    fn new() -> Self {
        let settings = Settings::calm(827);
        let script = host::Script {
            jobs: 1,
            window: Duration::ZERO,
            writable: 1000,
            changes: 1000,
            verdicts: 0,
            ..settings.host
        };
        let mut host = host::Host::new(script, settings.seed);
        let mut starts = Vec::new();
        host.fire(Time::ZERO, &mut starts);
        assert_eq!(starts.len(), 1);
        let mut check = Self {
            domain: run::Domain::new(&settings.run, settings.seed),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: settings.run },
            facts: Vec::new(),
            checks_ended: 0,
            conversation: Token::new(0),
            owner: Token::new(0),
        };
        let mut requests = check.step(starts.remove(0));
        while !requests.is_empty() {
            let mut next = Vec::new();
            for request in requests {
                match request {
                    Request::Admitted { .. } => {}
                    Request::Read { owner, .. } => {
                        next.extend(check.step(Event::Read { owner, read: run::Read::Missing }));
                    }
                    Request::Probe { owner, .. } => next.extend(check.step(Event::Probed { owner, executable: true })),
                    Request::Open { conversation, .. } => {
                        check.conversation = conversation;
                        assert!(check.step(Event::Started { conversation, peer: PEER }).is_empty());
                    }
                    other => panic!("unexpected preparation request: {other:?}"),
                }
            }
            requests = next;
        }
        let outcome = Declared::Change(Change {
            fields: Box::new([
                Field { name: b"title".as_slice().into(), value: b"Fix parser".as_slice().into() },
                Field { name: b"body".as_slice().into(), value: b"Run checks".as_slice().into() },
            ]),
        });
        let requests = check.step(Event::Delegated {
            conversation: check.conversation,
            name: run::CallName { activation: 1, completion: 1, position: 0 },
            call: CALL,
            ask: Ask::Finish { outcome },
            deadline: Time::ZERO.saturating_add(DEADLINE),
        });
        let [Request::Check { owner, .. }, Request::Checking { .. }] = requests.as_slice() else {
            panic!("one check starts: {requests:?}");
        };
        check.owner = *owner;
        check
    }

    fn step(&mut self, event: Event) -> Vec<Request> {
        let mut requests = Vec::new();
        let mut events = vec![event];
        let mut out = Queue::uncapped();
        while let Some(event) = events.pop() {
            run::step(&mut self.domain, &self.env, event, &mut out);
            self.drain(&mut out, &mut events, &mut requests);
        }
        requests
    }

    fn drain(&mut self, out: &mut Queue<Request>, events: &mut Vec<Event>, requests: &mut Vec<Request>) {
        while let Some(fact) = self.domain.pop_fact() {
            self.facts.push(fact);
        }
        while let Some(request) = out.pop() {
            match request {
                Request::CancelJobs { peer } => {
                    assert_eq!(peer, PEER);
                    events.push(Event::JobsCancelled { conversation: self.conversation });
                }
                Request::ChecksEnded { .. } => self.checks_ended += 1,
                other => requests.push(other),
            }
        }
    }

    fn assert_terminal(&self, expected: Exit) {
        let checks: Vec<_> = self
            .facts
            .iter()
            .filter_map(|fact| match fact.kind {
                FactKind::CheckStarted { run, .. } => Some((run, None)),
                FactKind::CheckFinished { run, exit } => Some((run, Some(exit))),
                _ => None,
            })
            .collect();
        let [(started, None), (finished, Some(exit))] = checks.as_slice() else {
            panic!("one start owes one terminal: {checks:?}");
        };
        assert_eq!(started, finished);
        assert_eq!(*exit, expected);
        assert_eq!(self.checks_ended, 1);
    }

    fn assert_settled(&mut self) {
        self.facts.clear();
        assert!(self.step(Event::Withdraw { conversation: self.conversation, call: CALL }).is_empty());
        assert!(self.facts.is_empty(), "settled withdrawal cannot make another terminal");
        let closes = self.step(Event::Cancel { run: run::owner(&self.domain, self.conversation).expect("live run") });
        assert_eq!(closes.as_slice(), [Request::Close { peer: PEER }]);
        self.step(Event::Ended {
            conversation: self.conversation,
            end: End::Closed,
            spend: run::Spend::ZERO,
            detail: Box::default(),
        });
        self.domain.reclaim();
        assert_eq!((self.domain.runs(), self.domain.conversations(), self.domain.calls()), (0, 0, 0));
    }
}

#[test]
fn withdrawn_checks_tell_one_cancelled_terminal_before_or_after_spawn() {
    for after_spawn in [false, true] {
        let mut check = PendingCheck::new();
        let requests = check.step(Event::Withdraw { conversation: check.conversation, call: CALL });
        assert_eq!(requests.as_slice(), [Request::Abort { owner: check.owner }]);
        let mut machine = smith_machine_world::World::new(827, &[]);
        let owner = machine.aborted_check(check.owner, after_spawn);
        assert_eq!(
            check.step(Event::Aborted { owner }).as_slice(),
            [Request::Return { spent: 0, call: CALL, result: Returned::Cancelled }]
        );
        check.assert_terminal(Exit::Cancelled);
        check.assert_settled();
        machine.settle();
    }
}

#[test]
fn expired_checks_tell_one_timed_out_terminal_before_or_after_spawn() {
    for after_spawn in [false, true] {
        let mut check = PendingCheck::new();
        check.env.now = Time::ZERO.saturating_add(DEADLINE);
        let mut out = Queue::uncapped();
        run::fire(&mut check.domain, &check.env, &mut out);
        let mut requests = Vec::new();
        check.drain(&mut out, &mut Vec::new(), &mut requests);
        assert_eq!(requests.as_slice(), [Request::Abort { owner: check.owner }]);
        assert!(!check.domain.is_due(check.env.now), "call deadline fires once");
        let mut machine = smith_machine_world::World::new(828, &[]);
        let owner = machine.aborted_check(check.owner, after_spawn);
        assert_eq!(
            check.step(Event::Aborted { owner }).as_slice(),
            [Request::Return { spent: 0, call: CALL, result: Returned::TimedOut }]
        );
        check.assert_terminal(Exit::TimedOut);
        check.assert_settled();
        machine.settle();
    }
}
