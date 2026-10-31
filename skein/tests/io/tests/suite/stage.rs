//! io's stage (io.md, 2.1): `skein_io::stage::{up, down}` driven by hand, the
//! kernel played by the test, each step's order and the room each entry
//! point waits for in the kernel's submission queue, from both sides.

use std::net::{Ipv4Addr, SocketAddr};

use skein_io::kernel::{Complete, Done, Fd, Op, Submit, Usage};
use skein_io::{
    Addr, Error, Event, Io, Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, MAX_OUT_RESUME, MAX_OUT_UP, Measured, Request, stage,
};
use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

/// Two entities, one refusal held and one accept an iteration: small enough
/// for every case here to reach its bound.
const LIMITS: Limits = Limits {
    sockets: 2,
    names: 0,
    refusals: 1,
    intake: 64,
    receive: 32,
    output: 64,
    sends: 2,
    accepts: 1,
    backlog: 1,
    close_timeout: Duration::from_secs(1),
    retry: Duration::from_millis(1),
};

/// Room for everything a case submits at once.
const ROOMY: u32 = 16;

/// An append stream's write deadline, which the deadline cases pass.
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);

const OWNER: Token = Token::new(99);

/// io, its environment, and the loop's queues around it.
struct Rig {
    io: Io,
    env: Env<Limits>,
    completions: Queue<Complete>,
    events: Queue<Event>,
    requests: Queue<Request>,
    subs: Queue<Submit>,
}

impl Rig {
    fn new(limits: Limits) -> Rig {
        Rig {
            io: Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            completions: Queue::with_capacity(ROOMY),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            subs: Queue::with_capacity(ROOMY),
        }
    }

    fn up(&mut self) {
        stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.subs);
    }

    fn down(&mut self) {
        stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.subs);
    }

    /// One request taken by the down stage, with room for it.
    fn ask(&mut self, request: Request) {
        self.requests.push(request);
        self.down();
        assert!(self.requests.is_empty(), "the request was taken");
    }

    /// The reclaim point: what the iteration readied is ready for the next.
    fn next(&mut self) {
        self.io.reclaim();
    }

    /// What io told since the last look, oldest first.
    fn told(&mut self) -> Vec<Event> {
        let mut told = Vec::new();
        while let Some(event) = self.events.pop() {
            told.push(event);
        }
        told
    }

    /// What io submitted since the last look: the kernel takes it.
    fn taken(&mut self) -> Vec<Submit> {
        let mut taken = Vec::new();
        while let Some(submit) = self.subs.pop() {
            taken.push(submit);
        }
        taken
    }

    /// The kernel's queue, `room` records long, empty.
    fn room(&mut self, room: u32) {
        assert!(self.subs.is_empty(), "the kernel took what was submitted");
        self.subs = Queue::with_capacity(room);
    }

    /// A process usage asked and answered: its completion waits in the queue.
    fn usage_answered(&mut self) {
        self.ask(Request::Usage { owner: OWNER });
        let [Submit { op, kind }] = self.taken().try_into().expect("one usage read");
        self.completions.push(Complete { op, kind, result: Ok(Done::Usage(Usage::ZERO)) });
    }

    /// A write pipe whose owner asked for room: on the ready list for the
    /// next stage, which tells it `Room` and submits nothing.
    fn room_asked(&mut self, fd: i32) -> Token {
        let pipe = self.io.adopt_write_pipe(Fd::new(fd), None).expect("a pipe slot");
        self.ask(Request::Stream { stream: pipe, down: Down::Demand { read: Read::Nothing, room: 1 } });
        pipe
    }

    /// An append stream with one write in flight since `Time::ZERO`, due to
    /// fail at its write deadline: the write the kernel took, to cancel.
    fn write_in_flight(&mut self, fd: i32) -> (Token, Submit) {
        let append = self.io.adopt_append(Fd::new(fd), WRITE_TIMEOUT).expect("an append slot");
        self.ask(Request::Stream { stream: append, down: Down::Demand { read: Read::Nothing, room: 1 } });
        self.next();
        self.up();
        assert_eq!(self.told(), [Event::Stream { owner: append, up: Up::Room }]);
        self.ask(Request::Stream { stream: append, down: Down::Send(Box::from(b"x".as_slice())) });
        let [write] = self.taken().try_into().expect("one append write");
        assert!(matches!(write.kind, Op::Append { .. }), "{write:?}");
        (append, write)
    }

    /// Time passes to the write deadline.
    fn at_write_deadline(&mut self) {
        self.env.now = Time::ZERO.saturating_add(WRITE_TIMEOUT);
    }
}

fn usage() -> Event {
    Event::Usage { owner: OWNER, usage: Measured::Read(Usage::ZERO) }
}

