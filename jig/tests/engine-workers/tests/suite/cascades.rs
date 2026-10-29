use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, Request};
use skein_channel::Lower;
use skein_lib::{Token, stream};

// More buffered records than either channel staging carrier can hold.
const BUFFERED_RECORDS: u32 = skein_channel::MAX_UP + skein_channel::MAX_DOWN + 1;
const RUN: Token = Token::new(7);
const ATTEMPT: Token = Token::new(2);
const TURN: u32 = 3; // fixtures::turn_record's number.

/// Predicates: `channel_events_dispatched` and `lower_requests_dispatched`.
/// A refused joined channel has queued output and input beyond both staging
/// carriers; its refusal send, finish and terminal precede a later entrance.
#[test]
fn a_channel_cascade_settles_before_the_next_entrance() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, channel) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    let (later_connection, later_channel) = world.reporting(fixtures::WORKER_B, fixtures::SECRET_B);
    world.withhold_write_room(connection);
    world.down(fixtures::assignment_down(channel));
    world.run();
    world.inject(connection, fixtures::frame(0x0101, &[255]).bytes());
    for _ in 0..BUFFERED_RECORDS {
        world.inject(connection, fixtures::turn_frame().bytes());
    }
    world.run();
    world.grant_one_frame(connection);
    let credit = world.next_lower(connection).expect("refusal output credit");
    let first = world.step_entrance(credit);
    let later = world.step_entrance(Event::TurnBusy { channel: later_channel, run: RUN, attempt: ATTEMPT, turn: TURN });
    let buffered = world.buffered_input(connection);
    assert!(
        buffered
            && matches!((first.as_slice(), later.as_slice()), (
        [Request::Lower { connection: send, lower: Lower::Write(stream::OutputDown::Send { bytes, .. }) },
         Request::Lower { connection: finish, lower: Lower::Read(stream::Down::Finish) },
         Request::Close { connection: close }],
        [Request::Lower { connection: next, lower: Lower::Write(stream::OutputDown::Room { .. }) }]
    ) if *send == connection && *finish == connection && *close == connection
        && *next == later_connection && skein_channel::parse_header(&bytes[..8]).expect("refusal header").kind == 3),
        "first: {first:?}; later: {later:?}"
    );
    world.dispatch_entrance(first);
    world.dispatch_entrance(later);
    world.down(Event::Abort);
    world.run();
    assert!(matches!(
        world.root(),
        [Request::Hello { .. }, Request::Hello { .. }, Request::Lost { .. }, Request::Lost { .. }, Request::Closed]
    ));
}
