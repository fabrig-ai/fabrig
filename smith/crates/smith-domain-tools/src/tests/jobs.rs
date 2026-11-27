//! Job cell admission and release, and content-free observations of job outcomes.

use super::{ALL, Harness, authority, bytes};
use crate::{
    BytesClamp, Done, Event, Exit, FactKind, JobEnd, JobNumber, JobProblem, Outcome, Request, Timeout, TimeoutClamp,
    Tool, Verdict, Yield,
};
use skein_lib::{Duration, ReplyTo, Token};

fn observe(end: JobEnd) -> FactKind {
    crate::facts::answered(
        Token::new(1),
        Tool::Shell,
        &Outcome::Job {
            job: JobNumber(2),
            head: b"abc".as_slice().into(),
            tail: b"de".as_slice().into(),
            dropped: 7,
            output: BytesClamp::Clamped { asked: 90, ceiling: 5 },
            wait: TimeoutClamp::Within,
            end,
        },
        crate::facts::Observation { omitted_bytes: 0, clamped: false },
    )
}

#[test]
fn a_running_commands_fact_keeps_its_loss_and_clamp() {
    let outcome = Outcome::Running {
        timeout: TimeoutClamp::Within,
        job: JobNumber(2),
        head: b"abc".as_slice().into(),
        tail: b"de".as_slice().into(),
        dropped: 7,
        output: BytesClamp::Within,
        after: TimeoutClamp::Clamped { asked: Duration::from_secs(90), ceiling: Duration::from_secs(30) },
    };
    assert_eq!(
        crate::facts::answered(
            Token::new(1),
            Tool::Shell,
            &outcome,
            crate::facts::Observation { omitted_bytes: 0, clamped: false }
        ),
        FactKind::Answered {
            session: Token::new(1),
            tool: Tool::Shell,
            verdict: Verdict::Running,
            bytes: 5,
            omitted_bytes: 7,
            job: Some(JobNumber(2)),
            clamped: true
        }
    );
}

#[test]
fn a_running_jobs_fact_keeps_its_loss_and_clamp() {
    assert_eq!(
        observe(JobEnd::Running),
        FactKind::Answered {
            session: Token::new(1),
            tool: Tool::Shell,
            verdict: Verdict::Running,
            bytes: 5,
            omitted_bytes: 7,
            job: Some(JobNumber(2)),
            clamped: true
        }
    );
}

#[test]
fn an_exited_jobs_fact_keeps_its_observed_exit() {
    assert_eq!(
        observe(JobEnd::Ended(Exit::Code { code: 17 })),
        FactKind::Answered {
            session: Token::new(1),
            tool: Tool::Shell,
            verdict: Verdict::Exited { exit: Exit::Code { code: 17 } },
            bytes: 5,
            omitted_bytes: 7,
            job: Some(JobNumber(2)),
            clamped: true
        }
    );
}

#[test]
fn a_stopped_jobs_fact_keeps_its_loss_and_clamp() {
    assert_eq!(
        observe(JobEnd::Stopped),
        FactKind::Answered {
            session: Token::new(1),
            tool: Tool::Shell,
            verdict: Verdict::Stopped,
            bytes: 5,
            omitted_bytes: 7,
            job: Some(JobNumber(2)),
            clamped: true
        }
    );
}

#[test]
fn a_cancelled_jobs_fact_keeps_its_loss_and_clamp() {
    assert_eq!(
        observe(JobEnd::Cancelled),
        FactKind::Answered {
            session: Token::new(1),
            tool: Tool::Shell,
            verdict: Verdict::Cancelled,
            bytes: 5,
            omitted_bytes: 7,
            job: Some(JobNumber(2)),
            clamped: true
        }
    );
}

#[test]
fn a_running_commands_fact_preserves_its_deadline_clamp_alone() {
    let outcome = Outcome::Running {
        job: JobNumber(2),
        head: Box::default(),
        tail: Box::default(),
        dropped: 0,
        output: BytesClamp::Within,
        after: TimeoutClamp::Within,
        timeout: TimeoutClamp::Clamped { asked: Duration::from_secs(180), ceiling: Duration::from_secs(120) },
    };
    assert_eq!(
        crate::facts::answered(
            Token::new(1),
            Tool::Shell,
            &outcome,
            crate::facts::Observation { omitted_bytes: 0, clamped: false }
        ),
        FactKind::Answered {
            session: Token::new(1),
            tool: Tool::Shell,
            verdict: Verdict::Running,
            bytes: 0,
            omitted_bytes: 0,
            job: Some(JobNumber(2)),
            clamped: true
        }
    );
}

#[test]
fn parallel_job_reads_reserve_each_calls_capture_budget() {
    let limits = crate::Limits {
        read_bytes: 1,
        tool_payload: 1,
        list_bytes: 1,
        search_hits: 0,
        search_bytes: 0,
        path_bytes: 1,
        ..super::LIMITS
    };
    assert_eq!(crate::batch_result_bound(&limits, 3), Some(576));
}

#[test]
fn a_job_read_has_the_read_effect() {
    let call =
        crate::Call::Job { job: JobNumber(1), wait: crate::Wait::Default, output: crate::Budget::Default, stop: false };
    assert_eq!(crate::effect(&call), crate::Effect::Read);
}

#[test]
fn a_job_stop_has_the_write_effect() {
    let call =
        crate::Call::Job { job: JobNumber(1), wait: crate::Wait::Default, output: crate::Budget::Default, stop: true };
    assert_eq!(crate::effect(&call), crate::Effect::Write);
}

