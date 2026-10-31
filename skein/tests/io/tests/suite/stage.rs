//! io's stage ordering and submission gates (skein/docs/design/io.md, 2).

use skein_io::{self as io, Event, Request, kernel, stage};
use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

const ENTITIES: u32 = 2;
const KERNEL_ROOM: u32 = 2;
const ROOM: u32 = 1;
const ONE: u32 = 1;
const OWNER: Token = Token::new(99);
const HELD: Token = Token::new(100);
const DEADLINE: Duration = Duration::from_secs(1);
const LIMITS: io::Limits = io::Limits { sockets: ENTITIES, close_timeout: DEADLINE, ..super::limits::LIMITS };

/// A kernel played by hand, holding an append write while its deadline,
/// a ready pipe's room and a usage completion meet in one up pass.
struct Stage {
    io: io::Io,
    env: Env<io::Limits>,
    completions: Queue<kernel::Complete>,
    events: Queue<Event>,
    submissions: Queue<kernel::Submit>,
    timed: Token,
    ready: Token,
    write: kernel::Submit,
}

impl Stage {
    fn new() -> Self {
        const APPEND_FD: kernel::Fd = kernel::Fd::new(40);
        const PIPE_FD: kernel::Fd = kernel::Fd::new(41);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
        let mut machine = io::Io::new(&LIMITS);
        let mut submissions = Queue::with_capacity(KERNEL_ROOM);
        let mut completions = Queue::with_capacity(KERNEL_ROOM);
        let mut events = Queue::uncapped();
        let timed = machine.adopt_append(APPEND_FD, DEADLINE).expect("append slot");
        let ready = machine.adopt_write_pipe(PIPE_FD, None).expect("pipe slot");
        io::down(
            &mut machine,
            &env,
            Request::Stream { stream: timed, down: Down::Demand { read: Read::Nothing, room: ROOM } },
            &mut submissions,
        );
        machine.reclaim();
        stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions);
        assert_eq!(events.pop(), Some(Event::Stream { owner: timed, up: Up::Room }));
        io::down(
            &mut machine,
            &env,
            Request::Stream { stream: timed, down: Down::Send(Box::from(b"x".as_slice())) },
            &mut submissions,
        );
        let write = submissions.pop().expect("append write");
        io::down(&mut machine, &env, Request::Usage { owner: OWNER }, &mut submissions);
        let usage = submissions.pop().expect("usage query");
        completions.push(kernel::Complete {
            op: usage.op,
            kind: usage.kind,
            result: Ok(kernel::Done::Usage(kernel::Usage::ZERO)),
        });
        io::down(
            &mut machine,
            &env,
            Request::Stream { stream: ready, down: Down::Demand { read: Read::Nothing, room: ROOM } },
            &mut submissions,
        );
        machine.reclaim();
        let env = Env { now: Time::ZERO.saturating_add(DEADLINE), ..env };
        Self { io: machine, env, completions, events, submissions, timed, ready, write }
    }

    fn up(&mut self) {
        stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions);
    }

    fn hold_one_short(&mut self, max: io::MaxOut) {
        let room = max.submissions.checked_sub(ONE).expect("each entry point can submit");
        for _ in room..self.submissions.capacity() {
            self.submissions.push(kernel::Submit { op: HELD, kind: kernel::Op::Usage });
        }
        assert_eq!(self.submissions.room(), room);
    }

    fn release(&mut self) {
        for _ in 0..self.submissions.len() {
            let held = self.submissions.pop().expect("kernel takes a held submission");
            assert_eq!(held.op, HELD);
        }
    }

    fn assert_ordered(&mut self) {
        let events: Vec<_> = self.events.iter().collect();
        assert_eq!(
            events,
            [
                &Event::Stream { owner: self.ready, up: Up::Room },
                &Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) },
                &Event::Stream { owner: self.timed, up: Up::Failed(Fault::Other) },
            ]
        );
        assert!(!self.io.is_ready() && self.completions.is_empty() && !self.io.is_due(self.env.now));
        assert_eq!(
            self.submissions.pop().expect("deadline cancels the write").kind,
            kernel::Op::Cancel { target: self.write.op }
        );
        assert!(self.submissions.is_empty());
    }
}

