//! Jobs through the root's protocol boundary (temper/docs/design/domain/worker/jobs.md).

use alloc::boxed::Box;
use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, List, Queue, Token, Writer};

use super::{fixtures, harness::Harness};
use crate::checkout::git::{Commit, Done, Op};
use crate::{Domain, Event, Request, host, tooling, wire};

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("test path length");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured test path");
    }
    out.finish()
}

fn pushed(rows: &[Request]) -> bool {
    for row in rows {
        if let Request::Io { op: Op::Push { .. }, .. } = row {
            return true;
        }
    }
    false
}

fn requested(rows: &[Request]) -> bool {
    for row in rows {
        if let Request::Job { .. } = row {
            return true;
        }
    }
    false
}

fn kind(delivers: bool) -> tooling::Kind {
    tooling::Kind {
        bound: JobBound { kind: JobKind(9), spec: 128, result: 128, deadline: Duration::from_secs(10) },
        verb: Box::from(b"check".as_slice()),
        delivers,
    }
}

fn harness(delivers: bool) -> Harness {
    let mut h = Harness::new();
    h.domain = Domain::new(&h.env.limits, Box::new([kind(delivers)]));
    h.step(Event::Ready);
    h
}

fn assignment(decoded: bool, writable: bool) -> Event {
    let mut items = fixtures::items(&[b"home", b"work"]);
    if writable {
        items.repositories[1].access =
            wire::Access::Writable { push: Box::from(b"topic".as_slice()), landing: None, expected: None };
    }
    let job = host::JobAssignment {
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        kind: JobKind(9),
        spec: Box::from(b"spec".as_slice()),
        deadline: Duration::from_secs(5),
        workspace: Some(host::Workspace { workstream: fixtures::WORKSTREAM, items: Token::new(0) }),
        grants: Box::new([]),
    };
    let spec = if decoded {
        Ok(wire::JobSpec {
            command: Box::from(b"home/tools/check".as_slice()),
            request: Box::from(b"request".as_slice()),
        })
    } else {
        Err(wire::Undecodable)
    };
    Event::AssignJob { job, items: Ok(items), spec }
}

/// The scripted git boundary settles only the checkout's operations.
fn git(h: &mut Harness, initial: Box<[Request]>, stale: bool) -> Box<[Request]> {
    let mut pending = Queue::uncapped();
    for row in initial {
        pending.push(row);
    }
    let mut rows = List::with_capacity(128);
    for _ in 0..128_u32 {
        let Some(row) = pending.pop() else { return rows.into_boxed() };
        if let Request::Io { owner, op, .. } = &row {
            let done = match op {
                Op::Fetch { .. } => Done::Fetched { commit: Commit::new([1; 32]) },
                Op::Commit { title, body, at, .. } => {
                    assert_eq!(title.as_ref(), b"job 9");
                    assert!(body.is_empty());
                    assert_eq!(at.repository.as_ref(), b"work");
                    Done::Committed { commit: Commit::new([2; 32]) }
                }
                Op::Push { at, commit, .. } => {
                    assert_eq!(at.repository.as_ref(), b"work");
                    assert_eq!(*commit, Commit::new([2; 32]));
                    if stale { Done::Rejected } else { Done::Succeeded }
                }
                Op::DiffPaths { .. } => panic!("this job scripts no design delivery"),
                Op::Brings { .. } => Done::Brings { brings: false },
                Op::Look { .. } | Op::IsAncestor { .. } => Done::IsAncestor { contains: true },
                Op::Point { .. }
                | Op::Store { .. }
                | Op::Make { .. }
                | Op::Clone { .. }
                | Op::Create { .. }
                | Op::CheckOut { .. } => Done::Succeeded,
                Op::Merge { .. } => Done::Merged,
            };
            for request in h.step(Event::Done { owner: *owner, done }) {
                pending.push(request);
            }
        }
        rows.push(row).expect("bounded checkout script");
    }
    panic!("checkout settles");
}

fn owner(rows: &[Request]) -> Token {
    for row in rows {
        if let Request::Job { owner, op: tooling::Request::Write { bytes, .. } } = row {
            assert_eq!(bytes.as_ref(), b"request");
            return *owner;
        }
    }
    panic!("job writes its request: {rows:?}");
}

fn job(h: &mut Harness, owner: Token, event: tooling::Event) -> Box<[Request]> {
    h.step(Event::Job { owner, event })
}

fn running(h: &mut Harness, writable: bool) -> Token {
    let rows = h.step(assignment(true, writable));
    let rows = git(h, rows, false);
    let owner = owner(&rows);
    let rows = job(h, owner, tooling::Event::Written { owner, written: true });
    let [Request::Job { op: tooling::Request::Spawn { .. }, .. }] = rows.as_ref() else { panic!("job spawns") };
    assert!(job(h, owner, tooling::Event::Spawned { owner }).is_empty());
    assert_eq!(h.domain.hosting().len(), 1);
    owner
}

