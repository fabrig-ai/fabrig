//! Job stories through the unit harness (jig/docs/design/domain/hosts/hub.md, 9).
use super::*;
use crate::{FromJob, JobAssignment, JobBound, JobEnd, JobEnding, JobFailure, JobInvalid, JobKind, JobRefusal, ToJob};
use skein_lib::Duration;

const KIND: JobKind = JobKind(7);
const BOUND: JobBound = JobBound { kind: KIND, spec: 4, result: 1, deadline: Duration::from_secs(10) };
const RUN: Token = Token::new(7);
const ATTEMPT: Token = Token::new(70);
const SPACE: Token = Token::new(700);

#[test]
fn job_refused_answers_a_run_and_a_stop() {
    for (job, answer) in [
        (
            ToJob::Run { owner: RUN, kind: KIND, spec: Box::new([]), deadline: Duration::ZERO, workspace: None },
            FromJob::StartRefused { owner: RUN, why: Box::from(&b"no job capability"[..]) },
        ),
        (ToJob::Stop { owner: ATTEMPT }, FromJob::Gone { owner: ATTEMPT, detail: Box::new([]) }),
    ] {
        assert_eq!(job.refused(), answer);
    }
}

fn job() -> JobAssignment {
    JobAssignment {
        run: RUN,
        attempt: ATTEMPT,
        kind: KIND,
        spec: Box::from(&b"spec"[..]),
        deadline: BOUND.deadline,
        workspace: Some(Workspace { workstream: 7, items: Token::new(71) }),
        grants: Box::new([]),
    }
}
pub(super) fn opened() -> Event {
    Event::Opened {
        stop_bound: Duration::from_secs(1),
        workstreams: Box::new([]),
        bounds: crate::Bounds {
            program_endpoints: Box::new([]),
            endpoints: Box::new([]),
            turns: 1,
            spend: 1,
            time: Duration::from_secs(1),
            waiting: Duration::from_secs(1),
            checks: Duration::from_secs(1),
            message_bytes: 1,
            inbox: 1,
            unacknowledged: 1,
            reply_bytes: None,
        },
        jobs: Box::new([BOUND]),
    }
}
fn harness() -> Harness {
    let mut h = Harness::new();
    h.step(opened());
    h
}
fn assign(h: &mut Harness) -> Token {
    let requests = h.step(Event::Down(LinkDown::AssignJob { job: job() }));
    let [Request::Prepare { owner, .. }] = &*requests else { panic!("prepare: {requests:?}") };
    *owner
}
fn active(h: &mut Harness) -> Token {
    let owner = assign(h);
    assert_eq!(
        &*h.step(Event::Prepared { owner, workspace: SPACE }),
        [Request::Job(ToJob::Run {
            owner,
            kind: KIND,
            spec: Box::from(&b"spec"[..]),
            deadline: BOUND.deadline,
            workspace: Some(SPACE),
        })]
    );
    assert!(h.step(Event::Job(FromJob::Started { owner })).is_empty());
    owner
}
fn finished(owner: Token, delivery: Option<Box<[u8]>>) -> Event {
    Event::Job(FromJob::Ended { owner, end: JobEnd::Finished { result: Box::from(&b"r"[..]), delivery } })
}
fn gone(owner: Token) -> Event {
    Event::Job(FromJob::Gone { owner, detail: Box::new([]) })
}
fn answer(ending: JobEnding) -> Request {
    Request::Up(LinkUp::JobAnswer { run: RUN, attempt: ATTEMPT, answer: ending })
}
fn work() -> Work {
    Work { left: None, saved: None }
}
fn failed(failure: JobFailure) -> JobEnding {
    JobEnding::Failed { failure, detail: Box::new([]), work: work() }
}
fn delivery(owner: Token, outcome: DeliveryOutcome) -> Event {
    Event::Delivered {
        owner,
        delivery: Delivery { outcome, left: SPACE, changed: outcome == DeliveryOutcome::Delivered },
    }
}
fn delivering(h: &mut Harness, owner: Token) -> Token {
    let requests = h.step(finished(owner, Some(Box::from(&b"delivery"[..]))));
    let [Request::DeliverJob { owner, workspace, delivery }] = &*requests else { panic!("deliver: {requests:?}") };
    assert_eq!((*workspace, &**delivery), (SPACE, &b"delivery"[..]));
    *owner
}

