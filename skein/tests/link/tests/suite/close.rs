use skein_lib::stream::{Down, Fault, Read, Up};
use skein_lib::{Time, bytes};
use skein_link::{Event, Moved, Start};
use skein_link_world::{DEADLINE, NOW, SOCKET, Trace, World};

#[test]
fn due_says_the_close_deadline_has_come() {
    let mut world = World::ready();
    assert!(!world.link.due(DEADLINE), "an open link has no deadline");
    world.room = false;
    world.close(false);
    world.settle();
    assert!(!world.link.due(Time::from_nanos(3_999_999_999)), "one nanosecond before DEADLINE");
    assert!(world.link.due(DEADLINE), "the close is due at its deadline");
    world.link.fire(DEADLINE, &mut world.below);
    assert!(!world.link.due(DEADLINE), "the abort leaves no deadline");
}

#[test]
fn a_graceful_close_sends_close_notify_before_the_socket_closes() {
    let mut world = World::ready();
    world.close(false);
    world.settle();
    assert!(world.wire.server.closed, "the peer reads close_notify");
    let send = world.trace.iter().position(|entry| *entry == Trace::Send).expect("close_notify sent");
    let close = world.trace.iter().position(|entry| *entry == Trace::Close).expect("the socket closes");
    assert!(send < close, "close_notify goes before the socket's Close");
    assert!(world.above.is_empty(), "closing gives nothing above");
    assert!(!world.link.has_work());
    assert_eq!(world.link.next_deadline(), None);
}

fn late_stream(world: &mut World) {
    world.link.down(&world.env, Down::Send(bytes::copy_of(b"late")));
    world.link.up(&world.env, Up::Room);
    world.link.up(&world.env, Up::Bytes(bytes::copy_of(b"late")));
    world.link.up(&world.env, Up::End);
    assert_eq!(world.link.connected(&world.env), Start::Closing);
    world.settle();
}

#[test]
fn nothing_follows_the_sockets_close() {
    let mut world = World::ready();
    world.close(false);
    // A request from above is dropped even while TLS still waits for room.
    world.link.down(&world.env, Down::Finish);
    assert_eq!(world.link.connected(&world.env), Start::Closing);
    world.settle();
    late_stream(&mut world);
    let close = world.trace.iter().position(|entry| *entry == Trace::Close).expect("socket Close");
    assert_eq!(world.trace.get(close + 1..), Some([].as_slice()));
    world.link.closed();
    world.close(true);
    late_stream(&mut world);
    assert_eq!(world.trace.get(close + 1..), Some([].as_slice()));
    assert_eq!(world.link.pump(&mut world.below), Moved::Nothing);
}

#[test]
fn an_abort_sends_nothing() {
    let mut world = World::ready();
    // Leave both queues occupied when the abort arrives.
    world.link.down(&world.env, Down::Demand { read: Read::Nothing, room: 1 });
    world.settle();
    world.link.down(&world.env, Down::Send(bytes::copy_of(b"x")));
    world.link.up(&world.env, Up::End);
    world.close(true);
    assert!(!world.link.has_work(), "abort drops both queues");
    world.trace.clear();
    world.above.clear();
    world.settle();
    world.close(false);
    world.close(true);
    late_stream(&mut world);
    assert_eq!(world.trace, [Trace::Abort]);
    assert_eq!(world.above.len(), 0);
    assert!(!world.wire.server.closed, "abort sends no close_notify");
    assert_eq!(world.link.next_deadline(), None);
}

#[test]
fn an_abort_follows_a_graceful_close() {
    for (flushing, expected) in
        [(true, vec![Trace::Abort]), (false, vec![Trace::Demand, Trace::Send, Trace::Close, Trace::Abort])]
    {
        let mut world = World::ready();
        world.close(false);
        if !flushing {
            world.settle();
        }
        world.close(true);
        assert!(!world.link.has_work(), "abort drops work, flushing={flushing}");
        world.settle();
        assert_eq!(world.trace, expected, "flushing={flushing}");
        assert_eq!(world.above.len(), 0);
        assert_eq!(world.link.next_deadline(), None);
    }
}

#[test]
fn a_close_after_the_peers_close_notify() {
    let mut world = World::ready();
    world.wire.server.close_notify();
    world.wire.pull();
    world.link.down(&world.env, Down::Demand { read: Read::Fill(1), room: 0 });
    world.settle();
    assert_eq!(world.above, [Event::Stream(Up::End)]);
    world.above.clear();
    world.trace.clear();
    world.close(false);
    world.settle();
    assert!(world.wire.server.closed, "the peer reads our close_notify after its own");
    assert_eq!(world.trace.last(), Some(&Trace::Close));
    assert_eq!(world.above.len(), 0);
}

