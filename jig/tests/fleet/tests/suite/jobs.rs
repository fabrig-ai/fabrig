//! Job hosts in the fleet's scripted world (jig/docs/design/domain/engine/executors.md, 5).
use jig_domain_fleet::{Answer, Call, Domain, Event, HelloRefusal, Limits, Message, Request, Undelivered};
use jig_domain_link::{Form, Grant, Hello, Hosting, JobBound, JobKind, Phase};
use jig_fleet_world::{Settings, World, fixtures};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
const LIMITS: Limits = Limits { grace: Duration::from_secs(10), ..jig_fleet_world::LIMITS };
const C1: Token = Token::new(101);
const C2: Token = Token::new(102);
const C3: Token = Token::new(103);
const R1: Token = Token::new(1);
const R2: Token = Token::new(2);
const A1: Token = Token::new(11);
const A2: Token = Token::new(12);
fn to(attempt: Token) -> ReplyTo {
    ReplyTo::new(attempt)
}
fn payload(value: u64) -> Token {
    Token::new(value + 1000)
}
struct Harness {
    domain: Domain,
    env: Env<Limits>,
}
impl Harness {
    fn new(limits: Limits) -> Self {
        Self {
            domain: fixtures::domain(&limits, fixtures::needs()),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        }
    }
    fn step(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::uncapped();
        jig_domain_fleet::step(&mut self.domain, &self.env, event, &mut out);
        self.drain(&mut out)
    }
    fn drain(&mut self, out: &mut Queue<Request>) -> Vec<Request> {
        let mut result = Vec::new();
        while let Some(request) = out.pop() {
            if let Request::Sources { channel, .. } = request {
                jig_domain_fleet::step(
                    &mut self.domain,
                    &self.env,
                    Event::Logins { channel, logins: Box::new([]) },
                    out,
                );
            } else {
                result.push(request);
            }
        }
        self.domain.reclaim();
        result
    }
    fn settle(&mut self) -> Vec<Request> {
        let mut out = Queue::uncapped();
        for _ in 0..100 {
            if !self.domain.is_ready() {
                break;
            }
            jig_domain_fleet::resume(&mut self.domain, &self.env, true, &mut out);
        }
        assert!(!self.domain.is_ready());
        self.drain(&mut out)
    }
    fn at(&mut self, secs: u64) -> Vec<Request> {
        self.env.now = Time::ZERO.saturating_add(Duration::from_secs(secs));
        let mut out = Queue::uncapped();
        for _ in 0..100 {
            if !self.domain.is_due(self.env.now) {
                break;
            }
            jig_domain_fleet::fire(&mut self.domain, &self.env, &mut out);
        }
        assert!(!self.domain.is_due(self.env.now));
        self.drain(&mut out)
    }
}

use skein_lib::Duration;

/// jig/docs/design/domain/engine/executors.md, 5: jobs finish and acknowledge without agent traffic.
#[test]
fn jobs_settle_in_the_workers_world_without_messages_relays_or_grants() {
    for seed in 0..5 {
        let settings = Settings {
            job: Some(JobBound { kind: JobKind(7), spec: 4, result: 8, deadline: Duration::from_secs(3) }),
            parks: 0,
            inbound: 0,
            ..Settings::calm(seed)
        };
        let mut world = World::new(settings);
        world.run(400_000);
        let stats = world.stats();
        assert!(stats.admitted > 0 && stats.assigned > 0, "{stats:?}");
        assert_eq!((stats.calls, stats.replies, stats.delivered, world.grants_sent()), (0, 0, 0, 0));
        assert_eq!(
            stats.starts,
            stats.endings.get("ended").copied().unwrap_or(0) + stats.endings.get("failed").copied().unwrap_or(0)
        );
    }
}

const JOB: JobBound = JobBound { kind: JobKind(7), spec: 4, result: 8, deadline: Duration::from_secs(3) };

fn job(run: Token, attempt: Token, bound: JobBound, adopt: bool) -> Event {
    Event::StartJob { reply_to: to(attempt), run, attempt, workstream: 1, job: bound, adopt }
}

