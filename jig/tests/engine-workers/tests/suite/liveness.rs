use jig_engine_workers_world::{
    World,
    fixtures::{self, SECRET_A, WORKER_A},
    referee,
};
use jig_protocol_engine_workers::{Event, Request};
use skein_lib::{Duration, Token};

fn world() -> World {
    World::new(&fixtures::limits(), &fixtures::workers(), 1)
}
fn reporting(world: &mut World) -> (Token, Token) {
    world.reporting(WORKER_A, SECRET_A)
}
fn ping(world: &mut World, connection: Token) {
    world.play(connection, fixtures::frame(5, &[]));
    world.run();
}
fn busy(world: &mut World, channel: Token) {
    world.down(Event::TurnBusy { channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
}
fn observed(world: &World, connection: Token) -> Vec<u16> {
    world.observed(connection).iter().map(|r| r.kind).collect()
}
fn lost(world: &World) -> usize {
    world.root().iter().filter(|r| matches!(r, Request::Lost { .. })).count()
}

#[test]
fn a_silent_worker_is_lost_at_the_silence_deadline_and_not_before() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.advance(Duration::from_millis(14_999));
    assert!(!world.gone(connection));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
    referee::judge(world.root()).expect("ordered loss");
}

#[test]
fn a_worker_that_stops_reading_is_lost_by_stall() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    for _ in 0..5 {
        world.advance(Duration::from_secs(5));
        ping(&mut world, connection);
    }
    world.advance(Duration::from_millis(4_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn an_idle_channel_sends_a_ping_after_the_interval() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.advance(Duration::from_millis(4_999));
    assert_eq!((world.observed(connection)).len(), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(observed(&world, connection), [5]);
    assert_eq!(world.root().len(), 1);
}

#[test]
fn a_link_that_fills_its_cap_is_lost() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    let cap = jig_protocol_worker_wire::sizes::engine_cap(&fixtures::SIZES).expect("cap");
    for _ in 0..cap.frames {
        busy(&mut world, channel);
    }
    assert!(!world.gone(connection));
    busy(&mut world, channel);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn a_stream_ending_mid_frame_loses_the_channel_once() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    let frame = fixtures::turn_frame();
    world.inject(connection, &frame.bytes()[..10]);
    world.end_stream(connection);
    world.run();
    world.end_stream(connection);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn a_ping_restarts_the_silence_deadline() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.advance(Duration::from_secs(10));
    ping(&mut world, connection);
    world.advance(Duration::from_millis(14_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
}

#[test]
fn a_drained_record_restarts_the_ping_interval() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.advance(Duration::from_secs(4));
    busy(&mut world, channel);
    world.run();
    world.advance(Duration::from_millis(4_999));
    assert_eq!(observed(&world, connection), [0x0184]);
    world.advance(Duration::from_millis(1));
    assert_eq!(observed(&world, connection), [0x0184, 5]);
}

#[test]
fn a_stream_failure_loses_the_channel_once() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.fail_stream(connection);
    world.run();
    world.fail_stream(connection);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn a_workers_refusal_loses_the_channel_once() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    let frame = skein_channel::control_frame(
        &skein_channel::Control::Refuse { reason: 258, text: Box::new([]) },
        &fixtures::channel_limits(),
    )
    .expect("refusal");
    world.play(connection, frame);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn draining_one_frame_restarts_the_stall_deadline() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    busy(&mut world, channel);
    world.advance(Duration::from_secs(10));
    ping(&mut world, connection);
    world.grant_one_frame(connection);
    world.run();
    for _ in 0..2 {
        world.advance(Duration::from_secs(10));
        ping(&mut world, connection);
    }
    world.advance(Duration::from_millis(9_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table checks the start, refusal order and two-minute ping trace for each boundary row"
)]
fn a_start_past_the_room_is_answered_and_the_link_lives() {
    use jig_domain_host as host;
    use jig_protocol_worker_wire as wire;
    fn linked(room: u32) -> (World, Token, Token) {
        let limits = jig_protocol_engine_workers::Limits {
            sizes: wire::sizes::Sizes { charter: 1024, detail: 128, ..fixtures::SIZES },
            room,
            ..fixtures::LIMITS
        };
        let codec = wire::sizes::limits(&limits.sizes).expect("row codec");
        let schema = wire::schema(&codec).expect("row schema");
        let mut peer = fixtures::peer_with_output(schema, wire::VERSION, WORKER_A, SECRET_A, fixtures::PAYLOAD, room);
        peer.play(fixtures::hello_frame());
        let mut world = World::new(&limits, &fixtures::workers(), 1);
        let connection = world.connect(peer);
        world.run();
        let Request::Hello { channel, .. } = world.root()[0] else { panic!("hello") };
        (world, connection, channel)
    }
    fn record(job: bool) -> host::LinkDown {
        if job {
            host::LinkDown::AssignJob {
                job: host::JobAssignment {
                    run: Token::new(7),
                    attempt: Token::new(2),
                    kind: jig_domain_link::JobKind(1),
                    spec: vec![b'x'; 1024].into(),
                    deadline: Duration::from_secs(60),
                    workspace: None,
                    grants: Box::new([]),
                },
            }
        } else {
            let mut assignment = fixtures::assignment();
            assignment.assignment.charter = vec![b'x'; 1024].into();
            host::LinkDown::Assign { assignment }
        }
    }
    fn send(world: &mut World, channel: Token, job: bool) {
        world.down(Event::Down { channel, down: record(job), parts: fixtures::parts(), values: fixtures::values() });
        world.run();
    }
    let mut failures = Vec::new();
    for (name, job, past) in [
        ("assignment at the room", false, false),
        ("assignment past the room", false, true),
        ("job at the room", true, false),
        ("job past the room", true, true),
    ] {
        let kind = if job { wire::Kind::Job } else { wire::Kind::Assignment };
        let (mut probe, connection, channel) = linked(u32::MAX);
        send(&mut probe, channel, job);
        let body =
            probe.observed(connection).iter().find(|r| r.kind == kind.number()).expect("start encodes").body.clone();
        let bytes = u32::try_from(body.len()).expect("small frame") + 8;
        let room = bytes - u32::from(past);
        let (mut world, connection, channel) = linked(room);
        world.cap_write_room(connection, room);
        world.advance(Duration::from_secs(1));
        send(&mut world, channel, job);
        let mut ping_seconds = Vec::new();
        let mut previous = 0;
        for second in 1..=120 {
            ping(&mut world, connection);
            world.advance(Duration::from_secs(1));
            let count = world.observed(connection).iter().filter(|r| r.kind == 5).count();
            if count > previous {
                ping_seconds.push(second);
            }
            previous = count;
        }
        let starts: Vec<_> = world.observed(connection).iter().filter(|r| r.kind == kind.number()).collect();
        let answers: Vec<_> = world
            .root()
            .iter()
            .filter_map(|r| match r {
                Request::Up { channel: found, up, parts } => Some((*found, up, parts)),
                _ => None,
            })
            .collect();
        let unsent: Vec<_> = world
            .root()
            .iter()
            .filter_map(|r| {
                if let Request::Unsent { worker, channel, unsent } = r {
                    Some((*worker, *channel, *unsent))
                } else {
                    None
                }
            })
            .collect();
        let words =
            format!("{} of {bytes} bytes past the link's room of {room} bytes", if job { "job" } else { "assignment" });
        let refused = match answers.as_slice() {
            [(found, host::LinkUp::Answer { run, attempt, answer }, parts)] => {
                *found == channel
                    && *run == Token::new(7)
                    && *attempt == Token::new(2)
                    && parts.is_empty()
                    && answer.read.is_none()
                    && answer.refused.is_empty()
                    && answer.turns == 0
                    && answer.spent == 0
                    && answer.ending
                        == host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
                            cause: host::StartCause::Other,
                            why: words.as_bytes().into(),
                        }))
            }
            [(found, host::LinkUp::JobAnswer { run, attempt, answer }, parts)] => {
                *found == channel
                    && *run == Token::new(7)
                    && *attempt == Token::new(2)
                    && parts.is_empty()
                    && *answer
                        == host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start {
                            why: words.as_bytes().into(),
                        }))
            }
            _ => false,
        };
        let expected_unsent =
            jig_protocol_engine_workers::Unsent { kind, bytes, bound: jig_protocol_engine_workers::Bound::Room(room) };
        let notified = unsent == [(0, channel, expected_unsent)]
            && matches!(world.root().get(1), Some(Request::Unsent { .. }))
            && matches!(world.root().get(2), Some(Request::Up { .. }));
        let ok = if past {
            starts.is_empty() && refused && notified
        } else {
            starts.len() == 1 && starts[0].body == body && answers.is_empty() && unsent.is_empty()
        };
        let answer_debug = format!("{answers:?}");
        let expected_pings: Vec<_> = (5..=120).step_by(5).collect();
        world.down(Event::Down {
            channel,
            down: host::LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(2) },
            parts: Box::new([]),
            values: Box::new([]),
        });
        world.run();
        let acknowledge = world.observed(connection).iter().any(|r| r.kind == wire::Kind::Acknowledge.number());
        if !ok || ping_seconds != expected_pings || !acknowledge || !world.ended().is_empty() || lost(&world) != 0 {
            failures.push(format!("{name}: answers={answer_debug}, unsent={unsent:?}, pings={ping_seconds:?}, acknowledge={acknowledge}, ended={:?}, lost={}", world.ended(), lost(&world)));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// The resumed start keeps every call across the engine's stream's room
// (jig/docs/design/protocol/channel/README.md, 8, 10 and 11).
#[test]
fn a_resumed_start_fits_the_room() {
    use jig_domain_host as host;
    use jig_protocol_worker_wire as wire;
    // Five 256-byte answers pass this room; two whole answers and three
    // 67-byte cut bodies fit beside the fixture's other assignment fields.
    const ROOM: u32 = 1100;
    const ANSWER: usize = 256;
    let limits = jig_protocol_engine_workers::Limits {
        sizes: wire::sizes::Sizes { answered: 5, answer: 256, ..fixtures::SIZES },
        room: ROOM,
        ..fixtures::LIMITS
    };
    for (calls, whole) in [(3, 3), (5, 2)] {
        let codec = wire::sizes::limits(&limits.sizes).expect("row codec");
        let schema = wire::schema(&codec).expect("row schema");
        let mut peer = fixtures::peer_with_output(schema, wire::VERSION, WORKER_A, SECRET_A, fixtures::PAYLOAD, ROOM);
        peer.play(fixtures::hello_frame());
        let mut world = World::new(&limits, &fixtures::workers(), 1);
        let connection = world.connect(peer);
        world.cap_write_room(connection, ROOM);
        world.run();
        let Request::Hello { channel, .. } = world.root()[0] else { panic!("hello") };
        let mut assignment = fixtures::assignment();
        assignment.answered = (0..calls)
            .map(|index| host::AnsweredCall {
                name: vec![index].into(),
                tool: b"read".as_slice().into(),
                answer: host::SettledAnswer::Host { error: true, body: vec![index; ANSWER].into() },
            })
            .collect();
        world.down(Event::Down {
            channel,
            down: host::LinkDown::Assign { assignment },
            parts: fixtures::parts(),
            values: fixtures::values(),
        });
        world.run();
        let frames: Vec<_> =
            world.observed(connection).iter().filter(|r| r.kind == wire::Kind::Assignment.number()).collect();
        assert_eq!(frames.len(), 1, "{calls} committed calls: {:?}", world.root());
        let frame = frames[0];
        assert!(frame.body.len() + 8 <= usize::try_from(ROOM).expect("small room"));
        let mut reader = skein_lib::Reader::new(&frame.body);
        let record = wire::Assignment::decode(&codec, &mut reader).expect("assignment");
        assert_eq!(record.answered().len(), u32::from(calls));
        for (index, call) in record.answered().iter().enumerate() {
            let index = u8::try_from(index).expect("small call count");
            assert_eq!(call.name(), &[index]);
            assert_eq!(call.tool(), b"read");
            let wire::SettledAnswer::Host(answer) = call.answer() else { panic!("host answer") };
            assert!(answer.error());
            let expected = if index < calls - whole { wire::fit::cut_words(256) } else { vec![index; ANSWER].into() };
            assert_eq!(answer.body(), expected.as_ref(), "{calls} calls, call {index}");
        }
        assert!(!world.root().iter().any(|r| matches!(r, Request::Unsent { .. })));
    }
}

#[test]
fn the_room_holds_the_smallest_resumed_start() {
    use jig_protocol_engine_workers::{Component, Limits, valid};
    use jig_protocol_worker_wire::sizes;
    // Header8 + fixed57 + five calls (fields14 + name16 + tool16 + words66).
    const FLOOR: u32 = 625;
    let sizes = sizes::Sizes { answered: 5, ..fixtures::SIZES };
    for (room, accepted) in [(FLOOR, true), (FLOOR - 1, false)] {
        let limits = Limits { sizes, room, ..fixtures::LIMITS };
        let valid = valid(&limits);
        let made = Component::new(&limits, fixtures::workers(), fixtures::PAYLOAD).is_ok();
        assert_eq!((valid, made), (accepted, accepted), "room {room}: (valid, made)");
    }
}
