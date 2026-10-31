//! io's stage ordering and admission at the kernel edge (io.md, 2 and 8).

use skein_io::{self as io, Event, Request, kernel, stage};
use skein_io_world::scenarios::{limits, local};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

#[test]
fn the_stage_tells_ready_results_before_completions_and_timers() {
    // A held write leaves room for ready work, but completions and timers
    // still wait for their two kernel submissions.
    const KERNEL_ROOM: u32 = 2;
    const OWNER: Token = Token::new(99);
    const DEADLINE: Duration = Duration::from_secs(1);
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
    assert_eq!(told.iter().collect::<Vec<_>>(), [&Event::Stream { owner: ready, up: Up::Room }]);
    assert!(!machine.is_ready(), "ready work fits beside the held write");
    let blocked = machine.is_due(now) && completions.len() == 1;
    let write = submissions.pop().expect("kernel takes the held write");
    let late = Env { now, ..env };
    stage::up(&mut machine, &late, &mut completions, &mut told, &mut submissions, now);
    let events: Vec<_> = told.iter().collect();
    assert!(blocked, "completions and timers wait at the kernel edge");
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
    assert!(submissions.is_empty(), "the deadline cancels the held write once");
}

#[test]
fn the_up_stage_stops_at_the_kernel_room_and_resumes_next_pass() {
    const KERNEL_ROOM: u32 = 1;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits(2) };
    let mut machine = io::Io::new(&env.limits);
    let first = kernel::Fd::new(40);
    let second = kernel::Fd::new(41);
    machine.adopt_read_pipe(first).expect("first pipe slot");
    machine.adopt_read_pipe(second).expect("second pipe slot");
    machine.reclaim();
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut completions = Queue::uncapped();
    let mut events = Queue::uncapped();

    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert!(machine.is_ready(), "the second pipe waits at the full kernel queue");
    assert_eq!(submissions.len(), KERNEL_ROOM);
    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert!(machine.is_ready(), "another pass without kernel room takes nothing");
    let kernel::Op::PipeRead { fd, .. } = submissions.pop().expect("first pipe read").kind else {
        panic!("a readable pipe starts a read");
    };
    assert_eq!(fd, first);

    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert!(!machine.is_ready(), "the next pass takes the remaining pipe");
    let kernel::Op::PipeRead { fd, .. } = submissions.pop().expect("second pipe read").kind else {
        panic!("a readable pipe starts a read");
    };
    assert_eq!(fd, second);
    assert!(submissions.is_empty() && events.is_empty());
}

#[test]
fn the_up_stage_leaves_a_completion_queued_until_its_kernel_room_is_free() {
    const KERNEL_ROOM: u32 = 2;
    const OWNER: Token = Token::new(99);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits(1) };
    let mut machine = io::Io::new(&env.limits);
    machine.adopt_read_pipe(kernel::Fd::new(40)).expect("pipe slot");
    machine.reclaim();
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut completions = Queue::uncapped();
    let mut events = Queue::uncapped();
    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    let read = submissions.pop().expect("held pipe read");
    io::down(&mut machine, &env, Request::Usage { owner: OWNER }, &mut submissions);
    let usage = submissions.pop().expect("usage query");
    completions.push(kernel::Complete {
        op: usage.op,
        kind: usage.kind,
        result: Ok(kernel::Done::Usage(kernel::Usage::ZERO)),
    });
    submissions.push(read);

    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert_eq!(completions.len(), 1, "one free submission is too few for a completion");
    assert!(events.is_empty());
    drop(submissions.pop().expect("kernel takes the pipe read"));
    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert!(completions.is_empty());
    assert_eq!(events.pop(), Some(Event::Usage { owner: OWNER, usage: io::Measured::Read(kernel::Usage::ZERO) }));
    assert!(events.is_empty() && submissions.is_empty());
}

