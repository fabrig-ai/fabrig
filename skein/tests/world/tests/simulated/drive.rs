//! The shipped loop over scripted kernel records and clock readings (skein/docs/design/runtime/shell.md, 8 and 11).

use std::cell::RefCell;
use std::slice;

use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Op, ServiceSignal, Signalled, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Host, Now, Turns, Wait, drive_with};
use skein_world_tests::hosted::{Act, Script};

#[test]
fn worlds_drain_once_per_iteration_in_the_simulator() {
    let mut simulated = skein_world::World::new(
        7,
        skein_sim::Config::calm(),
        skein_world_tests::hosted::Judge,
        skein_world::Memory::Unchecked,
    );
    simulated.spawn(|| Script::new(&[]));
    let outcome = simulated.run();
    assert_eq!(outcome.procs[0].iterations, outcome.procs[0].drains);
    assert!(outcome.procs[0].drains > 0);
}

// One clock script: start, I/O wake, then the two candidate deadlines.
const START: Time = Time::from_nanos(10);
const READY: Time = Time::from_nanos(20);
const EARLY: Time = Time::from_nanos(30);
const LATE: Time = Time::from_nanos(50);
const WALL: Wall = Wall::from_nanos(100);
const OFFSET: Duration = Duration::from_nanos(1000);
const NATIVE_EARLY: Time = Time::from_nanos(1030);
const NATIVE_LATE: Time = Time::from_nanos(1050);

// A pipe and a signal source in the scripted descriptor table.
const READ: Fd = Fd::new(3);
const WRITE: Fd = Fd::new(4);
const SIGNALS: Fd = Fd::new(5);
const HOOK_BYTES: &[u8] = b"hook";
const READ_BYTES: usize = 32; // Script's pipe read window.
const MAX_OPERATIONS: u32 = 3; // The closing host closes all three descriptors at once.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Call {
    Reap,
    Now(Now),
    Iterate(Now),
    Drain,
    Until(Option<Time>),
    Submit(Wait),
    InFlight(u32),
}

type Trace = RefCell<Vec<Call>>;

// One submit accepts these records, then returns with the next clock reading and
// the listed operations ready to reap. Unanswered operations stay in flight.
struct Wake<'a> {
    at: Time,
    records: &'a [Submit],
    replies: &'a [Reply],
}

enum Reply {
    Done(Token, Result<Done, Error>),
    Read(Token, &'static [u8]),
}

struct ScriptedTurns<'a> {
    trace: &'a Trace,
    wakes: slice::Iter<'a, Wake<'a>>,
    now: Time,
    pending: Vec<Submit>,
    ready: Queue<Complete>,
}

impl Turns for ScriptedTurns<'_> {
    fn reap(&mut self, out: &mut Queue<Complete>) {
        self.trace.borrow_mut().push(Call::Reap);
        while let Some(complete) = self.ready.pop() {
            out.push(complete);
        }
    }

    fn now(&self) -> Now {
        let now = Now { now: self.now, wall: WALL };
        self.trace.borrow_mut().push(Call::Now(now));
        now
    }

    fn until(&self, deadline: Option<Time>) -> Option<Time> {
        self.trace.borrow_mut().push(Call::Until(deadline));
        deadline.map(|at| at.saturating_add(OFFSET))
    }

    fn in_flight(&self) -> u32 {
        let count = u32::try_from(self.pending.len()).expect("small script") + self.ready.len();
        self.trace.borrow_mut().push(Call::InFlight(count));
        count
    }

    fn submit(&mut self, records: &mut Queue<Submit>, wait: Wait) {
        self.trace.borrow_mut().push(Call::Submit(wait));
        let wake = self.wakes.next().expect("no submit after the script ends");
        for expected in wake.records {
            let record = records.pop().expect("the host submitted the scripted record");
            assert_eq!(&record, expected);
            self.pending.push(record);
        }
        assert!(records.is_empty(), "the host submits only the scripted records");
        for reply in wake.replies {
            let (token, result) = match reply {
                Reply::Done(token, result) => (*token, result.clone()),
                Reply::Read(token, bytes) => (*token, Ok(Done::Count(u32::try_from(bytes.len()).expect("small read")))),
            };
            let index = self.pending.iter().position(|record| record.op == token).expect("reply to a pending record");
            let Submit { op, mut kind } = self.pending.remove(index);
            if let Reply::Read(_, bytes) = reply {
                let Op::PipeRead { buf, .. } = &mut kind else { panic!("read reply needs a pipe buffer") };
                buf[..bytes.len()].copy_from_slice(bytes);
            }
            self.ready.push(Complete { op, kind, result });
        }
        self.now = wake.at;
    }
}

