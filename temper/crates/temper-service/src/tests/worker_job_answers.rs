//! Job answers through the root and service link (temper/docs/design/domain/worker/jobs.md).
use super::fixtures;
use crate::Worker;
use jig_domain_host as hub;
use jig_domain_link::JobKind;
use root::tooling::{Event, Exit, Request};
use skein_io as io;
use skein_lib::{List, Queue, Token, Wall};
use temper_worker_domain as root;

const JOB: Token = Token::new(81);
const ROOT_STEPS: u16 = 128;
const LINK_STEPS: u16 = 1000;

const MERGE_RESULT: &[u8] = br#"{"settled":["CHANGES.txt"],"left":[]}"#;
const MERGE_HEAD: [u8; 32] = [1; 32];
const MERGE_BASE: [u8; 32] = [2; 32];
const MERGE_PUSHED: [u8; 32] = [3; 32];

fn merge_assignment() -> root::Event {
    let mut items = fixtures::one_repository();
    items.repositories[0].start =
        root::wire::Start::Merge { branch: b"temper/task-7".as_slice().into(), base: MERGE_BASE };
    root::Event::AssignJob {
        job: hub::JobAssignment {
            run: JOB,
            attempt: Token::new(2),
            kind: JobKind(3),
            spec: Box::new([]),
            deadline: skein_lib::Duration::from_secs(10),
            workspace: Some(hub::Workspace { workstream: 7, items: Token::new(0) }),
            grants: Box::new([]),
        },
        items: Ok(items),
        spec: Ok(root::wire::JobSpec { command: b"app/merge".as_slice().into(), request: b"{}".as_slice().into() }),
    }
}

fn merge_git(op: root::checkout::git::Op) -> root::checkout::git::Done {
    use root::checkout::git::{Commit, Done, Op, Want};
    match op {
        Op::Look { .. } => Done::Looked { holds: false },
        Op::Fetch { want, .. } => Done::Fetched {
            commit: match want {
                Want::Commit { commit } => commit,
                Want::Branch { .. } | Want::Default => Commit::new(MERGE_HEAD),
            },
        },
        Op::Merge { theirs, .. } => {
            assert_eq!(theirs, Commit::new(MERGE_BASE));
            Done::Conflicted { files: Box::new([b"CHANGES.txt".as_slice().into()]) }
        }
        Op::Commit { parent, merging, .. } => {
            assert_eq!(parent, Commit::new(MERGE_HEAD));
            assert_eq!(merging, Some(Commit::new(MERGE_BASE)), "the delivery preserves both parents");
            Done::Committed { commit: Commit::new(MERGE_PUSHED) }
        }
        Op::Push { commit, branch, .. } => {
            assert_eq!(commit, Commit::new(MERGE_PUSHED));
            assert_eq!(branch.as_ref(), b"temper/task-7");
            Done::Succeeded
        }
        Op::DiffPaths { .. } => Done::Paths { files: Box::new([]) },
        Op::IsAncestor { .. } => Done::IsAncestor { contains: true },
        Op::Point { .. }
        | Op::Store { .. }
        | Op::Make { .. }
        | Op::Clone { .. }
        | Op::Create { .. }
        | Op::CheckOut { .. } => Done::Succeeded,
    }
}

#[derive(Clone, Copy)]
enum GitFault {
    None,
    CloneRefused,
    BaseBroken,
    PushStale,
    PushRefused,
    PushFailed,
}

// The root and service encoding are real; only job and git completions are scripted.
#[expect(clippy::wildcard_enum_match_arm, reason = "all other root requests use the service's routing")]
fn merge_step_script(worker: &mut Worker, event: root::Event, fault: GitFault) -> Box<[(Token, Request)]> {
    let mut out = Queue::uncapped();
    root::step(&mut worker.root, &worker.env, event, &mut out);
    let mut jobs = List::with_capacity(1);
    for _ in 0..ROOT_STEPS {
        let Some(request) = out.pop() else { return jobs.into_boxed() };
        match request {
            root::Request::Io { owner, op, .. } => {
                let done = match (&op, fault) {
                    (root::checkout::git::Op::Store { .. }, GitFault::CloneRefused) => {
                        root::checkout::git::Done::Failed { fault: root::checkout::git::Fault::Refused }
                    }
                    (
                        root::checkout::git::Op::Fetch { want: root::checkout::git::Want::Commit { commit }, .. },
                        GitFault::BaseBroken,
                    ) if commit.raw() == MERGE_BASE => root::checkout::git::Done::FailedWithOutput {
                        fault: root::checkout::git::Fault::Broken,
                        diagnostic: root::checkout::git::PushDiagnostic::new(b"fatal: base fetch failed", 0),
                    },
                    (root::checkout::git::Op::Push { .. }, GitFault::PushStale) => root::checkout::git::Done::Rejected,
                    (root::checkout::git::Op::Push { .. }, GitFault::PushRefused) => {
                        root::checkout::git::Done::FailedWithOutput {
                            fault: root::checkout::git::Fault::Refused,
                            diagnostic: root::checkout::git::PushDiagnostic::new(
                                b"fatal: Authentication failed for repo",
                                0,
                            ),
                        }
                    }
                    (root::checkout::git::Op::Push { .. }, GitFault::PushFailed) => {
                        root::checkout::git::Done::FailedWithOutput {
                            fault: root::checkout::git::Fault::Unreachable,
                            diagnostic: root::checkout::git::PushDiagnostic::new(b"fatal: early EOF", 0),
                        }
                    }
                    _ => merge_git(op),
                };
                root::step(&mut worker.root, &worker.env, root::Event::Done { owner, done }, &mut out);
            }
            root::Request::Job { owner, op } => jobs.push((owner, op)).expect("one job operation at a time"),
            other => worker.route(other),
        }
    }
    panic!("the checkout's bounded script settles");
}

