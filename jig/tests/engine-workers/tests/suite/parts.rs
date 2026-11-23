//! Parts wait for the owner's durable acknowledgement (channel/records.md, 5).
use jig_domain_host as host;
use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, Request};
use jig_protocol_worker_wire::{self as wire, Record};
use skein_lib::{Reader, Token};

#[test]
fn a_part_waits_for_its_owner_to_acknowledge_it() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, _) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    let codec = wire::sizes::limits(&fixtures::limits().sizes).unwrap();
    let part = wire::v22::Part {
        task: 7,
        attempt: 2,
        number: 3,
        turn: 1,
        thread: 2,
        call: 4,
        kind: wire::PartKind::Window,
        body: b"opaque".as_slice().into(),
    };
    let before = world.root().len();
    let sent = world.observed(connection).len();
    world.play(connection, fixtures::frame(wire::Kind::Part.number(), &part.encode(&codec).unwrap()));
    world.run();
    let [Request::Up { channel, up: host::LinkUp::Part { run, attempt, part: received }, parts }] =
        &world.root()[before..]
    else {
        panic!("one part handed to the owner")
    };
    assert!(parts.is_empty());
    assert_eq!((*run, *attempt), (Token::new(7), Token::new(2)));
    assert_eq!((received.number, received.turn, received.thread, received.call), (3, 1, 2, 4));
    assert_eq!(received.kind, jig_domain_link::PartKind::Window);
    assert_eq!(received.body.as_ref(), b"opaque");
    assert_eq!(world.observed(connection).len(), sent, "not acknowledged before the owner keeps it");
    let channel = *channel;
    world.down(Event::Down {
        channel,
        down: host::LinkDown::PartKept { run: Token::new(7), attempt: Token::new(2), part: 3 },
        parts: Box::new([]),
        values: Box::new([]),
    });
    world.run();
    let [record] = &world.observed(connection)[sent..] else { panic!("one acknowledgement") };
    assert_eq!(record.kind, wire::Kind::PartKept.number());
    assert_eq!(
        wire::PartKept::decode(&codec, &mut Reader::new(&record.body)).unwrap(),
        wire::PartKept { task: 7, attempt: 2, number: 3 }
    );
}

#[test]
fn part_answers_are_sent_only_where_the_agreed_version_has_them() {
    for version in [21, 22] {
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        let mut peer = super::pause::peer(version);
        peer.play(super::pause::hello(version));
        let connection = world.connect(peer);
        world.run();
        let channel = super::pause::channel(&world);
        let sent = world.observed(connection).len();
        let before = world.root().len();
        world.down(Event::Down {
            channel,
            down: host::LinkDown::PartKept { run: Token::new(7), attempt: Token::new(2), part: 3 },
            parts: Box::new([]),
            values: Box::new([]),
        });
        world.down(Event::PartBusy { channel, run: Token::new(7), attempt: Token::new(2), part: 4 });
        world.run();
        let sent = &world.observed(connection)[sent..];
        if version == 22 {
            assert_eq!(
                sent.iter().map(|r| r.kind).collect::<Vec<_>>(),
                [wire::Kind::PartKept.number(), wire::Kind::PartBusy.number()]
            );
        } else {
            assert_eq!(sent.len(), 0, "no part answer at 21");
        }
        assert!(
            !world.root()[before..].iter().any(|r| matches!(r, Request::Unsent { .. })),
            "an absent optional kind is no mismatch"
        );
    }
}
