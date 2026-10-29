//! What io does beside its two machines (io.md, 2 and 3.1): refusals at a
//! full slab, held until the next up pass; stale tokens, dropped going down
//! and asserted going up; and its admission limits.

use skein_lib::stream::{Down, Read};
use skein_lib::{Duration, Token};

use super::{Kind, Rig, addr, connected, limits, listening, owner};
use crate::kernel::{Complete, Done, Fd, Op};
use crate::{Error, Event, Io, Limits, Name, Request, operations};

#[test]
fn a_connect_past_the_socket_slab_is_refused_and_told_in_the_next_up_pass() {
    let mut rig = Rig::new(Limits { sockets: 1, refusals: 2, ..limits() });
    let _socket = rig.down(Request::Connect { owner: owner(1), addr: addr(80) }).take(Kind::Socket);
    rig.down(Request::Connect { owner: owner(2), addr: addr(80) }).nothing();
    rig.down(Request::Listen { owner: owner(3), addr: addr(0) }).nothing();
    assert!(!rig.io.takes(), "both refusals held: io takes no more requests");
    let mut told = rig.next();
    match told.events.pop() {
        Some(Event::Connecting { owner: connecting, .. }) => assert_eq!(connecting, owner(1)),
        other => panic!("the connect io took is told Connecting last: {other:?}"),
    }
    assert_eq!(
        told.events,
        [
            Event::Failed { owner: owner(2), error: Error::Busy },
            Event::Closed { owner: owner(2) },
            Event::Failed { owner: owner(3), error: Error::Busy },
            Event::Closed { owner: owner(3) },
        ],
        "the refusals first, in order"
    );
    assert!(rig.io.takes(), "told, the refusals are let go");
}

#[test]
fn a_request_naming_a_socket_that_is_gone_is_dropped() {
    let mut rig = Rig::new(limits());
    let (socket, recv) = connected(&mut rig, owner(1), Fd::new(3));
    let cancel = rig.down(Request::Abort { entity: socket }).take(Kind::Cancel);
    rig.complete(recv, Err(crate::kernel::Error::Cancelled)).nothing();
    let close = rig.complete(cancel, Ok(Done::Nothing)).take(Kind::Close);
    assert_eq!(rig.complete(close, Ok(Done::Nothing)).events, [Event::Closed { owner: owner(1) }]);
    // Retired, not yet reclaimed: still found, and closed.
    rig.down(Request::Stream { stream: socket, down: Down::Finish }).nothing();
    rig.down(Request::Abort { entity: socket }).nothing();
    rig.next().nothing();
    // Reclaimed, and the slot taken by another socket: the old token is stale.
    let (_other, _recv) = connected(&mut rig, owner(2), Fd::new(4));
    rig.down(Request::Stream { stream: socket, down: Down::Demand { read: Read::Fill(1), room: 1 } }).nothing();
    rig.down(Request::Close { entity: socket }).nothing();
    rig.down(Request::Bind { socket, owner: owner(3) }).nothing();
    rig.next().nothing();
}

#[test]
#[should_panic(expected = "a completion names an operation in flight")]
fn a_completion_naming_no_operation_in_flight_is_a_bug() {
    let mut rig = Rig::new(limits());
    let mut up = skein_lib::Queue::with_capacity(4);
    let mut subs = skein_lib::Queue::with_capacity(4);
    let complete = Complete { op: Token::new(7), kind: Op::Close { fd: Fd::new(3) }, result: Ok(Done::Nothing) };
    crate::up(&mut rig.io, &rig.env, complete, &mut up, &mut subs);
}

#[test]
#[should_panic(expected = "a completion names an operation in flight")]
fn a_completion_of_an_operation_completed_and_reclaimed_is_a_bug() {
    let mut rig = Rig::new(limits());
    let (_listener, _addr, accept) = listening(&mut rig, owner(1), Fd::new(3));
    let stale = crate::kernel::Submit { op: accept.op, kind: Op::Accept { fd: Fd::new(3) } };
    rig.complete(accept, Err(crate::kernel::Error::NoBufferSpace)).nothing();
    let _retried = rig.next();
    let _out = rig.complete(stale, Ok(Done::Nothing));
}