fn merge_step(worker: &mut Worker, event: root::Event) -> Box<[(Token, Request)]> {
    merge_step_script(worker, event, GitFault::None)
}

fn merge_event(worker: &mut Worker, owner: Token, event: Event) -> Box<[(Token, Request)]> {
    merge_step(worker, root::Event::Job { owner, event })
}

fn start_merge(worker: &mut Worker) -> Token {
    let rows = merge_step(worker, merge_assignment());
    let [(owner, Request::Write { bytes, .. })] = rows.as_ref() else { panic!("job writes its request") };
    assert_eq!(bytes.as_ref(), b"{}");
    let owner = *owner;
    let rows = merge_event(worker, owner, Event::Written { owner, written: true });
    let [(_, Request::Spawn { .. })] = rows.as_ref() else { panic!("job spawns after its request write") };
    assert!(merge_event(worker, owner, Event::Spawned { owner }).is_empty());
    owner
}

fn deliver_merge(worker: &mut Worker, owner: Token) {
    let rows = merge_event(worker, owner, Event::Exited { owner, exit: Exit::Code(0), tail: Box::new([]) });
    let [(_, Request::Read { most, .. })] = rows.as_ref() else { panic!("job reads its result") };
    assert!(usize::try_from(*most).unwrap() >= MERGE_RESULT.len());
    assert!(merge_event(worker, owner, Event::Read { owner, bytes: Some(MERGE_RESULT.into()) }).is_empty());
    assert_eq!(worker.root.kept_answers(), 0, "the answer waits for Gone after the push");
}

fn merge_answer(
    peer: &mut fixtures::worker_peers::LinkPeer,
    worker: &mut Worker,
) -> (hub::LinkUp, Box<[jig_protocol_worker_wire::Part]>) {
    for _ in 0..LINK_STEPS {
        if let Some(jig_protocol_engine_workers::Request::Up {
            up: up @ hub::LinkUp::JobAnswer { .. }, parts, ..
        }) = peer.turn(worker)
        {
            return (up, parts);
        }
    }
    panic!("worker sends its job answer with the parts its tokens name");
}

fn assert_merge_answer(up: &hub::LinkUp, parts: &[jig_protocol_worker_wire::Part]) {
    use skein_lib::Reader;
    use temper_protocol_worker_wire::v1;
    let hub::LinkUp::JobAnswer { run, attempt, answer: hub::JobEnding::Finished { result, work } } = up else {
        panic!("one finished job answer: {up:?}");
    };
    assert_eq!((*run, *attempt), (JOB, Token::new(2)));
    assert_eq!(result.as_ref(), MERGE_RESULT);
    let [part] = parts else { panic!("one left part") };
    assert_eq!(work.left, Some(part.token));
    assert_eq!(work.saved, None);
    let mut reader = Reader::new(&part.bytes);
    let left = v1::Left::decode(&fixtures::wire_limits(), &mut reader).unwrap();
    assert!(reader.is_empty());
    assert_eq!(left.landed().len(), 1);
    let landed = left.landed().get(0).unwrap();
    assert_eq!(landed.tag(), temper_engine_domain_forge::tag(1, 3));
    assert_eq!(landed.commit(), &MERGE_PUSHED);
}

fn merge_worker() -> (Worker, fixtures::worker_peers::LinkPeer) {
    let mut worker = fixtures::started_worker().worker;
    worker.env.limits.checkout.conflicts = 1;
    worker.root = root::Domain::new(
        &worker.env.limits,
        Box::new([root::tooling::Kind {
            bound: crate::limits::merge_bound(),
            verb: b"merge".as_slice().into(),
            delivers: true,
        }]),
    );
    let mut peer = fixtures::worker_peers::LinkPeer::new();
    peer.open(&mut worker);
    (worker, peer)
}

