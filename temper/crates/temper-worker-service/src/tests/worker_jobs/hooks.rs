//! The checkout decides whether a job can run its hook (temper/docs/design/protocol/hooks.md, 4.3).
use super::{CHILD, DEADLINE, JOB, STDERR, Worker, demand, eof, event, fixtures, released, spawned};
use crate::testing::{jobs_deadline, request_command, request_with_hooks, take_spawn};
use skein_io as io;
use skein_io::kernel::{self, Done, Error, Fd, Kind, Op, OpenHow, Stat, Submit};
use skein_lib::{Duration, Time, stream};
use temper_service_core::limits::HOOK_TIME;
use temper_worker_domain::tooling::{CONFIG_BYTES, Conflict, Event, Exit, Hooks, Request};

const ROOT: Fd = Fd::new(9);
const HOOKS: Fd = Fd::new(101);
const FILE: Fd = Fd::new(102);
const CONFIG: Fd = Fd::new(103);
const JOB_TIME: Time = at(3600);
const FALLBACK: &[u8] = b"workspaces/7/repo/tools/check";
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
}

const fn at(seconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(seconds))
}

fn discover(worker: &mut Worker, program: &[u8], verb: &[u8], hooks: Hooks) {
    request_with_hooks(worker, Box::new([program.into(), verb.into(), REQUEST.into()]), JOB_TIME, hooks);
}

fn find(worker: &mut Worker, program: &[u8], verb: &[u8], fallback: &[u8]) {
    discover(worker, program, verb, Hooks::new(Some(fallback.into()), 3600));
}

fn directory() -> Step {
    Step {
        op: Op::Open { root: ROOT, path: HOOK_DIRECTORY.into(), how: OpenHow::DirectoryNoFollow },
        result: Ok(Done::Fd(HOOKS)),
    }
}

fn config_open(result: Result<Done, Error>) -> Step {
    Step { op: Op::Open { root: HOOKS, path: b"config.toml".as_slice().into(), how: OpenHow::ReadNoFollow }, result }
}

fn open_file(name: &[u8], result: Result<Done, Error>) -> Step {
    Step { op: Op::Open { root: HOOKS, path: name.into(), how: OpenHow::PathSiblingFile }, result }
}

fn stat(kind: Kind, mode: u32) -> Step {
    Step { op: Op::Stat { fd: FILE }, result: Ok(Done::Stat(Stat { kind, mode, owner: 1000, size: 7, links: 1 })) }
}

fn close(fd: Fd) -> Step {
    Step { op: Op::Close { fd }, result: Ok(Done::Nothing) }
}

fn complete(worker: &mut Worker, submit: Submit, result: Result<Done, Error>, now: Time) {
    worker.complete(now, kernel::Complete { op: submit.op, kind: submit.kind, result }).unwrap();
}

