//! Job placement and shared claims (jig/docs/design/domain/engine/executors.md, 5).
use super::*;
use jig_domain_link::{JobBound, JobKind};

const JOB: JobBound = JobBound { kind: JobKind(7), spec: 4, result: 8, deadline: Duration::from_secs(3) };

fn job(run: Token, attempt: Token, bound: JobBound, adopt: bool) -> Event {
    Event::StartJob { reply_to: to(attempt), run, attempt, workstream: 1, job: bound, adopt }
}

fn worker(h: &mut Harness, channel: Token, jobs: Box<[JobBound]>) {
    let worker = u32::try_from(channel.raw().checked_sub(101).expect("fixture worker channel")).unwrap();
    greet(h, channel, worker, jobs);
}

fn greet(h: &mut Harness, channel: Token, worker: u32, jobs: Box<[JobBound]>) {
    let hello = Hello {
        fresh: false,
        reads_usage: true,
        slots: 2,
        stop_bound: Duration::ZERO,
        bounds: fixtures::bounds(),
        jobs,
        workstreams: Box::new([]),
        hosting: Box::new([]),
    };
    assert!(h.step(Event::Hello { channel, worker, hello }).is_empty());
}

/// jig/docs/design/domain/engine/executors.md, 5: busy releases its slot and retries the same fenced attempt.
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
        &[Request::Answered {
            to: to(A1),
            run: R1,
            attempt: A1,
            answer: Answer::Ended,
            payload: payload(2),
            worker: Some(1)
        }]
    );
    assert_eq!(
        &*h.step(Event::Acknowledge { run: R1, attempt: A1 }),
        &[Request::Acknowledge { channel: C2, run: R1, attempt: A1 }]
    );
}

/// jig/docs/design/domain/engine/executors.md, 5: affinity and capacity are shared with runs.
#[test]
fn a_job_prefers_its_workstream_and_waits_for_a_free_slot() {
    let mut h = Harness::new(LIMITS);
    worker(&mut h, C1, Box::new([JOB]));
    let hello = Hello {
        fresh: false,
        reads_usage: true,
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

/// jig/docs/design/domain/engine/executors.md, 5: stray traffic is dropped once a claim identifies a job.
#[test]
fn adopting_a_job_drops_a_turn_retained_before_its_claim() {
    let mut h = Harness::new(Limits { turns: 2, ..LIMITS });
    let hello = Hello {
        fresh: false,
        reads_usage: true,
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
/// jig/docs/design/domain/engine/executors.md, 5: a job does not wait on a run's login inventory.
#[test]
fn a_job_needs_no_run_models_or_login_inventory() {
    let mut h = Harness::new(LIMITS);
    let hello = Hello {
        fresh: false,
        reads_usage: true,
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

#[test]
fn a_disk_pause_and_crossed_busy_recover_the_same_job_attempt() {
    use jig_domain_link::PauseState;
    for drained in [false, true] {
        let mut h = Harness::new(LIMITS);
        worker(&mut h, C1, Box::new([JOB]));
        h.step(job(R1, A1, JOB, false));
        let assignment = Request::AssignJob { channel: C1, run: R1, attempt: A1 };
        assert!(h.settle().contains(&assignment));
        h.step(Event::Paused { channel: C1, state: PauseState::Paused });
        h.step(Event::Answer { channel: C1, run: R1, attempt: A1, answer: Answer::Busy, payload: payload(1) });
        assert!(h.settle().is_empty());
        if drained {
            h.step(Event::Paused { channel: C1, state: PauseState::Drained });
            assert!(h.settle().is_empty());
        }
        h.step(Event::Paused { channel: C1, state: PauseState::Taking });
        assert!(h.settle().contains(&assignment));
    }
}

/// Judge declarations in kind, spec, result, deadline order (jig/docs/design/domain/engine/fleet.md, 2).
#[test]
fn job_short_reports_the_first_short_bound_in_its_own_unit() {
    use crate::{Figures, JobShort, JobShortfall, job_short};
    let required = JobBound { kind: JobKind(4), spec: 16_384, result: 512, deadline: Duration::from_secs(1800) };
    let mut offered = JobBound { kind: JobKind(4), spec: 1024, result: 256, deadline: Duration::from_secs(600) };
    let other = JobBound { kind: JobKind(3), ..required };
    assert_eq!(job_short(&[other], required), Some(JobShortfall { why: JobShort::Kind, figures: None }));
    for (why, figures) in [
        (JobShort::Spec, Figures { required: 16_384, offered: 1024 }),
        (JobShort::Result, Figures { required: 512, offered: 256 }),
        (JobShort::Deadline, Figures { required: 1_800_000, offered: 600_000 }),
    ] {
        assert_eq!(job_short(&[other, offered], required), Some(JobShortfall { why, figures: Some(figures) }));
        match why {
            JobShort::Spec => offered.spec = required.spec,
            JobShort::Result => offered.result = required.result,
            JobShort::Deadline => offered.deadline = required.deadline,
            JobShort::Kind => unreachable!(),
        }
    }
    assert_eq!(job_short(&[offered], required), None);
    assert_eq!(job_short(&[JobBound { deadline: Duration::from_secs(1801), ..offered }], required), None);
}

#[test]
fn job_refusals_follow_worker_settings_before_the_engine_not_channel_tokens() {
    use crate::{JobRefusal, JobShort, JobShortfall, job_short};
    let engine = Token::new(0);
    let mut h = Harness::new(Limits { engine_slots: 2, ..LIMITS });
    assert_eq!(h.domain.job_refusal(R1, JOB), None, "engine not yet admitted");
    greet(&mut h, engine, 0, Box::new([]));
    let offered = JobBound { deadline: Duration::from_secs(2), ..JOB };
    greet(&mut h, C1, 1, Box::new([]));
    greet(&mut h, C2, 0, Box::new([offered]));
    assert_eq!(h.domain.job_refusal(R1, JOB), Some(JobRefusal { worker: Some(1), short: job_short(&[offered], JOB) }));
    let missing = Some(JobShortfall { why: JobShort::Kind, figures: None });
    h.step(Event::Paused { channel: C2, state: jig_domain_link::PauseState::Paused });
    assert_eq!(h.domain.job_refusal(R1, JOB), Some(JobRefusal { worker: Some(2), short: missing }));
    h.step(Event::Draining { channel: C1 });
    assert_eq!(h.domain.job_refusal(R1, JOB), Some(JobRefusal { worker: None, short: missing }));
}
