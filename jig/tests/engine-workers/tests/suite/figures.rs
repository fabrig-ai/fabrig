//! The two opening purposes (protocol/channel/figures.md, 7).
use jig_engine_workers_world::{World, fixtures as f};
use jig_protocol_engine_workers::{Event, Loss, Request};
use jig_protocol_worker_wire as w;
use skein_lib::{Duration, List, Reader, Writer};

fn figures() -> w::Figures {
    let limit =
        |name: &[u8], value| w::Limit::new(&w::CEILINGS, w::LimitParts { name: name.into(), value }).expect("limit");
    let endpoint = w::AgentEndpoint::new(
        &w::CEILINGS,
        w::AgentEndpointParts {
            name: b"one".as_slice().into(),
            number: 7,
            dialect: 3,
            provider: b"provider".as_slice().into(),
            program: Some(b"program".as_slice().into()),
            host: b"localhost".as_slice().into(),
            port: 443,
            transport: w::Transport::Tls(
                w::Tls::new(
                    &w::CEILINGS,
                    w::TlsParts {
                        server_name: b"localhost".as_slice().into(),
                        trust: Some(b"roots.pem".as_slice().into()),
                    },
                )
                .expect("tls"),
            ),
            headers: List::from_boxed(Box::new([w::Header::new(
                &w::CEILINGS,
                w::HeaderParts { name: b"X-Test".as_slice().into(), value: b"value".as_slice().into() },
            )
            .expect("header")])),
            connect: Duration::from_secs(1),
            handshake: Duration::from_secs(2),
            limits: w::EndpointLimits::new(
                &w::CEILINGS,
                w::EndpointLimitsParts {
                    window: 8192,
                    output: 1024,
                    reasoning_item: 512,
                    head: Duration::from_secs(3),
                    idle: Duration::from_secs(4),
                    oversized: w::Oversized::Drop,
                },
            )
            .expect("endpoint limits"),
        },
    )
    .expect("endpoint");
    let agents = w::Agents::new(
        &w::CEILINGS,
        w::AgentsParts {
            profile: List::from_boxed(Box::new([limit(b"policy.max_turns", 800)])),
            endpoints: List::from_boxed(Box::new([endpoint])),
        },
    )
    .expect("agents");
    w::Figures::new(
        &w::CEILINGS,
        w::FiguresParts { places: 999, limits: List::from_boxed(Box::new([limit(b"hub.slots", 4)])), agents },
    )
    .expect("figures")
}
fn peer(name: &[u8], secret: &[u8], schema: skein_channel::Schema) -> skein_fake_channel::ScriptedPeer {
    stating(name, secret, schema, f::APPLICATION_VERSION, f::LIMITS.room)
}
fn stating(
    name: &[u8],
    secret: &[u8],
    schema: skein_channel::Schema,
    version: u32,
    output: u32,
) -> skein_fake_channel::ScriptedPeer {
    let mut parts = w::Credential::decode(&w::CEILINGS, &mut Reader::new(&f::credential(name, secret, version)))
        .expect("credential")
        .into_parts();
    parts.opening = w::Opening::Figures;
    parts.output = output;
    let record = w::Credential::new(&w::CEILINGS, parts).expect("figures credential");
    let mut bytes = Writer::new(usize::try_from(record.measure()).expect("credential size"));
    record.encode(&mut bytes).expect("credential");
    skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Initiator,
        f::channel_limits(),
        w::VERSION,
        bytes.finish(),
    )
    .expect("figures peer")
}
fn schema() -> skein_channel::Schema {
    let mut sizes = f::SIZES;
    sizes.slots = 3;
    w::schema(&w::sizes::limits(&sizes).expect("three fleet slots")).expect("schema")
}
fn link_peer(name: &[u8], secret: &[u8]) -> skein_fake_channel::ScriptedPeer {
    f::peer(schema(), w::VERSION, name, secret, f::APPLICATION_VERSION)
}
fn world() -> World {
    world_with_room(f::LIMITS.room)
}
fn world_with_room(room: u32) -> World {
    let mut limits = f::limits();
    limits.room = room;
    limits.sizes.slots = 3;
    World::new(&limits, &f::workers(), 1)
}
fn heard(world: &World, connection: skein_lib::Token) -> w::Figures {
    let records = world.observed(connection);
    assert_eq!(
        records.iter().map(|r| r.kind).collect::<Vec<_>>(),
        [w::Kind::Wires.number(), w::Kind::Figures.number()]
    );
    w::Figures::decode(&w::CEILINGS, &mut Reader::new(&records[1].body)).expect("figures")
}
#[test]
fn figures_take_no_place_and_replace_no_live_link() {
    let mut world = world();
    let owner = figures();
    world.down(Event::Figures { figures: owner.clone() });
    let live = world.connect(link_peer(f::WORKER_A, f::SECRET_A));
    let other = world.connect(link_peer(f::WORKER_B, f::SECRET_B));
    world.play(live, f::hello_frame());
    world.play(other, f::hello_frame());
    world.run();
    let root_count = world.root().len();
    let openings = world.openings().len();
    for (name, secret) in [(f::WORKER_A, f::SECRET_A), (f::WORKER_C, f::SECRET_C)] {
        let connection = world.connect(peer(name, secret, schema()));
        world.run();
        let got = heard(&world, connection);
        assert_eq!(got.places(), 3);
        assert_eq!(got.limits(), owner.limits());
        assert_eq!(got.agents(), owner.agents());
        assert!(world.gone(connection), "finished after figures drain");
        assert_eq!(world.refusal(connection), None);
    }
    assert!(!world.gone(live));
    assert!(!world.gone(other));
    assert_eq!(world.root().len(), root_count, "no hello or lost channel reaches the fleet");
    assert!(world.openings()[openings..].iter().all(|r| matches!(r, Request::FiguresTaken { .. })));
}
// A derived engine room and a worker's different stock room (figures.md, 3).
const ENGINE_ROOM: u32 = 1_048_576;
const WORKER_ROOM: u32 = ENGINE_ROOM + 4096;

