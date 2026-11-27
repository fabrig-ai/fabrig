//! The service's job operations (temper/docs/design/protocol/tooling.md, 2).
#[path = "worker_jobs/hooks.rs"]
mod hooks;

use super::Worker;
use crate::tests::fixtures;
use io::kernel::{self, Done, Op};
use jig_domain_host as hub;
use jig_domain_link::JobKind;
use jig_protocol_worker_link as link;
use root::tooling::{Event, Exit, Request};
use skein_io::{self as io, kernel::Fd};
use skein_lib::{Time, Token, Writer, stream};
use temper_worker_domain as root;

use crate::testing::{JOB, event, launch_args};
const CHILD: Token = Token::new(82);
const STDOUT: Token = Token::new(83);
const STDERR: Token = Token::new(84);
const DEADLINE: Time = Time::ZERO.saturating_add(skein_lib::Duration::from_secs(2));

fn launch(worker: &mut Worker) -> (Token, kernel::Spawn) {
    launch_args(
        worker,
        Box::new([
            b"workspaces/7/home/tool".as_slice().into(),
            b"prepare".as_slice().into(),
            b"workspaces/7/request.json".as_slice().into(),
        ]),
        DEADLINE,
    )
}
fn demand(pipe: Token) -> io::Request {
    io::Request::Stream {
        stream: pipe,
        down: stream::Down::Demand { read: stream::Read::Arrived { max: 4096 }, room: 0 },
    }
}
fn spawned(worker: &mut Worker, owner: Token) {
    assert!(worker.accepts(&io::Event::Spawned { owner, child: CHILD, pipes: Box::new([STDOUT, STDERR]) }));
    worker.up(Time::ZERO, io::Event::Spawned { owner, child: CHILD, pipes: Box::new([STDOUT, STDERR]) });
    assert_eq!(event(worker), Event::Spawned { owner: JOB });
    for pipe in [STDOUT, STDERR] {
        assert_eq!(worker.io().pop(), Some(demand(pipe)));
    }
    assert!(worker.io().is_empty());
}
fn eof(worker: &mut Worker) {
    for owner in [STDOUT, STDERR] {
        worker.up(Time::ZERO, io::Event::Stream { owner, up: stream::Up::End });
        assert_eq!(worker.io().pop(), Some(io::Request::Close { entity: owner }));
    }
    assert_eq!(worker.io().pop(), Some(io::Request::Close { entity: CHILD }));
}
fn released(worker: &mut Worker, owner: Token) {
    assert!(worker.accepts(&io::Event::Closed { owner }));
    worker.up(Time::ZERO, io::Event::Closed { owner });
    assert_eq!(event(worker), Event::Gone { owner: JOB });
    assert!(worker.events.is_empty());
    assert!(!worker.accepts(&io::Event::Closed { owner }));
    assert!(worker.jobs.empty());
}

/// temper/docs/design/protocol/tooling.md, 2: the explicit environment, arguments and group signal.
#[test]
fn a_jobs_command_gets_exactly_four_environment_variables_and_its_own_group() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    let (owner, spawn) = launch(worker);
    assert_eq!(spawn.program.as_ref(), b"/worker/workspaces/7/home/tool");
    assert_eq!(
        spawn.args.as_ref(),
        [Box::from(b"prepare".as_slice()), Box::from(b"/worker/workspaces/7/request.json".as_slice())]
    );
    assert_eq!(spawn.root, Fd::new(9));
    assert_eq!(spawn.dir.as_ref(), b"workspaces/7");
    assert_eq!(
        spawn.env.as_ref(),
        [
            Box::from(b"PATH=/usr/bin:/bin".as_slice()),
            Box::from(b"HOME=/worker/workspaces/7".as_slice()),
            Box::from(b"LANG=C.UTF-8".as_slice()),
            Box::from(b"TEMPER_JOB=/worker/workspaces/7".as_slice()),
        ]
    );
    assert_eq!(
        spawn.pipes.as_ref(),
        [
            kernel::Pipe { child: 1, way: kernel::Way::Out, parent: None },
            kernel::Pipe { child: 2, way: kernel::Way::Out, parent: None }
        ]
    );
    // Skein starts every plain Spawn in its own group; the service addresses that group.
    spawned(worker, owner);
    worker.job_request(JOB, Request::Kill { owner: JOB });
    assert_eq!(
        worker.io().pop(),
        Some(io::Request::Signal {
            child: CHILD,
            signal: kernel::Signal::Kill,
            to: kernel::Target::Group,
            reason: kernel::Reason::Cancelled
        })
    );
    worker.up(Time::ZERO, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
    eof(worker);
    assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Signal, tail: Box::new([]) });
    released(worker, owner);
}

