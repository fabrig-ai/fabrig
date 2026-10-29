use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures};
use skein_channel::Lower;
use skein_lib::stream;

// Worker-link staging holds three channel cascades; buffer more records.
const BUFFERED_RECORDS: u32 = (skein_channel::MAX_UP + skein_channel::MAX_DOWN) * 3 + 1;
const CALL: &[u8] = b"pending-call";

/// Predicates: `channel_events_dispatched` and `lower_requests_dispatched`.
/// A refused joined channel has queued output and input beyond both staging
/// carriers; its refusal send, finish and terminal precede a later entrance.
#[test]
fn a_channel_cascade_settles_before_the_next_entrance() {
    let mut world = World::new(&fixtures::limits(), fixtures::SEED_ONE);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world.withhold_write_room();
    world.down(Event::Up { up: fixtures::call(CALL), parts: Box::new([]) });
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
    world.buffer_input(fixtures::frame(0x0184, &[255]).bytes());
    for _ in 0..BUFFERED_RECORDS {
        world.buffer_input(fixtures::turn_kept().bytes());
    }
    world.run();
    world.allow_write_room();
    let credit = world.next_lower().expect("refusal output credit");
    let first = world.step_entrance(credit);
    let later = world.step_entrance(Event::CancelRelay {
        run: fixtures::RUN_SEVEN,
        attempt: fixtures::ATTEMPT_TWO,
        call: CALL.into(),
    });
    let buffered = world.buffered_input();
    assert!(
        buffered
            && matches!((first.as_slice(), later.as_slice()), (
        [Request::Lower { connection: send, lower: Lower::Write(stream::OutputDown::Send { bytes, .. }) },
         Request::Lower { connection: finish, lower: Lower::Read(stream::Down::Finish) },
         Request::Close { connection: close }],
        [Request::RelayCancelled { run, attempt, call }]
    ) if send == finish && finish == close && *run == fixtures::RUN_SEVEN
        && *attempt == fixtures::ATTEMPT_TWO && call.as_ref() == CALL
        && skein_channel::parse_header(&bytes[..8]).expect("refusal header").kind == 3),
        "first: {first:?}; later: {later:?}"
    );
    world.dispatch_entrance(first);
    world.dispatch_entrance(later);
    world.down(Event::Stop);
    assert!(matches!(world.root(), [Request::Ready, Request::Lost, Request::RelayCancelled { .. }, Request::Closed]));
}
