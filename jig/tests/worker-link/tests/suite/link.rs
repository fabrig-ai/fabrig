use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world
}
fn hello(world: &mut World) {
    world.down(Event::Hello { hello: fixtures::hello() });
}
fn redial(world: &mut World) {
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(1600));
}
fn ready(world: &World) -> usize {
    world.root().iter().filter(|event| matches!(event, Request::Ready)).count()
}
#[test]
fn a_room_below_a_frame_the_worker_sends_is_not_made() {
    use jig_protocol_worker_link::{Component, Limits, Unusable, valid};
    use jig_protocol_worker_wire::{self as wire, Kind, sizes};

    // A large body makes each named kind exceed the channel's 270-byte refusal.
    const LARGE: u32 = 4096;
    const HEADER: u32 = 8;
    const REFUSE: u32 = 270;
    let base = fixtures::SIZES;
    let turn = sizes::Sizes { turn: LARGE, ..base };
    let call = sizes::Sizes { call_input: LARGE, ..base };
    let answer = sizes::Sizes { result: LARGE, ..base };
    let frame = |sizes: &sizes::Sizes, kind| sizes::largest_body(sizes, kind).expect("finite frame") + HEADER;
    let largest = wire::KINDS
        .iter()
        .filter(|rule| rule.direction == skein_channel::Direction::FromInitiator)
        .map(|rule| frame(&base, rule.body))
        .max()
        .expect("worker sends records")
        .max(REFUSE);
    for (name, sizes, room, made) in [
        ("largest frame fits", base, largest, true),
        ("turn", turn, frame(&turn, Kind::Turn) - 1, false),
        ("call", call, frame(&call, Kind::Call) - 1, false),
        ("answer", answer, frame(&answer, Kind::Answer) - 1, false),
        ("hello", base, frame(&base, Kind::Hello) - 1, false),
        ("refuse", base, REFUSE - 1, false),
    ] {
        let limits = Limits { sizes, room, ..fixtures::LIMITS };
        let result =
            Component::new(&limits, jig_protocol_worker_wire::spoken(), fixtures::credential(), fixtures::SEED_ONE)
                .map(|_| ());
        assert_eq!(result, if made { Ok(()) } else { Err(Unusable::Limits) }, "{name}: room {room}");
        assert_eq!(valid(&limits), made, "{name}: room {room}");
    }
}
#[test]
fn the_hello_is_the_first_record_on_every_channel() {
    let mut world = world();
    for _ in 0..2 {
        world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
        hello(&mut world);
        hello(&mut world);
        world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
        redial(&mut world);
    }
    let records = world.records();
    for channel in &records[..2] {
        assert_eq!(channel.iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100, 0x0101]);
    }
}
#[test]
fn the_root_hears_ready_once_per_channel() {
    let mut world = world();
    assert_eq!(ready(&world), 1);
    hello(&mut world);
    redial(&mut world);
    assert_eq!(ready(&world), 2);
}
#[test]
fn a_lost_channel_is_told_once() {
    let mut world = world();
    world.lose_channel();
    world.lose_channel();
    assert_eq!(world.root().iter().filter(|event| matches!(event, Request::Lost)).count(), 1);
}
#[test]
fn a_turn_while_the_link_is_down_is_dropped() {
    let mut world = world();
    world.lose_channel();
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    hello(&mut world);
    assert_eq!(world.records()[1].iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100]);
}
#[test]
fn a_channel_carrying_the_hello_and_proving_resets_the_backoff() {
    let mut world = world();
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    hello(&mut world);
    world.advance(fixtures::LIMITS.proved);
    let lost = world.now();
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    let gap = world.dials()[2].saturating_since(lost);
    assert!(gap >= Duration::from_millis(50) && gap <= Duration::from_millis(100), "{gap:?}");
}