// Observe the host in the same transcript as its kernel and clock; give its
// pause independent mechanism and policy deadlines when testing wait selection.
struct Observed<'a> {
    trace: &'a Trace,
    script: &'a mut Script,
    mechanism: Option<Time>,
    policy: Option<Time>,
}

impl Host for Observed<'_> {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.trace.borrow_mut().push(Call::Iterate(Now { now, wall }));
        self.script.iterate(now, wall);
    }

    fn drain(&mut self) {
        self.trace.borrow_mut().push(Call::Drain);
        self.script.drain();
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        self.script.completions()
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.script.submissions()
    }

    fn work_pending(&self, now: Time) -> bool {
        self.script.work_pending(now)
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.mechanism
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.policy
    }

    fn is_empty(&self) -> bool {
        self.script.is_empty()
    }

    fn exit(&self) -> Option<Exit> {
        self.script.exit()
    }

    fn operations(&self) -> u32 {
        self.script.operations()
    }
}

fn run(host: &mut impl Host, trace: &Trace, wakes: &[Wake<'_>]) -> Exit {
    let mut turns =
        ScriptedTurns { trace, wakes: wakes.iter(), now: START, pending: Vec::new(), ready: Queue::uncapped() };
    let exit = drive_with(&mut turns, host);
    assert_eq!(turns.wakes.len(), 0, "all scripted submits ran before exit");
    exit
}

fn observe(
    script: &mut Script,
    mechanism: Option<Time>,
    policy: Option<Time>,
    wakes: &[Wake<'_>],
) -> (Exit, Vec<Call>) {
    let trace = Trace::default();
    let exit = run(&mut Observed { trace: &trace, script, mechanism, policy }, &trace, wakes);
    (exit, trace.into_inner())
}

fn turn(now: Time, deadline: Option<Time>, last: Call) -> [Call; 6] {
    let now = Now { now, wall: WALL };
    [Call::Reap, Call::Now(now), Call::Iterate(now), Call::Drain, Call::Until(deadline), last]
}

fn read(op: Token) -> Submit {
    Submit { op, kind: Op::PipeRead { fd: READ, buf: Box::new([0; READ_BYTES]) } }
}

fn close(op: Token, fd: Fd) -> Submit {
    Submit { op, kind: Op::Close { fd } }
}

#[test]
fn a_host_that_ends_by_itself_returns_its_exit_at_once() {
    let mut host = Script::new(&[]);
    host.terminal = Some(Exit::Code(7));
    let (exit, trace) = observe(&mut host, None, None, &[]);
    assert_eq!(exit, Exit::Code(7));
    assert_eq!(trace, turn(START, None, Call::InFlight(0)));
}

#[test]
fn the_hook_runs_once_per_iteration_after_iterate_and_before_the_submit() {
    const PIPE_OPERATIONS: usize = 4; // Write, read, then both pipe ends close.
    let write_op = Token::new(1);
    let read_op = Token::new(2);
    let close_write = Token::new(3);
    let close_read = Token::new(4);
    let mut host = Script::pipes(vec![READ, WRITE], &[Act::Write(1, HOOK_BYTES), Act::Read(0)]);
    let (_, trace) = observe(
        &mut host,
        None,
        None,
        &[
            Wake {
                at: START,
                records: &[Submit {
                    op: write_op,
                    kind: Op::PipeWrite { fd: WRITE, bytes: HOOK_BYTES.into(), from: 0 },
                }],
                replies: &[Reply::Done(
                    write_op,
                    Ok(Done::Count(u32::try_from(HOOK_BYTES.len()).expect("small write"))),
                )],
            },
            Wake { at: START, records: &[read(read_op)], replies: &[Reply::Read(read_op, HOOK_BYTES)] },
            Wake {
                at: START,
                records: &[close(close_write, WRITE)],
                replies: &[Reply::Done(close_write, Ok(Done::Nothing))],
            },
            Wake {
                at: START,
                records: &[close(close_read, READ)],
                replies: &[Reply::Done(close_read, Ok(Done::Nothing))],
            },
        ],
    );
    assert_eq!(host.received, HOOK_BYTES);
    assert_eq!(host.drains, host.iterations);
    let mut expected = [turn(START, None, Call::Submit(Wait::Forever)); PIPE_OPERATIONS].concat();
    expected.extend(turn(START, None, Call::InFlight(0)));
    assert_eq!(trace, expected);
}

#[test]
fn pending_work_submits_without_waiting_even_with_a_deadline() {
    let mut host = Script::new(&[Act::Pause(START)]);
    let (_, trace) = observe(&mut host, None, Some(LATE), &[Wake { at: START, records: &[], replies: &[] }]);
    assert_eq!(
        trace.as_slice(),
        [turn(START, Some(LATE), Call::Submit(Wait::No)), turn(START, Some(LATE), Call::InFlight(0))].concat()
    );
}

#[test]
fn idle_work_waits_until_the_earlier_translated_deadline() {
    for (mechanism, policy) in [(EARLY, LATE), (LATE, EARLY)] {
        let mut host = Script::new(&[Act::Pause(EARLY)]);
        let (_, trace) =
            observe(&mut host, Some(mechanism), Some(policy), &[Wake { at: EARLY, records: &[], replies: &[] }]);
        assert_eq!(
            trace,
            [
                turn(START, Some(EARLY), Call::Submit(Wait::Until(NATIVE_EARLY))),
                turn(EARLY, Some(EARLY), Call::InFlight(0))
            ]
            .concat()
        );
        assert_eq!(host.drains, host.iterations);
    }
}

fn read_before_deadline(policy: Option<Time>) -> (Script, Vec<Call>) {
    let read_op = Token::new(1);
    let close_op = Token::new(2);
    let mut host = Script::pipes(vec![READ], &[Act::Read(0)]);
    let (_, trace) = observe(
        &mut host,
        None,
        policy,
        &[
            Wake { at: READY, records: &[read(read_op)], replies: &[Reply::Read(read_op, HOOK_BYTES)] },
            Wake { at: READY, records: &[close(close_op, READ)], replies: &[Reply::Done(close_op, Ok(Done::Nothing))] },
        ],
    );
    (host, trace)
}

#[test]
fn without_a_deadline_an_idle_host_waits_for_a_completion() {
    let (_, trace) = read_before_deadline(None);
    assert_eq!(trace.first_chunk(), Some(&turn(START, None, Call::Submit(Wait::Forever))));
}

#[test]
fn a_completion_reaches_the_host_before_the_deadline() {
    let (host, trace) = read_before_deadline(Some(LATE));
    // The close is submitted on the first turn after the read completes, at READY rather than LATE.
    let wait = Wait::Until(NATIVE_LATE);
    assert_eq!(
        trace,
        [
            turn(START, Some(LATE), Call::Submit(wait)),
            turn(READY, Some(LATE), Call::Submit(wait)),
            turn(READY, Some(LATE), Call::InFlight(0))
        ]
        .concat()
    );
    assert_eq!(host.received, HOOK_BYTES);
}

const DRAIN_READ: Token = Token::new(1);
const SIGNAL_READ: Token = Token::new(2);
const ABORT: Token = Token::new(3);
const CLOSE_SIGNALS: Token = Token::new(4);
const CLOSE_READ: Token = Token::new(5);
const CLOSE_WRITE: Token = Token::new(6);

// After its last word, a silent pipe holds the host's graceful close until
// shutdown aborts the read. Only after that settles can the descriptors close.
struct Closing {
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    started: bool,
    releasing: bool,
    pending: u32,
    heard: bool,
}

impl Host for Closing {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.pending -= 1;
            if matches!(
                complete.result,
                Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Terminate, .. }))
            ) {
                self.heard = true;
                self.submissions.push(Submit { op: ABORT, kind: Op::Cancel { target: DRAIN_READ } });
                self.pending += 1;
            }
        }
        if !self.started {
            self.submissions.push(read(DRAIN_READ));
            self.submissions.push(Submit { op: SIGNAL_READ, kind: Op::ReadSignal { fd: SIGNALS } });
            self.pending = 2;
            self.started = true;
        } else if !self.releasing && self.pending == 0 {
            for (op, fd) in [(CLOSE_SIGNALS, SIGNALS), (CLOSE_READ, READ), (CLOSE_WRITE, WRITE)] {
                self.submissions.push(close(op, fd));
                self.pending += 1;
            }
            self.releasing = true;
        }
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, _now: Time) -> bool {
        !self.completions.is_empty() || !self.submissions.is_empty()
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        Some(LATE)
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }

    fn is_empty(&self) -> bool {
        self.releasing && self.pending == 0 && self.submissions.is_empty()
    }

    fn operations(&self) -> u32 {
        MAX_OPERATIONS
    }
}

