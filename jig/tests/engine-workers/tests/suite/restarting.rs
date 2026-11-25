//! The word is gated by the admitted channel (protocol/channel/restart.md, 3).
use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Record;
use skein_lib::{Duration, Reader};

#[test]
fn a_restart_tells_each_worker_that_can_hear_it_and_can_be_asked_again() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (current, _) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    let codec = wire::sizes::limits(&fixtures::limits().sizes).expect("codec");
    let previous = wire::VERSION.checked_sub(1).expect("previous channel version");
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
            let words: Vec<_> =
                world.observed(connection).iter().filter(|r| r.kind == wire::Kind::Restarting.number()).collect();
            assert_eq!(words.len(), count);
            for word in words {
                let record = wire::Restarting::decode(&codec, &mut Reader::new(&word.body)).expect("restart word");
                assert_eq!(record.wait_ms, 1_200_000);
            }
        }
        assert!(!world.observed(older).iter().any(|r| r.kind == wire::Kind::Restarting.number()));
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
    assert!(!world.observed(connection).iter().any(|r| r.kind == wire::Kind::Restarting.number()));
    world.play(connection, fixtures::hello_frame());
    world.withhold_write_room(connection);
    world.down(Event::Close);
    world.down(Event::Restarting { wait: Duration::from_secs(1200) });
    world.run();
    assert!(!world.root().iter().any(|r| matches!(r, Request::Told { .. })));
    assert!(!world.observed(connection).iter().any(|r| r.kind == wire::Kind::Restarting.number()));
}

#[test]
fn a_restart_is_reported_unsent_when_the_layout_lacks_the_word() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let mut peer = fixtures::worker_without(wire::Kind::Restarting);
    peer.play(fixtures::hello_frame());
    let connection = world.connect(peer);
    world.run();
    let _ = super::pause::channel(&world);
    let sent = world.observed(connection).len();
    let before = world.root().len();
    world.down(Event::Restarting { wait: Duration::from_secs(1200) });
    world.run();
    assert_eq!(world.observed(connection).len(), sent);
    let [Request::Told { worker, version, wait: None }] = &world.root()[before..] else { panic!("one untold worker") };
    assert_eq!((worker.as_ref(), *version), (fixtures::WORKER_A, wire::VERSION));
}

#[test]
fn usage_reads_and_hello_capability_follow_the_workers_layout() {
    use jig_domain_host::LinkDown;
    for has_usage in [true, false] {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let codec = wire::sizes::limits(&fixtures::limits().sizes).expect("codec");
        let mut peer = if has_usage {
            fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::APPLICATION_VERSION)
        } else {
            fixtures::worker_without(wire::Kind::ReadUsage)
        };
        peer.play(fixtures::hello_frame());
        let connection = world.connect(peer);
        world.run();
        let (channel, reads_usage) = world
            .root()
            .iter()
            .find_map(|request| match request {
                Request::Hello { channel, hello, .. } => Some((*channel, hello.reads_usage)),
                _ => None,
            })
            .expect("hello");
        assert_eq!(reads_usage, has_usage);
        world.down(Event::Down {
            channel,
            down: LinkDown::ReadUsage {
                read: 42,
                source: jig_domain_link::SourceName::new(b"personal").expect("source"),
            },
            parts: Box::new([]),
            values: Box::new([]),
        });
        world.run();
        let reads: Vec<_> =
            world.observed(connection).iter().filter(|record| record.kind == wire::Kind::ReadUsage.number()).collect();
        assert_eq!(reads.len(), usize::from(has_usage));
        assert!(
            !world.root().iter().any(|request| matches!(request,
            Request::Unsent { unsent, .. } if unsent.kind == wire::Kind::ReadUsage)),
            "a channel without the kind is skipped, not reported as an encoding or translation fault"
        );
        if let Some(record) = reads.first() {
            let record = wire::ReadUsage::decode(&codec, &mut Reader::new(&record.body)).expect("read usage");
            assert_eq!((record.read, record.source.as_ref()), (42, b"personal".as_slice()));
        }
        // A channel without the kind remains usable after declining a usage read.
        world.down(Event::Down {
            channel,
            down: LinkDown::Sources { sources: Box::new([]) },
            parts: Box::new([]),
            values: Box::new([]),
        });
        world.run();
        assert!(world.observed(connection).iter().any(|record| record.kind == wire::Kind::Sources.number()));
    }
}
