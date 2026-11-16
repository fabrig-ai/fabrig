//! A restart wait belongs to the channel that heard it (protocol/channel/restart.md, 4).
use jig_protocol_worker_link::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, List, Time};

pub(super) fn word(wait_ms: u64) -> skein_channel::Frame {
    let record = wire::Restarting::new(&fixtures::codec(), wire::RestartingParts { wait_ms }).expect("word");
    fixtures::frame(0x018f, &fixtures::encoded(record.measure(), |writer| record.encode(writer)))
}
pub(super) fn sources() -> skein_channel::Frame {
    let record = wire::Sources::new(&fixtures::codec(), wire::SourcesParts { sources: List::with_capacity(0) })
        .expect("sources");
    fixtures::frame(0x0188, &fixtures::encoded(record.measure(), |writer| record.encode(writer)))
}
pub(super) fn limits() -> jig_protocol_worker_link::Limits {
    jig_protocol_worker_link::Limits {
        grace: Duration::from_secs(300),
        backoff: Duration::from_secs(80),
        ceiling: Duration::from_secs(600),
        ..fixtures::limits()
    }
}
fn world() -> World {
    let mut world = World::new(&limits(), 1);
    world.script_root();
    world.engine(fixtures::engine_accepting());
    world.run();
    world
}
pub(super) fn passed(world: &World) -> usize {
    world.root().iter().filter(|r| matches!(r, Request::GracePassed)).count()
}
fn fail_dials(world: &mut World) {
    for _ in 0..80 {
        world.refuse_dial();
    }
}
fn back(world: &mut World) {
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_secs(75));
    if world.dial_pending() {
        world.connect_pending();
    }
    assert!(world.has_channel());
}

#[test]
fn the_last_word_extends_either_loss_and_dials_stay_within_a_quarter_grace() {
    for fault in [false, true] {
        let mut world = world();
        world.advance(Duration::from_secs(2));
        world.play(word(600_000));
        world.play(word(1_200_000));
        let heard: Vec<_> = world
            .root()
            .iter()
            .filter_map(|r| match r {
                Request::Restarting { wait } => Some(*wait),
                _ => None,
            })
            .collect();
        assert_eq!(heard, [Duration::from_secs(600), Duration::from_secs(1200)]);
        fail_dials(&mut world);
        let lost = world.now();
        if fault {
            world.down(Event::SourceFault);
        } else {
            world.lose_channel();
        }
        world.advance(Duration::from_secs(300));
        assert_eq!(passed(&world), 0);
        world.advance(Duration::from_millis(899_999));
        assert_eq!(passed(&world), 0);
        world.advance(Duration::from_millis(1));
        assert_eq!(passed(&world), 1);
        let end = lost.saturating_add(Duration::from_secs(1200));
        let final_dial = Time::from_nanos(end.as_nanos() - limits().handshake.as_nanos());
        assert!(world.dials().contains(&final_dial), "last handshake gets its whole deadline");
        let dials: Vec<_> = world.dials().iter().copied().filter(|at| *at >= lost && *at <= end).collect();
        for pair in dials.windows(2) {
            assert!(pair[1].saturating_since(pair[0]) <= Duration::from_secs(75));
        }
        world.advance(Duration::from_secs(300));
        assert_eq!(passed(&world), 1);
    }
}

#[test]
fn a_word_shorter_than_the_grace_keeps_the_grace() {
    let mut world = world();
    world.play(word(60_000));
    fail_dials(&mut world);
    world.lose_channel();
    world.advance(Duration::from_millis(299_999));
    assert_eq!(passed(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(passed(&world), 1);
}

#[test]
fn a_new_admitted_channel_has_no_predecessors_word() {
    let mut world = world();
    world.play(word(1_200_000));
    world.lose_channel();
    back(&mut world);
    world.play(sources());
    let loss = world.now();
    fail_dials(&mut world);
    world.lose_channel();
    world.advance(Duration::from_millis(299_999));
    assert_eq!(passed(&world), 0, "fresh grace from {loss:?}");
    world.advance(Duration::from_millis(1));
    assert_eq!(passed(&world), 1);
}

#[test]
fn refused_or_unadmitted_channels_leave_the_first_wait_running() {
    for refused in [false, true] {
        let mut world = world();
        world.play(word(1_200_000));
        world.lose_channel();
        back(&mut world);
        fail_dials(&mut world);
        if refused {
            world.play(
                skein_channel::control_frame(
                    &skein_channel::Control::Refuse { reason: wire::Reason::NotAdmitted.code(), text: Box::new([]) },
                    &fixtures::channel_limits(),
                )
                .expect("refusal"),
            );
        } else {
            world.lose_channel();
        }
        let before = Time::ZERO.saturating_add(Duration::from_millis(1_199_999));
        world.advance(before.saturating_since(world.now()));
        assert_eq!(passed(&world), 0);
        world.advance(Duration::from_millis(1));
        assert_eq!(passed(&world), 1);
    }
}
