//! Live timers leave operations in flight alone (protocol/channel/figures.md, 3 and 7).
use jig_engine_workers_world::{World, fixtures as f};
use jig_protocol_engine_workers::{Event, Loss, Request, Timers};
use skein_lib::{Duration, Token};

fn timers() -> Timers {
    Timers { handshake: f::LIMITS.handshake, hello: f::LIMITS.hello, stall: f::LIMITS.stall }
}
fn world() -> World {
    World::new(&f::limits(), &f::workers(), 1)
}
fn opening(world: &mut World) -> Token {
    let connection = world.connect(f::worker(f::WORKER_A, f::SECRET_A, f::APPLICATION_VERSION));
    world.run();
    connection
}
fn silent_opening(world: &mut World) -> Token {
    let mut peer = f::worker(f::WORKER_A, f::SECRET_A, f::APPLICATION_VERSION);
    // Withhold Open, so the handshake never begins at the responder.
    assert!(peer.pop_output().is_some());
    let connection = world.connect(peer);
    world.run();
    connection
}
fn deadline_loss(world: &World, connection: Token) {
    assert!(world.gone(connection));
    assert!(world.ended().iter().any(|r| matches!(r,
        Request::Ended { connection: ended, why: Loss::Deadline, .. } if *ended == connection
    )));
}
#[test]
fn an_opening_keeps_its_hello_deadline_and_the_next_opening_takes_the_new_one() {
    let mut world = world();
    let first = opening(&mut world);
    assert!(world.retime(Timers { hello: Duration::from_secs(2), ..timers() }));
    world.advance(Duration::from_millis(999));
    assert!(!world.gone(first));
    world.advance(Duration::from_millis(1));
    deadline_loss(&world, first);
    let next = opening(&mut world);
    world.advance(Duration::from_millis(1_999));
    assert!(!world.gone(next));
    world.advance(Duration::from_millis(1));
    deadline_loss(&world, next);
    assert!(world.root().is_empty());
}

#[test]
fn a_connection_keeps_its_handshake_deadline_and_the_next_connection_takes_the_new_one() {
    let mut world = world();
    let first = silent_opening(&mut world);
    assert!(world.retime(Timers { handshake: Duration::from_secs(2), ..timers() }));
    world.advance(Duration::from_millis(999));
    assert!(!world.gone(first));
    world.advance(Duration::from_millis(1));
    deadline_loss(&world, first);
    let next = silent_opening(&mut world);
    world.advance(Duration::from_millis(1_999));
    assert!(!world.gone(next));
    world.advance(Duration::from_millis(1));
    deadline_loss(&world, next);
    assert!(world.root().is_empty());
}

#[test]
fn a_link_stays_up_and_its_next_stall_takes_the_new_span() {
    let mut world = world();
    let (connection, channel) = world.reporting(f::WORKER_A, f::SECRET_A);
    let root = world.root().len();
    let records = world.observed(connection).len();
    let openings = world.openings().len();
    assert!(world.retime(Timers { stall: Duration::from_secs(40), ..timers() }));
    world.run();
    assert_eq!(world.root().len(), root);
    assert_eq!(world.observed(connection).len(), records);
    assert_eq!(world.openings().len(), openings);
    assert!(world.ended().is_empty());
    assert!(!world.gone(connection));
    world.withhold_write_room(connection);
    world.down(Event::TurnBusy { channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
    world.run();
    for _ in 0..7 {
        world.advance(Duration::from_secs(5));
        world.play(connection, f::frame(5, &[]));
        world.run();
    }
    assert!(!world.gone(connection), "the old 30 s stall no longer applies");
    world.advance(Duration::from_millis(4_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert!(world.ended().iter().any(|r| matches!(r, Request::Ended { why: Loss::Stall, .. })));
    assert_eq!(world.root().iter().filter(|r| matches!(r, Request::Lost { .. })).count(), 1);
}

#[test]
fn zero_timers_change_neither_the_next_handshake_nor_the_next_hello() {
    let longer =
        Timers { handshake: Duration::from_secs(2), hello: Duration::from_secs(2), stall: Duration::from_secs(40) };
    for invalid in [
        Timers { handshake: Duration::ZERO, ..longer },
        Timers { hello: Duration::ZERO, ..longer },
        Timers { stall: Duration::ZERO, ..longer },
    ] {
        let mut world = world();
        assert!(!world.retime(invalid), "{invalid:?}");
        let handshake = silent_opening(&mut world);
        world.advance(Duration::from_millis(999));
        assert!(!world.gone(handshake));
        world.advance(Duration::from_millis(1));
        deadline_loss(&world, handshake);
        let hello = opening(&mut world);
        world.advance(Duration::from_millis(999));
        assert!(!world.gone(hello));
        world.advance(Duration::from_millis(1));
        deadline_loss(&world, hello);
        assert!(world.root().is_empty());
    }
}
