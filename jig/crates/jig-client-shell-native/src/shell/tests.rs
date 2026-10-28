use super::*;
use crate::diagnostics::Kind;
use skein_lib::{Duration, stream::Up};
use skein_world::stream::Wire;

#[test]
fn an_activity_read_waits_for_the_http_slot_its_old_watch_still_owns() {
    let mut shell = Shell::new(Settings {
        engine: io::Addr::Ip("127.0.0.1:8080".parse().expect("engine address")),
        directory: b"client".as_slice().into(),
        limits: crate::tests::limits(),
        io: crate::tests::file_io_limits(),
    });
    // Logical watch cancellation does not complete its native HTTP call.
    for exchange in [1, 2] {
        shell.down(client::Request::Exchange {
            exchange: Token::new(exchange),
            credential: None,
            message: client::Message::Unwatch(b"old watch".as_slice().into()),
        });
    }
    shell.release_held(Time::ZERO);
    shell.down(client::Request::Exchange {
        exchange: Token::new(3),
        credential: None,
        message: client::Message::Read(b"activity".as_slice().into()),
    });
    shell.release_held(Time::ZERO);
    let mut events = Queue::with_capacity(64);
    let mut below = Queue::with_capacity(64);
    let queued = shell.pending();
    assert_eq!(queued.active, 0);
    assert_eq!(queued.queued_kinds.as_slice(), &[Kind::Unwatch, Kind::Unwatch]);
    assert_eq!(queued.held, 1);
    assert!(!format!("{queued:?}").contains("old watch"), "snapshot excludes message bodies");
    shell.transport.connections.take(Time::ZERO, &mut events, &mut below);
    assert!(events.is_empty(), "local HTTP pressure must not report Lost");
    assert!(!shell.held_ready(), "waiting for room does not spin the host");
    let snapshot = shell.pending();
    assert_eq!(snapshot.held, 1);
    assert_eq!(snapshot.held_kinds.as_slice(), &[Kind::Read]);
    assert_eq!(snapshot.active_kinds.as_slice(), &[Kind::Unwatch, Kind::Unwatch]);
    let first = below.iter().next().expect("started HTTP connection").0;
    let mut wire = Wire::new(1);
    wire.write(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n\0");
    for _ in 0..4000 {
        while let Some((conn, down)) = below.pop() {
            if conn == first {
                wire.take(down);
            }
        }
        if let Some(up) = wire.answer() {
            shell.transport.connections.up(Time::ZERO, first, up, &mut events, &mut below);
        }
    }
    assert!(matches!(events.pop(), Some(client::Event::Reply { exchange, .. }) if exchange == Token::new(1)));
    assert!(events.is_empty());
    assert!(shell.held_ready(), "HTTP completion wakes the original demand without a timer or act");
    shell.release_held(Time::ZERO);
    assert!(shell.held.is_empty());
    shell.transport.connections.take(Time::ZERO, &mut events, &mut below);
    let current = below.iter().next().expect("admitted activity read").0;
    assert_ne!(current, first);
    let mut wire = Wire::new(1);
    wire.write(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\ncommitted");
    for _ in 0..4000 {
        while let Some((conn, down)) = below.pop() {
            assert_eq!(conn, current, "only the retained read starts");
            wire.take(down);
        }
        if let Some(up) = wire.answer() {
            shell.transport.connections.up(Time::ZERO, current, up, &mut events, &mut below);
        }
    }
    assert!(matches!(events.pop(), Some(client::Event::Reply { exchange, message })
        if exchange == Token::new(3) && message.as_ref() == b"committed"));
    assert!(events.is_empty(), "exactly one terminal for the original read");
    assert_eq!(
        shell.transport.connections.next_deadline(),
        Some(Time::ZERO.saturating_add(Duration::from_secs(1))),
        "the old call's deadline is unchanged"
    );
    shell.transport.connections.up(Time::ZERO, current, Up::End, &mut events, &mut below);
    assert!(events.is_empty(), "late socket termination does not answer the read twice");
}