fn step(worker: &mut Worker, row: Step, now: Time) {
    assert!(worker.io().is_empty(), "nothing spawns before the check drains");
    assert!(worker.events.is_empty(), "the check still owns the job");
    let submit = worker.submitted().pop().expect("next check operation");
    assert_eq!(submit.kind, row.op);
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
    find(worker, MERGE, b"merge", FALLBACK);
    steps(
        worker,
        [
            directory(),
            open_file(b"merge", Ok(Done::Fd(FILE))),
            stat(Kind::File, 0o651),
            close(FILE),
            config_open(Err(Error::NotFound)),
            close(HOOKS),
        ],
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
        // The kernel returns the sibling descriptor from the hook name itself.
        [
            directory(),
            open_file(b"close", Ok(Done::Fd(FILE))),
            stat(Kind::File, 0o755),
            close(FILE),
            config_open(Err(Error::NotFound)),
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
    for (inspection, why) in [
        (
            Box::<[Step]>::from([open_file(b"close", Err(Error::NotFound))]),
            b"hook `.temper/hooks/close`: missing".as_slice(),
        ),
        (
            Box::<[Step]>::from([open_file(b"close", Ok(Done::Fd(FILE))), stat(Kind::File, 0o644), close(FILE)]),
            b"hook `.temper/hooks/close`: not executable".as_slice(),
        ),
        (
            Box::<[Step]>::from([open_file(b"close", Err(Error::TooManyLinks))]),
            b"hook `.temper/hooks/close`: a link that names no regular file beside it".as_slice(),
        ),
        (
            Box::<[Step]>::from([open_file(b"close", Ok(Done::Fd(FILE))), stat(Kind::Directory, 0o755), close(FILE)]),
            b"hook `.temper/hooks/close`: not a regular file".as_slice(),
        ),
        (
            Box::<[Step]>::from([open_file(b"close", Ok(Done::Fd(FILE))), stat(Kind::Other, 0o755), close(FILE)]),
            b"hook `.temper/hooks/close`: not a regular file".as_slice(),
        ),
    ] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        request(worker, CLOSE, b"close");
        steps(worker, [directory()].into_iter().chain(inspection).chain([close(HOOKS)]));
        refused(worker, Exit::NotRun, why);
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
    steps(worker, [directory(), open_file(b"merge", Ok(Done::Fd(FILE))), stat(Kind::File, 0o755), close(FILE)]);
    step(worker, config_open(Err(Error::NotFound)), DEADLINE);
    step(worker, close(HOOKS), DEADLINE);
    refused(worker, Exit::Deadline, b"");
}

/// temper/docs/design/protocol/hooks.md, 4.3: a directory opts in even where the moment is absent.
#[test]
fn discovery_leaves_an_absent_moment_alone_even_with_a_fallback() {
    for (program, moment) in [(MERGE, b"merge".as_slice()), (CLOSE, b"close".as_slice())] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        find(worker, program, moment, FALLBACK);
        steps(worker, [directory(), open_file(moment, Err(Error::NotFound)), close(HOOKS)]);
        refused(worker, Exit::Absent, b"");
    }
}

const NO_DIRECTORY: [Error; 3] = [Error::NotFound, Error::NotADirectory, Error::TooManyLinks];

fn without_hooks(worker: &mut Worker, error: Error, fallback: &[u8], ceiling: u32) {
    discover(worker, CLOSE, b"close", Hooks::new(Some(fallback.into()), ceiling));
    step(worker, Step { result: Err(error), ..directory() }, Time::ZERO);
}

/// temper/docs/design/protocol/hooks.md, 4.3: a missing directory, file or link never opts in.
#[test]
fn a_checkout_without_a_hooks_directory_uses_its_fallback() {
    for error in NO_DIRECTORY {
        for (ceiling, duration) in [(3600, HOOK_TIME), (300, Duration::from_secs(300))] {
            let mut stand = fixtures::started_worker();
            let worker = &mut stand.worker;
            without_hooks(worker, error, FALLBACK, ceiling);
            let (_, spawn) = take_spawn(worker);
            assert_eq!(spawn.program.as_ref(), b"/worker/workspaces/7/repo/tools/check");
            assert_eq!(spawn.args[0].as_ref(), b"close");
            assert_eq!(jobs_deadline(worker), Some(Time::ZERO.saturating_add(duration)));
            assert!(worker.submitted().is_empty(), "a fallback has no hook or config check");
            assert!(worker.events.is_empty(), "a fallback has no cut line");
        }
    }
}

/// temper/docs/design/protocol/hooks.md, 4.3: absence also holds when there is no fallback.
#[test]
fn a_checkout_without_a_directory_or_fallback_answers_none() {
    for error in NO_DIRECTORY {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        without_hooks(worker, error, b"", 3600);
        refused(worker, Exit::Absent, b"");
    }
}

/// temper/docs/design/protocol/hooks.md, 4.3: a fallback is an ordinary command, not another discovery.
#[test]
fn a_missing_fallback_exits_as_a_failed_spawn() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    without_hooks(worker, Error::NotFound, FALLBACK, 3600);
    let (owner, _) = take_spawn(worker);
    worker.up(Time::ZERO, io::Event::Failed { owner, error: io::Error::Other });
    worker.up(Time::ZERO, io::Event::Closed { owner });
    refused(
        worker,
        Exit::Signal,
        b"spawn command /worker/workspaces/7/repo/tools/check in /worker/workspaces/7 failed: Other",
    );
}

