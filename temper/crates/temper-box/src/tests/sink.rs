use crate::sink::Sink;
use skein_io::Request;
use skein_lib::{
    Queue, Token, bytes,
    stream::{Down, Fault, Read, Up},
};
#[test]
fn a_sink_sends_no_more_than_the_room_it_was_granted() {
    let token = Token::new(3);
    let mut sink = Sink::new(token, 1);
    let mut out = Queue::with_capacity(4);
    sink.push(bytes::copy_of(b"runner: hello\n")).expect("one queue entry");
    assert_eq!(sink.push(bytes::copy_of(b"full")), Err(bytes::copy_of(b"full")));
    sink.tick(false, &mut out);
    let Some(Request::Stream { stream, down: Down::Demand { read, room } }) = out.pop() else {
        unreachable!("a queued line demands write room");
    };
    assert_eq!((stream, read, room), (token, Read::Nothing, 14));
    assert!(!sink.up(Up::Room, &mut out), "room is no failure");
    let Some(Request::Stream { stream, down: Down::Send(bytes) }) = out.pop() else {
        unreachable!("room permits one send");
    };
    assert_eq!(stream, token);
    assert_eq!(&*bytes, b"runner: hello\n");
    assert_eq!(bytes.len(), 14, "send stays within its grant");
    assert!(out.is_empty(), "one Room sends one entry");
}
#[test]
fn a_failed_sink_is_reported_once() {
    let mut sink = Sink::new(Token::new(3), 1);
    let mut out = Queue::with_capacity(4);
    assert!(sink.up(Up::Failed(Fault::Other), &mut out), "first failure reported");
    assert!(!sink.up(Up::Failed(Fault::Other), &mut out), "duplicate failure suppressed");
    sink.tick(false, &mut out);
    assert_eq!(out.len(), 1, "failed sink closes once");
    sink.tick(false, &mut out);
    assert_eq!(out.len(), 1, "close is not repeated");
}

use crate::{
    sink::Terminal,
    terminal::{Formatter, Item, Target},
};
const STDOUT: Token = Token::new(20);
const STDERR: Token = Token::new(21);
const OUTPUT: u32 = 4099;
const FIRST_ENTRIES: u32 = 3; // setup 7.2: prelude, URL, footer, active included.
fn item(target: Target, words: &[u8]) -> Item {
    Item { target, bytes: bytes::copy_of(words) }
}
fn fifo() -> Terminal {
    Terminal::new(STDOUT, STDERR, FIRST_ENTRIES)
}
fn demand(token: Token, room: u32) -> Request {
    Request::Stream { stream: token, down: Down::Demand { read: Read::Nothing, room } }
}
#[test]
fn terminal_serializes_both_targets_through_full_budget_acknowledgements() {
    let mut terminal = fifo();
    let mut out = Queue::with_capacity(8);
    terminal.push(item(Target::Stderr, b"one\n"));
    terminal.push(item(Target::Stdout, b"two\n"));
    terminal.tick(false, &mut out);
    assert_eq!(out.pop(), Some(demand(STDERR, 4)));
    terminal.tick(false, &mut out);
    assert!(out.is_empty(), "no Send before the small grant");
    assert!(!terminal.up(STDERR, Up::Room, OUTPUT, &mut out));
    assert_eq!(out.pop(), Some(Request::Stream { stream: STDERR, down: Down::Send(bytes::copy_of(b"one\n")) }));
    assert_eq!(out.pop(), Some(demand(STDERR, OUTPUT)));
    terminal.tick(false, &mut out);
    assert!(out.is_empty(), "neither descriptor advances before final ACK");
    assert!(!terminal.up(STDOUT, Up::Room, OUTPUT, &mut out));
    terminal.tick(false, &mut out);
    assert!(out.is_empty(), "another descriptor cannot release the active item");
    assert!(!terminal.up(STDERR, Up::Room, OUTPUT, &mut out));
    terminal.tick(false, &mut out);
    assert_eq!(out.pop(), Some(demand(STDOUT, 4)));
}
#[test]
fn block_admission_counts_the_active_item_and_is_atomic() {
    let mut terminal = fifo();
    let mut out = Queue::with_capacity(8);
    let mut formatter = Formatter::new();
    terminal.push(item(Target::Stderr, b"held\n"));
    terminal.tick(false, &mut out);
    out.pop();
    terminal.up(STDERR, Up::Room, OUTPUT, &mut out);
    out.pop();
    out.pop();
    let block = formatter.block(bytes::copy_of(b"link\n"), skein_lib::Wall::EPOCH);
    assert!(terminal.block(block).is_err(), "two slots cannot admit the first three");
    assert!(!terminal.takes(FIRST_ENTRIES), "active payload has moved but still owns a slot");
    terminal.up(STDERR, Up::Room, OUTPUT, &mut out);
    terminal.block(formatter.block(bytes::copy_of(b"link\n"), skein_lib::Wall::EPOCH)).unwrap();
    formatter.admitted_block();
    assert!(!terminal.takes(1), "the first block fills three adjacent slots");
    // A fresh two-item block also refuses when only one slot is available.
    let mut fresh = fifo();
    fresh.push(item(Target::Stderr, b"one"));
    fresh.push(item(Target::Stdout, b"two"));
    assert!(fresh.block(formatter.block(bytes::copy_of(b"fresh\n"), skein_lib::Wall::EPOCH)).is_err());
    assert!(fresh.takes(1), "refusal added no partial announcement");
}
#[test]
fn terminal_close_waits_for_all_items_and_failure_teardown_settles() {
    for fail in [false, true] {
        let mut terminal = fifo();
        let mut out = Queue::with_capacity(8);
        terminal.push(item(Target::Stdout, b"first"));
        terminal.push(item(Target::Stderr, b"last"));
        terminal.tick(true, &mut out);
        assert_eq!(out.pop(), Some(demand(STDOUT, 5)));
        if fail {
            assert!(terminal.up(STDOUT, Up::Failed(Fault::Other), OUTPUT, &mut out));
            assert!(!terminal.up(STDOUT, Up::Failed(Fault::Other), OUTPUT, &mut out));
        } else {
            for token in [STDOUT, STDERR] {
                terminal.up(token, Up::Room, OUTPUT, &mut out);
                let Some(Request::Stream { down: Down::Send(_), .. }) = out.pop() else { panic!("one item Send") };
                assert_eq!(out.pop(), Some(demand(token, OUTPUT)));
                terminal.tick(true, &mut out);
                assert!(out.is_empty(), "close waits for full-budget ACK");
                terminal.up(token, Up::Room, OUTPUT, &mut out);
                terminal.tick(true, &mut out);
                if token == STDOUT {
                    assert_eq!(out.pop(), Some(demand(STDERR, 4)));
                }
            }
        }
        if fail {
            terminal.tick(true, &mut out);
        }
        assert_eq!(out.pop(), Some(Request::Close { entity: STDOUT }));
        assert_eq!(out.pop(), Some(Request::Close { entity: STDERR }));
        terminal.closed(STDOUT);
        terminal.closed(STDERR);
        assert!(!terminal.pending(true), "all output ownership settled");
    }
}