/// jig/docs/design/domain/hosts/hub.md, 9: the prepared workspace is the job's, and ACK frees its slot.
#[test]
fn a_job_of_a_declared_kind_runs_in_its_workspace_and_answers_its_result() {
    let mut h = harness();
    let owner = active(&mut h);
    assert!(h.step(finished(owner, None)).is_empty());
    let expected = answer(JobEnding::Finished { result: Box::from(&b"r"[..]), work: work() });
    assert_eq!(&*h.step(gone(owner)), [Request::Release { workspace: SPACE }, expected]);
    assert_eq!(h.domain.kept_answers(), 1);
    h.step(Event::Down(LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }));
    assert_eq!(h.domain.hosting(owner), None);
    h.domain.reclaim();
    assert_eq!(h.domain.hosted(), 0);
}

/// jig/docs/design/domain/hosts/hub.md, 9: invalid assignments never prepare or keep an answer.
#[test]
fn a_job_of_a_kind_not_declared_is_refused_as_invalid() {
    let mut unknown = job();
    unknown.kind = JobKind(8);
    let mut spec = job();
    spec.spec = Box::from(&b"spec!"[..]);
    let mut deadline = job();
    deadline.deadline = Duration::from_secs(11);
    let mut grants = job();
    grants.grants = Box::new([crate::Grant {
        form: crate::Form::Credential(crate::Scheme::Key),
        account: 1,
        generation: 1,
        valid: Duration::from_secs(1),
    }]);
    let mut count = job();
    count.grants = Box::new(
        [crate::Grant { form: crate::Form::Application, account: 1, generation: 1, valid: Duration::from_secs(1) }; 3],
    );
    for (job, invalid) in [
        (unknown, JobInvalid::Kind),
        (spec, JobInvalid::Spec),
        (deadline, JobInvalid::Deadline),
        (grants, JobInvalid::Grants),
        (count, JobInvalid::Grants),
    ] {
        let mut h = harness();
        assert_eq!(
            &*h.step(Event::Down(LinkDown::AssignJob { job })),
            [answer(JobEnding::Refused(JobRefusal::Invalid(invalid)))]
        );
        assert_eq!((h.domain.hosted(), h.domain.kept_answers()), (0, 0));
    }
}

/// jig/docs/design/domain/hosts/hub.md, 9: delivery completes before release, preserving changed work.
#[test]
fn a_finished_job_delivers_through_its_workspace_before_release() {
    let mut h = harness();
    let owner = active(&mut h);
    let call = delivering(&mut h, owner);
    assert!(h.step(gone(owner)).is_empty());
    assert_eq!(
        &*h.step(delivery(call, DeliveryOutcome::Delivered)),
        [
            Request::Release { workspace: SPACE },
            answer(JobEnding::Finished { result: Box::from(&b"r"[..]), work: Work { left: Some(SPACE), saved: None } })
        ]
    );
}

/// jig/docs/design/domain/hosts/hub.md, 9: a stale delivery fails the job.
#[test]
fn a_stale_job_delivery_fails_undelivered() {
    let mut h = harness();
    let owner = active(&mut h);
    let call = delivering(&mut h, owner);
    assert!(h.step(delivery(call, DeliveryOutcome::Stale)).is_empty());
    assert_eq!(
        &*h.step(gone(owner)),
        [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Undelivered(DeliveryOutcome::Stale)))]
    );
}

/// jig/docs/design/domain/hosts/hub.md, 9: cancellation waits for Gone, and ignores a late successful end.
#[test]
fn a_cancelled_job_is_stopped_and_answered_once_its_processes_are_gone() {
    let mut h = harness();
    let owner = active(&mut h);
    assert_eq!(
        &*h.step(Event::Down(LinkDown::Cancel { run: RUN, attempt: ATTEMPT })),
        [Request::Job(ToJob::Stop { owner })]
    );
    assert!(h.step(finished(owner, Some(Box::from(&b"never deliver"[..])))).is_empty());
    assert_eq!(
        &*h.step(gone(owner)),
        [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Cancelled(Reason::Engine)))]
    );
    assert!(h.step(gone(owner)).is_empty());
}

