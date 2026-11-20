//! Operator notes at the root boundary (domain/worker/README.md, 4.3).
use alloc::boxed::Box;

use super::{fixtures, harness::Harness};
use crate::checkout::git::{Commit, Done, Into, Op, Place, Want};
use crate::{Domain, Event, Git, Request, checkout, host, route, tooling, wire};
use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, List, Queue, Time, Token};

const COMMIT: Commit = Commit::new([0xab; 32]);
const KIND: JobKind = JobKind(9);

fn note(job: Option<JobKind>, repository: &[u8], what: Git) -> Request {
    let job = match job {
        Some(kind) => Some((kind, b"check".as_slice().into())),
        None => None,
    };
    Request::Git { run: fixtures::RUN, attempt: fixtures::ATTEMPT, job, repository: repository.into(), what }
}

fn prepared(job: Option<JobKind>) -> (Harness, Token, Box<[Request]>) {
    let mut h = Harness::new();
    let kind = tooling::Kind {
        bound: JobBound { kind: KIND, spec: 128, result: 128, deadline: Duration::from_secs(10) },
        verb: b"check".as_slice().into(),
        delivers: false,
    };
    h.domain = Domain::new(&h.env.limits, Box::new([kind]));
    h.step(Event::Ready);
    let mut items = fixtures::items(&[b"app", b"library"]);
    items.repositories[1].start = wire::Start::Commit { commit: COMMIT.raw() };
    let event = match job {
        None => Event::Assign { assignment: fixtures::assignment(true, Box::new([])), items: Ok(items) },
        Some(kind) => Event::AssignJob {
            job: host::JobAssignment {
                run: fixtures::RUN,
                attempt: fixtures::ATTEMPT,
                kind,
                spec: b"spec".as_slice().into(),
                deadline: Duration::from_secs(5),
                workspace: Some(host::Workspace { workstream: fixtures::WORKSTREAM, items: Token::new(0) }),
                grants: Box::new([]),
            },
            items: Ok(items),
            spec: Ok(wire::JobSpec { command: b"app/check".as_slice().into(), request: Box::new([]) }),
        },
    };
    let mut pending = Queue::uncapped();
    for row in h.step(event) {
        pending.push(row);
    }
    let mut notes = List::with_capacity(8);
    let mut hold = None;
    for _ in 0..64_u32 {
        let Some(request) = pending.pop() else {
            return (h, hold.expect("checkout had io"), notes.into_boxed());
        };
        match request {
            Request::Io { owner, op, .. } => {
                hold = Some(owner);
                let done = match op {
                    Op::Fetch { .. } => Done::Fetched { commit: COMMIT },
                    Op::Look { .. } => Done::Looked { holds: false },
                    Op::Pack { .. }
                    | Op::Reds { .. }
                    | Op::Reset { .. }
                    | Op::Make { .. }
                    | Op::Store { .. }
                    | Op::Clone { .. }
                    | Op::Point { .. }
                    | Op::Create { .. }
                    | Op::CheckOut { .. } => Done::Succeeded,
                    Op::IsAncestor { .. }
                    | Op::Brings { .. }
                    | Op::DiffPaths { .. }
                    | Op::Merge { .. }
                    | Op::Commit { .. }
                    | Op::Push { .. } => panic!("read-only prepare does not ask this operation"),
                };
                for row in h.step(Event::Done { owner, done }) {
                    pending.push(row);
                }
            }
            request @ Request::Git { .. } => notes.push(request).expect("four prepare notes"),
            Request::Decode { .. } | Request::Job { .. } | Request::Link(_) => {}
            Request::AgentStopped { .. }
            | Request::Unprepared { .. }
            | Request::ResultsIgnored { .. }
            | Request::RedsAsk { .. }
            | Request::RedsAnswer { .. }
            | Request::ReadUsage { .. }
            | Request::ReadList { .. }
            | Request::CancelRead { .. }
            | Request::Source { .. }
            | Request::SourceFault
            | Request::Hello(_)
            | Request::Answer { .. }
            | Request::JobAnswer { .. }
            | Request::CancelRelay { .. }
            | Request::Stop
            | Request::Abandon
            | Request::Process(_)
            | Request::Encode { .. }
            | Request::Fact { .. }
            | Request::CancelIo { .. } => panic!("unexpected prepare request: {request:?}"),
        }
    }
    panic!("prepare settles");
}