#[test]
fn the_up_stage_leaves_a_timer_due_until_its_kernel_room_is_free() {
    const KERNEL_ROOM: u32 = 2;
    const DEADLINE: Duration = Duration::from_secs(1);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits(1) };
    let mut machine = io::Io::new(&env.limits);
    let timed = machine.adopt_append(kernel::Fd::new(40), DEADLINE).expect("append slot");
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut completions = Queue::uncapped();
    let mut events = Queue::uncapped();
    io::down(
        &mut machine,
        &env,
        Request::Stream { stream: timed, down: Down::Demand { read: Read::Nothing, room: 1 } },
        &mut submissions,
    );
    machine.reclaim();
    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert_eq!(events.pop(), Some(Event::Stream { owner: timed, up: Up::Room }));
    io::down(
        &mut machine,
        &env,
        Request::Stream { stream: timed, down: Down::Send(Box::from(b"x".as_slice())) },
        &mut submissions,
    );
    let late = Env { now: env.now.saturating_add(DEADLINE), ..env };

    stage::up(&mut machine, &late, &mut completions, &mut events, &mut submissions, late.now);
    assert!(machine.is_due(late.now), "one free submission is too few for a timer");
    assert!(events.is_empty());
    let write = submissions.pop().expect("kernel takes the held append");
    stage::up(&mut machine, &late, &mut completions, &mut events, &mut submissions, late.now);
    assert!(!machine.is_due(late.now));
    assert_eq!(events.pop(), Some(Event::Stream { owner: timed, up: Up::Failed(skein_lib::stream::Fault::Other) }));
    assert_eq!(submissions.pop().expect("deadline cancels the write").kind, kernel::Op::Cancel { target: write.op });
    assert!(events.is_empty() && submissions.is_empty());
}

#[test]
fn the_down_stage_stops_at_the_kernel_room_and_leaves_the_rest_queued() {
    const KERNEL_ROOM: u32 = 2;
    const FIRST: Token = Token::new(98);
    const SECOND: Token = Token::new(99);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits(2) };
    let mut machine = io::Io::new(&env.limits);
    let mut submissions = Queue::with_capacity(KERNEL_ROOM);
    let mut requests = Queue::uncapped();
    requests.push(Request::Connect { owner: FIRST, addr: local(1) });
    requests.push(Request::Connect { owner: SECOND, addr: local(2) });

    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert_eq!(submissions.len(), 1, "the first request leaves too little room for another step");
    assert_eq!(requests.iter().collect::<Vec<_>>(), [&Request::Connect { owner: SECOND, addr: local(2) }]);
    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert_eq!(requests.len(), 1, "another pass without kernel room leaves the request queued");
    drop(submissions.pop().expect("kernel takes the first socket creation"));

    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert!(requests.is_empty(), "the next pass takes the remaining request");
    assert_eq!(submissions.len(), 1);
}

#[test]
fn the_down_stage_leaves_requests_queued_until_io_can_hold_another_refusal() {
    const FIRST: Token = Token::new(98);
    const SECOND: Token = Token::new(99);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: super::limits::LIMITS };
    let mut machine = io::Io::new(&env.limits);
    let mut submissions = Queue::with_capacity(2);
    let mut requests = Queue::uncapped();
    let mut completions = Queue::uncapped();
    let mut events = Queue::uncapped();
    requests.push(Request::Connect { owner: FIRST, addr: local(1) });
    requests.push(Request::Connect { owner: SECOND, addr: local(2) });

    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert!(!machine.takes(), "the first refusal fills io's refusal queue");
    assert_eq!(requests.iter().collect::<Vec<_>>(), [&Request::Connect { owner: SECOND, addr: local(2) }]);
    assert_eq!(submissions.room(), 2, "kernel room does not admit work past the refusal limit");
    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert_eq!(events.pop(), Some(Event::Failed { owner: FIRST, error: io::Error::Busy }));
    assert_eq!(events.pop(), Some(Event::Closed { owner: FIRST }));
    assert!(machine.takes());

    stage::down(&mut machine, &env, &mut requests, &mut submissions);
    assert!(requests.is_empty());
    stage::up(&mut machine, &env, &mut completions, &mut events, &mut submissions, env.now);
    assert_eq!(events.pop(), Some(Event::Failed { owner: SECOND, error: io::Error::Busy }));
    assert_eq!(events.pop(), Some(Event::Closed { owner: SECOND }));
    assert!(events.is_empty() && submissions.is_empty());
}
