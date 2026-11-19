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

#[test]
fn usage_reads_and_hello_capability_follow_the_agreed_channel_version() {
    use jig_domain_host::LinkDown;
    use skein_lib::{Secret, Writer};
    for version in [17, 18] {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let codec = wire::sizes::limits(&fixtures::limits().sizes).expect("codec");
        let mut schema = fixtures::schema();
        schema.versions.truncate(u32::from(version - wire::wire::FLOOR + 1));
        let mut wires = fixtures::statement(fixtures::APPLICATION_VERSION);
        for entry in wires.iter_mut() {
            if entry.name() == b"jig-worker-wire" {
                let mut parts = entry.clone().into_parts();
                parts.layouts.truncate(u32::from(version - parts.lowest + 1));
                *entry = wire::Wire::new(&codec, parts).expect("capped statement");
            }
        }
        let credential = wire::Credential::new(
            &codec,
            wire::CredentialParts {
                opening: wire::Opening::Link,
                name: fixtures::WORKER_A.into(),
                secret: Secret::new(fixtures::SECRET_A.into()),
                wires,
                output: fixtures::LIMITS.room,
            },
        )
        .expect("credential");
        let mut bytes = Writer::new(usize::try_from(credential.measure()).expect("credential length"));
        credential.encode(&mut bytes).expect("credential");
        let peer = skein_fake_channel::ScriptedPeer::new(
            schema,
            skein_channel::Role::Initiator,
            fixtures::channel_limits(),
            version,
            bytes.finish(),
        )
        .expect("capped worker");
        let connection = world.connect(peer);
        world.run();
        assert!(world.ready(connection));
        world.play(connection, fixtures::hello_frame());
        world.run();
        let (channel, reads_usage) = world
            .root()
            .iter()
            .find_map(|request| match request {
                Request::Hello { channel, hello, .. } => Some((*channel, hello.reads_usage)),
                _ => None,
            })
            .expect("hello");
        assert_eq!(reads_usage, version == 18);
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
        assert_eq!(reads.len(), usize::from(version == 18));
        assert!(
            !world.root().iter().any(|request| matches!(request,
            Request::Unsent { unsent, .. } if unsent.kind == wire::Kind::ReadUsage)),
            "an older channel is skipped, not reported as an encoding or translation fault"
        );
        if let Some(record) = reads.first() {
            let record = wire::ReadUsage::decode(&codec, &mut Reader::new(&record.body)).expect("read usage");
            assert_eq!((record.read(), record.source()), (42, b"personal".as_slice()));
        }
        // An older channel remains usable after declining a usage read.
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