#[test]
fn ready_results_precede_completions_and_deadlines() {
    let mut world = Stage::new();
    assert!(world.io.is_ready() && !world.completions.is_empty() && world.io.is_due(world.env.now));
    world.up();
    world.assert_ordered();
}

#[test]
fn resume_waits_for_its_submission_room_and_keeps_later_passes_queued() {
    let mut world = Stage::new();
    world.hold_one_short(io::MAX_OUT_RESUME);
    world.up();
    assert!(world.events.is_empty() && world.io.is_ready());
    assert_eq!(world.completions.len(), ONE);
    assert!(world.io.is_due(world.env.now));
    world.release();
    world.up();
    world.assert_ordered();
}

#[test]
fn completions_wait_for_their_submission_room_after_ready_results() {
    let mut world = Stage::new();
    world.hold_one_short(io::MAX_OUT_UP);
    world.up();
    assert_eq!(world.events.iter().collect::<Vec<_>>(), [&Event::Stream { owner: world.ready, up: Up::Room }]);
    assert!(!world.io.is_ready());
    assert_eq!(world.completions.len(), ONE);
    assert!(world.io.is_due(world.env.now));
    world.release();
    world.up();
    world.assert_ordered();
}

#[test]
fn fire_waits_for_its_submission_room_and_keeps_the_deadline_due() {
    let mut world = Stage::new();
    world.env.now = Time::ZERO;
    world.up();
    assert!(!world.io.is_ready() && world.completions.is_empty());
    let told = world.events.len();
    world.env.now = Time::ZERO.saturating_add(DEADLINE);
    world.hold_one_short(io::MAX_OUT_FIRE);
    world.up();
    assert_eq!(world.events.len(), told);
    assert!(world.io.is_due(world.env.now));
    world.release();
    world.up();
    world.assert_ordered();
}

#[test]
fn down_waits_for_its_submission_room_and_keeps_requests_in_order() {
    const FIRST: Token = Token::new(10);
    const SECOND: Token = Token::new(11);
    const STALE: Token = Token::new(12);
    let mut machine = io::Io::new(&LIMITS);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut requests = Queue::uncapped();
    requests.push(Request::Usage { owner: FIRST });
    requests.push(Request::Window { owner: SECOND, stream: STALE });
    submissions.push(kernel::Submit { op: HELD, kind: kernel::Op::Usage });
    assert_eq!(submissions.room(), io::MAX_OUT_DOWN.submissions.checked_sub(ONE).expect("down submits"));
    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert_eq!(
        requests.iter().collect::<Vec<_>>(),
        [&Request::Usage { owner: FIRST }, &Request::Window { owner: SECOND, stream: STALE }]
    );
    assert_eq!(submissions.pop().expect("kernel takes the held submission").op, HELD);
    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert_eq!(requests.iter().collect::<Vec<_>>(), [&Request::Window { owner: SECOND, stream: STALE }]);
    assert_eq!(submissions.pop().expect("usage submitted first").kind, kernel::Op::Usage);
    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert!(requests.is_empty());
    assert_eq!(
        submissions.pop().expect("window submitted second").kind,
        kernel::Op::Window { fd: kernel::Fd::new(-1) }
    );
}

#[test]
fn down_leaves_requests_queued_until_io_takes_another_refusal() {
    const REFUSED: Token = Token::new(20);
    const NEXT: Token = Token::new(21);
    const IP: [u8; 4] = [127, 0, 0, 1];
    const PORT: u16 = 1;
    let addr = io::Addr::Ip((IP, PORT).into());
    let mut world = Stage::new();
    let mut requests = Queue::uncapped();
    requests.push(Request::Connect { owner: REFUSED, addr });
    requests.push(Request::Connect { owner: NEXT, addr });
    stage::down(&mut world.io, &world.env, &mut requests, &mut world.submissions);
    assert!(!world.io.takes());
    assert_eq!(requests.iter().collect::<Vec<_>>(), [&Request::Connect { owner: NEXT, addr }]);
    world.env.now = Time::ZERO;
    world.up();
    assert!(world.io.takes());
    stage::down(&mut world.io, &world.env, &mut requests, &mut world.submissions);
    assert!(requests.is_empty());
    assert!(!world.io.takes());
}