fn worker(h: &mut Harness, channel: Token, jobs: Box<[JobBound]>) {
    let mut hello = Hello {
        slots: 2,
        stop_bound: Duration::ZERO,
        bounds: fixtures::bounds(),
        jobs: Box::new([]),
        workstreams: Box::new([]),
        hosting: Box::new([]),
    };
    hello.jobs = jobs;
    assert_eq!(
        h.step(Event::Hello {
            channel,
            worker: u32::try_from(channel.raw() - 101).expect("fixture worker position"),
            hello
        })
        .len(),
        0
    );
}

/// jig/docs/design/domain/engine/executors.md, 5: placement depends only on job bounds and room.
#[test]
fn a_job_is_placed_only_on_a_worker_whose_hello_declares_its_kind() {
    let mut h = Harness::new(LIMITS);
    worker(&mut h, C1, Box::new([]));
    worker(&mut h, C2, Box::new([JOB]));
    assert_eq!(h.step(job(R1, A1, JOB, false)).len(), 0);
    assert_eq!(
        h.settle(),
        &[Request::AssignJob { channel: C2, run: R1, attempt: A1 }, Request::Placed { run: R1, attempt: A1 },]
    );
    let other = JobBound { kind: JobKind(9), ..JOB };
    h.step(job(R2, A2, other, false));
    assert_eq!(h.settle().len(), 0);
    worker(&mut h, C3, Box::new([other]));
    assert_eq!(
        h.settle(),
        &[Request::AssignJob { channel: C3, run: R2, attempt: A2 }, Request::Placed { run: R2, attempt: A2 },]
    );
}

/// jig/docs/design/domain/engine/executors.md, 5: every bound is inclusive.
#[test]
fn a_job_past_a_workers_bounds_for_its_kind_is_not_placed_there() {
    for bound in [
        JobBound { spec: 5, ..JOB },
        JobBound { result: 9, ..JOB },
        JobBound { deadline: Duration::from_nanos(3_000_000_001), ..JOB },
    ] {
        let mut h = Harness::new(LIMITS);
        worker(&mut h, C1, Box::new([JOB]));
        h.step(job(R1, A1, bound, false));
        assert_eq!(h.settle().len(), 0);
    }
}

/// jig/docs/design/domain/engine/executors.md, 5; jig/docs/design/protocol/channel/README.md, 4.3.
#[test]
fn a_hello_with_job_kinds_out_of_order_is_not_admitted() {
    for jobs in [Box::new([JobBound { kind: JobKind(9), ..JOB }, JOB]), Box::new([JOB, JOB])] {
        let mut h = Harness::new(LIMITS);
        let mut hello = Hello {
            slots: 2,
            stop_bound: Duration::ZERO,
            bounds: fixtures::bounds(),
            jobs: Box::new([]),
            workstreams: Box::new([]),
            hosting: Box::new([]),
        };
        hello.jobs = jobs;
        assert!(matches!(
            h.step(Event::Hello { channel: C1, worker: 0, hello }).as_slice(),
            [Request::Refuse { channel: C1, .. }]
        ));
        assert_eq!(h.domain.hello_refusals().get(0).expect("refused hello history").why, HelloRefusal::JobKinds);
    }
}

