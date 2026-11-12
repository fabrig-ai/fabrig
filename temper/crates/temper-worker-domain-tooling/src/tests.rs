//! Job stories at the capability's boundary.

use alloc::boxed::Box;
use jig_domain_link::{JobBound, JobKind};
use skein_lib::bytes::copy_of;
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};

use crate::{Domain, End, Event, Exit, Invalid, Kind, Limits, Request, Spec, step, valid};

const OWNER: Token = Token::new(1);
const NOW: Time = Time::from_nanos(1_000);
const DEADLINE: Duration = Duration::from_secs(10);

fn limits() -> Limits {
    Limits { file_bytes: 64, tail_bytes: 8, path_bytes: 96, jobs: 2, path: copy_of(b"/usr/bin:/bin") }
}

fn kind(number: u32, delivers: bool) -> Kind {
    Kind {
        bound: JobBound { kind: JobKind(number), spec: 32, result: 16, deadline: Duration::from_secs(30) },
        verb: copy_of(b"check"),
        delivers,
    }
}

fn spec() -> Spec {
    Spec { command: copy_of(b"home/bin/tool"), request: copy_of(b"{request}") }
}

fn run(owner: Token, kind: u32) -> Event {
    Event::Run { owner, kind: JobKind(kind), spec: spec(), deadline: DEADLINE, directory: copy_of(b"/job") }
}

struct Harness {
    domain: Domain,
    env: Env<Limits>,
    out: Queue<Request>,
}

impl Harness {
    fn new() -> Harness {
        Self::with(limits())
    }

