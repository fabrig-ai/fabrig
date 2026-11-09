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
    assert!(world.observed(connection).is_empty());
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
fn a_start_past_the_room_is_answered_and_the_link_lives() {
    use jig_domain_host as host;
    use jig_protocol_worker_wire as wire;
    fn linked() -> (World, Token, Token) {
        let limits = jig_protocol_engine_workers::Limits {
            sizes: wire::sizes::Sizes { charter: 1024, ..fixtures::SIZES },
            ..fixtures::LIMITS
        };
        let codec = wire::sizes::limits(&limits.sizes).expect("row codec");
        let schema = wire::schema(&codec).expect("row schema");
        let mut peer = fixtures::peer(schema, wire::VERSION, WORKER_A, SECRET_A, fixtures::PAYLOAD);
        peer.play(fixtures::hello_frame());
        let mut world = World::new(&limits, &fixtures::workers(), 1);
        let connection = world.connect(peer);
        world.run();
        let Request::Hello { channel, .. } = world.root()[0] else { panic!("hello") };
        (world, connection, channel)
    }
    fn record(job: bool) -> host::LinkDown {
        if job {
            host::LinkDown::AssignJob { job: host::JobAssignment {
                run: Token::new(7), attempt: Token::new(2), kind: jig_domain_link::JobKind(1),
                spec: vec![b'x'; 1024].into(), deadline: Duration::from_secs(60), workspace: None, grants: Box::new([]),
            }}
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
        ("assignment at the room", false, false), ("assignment past the room", false, true),
        ("job at the room", true, false), ("job past the room", true, true),
    ] {
        let kind = if job { wire::Kind::Job } else { wire::Kind::Assignment };
        let (mut probe, connection, channel) = linked();
        send(&mut probe, channel, job);
        let body = probe.observed(connection).iter().find(|r| r.kind == kind.number()).expect("start encodes").body.clone();
        let bytes = u32::try_from(body.len()).expect("small frame") + 8;
        let room = bytes - u32::from(past);
        let (mut world, connection, channel) = linked();
        world.cap_write_room(connection, room);
        send(&mut world, channel, job);
        world.down(Event::Down { channel, down: host::LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(2) }, parts: Box::new([]), values: Box::new([]) });
        world.run();
        let mut ping_seconds = Vec::new();
        let mut previous = 0;
        for second in 1..=120 {
            ping(&mut world, connection);
            world.advance(Duration::from_secs(1));
            let count = world.observed(connection).iter().filter(|r| r.kind == 5).count();
            if count > previous { ping_seconds.push(second); }
            previous = count;
        }
        let starts: Vec<_> = world.observed(connection).iter().filter(|r| r.kind == kind.number()).collect();
        let answers: Vec<_> = world.root().iter().filter_map(|r| match r {
            Request::Up { up: host::LinkUp::Answer { answer, .. }, .. } => Some(format!("{:?}", answer.ending)),
            Request::Up { up: host::LinkUp::JobAnswer { answer, .. }, .. } => Some(format!("{answer:?}")),
            _ => None,
        }).collect();
        let unsent = world.root().iter().filter(|r| format!("{r:?}").starts_with("Unsent {")).count();
        let words = format!("{} of {bytes} bytes past the link's room of {room} bytes", if job { "job" } else { "assignment" });
        let prefix = &words[..usize::try_from(fixtures::SIZES.detail).expect("detail")];
        let refused = answers.len() == 1 && answers[0].contains("Refused(Invalid(Start")
            && format!("{:?}", world.root()).contains(&format!("{:?}", prefix.as_bytes()));
        let ok = if past { starts.is_empty() && refused && unsent == 1 }
            else { starts.len() == 1 && starts[0].body == body && answers.is_empty() && unsent == 0 };
        let expected_pings: Vec<_> = (5..=120).step_by(5).collect();
        let acknowledge = world.observed(connection).iter().any(|r| r.kind == wire::Kind::Acknowledge.number());
        if !ok || ping_seconds != expected_pings || !acknowledge || !world.ended().is_empty() || lost(&world) != 0 {
            failures.push(format!("{name}: answers={answers:?}, unsent={unsent}, pings={ping_seconds:?}, acknowledge={acknowledge}, ended={:?}, lost={}", world.ended(), lost(&world)));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
