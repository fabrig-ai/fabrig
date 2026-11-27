//! The checkout decides whether a job can run its hook (temper/docs/design/protocol/hooks.md, 4.3).
use super::{DEADLINE, JOB, Worker, event, fixtures};
use crate::testing::{request_command, take_spawn};
use skein_io::kernel::{self, Done, Error, Fd, Kind, Op, OpenHow, Stat, Submit};
use skein_lib::Time;
use temper_worker_domain::tooling::{Event, Exit, Request};

const ROOT: Fd = Fd::new(9);
const HOOKS: Fd = Fd::new(101);
const FILE: Fd = Fd::new(102);
const HOOK_DIRECTORY: &[u8] = b"workspaces/7/repo/.temper/hooks";
const MERGE: &[u8] = b"workspaces/7/repo/.temper/hooks/merge";
const CLOSE: &[u8] = b"workspaces/7/repo/.temper/hooks/close";
const REQUEST: &[u8] = b"workspaces/7/request.json";

fn request(worker: &mut Worker, program: &[u8], verb: &[u8]) {
    request_command(worker, Box::new([program.into(), verb.into(), REQUEST.into()]), DEADLINE);
}

struct Step {
    op: Op,
    result: Result<Done, Error>,
    target: &'static [u8],
}

fn directory() -> Step {
    Step {
        op: Op::Open { root: ROOT, path: HOOK_DIRECTORY.into(), how: OpenHow::DirectoryNoFollow },
        result: Ok(Done::Fd(HOOKS)),
        target: b"",
    }
}

fn read_link(name: &[u8], result: Result<Done, Error>, target: &'static [u8]) -> Step {
    Step {
        op: Op::ReadLink { dir: HOOKS, name: name.into(), buf: skein_lib::bytes::zeroed(kernel::LINK_BYTES) },
        result,
        target,
    }
}

fn a_file(name: &[u8]) -> Step {
    read_link(name, Err(Error::NotALink), b"")
}

fn a_link(name: &[u8], target: &'static [u8]) -> Step {
    read_link(name, Ok(Done::Count(u32::try_from(target.len()).unwrap())), target)
}

fn open_file(name: &[u8]) -> Step {
    Step {
        op: Op::Open { root: HOOKS, path: name.into(), how: OpenHow::PathNoFollow },
        result: Ok(Done::Fd(FILE)),
        target: b"",
    }
}

fn stat(kind: Kind, mode: u32) -> Step {
    Step {
        op: Op::Stat { fd: FILE },
        result: Ok(Done::Stat(Stat { kind, mode, owner: 1000, size: 7, links: 1 })),
        target: b"",
    }
}

fn close(fd: Fd) -> Step {
    Step { op: Op::Close { fd }, result: Ok(Done::Nothing), target: b"" }
}

fn complete(worker: &mut Worker, submit: Submit, result: Result<Done, Error>, now: Time) {
    worker.complete(now, kernel::Complete { op: submit.op, kind: submit.kind, result }).unwrap();
}

fn step(worker: &mut Worker, row: Step, now: Time) {
    assert!(worker.io().is_empty(), "nothing spawns before the check drains");
    assert!(worker.events.is_empty(), "the check still owns the job");
    let mut submit = worker.submitted().pop().expect("next check operation");
    assert_eq!(submit.kind, row.op);
    if let Op::ReadLink { buf, .. } = &mut submit.kind {
        for (slot, byte) in buf.iter_mut().zip(row.target) {
            *slot = *byte;
        }
    }
    complete(worker, submit, row.result, now);
}

fn steps(worker: &mut Worker, rows: impl IntoIterator<Item = Step>) {
    for row in rows {
        step(worker, row, Time::ZERO);
    }
}

/// temper/docs/design/protocol/hooks.md, 2.2 and 4.3.
#[test]
fn a_merge_hook_runs_after_its_executable_is_checked() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    request(worker, MERGE, b"merge");
    steps(
        worker,
        [directory(), a_file(b"merge"), open_file(b"merge"), stat(Kind::File, 0o651), close(FILE), close(HOOKS)],
    );
    let (_, spawn) = take_spawn(worker);
    assert_eq!(spawn.program.as_ref(), b"/worker/workspaces/7/repo/.temper/hooks/merge");
    assert_eq!(spawn.dir.as_ref(), b"workspaces/7");
    assert_eq!(
        spawn.args.as_ref(),
        [Box::from(b"merge".as_slice()), Box::from(b"/worker/workspaces/7/request.json".as_slice())]
    );
}