    fn with(limits: Limits) -> Harness {
        assert!(valid(&limits));
        Harness {
            domain: Domain::new(&limits, Box::new([kind(1, false), kind(2, true)])).unwrap(),
            env: Env { now: NOW, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
        }
    }

    fn step(&mut self, event: Event) -> List<Request> {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let mut requests = List::uncapped();
        while let Some(request) = self.out.pop() {
            requests.push(request).unwrap();
        }
        requests
    }

    fn one(&mut self, event: Event) -> Request {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let request = self.out.pop().expect("one request");
        assert!(self.out.is_empty(), "only one: {:?}", self.out);
        request
    }

    fn none(&mut self, event: Event) {
        let requests = self.step(event);
        assert!(requests.is_empty(), "{requests:?}");
    }

    fn running(&mut self, owner: Token, kind: u32) -> Request {
        assert_eq!(
            self.one(run(owner, kind)),
            Request::Write { owner, path: copy_of(b"/job/request.json"), bytes: copy_of(b"{request}") }
        );
        let spawn = self.one(Event::Written { owner, written: true });
        assert_eq!(self.one(Event::Spawned { owner }), Request::Started { owner });
        spawn
    }

    fn exited(&mut self, exit: Exit) -> Request {
        self.one(Event::Exited { owner: OWNER, exit, tail: copy_of(b"stderr") })
    }

    fn gone(&mut self, owner: Token, detail: &[u8]) {
        assert_eq!(self.one(Event::Gone { owner }), Request::Gone { owner, detail: copy_of(detail) });
        self.none(Event::Gone { owner });
    }
}

/// temper/docs/design/domain/worker/jobs.md: its command, directory and result.
#[test]
fn a_job_runs_its_command_in_its_directory_and_finishes_with_its_result_file() {
    for (number, deliver) in [(1, false), (2, true)] {
        let mut h = Harness::new();
        assert_eq!(
            h.running(OWNER, number),
            Request::Spawn {
                owner: OWNER,
                program: copy_of(b"/job/home/bin/tool"),
                args: Box::new([copy_of(b"/job/home/bin/tool"), copy_of(b"check"), copy_of(b"/job/request.json")]),
                env: Box::new([
                    (copy_of(b"PATH"), copy_of(b"/usr/bin:/bin")),
                    (copy_of(b"HOME"), copy_of(b"/job")),
                    (copy_of(b"TEMPER_JOB"), copy_of(b"/job")),
                    (copy_of(b"LANG"), copy_of(b"C.UTF-8")),
                ]),
                dir: copy_of(b"/job"),
                deadline: NOW.saturating_add(DEADLINE),
            }
        );
        assert_eq!(
            h.exited(Exit::Code(0)),
            Request::Read { owner: OWNER, path: copy_of(b"/job/result.json"), most: 16 }
        );
        assert_eq!(
            h.one(Event::Read { owner: OWNER, bytes: Some(copy_of(b"result")) }),
            Request::Ended { owner: OWNER, end: End::Finished { result: copy_of(b"result"), deliver } }
        );
        h.gone(OWNER, b"stderr");
    }
}

/// temper/docs/design/protocol/tooling.md, 4.1: a merge command is under its repository's own name.
#[test]
fn a_merge_command_runs_under_its_repository_in_the_jobs_directory() {
    let mut h = Harness::new();
    assert_eq!(
        h.one(Event::Run {
            owner: OWNER,
            kind: JobKind(2),
            spec: Spec { command: copy_of(b"fabrig/.temper/merge"), ..spec() },
            deadline: DEADLINE,
            directory: copy_of(b"/job"),
        }),
        Request::Write { owner: OWNER, path: copy_of(b"/job/request.json"), bytes: copy_of(b"{request}") }
    );
    let Request::Spawn { program, args, dir, .. } = h.one(Event::Written { owner: OWNER, written: true }) else {
        panic!("expected a spawn")
    };
    assert_eq!(program.as_ref(), b"/job/fabrig/.temper/merge");
    assert_eq!(args[0].as_ref(), program.as_ref());
    assert_eq!(dir.as_ref(), b"/job");
    assert_eq!(h.one(Event::Stop { owner: OWNER }), Request::Kill { owner: OWNER });
    h.gone(OWNER, b"");
}

/// temper/docs/design/domain/worker/jobs.md: exit 3 finishes without delivery.
#[test]
fn exit_three_finishes_and_never_delivers() {
    let mut h = Harness::new();
    h.running(OWNER, 2);
    assert_eq!(h.exited(Exit::Code(3)), Request::Read { owner: OWNER, path: copy_of(b"/job/result.json"), most: 16 });
    assert_eq!(
        h.one(Event::Read { owner: OWNER, bytes: Some(copy_of(b"conflict")) }),
        Request::Ended { owner: OWNER, end: End::Finished { result: copy_of(b"conflict"), deliver: false } }
    );
    h.gone(OWNER, b"stderr");
}

/// temper/docs/design/domain/worker/jobs.md: failures retain bounded error output.
#[test]
fn any_other_exit_fails_with_the_end_of_its_error_output() {
    for (exit, detail) in [
        (Exit::Code(1), b"exit 1: the tail".as_slice()),
        (Exit::Code(2), b"exit 2: the tail".as_slice()),
        (Exit::Code(4), b"exit 4: the tail".as_slice()),
        (Exit::Code(126), b"exit 126: the tail".as_slice()),
        (Exit::Code(127), b"exit 127: the tail".as_slice()),
        (Exit::Code(255), b"exit 255: the tail".as_slice()),
        (Exit::Signal, b"signal: the tail".as_slice()),
    ] {
        let mut h = Harness::new();
        h.running(OWNER, 1);
        assert_eq!(
            h.one(Event::Exited { owner: OWNER, exit, tail: copy_of(b"discard:the tail") }),
            Request::Ended { owner: OWNER, end: End::Failed { detail: copy_of(detail) } }
        );
        h.none(Event::Read { owner: OWNER, bytes: None });
        h.gone(OWNER, b"the tail");
    }
}

/// temper/docs/design/domain/worker/jobs.md: missing and oversized results fail.
#[test]
fn a_missing_or_oversized_result_file_fails_the_job() {
    for bytes in [None, Some(copy_of(b"seventeen bytes!!"))] {
        let mut h = Harness::new();
        h.running(OWNER, 1);
        h.exited(Exit::Code(0));
        assert_eq!(
            h.one(Event::Read { owner: OWNER, bytes }),
            Request::Ended {
                owner: OWNER,
                end: End::Failed { detail: copy_of(b"read result.json failed or past bound after exit 0: stderr") }
            }
        );
        h.gone(OWNER, b"stderr");
    }
    let mut h = Harness::new();
    h.running(OWNER, 1);
    h.exited(Exit::Code(0));
    let result = copy_of(b"sixteen bytes!!!");
    assert_eq!(result.len(), 16);
    assert_eq!(
        h.one(Event::Read { owner: OWNER, bytes: Some(result.clone()) }),
        Request::Ended { owner: OWNER, end: End::Finished { result, deliver: false } }
    );
    h.gone(OWNER, b"stderr");
}

/// temper/docs/design/domain/worker/jobs.md: protocol kills the group at its deadline.
#[test]
fn its_deadline_kills_it_and_ends_it_deadline() {
    let mut h = Harness::new();
    match h.running(OWNER, 1) {
        Request::Spawn { deadline, .. } => assert_eq!(deadline, NOW.saturating_add(DEADLINE)),
        other @ (Request::Write { .. }
        | Request::Kill { .. }
        | Request::Read { .. }
        | Request::Started { .. }
        | Request::StartRefused { .. }
        | Request::Ended { .. }
        | Request::Gone { .. }) => panic!("expected a spawn: {other:?}"),
    }
    assert_eq!(h.exited(Exit::Deadline), Request::Ended { owner: OWNER, end: End::Deadline });
    h.gone(OWNER, b"stderr");
}

/// temper/docs/design/domain/worker/jobs.md: stop waits for every process, once.
#[test]
fn a_stop_kills_it_and_says_gone_once_every_process_is_gone() {
    let mut h = Harness::new();
    h.running(OWNER, 1);
    assert_eq!(h.one(Event::Stop { owner: OWNER }), Request::Kill { owner: OWNER });
    h.none(Event::Stop { owner: OWNER });
    h.none(Event::Exited { owner: OWNER, exit: Exit::Signal, tail: copy_of(b"stopped") });
    h.gone(OWNER, b"stopped");
    h.none(Event::Stop { owner: OWNER });
}

/// temper/docs/design/domain/worker/jobs.md: no command starts after an early stop.
#[test]
fn a_stop_before_the_spawn_says_gone_at_once() {
    let mut h = Harness::new();
    h.one(run(OWNER, 1));
    assert_eq!(h.one(Event::Stop { owner: OWNER }), Request::Gone { owner: OWNER, detail: copy_of(b"") });
    h.none(Event::Written { owner: OWNER, written: true });
    h.none(Event::Stop { owner: OWNER });
    h.none(Event::Gone { owner: OWNER });
}

/// temper/docs/design/domain/worker/jobs.md: a stop also covers a spawn not yet acknowledged.
#[test]
fn a_stop_while_spawning_suppresses_started_and_ended() {
    let mut h = Harness::new();
    h.one(run(OWNER, 1));
    h.one(Event::Written { owner: OWNER, written: true });
    assert_eq!(h.one(Event::Stop { owner: OWNER }), Request::Kill { owner: OWNER });
    h.none(Event::Spawned { owner: OWNER });
    h.none(Event::Exited { owner: OWNER, exit: Exit::Code(0), tail: copy_of(b"stopped") });
    h.gone(OWNER, b"stopped");
}

/// temper/docs/design/domain/worker/jobs.md: a stopped result read ends nothing.
#[test]
fn a_stop_while_reading_suppresses_the_result() {
    let mut h = Harness::new();
    h.running(OWNER, 2);
    h.exited(Exit::Code(0));
    assert_eq!(h.one(Event::Stop { owner: OWNER }), Request::Kill { owner: OWNER });
    h.none(Event::Read { owner: OWNER, bytes: Some(copy_of(b"result")) });
    h.gone(OWNER, b"stderr");
}

/// temper/docs/design/domain/worker/jobs.md: `Gone` follows the result even if io races.
#[test]
fn gone_waits_for_the_result_read_and_is_last() {
    let mut h = Harness::new();
    h.running(OWNER, 1);
    h.exited(Exit::Code(0));
    h.none(Event::Gone { owner: OWNER });
    let requests = h.step(Event::Read { owner: OWNER, bytes: Some(copy_of(b"result")) });
    assert_eq!(
        requests.as_slice(),
        &[
            Request::Ended { owner: OWNER, end: End::Finished { result: copy_of(b"result"), deliver: false } },
            Request::Gone { owner: OWNER, detail: copy_of(b"stderr") },
        ]
    );
    h.none(Event::Gone { owner: OWNER });
    h.none(Event::Read { owner: OWNER, bytes: None });
}

/// temper/docs/design/domain/worker/jobs.md: writing failure leaves no process behind.
#[test]
fn a_failed_request_write_fails_and_says_gone() {
    let mut h = Harness::new();
    h.one(run(OWNER, 1));
    assert_eq!(
        h.step(Event::Written { owner: OWNER, written: false }).as_slice(),
        &[
            Request::Ended { owner: OWNER, end: End::Failed { detail: copy_of(b"write request.json failed") } },
            Request::Gone { owner: OWNER, detail: copy_of(b"write request.json failed") },
        ]
    );
    h.none(Event::Written { owner: OWNER, written: true });
}

/// temper/docs/design/domain/worker/jobs.md: admission checks kinds and bounds.
#[test]
fn a_run_of_a_kind_not_entered_or_past_its_bounds_is_refused() {
    for (number, request, deadline, why) in [
        (3, copy_of(b"ok"), DEADLINE, b"kind".as_slice()),
        (1, skein_lib::bytes::zeroed(33), DEADLINE, b"request".as_slice()),
        (1, skein_lib::bytes::zeroed(65), DEADLINE, b"request".as_slice()),
        (1, copy_of(b"ok"), Duration::from_secs(31), b"deadline".as_slice()),
    ] {
        let mut h = Harness::new();
        let event = Event::Run {
            owner: OWNER,
            kind: JobKind(number),
            spec: Spec { request, ..spec() },
            deadline,
            directory: copy_of(b"/job"),
        };
        match h.one(event) {
            Request::StartRefused { owner, why: detail } => {
                assert_eq!(owner, OWNER);
                assert!(detail.starts_with(why));
            }
            other @ (Request::Write { .. }
            | Request::Spawn { .. }
            | Request::Kill { .. }
            | Request::Read { .. }
            | Request::Started { .. }
            | Request::Ended { .. }
            | Request::Gone { .. }) => panic!("expected refusal: {other:?}"),
        }
        h.none(Event::Gone { owner: OWNER });
    }
    let mut h = Harness::new();
    let request = skein_lib::bytes::zeroed(32);
    assert_eq!(
        h.one(Event::Run {
            owner: OWNER,
            kind: JobKind(1),
            spec: Spec { request: request.clone(), ..spec() },
            deadline: Duration::from_secs(30),
            directory: copy_of(b"/job"),
        }),
        Request::Write { owner: OWNER, path: copy_of(b"/job/request.json"), bytes: request }
    );
}

/// temper/docs/design/domain/worker/jobs.md: paths cannot escape the job's directory.
#[test]
fn unsafe_or_oversized_command_paths_are_refused() {
    for command in [b"..".as_slice(), b"../tool", b"bin/../tool", b"bin/..", b"/tool", b"to\0ol", b""] {
        let mut h = Harness::new();
        assert_eq!(
            h.one(Event::Run {
                owner: OWNER,
                kind: JobKind(2),
                spec: Spec { command: copy_of(command), ..spec() },
                deadline: DEADLINE,
                directory: copy_of(b"/job"),
            }),
            Request::StartRefused { owner: OWNER, why: copy_of(b"command path") }
        );
    }
    for path_bytes in [17, 18] {
        let mut h = Harness::with(Limits { path_bytes, ..limits() });
        match h.one(run(OWNER, 1)) {
            Request::StartRefused { why, .. } => {
                assert_eq!(path_bytes, 17);
                assert_eq!(why.as_ref(), b"command path bound");
            }
            Request::Write { .. } => assert_eq!(path_bytes, 18),
            other @ (Request::Spawn { .. }
            | Request::Kill { .. }
            | Request::Read { .. }
            | Request::Started { .. }
            | Request::Ended { .. }
            | Request::Gone { .. }) => panic!("expected path boundary: {other:?}"),
        }
    }
}

/// temper/docs/design/domain/worker/jobs.md: every command and directory is bounded.
#[test]
fn directories_and_generated_paths_are_checked_before_writing() {
    for directory in [copy_of(b""), copy_of(b"job\0"), skein_lib::bytes::zeroed(97)] {
        let mut h = Harness::new();
        assert_eq!(
            h.one(Event::Run { owner: OWNER, kind: JobKind(1), spec: spec(), deadline: DEADLINE, directory }),
            Request::StartRefused { owner: OWNER, why: copy_of(b"directory path") }
        );
    }
    let mut h = Harness::with(Limits { path_bytes: 15, ..limits() });
    assert_eq!(
        h.one(Event::Run {
            owner: OWNER,
            kind: JobKind(1),
            spec: Spec { command: copy_of(b"a"), ..spec() },
            deadline: DEADLINE,
            directory: copy_of(b"/job"),
        }),
        Request::StartRefused { owner: OWNER, why: copy_of(b"request path bound") }
    );
}

/// temper/docs/design/domain/worker/jobs.md: jobs are independent and bounded in flight.
#[test]
fn a_busy_owner_and_a_full_capability_refuse_without_changing_live_jobs() {
    let mut h = Harness::with(Limits { jobs: 1, ..limits() });
    h.running(OWNER, 1);
    assert_eq!(h.one(run(OWNER, 2)), Request::StartRefused { owner: OWNER, why: copy_of(b"owner already in flight") });
    let other = Token::new(2);
    assert_eq!(h.one(run(other, 1)), Request::StartRefused { owner: other, why: copy_of(b"jobs bound") });
    h.exited(Exit::Code(0));
    assert_eq!(
        h.one(Event::Read { owner: OWNER, bytes: Some(copy_of(b"ok")) }),
        Request::Ended { owner: OWNER, end: End::Finished { result: copy_of(b"ok"), deliver: false } }
    );
    h.gone(OWNER, b"stderr");
    h.one(run(other, 2));
    assert_eq!(h.one(Event::Stop { owner: other }), Request::Gone { owner: other, detail: copy_of(b"") });
}

/// temper/docs/design/domain/worker/jobs.md: kinds are declared once, in order.
#[test]
fn bounds_list_the_kinds_ascending() {
    let h = Harness::new();
    assert_eq!(h.domain.bounds().as_ref(), &[kind(1, false).bound, kind(2, true).bound]);
    let domain = Domain::new(&limits(), Box::new([])).unwrap();
    assert!(domain.bounds().is_empty());
}

/// temper/docs/design/domain/worker/jobs.md: unordered and repeated kinds are invalid.
#[test]
fn new_refuses_kinds_out_of_order() {
    for entries in [Box::new([kind(2, false), kind(1, false)]), Box::new([kind(1, false), kind(1, true)])] {
        assert_eq!(Domain::new(&limits(), entries).unwrap_err(), Invalid::Kinds);
    }
}

/// temper/docs/design/domain/worker/jobs.md: invalid entries cannot be advertised.
#[test]
fn new_checks_verbs_file_bounds_and_kind_count() {
    for verb in [b"".as_slice(), b"a/b", b"a\0b"] {
        let entry = Kind { verb: copy_of(verb), ..kind(1, false) };
        assert_eq!(Domain::new(&limits(), Box::new([entry])).unwrap_err(), Invalid::Verb);
    }
    for (spec, result, failure) in [(65, 16, Invalid::Spec), (32, 65, Invalid::Result)] {
        let entry = kind(1, false);
        let entry = Kind { bound: JobBound { spec, result, ..entry.bound }, ..entry };
        assert_eq!(Domain::new(&limits(), Box::new([entry])).unwrap_err(), failure);
    }
    let mut kinds = List::with_capacity(65_536);
    for number in 0..65_536 {
        kinds.push(kind(number, false)).unwrap();
    }
    assert_eq!(Domain::new(&limits(), kinds.into_boxed()).unwrap_err(), Invalid::Kinds);
}

/// temper/docs/design/domain/worker/jobs.md: every configurable bound is nonzero.
#[test]
fn limits_refuse_zeros() {
    for invalid in [
        Limits { file_bytes: 0, ..limits() },
        Limits { tail_bytes: 0, ..limits() },
        Limits { path_bytes: 0, ..limits() },
        Limits { jobs: 0, ..limits() },
        Limits { path: copy_of(b""), ..limits() },
    ] {
        assert!(!valid(&invalid));
        assert_eq!(Domain::new(&invalid, Box::new([])).unwrap_err(), Invalid::Limits);
    }
}

/// temper/docs/design/domain/worker/jobs.md: a failed setup names its step.
#[test]
fn failed_setup_names_the_request_write_or_process_release() {
    let mut h = Harness::new();
    h.one(run(OWNER, 1));
    let rows = h.step(Event::Written { owner: OWNER, written: false });
    assert_eq!(
        rows.as_slice(),
        &[
            Request::Ended { owner: OWNER, end: End::Failed { detail: copy_of(b"write request.json failed") } },
            Request::Gone { owner: OWNER, detail: copy_of(b"write request.json failed") },
        ]
    );
    let mut h = Harness::new();
    h.running(OWNER, 1);
    let rows = h.step(Event::Gone { owner: OWNER });
    assert_eq!(
        rows.as_slice().first(),
        Some(&Request::Ended {
            owner: OWNER,
            end: End::Failed { detail: copy_of(b"process gone without exit status: ") }
        })
    );
}

/// temper/docs/design/domain/worker/jobs.md: exit 3 still requires its result file.
#[test]
fn exit_three_with_no_result_keeps_its_status_and_stderr() {
    let mut h = Harness::new();
    h.running(OWNER, 1);
    h.exited(Exit::Code(3));
    assert_eq!(
        h.one(Event::Read { owner: OWNER, bytes: None }),
        Request::Ended {
            owner: OWNER,
            end: End::Failed { detail: copy_of(b"read result.json failed or past bound after exit 3: stderr") }
        }
    );
}