/// jig/docs/design/domain/hosts/hub.md, 9: a begun delivery settles even after cancel and Gone.
#[test]
fn a_cancelled_job_waits_for_its_delivery() {
    for reason in [Reason::Engine, Reason::Contact] {
    let mut h = harness();
    let owner = active(&mut h);
    let call = delivering(&mut h, owner);
    assert!(h.step(Event::CancelAll { reason }).is_empty());
    crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::Job(ToJob::Stop { owner })));
    assert!(h.out.is_empty());
    assert!(h.step(gone(owner)).is_empty());
    assert_eq!(
        &*h.step(delivery(call, DeliveryOutcome::Delivered)),
        [
            Request::Release { workspace: SPACE },
            answer(JobEnding::Failed {
                failure: JobFailure::Cancelled(reason),
                detail: Box::new([]),
                work: Work { left: Some(SPACE), saved: None }
            })
        ]
    );
    }
}

/// jig/docs/design/domain/hosts/hub.md, 9: a capability's deadline has its typed failure.
#[test]
fn a_job_past_its_deadline_fails_deadline() {
    let mut h = harness();
    let owner = active(&mut h);
    assert!(h.step(Event::Job(FromJob::Ended { owner, end: JobEnd::Deadline })).is_empty());
    assert_eq!(&*h.step(gone(owner)), [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Deadline))]);
}

/// jig/docs/design/domain/hosts/hub.md, 9: the kind's result bound is checked before delivery.
#[test]
fn a_result_past_its_kinds_bound_fails_and_delivers_nothing() {
    let mut h = harness();
    let owner = active(&mut h);
    assert!(
        h.step(Event::Job(FromJob::Ended {
            owner,
            end: JobEnd::Finished { result: Box::from(&b"rr"[..]), delivery: Some(Box::from(&b"never deliver"[..])) }
        }))
        .is_empty()
    );
    assert_eq!(&*h.step(gone(owner)), [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Result))]);
}

/// jig/docs/design/domain/hosts/hub.md, 9: every hello lists job kinds and the shared slot's phase.
#[test]
fn the_hello_states_the_kinds_and_lists_a_hosted_job() {
    let mut h = harness();
    let owner = assign(&mut h);
    for (event, phase) in [
        (None, crate::Phase::Preparing),
        (Some(Event::Prepared { owner, workspace: SPACE }), crate::Phase::Starting),
        (Some(Event::Job(FromJob::Started { owner })), crate::Phase::Active),
        (Some(finished(owner, None)), crate::Phase::Ending),
        (Some(gone(owner)), crate::Phase::Answered),
    ] {
        if let Some(event) = event {
            h.step(event);
        }
        let requests = h.step(opened());
        let [Request::Hello { hello }] = &*requests else { panic!("hello: {requests:?}") };
        assert_eq!(&*hello.jobs, [BOUND]);
        assert_eq!(&*hello.hosting, [crate::Hosting { run: RUN, attempt: ATTEMPT, phase }]);
    }
    assert_eq!(
        h.domain.answer_at(0),
        Some(LinkUp::JobAnswer {
            run: RUN,
            attempt: ATTEMPT,
            answer: JobEnding::Finished { result: Box::from(&b"r"[..]), work: work() }
        })
    );
}

/// jig/docs/design/domain/hosts/hub.md, 9: cancellation of preparation waits for its terminal and never runs the job.
#[test]
fn a_job_cancelled_during_preparation_releases_only_after_preparation_settles() {
    for reason in [Reason::Engine, Reason::Contact, Reason::Shutdown] {
        let mut h = harness();
        let owner = assign(&mut h);
        h.step(Event::CancelAll { reason });
        crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
        assert_eq!(h.out.pop(), Some(Request::Abort { owner }));
        assert!(h.out.is_empty());
        assert_eq!(
            &*h.step(Event::Prepared { owner, workspace: SPACE }),
            [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Cancelled(reason)))]
        );
    }
}

/// jig/docs/design/domain/hosts/hub.md, 9: every contact or shutdown cancellation stops a job, preserving the first reason.
#[test]
fn cancel_all_stops_a_starting_job_and_keeps_its_first_reason() {
    for reason in [Reason::Contact, Reason::Shutdown] {
        let mut h = harness();
        let owner = assign(&mut h);
        h.step(Event::Prepared { owner, workspace: SPACE });
        h.step(Event::CancelAll { reason });
        crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
        assert_eq!(h.out.pop(), Some(Request::Job(ToJob::Stop { owner })));
        h.step(Event::Down(LinkDown::Cancel { run: RUN, attempt: ATTEMPT }));
        assert!(h.step(Event::Job(FromJob::Started { owner })).is_empty());
        assert_eq!(
            &*h.step(gone(owner)),
            [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Cancelled(reason)))]
        );
    }
}