/// temper/docs/design/protocol/tooling.md, 2: the deadline kills the group, even after its leader exits.
#[test]
fn a_job_past_its_deadline_has_its_group_killed_and_exits_deadline() {
    for leader_exited in [false, true] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        let (owner, _) = launch(worker);
        spawned(worker, owner);
        if leader_exited {
            worker.up(Time::ZERO, io::Event::Exited { owner, exit: kernel::Exit::Code(0) });
        }
        worker.jobs_fire(DEADLINE, false);
        assert_eq!(
            worker.io().pop(),
            Some(io::Request::Signal {
                child: CHILD,
                signal: kernel::Signal::Kill,
                to: kernel::Target::Group,
                reason: kernel::Reason::Deadline
            })
        );
        worker.jobs_fire(DEADLINE, false);
        assert!(worker.io().is_empty(), "the alarm fires once");
        if !leader_exited {
            worker.up(DEADLINE, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
        }
        eof(worker);
        assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Deadline, tail: Box::new([]) });
        released(worker, owner);
    }
}

/// temper/docs/design/protocol/tooling.md, 2: kill waits for the process and pipes' release before Gone.
#[test]
fn a_kill_ends_the_group_and_gone_follows_the_release() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    let (owner, _) = launch(worker);
    spawned(worker, owner);
    worker.job_request(JOB, Request::Kill { owner: JOB });
    assert_eq!(
        worker.io().pop(),
        Some(io::Request::Signal {
            child: CHILD,
            signal: kernel::Signal::Kill,
            to: kernel::Target::Group,
            reason: kernel::Reason::Cancelled
        })
    );
    assert!(worker.events.is_empty(), "a kill request alone proves no process gone");
    worker.up(Time::ZERO, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
    assert!(worker.events.is_empty(), "the tail can still arrive");
    eof(worker);
    assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Signal, tail: Box::new([]) });
    for pipe in [STDERR, STDOUT] {
        worker.up(Time::ZERO, io::Event::Closed { owner: pipe });
        assert!(worker.events.is_empty(), "pipes alone prove no child release");
    }
    released(worker, owner);
}

#[derive(Debug, PartialEq, Eq)]
enum FileOp {
    Open(kernel::OpenHow),
    Read,
    Size,
    Write(u32, u64),
    Truncate(u64),
    Close,
}
#[expect(clippy::wildcard_enum_match_arm, reason = "unexpected file operations fail the test")]
fn complete(worker: &mut Worker, result: Result<Done, kernel::Error>) -> FileOp {
    let submit = worker.submitted().pop().expect("one job file operation");
    assert!(worker.submitted().is_empty());
    let kind = match &submit.kind {
        Op::Open { how, .. } => FileOp::Open(*how),
        Op::Write { from, at, .. } => FileOp::Write(*from, *at),
        Op::Read { .. } => FileOp::Read,
        Op::Stat { .. } => FileOp::Size,
        Op::Truncate { length, .. } => FileOp::Truncate(*length),
        Op::Close { .. } => FileOp::Close,
        other => panic!("unexpected file operation: {other:?}"),
    };
    worker.complete(Time::ZERO, kernel::Complete { op: submit.op, kind: submit.kind, result }).unwrap();
    kind
}
fn read_bytes(worker: &mut Worker, bytes: &[u8]) {
    let mut submit = worker.submitted().pop().expect("read operation");
    let Op::Read { buf, .. } = &mut submit.kind else { panic!("read operation") };
    for (destination, source) in buf.get_mut(..bytes.len()).unwrap().iter_mut().zip(bytes) {
        *destination = *source;
    }
    worker
        .complete(
            Time::ZERO,
            kernel::Complete {
                op: submit.op,
                kind: submit.kind,
                result: Ok(Done::Count(u32::try_from(bytes.len()).unwrap())),
            },
        )
        .unwrap();
}
fn read(worker: &mut Worker, most: u32) {
    worker.job_request(JOB, Request::Read { owner: JOB, path: b"workspaces/7/result.json".as_slice().into(), most });
    assert_eq!(complete(worker, Ok(Done::Fd(Fd::new(101)))), FileOp::Open(kernel::OpenHow::ReadNoFollow));
}
fn close_file(worker: &mut Worker) {
    assert!(worker.events.is_empty(), "the descriptor closes before its terminal");
    assert_eq!(complete(worker, Ok(Done::Nothing)), FileOp::Close);
}