#[test]
fn the_ring_has_four_operations_per_entity_and_two_independent_slots() {
    let base = limits();
    assert_eq!(operations(&base), Some(10), "four operations an entity, plus process usage and terminal size");
    assert_eq!(operations(&Limits { sockets: u32::MAX, ..base }), None);
}

#[test]
fn the_startup_check_is_the_intake_and_output_caps() {
    let limits = Limits { intake: 300, output: 200, ..limits() };
    assert_eq!((limits.largest_read(), limits.largest_room()), (300, 200));
}

#[test]
#[should_panic(expected = "usable limits")]
fn io_does_not_run_under_limits_it_cannot_use() {
    let _io = Io::new(&Limits { refusals: 0, ..limits() });
}

#[test]
fn every_limit_of_zero_but_the_sockets_is_unusable() {
    let base = limits();
    for unusable in [
        Limits { refusals: 0, ..base },
        Limits { intake: 0, ..base },
        Limits { receive: 0, ..base },
        Limits { output: 0, ..base },
        Limits { sends: 0, ..base },
        Limits { accepts: 0, ..base },
    ] {
        assert!(!unusable.is_usable(), "{unusable:?}");
    }
    assert!(Limits { sockets: 0, backlog: 0, close_timeout: Duration::ZERO, ..base }.is_usable());
}

#[test]
fn usage_has_one_owner_terminal_and_no_entity() {
    for result in [Ok(Done::Usage(crate::kernel::Usage::ZERO)), Err(crate::kernel::Error::Other(5))] {
        let mut rig = Rig::new(Limits { sockets: 0, ..limits() });
        let read = rig.down(Request::Usage { owner: owner(7) }).take(Kind::Usage);
        assert_eq!(rig.io.sockets(), 0);
        assert_eq!(read.kind, Op::Usage);
        let measured = match result {
            Ok(Done::Usage(usage)) => crate::Measured::Read(usage),
            Err(_) => crate::Measured::Unread,
            _ => unreachable!(),
        };
        assert_eq!(rig.complete(read, result).events, [Event::Usage { owner: owner(7), usage: measured }]);
        rig.empty();
    }
}

#[test]
#[should_panic(expected = "one Usage request in flight")]
fn a_second_usage_in_flight_is_a_bug() {
    let mut rig = Rig::new(limits());
    let _read = rig.down(Request::Usage { owner: owner(7) });
    let _other = rig.down(Request::Usage { owner: owner(8) });
}

#[test]
fn usage_can_repeat_in_the_same_iteration_before_flights_reclaim() {
    let mut rig = Rig::new(Limits { sockets: 0, ..limits() });
    for token in 0..2 {
        let read = rig.down(Request::Usage { owner: owner(token) }).take(Kind::Usage);
        assert_eq!(rig.complete(read, Ok(Done::Usage(crate::kernel::Usage::ZERO))).events.len(), 1);
    }
    rig.empty();
}

#[test]
fn the_names_table_holds_every_admitted_listener() {
    // Five distinct names fill the table and the entity slab together.
    const NAMES: u32 = 5;
    let mut rig = Rig::new(Limits { sockets: NAMES, names: NAMES, ..limits() });
    for index in 0..NAMES {
        let addr = super::name(&index.to_le_bytes());
        rig.down(Request::Listen { owner: owner(u64::from(index)), addr }).nothing();
        let _listening = rig.next();
    }
    assert_eq!(rig.io.tables.names.len(), NAMES);
    for index in 0..NAMES {
        let name = Name::new(&index.to_le_bytes()).expect("a four-byte name");
        assert!(rig.io.tables.names.contains_key(&name), "listener {index} retains its name");
    }
}
