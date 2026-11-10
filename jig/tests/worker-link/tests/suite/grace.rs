use jig_protocol_worker_link::{Event, Loss, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, List, Time};
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}
fn queued(world: &mut World) {
    world.withhold_write_room();
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
}
fn pending_stop() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.run();
    world.down(Event::Stop);
    world
}
fn count(world: &World, grace: bool) -> usize {
    world
        .root()
        .iter()
        .filter(
            |request| if grace { matches!(request, Request::GracePassed) } else { matches!(request, Request::Closed) },
        )
        .count()
}
#[test]
fn the_grace_passing_with_no_channel_tells_the_root_once() {
    let mut world = world();
    world.lose_channel();
    world.advance(Duration::from_millis(9_999));
    assert_eq!(count(&world, true), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, true), 1);
    world.advance(Duration::from_secs(20));
    assert_eq!(count(&world, true), 1);
}
#[test]
fn a_channel_back_just_within_the_grace_stops_its_clock() {
    let mut world = world();
    world.lose_channel();
    world.advance(Duration::from_millis(9_999));
    world.engine(fixtures::engine_accepting());
    world.connect_pending();
    world.down(Event::Hello { hello: fixtures::hello() });
    world.play(sources());
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, true), 0);
    assert!(world.has_channel());
}
fn sources() -> skein_channel::Frame {
    let record = wire::Sources::new(&fixtures::codec(), wire::SourcesParts { sources: List::with_capacity(0) })
        .expect("empty sources");
    fixtures::frame(0x0188, &fixtures::encoded(record.measure(), |writer| record.encode(writer)))
}
fn reconnect_after(world: &mut World, span: Duration) {
    world.advance(span);
    world.engine(fixtures::engine_accepting());
    world.connect_pending();
    assert!(world.records().last().expect("new channel").iter().any(|record| record.kind == 0x0100));
}
#[test]
fn refused_hellos_leave_the_grace_running_from_the_first_loss() {
    let mut world = world();
    world.script_root();
    world.lose_channel();
    for _ in 0..3 {
        reconnect_after(&mut world, Duration::from_secs(2));
        world.play(
            skein_channel::control_frame(
                &skein_channel::Control::Refuse { reason: wire::Reason::NotAdmitted.code(), text: Box::new([]) },
                &fixtures::channel_limits(),
            )
            .expect("hello refusal"),
        );
        assert!(!world.has_channel());
    }
    assert_eq!(
        world
            .facts()
            .iter()
            .filter(|why| **why == Loss::Closed(skein_channel::Closed::RefusedPeer(wire::Reason::NotAdmitted.code())))
            .count(),
        3,
    );
    world.advance(Duration::from_millis(3_999));
    assert_eq!(count(&world, true), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, true), 1);
    world.advance(Duration::from_secs(20));
    assert_eq!(count(&world, true), 1);
}
#[test]
fn ready_channels_lost_before_an_engine_record_keep_the_first_grace() {
    let mut world = world();
    world.script_root();
    world.lose_channel();
    for _ in 0..3 {
        reconnect_after(&mut world, Duration::from_secs(2));
        world.lose_channel();
    }
    world.advance(Duration::from_millis(3_999));
    assert_eq!(count(&world, true), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, true), 1);
    world.advance(Duration::from_secs(20));
    assert_eq!(count(&world, true), 1);
}
#[test]
fn the_first_engine_record_stops_the_grace_and_the_next_loss_gets_a_fresh_one() {
    // Sources is the admission record; another decoded record must work too.
    for first in [sources(), fixtures::acknowledge(), fixtures::turn_busy()] {
        let mut world = world();
        world.script_root();
        world.lose_channel();
        reconnect_after(&mut world, Duration::from_millis(9_999));
        assert_eq!(world.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(10))));
        world.play(first);
        world.advance(Duration::from_secs(2));
        assert_eq!(count(&world, true), 0);
        assert!(world.has_channel());
        world.lose_channel();
        world.advance(Duration::from_millis(9_999));
        assert_eq!(count(&world, true), 0);
        world.advance(Duration::from_millis(1));
        assert_eq!(count(&world, true), 1);
        world.advance(Duration::from_secs(20));
        assert_eq!(count(&world, true), 1);
    }
}
#[test]
fn failed_dials_wait_at_most_a_quarter_grace_only_while_it_runs() {
    let limits = jig_protocol_worker_link::Limits {
        grace: Duration::from_secs(60),
        backoff: Duration::from_millis(250),
        ceiling: Duration::from_secs(30),
        proved: Duration::from_secs(60),
        ..fixtures::limits()
    };
    for armed in [false, true] {
        let mut world = World::new(&limits, fixtures::SEED_ONE);
        if armed {
            world.engine(fixtures::engine_accepting());
            world.run();
            world.down(Event::Hello { hello: fixtures::hello() });
        }
        for _ in 0..40 {
            world.refuse_dial();
        }
        if armed {
            world.lose_channel();
        } else {
            world.run();
        }
        world.advance(Duration::from_secs(180));
        let mut capped = 0;
        let mut uncapped = 0;
        for pair in world.dials().windows(2) {
            let gap = pair[1].saturating_since(pair[0]);
            assert!(gap <= limits.ceiling, "{gap:?}");
            if armed && pair[1] < Time::ZERO.saturating_add(limits.grace) {
                assert!(gap <= Duration::from_secs(15), "armed grace: {gap:?}");
                if gap == Duration::from_secs(15) {
                    capped += 1;
                }
            } else if gap > Duration::from_secs(15) {
                uncapped += 1;
            }
        }
        if armed {
            assert!(capped >= 2, "several redials reach the quarter-grace cap");
            assert_eq!(count(&world, true), 1);
        }
        assert!(uncapped > 0, "the backoff ceiling holds without an armed grace");
    }
}
#[test]
fn a_stop_never_redials() {
    let mut world = world();
    world.down(Event::Stop);
    world.advance(Duration::from_secs(60));
    assert_eq!(world.dials().len(), 1);
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
    world.down(Event::Stop);
    world.down(Event::Abort);
    assert_eq!(count(&world, false), 1);
}
#[test]
fn stopping_a_pending_dial_waits_for_its_terminal() {
    let mut world = pending_stop();
    assert_eq!(count(&world, false), 0);
    world.down(Event::Unreachable);
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
}
#[test]
fn an_abort_closes_even_when_output_cannot_drain() {
    let mut world = world();
    queued(&mut world);
    world.down(Event::Abort);
    assert_eq!(count(&world, false), 1);
    assert!(!world.has_channel());
    assert_eq!(world.next_deadline(), None);
}
#[test]
fn a_stop_drains_queued_output_before_closed() {
    let mut world = world();
    queued(&mut world);
    world.down(Event::Stop);
    assert_eq!(count(&world, false), 0);
    world.grant_write_room();
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 0x0101).count(), 1);
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
}

#[test]
fn a_stop_with_stalled_output_is_bounded() {
    let mut world = world();
    queued(&mut world);
    world.down(Event::Stop);
    world.advance(Duration::from_millis(29_999));
    assert_eq!(count(&world, false), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
}

#[test]
fn stopping_a_pending_dial_closes_its_late_connection() {
    let mut world = pending_stop();
    world.engine(fixtures::engine_accepting());
    world.connect_pending();
    assert_eq!(count(&world, false), 1);
    assert!(!world.has_channel());
    assert_eq!(world.next_deadline(), None);
}
