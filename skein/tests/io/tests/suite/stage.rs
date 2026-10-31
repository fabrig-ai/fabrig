//! io's stage ordering and kernel edge gates (io.md, 2).

use skein_io::{self as io, Event, Request, kernel, stage};
use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

use super::limits::LIMITS;

// Two submissions hold any one step's maximum (io.md, 2).
const KERNEL_ROOM: u32 = 2;
const OWNER: Token = Token::new(99);
const DEADLINE: Duration = Duration::from_secs(1);
const TIMED_FD: kernel::Fd = kernel::Fd::new(40);
const READY_FD: kernel::Fd = kernel::Fd::new(41);
const TERMINAL_FD: kernel::Fd = kernel::Fd::new(42);

struct World {
    io: io::Io,
    env: Env<io::Limits>,
    completions: Queue<kernel::Complete>,
    events: Queue<Event>,
    requests: Queue<Request>,
    submissions: Queue<kernel::Submit>,
}

impl World {
    fn new(sockets: u32) -> Self {
        let limits = io::Limits { sockets, close_timeout: DEADLINE, ..LIMITS };
        Self {
            io: io::Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            completions: Queue::with_capacity(KERNEL_ROOM),
            events: Queue::uncapped(),
            requests: Queue::uncapped(),
            submissions: Queue::with_capacity(KERNEL_ROOM),
        }
    }

    fn up(&mut self) {
        stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions);
    }

    fn down(&mut self) {
        stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.submissions);
    }

    fn request(&mut self, request: Request) {
        self.requests.push(request);
        self.down();
    }
}

struct Pending {
    world: World,
    write: kernel::Submit,
    timed: Token,
    ready: Token,
}

fn pending(sockets: u32) -> Pending {
    // An append with a write deadline and a pipe with a ready room event.
    let mut world = World::new(sockets);
    let timed = world.io.adopt_append(TIMED_FD, DEADLINE).expect("append slot");
    let ready = world.io.adopt_write_pipe(READY_FD, None).expect("pipe slot");
    world.request(Request::Stream { stream: timed, down: Down::Demand { read: Read::Nothing, room: 1 } });
    world.io.reclaim();
    world.up();
    drop(world.events.pop());
    world.request(Request::Stream { stream: timed, down: Down::Send(Box::from(b"x".as_slice())) });
    let write = world.submissions.pop().expect("append write");
    world.request(Request::Usage { owner: OWNER });
    let usage = world.submissions.pop().expect("usage query");
    world.completions.push(kernel::Complete {
        op: usage.op,
        kind: usage.kind,
        result: Ok(kernel::Done::Usage(kernel::Usage::ZERO)),
    });
    world.request(Request::Stream { stream: ready, down: Down::Demand { read: Read::Nothing, room: 1 } });
    world.io.reclaim();
    world.env.now = Time::ZERO.saturating_add(DEADLINE);
    Pending { world, write, timed, ready }
}

#[test]
fn the_stage_tells_ready_results_before_completions_and_timers() {
    // The timed append, the ready pipe and the window query's terminal.
    const STREAMS: u32 = 3;
    let Pending { mut world, write, timed, ready } = pending(STREAMS);
    // A window query beside the held write fills the kernel edge, so even
    // resume's one submission must wait before the two-submission steps.
    let terminal = world.io.adopt_read_pipe(TERMINAL_FD).expect("terminal slot");
    world.request(Request::Window { owner: OWNER, stream: terminal });
    world.submissions.push(write);
    world.up();
    let blocked = world.events.is_empty()
        && world.io.is_ready()
        && world.io.is_due(world.env.now)
        && world.completions.len() == 1;
    let _window = world.submissions.pop().expect("kernel takes the window query");
    let write = world.submissions.pop().expect("kernel takes the held write");
    world.up();
    let events: Vec<_> = world.events.iter().collect();
    assert!(blocked, "all steps wait at the full kernel edge");
    assert_eq!(
        events,
        [
            &Event::Stream { owner: ready, up: Up::Room },
            &Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) },
            &Event::Stream { owner: timed, up: Up::Failed(Fault::Other) },
        ]
    );
    assert!(!world.io.is_ready() && world.completions.is_empty() && !world.io.is_due(world.env.now));
    assert_eq!(
        world.submissions.pop().expect("deadline cancels the write").kind,
        kernel::Op::Cancel { target: write.op }
    );
}

