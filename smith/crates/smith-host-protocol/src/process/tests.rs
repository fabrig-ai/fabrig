//! Scratch FIFO translation and opening teardown (protocol/hosts.md, section 3).

use super::{Phase, Process, ProcessEvent};
use crate::{OpenEvent, tests};
use skein_channel::{Closed, Lower};
use skein_io::{Event, Request};
use skein_lib::{Queue, Time, Token, stream};

const AGENT: Token = Token::new(7);
const CHILD: Token = Token::new(8);
const INPUT: Token = Token::new(9);
const OUTPUT: Token = Token::new(10);
const ERROR: Token = Token::new(11);

fn opening() -> (Process, Queue<ProcessEvent>, Queue<Request>) {
    let mut process = Process::new(AGENT, &tests::process_limits(), 0, tests::endpoints()).expect("channel");
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    process.spawn(tests::launch(), Time::from_nanos(100), &mut below);
    drop(below.pop());
    process.from_io(
        Time::ZERO,
        Event::Spawned { owner: process.io_owner(), child: CHILD, pipes: Box::new([INPUT, OUTPUT, ERROR]) },
        &mut above,
        &mut below,
    );
    while below.pop().is_some() {}
    (process, above, below)
}

#[test]
fn buffered_opening_operations_translate_in_fifo_order_before_teardown() {
    // The opening read, Open room and Open send are all held.
    const BUFFERED: u32 = 3;
    let (mut process, mut above, mut below) = opening();
    let mut opening_right = None;
    for request in &process.channel_below {
        match request {
            Lower::Write(stream::OutputDown::Room { right, .. }) => opening_right = Some(*right),
            Lower::Read(_) | Lower::Write(_) | Lower::FinishWrite => {}
        }
    }
    let right = opening_right.expect("opening output right");
    process.from_io(
        Time::ZERO,
        Event::Output {
            owner: INPUT,
            up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
        },
        &mut above,
        &mut below,
    );
    process.fire(Time::ZERO, &mut above, &mut below);
    assert_eq!(process.channel_below.len(), BUFFERED);
    while let Some(request) = process.next_channel_down() {
        process.channel_down(request, &mut below);
    }
    let Some(Request::Stream { stream: OUTPUT, down: stream::Down::Demand { .. } }) = below.pop() else {
        panic!("opening read stays first")
    };
    let Some(Request::Output { stream: INPUT, down: stream::OutputDown::Room { .. } }) = below.pop() else {
        panic!("Open room stays before its send")
    };
    let Some(Request::Output { stream: INPUT, down: stream::OutputDown::Send { .. } }) = below.pop() else {
        panic!("Open send stays last")
    };
    assert!(below.is_empty());
    process.fire(Time::from_nanos(100), &mut above, &mut below);
    for entity in [CHILD, INPUT, OUTPUT] {
        assert_eq!(below.pop(), Some(Request::Abort { entity }));
    }
    for owner in [INPUT, OUTPUT, ERROR, process.io_owner()] {
        process.from_io(Time::from_nanos(100), Event::Closed { owner }, &mut above, &mut below);
    }
    assert_eq!(above.pop(), Some(ProcessEvent::Unspawned { agent: AGENT, detail: Box::new([]) }));
    assert!(above.is_empty());
}

#[test]
fn queued_channel_events_keep_opening_sends_and_hangup_order() {
    const FIRST: Token = Token::new(21);
    const SECOND: Token = Token::new(22);
    let (mut process, mut above, mut below) = opening();
    process.channel_events.push(OpenEvent::Opened { version: 5 });
    process.channel_events.push(OpenEvent::Sent { token: FIRST });
    process.channel_events.push(OpenEvent::Sent { token: SECOND });
    process.channel_events.push(OpenEvent::Hangup { why: Closed::Stream });
    process.drain_channel(&mut above, &mut below);
    assert_eq!(process.phase, Phase::Ready);
    assert_eq!(above.pop(), Some(ProcessEvent::Spawned { agent: AGENT, process: CHILD }));
    for event in
        [OpenEvent::Sent { token: FIRST }, OpenEvent::Sent { token: SECOND }, OpenEvent::Hangup { why: Closed::Stream }]
    {
        assert_eq!(above.pop(), Some(ProcessEvent::Channel { agent: AGENT, event: Box::new(event) }));
    }
    assert!(above.is_empty());
    assert!(process.channel_events.is_empty());
    assert_eq!(below.pop(), Some(Request::Close { entity: OUTPUT }));
    assert!(below.is_empty());
}
