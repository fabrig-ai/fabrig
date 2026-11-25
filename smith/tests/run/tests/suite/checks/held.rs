//! A check's held clock across the run and machine boundary (domain/run-delivery.md, 1.1).

use super::*;
use skein_io::{Event as IoEvent, Request as IoRequest, kernel};
use skein_lib::{Rng, stream::Up};
use skein_world::streams::{Stream, Streams};
use smith_protocol_machine::{Below, BelowEvent, Component, FromDomain, ToDomain};

const CHILD: Token = Token::new(40);
const STDOUT: Token = Token::new(41);
const STDERR: Token = Token::new(42);

/// Draw a marker and its read splits; faults choose which pipe it reaches.
pub(crate) fn draw_marker(workload: &mut Rng, faults: &mut Rng) -> (Vec<Vec<u8>>, bool) {
    let bytes = match workload.below(8) {
        0 | 1 => b"smith-check: waiting\n".to_vec(),
        2 | 3 => b"smith-check: admitted\n".to_vec(),
        4 => b"smith-check: waiting now\n".to_vec(),
        5 => b"smith-check: waiting".to_vec(),
        6 => b"\n".to_vec(),
        7 => vec![b'x'; 1024],
        _ => unreachable!("eight draws"),
    };
    let stderr = !faults.chance(200);
    let mut chunks = Vec::new();
    let mut from = 0;
    while from < bytes.len() {
        let to = (from + usize::try_from(workload.between(1, 24)).expect("small chunk")).min(bytes.len());
        chunks.push(bytes[from..to].to_vec());
        from = to;
    }
    (chunks, stderr)
}

#[test]
fn check_workload_is_independent_of_prior_fault_draws() {
    let streams = Streams::new(0);
    let mut workload = streams.rng(Stream::Workload, "runs");
    let mut faults = streams.rng(Stream::Faults, "checks");
    let expected = (0..100).map(|_| draw_marker(&mut workload, &mut faults).0).collect::<Vec<_>>();
    faults.next_u64();
    let mut workload = streams.rng(Stream::Workload, "runs");
    let after_faults = (0..100).map(|_| draw_marker(&mut workload, &mut faults).0).collect::<Vec<_>>();
    assert!(after_faults == expected, "fault draws cannot change the markers or their read splits");
}

pub(crate) struct Check {
    run: PendingCheck,
    machine: Component,
    env: Env<smith_protocol_machine::Limits>,
    above: Queue<ToDomain>,
    below: Queue<Below>,
}

impl Check {
    pub(crate) fn new(call_seconds: u64, run_seconds: u64) -> Self {
        let mut run = PendingCheck::starting(call_seconds, run_seconds);
        let request = run.request.take().expect("first check");
        Self::start(run, request)
    }

    fn start(run: PendingCheck, request: Request) -> Self {
        let Request::Check { owner, program, results, bound, cap, tail } = request else { panic!("check") };
        let limits = smith_protocol_machine::Limits { output_bytes: tail, ..smith_protocol_machine_fixtures::SMALL };
        let mut machine = Component::new(
            &limits,
            smith_protocol_machine::Environment::encode(&[], &limits).expect("environment"),
            Box::from(b"/usr/bin/rg".as_slice()),
        );
        machine.workspace(&run::Workspace {
            directories: Box::new([run::Directory {
                results: None,
                host: b"/host".as_slice().into(),
                name: b"code".as_slice().into(),
                root: program.root,
                writable: true,
                git: true,
                conflicts: Box::new([]),
            }]),
        });
        let mut check = Self {
            env: Env { now: run.env.now, wall: Wall::EPOCH, limits },
            run,
            machine,
            above: Queue::uncapped(),
            below: Queue::uncapped(),
        };
        check.machine.from_domain(
            &check.env,
            FromDomain::Check { owner, program, results, bound, cap, tail },
            &mut check.above,
            &mut check.below,
        );
        assert!(matches!(check.below.pop(), Some(Below::Spawn { .. })));
        check.event(BelowEvent::Process(IoEvent::Spawned { owner, child: CHILD, pipes: Box::new([STDOUT, STDERR]) }));
        check.clear_reads();
        check
    }

    fn event(&mut self, event: BelowEvent) {
        self.machine.from_below(&self.env, event, &mut self.above, &mut self.below);
    }