#[test]
fn the_up_stage_waits_for_room_at_each_step() {
    // The timed append and the ready pipe.
    const STREAMS: u32 = 2;
    let Pending { mut world, write, timed, ready } = pending(STREAMS);
    // resume needs one slot; no slot leaves all three kinds of work held.
    world.submissions = Queue::with_capacity(0);
    world.up();
    assert!(world.events.is_empty());
    assert!(world.io.is_ready());
    assert_eq!(world.completions.len(), 1);
    assert!(world.io.is_due(world.env.now));

    // One slot admits resume, but not up's two submissions.
    world.submissions = Queue::with_capacity(1);
    world.up();
    assert_eq!(world.events.pop(), Some(Event::Stream { owner: ready, up: Up::Room }));
    assert!(world.events.is_empty());
    assert!(!world.io.is_ready());
    assert_eq!(world.completions.len(), 1);
    assert!(world.io.is_due(world.env.now));

    // At up's bound the completion is consumed. Keep the timer in the
    // future so its own gate can be exercised independently.
    world.submissions = Queue::with_capacity(KERNEL_ROOM);
    world.env.now = Time::ZERO;
    world.up();
    assert_eq!(world.events.pop(), Some(Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) }));
    assert!(world.completions.is_empty());
    world.submissions.push(write);
    world.env.now = Time::ZERO.saturating_add(DEADLINE);
    world.up();
    assert!(world.events.is_empty());
    assert!(world.io.is_due(world.env.now));
    assert_eq!(world.submissions.len(), 1);

    // Two slots admit fire, which removes the deadline and cancels its write.
    let write = world.submissions.pop().expect("kernel takes the held write");
    world.up();
    assert_eq!(world.events.pop(), Some(Event::Stream { owner: timed, up: Up::Failed(Fault::Other) }));
    assert!(!world.io.is_due(world.env.now));
    assert_eq!(
        world.submissions.pop().expect("deadline cancels the write").kind,
        kernel::Op::Cancel { target: write.op }
    );
}

#[test]
fn the_down_stage_keeps_what_io_does_not_take_in_order() {
    const FIRST: Token = Token::new(1);
    const SECOND: Token = Token::new(2);
    const THIRD: Token = Token::new(3);
    const REQUESTS: u32 = 3;
    let addr = io::Addr::Ip(([127, 0, 0, 1], 1234).into());
    // With entity slots the first connect leaves fewer than MAX_OUT_DOWN
    // submissions free. Without slots the first refusal makes takes false.
    for (sockets, submissions, takes) in [(REQUESTS, 1, true), (0, 0, false)] {
        let mut world = World::new(sockets);
        for owner in [FIRST, SECOND, THIRD] {
            world.requests.push(Request::Connect { owner, addr });
        }
        world.down();
        assert_eq!(world.submissions.len(), submissions);
        assert_eq!(world.io.takes(), takes);
        world.down();
        assert_eq!(
            world.requests.iter().collect::<Vec<_>>(),
            [&Request::Connect { owner: SECOND, addr }, &Request::Connect { owner: THIRD, addr }]
        );
        assert_eq!(world.submissions.len(), submissions);

        // Once the kernel takes the submission, or resume tells the refusal,
        // the next request is the second one, with the third still held.
        drop(world.submissions.pop());
        world.io.reclaim();
        world.up();
        world.down();
        assert_eq!(world.requests.iter().collect::<Vec<_>>(), [&Request::Connect { owner: THIRD, addr }]);
    }
}