#[test]
fn figures_with_unequal_output_room_are_answered_and_closed() {
    let mut world = world_with_room(ENGINE_ROOM);
    let owner = figures();
    world.down(Event::Figures { figures: owner.clone() });
    let connection = world.connect(stating(f::WORKER_A, f::SECRET_A, schema(), f::APPLICATION_VERSION, WORKER_ROOM));
    world.run();
    let got = heard(&world, connection);
    assert_eq!(got.places(), 3);
    assert_eq!(got.limits(), owner.limits());
    assert_eq!(got.agents(), owner.agents());
    assert!(world.gone(connection), "finished after figures drain");
    assert_eq!(world.refusal(connection), None, "no skein limits refusal");
    assert!(matches!(world.openings(), [Request::FiguresTaken { worker: 0 }]));
    assert!(
        !world.ended().iter().any(|r| matches!(r, Request::Ended { why: Loss::OutputRoom { .. }, .. })),
        "no output room loss"
    );
    assert!(world.root().is_empty(), "no hello or lost channel reaches the fleet");
}
#[test]
fn figures_refusals_check_authentication_and_wires_before_starting() {
    let mut world = world_with_room(ENGINE_ROOM);
    for (secret, version, expected) in [
        (b"wrong".as_slice(), f::OTHER_VERSION, 256),
        (f::SECRET_A, f::OTHER_VERSION, 257),
        (f::SECRET_A, f::APPLICATION_VERSION, 262),
    ] {
        let connection = world.connect(stating(f::WORKER_A, secret, schema(), version, WORKER_ROOM));
        world.run();
        assert_eq!(world.refusal(connection).expect("refusal").0, expected);
        assert!(world.root().is_empty());
    }
    world.down(Event::Figures { figures: figures() });
    let connection = world.connect(peer(f::WORKER_A, f::SECRET_A, schema()));
    world.run();
    assert_eq!(heard(&world, connection).places(), 3);
}
#[test]
fn figures_terms_require_only_wires_and_figures() {
    let mut world = world();
    world.down(Event::Figures { figures: figures() });
    let mut schema = schema();
    for version in schema.versions.iter_mut() {
        version.kinds = List::from_boxed(
            version
                .kinds
                .iter()
                .filter(|kind| {
                    kind.direction == skein_channel::Direction::FromInitiator
                        || [w::Kind::Wires.number(), w::Kind::Figures.number()].contains(&kind.kind)
                })
                .copied()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
    let connection = world.connect(peer(f::WORKER_A, f::SECRET_A, schema));
    world.run();
    assert_eq!(heard(&world, connection).places(), 3);
    assert!(world.root().is_empty());
}
#[test]
fn link_openings_and_new_figures_keep_the_hello_unchanged() {
    let mut world = world();
    let first = world.connect(link_peer(f::WORKER_A, f::SECRET_A));
    world.run();
    assert_eq!(world.observed(first).iter().map(|r| r.kind).collect::<Vec<_>>(), [w::Kind::Wires.number()]);
    let owner = figures();
    world.down(Event::Figures { figures: owner.clone() });
    world.run();
    assert_eq!(heard(&world, first).limits(), owner.limits());
    let second = world.connect(link_peer(f::WORKER_B, f::SECRET_B));
    world.run();
    assert_eq!(heard(&world, second).agents(), owner.agents());
    world.play(first, f::hello_frame());
    world.play(second, f::hello_frame());
    world.run();
    assert_eq!(world.root().len(), 2);
    assert!(world.root().iter().all(|r| matches!(r, Request::Hello { hello, .. } if hello.slots == 1)));
    world.down(Event::Figures { figures: owner.clone() });
    world.run();
    for connection in [first, second] {
        let records = world.observed(connection);
        assert_eq!(records.len(), 3);
        let got = w::Figures::decode(&w::CEILINGS, &mut Reader::new(&records[2].body)).expect("updated figures");
        assert_eq!(got.agents(), owner.agents());
        assert_eq!(got.places(), 3);
    }
}