    pub(crate) fn deadline(&self) -> Time {
        self.machine.next_deadline().expect("live check deadline")
    }

    pub(crate) fn at(&mut self, seconds: u64) {
        self.env.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        self.run.env.now = self.env.now;
    }

    pub(crate) fn bytes(&mut self, pipe: Token, bytes: &[u8]) {
        self.event(BelowEvent::Pipe(IoEvent::Stream { owner: pipe, up: Up::Bytes(bytes.into()) }));
        self.clear_reads();
    }

    fn clear_reads(&mut self) {
        while let Some(request) = self.below.pop() {
            assert!(matches!(request, Below::Process(IoRequest::Stream { .. })), "unexpected stop: {request:?}");
        }
    }

    fn fire(&mut self) {
        self.machine.fire(&self.env, &mut self.above, &mut self.below);
    }

    fn assert_stopped(&mut self) {
        assert_eq!(
            self.below.pop(),
            Some(Below::Process(IoRequest::Signal {
                child: CHILD,
                to: kernel::Target::Group,
                signal: kernel::Signal::Terminate,
                reason: kernel::Reason::Unknown,
            }))
        );
        assert!(self.below.is_empty());
    }

    pub(crate) fn settle_at_deadline(mut self, seconds: u64) {
        self.at(seconds);
        self.fire();
        self.assert_stopped();
        let (_, requests) = self.finish();
        assert!(matches!(requests.first(), Some(Request::Check { .. } | Request::Return { .. })), "{requests:?}");
    }

    fn finish(mut self) -> (PendingCheck, Vec<Request>) {
        self.event(BelowEvent::Process(IoEvent::Exited { owner: self.run.owner, exit: kernel::Exit::Code(0) }));
        for pipe in [STDOUT, STDERR] {
            self.event(BelowEvent::Pipe(IoEvent::Closed { owner: pipe }));
        }
        self.event(BelowEvent::Process(IoEvent::Closed { owner: self.run.owner }));
        let Some(ToDomain::Checked { owner, ran }) = self.above.pop() else { panic!("checked") };
        assert!(self.above.is_empty());
        self.machine.close(&mut self.below);
        assert!(self.machine.closed());
        let requests = self.run.step(Event::Checked { owner, ran });
        (self.run, requests)
    }
}

fn time(seconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(seconds))
}

fn assert_feedback(requests: &[Request], seconds: u64) {
    let [Request::Return { result: Returned::ChecksFailed { given, ran, .. }, .. }, rest @ ..] = requests else {
        panic!("timeout feedback without a retry: {requests:?}");
    };
    assert!(rest.iter().all(|request| matches!(request, Request::Close { .. })), "no retry: {rest:?}");
    assert_eq!(*given, Duration::from_secs(seconds));
    assert_eq!(ran.exit, Exit::TimedOut);
}

#[test]
fn a_check_waiting_for_a_place_delivers_with_its_held_time_left() {
    let mut check = Check::new(500, 600);
    check.at(1);
    check.bytes(STDERR, b"smith-check: waiting\n");
    check.at(60);
    check.fire();
    assert!(check.below.is_empty(), "the wait must not spend the 60-second bound: {:?}", check.below);
    check.at(100);
    check.bytes(STDERR, b"smith-check: admitted\n");
    assert_eq!(check.deadline(), time(159));
    check.at(130);
    let (mut run, requests) = check.finish();
    let [Request::Deliver { owner, .. }] = requests.as_slice() else { panic!("delivery: {requests:?}") };
    let requests = run.step(Event::Delivered {
        owner: *owner,
        delivery: run::Delivery::Delivered(
            run::Delivered::new(Box::new([run::Receipt::new(0, b"saved".as_slice().into()).expect("receipt")]))
                .expect("delivery"),
        ),
    });
    assert!(matches!(requests.first(), Some(Request::Return { result: Returned::Delivered(_), .. })), "{requests:?}");
}

#[test]
fn a_resumed_check_times_out_at_159_and_reports_60_seconds() {
    let mut check = Check::new(200, 600);
    check.at(1);
    check.bytes(STDERR, b"smith-check: waiting\n");
    check.at(100);
    check.bytes(STDERR, b"smith-check: admitted\n");
    assert_eq!(check.deadline(), time(159));
    check.at(158);
    check.fire();
    assert!(check.below.is_empty());
    check.at(159);
    check.fire();
    check.assert_stopped();
    // A late marker while stopping is output, not another clock transition.
    check.at(160);
    check.bytes(STDERR, b"smith-check: waiting\n");
    assert_eq!(check.deadline(), time(159).saturating_add(check.env.limits.group_stop));
    check.at(170);
    let (_, requests) = check.finish();
    assert_feedback(&requests, 60);
}