#[test]
fn a_close_before_the_handshake_is_done() {
    let mut world = World::fresh();
    world.link.connecting(SOCKET, &mut world.below);
    assert_eq!(world.link.connected(&world.env), Start::Handshaking);
    world.close(false);
    world.settle();
    assert_eq!(world.trace.last(), Some(&Trace::Close));
    assert!(!world.wire.server.closed, "an unfinished handshake has no close_notify");
    assert_eq!(world.above.len(), 0);
    assert!(!world.link.has_work());
}

#[test]
fn a_close_before_connecting() {
    for (abort, expected) in [(false, Trace::Close), (true, Trace::Abort)] {
        let mut world = World::fresh();
        world.close(false);
        world.settle();
        assert!(world.trace.is_empty(), "no request without a socket token");
        if abort {
            world.close(true);
        }
        world.link.connecting(SOCKET, &mut world.below);
        assert_eq!(world.link.connected(&world.env), Start::Closing);
        world.settle();
        assert_eq!(world.trace, [expected]);
    }
}

#[test]
fn a_second_close_changes_nothing() {
    let mut world = World::ready();
    world.close(false);
    world.close(false);
    world.settle();
    let trace = world.trace.clone();
    world.close(false);
    world.settle();
    assert_eq!(world.trace, trace);
    assert_eq!(world.trace.iter().filter(|entry| **entry == Trace::Close).count(), 1);
    assert!(world.wire.server.closed);
}

#[test]
fn a_close_whose_room_never_comes_is_aborted_at_its_deadline() {
    let mut world = World::ready();
    world.room = false;
    world.close(false);
    world.settle();
    assert_eq!(world.link.next_deadline(), Some(DEADLINE), "now plus close_timeout");
    assert!(!world.link.has_work(), "waiting for room is not work");
    world.link.fire(NOW, &mut world.below);
    world.link.fire(Time::from_nanos(3_999_999_999), &mut world.below);
    assert!(world.below.is_empty(), "nothing before the deadline");
    world.link.fire(DEADLINE, &mut world.below);
    world.settle();
    assert_eq!(world.trace, [Trace::Demand, Trace::Abort]);
    assert_eq!(world.link.next_deadline(), None);
    assert!(!world.link.has_work());
    assert!(!world.wire.server.closed);
    world.link.fire(DEADLINE, &mut world.below);
    assert!(world.below.is_empty(), "the deadline fires once");
}

fn stalled_close() -> World {
    let mut world = World::ready();
    world.room = false;
    world.close(false);
    world.settle();
    world
}

#[test]
fn a_stream_failure_during_a_close_closes_the_socket() {
    let mut world = stalled_close();
    world.link.up(&world.env, Up::Failed(Fault::Reset));
    world.settle();
    assert_eq!(world.trace, [Trace::Demand, Trace::Close]);
    assert!(world.above.is_empty(), "closing gives nothing above");
    assert_eq!(world.link.next_deadline(), None);
    assert!(!world.link.has_work());
}

#[test]
fn a_stream_end_during_a_close_waits_for_its_deadline() {
    let mut world = stalled_close();
    world.link.up(&world.env, Up::End);
    world.settle();
    assert_eq!(world.trace, [Trace::Demand], "the demand for room stays outstanding");
    assert_eq!(world.above.len(), 0);
    assert!(!world.link.has_work());
    assert_eq!(world.link.next_deadline(), Some(DEADLINE));
    world.link.fire(DEADLINE, &mut world.below);
    world.settle();
    assert_eq!(world.trace, [Trace::Demand, Trace::Abort]);
    assert_eq!(world.link.next_deadline(), None);
}

#[test]
fn a_close_after_tls_failed_closes_the_socket() {
    let mut world = World::ready();
    world.room = false;
    world.link.up(&world.env, Up::Failed(Fault::Reset));
    world.settle();
    assert_eq!(world.above, [Event::Stream(Up::Failed(Fault::Reset)), Event::Failed]);
    world.above.clear();
    world.close(false);
    world.settle();
    assert_eq!(world.trace, [Trace::Close]);
    assert!(world.above.is_empty(), "the close gives nothing more above");
    assert_eq!(world.link.next_deadline(), None);
    assert!(!world.link.has_work());
}
