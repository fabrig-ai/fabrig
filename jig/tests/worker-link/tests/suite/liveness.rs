use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Loss, Request};
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}
fn lost(world: &World) -> usize {
    world.root().iter().filter(|request| matches!(request, Request::Lost)).count()
}
fn turn(world: &mut World) {
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
}
fn ping(world: &mut World) {
    world.play(fixtures::frame(5, &[]));
}
fn fact(world: &mut World) {
    world.down(Event::Up {
        up: host::LinkUp::Activity {
            run: fixtures::RUN_SEVEN,
            attempt: fixtures::ATTEMPT_TWO,
            activity: host::AgentActivity::LongDone,
        },
        parts: Box::new([]),
    });
}
#[test]
fn a_silent_engine_is_lost_at_the_silence_deadline() {
    let mut world = world();
    world.silence_engine();
    world.advance(Duration::from_millis(14_999));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1);
}
#[test]
fn an_engine_that_stops_reading_is_lost_by_stall() {
    let mut world = world();
    world.withhold_write_room();
    turn(&mut world);
    for _ in 0..5 {
        world.advance(Duration::from_secs(5));
        ping(&mut world);
    }
    world.advance(Duration::from_millis(4_999));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1);
}
#[test]
fn facts_are_dropped_past_the_reserve_while_turns_still_go() {
    let mut world = world();
    world.withhold_write_room();
    for _ in 0..17 {
        fact(&mut world);
    }
    assert_eq!(world.dropped_facts(), 2);
    turn(&mut world);
    assert_eq!(lost(&world), 0);
    world.grant_write_room();
    let channels = world.records();
    assert_eq!(
        channels[0].iter().map(|record| record.kind).collect::<Vec<_>>(),
        [vec![0x0100], vec![0x0104; 15], vec![0x0101]].concat()
    );
}
#[test]
fn an_idle_channel_sends_a_ping_after_the_interval() {
    let mut world = world();
    world.advance(Duration::from_millis(4_999));
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 5).count(), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 5).count(), 1);
}
#[test]
fn a_ping_restarts_the_silence_deadline() {
    let mut world = world();
    world.advance(Duration::from_secs(14));
    ping(&mut world);
    world.advance(Duration::from_millis(14_999));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1);
}
#[test]
fn a_frame_arriving_in_pieces_keeps_the_link_past_the_silence_deadline() {
    // The channel header is eight bytes; the body arrives ten seconds later.
    const HEADER: usize = 8;
    let mut world = world();
    world.silence_engine();
    let frame = fixtures::turn_kept();
    world.advance(Duration::from_secs(10));
    world.buffer_input(&frame.bytes()[..HEADER]);
    world.run();
    world.advance(Duration::from_secs(5));
    assert_eq!(lost(&world), 0, "the header at 10 s keeps the link at 15 s");
    world.advance(Duration::from_secs(5));
    world.buffer_input(&frame.bytes()[HEADER..]);
    world.run();
    world.advance(Duration::from_millis(4_999));
    assert_eq!(lost(&world), 0, "the body at 20 s keeps the link at 24.999 s");
    world.advance(Duration::from_secs(10));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1, "silence ends the link at 35 s");
}
#[test]
fn a_nonfact_that_fills_the_cap_loses_the_link() {
    let mut world = world();
    world.withhold_write_room();
    let cap = jig_protocol_worker_wire::sizes::worker_cap(&fixtures::SIZES).expect("cap");
    for _ in 0..=cap.cap.frames {
        if world.has_channel() {
            turn(&mut world);
        }
    }
    assert_eq!(lost(&world), 1);
}