#[test]
fn an_unadmitted_check_times_out_at_the_call_or_run_cap_without_retry() {
    for (call_seconds, run_seconds) in [(150, 600), (500, 150)] {
        let mut check = Check::new(call_seconds, run_seconds);
        check.at(1);
        check.bytes(STDERR, b"smith-check: waiting\n");
        assert_eq!(check.deadline(), time(150));
        check.at(149);
        check.fire();
        assert!(check.below.is_empty());
        check.at(150);
        check.fire();
        check.assert_stopped();
        let (_, requests) = check.finish();
        assert_feedback(&requests, 60);
    }
}

#[test]
fn only_exact_newline_ended_stderr_markers_change_a_checks_clock() {
    for (pipe, bytes) in [
        (STDERR, b"smith-check: admitted\n".as_slice()),
        (STDOUT, b"smith-check: waiting\n".as_slice()),
        (STDERR, b"smith-check: waiting now\n".as_slice()),
        (STDERR, b"smith-check: waiting\r\n".as_slice()),
        (STDERR, b"xsmith-check: waiting\n".as_slice()),
        (STDERR, b"smith-check: waiting".as_slice()),
    ] {
        let mut check = Check::new(200, 600);
        check.at(1);
        check.bytes(pipe, bytes);
        assert_eq!(check.deadline(), time(60), "pipe {pipe:?}, bytes {bytes:?}");
        check.at(60);
        check.fire();
        check.assert_stopped();
        let (_, requests) = check.finish();
        // This timeout has room to retry; it must not have become a success.
        assert!(matches!(requests.first(), Some(Request::Check { .. })), "{requests:?}");
    }
}

#[test]
fn split_markers_count_at_the_newline_and_duplicate_waits_change_nothing() {
    let mut check = Check::new(200, 600);
    check.at(1);
    check.bytes(STDERR, b"smith-check: wait");
    assert_eq!(check.deadline(), time(60));
    check.at(2);
    check.bytes(STDERR, b"ing\n");
    assert_eq!(check.deadline(), time(200));
    check.at(50);
    check.bytes(STDERR, b"smith-check: waiting\n");
    assert_eq!(check.deadline(), time(200));
    check.at(100);
    check.bytes(STDERR, b"smith-check: admitted");
    assert_eq!(check.deadline(), time(200));
    check.at(105);
    check.bytes(STDERR, b"\n");
    assert_eq!(check.deadline(), time(163));
    check.at(106);
    check.bytes(STDERR, b"smith-check: admitted\n");
    assert_eq!(check.deadline(), time(163));
    check.at(130);
    let (_, requests) = check.finish();
    assert!(matches!(requests.first(), Some(Request::Deliver { .. })), "{requests:?}");
}

#[test]
fn a_waited_timeout_retries_with_twice_the_held_bound_and_omits_its_own_wait() {
    let mut check = Check::new(500, 600);
    check.at(1);
    check.bytes(STDERR, b"smith-check: waiting\n");
    check.at(100);
    check.bytes(STDERR, b"smith-check: admitted\n");
    check.at(159);
    check.fire();
    check.assert_stopped();
    let (run, mut requests) = check.finish();
    let [Request::Check { bound, cap, .. }, Request::Checking { deadline, .. }] = requests.as_slice() else {
        panic!("retry: {requests:?}");
    };
    assert_eq!(*bound, Duration::from_secs(120));
    assert_eq!((*cap, *deadline), (time(500), time(279)));
    let mut retry = Check::start(run, requests.remove(0));
    retry.at(160);
    retry.bytes(STDERR, b"smith-check: waiting\n");
    retry.at(279);
    retry.fire();
    assert!(retry.below.is_empty());
    retry.at(300);
    retry.bytes(STDERR, b"smith-check: admitted\n");
    assert_eq!(retry.deadline(), time(419));
    retry.at(400);
    let (_, requests) = retry.finish();
    assert!(matches!(requests.first(), Some(Request::Deliver { .. })), "{requests:?}");
}