/// temper/docs/design/domain/worker/jobs.md; jig/docs/design/protocol/channel/records.md, 4, and jig/docs/design/protocol/channel/README.md, 5.2.
#[test]
fn a_delivering_job_answers_with_left_parts_and_replays_until_acknowledged() {
    let (mut worker, mut peer) = merge_worker();
    let owner = start_merge(&mut worker);
    deliver_merge(&mut worker, owner);
    assert!(merge_event(&mut worker, owner, Event::Gone { owner }).is_empty());
    let (answer, parts) = merge_answer(&mut peer, &mut worker);
    assert_merge_answer(&answer, &parts);
    while let Some(note) = worker.notes().pop() {
        if let crate::Note::WorkerJob { .. } = note {
            panic!("finished jobs have no failure note");
        }
    }
    assert_eq!(worker.root.kept_answers(), 1);
    assert_eq!(worker.root.workspaces(), 0);
    worker.up(fixtures::NOW, io::Event::Closed { owner: peer.owner });
    worker.iterate(fixtures::NOW, Wall::EPOCH);
    worker.iterate(worker.next_deadline().expect("redial"), Wall::EPOCH);
    worker.io().clear();
    let mut peer = fixtures::worker_peers::LinkPeer::new();
    peer.open(&mut worker);
    let replay = merge_answer(&mut peer, &mut worker);
    assert_eq!(replay, (answer, parts), "reconnect retains identical application parts");
    merge_step(&mut worker, root::Event::Link(hub::LinkDown::Acknowledge { run: JOB, attempt: Token::new(2) }));
    assert_eq!(worker.root.kept_answers(), 0);
    assert!(worker.root.hosting().is_empty());
    worker.iterate(fixtures::NOW, Wall::EPOCH);
    start_merge(&mut worker);
    assert_eq!(worker.root.hosting().len(), 1, "acknowledgement releases the only slot");
}

/// Failed jobs retain delivery or preparation resources; entrance refusals name none.
#[test]
fn failed_job_answers_carry_their_resources_and_refusals_carry_none() {
    use skein_lib::Reader;
    use temper_protocol_worker_wire::v1;
    for unprepared in [false, true] {
        let (mut worker, mut peer) = merge_worker();
        if unprepared {
            assert!(merge_step_script(&mut worker, merge_assignment(), GitFault::CloneRefused).is_empty());
        } else {
            let owner = start_merge(&mut worker);
            deliver_merge(&mut worker, owner);
            merge_step(&mut worker, root::Event::Link(hub::LinkDown::Cancel { run: JOB, attempt: Token::new(2) }));
            merge_event(&mut worker, owner, Event::Gone { owner });
        }
        let (answer, parts) = merge_answer(&mut peer, &mut worker);
        let hub::LinkUp::JobAnswer { answer: hub::JobEnding::Failed { failure, work, .. }, .. } = answer else {
            panic!("failed job answers");
        };
        let [part] = parts.as_ref() else { panic!("one named resource") };
        if unprepared {
            assert_eq!(
                failure,
                hub::JobFailure::Unprepared(hub::Preparation::Permanent { resource: Some(part.token) })
            );
            let preparation = v1::Preparation::decode(&fixtures::wire_limits(), &mut Reader::new(&part.bytes)).unwrap();
            let v1::Preparation::Refused(tag) = preparation else { panic!("refused repository") };
            assert_eq!(tag.tag(), temper_engine_domain_forge::tag(1, 3));
            assert_eq!(work, hub::Work { left: None, saved: None });
        } else {
            assert_eq!(failure, hub::JobFailure::Cancelled(hub::Reason::Engine));
            assert_eq!(work.left, Some(part.token));
            let left = v1::Left::decode(&fixtures::wire_limits(), &mut Reader::new(&part.bytes)).unwrap();
            assert_eq!(left.landed().get(0).unwrap().commit(), &MERGE_PUSHED);
        }
        while worker.notes().pop().is_some() {}
        let root::Event::AssignJob { mut job, spec, .. } = merge_assignment() else { panic!("job assignment") };
        job.run = Token::new(99);
        merge_step(&mut worker, root::Event::AssignJob { job, spec, items: Err(root::wire::Undecodable) });
        let (answer, parts) = merge_answer(&mut peer, &mut worker);
        let hub::LinkUp::JobAnswer { answer: hub::JobEnding::Refused(reason), .. } = answer else {
            panic!("undecodable workspace is refused");
        };
        assert!(parts.is_empty());
        assert_eq!(
            worker.notes().pop(),
            Some(crate::Note::WorkerJob {
                run: 99,
                attempt: 2,
                kind: 3,
                verb: b"merge".as_slice().into(),
                ending: temper_engine_domain::JobDiagnosticEnding::Refused(reason),
                why: b"workspace items".as_slice().into(),
            })
        );
        assert!(worker.notes().is_empty(), "one worker note per refusal");
    }
}

