//! Repeated losses release the connection's room (protocol/channel/README.md, 9.2 and 11).
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
// protocol/channel/README.md, 11: fifty losses, each way a link is lost.
const LOSSES: usize = 50;
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
        for _ in 0..LOSSES {
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
        if hellos != LOSSES || losses != LOSSES - open || gone != LOSSES - open {
            failures.push(format!("{mode:?}: hellos={hellos}, lost={losses}, gone={gone}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}

#[derive(Clone, Copy, Debug)]
enum End {
    Cut(Cut),
    Unknown,
    NoRoom,
    Close,
    HelloDeadline,
    HandshakeDeadline,
    Full,
    PeerRefused,
    Gone,
    BeforeAuthentication,
    AfterClosed,
}

fn end(world: &mut World, mode: End) -> (Token, Option<u32>, Option<Token>) {
    match mode {
        End::Unknown => {
            let connection = world.connect(fixtures::reporting_worker(b"unknown", SECRET_A));
            world.run();
            (connection, None, None)
        }
        End::NoRoom => {
            let room = fixtures::LIMITS.workers + fixtures::LIMITS.opening;
            for _ in 0..room {
                world.connect(fixtures::worker(WORKER_A, SECRET_A, fixtures::APPLICATION_VERSION));
            }
            (world.connect(fixtures::reporting_worker(WORKER_A, SECRET_A)), None, None)
        }
        End::HelloDeadline => {
            let connection = world.connect(fixtures::worker(WORKER_A, SECRET_A, fixtures::APPLICATION_VERSION));
            world.run();
            world.advance(fixtures::LIMITS.hello);
            (connection, Some(0), None)
        }
        End::HandshakeDeadline => {
            let connection = world.connect(fixtures::worker(WORKER_A, SECRET_A, fixtures::APPLICATION_VERSION));
            world.withhold_write_room(connection);
            world.run();
            world.advance(fixtures::LIMITS.handshake);
            (connection, Some(0), None)
        }
        End::BeforeAuthentication => {
            let connection = world.connect(fixtures::worker(WORKER_A, SECRET_A, fixtures::APPLICATION_VERSION));
            world.fail_stream(connection);
            (connection, None, None)
        }
        End::AfterClosed => {
            world.down(Event::Close);
            (world.connect(fixtures::reporting_worker(WORKER_A, SECRET_A)), None, None)
        }
        End::Cut(_) | End::Close | End::Full | End::PeerRefused | End::Gone => {
            let (connection, channel) = world.reporting(WORKER_A, SECRET_A);
            match mode {
                End::Cut(Cut::Replaced) => {
                    world.reporting(WORKER_A, SECRET_A);
                }
                End::Cut(mode) => cut(world, connection, channel, mode),
                End::Close => {
                    world.hold_send(connection);
                    busy(world, channel);
                    world.run();
                    world.down(Event::Close);
                    world.advance(fixtures::LIMITS.stall);
                }
                End::Full => {
                    world.withhold_write_room(connection);
                    let cap = jig_protocol_worker_wire::sizes::engine_cap(&fixtures::SIZES).expect("cap");
                    for _ in 0..=cap.frames {
                        busy(world, channel);
                    }
                }
                End::PeerRefused => {
                    let frame = skein_channel::control_frame(
                        &skein_channel::Control::Refuse { reason: 258, text: Box::new([]) },
                        &fixtures::channel_limits(),
                    )
                    .expect("refusal");
                    world.play(connection, frame);
                }
                End::Gone => world.down(Event::Gone { connection }),
                End::Unknown
                | End::NoRoom
                | End::HelloDeadline
                | End::HandshakeDeadline
                | End::BeforeAuthentication
                | End::AfterClosed => unreachable!("opening rows handled above"),
            }
            (connection, Some(0), Some(channel))
        }
    }
}

#[test]
fn each_connections_end_says_why_once() {
    use jig_protocol_engine_workers::Loss;
    use jig_protocol_worker_wire::Reason;
    let rows = [
        (End::Cut(Cut::Silence), Loss::Silence),
        (End::Cut(Cut::Stall), Loss::Stall),
        (End::Cut(Cut::Held), Loss::Stall),
        (End::Cut(Cut::Failed), Loss::Stream),
        (End::Cut(Cut::Ended), Loss::Stream),
        (End::Cut(Cut::NotAdmitted), Loss::Refused { reason: Reason::NotAdmitted.code() }),
        (End::Cut(Cut::Replaced), Loss::Refused { reason: Reason::Replaced.code() }),
        (End::Cut(Cut::Framing), Loss::Framing),
        (End::Unknown, Loss::Refused { reason: Reason::Unauthorized.code() }),
        (End::NoRoom, Loss::NoRoom),
        (End::Close, Loss::Stopping),
        (End::HelloDeadline, Loss::Deadline),
        (End::HandshakeDeadline, Loss::Deadline),
        (End::Full, Loss::Full),
        (End::PeerRefused, Loss::RefusedPeer { reason: 258 }),
        (End::Gone, Loss::Stream),
        (End::BeforeAuthentication, Loss::Stream),
        (End::AfterClosed, Loss::Stopping),
    ];
    for (mode, why) in rows {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let (connection, worker, channel) = end(&mut world, mode);
        world.run();
        // Later lower failures and duplicate gone events must not change the first cause.
        world.fail_stream(connection);
        world.down(Event::Gone { connection });
        world.down(Event::Gone { connection });
        let ended: Vec<_> = world
            .ended()
            .iter()
            .filter_map(|r| match r {
                Request::Ended { connection: c, worker, channel, why } if *c == connection => {
                    Some((*worker, *channel, *why))
                }
                _ => None,
            })
            .collect();
        assert_eq!(ended, [(worker, channel, why)], "{mode:?}");
    }
}