/// temper/docs/design/protocol/hooks.md, 2.2: a link names exactly one sibling.
#[test]
fn a_close_hook_can_link_to_an_executable_sibling() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    request(worker, CLOSE, b"close");
    steps(
        worker,
        [
            directory(),
            a_link(b"close", b"checked"),
            open_file(b"checked"),
            stat(Kind::File, 0o755),
            close(FILE),
            close(HOOKS),
        ],
    );
    let (_, spawn) = take_spawn(worker);
    assert_eq!(spawn.program.as_ref(), b"/worker/workspaces/7/repo/.temper/hooks/close");
}

fn refused(worker: &mut Worker, exit: Exit, tail: &[u8]) {
    assert!(worker.io().is_empty(), "no child or signal");
    assert!(worker.submitted().is_empty(), "no further operation");
    assert_eq!(event(worker), Event::Exited { owner: JOB, exit, tail: tail.into() });
    assert_eq!(event(worker), Event::Gone { owner: JOB });
    assert!(worker.events.is_empty(), "one terminal pair");
    assert!(worker.jobs.empty(), "the check released its owner");
}

/// temper/docs/design/protocol/hooks.md, 2.2 and 4.3: a bad entry fails, never a fallback or no hook.
#[test]
fn an_invalid_hook_fails_and_spawns_nothing() {
    for (program, verb, inspection, tail) in [
        (
            MERGE,
            b"merge",
            Box::<[Step]>::from([read_link(b"merge", Err(Error::NotFound), b"")]),
            b"hook `.temper/hooks/merge`: missing".as_slice(),
        ),
        (
            MERGE,
            b"merge",
            Box::<[Step]>::from([a_file(b"merge"), open_file(b"merge"), stat(Kind::File, 0o644), close(FILE)]),
            b"hook `.temper/hooks/merge`: not executable".as_slice(),
        ),
        (
            CLOSE,
            b"close",
            Box::<[Step]>::from([a_link(b"close", b"../close.sh")]),
            b"hook `.temper/hooks/close`: a link to `../close.sh`, not to an executable file beside it".as_slice(),
        ),
        (
            CLOSE,
            b"close",
            Box::<[Step]>::from([a_link(b"close", b"sub/checked")]),
            b"hook `.temper/hooks/close`: a link to `sub/checked`, not to an executable file beside it".as_slice(),
        ),
        (
            CLOSE,
            b"close",
            Box::<[Step]>::from([
                a_link(b"close", b"checked"),
                open_file(b"checked"),
                stat(Kind::Symlink, 0o777),
                close(FILE),
            ]),
            b"hook `.temper/hooks/close`: link target is another link".as_slice(),
        ),
        (
            MERGE,
            b"merge",
            Box::<[Step]>::from([a_file(b"merge"), open_file(b"merge"), stat(Kind::Directory, 0o755), close(FILE)]),
            b"hook `.temper/hooks/merge`: not a regular file".as_slice(),
        ),
    ] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        request(worker, program, verb);
        steps(worker, [directory()].into_iter().chain(inspection).chain([close(HOOKS)]));
        refused(worker, Exit::NotRun, tail);
    }
}

/// temper/docs/design/domain/worker/jobs.md, 2: non-hook commands use the original spawn, whatever the name points to.
#[test]
fn a_legacy_merge_command_spawns_without_a_file_check() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    request(worker, b"workspaces/7/repo/.temper/merge", b"merge");
    let (_, spawn) = take_spawn(worker);
    assert_eq!(spawn.program.as_ref(), b"/worker/workspaces/7/repo/.temper/merge");
    assert!(worker.submitted().is_empty(), "even a legacy link is left for exec to follow");
}

/// temper/docs/design/protocol/tooling.md, 2: cancellation waits for the check's io, and there is no late spawn.
#[test]
fn a_cancelled_hook_check_closes_a_late_open() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    request(worker, CLOSE, b"close");
    let open = worker.submitted().pop().expect("directory open in flight");
    worker.job_request(JOB, Request::Kill { owner: JOB });
    let cancel = worker.submitted().pop().expect("cancel the pending open");
    assert_eq!(cancel.kind, Op::Cancel { target: open.op });
    complete(worker, open, Ok(Done::Fd(HOOKS)), Time::ZERO);
    step(worker, close(HOOKS), Time::ZERO);
    assert!(worker.events.is_empty(), "the cancellation still owns the check");
    complete(worker, cancel, Err(Error::TooLate), Time::ZERO);
    refused(worker, Exit::Signal, b"");
}

/// temper/docs/design/protocol/tooling.md, 2: a check completing at the deadline must not start the command.
#[test]
fn a_hook_checked_at_its_deadline_does_not_spawn() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    request(worker, MERGE, b"merge");
    steps(worker, [directory(), a_file(b"merge"), open_file(b"merge"), stat(Kind::File, 0o755), close(FILE)]);
    step(worker, close(HOOKS), DEADLINE);
    refused(worker, Exit::Deadline, b"");
}