/// jig/docs/design/domain/hosts/hub.md, 9: a refused capability start keeps no answer and releases its workspace.
#[test]
fn a_job_start_refused_releases_and_keeps_nothing() {
    let mut h = harness();
    let owner = assign(&mut h);
    h.step(Event::Prepared { owner, workspace: SPACE });
    assert_eq!(
        &*h.step(Event::Job(FromJob::StartRefused { owner, why: Box::from(&b"0123456789"[..]) })),
        [
            Request::Release { workspace: SPACE },
            answer(JobEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why: Box::from(&b"23456789"[..]) })))
        ]
    );
    h.domain.reclaim();
    assert_eq!((h.domain.hosted(), h.domain.kept_answers()), (0, 0));
}

/// jig/docs/design/domain/hosts/hub.md, 9: failed preparation retains a typed, bounded answer without a workspace.
#[test]
fn an_unprepared_job_answers_with_bounded_detail() {
    let mut h = harness();
    let owner = assign(&mut h);
    assert_eq!(
        &*h.step(Event::Unprepared { owner, failure: Preparation::Transient, detail: Box::from(&b"0123456789"[..]) }),
        [answer(JobEnding::Failed {
            failure: JobFailure::Unprepared(Preparation::Transient),
            detail: Box::from(&b"23456789"[..]),
            work: work()
        })]
    );
    assert_eq!(h.domain.kept_answers(), 1);
}

/// jig/docs/design/domain/hosts/hub.md, 9: a job with no items needs no workspace or delivery, and Gone without Ended fails it.
#[test]
fn an_itemless_job_that_goes_without_an_end_fails_unstarted() {
    let mut h = harness();
    let mut job = job();
    job.workspace = None;
    let requests = h.step(Event::Down(LinkDown::AssignJob { job }));
    let [Request::Job(ToJob::Run { owner, workspace: None, .. })] = &*requests else {
        panic!("itemless run: {requests:?}")
    };
    assert_eq!(&*h.step(gone(*owner)), [answer(failed(JobFailure::Unstarted))]);
}

/// jig/docs/design/domain/hosts/hub.md, 9: jobs and runs retain their slots until acknowledged.
#[test]
fn runs_jobs_and_kept_answers_share_the_slot_limit() {
    let mut h = harness();
    h.live(1);
    let owner = active(&mut h);
    h.step(finished(owner, None));
    h.step(gone(owner));
    let mut next = job();
    next.run = Token::new(8);
    assert_eq!(
        &*h.step(Event::Down(LinkDown::AssignJob { job: next })),
        [Request::Up(LinkUp::JobAnswer {
            run: Token::new(8),
            attempt: ATTEMPT,
            answer: JobEnding::Refused(JobRefusal::Busy)
        })]
    );
    h.step(Event::Down(LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }));
    h.domain.reclaim();
    let mut next = job();
    next.run = Token::new(8);
    let requests = h.step(Event::Down(LinkDown::AssignJob { job: next }));
    let [Request::Prepare { .. }] = &*requests else { panic!("prepare: {requests:?}") };
}

/// jig/docs/design/domain/hosts/hub.md, 9: application grants at their count bound are admitted and never given to the job.
#[test]
fn application_grants_at_the_bound_are_admitted() {
    let mut h = harness();
    let mut job = job();
    job.grants = Box::new(
        [crate::Grant { form: crate::Form::Application, account: 1, generation: 1, valid: Duration::from_secs(1) }; 2],
    );
    let requests = h.step(Event::Down(LinkDown::AssignJob { job }));
    let [Request::Prepare { .. }] = &*requests else { panic!("prepare: {requests:?}") };
}

/// jig/docs/design/domain/hosts/hub.md, 9: a job delivery has room even when the host admits no run calls.
#[test]
fn a_job_delivers_when_run_calls_are_disabled() {
    let mut h = Harness::new();
    h.env.limits.run_calls = 0;
    h.domain = Hub::new(&h.env.limits);
    h.step(opened());
    let owner = active(&mut h);
    let call = delivering(&mut h, owner);
    h.step(gone(owner));
    assert_eq!(
        &*h.step(delivery(call, DeliveryOutcome::Nothing)),
        [
            Request::Release { workspace: SPACE },
            answer(JobEnding::Finished { result: Box::from(&b"r"[..]), work: work() })
        ]
    );
}