enum FailureCase {
    BaseFetch,
    Command(Exit),
    Result,
    Delivery(GitFault),
}

fn fail_merge(worker: &mut Worker, case: FailureCase) {
    if let FailureCase::BaseFetch = case {
        assert!(merge_step_script(worker, merge_assignment(), GitFault::BaseBroken).is_empty());
        return;
    }
    let owner = start_merge(worker);
    let (exit, read, bytes, fault) = match case {
        FailureCase::BaseFetch => unreachable!(),
        FailureCase::Command(exit) => (exit, false, None, GitFault::None),
        FailureCase::Result => (Exit::Code(0), true, None, GitFault::None),
        FailureCase::Delivery(fault) => (Exit::Code(0), true, Some(MERGE_RESULT.into()), fault),
    };
    merge_event(worker, owner, Event::Exited { owner, exit, tail: b"stderr".as_slice().into() });
    if read {
        merge_step_script(worker, root::Event::Job { owner, event: Event::Read { owner, bytes } }, fault);
    }
    merge_event(worker, owner, Event::Gone { owner });
}

#[test]
fn failed_jobs_name_the_workers_why_once() {
    use hub::{DeliveryOutcome, JobFailure, Preparation};
    let fetch = b"fetch base: Broken: fatal: base fetch failed".as_slice();
    for (case, expected_failure, expected_detail, expected_why) in [
        (FailureCase::BaseFetch, JobFailure::Unprepared(Preparation::Transient), fetch, fetch),
        (FailureCase::Command(Exit::Code(1)), JobFailure::Failed, b"exit 1: stderr", b"exit 1: stderr"),
        (FailureCase::Command(Exit::Signal), JobFailure::Failed, b"signal: stderr", b"signal: stderr"),
        (FailureCase::Command(Exit::Deadline), JobFailure::Deadline, b"deadline passed", b"deadline passed"),
        (
            FailureCase::Result,
            JobFailure::Failed,
            b"read result.json failed or past bound after exit 0: stderr",
            b"read result.json failed or past bound after exit 0: stderr",
        ),
        (
            FailureCase::Delivery(GitFault::PushStale),
            JobFailure::Undelivered(DeliveryOutcome::Stale),
            b"delivery Stale: branch moved",
            b"delivery Stale: branch moved",
        ),
        (
            FailureCase::Delivery(GitFault::PushRefused),
            JobFailure::Undelivered(DeliveryOutcome::Failed),
            b"delivery Failed: push: Refused: fatal: Authentication failed for repo",
            b"delivery Failed: push: Refused: fatal: Authentication failed for repo",
        ),
        (
            FailureCase::Delivery(GitFault::PushFailed),
            JobFailure::Undelivered(DeliveryOutcome::Failed),
            b"delivery Failed: push: Unreachable: fatal: early EOF",
            b"delivery Failed: push: Unreachable: fatal: early EOF",
        ),
    ] {
        let (mut worker, mut peer) = merge_worker();
        while worker.notes().pop().is_some() {}
        fail_merge(&mut worker, case);
        let (answer, _) = merge_answer(&mut peer, &mut worker);
        let hub::LinkUp::JobAnswer { answer: hub::JobEnding::Failed { failure, detail, .. }, .. } = answer else {
            panic!("failed job answers");
        };
        assert_eq!(failure, expected_failure);
        let bound = usize::try_from(worker.env.limits.host.detail_bytes).unwrap();
        assert_eq!(detail.as_ref(), &expected_detail[expected_detail.len().saturating_sub(bound)..]);
        let mut found = 0_u32;
        while let Some(note) = worker.notes().pop() {
            if let crate::Note::WorkerJob { run, attempt, kind, verb, ending, why } = note {
                assert_eq!((run, attempt, kind, verb.as_ref()), (JOB.raw(), 2, 3, b"merge".as_slice()));
                assert_eq!(why.as_ref(), expected_why);
                assert_eq!(
                    ending,
                    temper_engine_domain::JobDiagnosticEnding::Failed { failure, detail: detail.clone() }
                );
                found += 1;
            }
        }
        assert_eq!(found, 1, "the worker names a failed job once");
        merge_step(&mut worker, root::Event::Lost);
        merge_step(&mut worker, root::Event::Ready);
        while let Some(note) = worker.notes().pop() {
            if let crate::Note::WorkerJob { .. } = note {
                panic!("a kept answer resent has no second note");
            }
        }
    }
}
