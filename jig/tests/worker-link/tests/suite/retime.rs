//! Live timers leave operations in flight alone (protocol/channel/figures.md, 4 and 7).
use jig_protocol_worker_link::{Event, Loss, Request, Timers};
use jig_worker_link_world::{World, fixtures as f};
use skein_lib::{Duration, Time};

fn timers() -> Timers {
    Timers {
        handshake: f::LIMITS.handshake,
        stall: f::LIMITS.stall,
        backoff: f::LIMITS.backoff,
        ceiling: f::LIMITS.ceiling,
        proved: f::LIMITS.proved,
        busy: f::LIMITS.busy,
    }
}
fn linked() -> World {
    let mut world = World::new(&f::limits(), 1);
    world.engine(f::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: f::hello() });
    world
}
fn turn(world: &mut World) {
    world.down(Event::Up { up: f::turn(), parts: Box::new([]) });
}
fn resends(world: &World) -> usize {
    world.root().iter().filter(|r| matches!(r, Request::Resend { .. })).count()
}
#[test]
fn a_dial_keeps_its_deadline_and_the_next_dial_takes_the_handshake() {
    let mut world = World::new(&f::limits(), 1);
    world.engine(f::engine_accepting());
    world.silence_next_engine();
    world.run();
    let until = world.next_deadline();
    let root = world.root().len();
    assert!(world.retime(Timers { handshake: Duration::from_secs(2), ..timers() }));
    assert_eq!(world.next_deadline(), until);
    world.run();
    assert_eq!(world.root().len(), root);
    assert_eq!(world.facts(), []);
    assert_eq!(world.dials(), [Time::ZERO]);
    world.advance(Duration::from_millis(999));
    assert_eq!(world.facts(), []);
    world.advance(Duration::from_millis(1));
    assert_eq!(world.facts(), [Loss::Handshake]);
    world.engine(f::engine_accepting());
    world.silence_next_engine();
    world.advance(Duration::from_millis(100));
    let dial = world.dials()[1];
    let deadline = dial.saturating_add(Duration::from_secs(2));
    assert_eq!(world.next_deadline(), Some(deadline));
    world.advance(Duration::from_nanos(deadline.as_nanos().saturating_sub(world.now().as_nanos()).saturating_sub(1)));
    assert_eq!(world.facts().len(), 1);
    world.advance(Duration::from_nanos(1));
    assert_eq!(world.facts(), [Loss::Handshake, Loss::Handshake]);
}

#[test]
fn a_pending_redial_keeps_its_time_and_the_next_wait_takes_backoff_and_ceiling() {
    let mut world = linked();
    world.lose_channel();
    let pending = world.next_deadline();
    assert!(world.retime(Timers { backoff: Duration::from_secs(1), ceiling: Duration::from_secs(1), ..timers() }));
    assert_eq!(world.next_deadline(), pending);
    world.engine(f::engine_accepting());
    world.advance(Duration::from_millis(100));
    assert_eq!(world.dials().len(), 2);
    for _ in 0..3 {
        let lost = world.now();
        world.lose_channel();
        world.engine(f::engine_accepting());
        let count = world.dials().len();
        world.advance(Duration::from_millis(499));
        assert_eq!(world.dials().len(), count);
        world.advance(Duration::from_millis(501));
        assert_eq!(world.dials().len(), count + 1);
        let wait = world.dials().last().expect("redial").saturating_since(lost);
        assert!((Duration::from_millis(500)..=Duration::from_secs(1)).contains(&wait), "{wait:?}");
    }
}