#[test]
fn termination_aborts_a_graceful_close_and_drive_awaits_settlement() {
    let mut host = Closing {
        completions: Queue::with_capacity(MAX_OPERATIONS),
        submissions: Queue::with_capacity(MAX_OPERATIONS),
        started: false,
        releasing: false,
        pending: 0,
        heard: false,
    };
    let trace = Trace::default();
    let exit = run(
        &mut host,
        &trace,
        &[
            Wake {
                at: READY,
                records: &[read(DRAIN_READ), Submit { op: SIGNAL_READ, kind: Op::ReadSignal { fd: SIGNALS } }],
                replies: &[Reply::Done(
                    SIGNAL_READ,
                    Ok(Done::Signalled(Signalled::Termination { signal: ServiceSignal::Terminate, sender: None })),
                )],
            },
            Wake {
                at: READY,
                records: &[Submit { op: ABORT, kind: Op::Cancel { target: DRAIN_READ } }],
                replies: &[Reply::Done(ABORT, Ok(Done::Nothing))],
            },
            Wake { at: READY, records: &[], replies: &[Reply::Done(DRAIN_READ, Err(Error::Cancelled))] },
            Wake {
                at: READY,
                records: &[close(CLOSE_SIGNALS, SIGNALS), close(CLOSE_READ, READ), close(CLOSE_WRITE, WRITE)],
                replies: &[Reply::Done(CLOSE_SIGNALS, Ok(Done::Nothing)), Reply::Done(CLOSE_READ, Ok(Done::Nothing))],
            },
            Wake { at: EARLY, records: &[], replies: &[Reply::Done(CLOSE_WRITE, Ok(Done::Nothing))] },
        ],
    );
    assert_eq!(exit, Exit::Code(0));
    assert!(host.heard);
    assert!(trace.borrow().ends_with(&[
        Call::Reap,
        Call::Now(Now { now: EARLY, wall: WALL }),
        Call::Until(Some(LATE)),
        Call::InFlight(0),
    ]));
}