fn local(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

/// A record the kernel has not taken yet, holding a slot of its queue.
fn held() -> Submit {
    Submit { op: Token::new(u64::MAX), kind: Op::Usage }
}

#[test]
fn up_tells_ready_work_then_completions_then_deadlines() {
    let mut rig = Rig::new(LIMITS);
    let (append, write) = rig.write_in_flight(40);
    rig.usage_answered();
    let pipe = rig.room_asked(41);
    rig.next();
    rig.at_write_deadline();
    rig.up();
    assert_eq!(
        rig.told(),
        [
            Event::Stream { owner: pipe, up: Up::Room },
            usage(),
            Event::Stream { owner: append, up: Up::Failed(Fault::Other) }
        ],
        "the ready list, then the completion, then the deadline"
    );
    let [cancel] = rig.taken().try_into().expect("the deadline cancels the write");
    assert_eq!(cancel.kind, Op::Cancel { target: write.op });
    assert!(!rig.io.is_ready() && rig.completions.is_empty() && !rig.io.is_due(rig.env.now));
}

#[test]
fn completions_and_deadlines_wait_while_the_kernel_queue_is_near_full() {
    let mut rig = Rig::new(LIMITS);
    let (append, write) = rig.write_in_flight(40);
    rig.usage_answered();
    let pipe = rig.room_asked(41);
    rig.next();
    rig.at_write_deadline();
    // One record the kernel has not taken leaves room for a resume, but not
    // for a completion's or a deadline's submissions.
    rig.room(MAX_OUT_UP.submissions);
    rig.subs.push(held());
    assert!(rig.subs.room() >= MAX_OUT_RESUME.submissions && rig.subs.room() < MAX_OUT_UP.submissions);
    rig.up();
    assert_eq!(rig.told(), [Event::Stream { owner: pipe, up: Up::Room }], "only the ready work fits");
    assert!(!rig.completions.is_empty() && rig.io.is_due(rig.env.now), "the completion and the deadline wait");
    assert_eq!(rig.taken(), [held()], "the kernel takes the record it held");
    rig.next();
    rig.up();
    assert_eq!(rig.told(), [usage(), Event::Stream { owner: append, up: Up::Failed(Fault::Other) }]);
    let [cancel] = rig.taken().try_into().expect("the deadline cancels the write");
    assert_eq!(cancel.kind, Op::Cancel { target: write.op });
}

#[test]
fn ready_work_waits_for_room_for_the_most_a_resume_submits() {
    let mut rig = Rig::new(LIMITS);
    // Each read pipe, resumed, submits its first read.
    let _first = rig.io.adopt_read_pipe(Fd::new(40)).expect("a pipe slot");
    let _second = rig.io.adopt_read_pipe(Fd::new(41)).expect("a pipe slot");
    rig.usage_answered();
    rig.next();
    rig.room(MAX_OUT_RESUME.submissions - 1);
    rig.up();
    assert!(rig.io.is_ready() && rig.subs.is_empty(), "no room, no resume");
    rig.room(MAX_OUT_RESUME.submissions);
    rig.up();
    assert!(rig.io.is_ready(), "the second pipe waits for room");
    assert_eq!(rig.completions.len(), 1, "the completion waits behind the ready list");
    let mut reads = rig.taken();
    rig.up();
    reads.append(&mut rig.taken());
    let fds: Vec<_> = reads
        .iter()
        .map(|read| {
            let Op::PipeRead { fd, .. } = read.kind else { panic!("a pipe read: {read:?}") };
            fd
        })
        .collect();
    assert_eq!(fds, [Fd::new(40), Fd::new(41)], "each pipe's read, one a stage, in turn");
    assert!(!rig.io.is_ready());
    assert_eq!(rig.completions.len(), 1, "a completion waits for room for the most up submits");
    assert!(rig.told().is_empty());
}

#[test]
fn a_completion_waits_for_room_for_the_most_up_submits() {
    let mut rig = Rig::new(LIMITS);
    rig.usage_answered();
    rig.room(MAX_OUT_UP.submissions - 1);
    rig.up();
    assert_eq!(rig.completions.len(), 1, "one short of the room, the completion waits");
    assert!(rig.told().is_empty());
    rig.room(MAX_OUT_UP.submissions);
    rig.up();
    assert!(rig.completions.is_empty());
    assert_eq!(rig.told(), [usage()]);
}

#[test]
fn a_due_deadline_waits_for_room_for_the_most_fire_submits() {
    let mut rig = Rig::new(LIMITS);
    let (append, write) = rig.write_in_flight(40);
    rig.next();
    rig.at_write_deadline();
    rig.room(MAX_OUT_FIRE.submissions - 1);
    rig.up();
    assert!(rig.io.is_due(rig.env.now), "one short of the room, the deadline waits");
    assert!(rig.told().is_empty());
    rig.room(MAX_OUT_FIRE.submissions);
    rig.up();
    assert!(!rig.io.is_due(rig.env.now));
    assert_eq!(rig.told(), [Event::Stream { owner: append, up: Up::Failed(Fault::Other) }]);
    let [cancel] = rig.taken().try_into().expect("the deadline cancels the write");
    assert_eq!(cancel.kind, Op::Cancel { target: write.op });
}

#[test]
fn a_deadline_is_due_at_the_stage_s_env_now() {
    let mut rig = Rig::new(LIMITS);
    let (_append, _write) = rig.write_in_flight(40);
    rig.next();
    rig.env.now = Time::from_nanos(WRITE_TIMEOUT.as_nanos() - 1);
    rig.up();
    assert!(rig.told().is_empty() && rig.subs.is_empty(), "a nanosecond early, nothing fires");
    rig.at_write_deadline();
    rig.up();
    assert_eq!(rig.told().len(), 1, "at the deadline, it fires");
}

#[test]
fn each_entity_is_resumed_at_most_once_a_stage() {
    // Two listeners under a batch of one accept: the second, resumed once
    // the batch is spent, readies itself again, for the next stage. A third
    // slot leaves room for an accepted socket, or neither would accept.
    let mut rig = Rig::new(Limits { sockets: 3, ..LIMITS });
    let first = rig.io.adopt_listener(Fd::new(40), local(4000)).expect("a listener slot");
    let second = rig.io.adopt_listener(Fd::new(41), local(4001)).expect("a listener slot");
    rig.ask(Request::Listen { owner: Token::new(1), addr: Addr::Ip(local(4000)) });
    rig.ask(Request::Listen { owner: Token::new(2), addr: Addr::Ip(local(4001)) });
    rig.next();
    rig.up();
    assert_eq!(
        rig.told(),
        [
            Event::Listening { owner: Token::new(1), listener: first, addr: Addr::Ip(local(4000)) },
            Event::Listening { owner: Token::new(2), listener: second, addr: Addr::Ip(local(4001)) },
        ]
    );
    assert!(!rig.io.is_ready(), "the second listener waits for the next stage");
    let [accept] = rig.taken().try_into().expect("one accept a batch");
    assert_eq!(accept.kind, Op::Accept { fd: Fd::new(40) });
    rig.next();
    rig.up();
    let [accept] = rig.taken().try_into().expect("the second listener's accept, in the next stage");
    assert_eq!(accept.kind, Op::Accept { fd: Fd::new(41) });
    assert!(rig.told().is_empty());
}

#[test]
fn down_takes_requests_in_order_while_the_kernel_has_room_for_the_most_down_submits() {
    // Each connect submits the making of its socket.
    let mut rig = Rig::new(LIMITS);
    let connect = |owner| Request::Connect { owner: Token::new(owner), addr: Addr::Ip(local(80)) };
    rig.requests.push(connect(1));
    rig.requests.push(connect(2));
    rig.room(MAX_OUT_DOWN.submissions - 1);
    rig.down();
    assert_eq!(rig.requests.len(), 2, "one short of the room, no request is taken");
    rig.room(MAX_OUT_DOWN.submissions);
    rig.down();
    assert_eq!(rig.requests.iter().collect::<Vec<_>>(), [&connect(2)], "the second waits for room");
    let [socket] = rig.taken().try_into().expect("the first connect's socket");
    assert!(matches!(socket.kind, Op::Socket { .. }), "{socket:?}");
    rig.down();
    assert!(rig.requests.is_empty());
    rig.next();
    rig.up();
    let owners: Vec<_> = rig
        .told()
        .into_iter()
        .map(|event| {
            let Event::Connecting { owner, .. } = event else {
                panic!("a connect is told Connecting first: {event:?}")
            };
            owner
        })
        .collect();
    assert_eq!(owners, [Token::new(1), Token::new(2)], "taken oldest first");
}

#[test]
fn down_holds_requests_while_io_cannot_hold_a_refusal() {
    // No entity slot: every connect is refused, and io holds one refusal.
    let mut rig = Rig::new(Limits { sockets: 0, ..LIMITS });
    let connect = |owner| Request::Connect { owner: Token::new(owner), addr: Addr::Ip(local(80)) };
    rig.requests.push(connect(1));
    rig.requests.push(connect(2));
    rig.down();
    assert!(!rig.io.takes(), "io holds the first refusal");
    assert_eq!(rig.requests.iter().collect::<Vec<_>>(), [&connect(2)], "the second waits for it to be told");
    rig.next();
    rig.up();
    assert_eq!(
        rig.told(),
        [Event::Failed { owner: Token::new(1), error: Error::Busy }, Event::Closed { owner: Token::new(1) }]
    );
    rig.down();
    assert!(rig.requests.is_empty() && !rig.io.takes(), "the second taken, and refused in turn");
    assert!(rig.subs.is_empty(), "a refusal submits nothing");
}
