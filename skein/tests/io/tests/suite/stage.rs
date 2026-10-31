//! Stage ordering and admission at the kernel edge (io.md, 2).

use skein_io::{self as io, Event, Request, kernel, stage};
use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

const KERNEL_ROOM: u32 = 2;
const OWNER: Token = Token::new(99);
const DEADLINE: Duration = Duration::from_secs(1);

// The kernel is played by hand so each stage's room check can be observed.
struct World {
    io: io::Io,
    env: Env<io::Limits>,
    events: Queue<Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}

impl World {
    fn new() -> Self {
        let limits = skein_io_world::scenarios::limits(3);
        Self {
            io: io::Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            events: Queue::uncapped(),
            completions: Queue::with_capacity(KERNEL_ROOM),
            submissions: Queue::with_capacity(KERNEL_ROOM),
        }
    }

    fn request(&mut self, request: Request) {
        io::down(&mut self.io, &self.env, request, &mut self.submissions);
    }

    fn up(&mut self, now: Time) {
        self.env.now = now;
        stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions, now);
    }
}

#[test]
fn ready_results_precede_completions_and_timers_at_each_room_check() {
    // With a completion waiting, one free slot blocks completions and timers;
    // without it, the same edge checks the timer stage directly.
    for complete_usage in [true, false] {
        let mut world = World::new();
        let timed = world.io.adopt_append(kernel::Fd::new(40), DEADLINE).expect("append slot");
        let ready = world.io.adopt_write_pipe(kernel::Fd::new(41), None).expect("pipe slot");
        world.request(Request::Stream { stream: timed, down: Down::Demand { read: Read::Nothing, room: 1 } });
        world.io.reclaim();
        world.up(Time::ZERO);
        assert_eq!(world.events.pop(), Some(Event::Stream { owner: timed, up: Up::Room }));
        world.request(Request::Stream { stream: timed, down: Down::Send(Box::from(b"x".as_slice())) });
        let write = world.submissions.pop().expect("held append write");
        world.request(Request::Usage { owner: OWNER });
        let usage = world.submissions.pop().expect("usage query");
        world.completions.push(kernel::Complete {
            op: usage.op,
            kind: usage.kind,
            result: Ok(kernel::Done::Usage(kernel::Usage::ZERO)),
        });
        if !complete_usage {
            world.up(Time::ZERO);
            assert_eq!(
                world.events.pop(),
                Some(Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) })
            );
        }
        world.submissions.push(write);
        world.request(Request::Stream { stream: ready, down: Down::Demand { read: Read::Nothing, room: 1 } });
        world.io.reclaim();
        // A second held kernel record fills the queue and blocks ready work.
        let held = kernel::Submit { op: Token::new(100), kind: kernel::Op::Usage };
        world.submissions.push(held);
        let now = Time::ZERO.saturating_add(DEADLINE);
        world.up(now);
        assert!(world.events.is_empty() && world.io.is_ready() && world.io.is_due(now));
        assert_eq!(world.completions.len(), u32::from(complete_usage));
        let write = world.submissions.pop().expect("kernel takes the write");
        world.up(now);
        assert_eq!(world.events.pop(), Some(Event::Stream { owner: ready, up: Up::Room }));
        assert!(world.events.is_empty() && !world.io.is_ready() && world.io.is_due(now));
        assert_eq!(world.completions.len(), u32::from(complete_usage));
        assert_eq!(world.submissions.len(), 1, "no timer cancellation without its maximum room");
        drop(world.submissions.pop().expect("kernel takes the other held record"));
        world.up(now);
        if complete_usage {
            assert_eq!(
                world.events.pop(),
                Some(Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) })
            );
        }
        assert_eq!(world.events.pop(), Some(Event::Stream { owner: timed, up: Up::Failed(Fault::Other) }));
        assert!(world.events.is_empty() && world.completions.is_empty() && !world.io.is_due(now));
        assert_eq!(
            world.submissions.pop().expect("deadline cancels the held write").kind,
            kernel::Op::Cancel { target: write.op }
        );
        assert!(world.submissions.is_empty());
    }
}

#[test]
fn down_leaves_requests_queued_until_their_maximum_submissions_fit() {
    let mut world = World::new();
    let mut requests = Queue::uncapped();
    for port in [1001_u16, 1002, 1003] {
        requests
            .push(Request::Listen { owner: Token::new(u64::from(port)), addr: skein_io_world::scenarios::local(port) });
    }
    for remaining in [2, 1, 0] {
        stage::down(&mut world.io, &world.env, &mut requests, &mut world.submissions);
        assert_eq!(requests.len(), remaining, "one request taken with two slots free");
        assert_eq!(world.submissions.len(), 1);
        stage::down(&mut world.io, &world.env, &mut requests, &mut world.submissions);
        assert_eq!(requests.len(), remaining, "one free slot leaves the next request queued");
        assert_eq!(world.submissions.len(), 1);
        assert_eq!(
            world.submissions.pop().expect("kernel takes the socket request").kind,
            kernel::Op::Socket { family: kernel::Family::Ipv4 }
        );
    }
}