fn expected(job: Option<JobKind>) -> Box<[Request]> {
    Box::new([
        note(job, b"app", Git::Fetch { want: Want::Branch { branch: b"main".as_slice().into() }, into: Into::Fetched }),
        note(job, b"app", Git::CheckOut { commit: COMMIT }),
        note(job, b"library", Git::Fetch { want: Want::Commit { commit: COMMIT }, into: Into::Fetched }),
        note(job, b"library", Git::CheckOut { commit: COMMIT }),
    ])
}

#[test]
fn a_runs_prepare_notes_each_fetch_and_check_out() {
    let (_, _, notes) = prepared(None);
    assert_eq!(notes, expected(None));
}

#[test]
fn a_jobs_prepare_notes_each_fetch_and_check_out_with_its_kind() {
    let (_, _, notes) = prepared(Some(KIND));
    assert_eq!(notes, expected(Some(KIND)));
}

fn routed(h: &mut Harness, owner: Token, op: Op) -> Box<[Request]> {
    h.domain.checkout_out.push(checkout::Request::Io { owner, op, deadline: Time::ZERO });
    route::hand_off(&mut h.domain, &h.env, &mut h.out);
    let mut rows = List::with_capacity(2);
    while let Some(row) = h.out.pop() {
        rows.push(row).expect("a note then io");
    }
    rows.into_boxed()
}

#[test]
fn a_prepared_hold_notes_only_fetch_and_check_out_before_forwarding_io() {
    let (mut h, hold, _) = prepared(None);
    let at = Place { workspace: Token::new(0), repository: b"app".as_slice().into(), git: None };
    for (op, what) in [
        (
            Op::Fetch {
                at: at.clone(),
                store: Token::new(0),
                remote: b"org/app".as_slice().into(),
                want: Want::Default,
                into: Into::Landing,
                identity: 0,
                account: None,
            },
            Some(Git::Fetch { want: Want::Default, into: Into::Landing }),
        ),
        (
            Op::CheckOut { at: at.clone(), commit: COMMIT, clean: true, identity: 0, account: None },
            Some(Git::CheckOut { commit: COMMIT }),
        ),
        (Op::Pack { at: at.clone(), store: Some(Token::new(0)) }, None),
        (Op::Pack { at: at.clone(), store: None }, None),
        (Op::Point { at: at.clone(), into: Into::Landing, commit: COMMIT }, None),
        (Op::Merge { at: at.clone(), theirs: COMMIT, identity: 0, account: None }, None),
        (
            Op::Push {
                at,
                remote: b"org/app".as_slice().into(),
                commit: COMMIT,
                branch: b"topic".as_slice().into(),
                expected: None,
                identity: 0,
                account: None,
            },
            None,
        ),
    ] {
        let mut expected = List::with_capacity(2);
        if let Some(what) = what {
            expected.push(note(None, b"app", what)).expect("one operator note");
        }
        expected.push(Request::Io { owner: hold, op: op.clone(), deadline: Time::ZERO }).expect("io after note");
        assert_eq!(routed(&mut h, hold, op.clone()), expected.into_boxed());
        // An unowned hold is still forwarded, without a note.
        let unknown = Token::new(u64::MAX);
        assert_eq!(
            routed(&mut h, unknown, op.clone()).as_ref(),
            &[Request::Io { owner: unknown, op, deadline: Time::ZERO }]
        );
    }
}