#[test]
fn a_job_runs_on_through_a_drain_but_shutdown_before_delivery_cancels_it() {
    for cancel in [false, true] {
        let mut h = harness();
        let owner = assign(&mut h);
        assert_eq!(h.step(Event::Drain).as_ref(), [Request::Up(LinkUp::Draining)]);
        crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
        assert!(h.out.is_empty(), "drain leaves job preparation alone");
        h.step(Event::Prepared { owner, workspace: SPACE });
        h.step(Event::Job(FromJob::Started { owner }));
        assert!(h.step(Event::Drain).is_empty());
        if cancel {
            assert!(h.step(Event::CancelAll { reason: Reason::Shutdown }).is_empty());
            crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
            assert_eq!(h.out.pop(), Some(Request::Job(ToJob::Stop { owner })));
            assert!(h.out.is_empty());
            assert!(h.step(finished(owner, Some(Box::new([])))).is_empty(), "no delivery after cancel");
            assert_eq!(h.step(gone(owner)).as_ref(), [Request::Release { workspace: SPACE }, answer(failed(JobFailure::Cancelled(Reason::Shutdown)))]);
        } else {
            let call = delivering(&mut h, owner);
            assert!(h.step(gone(owner)).is_empty());
            assert_eq!(h.step(delivery(call, DeliveryOutcome::Delivered)).as_ref(), [
                Request::Release { workspace: SPACE },
                answer(JobEnding::Finished { result: b"r".as_slice().into(), work: Work { left: Some(SPACE), saved: None } }),
            ]);
        }
    }
}

#[test]
fn shutdown_during_a_job_delivery_keeps_the_deliverys_result_not_cancellation() {
    for (outcome, gone_first) in [
        (DeliveryOutcome::Delivered, true), (DeliveryOutcome::Nothing, false),
        (DeliveryOutcome::Stale, true), (DeliveryOutcome::Refused, false), (DeliveryOutcome::Failed, true),
    ] {
        let mut h = harness();
        let owner = active(&mut h);
        let call = delivering(&mut h, owner);
        h.step(Event::Drain);
        assert!(h.step(Event::CancelAll { reason: Reason::Shutdown }).is_empty());
        crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
        assert_eq!(h.out.pop(), Some(Request::Job(ToJob::Stop { owner })));
        assert!(h.out.is_empty());
        h.step(Event::CancelAll { reason: Reason::Shutdown });
        crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
        assert!(h.out.is_empty(), "no second stop while delivery settles");
        let requests = if gone_first {
            assert!(h.step(gone(owner)).is_empty());
            h.step(delivery(call, outcome))
        } else {
            assert!(h.step(delivery(call, outcome)).is_empty());
            h.step(gone(owner))
        };
        let work = Work { left: (outcome == DeliveryOutcome::Delivered).then_some(SPACE), saved: None };
        let ending = match outcome {
            DeliveryOutcome::Delivered | DeliveryOutcome::Nothing => JobEnding::Finished { result: b"r".as_slice().into(), work },
            DeliveryOutcome::Stale | DeliveryOutcome::Refused | DeliveryOutcome::Failed => JobEnding::Failed {
                failure: JobFailure::Undelivered(outcome), detail: Box::new([]), work,
            },
        };
        assert_eq!(requests.as_ref(), [Request::Release { workspace: SPACE }, answer(ending)]);
    }
}

#[test]
fn shutdown_after_a_job_delivery_settles_does_not_erase_its_effect() {
    for outcome in [DeliveryOutcome::Delivered, DeliveryOutcome::Failed] {
        let mut h = harness();
        let owner = active(&mut h);
        let call = delivering(&mut h, owner);
        assert!(h.step(delivery(call, outcome)).is_empty());
        h.step(Event::CancelAll { reason: Reason::Shutdown });
        crate::resume_runs(&mut h.domain, &h.env, &mut h.out);
        assert_eq!(h.out.pop(), Some(Request::Job(ToJob::Stop { owner })));
        assert!(h.out.is_empty());
        let requests = h.step(gone(owner));
        let ending = if outcome == DeliveryOutcome::Delivered {
            JobEnding::Finished { result: b"r".as_slice().into(), work: Work { left: Some(SPACE), saved: None } }
        } else {
            failed(JobFailure::Undelivered(outcome))
        };
        assert_eq!(requests.as_ref(), [Request::Release { workspace: SPACE }, answer(ending)]);
    }
}