fn result(h: &mut Harness, owner: Token) -> Box<[Request]> {
    let rows = job(h, owner, tooling::Event::Exited { owner, exit: tooling::Exit::Code(0), tail: Box::new([]) });
    let [Request::Job { op: tooling::Request::Read { .. }, .. }] = rows.as_ref() else {
        panic!("job reads its result")
    };
    job(h, owner, tooling::Event::Read { owner, bytes: Some(Box::from(b"result".as_slice())) })
}

fn answer(rows: &[Request]) -> &host::JobEnding {
    let mut answer = None;
    for row in rows {
        if let Request::JobAnswer { run, attempt, answer: ending, .. } = row {
            assert_eq!((*run, *attempt), (fixtures::RUN, fixtures::ATTEMPT));
            assert!(answer.replace(ending).is_none(), "one job answer");
        }
    }
    answer.expect("job answers")
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn the_hello_states_the_job_capabilitys_kinds() {
    let mut h = harness(false);
    let rows = h.step(Event::Ready);
    let [Request::Hello(hello)] = rows.as_ref() else { panic!("one hello") };
    assert_eq!(hello.jobs.as_ref(), &[kind(false).bound]);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_job_runs_its_command_in_its_prepared_checkout_and_answers_its_result() {
    let mut h = harness(false);
    let rows = h.step(assignment(true, false));
    let rows = git(&mut h, rows, false);
    let owner = owner(&rows);
    let directory = crate::workspace::job_directory(&h.domain, *h.domain.jobs.get(&owner).unwrap()).0;
    let number = skein_lib::Decimal::of(directory.raw());
    let path = joined(&[b"workspaces/".as_slice(), number.as_bytes()]);
    for row in &rows {
        if let Request::Job { op: tooling::Request::Write { path: request, .. }, .. } = row {
            assert_eq!(request.as_ref(), joined(&[path.as_ref(), b"/request.json"]).as_ref());
        }
    }
    let rows = job(&mut h, owner, tooling::Event::Written { owner, written: true });
    let [Request::Job { op: tooling::Request::Spawn { program, dir, args, .. }, .. }] = rows.as_ref() else {
        panic!("spawn after request write: {rows:?}");
    };
    assert_eq!(dir.as_ref(), path.as_ref());
    assert_eq!(program.as_ref(), joined(&[path.as_ref(), b"/home/tools/check"]).as_ref());
    assert_eq!(args[1].as_ref(), b"check");
    job(&mut h, owner, tooling::Event::Spawned { owner });
    assert!(result(&mut h, owner).is_empty(), "the answer waits for Gone");
    assert_eq!(h.domain.workspaces(), 1);
    let rows = job(&mut h, owner, tooling::Event::Gone { owner });
    assert_eq!(
        answer(&rows),
        &host::JobEnding::Finished {
            result: Box::from(b"result".as_slice()),
            work: host::Work { left: None, saved: None },
        }
    );
    assert_eq!(h.domain.workspaces(), 0);
    assert_eq!(h.domain.hosting().len(), 0);
    assert!(h.domain.job_specs.is_empty());
    assert_eq!(h.domain.items.len(), 1);
    let replay = h.step(Event::Ready);
    assert_eq!(answer(&replay), answer(&rows));
    h.step(Event::Link(host::LinkDown::Acknowledge { run: fixtures::RUN, attempt: fixtures::ATTEMPT }));
    assert!(h.domain.items.is_empty());
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_delivering_kinds_job_pushes_its_writable_repositorys_head_before_answering() {
    let mut h = harness(true);
    let owner = running(&mut h, true);
    let rows = result(&mut h, owner);
    let [Request::Io { op: Op::Commit { .. }, .. }] = rows.as_ref() else { panic!("job delivers through commit") };
    assert_eq!(h.domain.kept_answers(), 0);
    let rows = git(&mut h, rows, false);
    assert!(pushed(&rows));
    assert_eq!(h.domain.workspaces(), 1, "Gone still holds the checkout");
    let rows = job(&mut h, owner, tooling::Event::Gone { owner });
    let host::JobEnding::Finished { result, work } = answer(&rows) else { panic!("delivered") };
    assert_eq!(result.as_ref(), b"result");
    assert!(work.left.is_some());
    assert!(work.saved.is_none());
    assert_eq!(h.domain.workspaces(), 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_delivering_jobs_stale_push_answers_failed_undelivered() {
    let mut h = harness(true);
    let owner = running(&mut h, true);
    let rows = result(&mut h, owner);
    git(&mut h, rows, true);
    let rows = job(&mut h, owner, tooling::Event::Gone { owner });
    let host::JobEnding::Failed { failure, work, .. } = answer(&rows) else { panic!("stale delivery fails") };
    assert_eq!(*failure, host::JobFailure::Undelivered(host::DeliveryOutcome::Stale));
    assert_eq!(*work, host::Work { left: None, saved: None });
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_cancelled_job_is_killed_and_answered_once_its_processes_are_gone() {
    let mut h = harness(true);
    let owner = running(&mut h, true);
    let rows = h.step(Event::Link(host::LinkDown::Cancel { run: fixtures::RUN, attempt: fixtures::ATTEMPT }));
    assert_eq!(rows.as_ref(), &[Request::Job { owner, op: tooling::Request::Kill { owner } }]);
    assert!(
        job(&mut h, owner, tooling::Event::Exited { owner, exit: tooling::Exit::Code(0), tail: Box::new([]) })
            .is_empty()
    );
    assert_eq!(h.domain.workspaces(), 1);
    let rows = job(&mut h, owner, tooling::Event::Gone { owner });
    let host::JobEnding::Failed { failure, work, .. } = answer(&rows) else { panic!("cancelled") };
    assert_eq!(*failure, host::JobFailure::Cancelled(host::Reason::Engine));
    assert_eq!(*work, host::Work { left: None, saved: None });
    assert!(job(&mut h, owner, tooling::Event::Gone { owner }).is_empty());
    assert_eq!(h.domain.workspaces(), 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_job_whose_spec_did_not_decode_is_refused_its_start() {
    let mut h = harness(false);
    let rows = h.step(assignment(false, false));
    let rows = git(&mut h, rows, false);
    assert_eq!(
        answer(&rows),
        &host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start {
            why: Box::from(b"job spec".as_slice()),
        }))
    );
    assert!(!requested(&rows));
    assert!(h.domain.job_specs.is_empty());
    assert!(h.domain.items.is_empty());
    assert_eq!(h.domain.workspaces(), 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_job_retry_during_preparation_keeps_the_first_decoded_spec() {
    let mut h = harness(false);
    let preparing = h.step(assignment(true, false));
    assert!(h.step(assignment(false, false)).is_empty());
    let rows = git(&mut h, preparing, false);
    assert!(requested(&rows), "the first, decoded spec starts");
    let owner = owner(&rows);
    let rows = h.step(Event::Shutdown);
    assert_eq!(&*rows, [Request::Link(host::LinkUp::Draining)], "shutdown schedules the job's cancellation");
    crate::resume(&mut h.domain, &h.env, &mut h.out);
    let mut stopped = List::with_capacity(h.out.len());
    while let Some(row) = h.out.pop() {
        stopped.push(row).expect("shutdown outputs");
    }
    let stopped = stopped.into_boxed();
    assert!(!requested(&stopped), "a pending write stops without spawning");
    let host::JobEnding::Failed { failure, .. } = answer(&stopped) else { panic!("shutdown cancels the job") };
    assert_eq!(*failure, host::JobFailure::Cancelled(host::Reason::Shutdown));
    h.domain.reclaim();
    // The root drains the cancellation's hand-offs through its entry point.
    let rows = h.step(Event::Job { owner, event: tooling::Event::Written { owner, written: true } });
    assert!(!requested(&rows), "a late write cannot spawn a cancelled job");
    assert_eq!(h.domain.workspaces(), 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_delivering_job_waits_for_a_push_already_begun_when_cancelled() {
    let mut h = harness(true);
    let owner = running(&mut h, true);
    let pending = result(&mut h, owner);
    h.step(Event::Link(host::LinkDown::Cancel { run: fixtures::RUN, attempt: fixtures::ATTEMPT }));
    assert!(job(&mut h, owner, tooling::Event::Gone { owner }).is_empty());
    assert_eq!(h.domain.workspaces(), 1, "the push retains its checkout");
    let rows = git(&mut h, pending, false);
    let host::JobEnding::Failed { failure, work, .. } = answer(&rows) else { panic!("cancelled") };
    assert_eq!(*failure, host::JobFailure::Cancelled(host::Reason::Engine));
    assert!(work.left.is_some(), "the completed push counts despite cancellation");
    assert_eq!(h.domain.workspaces(), 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_delivering_job_without_one_writable_repository_pushes_nothing() {
    let mut h = harness(true);
    let owner = running(&mut h, false);
    assert!(result(&mut h, owner).is_empty());
    let rows = job(&mut h, owner, tooling::Event::Gone { owner });
    let host::JobEnding::Failed { failure, .. } = answer(&rows) else { panic!("no delivery target") };
    assert_eq!(*failure, host::JobFailure::Failed);
}

/// temper/docs/design/domain/worker/jobs.md; temper/docs/design/protocol/tooling.md, 4.2.
#[test]
fn a_close_jobs_commit_title_names_its_goal() {
    for (kind, request, expected) in [
        (4, b"{\"version\":1,\"goal\":7,\"changes\":[]}".as_slice(), b"Settle goal 7's status".as_slice()),
        (4, b"{\"version\":1,\"goal\":12345678901234567890,", b"Settle goal 12345678901234567890's status"),
        (4, b"request", b"job 4"),
        (4, b"{\"version\":2,\"goal\":7,", b"job 4"),
        (4, b"{\"version\":1,\"goal\":,", b"job 4"),
        (4, b"{\"version\":1,\"goal\":123456789012345678901,", b"job 4"),
        (4, b"{\"version\":1,\"goal\":7x,", b"job 4"),
        (4, "{\"version\":1,\"goal\":７,".as_bytes(), b"job 4"),
        (4, b"{\"version\":1,\"goal\":7", b"job 4"),
        (3, b"{\"version\":1,\"goal\":7,", b"job 3"),
    ] {
        assert_eq!(crate::jobs::commit_title(JobKind(kind), request).as_ref(), expected, "{request:?}");
    }
}