/// temper/docs/design/protocol/tooling.md, 2 and 6: a result at its bound succeeds; one byte past it is None.
#[test]
fn a_result_file_past_its_bound_reads_as_none() {
    for (size, expected) in [(4, Some(Box::from(b"abcd".as_slice()))), (5, None)] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        read(worker, 4);
        read_bytes(worker, b"ab");
        read_bytes(worker, b"cd");
        assert_eq!(
            complete(
                worker,
                Ok(Done::Stat(kernel::Stat { kind: kernel::Kind::File, size, mode: 0o600, owner: 0, links: 1 }))
            ),
            FileOp::Size
        );
        close_file(worker);
        assert_eq!(event(worker), Event::Read { owner: JOB, bytes: expected });
        assert!(worker.jobs.empty());
    }
}

/// temper/docs/design/protocol/tooling.md, 4.1 and 6: a bounded result carries exactly the bytes read before EOF.
#[test]
fn a_result_file_shorter_than_its_bound_reads_exact_bytes() {
    // Merge kind's small-profile result bound (temper/docs/design/domain/worker/jobs.md).
    const RESULT_BOUND: u32 = 512;
    const CHUNK_BYTES: usize = br#"{"settled":"#.len();
    for bytes in [
        br#"{"settled":["CHANGES.txt"]}"#.as_slice(),
        br#"{"settled":["CHANGES.txt"],"left":["src/main.rs"]}"#.as_slice(),
        b"".as_slice(),
    ] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        read(worker, RESULT_BOUND);
        for chunk in bytes.chunks(CHUNK_BYTES) {
            read_bytes(worker, chunk);
            assert!(worker.events.is_empty(), "a short read alone is not EOF");
        }
        assert_eq!(complete(worker, Ok(Done::Count(0))), FileOp::Read);
        close_file(worker);
        assert_eq!(event(worker), Event::Read { owner: JOB, bytes: Some(bytes.into()) });
        assert!(worker.events.is_empty(), "one terminal event per read");
        assert!(worker.submitted().is_empty(), "the file has no remaining operation");
        assert!(worker.jobs.empty());
    }
}

/// temper/docs/design/protocol/tooling.md, 2: a spec or workspace decode failure crosses as Undecodable.
#[test]
fn a_job_spec_that_does_not_decode_reaches_the_root_undecodable() {
    use temper_protocol_worker_wire::v1::{self, Record};
    let valid = v1::JobSpec {
        command: b"tool".as_slice().into(),
        request: b"{}".as_slice().into(),
        refusal: true,
        find: false,
        fallback: Box::new([]),
    };
    let valid = valid.encode(&v1::CEILINGS).unwrap();
    let mut trailing = Writer::new(valid.len().checked_add(1).unwrap());
    trailing.put(&valid).unwrap();
    trailing.put(&[0]).unwrap();
    let trailing = trailing.finish();
    for (spec, good, items_good) in
        [(valid.as_ref(), true, true), (b"".as_slice(), false, true), (trailing.as_ref(), false, false)]
    {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        let link::Request::Down { down: hub::LinkDown::Assign { assignment }, mut parts } =
            fixtures::repository_assignment()
        else {
            panic!("workspace assignment")
        };
        if !items_good {
            parts[0].bytes = Box::new([]);
        }
        let job = hub::JobAssignment {
            run: JOB,
            attempt: Token::new(2),
            kind: JobKind(1),
            spec: spec.into(),
            deadline: skein_lib::Duration::from_secs(2),
            workspace: assignment.assignment.workspace,
            grants: Box::new([]),
        };
        worker.link(Time::ZERO, link::Request::Down { down: hub::LinkDown::AssignJob { job }, parts });
        let root::Event::AssignJob { job, items, spec } = worker.engine.pop().expect("root assignment entrance") else {
            panic!("root assignment")
        };
        assert_eq!(job.run, JOB);
        assert_eq!(items.is_ok(), items_good);
        assert_eq!(
            spec,
            if good {
                Ok(root::wire::JobSpec {
                    command: b"tool".as_slice().into(),
                    request: b"{}".as_slice().into(),
                    refusal: true,
                })
            } else {
                Err(root::wire::Undecodable)
            }
        );
    }
}

