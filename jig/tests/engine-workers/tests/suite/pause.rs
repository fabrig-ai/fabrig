//! Pause asks and reports at the admitted channel (protocol/channel/pause.md, 3).
use jig_domain_host::{LinkDown, LinkUp};
use jig_domain_link::{PauseMode, PauseState};
use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, Request};
use jig_protocol_worker_wire::{self as wire, Record};
use skein_lib::{Reader, Token};

fn pause(world: &mut World, channel: Token, mode: PauseMode) {
    world.down(Event::Down { channel, down: LinkDown::Pause { mode }, parts: Box::new([]), values: Box::new([]) });
    world.run();
}
fn peer(has_pause: bool) -> skein_fake_channel::ScriptedPeer {
    if has_pause {
        fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::APPLICATION_VERSION)
    } else {
        fixtures::worker_without(wire::Kind::Pause)
    }
}
pub(super) fn channel(world: &World) -> Token {
    world
        .root()
        .iter()
        .find_map(|request| match request {
            Request::Hello { channel, .. } => Some(*channel),
            _ => None,
        })
        .expect("admitted hello")
}

#[test]
fn every_pause_ask_is_sent_or_reported_once_by_the_workers_layout() {
    for has_pause in [true, false] {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let mut worker = peer(has_pause);
        worker.play(fixtures::hello_frame());
        let connection = world.connect(worker);
        world.run();
        let channel = channel(&world);
        let codec = wire::sizes::limits(&fixtures::limits().sizes).expect("codec");
        for (mode, value) in [
            (PauseMode::Resume, wire::PauseMode::Resume),
            (PauseMode::Pause, wire::PauseMode::Pause),
            (PauseMode::Drain, wire::PauseMode::Drain),
        ] {
            let before = world.root().len();
            let sent = world.observed(connection).len();
            pause(&mut world, channel, mode);
            if has_pause {
                let frames = &world.observed(connection)[sent..];
                assert_eq!(frames.len(), 1);
                assert_eq!(frames[0].kind, wire::Kind::Pause.number());
                assert_eq!(wire::Pause::decode(&codec, &mut Reader::new(&frames[0].body)).expect("pause").mode, value);
                assert_eq!(world.root().len(), before);
            } else {
                assert_eq!(world.observed(connection).len(), sent);
                let [Request::Unpaused { worker, version: got, mode: asked }] = &world.root()[before..] else {
                    panic!("one outcome per ask")
                };
                assert_eq!(worker.as_ref(), fixtures::WORKER_A);
                assert_eq!((*got, *asked), (wire::VERSION, mode));
            }
        }
    }
}

#[test]
fn a_pause_sends_and_reports_nothing_before_hello_or_after_closing() {
    for has_pause in [true, false] {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let connection = world.connect(peer(has_pause));
        world.run();
        assert!(world.ready(connection));
        // The pending channel's token is not yet an owner-visible admission.
        pause(&mut world, Token::new(1), PauseMode::Drain);
        assert!(!world.root().iter().any(|r| matches!(r, Request::Unpaused { .. })));
        assert!(!world.observed(connection).iter().any(|r| r.kind == wire::Kind::Pause.number()));
        world.play(connection, fixtures::hello_frame());
        world.run();
        let channel = channel(&world);
        world.withhold_write_room(connection);
        world.down(Event::Close);
        pause(&mut world, channel, PauseMode::Pause);
        assert!(!world.root().iter().any(|r| matches!(r, Request::Unpaused { .. })));
        assert!(!world.observed(connection).iter().any(|r| r.kind == wire::Kind::Pause.number()));
    }
}

#[test]
fn every_paused_state_reaches_the_owner() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, channel) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    let codec = wire::sizes::limits(&fixtures::limits().sizes).expect("codec");
    for (state, value) in [
        (wire::PauseState::Taking, PauseState::Taking),
        (wire::PauseState::Paused, PauseState::Paused),
        (wire::PauseState::Draining, PauseState::Draining),
        (wire::PauseState::Drained, PauseState::Drained),
    ] {
        let before = world.root().len();
        let bytes = wire::Paused { state }.encode(&codec).expect("paused");
        world.play(connection, fixtures::frame(0x0110, &bytes));
        world.run();
        assert_eq!(world.root().len(), before + 1);
        let Request::Up { channel: got, up, parts } = world.root().last().expect("paused up") else {
            panic!("paused up")
        };
        assert_eq!(*got, channel);
        assert_eq!(*up, LinkUp::Paused { state: value });
        assert!(parts.is_empty());
    }
}
