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

fn part_answer_at_layout(kind: wire::Kind, has_kind: bool) {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let mut peer = if has_kind {
        fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::APPLICATION_VERSION)
    } else {
        fixtures::worker_without(kind)
    };
    peer.play(fixtures::hello_frame());
    let connection = world.connect(peer);
    world.run();
    let channel = super::pause::channel(&world);
    let sent = world.observed(connection).len();
    let before = world.root().len();
    let event = match kind {
        wire::Kind::PartKept => Event::Down {
            channel,
            down: host::LinkDown::PartKept { run: Token::new(7), attempt: Token::new(2), part: 3 },
            parts: Box::new([]),
            values: Box::new([]),
        },
        wire::Kind::PartBusy => Event::PartBusy { channel, run: Token::new(7), attempt: Token::new(2), part: 4 },
        _ => panic!("a part answer"),
    };
    world.down(event);
    world.run();
    let expected: &[u16] = if has_kind { &[kind.number()] } else { &[] };
    assert_eq!(world.observed(connection)[sent..].iter().map(|r| r.kind).collect::<Vec<_>>(), expected);
    assert!(world.root()[before..].is_empty(), "no diagnostic for an absent optional kind");
}

#[test]
fn part_kept_is_sent_only_at_a_layout_that_has_it() {
    for has_kind in [true, false] {
        part_answer_at_layout(wire::Kind::PartKept, has_kind);
    }
}

#[test]
fn part_busy_is_sent_only_at_a_layout_that_has_it() {
    for has_kind in [true, false] {
        part_answer_at_layout(wire::Kind::PartBusy, has_kind);
    }
}