// The last cell and the last call place are exercised independently (tools.md, section 5).
const ONE_JOB: u32 = 1;
const ONE_CALL: u32 = 1;
const TWO_CALLS: u32 = 2;
const SESSION: u64 = 1;
const FIRST_JOB: JobNumber = JobNumber(1);
const NEXT_JOB: JobNumber = JobNumber(2);

fn yielding_shell() -> crate::Call {
    crate::Call::Shell {
        command: bytes(b"cargo build"),
        timeout: Timeout::Default,
        output: crate::Budget::Default,
        after: Yield::Asked(Duration::from_secs(1)),
    }
}

fn spawn(harness: &mut Harness, kit: Token, call: u64) -> Token {
    match harness.send(kit, call, yielding_shell()) {
        Some(Request::Io { owner, .. }) => owner,
        other => panic!("expected the yielding shell to be admitted, got {other:?}"),
    }
}

fn held() -> Done {
    Done::Held { head: Box::default(), tail: Box::default(), dropped: 0 }
}

fn running(job: JobNumber) -> Outcome {
    Outcome::Running {
        job,
        head: Box::default(),
        tail: Box::default(),
        dropped: 0,
        timeout: TimeoutClamp::Within,
        output: BytesClamp::Within,
        after: TimeoutClamp::Within,
    }
}

fn poll(job: JobNumber) -> crate::Call {
    crate::Call::Job { job, wait: crate::Wait::Default, output: crate::Budget::Default, stop: false }
}

fn polled(end: JobEnd) -> Done {
    Done::Job { head: Box::default(), tail: Box::default(), dropped: 0, end }
}

fn with_one_job(calls: u32) -> (Harness, Token) {
    let mut harness = Harness::new(crate::Limits { jobs: ONE_JOB, calls, ..super::LIMITS });
    let kit = harness.open(SESSION, authority(ALL));
    (harness, kit)
}

fn with_running_job(calls: u32) -> (Harness, Token, Token) {
    let (mut harness, kit) = with_one_job(calls);
    let owner = spawn(&mut harness, kit, 1);
    drop(harness.end(owner, held()));
    (harness, kit, owner)
}

#[test]
fn a_pending_yield_reserves_the_last_job_cell() {
    let (mut harness, kit) = with_one_job(TWO_CALLS);
    let owner = spawn(&mut harness, kit, 1);
    assert_eq!(
        harness.send(kit, 2, yielding_shell()),
        Some(Request::Invalid { to: ReplyTo::new(Token::new(2)), problem: JobProblem::Full { jobs: ONE_JOB } })
    );
    assert_eq!(harness.end(owner, held()), (1, running(FIRST_JOB)));
}

#[test]
fn reading_the_end_reopens_the_cell_not_its_number() {
    let (mut harness, kit, owner) = with_running_job(ONE_CALL);
    drop(harness.send(kit, 2, poll(FIRST_JOB)));
    drop(harness.end(owner, polled(JobEnd::Ended(Exit::Code { code: 0 }))));
    let next = spawn(&mut harness, kit, 3);
    assert_eq!(harness.end(next, held()), (3, running(NEXT_JOB)));
}

#[test]
fn exiting_before_yield_returns_the_reserved_job_cell() {
    let (mut harness, kit) = with_one_job(ONE_CALL);
    let owner = spawn(&mut harness, kit, 1);
    let exit = Exit::Code { code: 0 };
    let ended = Done::Exited { exit, head: Box::default(), tail: Box::default(), dropped: 0 };
    assert_eq!(
        harness.end(owner, ended),
        (
            1,
            Outcome::Exited {
                exit,
                head: Box::default(),
                tail: Box::default(),
                dropped: 0,
                timeout: TimeoutClamp::Within,
                output: BytesClamp::Within,
            }
        )
    );
    let next = spawn(&mut harness, kit, 2);
    assert_eq!(harness.end(next, held()), (2, running(FIRST_JOB)));
}

#[test]
fn job_reads_share_the_kits_room_with_other_calls() {
    let (mut harness, kit, _) = with_running_job(TWO_CALLS);
    drop(harness.start(kit, 2, super::read(b"a")));
    drop(harness.send(kit, 3, poll(FIRST_JOB)));
    assert_eq!(
        harness.send(kit, 4, poll(FIRST_JOB)),
        Some(Request::Answer { to: ReplyTo::new(Token::new(4)), outcome: Outcome::Busy })
    );
}

#[test]
fn a_job_reads_place_is_reusable_after_its_answer() {
    let (mut harness, kit, owner) = with_running_job(ONE_CALL);
    drop(harness.send(kit, 2, poll(FIRST_JOB)));
    drop(harness.end(owner, polled(JobEnd::Running)));
    assert_eq!(
        harness.send(kit, 3, poll(FIRST_JOB)),
        Some(Request::Job {
            owner,
            act: crate::JobAct::Read {
                head: super::LIMITS.shell_head,
                tail: super::LIMITS.shell_tail,
                wait: super::LIMITS.job_poll,
            },
        })
    );
}

#[test]
fn a_resumed_kits_first_job_follows_its_saved_highest() {
    let mut harness = Harness::new(super::LIMITS);
    let opened = harness.step(Event::Open { session: Token::new(SESSION), jobs: 3, authority: authority(ALL) });
    let Some(Request::Opened { kit, .. }) = opened else { panic!("kit admitted") };
    let owner = spawn(&mut harness, kit, 1);
    assert_eq!(harness.end(owner, held()), (1, running(JobNumber(4))));
}