/// temper/docs/design/protocol/hooks.md, 4.3: a found hook left in conflict does not run.
#[test]
fn discovery_refuses_the_moment_left_in_conflict() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    let mut hooks = Hooks::new(Some(FALLBACK.into()), 3600);
    hooks.conflict = Some(Conflict::Path);
    discover(worker, MERGE, b"merge", hooks);
    steps(worker, [directory(), open_file(b"merge", Err(Error::NotFound)), close(HOOKS)]);
    refused(worker, Exit::NotRun, b"hook `.temper/hooks/merge`: left in conflict");
}

fn checked_close(worker: &mut Worker, deadline: Time, ceiling: u32, discover: bool) {
    request_with_hooks(
        worker,
        Box::new([CLOSE.into(), b"close".as_slice().into(), REQUEST.into()]),
        deadline,
        Hooks::new(if discover { Some(Box::new([])) } else { None }, ceiling),
    );
    steps(worker, [directory(), open_file(b"close", Ok(Done::Fd(FILE))), stat(Kind::File, 0o755), close(FILE)]);
}

fn config_stat(kind: Kind, size: usize) -> Step {
    Step {
        op: Op::Stat { fd: CONFIG },
        result: Ok(Done::Stat(Stat { kind, mode: 0o644, owner: 1000, size: u64::try_from(size).unwrap(), links: 1 })),
    }
}

fn config_read(worker: &mut Worker, offset: usize, bytes: &[u8]) {
    assert!(worker.io().is_empty(), "the read precedes a spawn");
    let mut submit = worker.submitted().pop().expect("a bounded read");
    let Op::Read { fd, ref mut buf, at } = submit.kind else { panic!("read config.toml") };
    assert_eq!(fd, CONFIG);
    assert_eq!(at, u64::try_from(offset).unwrap());
    assert_eq!(buf.len(), (CONFIG_BYTES + 1).checked_sub(offset).expect("bounded offset"));
    assert!(buf.len() >= bytes.len());
    for (dst, src) in buf.iter_mut().zip(bytes) {
        *dst = *src;
    }
    complete(worker, submit, Ok(Done::Count(u32::try_from(bytes.len()).unwrap())), Time::ZERO);
}

fn config_file(worker: &mut Worker, file: Option<&[u8]>, now: Time) {
    if let Some(bytes) = file {
        steps(worker, [config_open(Ok(Done::Fd(CONFIG))), config_stat(Kind::File, bytes.len())]);
        if !bytes.is_empty() {
            // A short read does not end the file or skip any of its checks.
            config_read(worker, 0, &bytes[..1]);
            if bytes.len() > 1 {
                config_read(worker, 1, &bytes[1..]);
            }
        }
        config_read(worker, bytes.len(), b"");
        step(worker, close(CONFIG), Time::ZERO);
    } else {
        step(worker, config_open(Err(Error::NotFound)), Time::ZERO);
    }
    step(worker, close(HOOKS), now);
}

const HALF_HOUR: &[u8] = b"[close]\ntimeout_seconds = 1800\n";
const TWO_HOURS: &[u8] = b"[close]\ntimeout_seconds = 7200\n";
const CUT_WORDS: &[u8] = b"hook `.temper/hooks/close`: asks 7200 s, past the project's ceiling: runs at 3600 s";

/// temper/docs/design/protocol/hooks.md, 2.3: the checkout's time, ceiling, silent default and older spec.
#[test]
fn close_hook_time_uses_the_checkout_file_and_the_project_ceiling() {
    for (file, discover, ceiling, now, until) in [
        (Some(HALF_HOUR), true, 3600, at(2), at(1802)),
        (Some(HALF_HOUR), false, 3600, at(2), at(1802)),
        (Some(b"[merge]\ntimeout_seconds = 13".as_slice()), true, 3600, at(0), at(600)),
        (None, true, 3600, at(0), at(600)),
        (None, true, 300, at(0), at(300)),
    ] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        checked_close(worker, JOB_TIME, ceiling, discover);
        config_file(worker, file, now);
        let (_, spawn) = take_spawn(worker);
        assert_eq!(spawn.program.as_ref(), b"/worker/workspaces/7/repo/.temper/hooks/close");
        assert_eq!(jobs_deadline(worker), Some(until));
        assert!(worker.events.is_empty(), "only explicit time cuts issue a note");
    }
}

