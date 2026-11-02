//! Job placement and shared claims (jig/docs/design/domain/engine.md, 7.5).
use super::*;
use jig_domain_link::{JobBound, JobKind};

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
    assert!(
        h.step(Event::Hello {
            channel,
            worker: u32::try_from(channel.raw().checked_sub(101).expect("fixture worker channel")).unwrap(),
            hello
        })
        .is_empty()
    );
}

/// jig/docs/design/domain/engine.md, 7.5: busy releases its slot and retries the same fenced attempt.
#[test]
fn a_busy_job_is_placed_again_and_its_answer_is_acknowledged() {
    let mut h = Harness::new(LIMITS);
    worker(&mut h, C1, Box::new([JOB]));
    worker(&mut h, C2, Box::new([JOB]));
    h.step(job(R1, A1, JOB, false));
    assert!(h.settle().contains(&Request::AssignJob { channel: C1, run: R1, attempt: A1 }));
    assert_eq!(
        &*h.step(Event::Answer { channel: C1, run: R1, attempt: A1, answer: Answer::Busy, payload: payload(1) }),
        &[Request::Drop { payload: payload(1) }]
    );
    assert!(h.settle().contains(&Request::AssignJob { channel: C2, run: R1, attempt: A1 }));
    let result =
        h.step(Event::Answer { channel: C2, run: R1, attempt: A1, answer: Answer::Ended, payload: payload(2) });
    assert_eq!(
        &*result,
        &[Request::Answered { to: to(A1), run: R1, attempt: A1, answer: Answer::Ended, payload: payload(2) }]
    );
    assert_eq!(
        &*h.step(Event::Acknowledge { run: R1, attempt: A1 }),
        &[Request::Acknowledge { channel: C2, run: R1, attempt: A1 }]
    );
}

/// jig/docs/design/domain/engine.md, 7.5: affinity and capacity are shared with runs.
#[test]
fn a_job_prefers_its_workstream_and_waits_for_a_free_slot() {
    let mut h = Harness::new(LIMITS);
    worker(&mut h, C1, Box::new([JOB]));
    let hello = Hello {
        slots: 1,
        bounds: fixtures::bounds(),
        jobs: Box::new([JOB]),
        stop_bound: Duration::ZERO,
        workstreams: Box::new([1]),
        hosting: Box::new([]),
    };
    h.step(Event::Hello { channel: C2, worker: 1, hello });
    h.step(job(R1, A1, JOB, false));
    assert!(h.settle().contains(&Request::AssignJob { channel: C2, run: R1, attempt: A1 }));
    h.step(job(R2, A2, JOB, false));
    assert!(h.settle().contains(&Request::AssignJob { channel: C1, run: R2, attempt: A2 }));
}

/// jig/docs/design/domain/engine.md, 7.5: stray traffic is dropped once a claim identifies a job.
#[test]
fn adopting_a_job_drops_a_turn_retained_before_its_claim() {
    let mut h = Harness::new(Limits { turns: 2, ..LIMITS });
    let hello = Hello {
        slots: 1,
        bounds: fixtures::bounds(),
        jobs: Box::new([JOB]),
        stop_bound: Duration::ZERO,
        workstreams: Box::new([]),
        hosting: Box::new([Hosting { run: R1, attempt: A1, phase: Phase::Active }]),
    };
    h.step(Event::Hello { channel: C1, worker: 0, hello });
    assert!(h.step(Event::Turn { channel: C1, run: R1, attempt: A1, turn: 1, body: payload(6) }).is_empty());
    h.step(job(R1, A1, JOB, true));
    assert_eq!(&*h.settle(), &[Request::Drop { payload: payload(6) }]);
}
/// jig/docs/design/domain/engine.md, 7.5: a job does not wait on a run's login inventory.
#[test]
fn a_job_needs_no_run_models_or_login_inventory() {
    let mut h = Harness::new(LIMITS);
    let hello = Hello {
        slots: 1,
        bounds: fixtures::bounds(),
        jobs: Box::new([JOB]),
        stop_bound: Duration::ZERO,
        workstreams: Box::new([]),
        hosting: Box::new([]),
    };
    step(&mut h.domain, &h.env, Event::Hello { channel: C1, worker: 0, hello }, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::Sources { channel: C1, sources: Box::new([]) }));
    assert!(h.out.is_empty());
    h.step(job(R1, A1, JOB, false));
    assert!(h.settle().contains(&Request::AssignJob { channel: C1, run: R1, attempt: A1 }));
}