/// temper/docs/design/protocol/tooling.md, 2 and 6: stderr keeps only its last `tail_bytes`, including an unterminated end.
#[test]
fn a_jobs_error_tail_wraps_and_outlives_its_pipe() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    worker.env.limits.tooling.tail_bytes = 4;
    let (owner, _) = launch(worker);
    spawned(worker, owner);
    for bytes in [b"01234".as_slice(), b"567".as_slice()] {
        worker.up(Time::ZERO, io::Event::Stream { owner: STDERR, up: stream::Up::Bytes(bytes.into()) });
        assert_eq!(worker.io().pop(), Some(demand(STDERR)));
    }
    worker.up(Time::ZERO, io::Event::Exited { owner, exit: kernel::Exit::Code(3) });
    eof(worker);
    assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Code(3), tail: b"4567".as_slice().into() });
    released(worker, owner);
}

/// temper/docs/design/protocol/tooling.md, 2: a deadline or Kill racing the spawn still kills its eventual group.
#[test]
fn a_stop_before_spawn_waits_for_the_spawn_and_release() {
    for deadline in [false, true] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        let (owner, _) = launch(worker);
        if deadline {
            worker.jobs_fire(DEADLINE, false);
        } else {
            worker.job_request(JOB, Request::Kill { owner: JOB });
        }
        assert!(worker.io().is_empty());
        worker.up(DEADLINE, io::Event::Spawned { owner, child: CHILD, pipes: Box::new([STDOUT, STDERR]) });
        assert_eq!(event(worker), Event::Spawned { owner: JOB });
        assert_eq!(
            worker.io().pop(),
            Some(io::Request::Signal {
                child: CHILD,
                signal: kernel::Signal::Kill,
                to: kernel::Target::Group,
                reason: if deadline { kernel::Reason::Deadline } else { kernel::Reason::Cancelled }
            })
        );
        for pipe in [STDOUT, STDERR] {
            assert_eq!(worker.io().pop(), Some(demand(pipe)));
        }
        worker.up(DEADLINE, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
        eof(worker);
        assert_eq!(
            event(worker),
            Event::Exited {
                owner: JOB,
                exit: if deadline { Exit::Deadline } else { Exit::Signal },
                tail: Box::new([])
            }
        );
        released(worker, owner);
    }
}

/// temper/docs/design/protocol/tooling.md, 2: a spawn failure has one Exited and one Gone, after io closes it.
#[test]
fn a_failed_spawn_ends_once_and_releases_its_owner() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    let (owner, _) = launch(worker);
    worker.up(Time::ZERO, io::Event::Failed { owner, error: io::Error::Other });
    assert!(worker.events.is_empty());
    worker.up(Time::ZERO, io::Event::Closed { owner });
    assert_eq!(
        event(worker),
        Event::Exited {
            owner: JOB,
            exit: Exit::Signal,
            tail: b"spawn command /worker/workspaces/7/home/tool in /worker/workspaces/7 failed: Other"
                .as_slice()
                .into()
        }
    );
    assert_eq!(event(worker), Event::Gone { owner: JOB });
    assert!(worker.jobs.empty());
}

/// temper/docs/design/protocol/tooling.md, 6: the service profile bounds every job, with no configured kinds yet.
#[test]
fn the_jobs_profile_uses_the_workers_slots_and_tooling_bounds() {
    let worker = fixtures::worker();
    let limits = &worker.env.limits.tooling;
    assert_eq!((limits.file_bytes, limits.tail_bytes, limits.path_bytes, limits.jobs), (1_048_576, 65_536, 4096, 1));
    assert_eq!(limits.path.as_ref(), b"/usr/local/bin:/usr/bin:/bin");
    assert_eq!(worker.payload.job_spec_request, limits.file_bytes);
    assert_eq!(worker.payload.job_spec_command, limits.path_bytes);
}