/// temper/docs/design/protocol/hooks.md, 2.3: a fault in the config file breaks the checked hook.
#[test]
fn a_faulty_hook_time_file_is_a_command_failure_before_spawn() {
    const ZERO: &[u8] = b"[close]\ntimeout_seconds = 0\n";
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    checked_close(worker, JOB_TIME, 3600, true);
    config_file(worker, Some(ZERO), Time::ZERO);
    refused(worker, Exit::NotRun, b".temper/hooks/config.toml:2: a value of 0");
}

/// temper/docs/design/protocol/hooks.md, 2.3: the optional config is a bounded regular file.
#[test]
fn an_invalid_config_entry_never_spawns_the_hook() {
    for (inspection, why) in [
        (Box::<[Step]>::from([config_open(Err(Error::TooManyLinks))]), b"not a regular file".as_slice()),
        (
            Box::<[Step]>::from([
                config_open(Ok(Done::Fd(CONFIG))),
                config_stat(Kind::File, CONFIG_BYTES + 1),
                close(CONFIG),
            ]),
            b"file too large".as_slice(),
        ),
    ] {
        let mut stand = fixtures::started_worker();
        let worker = &mut stand.worker;
        checked_close(worker, JOB_TIME, 3600, true);
        steps(worker, inspection.into_iter().chain([close(HOOKS)]));
        let tail = skein_lib::bytes::concat(&[b".temper/hooks/config.toml:0: ", why]);
        refused(worker, Exit::NotRun, &tail);
    }
}

fn timed_close(worker: &mut Worker, deadline: Time) -> skein_lib::Token {
    checked_close(worker, deadline, 3600, true);
    config_file(worker, Some(HALF_HOUR), Time::ZERO);
    let (owner, _) = take_spawn(worker);
    spawned(worker, owner);
    owner
}

fn kill_group(worker: &mut Worker, until: Time) {
    worker.jobs_fire(until, false);
    assert_eq!(
        worker.io().pop(),
        Some(io::Request::Signal {
            child: CHILD,
            signal: kernel::Signal::Kill,
            to: kernel::Target::Group,
            reason: kernel::Reason::Deadline,
        }),
    );
}

/// temper/docs/design/protocol/hooks.md, 2.3: the hook's clock kills its group and reports a command failure.
#[test]
fn a_hook_over_its_time_kills_its_group_and_keeps_stderr() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    let owner = timed_close(worker, JOB_TIME);
    worker.jobs_fire(at(1799), false);
    assert!(worker.io().is_empty());
    let until = at(1800);
    kill_group(worker, until);
    worker.up(until, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
    worker.up(until, io::Event::Stream { owner: STDERR, up: stream::Up::Bytes(b"close stderr".as_slice().into()) });
    assert_eq!(worker.io().pop(), Some(demand(STDERR)));
    eof(worker);
    let tail = b"hook `.temper/hooks/close`: ran past its 1800 s\nclose stderr".as_slice().into();
    assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Time, tail });
    released(worker, owner);
}

/// temper/docs/design/protocol/hooks.md, 2.3: an earlier job deadline still wins.
#[test]
fn an_earlier_job_deadline_kills_even_a_timed_hook() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    let deadline = at(900);
    let owner = timed_close(worker, deadline);
    kill_group(worker, deadline);
    worker.up(deadline, io::Event::Exited { owner, exit: kernel::Exit::Signal(9) });
    eof(worker);
    assert_eq!(event(worker), Event::Exited { owner: JOB, exit: Exit::Deadline, tail: Box::new([]) });
    released(worker, owner);
}

/// temper/docs/design/protocol/hooks.md, 2.3: a value above the ceiling gets the ceiling, and a cut line.
#[test]
fn an_explicit_hook_time_above_the_ceiling_reports_the_cut() {
    let mut stand = fixtures::started_worker();
    let worker = &mut stand.worker;
    checked_close(worker, JOB_TIME, 3600, true);
    config_file(worker, Some(TWO_HOURS), Time::ZERO);
    take_spawn(worker);
    assert_eq!(jobs_deadline(worker), Some(JOB_TIME));
    assert_eq!(event(worker), Event::TimeCut { owner: JOB, line: CUT_WORDS.into() });
    assert!(worker.events.is_empty());
}
