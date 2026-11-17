//! The word is gated by the admitted channel (protocol/channel/restart.md, 3).
use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, Request};
use jig_protocol_worker_wire as wire;
use skein_lib::{Duration, Reader};

#[test]
fn a_restart_tells_each_worker_by_its_channels_version_and_can_be_asked_again() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (current, _) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    let codec = wire::sizes::limits(&fixtures::limits().sizes).expect("codec");
    let previous = wire::wire::FLOOR.checked_sub(1).expect("previous channel version");
    let mut schema = fixtures::schema();
    schema.versions.truncate(1);
    schema.versions.get_mut(0).expect("one version").version = previous;
    let old = skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Initiator,
        fixtures::channel_limits(),
        previous,
        Box::new([]),
    )
    .expect("older worker");
    let older = world.connect(old);
    world.run();
    assert_eq!(world.refusal(older).map(|(reason, _)| reason), Some(1));
    let (second, _) = world.reporting(fixtures::WORKER_B, fixtures::SECRET_B);
    assert_eq!(world.root().iter().filter(|r| matches!(r, Request::Hello { .. })).count(), 2);
    let wait = Duration::from_millis(1_200_000);
    for count in 1..=2 {
        world.down(Event::Restarting { wait });
        world.run();
        for connection in [current, second] {
            let words: Vec<_> = world.observed(connection).iter().filter(|r| r.kind == 0x018f).collect();
            assert_eq!(words.len(), count);
            for word in words {
                let record = wire::Restarting::decode(&codec, &mut Reader::new(&word.body)).expect("restart word");
                assert_eq!(record.wait_ms(), 1_200_000);
            }
        }
        assert!(!world.observed(older).iter().any(|r| r.kind == 0x018f));
        let told: Vec<_> = world
            .root()
            .iter()
            .filter_map(|r| match r {
                Request::Told { worker, version, wait } => Some((worker.as_ref(), *version, *wait)),
                _ => None,
            })
            .collect();
        assert_eq!(told.len(), count * 2);
        assert_eq!(told.iter().filter(|t| **t == (fixtures::WORKER_A, wire::VERSION, Some(wait))).count(), count);
        assert_eq!(told.iter().filter(|t| **t == (fixtures::WORKER_B, wire::VERSION, Some(wait))).count(), count);
    }
}

#[test]
fn a_restart_says_nothing_before_hello_or_after_closing() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let connection =
        world.connect(fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::APPLICATION_VERSION));
    world.run();
    assert!(world.ready(connection));
    world.down(Event::Restarting { wait: Duration::from_secs(1200) });
    world.run();
    assert!(!world.root().iter().any(|r| matches!(r, Request::Told { .. })));
    assert!(!world.observed(connection).iter().any(|r| r.kind == 0x018f));
    world.play(connection, fixtures::hello_frame());
    world.withhold_write_room(connection);
    world.down(Event::Close);
    world.down(Event::Restarting { wait: Duration::from_secs(1200) });
    world.run();
    assert!(!world.root().iter().any(|r| matches!(r, Request::Told { .. })));
    assert!(!world.observed(connection).iter().any(|r| r.kind == 0x018f));
}