/// temper/docs/design/protocol/tooling.md, 2 and 6: writes are bounded, handle short writes, and replace an existing file's length.
#[test]
fn a_request_write_is_bounded_and_truncates_an_existing_file() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    worker.env.limits.tooling.file_bytes = 4;
    worker.job_request(
        JOB,
        Request::Write {
            owner: JOB,
            path: b"workspaces/7/request.json".as_slice().into(),
            bytes: b"large".as_slice().into(),
        },
    );
    assert_eq!(event(worker), Event::Written { owner: JOB, written: false });
    assert!(worker.submitted().is_empty());
    worker.job_request(
        JOB,
        Request::Write {
            owner: JOB,
            path: b"workspaces/7/request.json".as_slice().into(),
            bytes: b"abcd".as_slice().into(),
        },
    );
    complete(worker, Err(kernel::Error::Exists));
    assert_eq!(complete(worker, Ok(Done::Fd(Fd::new(101)))), FileOp::Open(kernel::OpenHow::WriteNoFollow));
    assert_eq!(complete(worker, Ok(Done::Nothing)), FileOp::Truncate(0));
    assert_eq!(complete(worker, Ok(Done::Count(2))), FileOp::Write(0, 0));
    assert_eq!(complete(worker, Ok(Done::Count(2))), FileOp::Write(2, 2));
    close_file(worker);
    assert_eq!(event(worker), Event::Written { owner: JOB, written: true });
    assert!(worker.jobs.empty());
}

/// temper/docs/design/protocol/tooling.md, 2 and 6: missing results and read failures are None and release their descriptors.
#[test]
fn a_missing_or_unreadable_result_is_none() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    worker.job_request(JOB, Request::Read { owner: JOB, path: b"workspaces/7/result.json".as_slice().into(), most: 4 });
    complete(worker, Err(kernel::Error::NotFound));
    assert_eq!(event(worker), Event::Read { owner: JOB, bytes: None });
    read(worker, 4);
    complete(worker, Err(kernel::Error::Permission));
    close_file(worker);
    assert_eq!(event(worker), Event::Read { owner: JOB, bytes: None });
    assert!(worker.jobs.empty());
}

/// temper/docs/design/protocol/tooling.md, 2: the real child gets only the job environment and leads its own group.
/// temper/docs/design/protocol/tooling.md, 2: abort closes a running or late-spawned job's child and pipes before Gone.
#[test]
fn an_aborted_spawn_waits_for_its_child_and_pipes_to_close() {
    for before_spawn in [false, true] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        let (owner, _) = launch(worker);
        if before_spawn {
            worker.jobs_fire(Time::ZERO, true);
        }
        worker.up(Time::ZERO, io::Event::Spawned { owner, child: CHILD, pipes: Box::new([STDOUT, STDERR]) });
        assert_eq!(event(worker), Event::Spawned { owner: JOB });
        if !before_spawn {
            for pipe in [STDOUT, STDERR] {
                assert_eq!(worker.io().pop(), Some(demand(pipe)));
            }
            worker.jobs_fire(Time::ZERO, true);
        }
        assert_eq!(
            worker.io().pop(),
            Some(io::Request::Signal {
                child: CHILD,
                signal: kernel::Signal::Kill,
                to: kernel::Target::Group,
                reason: kernel::Reason::Closed
            })
        );
        for entity in [CHILD, STDOUT, STDERR] {
            assert_eq!(worker.io().pop(), Some(io::Request::Abort { entity }));
        }
        assert!(worker.events.is_empty());
        worker.up(Time::ZERO, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
        worker.up(Time::ZERO, io::Event::Closed { owner });
        assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Signal, tail: Box::new([]) });
        assert_eq!(event(worker), Event::Gone { owner: JOB });
        assert!(worker.jobs.empty());
    }
}

/// temper/docs/design/protocol/tooling.md, 2 and 6: a zero-byte bound admits only an empty file without reading it.
#[test]
fn a_zero_byte_result_bound_reads_no_bytes() {
    for (size, expected) in [(0, Some(Box::from(b"".as_slice()))), (1, None)] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        read(worker, 0);
        assert_eq!(
            complete(
                worker,
                Ok(Done::Stat(kernel::Stat { kind: kernel::Kind::File, size, mode: 0o600, owner: 0, links: 1 }))
            ),
            FileOp::Size
        );
        close_file(worker);
        assert_eq!(event(worker), Event::Read { owner: JOB, bytes: expected });
    }
}