/// jig/docs/design/domain/engine/executors.md, 5: jobs receive no messages, turns, relay or renewed grants.
#[test]
fn a_worker_never_gets_a_message_or_relay_for_a_job() {
    let mut h = Harness::new(LIMITS);
    worker(&mut h, C1, Box::new([JOB]));
    h.step(job(R1, A1, JOB, false));
    h.settle();
    let message = Message { name: Token::new(1), sender: Token::new(2), words: payload(3) };
    assert_eq!(
        h.step(Event::Inbound { run: R1, attempt: A1, message }),
        &[Request::Undelivered { run: R1, attempt: A1, message, undelivered: Undelivered::Gone }]
    );
    let call =
        Call { name: Box::new([]), tool: Box::new([]), writes: false, input: Box::new([]), deadline: Duration::ZERO };
    assert_eq!(
        h.step(Event::Relay { channel: C1, run: R1, attempt: A1, call }),
        &[Request::DropCall {
            call: Call {
                name: Box::new([]),
                tool: Box::new([]),
                writes: false,
                input: Box::new([]),
                deadline: Duration::ZERO
            }
        }]
    );
    assert_eq!(
        h.step(Event::Turn { channel: C1, run: R1, attempt: A1, turn: 1, body: payload(4) }),
        &[Request::Drop { payload: payload(4) }]
    );
    assert_eq!(
        h.step(Event::Grant {
            grant: Grant { form: Form::Application, account: 1, generation: 2, valid: Duration::from_secs(10) }
        })
        .len(),
        0
    );
}

/// jig/docs/design/domain/engine/executors.md, 5: a restored claim finds its worker with the run's adoption machinery.
#[test]
fn an_engine_restart_adopts_a_job_its_worker_still_hosts() {
    for listed_first in [false, true] {
        let mut h = Harness::new(LIMITS);
        if !listed_first {
            assert_eq!(h.step(job(R1, A1, JOB, true)).len(), 0);
        }
        let hello = Hello {
            slots: 2,
            bounds: fixtures::bounds(),
            jobs: Box::new([JOB]),
            stop_bound: Duration::ZERO,
            workstreams: Box::new([]),
            hosting: Box::new([Hosting { run: R1, attempt: A1, phase: Phase::Active }]),
        };
        let listed = h.step(Event::Hello { channel: C1, worker: 0, hello });
        if listed_first {
            assert_eq!(listed, &[Request::Listed { run: R1, attempt: A1 }]);
            assert_eq!(h.step(job(R1, A1, JOB, true)), &[Request::Placed { run: R1, attempt: A1 }]);
        } else {
            assert_eq!(listed, &[Request::Placed { run: R1, attempt: A1 }]);
        }
        assert_eq!(h.at(11).len(), 0);
        assert_eq!(
            h.step(Event::Answer { channel: C1, run: R1, attempt: A1, answer: Answer::Ended, payload: payload(5) }),
            &[Request::Answered { to: to(A1), run: R1, attempt: A1, answer: Answer::Ended, payload: payload(5) }]
        );
        h.step(Event::Acknowledge { run: R1, attempt: A1 });
        assert_eq!(h.domain.attempts(), 0);
    }
}

/// jig/docs/design/domain/engine/executors.md, 5: an unlisted job claim is lost past the adoption grace.
#[test]
fn an_engine_restart_loses_a_job_past_the_grace() {
    let mut h = Harness::new(LIMITS);
    h.step(job(R1, A1, JOB, true));
    h.step(Event::Loaded);
    assert_eq!(h.at(9).len(), 0);
    assert_eq!(h.at(10), &[Request::Lost { to: to(A1), run: R1, attempt: A1 }]);
    assert_eq!(h.domain.attempts(), 0);
}

/// jig/docs/design/domain/engine/executors.md, 5: the scripted world's restart restores jobs through adoption.
#[test]
fn jobs_are_adopted_after_a_restart_in_the_workers_world() {
    let mut adopted = 0;
    let mut found = 0;
    for seed in 0..5 {
        let settings = Settings {
            job: Some(JOB),
            restarts: 1,
            run: jig_fleet_world::Span::millis(5000, 20_000),
            parks: 0,
            inbound: 0,
            ..Settings::calm(seed)
        };
        let mut world = World::new(settings);
        world.run(400_000);
        let stats = world.stats();
        assert_eq!(stats.restarts, 1);
        assert_eq!((stats.calls, stats.replies, stats.delivered, world.grants_sent()), (0, 0, 0, 0));
        adopted += stats.adoptions;
        found += stats.endings.get("found").copied().unwrap_or(0);
    }
    assert!(adopted > 0 && found > 0, "the workers hosted jobs across the restart");
}
