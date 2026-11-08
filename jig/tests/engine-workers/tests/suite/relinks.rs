//! Repeated losses release the connection's room (protocol/channel.md, 10.2 and 12).
use jig_engine_workers_world::{
    World,
    fixtures::{self, SECRET_A, WORKER_A},
};
use jig_protocol_engine_workers::{Event, Request};
use skein_lib::{Duration, Token};

#[derive(Clone, Copy, Debug)]
enum Cut {
    Silence,
    Stall,
    Held,
    Failed,
    Ended,
    NotAdmitted,
    Replaced,
    Framing,
}
const CUTS: [Cut; 8] =
    [Cut::Silence, Cut::Stall, Cut::Held, Cut::Failed, Cut::Ended, Cut::NotAdmitted, Cut::Replaced, Cut::Framing];
fn busy(world: &mut World, channel: Token) {
    world.down(Event::TurnBusy { channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
}
fn cut(world: &mut World, connection: Token, channel: Token, mode: Cut) {
    match mode {
        Cut::Silence => world.advance(fixtures::LIMITS.silence),
        Cut::Stall | Cut::Held => {
            if matches!(mode, Cut::Held) {
                world.hold_send(connection);
            } else {
                world.withhold_write_room(connection);
            }
            busy(world, channel);
            for _ in 0..5 {
                world.advance(Duration::from_secs(5));
                world.play(connection, fixtures::frame(5, &[]));
                world.run();
            }
            world.advance(Duration::from_secs(5));
        }
        Cut::Failed => world.fail_stream(connection),
        Cut::Ended => world.end_stream(connection),
        Cut::NotAdmitted => world.down(Event::NotAdmitted { channel, text: Box::new([]) }),
        Cut::Replaced => {}
        Cut::Framing => world.inject(connection, &[0xff; 8]),
    }
    world.run();
    if !matches!(mode, Cut::Replaced) {
        world.advance(fixtures::LIMITS.stall);
    }
}
#[test]
fn a_worker_lost_fifty_times_each_way_is_taken_back_each_time() {
    let mut failures = Vec::new();
    for mode in CUTS {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let mut connections = Vec::new();
        for _ in 0..50 {
            let connection = world.connect(fixtures::reporting_worker(WORKER_A, SECRET_A));
            world.run();
            let Some(Request::Hello { channel, .. }) = world.root().last() else { break };
            let channel = *channel;
            if world.gone(connection) {
                break;
            }
            connections.push(connection);
            cut(&mut world, connection, channel, mode);
        }
        let hellos = world.root().iter().filter(|r| matches!(r, Request::Hello { .. })).count();
        let losses = world.root().iter().filter(|r| matches!(r, Request::Lost { .. })).count();
        let open = usize::from(matches!(mode, Cut::Replaced));
        let gone = connections.iter().filter(|c| world.gone(**c)).count();
        eprintln!("{mode:?}: hellos={hellos}, lost={losses}, gone={gone}");
        if hellos != 50 || losses != 50 - open || gone != 50 - open {
            failures.push(format!("{mode:?}: hellos={hellos}, lost={losses}, gone={gone}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}
