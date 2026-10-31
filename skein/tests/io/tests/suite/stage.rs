//! io's stage ordering and room at the kernel edge (io.md, 2).

use skein_io::{self as io, Event, Request, kernel};
use skein_io_world::scenarios::{limits, local};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

#[test]
fn the_stage_drains_ready_results_completions_and_timers_in_order_with_the_kernel_queue_near_full() {
    // A held write leaves room for ready work, but completions and timers
    // still wait for their two kernel submissions.
    const KERNEL_ROOM: u32 = 2;
    const OWNER: Token = Token::new(99);
    const DEADLINE: skein_lib::Duration = skein_lib::Duration::from_secs(1);
    let limits = limits(2);
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
    io::stage::up(&mut machine, &env, &mut completions, &mut told, &mut submissions, Time::ZERO);
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
    let mut full = Queue::with_capacity(0);
    io::stage::up(&mut machine, &env, &mut completions, &mut told, &mut full, now);
    assert!(machine.is_ready() && machine.is_due(now));
    assert_eq!(completions.len(), 1, "the completion waits behind ready work");
    assert!(told.is_empty(), "resume waits without its one submission slot");
    io::stage::up(&mut machine, &env, &mut completions, &mut told, &mut submissions, now);
    assert!(!machine.is_ready(), "one free submission lets ready work run");
    assert_eq!(told.len(), 1, "ready result precedes the held completion");
    assert!(machine.is_due(now) && completions.len() == 1, "completions and timers wait at the kernel edge");
    let write = submissions.pop().expect("kernel takes the held write");
    let late = Env { now, ..env };
    io::stage::up(&mut machine, &late, &mut completions, &mut told, &mut submissions, now);
    let events: Vec<_> = told.iter().collect();
    assert_eq!(
        events,
        [
            &Event::Stream { owner: ready, up: Up::Room },
            &Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) },
            &Event::Stream { owner: timed, up: Up::Failed(skein_lib::stream::Fault::Other) }
        ]
    );
    assert!(!machine.is_ready() && completions.is_empty() && !machine.is_due(now));
    assert_eq!(submissions.pop().expect("deadline cancels the write").kind, kernel::Op::Cancel { target: write.op });
}

#[test]
fn a_due_deadline_waits_for_its_submission_room() {
    let limits = limits(1);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut io = io::Io::new(&limits);
    let mut submissions = Queue::with_capacity(io::MAX_OUT_FIRE.submissions);
    let mut completions = Queue::uncapped();
    let mut events = Queue::uncapped();
    let deadline = Duration::from_secs(1);
    let stream = io.adopt_append(kernel::Fd::new(40), deadline).expect("append slot");
    io::down(
        &mut io,
        &env,
        Request::Stream { stream, down: Down::Demand { read: Read::Nothing, room: 1 } },
        &mut submissions,
    );
    io.reclaim();
    io::stage::up(&mut io, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert_eq!(events.pop(), Some(Event::Stream { owner: stream, up: Up::Room }));
    io::down(&mut io, &env, Request::Stream { stream, down: Down::Send(Box::from(b"x".as_slice())) }, &mut submissions);
    let late = Env { now: Time::ZERO.saturating_add(deadline), ..env };
    io::stage::up(&mut io, &late, &mut completions, &mut events, &mut submissions, late.now);
    assert!(io.is_due(late.now), "one free submission cannot hold MAX_OUT_FIRE");
    assert!(events.is_empty());
    let write = submissions.pop().expect("kernel takes the write");
    io::stage::up(&mut io, &late, &mut completions, &mut events, &mut submissions, late.now);
    assert!(!io.is_due(late.now));
    assert_eq!(events.pop(), Some(Event::Stream { owner: stream, up: Up::Failed(skein_lib::stream::Fault::Other) }));
    assert_eq!(submissions.pop().expect("deadline cancels the write").kind, kernel::Op::Cancel { target: write.op });
}

#[test]
fn down_leaves_requests_in_order_when_submissions_have_no_room() {
    let limits = limits(3);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut io = io::Io::new(&limits);
    let mut requests = Queue::uncapped();
    for owner in 1..=3 {
        requests.push(Request::Connect { owner: Token::new(owner), addr: local(9000) });
    }
    let mut too_small = Queue::with_capacity(io::MAX_OUT_DOWN.submissions - 1);
    io::stage::down(&mut io, &env, &mut requests, &mut too_small);
    assert_eq!(requests.len(), 3);
    assert_eq!(io.sockets(), 0);
    assert!(too_small.is_empty());
    let mut submissions = Queue::with_capacity(io::MAX_OUT_DOWN.submissions);
    io::stage::down(&mut io, &env, &mut requests, &mut submissions);
    assert_eq!(io.sockets(), 1, "the first request runs with MAX_OUT_DOWN room");
    assert_eq!(submissions.len(), 1);
    assert_eq!(requests.pop(), Some(Request::Connect { owner: Token::new(2), addr: local(9000) }));
    assert_eq!(requests.pop(), Some(Request::Connect { owner: Token::new(3), addr: local(9000) }));
    assert!(requests.is_empty());
}

#[test]
fn down_leaves_requests_in_order_while_io_cannot_take_a_refusal() {
    let limits = io::Limits { refusals: 1, ..limits(0) };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut io = io::Io::new(&limits);
    let mut requests = Queue::uncapped();
    for owner in 1..=3 {
        requests.push(Request::Connect { owner: Token::new(owner), addr: local(9000) });
    }
    let mut submissions = Queue::with_capacity(io::MAX_OUT_DOWN.submissions);
    io::stage::down(&mut io, &env, &mut requests, &mut submissions);
    assert!(!io.takes());
    assert_eq!(requests.len(), 2);
    io::stage::down(&mut io, &env, &mut requests, &mut submissions);
    assert_eq!(requests.len(), 2, "a full refusal list leaves requests queued");
    let mut completions = Queue::uncapped();
    let mut events = Queue::uncapped();
    io::stage::up(&mut io, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert!(io.takes());
    assert_eq!(events.pop(), Some(Event::Failed { owner: Token::new(1), error: io::Error::Busy }));
    assert_eq!(events.pop(), Some(Event::Closed { owner: Token::new(1) }));
    io::stage::down(&mut io, &env, &mut requests, &mut submissions);
    assert!(!io.takes());
    assert_eq!(requests.pop(), Some(Request::Connect { owner: Token::new(3), addr: local(9000) }));
    assert!(requests.is_empty());
    io::stage::up(&mut io, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert_eq!(events.pop(), Some(Event::Failed { owner: Token::new(2), error: io::Error::Busy }));
    assert_eq!(events.pop(), Some(Event::Closed { owner: Token::new(2) }));
    assert!(events.is_empty() && submissions.is_empty());
}
