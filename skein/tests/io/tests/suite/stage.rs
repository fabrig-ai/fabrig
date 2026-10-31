//! io's stages of the loop's passes (`skein_io::stage`; io.md, 2), driven by
//! hand with the kernel's submission queue near full: each entry waits until
//! what it may submit fits (testing-strategy.md, 6), and what waits stays in
//! order for the next iteration.

use skein_io::{self as io, Event, Request, kernel, stage};
use skein_io_world::scenarios::{limits, local};
use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

#[test]
fn the_stage_drains_ready_work_before_completions_and_timers_with_the_kernel_queue_near_full() {
    // A held write leaves room for ready work, but completions and timers
    // still wait for their two kernel submissions.
    const KERNEL_ROOM: u32 = 2;
    const OWNER: Token = Token::new(99);
    const DEADLINE: Duration = Duration::from_secs(1);
    let limits = limits(3);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut machine = io::Io::new(&limits);
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut completions = Queue::with_capacity(KERNEL_ROOM);
    let mut told = Queue::uncapped();
    let timed = machine.adopt_append(kernel::Fd::new(40), DEADLINE).expect("append slot");
    let ready = machine.adopt_write_pipe(kernel::Fd::new(41), None).expect("pipe slot");
    io::down(
        &mut machine,
        &env,
        Request::Stream { stream: timed, down: Down::Demand { read: Read::Nothing, room: 1 } },
        &mut submissions,
    );
    machine.reclaim();
    stage::up(&mut machine, &env, &mut completions, &mut told, &mut submissions, Time::ZERO);
    drop(told.pop());
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
    submissions.push(write);
    io::down(
        &mut machine,
        &env,
        Request::Stream { stream: ready, down: Down::Demand { read: Read::Nothing, room: 1 } },
        &mut submissions,
    );
    machine.reclaim();
    let now = Time::ZERO.saturating_add(DEADLINE);
    stage::up(&mut machine, &env, &mut completions, &mut told, &mut submissions, now);
    let ahead = told.len() == 1 && !machine.is_ready();
    let blocked = machine.is_due(now) && !completions.is_empty();
    let write = submissions.pop().expect("kernel takes the held write");
    let late = Env { now, ..env };
    stage::up(&mut machine, &late, &mut completions, &mut told, &mut submissions, now);
    let events: Vec<_> = told.iter().collect();
    assert!(ahead, "ready work takes the one submission it may make");
    assert!(blocked, "completions and timers wait at the kernel edge");
    assert_eq!(
        events,
        [
            &Event::Stream { owner: ready, up: Up::Room },
            &Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) },
            &Event::Stream { owner: timed, up: Up::Failed(Fault::Other) }
        ]
    );
    assert!(!machine.is_ready() && completions.is_empty() && !machine.is_due(now));
    assert_eq!(submissions.pop().expect("deadline cancels the write").kind, kernel::Op::Cancel { target: write.op });
}

#[test]
fn the_down_stage_stops_at_the_kernel_edge_and_at_a_full_refusal_list_leaving_the_rest_in_order() {
    // A connect submits one socket, so two are taken from a kernel queue of
    // three, and the third waits at one free submission, short of
    // `MAX_OUT_DOWN`.
    const KERNEL_ROOM: u32 = 3;
    const SOCKETS: u32 = 2;
    const SOCKET: kernel::Op = kernel::Op::Socket { family: kernel::Family::Ipv4 };
    let limits = limits(SOCKETS);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut machine = io::Io::new(&limits);
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut asked = Queue::uncapped();
    // Two connects fill the slab; the next two are refused, filling the
    // refusal list (`limits(_).refusals`); the last waits behind them.
    for owner in 1..=5 {
        asked.push(connect(owner));
    }
    stage::down(&mut machine, &env, &mut asked, &mut submissions);
    let at_edge: Vec<_> = asked.iter().collect();
    assert_eq!(at_edge, [&connect(3), &connect(4), &connect(5)], "the kernel edge holds the rest, in order");
    let first = submissions.pop().expect("the first connect's socket").kind;
    let second = submissions.pop().expect("the second connect's socket").kind;
    assert_eq!([first, second], [SOCKET, SOCKET], "each taken connect opens its socket");
    stage::down(&mut machine, &env, &mut asked, &mut submissions);
    let refused: Vec<_> = asked.iter().collect();
    assert_eq!(refused, [&connect(5)], "a full refusal list holds the rest");
    assert!(!machine.takes() && submissions.is_empty(), "a refusal submits nothing");
}

/// A connect for `owner`, to an address no one listens at.
fn connect(owner: u64) -> Request {
    Request::Connect { owner: Token::new(owner), addr: local(7) }
}