fn piece(world: &mut World) {
    world.down(Event::Up {
        up: host::LinkUp::Piece {
            run: fixtures::RUN_SEVEN,
            attempt: fixtures::ATTEMPT_TWO,
            piece: host::Piece {
                turn: 1,
                retry: 2,
                kind: host::LiveKind::Text,
                offset: u64::from(u32::MAX) + 9,
                bytes: Box::from(&b"words"[..]),
            },
        },
        parts: Box::new([]),
    });
}
#[test]
fn a_piece_goes_up_as_a_fact_with_every_field() {
    use jig_protocol_worker_wire as wire;
    let mut world = world();
    piece(&mut world);
    assert_eq!(world.dropped_facts(), 0);
    let records = world.records();
    let record = &records[0][1];
    assert_eq!(record.kind, wire::Kind::Fact.number());
    let fact = wire::Fact::decode(&fixtures::codec(), &mut skein_lib::Reader::new(&record.body)).expect("piece fact");
    assert_eq!((fact.task(), fact.attempt()), (7, 2));
    let wire::FactValue::Piece(piece) = fact.value() else { panic!("piece") };
    assert_eq!(
        (piece.turn(), piece.retry(), piece.kind(), piece.offset(), piece.bytes()),
        (1, 2, &wire::LiveKind::Text, u64::from(u32::MAX) + 9, b"words".as_slice())
    );
    assert_eq!(lost(&world), 0);
}
#[test]
fn pieces_past_the_fact_room_are_counted_while_the_next_turn_goes_up() {
    let mut world = world();
    world.withhold_write_room();
    // Text piece frame: task16 + value1 + turn/retry8 + kind1 + offset8 + length4 + bytes5 + header8 =51.
    // The fixture reserves390 bytes, so seven fit and two are dropped.
    for _ in 0..9 {
        piece(&mut world);
    }
    assert_eq!(world.dropped_facts(), 2);
    assert_eq!(world.dropped_records(), 0);
    turn(&mut world);
    assert_eq!(lost(&world), 0);
    world.grant_write_room();
    let records = world.records();
    assert_eq!(
        records[0].iter().map(|record| record.kind).collect::<Vec<_>>(),
        [vec![0x0100], vec![0x0104; 7], vec![0x0101]].concat()
    );
}

fn fill_busy_table(world: &mut World) {
    use jig_protocol_worker_wire as wire;
    // One key per configured slot and retained turn, then one beyond room.
    let room = fixtures::SIZES.slots * fixtures::SIZES.turns;
    for number in 1..=room + 1 {
        let busy = wire::TurnBusy::new(
            &fixtures::codec(),
            wire::TurnBusyParts { task: fixtures::RUN_SEVEN.raw(), attempt: fixtures::ATTEMPT_TWO.raw(), number },
        )
        .expect("busy");
        world.play(fixtures::frame(0x0184, &fixtures::encoded(busy.measure(), |writer| busy.encode(writer))));
    }
}
#[test]
fn each_loss_says_why() {
    struct Case {
        why: Loss,
        setup: fn() -> World,
        end: fn(&mut World),
    }
    let cases = [
        Case {
            why: Loss::Silence,
            setup: world,
            end: |world| {
                world.silence_engine();
                world.advance(fixtures::LIMITS.silence);
            },
        },
        Case {
            why: Loss::Stall,
            setup: world,
            end: |world| {
                world.withhold_write_room();
                turn(world);
                // Keep input alive while no output drains for the full stall span.
                for _ in 0..6 {
                    ping(world);
                    world.advance(fixtures::LIMITS.ping);
                }
            },
        },
        Case {
            why: Loss::Handshake,
            setup: || {
                let mut world = World::new(&fixtures::limits(), fixtures::SEED_ONE);
                world.engine(fixtures::engine_accepting());
                world.silence_next_engine();
                world.run();
                world
            },
            end: |world| world.advance(fixtures::LIMITS.handshake),
        },
        Case {
            why: Loss::Full,
            setup: world,
            end: |world| {
                world.withhold_write_room();
                let cap = jig_protocol_worker_wire::sizes::worker_cap(&fixtures::SIZES).expect("cap");
                for _ in 0..=cap.cap.frames {
                    if world.has_channel() {
                        turn(world);
                    }
                }
            },
        },
        Case { why: Loss::Busy, setup: world, end: fill_busy_table },
        Case { why: Loss::Fault, setup: world, end: |world| world.down(Event::SourceFault) },
        Case {
            why: Loss::Unreachable,
            setup: || {
                let mut world = World::new(&fixtures::limits(), fixtures::SEED_ONE);
                world.refuse_dial();
                world
            },
            end: World::run,
        },
        Case {
            why: Loss::Closed(skein_channel::Closed::RefusedPeer(256)),
            setup: world,
            end: |world| {
                let frame = skein_channel::control_frame(
                    &skein_channel::Control::Refuse { reason: 256, text: Box::new([]) },
                    &fixtures::channel_limits(),
                )
                .expect("refusal");
                world.play(frame);
            },
        },
        Case { why: Loss::Closed(skein_channel::Closed::Stream), setup: world, end: World::lose_channel },
    ];
    for case in cases {
        let mut world = (case.setup)();
        (case.end)(&mut world);
        assert_eq!(world.facts(), [case.why], "{:?}", case.why);
        // Closing, Gone and the ensuing dial must not add a second fact.
        world.advance(fixtures::LIMITS.backoff);
        assert_eq!(world.facts(), [case.why], "{:?} after closing", case.why);
    }
}