#[test]
fn a_link_stays_up_and_its_next_stall_takes_the_new_span() {
    let mut world = linked();
    let root = world.root().len();
    let records = world.records()[0].len();
    let deadline = world.next_deadline();
    assert!(world.retime(Timers { stall: Duration::from_secs(40), ..timers() }));
    assert_eq!(world.next_deadline(), deadline);
    world.run();
    assert_eq!(world.root().len(), root);
    assert_eq!(world.records()[0].len(), records);
    assert!(world.has_channel());
    assert_eq!(world.facts(), []);
    assert_eq!(world.dials().len(), 1);
    world.withhold_write_room();
    turn(&mut world);
    for _ in 0..7 {
        world.advance(Duration::from_secs(5));
        assert_eq!(world.facts(), [], "the new stall has not elapsed");
        world.play(f::frame(5, &[]));
    }
    assert_eq!(world.facts(), [], "the old 30 s stall no longer applies");
    world.advance(Duration::from_millis(4_999));
    assert_eq!(world.facts(), []);
    world.advance(Duration::from_millis(1));
    assert_eq!(world.facts(), [Loss::Stall]);
}

#[test]
fn a_busy_span_keeps_its_deadline_and_the_next_add_takes_the_new_span() {
    let mut world = linked();
    turn(&mut world);
    world.play(f::turn_busy());
    assert!(world.retime(Timers { busy: Duration::from_secs(1), ..timers() }));
    world.advance(Duration::from_millis(200));
    assert_eq!(resends(&world), 1, "the armed busy deadline stays");
    turn(&mut world);
    world.play(f::turn_busy());
    world.advance(Duration::from_millis(499));
    assert_eq!(resends(&world), 1);
    world.advance(Duration::from_millis(501));
    assert_eq!(resends(&world), 2);
    assert_eq!(world.facts(), []);
    assert_eq!(world.dials().len(), 1);
}

#[test]
fn the_next_proving_uses_the_new_span_before_resetting_backoff() {
    let mut world = linked();
    assert!(world.retime(Timers { proved: Duration::from_secs(4), ..timers() }));
    world.lose_channel();
    world.engine(f::engine_accepting());
    world.advance(Duration::from_millis(100));
    world.down(Event::Hello { hello: f::hello() });
    world.advance(Duration::from_secs(2));
    world.lose_channel();
    let retry = world.next_deadline().expect("redial wait").saturating_since(world.now());
    assert!((Duration::from_millis(100)..=Duration::from_millis(200)).contains(&retry), "{retry:?}");
    world.engine(f::engine_accepting());
    world.advance(Duration::from_millis(200));
    world.down(Event::Hello { hello: f::hello() });
    world.advance(Duration::from_secs(4));
    world.lose_channel();
    let retry = world.next_deadline().expect("redial wait").saturating_since(world.now());
    assert!((Duration::from_millis(50)..=Duration::from_millis(100)).contains(&retry), "{retry:?}");
}

#[test]
fn invalid_timers_leave_the_next_handshake_unchanged() {
    let longer = Timers { handshake: Duration::from_secs(2), ..timers() };
    for invalid in [
        Timers { handshake: Duration::ZERO, ..longer },
        Timers { stall: Duration::ZERO, ..longer },
        Timers { backoff: Duration::ZERO, ..longer },
        Timers { ceiling: Duration::ZERO, ..longer },
        Timers { proved: Duration::ZERO, ..longer },
        Timers { busy: Duration::ZERO, ..longer },
        Timers { backoff: Duration::from_secs(2), ceiling: Duration::from_secs(1), ..longer },
    ] {
        let mut world = linked();
        world.lose_channel();
        let deadline = world.next_deadline();
        assert!(!world.retime(invalid), "{invalid:?}");
        assert_eq!(world.next_deadline(), deadline);
        world.engine(f::engine_accepting());
        world.silence_next_engine();
        world.advance(Duration::from_millis(100));
        let dial = *world.dials().last().expect("redial");
        assert_eq!(world.next_deadline(), Some(dial.saturating_add(f::LIMITS.handshake)), "{invalid:?}");
        assert_eq!(world.facts(), [Loss::Closed(skein_channel::Closed::Stream)]);
    }
}
