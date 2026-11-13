//! Independent literals for net/channel.md, sections 3 and 5.1.
#![expect(clippy::disallowed_types, reason = "test harnesses use ordinary Rust collections")]
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use skein_lib::{List, Queue, Token, stream};

use super::{
    Control, Direction, Event, FrameError, Kind, Limits, Lower, LowerEvent, Machine, Request, Role, Room, Schema,
    SchemaError, StreamMode, Term, Unsent, Version, control_frame, decode_control, frame_writer, parse_header,
};

fn limits() -> Limits {
    Limits { chunk: 16, credential: 64, skip: 128, output_bytes: 1024, output_frames: 4, kinds: 4 }
}

fn schema() -> Schema {
    let mut kinds = List::with_capacity(2);
    kinds.push(Kind { kind: 0x0100, direction: Direction::FromInitiator, largest: 32 }).expect("kind fits");
    let mut versions = List::with_capacity(2);
    versions.push(Version { version: 1, kinds }).expect("version fits");
    let mut kinds = List::with_capacity(2);
    kinds.push(Kind { kind: 0x0101, direction: Direction::FromResponder, largest: 64 }).expect("kind fits");
    versions.push(Version { version: 2, kinds }).expect("version fits");
    Schema { magic: *b"test", versions }
}

#[test]
fn checked_schema_rejects_gaps_duplicates_reserved_and_oversize() {
    let mut source = schema();
    assert_eq!(source.check(Role::Responder, &limits()), Ok(()));
    assert_eq!(source.range(), Some((1, 2)));
    assert_eq!(source.version(2).expect("version 2").kind(0x0101).expect("kind").largest, 64);
    source.versions.get_mut(1).expect("second version").version = 3;
    assert_eq!(source.check(Role::Responder, &limits()), Err(SchemaError::Versions));
    source.versions.get_mut(1).expect("second version").version = 2;
    source
        .versions
        .get_mut(1)
        .expect("second version")
        .kinds
        .push(Kind { kind: 0x0101, direction: Direction::FromResponder, largest: 1 })
        .expect("second kind");
    assert_eq!(source.check(Role::Responder, &limits()), Err(SchemaError::Kind));
    source.versions.get_mut(1).expect("second version").kinds.get_mut(1).expect("second kind").kind = 6;
    assert_eq!(source.check(Role::Responder, &limits()), Err(SchemaError::Kind));
    source.versions.get_mut(1).expect("second version").kinds.get_mut(1).expect("second kind").kind = 0x0102;
    source.versions.get_mut(1).expect("second version").kinds.get_mut(1).expect("second kind").largest = 1017;
    assert_eq!(source.check(Role::Responder, &limits()), Err(SchemaError::Kind));
}

#[test]
fn golden_six_control_frames_round_trip() {
    let mut terms = List::with_capacity(1);
    terms.push(Term { kind: 0x0100, largest: 32 }).expect("one term");
    let cases = [
        (
            Control::Open { magic: *b"test", lowest: 1, highest: 2, features: 0, credential: Box::from(*b"xy") },
            &[0, 1, 0, 0, 0, 0, 0, 16, 116, 101, 115, 116, 0, 1, 0, 2, 0, 0, 0, 0, 0, 2, 120, 121][..],
        ),
        (Control::Accept { version: 2, features: 0 }, &[0, 2, 0, 0, 0, 0, 0, 4, 0, 2, 0, 0][..]),
        (
            Control::Refuse { reason: 3, text: Box::from(*b"bad") },
            &[0, 3, 0, 0, 0, 0, 0, 9, 0, 3, 0, 0, 0, 3, 98, 97, 100][..],
        ),
        (Control::Terms { entries: terms }, &[0, 4, 0, 0, 0, 0, 0, 10, 0, 0, 0, 1, 1, 0, 0, 0, 0, 32][..]),
        (Control::Ping, &[0, 5, 0, 0, 0, 0, 0, 0][..]),
        (Control::Unsupported { kind: 0x0100 }, &[0, 6, 0, 0, 0, 0, 0, 2, 1, 0][..]),
    ];
    for (control, expected) in cases {
        let frame = control_frame(&control, &limits()).expect("control encodes");
        assert_eq!(frame.bytes(), expected);
        let header = parse_header(frame.bytes().get(..8).expect("header")).expect("valid header");
        assert_eq!(header.kind, frame.kind());
        assert_eq!(header.body_len, frame.body_len());
        let body = frame.bytes().get(8..).expect("body");
        let decoded = decode_control(header.kind, body, &limits()).expect("control decodes");
        let encoded_again = control_frame(&decoded, &limits()).expect("control reencodes");
        assert_eq!(encoded_again.bytes(), expected);
    }
}

#[test]
fn malformed_control_and_incomplete_writer_are_refused() {
    assert_eq!(parse_header(&[0, 5, 0, 1, 0, 0, 0, 0]), Err(FrameError::Malformed));
    assert_eq!(decode_control(4, &[0, 0, 0, 2], &limits()).err(), Some(FrameError::Malformed));
    assert_eq!(decode_control(3, &[0, 3, 0, 0, 1, 1], &limits()).err(), Some(FrameError::TooLarge));
    let mut writer = frame_writer(0x0100, 3).expect("measured frame");
    writer.put(b"ab").expect("two bytes fit");
    assert_eq!(writer.finish().err(), Some(FrameError::Incomplete));
}

struct Peer {
    machine: Machine,
    events: Queue<Event>,
    below: Queue<Lower>,
    pending: Option<usize>,
    input: VecDeque<u8>,
}

impl Peer {
    fn new(role: Role) -> Peer {
        Peer::new_mode(role, StreamMode::One)
    }

    fn new_mode(role: Role, mode: StreamMode) -> Peer {
        Peer {
            machine: Machine::with_mode(schema(), role, limits(), mode).expect("checked schema"),
            events: Queue::with_capacity(64),
            below: Queue::with_capacity(64),
            pending: None,
            input: VecDeque::new(),
        }
    }

    fn down(&mut self, request: Request) {
        self.machine.down(request, &mut self.events, &mut self.below);
    }

    fn step(&mut self, other: &mut Peer) {
        self.machine.poll(&mut self.events, &mut self.below);
        while let Some(lower) = self.below.pop() {
            match lower {
                Lower::Read(stream::Down::Demand { read: stream::Read::Fill(count), room: 0 }) => {
                    self.pending = Some(usize::try_from(count).expect("bounded count"));
                }
                Lower::Read(stream::Down::Demand { read: stream::Read::Nothing, room: 0 } | stream::Down::Finish)
                | Lower::FinishWrite => {}
                Lower::Read(_) => panic!("unexpected classic lower request"),
                Lower::Write(stream::OutputDown::Room { right, .. }) => {
                    self.machine.up(
                        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
                        &mut self.events,
                        &mut self.below,
                    );
                }
                Lower::Write(stream::OutputDown::Send { bytes, .. }) => {
                    other.input.extend(bytes.iter().copied());
                }
                Lower::Write(_) => panic!("unexpected independent output operation"),
            }
        }
        if let Some(count) = self.pending
            && self.input.len() >= count
        {
            let mut bytes = Vec::with_capacity(count);
            for _ in 0..count {
                bytes.push(self.input.pop_front().expect("enough input"));
            }
            self.pending = None;
            self.machine.up(
                LowerEvent::Read(stream::Up::Bytes(bytes.into_boxed_slice())),
                &mut self.events,
                &mut self.below,
            );
        }
    }
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the test fails on every unexpected event")]
fn opening_round_trip_exposes_peer_terms_and_owner_can_refuse_limits() {
    let mut initiator = Peer::new(Role::Initiator);
    let mut responder = Peer::new(Role::Responder);
    initiator.down(Request::Open { credential: Box::from(*b"key") });
    let mut opening_seen = false;
    let mut ready_initiator = false;
    let mut ready_responder = false;
    for _ in 0_u32..80_u32 {
        initiator.step(&mut responder);
        responder.step(&mut initiator);
        while let Some(event) = responder.events.pop() {
            match event {
                Event::Opening { credential, lowest, highest } => {
                    assert_eq!(&*credential, b"key");
                    assert_eq!((lowest, highest), (1, 2));
                    opening_seen = true;
                    responder.down(Request::Accept { version: 2 });
                }
                Event::Ready { version, terms } => {
                    assert_eq!(version, 2);
                    assert_eq!(terms.len(), 1);
                    ready_responder = true;
                }
                Event::Drained | Event::Closed { .. } => {}
                event => panic!("unexpected responder event: {event:?}"),
            }
        }
        while let Some(event) = initiator.events.pop() {
            match event {
                Event::Ready { version, terms } => {
                    assert_eq!(version, 2);
                    assert_eq!(terms.len(), 0);
                    ready_initiator = true;
                    initiator.down(Request::Refuse { reason: 2, text: Box::from(*b"kind 257") });
                }
                Event::Drained | Event::Closed { .. } => {}
                event => panic!("unexpected initiator event: {event:?}"),
            }
        }
        if ready_initiator && ready_responder && !responder.input.is_empty() {
            break;
        }
    }
    assert!(opening_seen && ready_initiator && ready_responder);
    let mut bytes = Vec::new();
    while let Some(byte) = responder.input.pop_front() {
        bytes.push(byte);
    }
    let header = parse_header(bytes.get(..8).expect("late Refuse header")).expect("valid header");
    assert_eq!(header.kind, 3);
    let control = decode_control(3, bytes.get(8..).expect("late Refuse body"), &limits()).expect("late Refuse");
    match control {
        Control::Refuse { reason, text } => {
            assert_eq!(reason, 2);
            assert_eq!(&*text, b"kind 257");
        }
        _ => panic!("expected Refuse"),
    }
    assert_eq!(initiator.machine.version(), None);
    assert_eq!(responder.machine.version(), Some(2));
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the test fails on an unexpected control message")]
fn opening_refusal_reasons_cover_magic_range_version_and_owner() {
    let cases = [(*b"bad!", 1_u16, 2_u16, 3_u16), (*b"test", 2, 1, 3), (*b"test", 3, 4, 1)];
    for (magic, lowest, highest, expected) in cases {
        let mut initiator = Peer::new(Role::Initiator);
        let mut responder = Peer::new(Role::Responder);
        let frame = control_frame(
            &Control::Open { magic, lowest, highest, features: 0, credential: Box::from(*b"key") },
            &limits(),
        )
        .expect("bounded Open");
        responder.input.extend(frame.bytes().iter().copied());
        for _ in 0_u32..30_u32 {
            responder.step(&mut initiator);
        }
        let mut bytes = Vec::new();
        while let Some(byte) = initiator.input.pop_front() {
            bytes.push(byte);
        }
        let header = parse_header(bytes.get(..8).expect("Refuse header")).expect("valid header");
        assert_eq!(header.kind, 3);
        let control = decode_control(3, bytes.get(8..).expect("Refuse body"), &limits()).expect("Refuse");
        match control {
            Control::Refuse { reason, .. } => assert_eq!(reason, expected),
            _ => panic!("expected Refuse"),
        }
    }

    let mut initiator = Peer::new(Role::Initiator);
    let mut responder = Peer::new(Role::Responder);
    initiator.down(Request::Open { credential: Box::from(*b"key") });
    let mut opening_seen = false;
    for _ in 0_u32..40_u32 {
        initiator.step(&mut responder);
        responder.step(&mut initiator);
        while let Some(event) = responder.events.pop() {
            if let Event::Opening { .. } = event {
                opening_seen = true;
                responder.down(Request::Refuse { reason: 256, text: Box::from(*b"busy") });
            }
        }
        if opening_seen {
            break;
        }
    }
    assert!(opening_seen);
    let mut saw_refusal = false;
    for _ in 0_u32..40_u32 {
        responder.step(&mut initiator);
        initiator.step(&mut responder);
        while let Some(event) = initiator.events.pop() {
            if let Event::Refused { reason, text } = event {
                assert_eq!(reason, 256);
                assert_eq!(&*text, b"busy");
                saw_refusal = true;
            }
        }
        if saw_refusal {
            break;
        }
    }
    assert!(saw_refusal);
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the test fails on unexpected opening events")]
fn a_version_refusal_names_both_ranges_to_the_peer_and_its_owner() {
    let mut peer = Peer::new(Role::Initiator);
    let mut responder = Peer::new(Role::Responder);
    let frame = control_frame(
        &Control::Open { magic: *b"test", lowest: 3, highest: 4, features: 0, credential: Box::from(*b"unread") },
        &limits(),
    )
    .expect("Open");
    responder.input.extend(frame.bytes().iter().copied());
    for _ in 0_u32..30 {
        responder.step(&mut peer);
    }
    let mut sent = Vec::new();
    for byte in peer.input {
        sent.push(byte);
    }
    let Control::Refuse { reason, text } =
        decode_control(3, sent.get(8..).expect("Refuse body"), &limits()).expect("Refuse")
    else {
        panic!("refusal")
    };
    assert_eq!(reason, 1);
    assert_eq!(text.as_ref(), b"speaks 1..2, offered 3..4");
    let mut terminal = 0_u32;
    while let Some(event) = responder.events.pop() {
        match event {
            Event::Opening { .. } => panic!("the credential must not reach the owner"),
            Event::Closed { why, text: sent } => {
                assert_eq!(why, super::Closed::RefusedHere(reason));
                assert_eq!(sent, text);
                terminal += 1;
            }
            Event::Drained => {}
            event => panic!("unexpected opening event {event:?}"),
        }
    }
    assert_eq!(terminal, 1);
}

fn ready_pair() -> (Peer, Peer) {
    ready_pair_mode(StreamMode::One)
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the test fails on unexpected opening events")]
fn ready_pair_mode(mode: StreamMode) -> (Peer, Peer) {
    let mut initiator = Peer::new_mode(Role::Initiator, mode);
    let mut responder = Peer::new_mode(Role::Responder, mode);
    initiator.down(Request::Open { credential: Box::from([]) });
    let mut first_ready = false;
    let mut second_ready = false;
    for _ in 0_u32..80_u32 {
        initiator.step(&mut responder);
        responder.step(&mut initiator);
        while let Some(event) = responder.events.pop() {
            match event {
                Event::Opening { .. } => responder.down(Request::Accept { version: 2 }),
                Event::Ready { version: 2, .. } => second_ready = true,
                Event::Drained => {}
                event => panic!("unexpected responder opening event: {event:?}"),
            }
        }
        while let Some(event) = initiator.events.pop() {
            match event {
                Event::Ready { version: 2, .. } => first_ready = true,
                Event::Drained => {}
                event => panic!("unexpected initiator opening event: {event:?}"),
            }
        }
        if first_ready && second_ready {
            return (initiator, responder);
        }
    }
    panic!("both peers should become ready");
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the test fails on unexpected events")]
fn one_read_skips_unknown_body_then_delivers_chunked_known_body() {
    let (mut initiator, mut responder) = ready_pair();
    let mut unknown = frame_writer(0x01ff, 40).expect("bounded unknown frame");
    unknown.put(&[7_u8; 40]).expect("measured body");
    let unknown = unknown.finish().expect("filled frame");
    let mut known = frame_writer(0x0101, 32).expect("bounded known frame");
    known.put(&[9_u8; 32]).expect("measured body");
    let known = known.finish().expect("filled frame");
    initiator.input.extend(unknown.bytes().iter().copied());
    initiator.input.extend(known.bytes().iter().copied());
    initiator.input.extend(known.bytes().iter().copied());
    initiator.down(Request::Read);
    let mut body_count = 0_u32;
    for _ in 0_u32..40_u32 {
        initiator.step(&mut responder);
        while let Some(event) = initiator.events.pop() {
            match event {
                Event::Body { kind, body } => {
                    assert_eq!(kind, 0x0101);
                    assert_eq!(&*body, &[9_u8; 32]);
                    body_count = body_count.checked_add(1).expect("bounded count");
                }
                Event::Drained => {}
                event => panic!("unexpected read event: {event:?}"),
            }
        }
    }
    assert_eq!(body_count, 1);
    assert_eq!(initiator.input.len(), usize::try_from(known.wire_len()).expect("bounded frame"));
    initiator.down(Request::Read);
    for _ in 0_u32..20_u32 {
        initiator.step(&mut responder);
    }
    match initiator.events.pop() {
        Some(Event::Body { kind: 0x0101, .. }) => {}
        event => panic!("second Read should deliver the second body: {event:?}"),
    }
    let mut answer = Vec::new();
    while let Some(byte) = responder.input.pop_front() {
        answer.push(byte);
    }
    let header = parse_header(answer.get(..8).expect("Unsupported header")).expect("valid header");
    assert_eq!(header.kind, 6);
    match decode_control(header.kind, answer.get(8..).expect("Unsupported body"), &limits()).expect("Unsupported") {
        Control::Unsupported { kind } => assert_eq!(kind, 0x01ff),
        _ => panic!("expected Unsupported"),
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test uses an ordinary Rust vector for peer bytes")]
fn wrong_direction_and_oversized_headers_refuse_before_body() {
    let cases = [(0x0101_u16, 0_u32, true), (0x0101, 65, false), (0x01ff, 129, false)];
    for (kind, body_len, target_responder) in cases {
        let (mut initiator, mut responder) = ready_pair();
        let (target, other) =
            if target_responder { (&mut responder, &mut initiator) } else { (&mut initiator, &mut responder) };
        let mut frame = frame_writer(kind, body_len).expect("bounded test frame");
        let body = alloc::vec![0_u8; usize::try_from(body_len).expect("small body")];
        frame.put(&body).expect("measured frame");
        let frame = frame.finish().expect("complete frame");
        target.input.extend(frame.bytes().get(..8).expect("header").iter().copied());
        target.down(Request::Read);
        for _ in 0_u32..25_u32 {
            target.step(other);
        }
        let mut refused = false;
        while let Some(event) = target.events.pop() {
            if let Event::Closed { why: super::Closed::Framing, .. } = event {
                refused = true;
            }
        }
        assert!(refused, "invalid header {kind} length {body_len} should send framing refusal");
        let mut answer = Vec::new();
        while let Some(byte) = other.input.pop_front() {
            answer.push(byte);
        }
        let header = parse_header(answer.get(..8).expect("Refuse header")).expect("valid header");
        assert_eq!(header.kind, 3);
    }
}

#[test]
fn end_between_frames_answers_read_and_end_inside_header_or_body_closes() {
    let (mut initiator, mut responder) = ready_pair();
    initiator.down(Request::Read);
    initiator.step(&mut responder);
    initiator.machine.up(LowerEvent::Read(stream::Up::End), &mut initiator.events, &mut initiator.below);
    match initiator.events.pop() {
        Some(Event::Ended) => {}
        event => panic!("expected Ended: {event:?}"),
    }

    let (mut initiator, mut responder) = ready_pair();
    initiator.down(Request::Read);
    initiator.input.push_back(1);
    initiator.step(&mut responder);
    initiator.machine.up(LowerEvent::Read(stream::Up::End), &mut initiator.events, &mut initiator.below);
    match initiator.events.pop() {
        Some(Event::Closed { why: super::Closed::Truncated, .. }) => {}
        event => panic!("expected truncated header: {event:?}"),
    }

    let (mut initiator, mut responder) = ready_pair();
    let mut frame = frame_writer(0x0101, 32).expect("measured frame");
    frame.put(&[1_u8; 32]).expect("body fits");
    let frame = frame.finish().expect("complete frame");
    initiator.input.extend(frame.bytes().get(..10).expect("head and partial body").iter().copied());
    initiator.down(Request::Read);
    for _ in 0_u32..4_u32 {
        initiator.step(&mut responder);
    }
    initiator.machine.up(LowerEvent::Read(stream::Up::End), &mut initiator.events, &mut initiator.below);
    match initiator.events.pop() {
        Some(Event::Closed { why: super::Closed::Truncated, .. }) => {}
        event => panic!("expected truncated body: {event:?}"),
    }
}

#[test]
fn send_checks_direction_peer_terms_size_and_room_in_order() {
    let source = schema();
    let mut writer = frame_writer(0x0101, 9).expect("measured frame");
    writer.put(&[5_u8; 9]).expect("body fits");
    let frame = writer.finish().expect("complete frame");
    let mut terms = List::with_capacity(1);
    assert_eq!(
        super::write::admit(&source, Role::Initiator, 2, &terms, Room { bytes: 1024, frames: 4 }, &frame),
        Err(Unsent::WrongDirection)
    );
    assert_eq!(
        super::write::admit(&source, Role::Responder, 2, &terms, Room { bytes: 1024, frames: 4 }, &frame),
        Err(Unsent::PeerDoesNotTake)
    );
    terms.push(Term { kind: 0x0101, largest: 8 }).expect("term fits");
    assert_eq!(
        super::write::admit(&source, Role::Responder, 2, &terms, Room { bytes: 1024, frames: 4 }, &frame),
        Err(Unsent::TooLarge)
    );
    terms.get_mut(0).expect("term").largest = 64;
    assert_eq!(
        super::write::admit(&source, Role::Responder, 2, &terms, Room { bytes: 16, frames: 4 }, &frame),
        Err(Unsent::Full)
    );
    assert_eq!(
        super::write::admit(&source, Role::Responder, 2, &terms, Room { bytes: 1024, frames: 0 }, &frame),
        Err(Unsent::Full)
    );
    assert_eq!(
        super::write::admit(&source, Role::Responder, 2, &terms, Room { bytes: 1024, frames: 4 }, &frame),
        Ok(())
    );
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the protocol story fails on unexpected owner events")]
fn measured_frame_moves_between_two_peers_while_read_and_output_progress() {
    let (mut initiator, mut responder) = ready_pair();
    let mut writer = frame_writer(0x0101, 32).expect("measured frame");
    writer.put(&[12_u8; 32]).expect("body fits");
    responder.down(Request::Send { token: Token::new(42), frame: writer.finish().expect("frame") });
    assert_eq!(responder.machine.room(), Room { bytes: 984, frames: 3 });
    match responder.events.pop() {
        Some(Event::Sent { token }) => assert_eq!(token, Token::new(42)),
        event => panic!("expected queue admission: {event:?}"),
    }
    initiator.down(Request::Read);
    let mut drained = 0_u32;
    let mut received = 0_u32;
    for _ in 0_u32..30_u32 {
        responder.step(&mut initiator);
        initiator.step(&mut responder);
        while let Some(event) = responder.events.pop() {
            match event {
                Event::Drained => drained = drained.checked_add(1).expect("bounded count"),
                event => panic!("unexpected sender event: {event:?}"),
            }
        }
        while let Some(event) = initiator.events.pop() {
            match event {
                Event::Body { kind, body } => {
                    assert_eq!(kind, 0x0101);
                    assert_eq!(&*body, &[12_u8; 32]);
                    received = received.checked_add(1).expect("bounded count");
                }
                event => panic!("unexpected receiver event: {event:?}"),
            }
        }
    }
    assert_eq!(drained, 1);
    assert_eq!(received, 1);
    assert_eq!(responder.machine.room(), Room { bytes: 1024, frames: 4 });
}

#[test]
fn refusal_cancels_a_pending_right_and_finishes_after_its_last_word() {
    let (_, mut responder) = ready_pair();
    let mut writer = frame_writer(0x0101, 4).expect("measured frame");
    writer.put(b"drop").expect("body fits");
    responder.down(Request::Send { token: Token::new(1), frame: writer.finish().expect("frame") });
    match responder.events.pop() {
        Some(Event::Sent { token }) => assert_eq!(token, Token::new(1)),
        event => panic!("expected admission: {event:?}"),
    }
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let old_right = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected output request: {record:?}"),
    };
    responder.down(Request::Refuse { reason: 256, text: Box::from(*b"bye") });
    responder.machine.poll(&mut responder.events, &mut responder.below);
    match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Cancel { right })) => assert_eq!(right, old_right),
        record => panic!("expected named cancellation: {record:?}"),
    }
    responder.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right: old_right, outcome: stream::OutputOutcome::Cancelled }),
        &mut responder.events,
        &mut responder.below,
    );
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let refuse_right = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected Refuse room: {record:?}"),
    };
    assert_ne!(refuse_right, old_right);
    responder.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right: refuse_right, outcome: stream::OutputOutcome::Granted }),
        &mut responder.events,
        &mut responder.below,
    );
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let bytes = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) => bytes,
        record => panic!("expected Refuse send: {record:?}"),
    };
    let header = parse_header(bytes.get(..8).expect("header")).expect("framed refusal");
    assert_eq!(header.kind, 3);
    match responder.below.pop() {
        Some(Lower::Read(stream::Down::Finish)) => {}
        record => panic!("expected stream Finish: {record:?}"),
    }
    match responder.events.pop() {
        Some(Event::Drained) => {}
        event => panic!("expected drain: {event:?}"),
    }
    match responder.events.pop() {
        Some(Event::Closed { why: super::Closed::RefusedHere(256), text }) => assert_eq!(text.as_ref(), b"bye"),
        event => panic!("expected refusal terminal: {event:?}"),
    }
    responder.machine.poll(&mut responder.events, &mut responder.below);
    assert!(responder.events.is_empty());
}

#[test]
fn close_releases_a_grant_that_beat_cancellation() {
    let (_, mut responder) = ready_pair();
    let mut writer = frame_writer(0x0101, 1).expect("measured frame");
    writer.put(b"x").expect("body fits");
    responder.down(Request::Send { token: Token::new(2), frame: writer.finish().expect("frame") });
    drop(responder.events.pop());
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let right = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected room: {record:?}"),
    };
    responder.down(Request::Close);
    match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Cancel { right: cancelled })) => assert_eq!(cancelled, right),
        record => panic!("expected Cancel: {record:?}"),
    }
    responder.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
        &mut responder.events,
        &mut responder.below,
    );
    responder.machine.poll(&mut responder.events, &mut responder.below);
    match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Release { right: released })) => assert_eq!(released, right),
        record => panic!("expected Release: {record:?}"),
    }
    match responder.events.pop() {
        Some(Event::Closed { why: super::Closed::Owner, .. }) => {}
        event => panic!("expected owner terminal: {event:?}"),
    }
    assert!(responder.events.is_empty());
}

#[test]
fn one_stream_finish_waits_for_read_but_two_pipes_finish_independently() {
    let (mut one, _) = ready_pair_mode(StreamMode::One);
    one.down(Request::Read);
    one.machine.poll(&mut one.events, &mut one.below);
    match one.below.pop() {
        Some(Lower::Read(stream::Down::Demand { read: stream::Read::Fill(1), room: 0 })) => {}
        record => panic!("expected first header byte: {record:?}"),
    }
    one.down(Request::Finish);
    one.machine.poll(&mut one.events, &mut one.below);
    assert!(one.below.is_empty());
    one.machine.up(LowerEvent::Read(stream::Up::End), &mut one.events, &mut one.below);
    one.machine.poll(&mut one.events, &mut one.below);
    match one.below.pop() {
        Some(Lower::Read(stream::Down::Demand { read: stream::Read::Nothing, room: 0 })) => {}
        record => panic!("expected crossed-read withdrawal: {record:?}"),
    }
    match one.below.pop() {
        Some(Lower::Read(stream::Down::Finish)) => {}
        record => panic!("expected delayed Finish: {record:?}"),
    }

    let (mut two, mut peer) = ready_pair_mode(StreamMode::Two);
    two.down(Request::Read);
    two.machine.poll(&mut two.events, &mut two.below);
    drop(two.below.pop());
    two.pending = Some(1);
    two.down(Request::Finish);
    two.machine.poll(&mut two.events, &mut two.below);
    match two.below.pop() {
        Some(Lower::FinishWrite) => {}
        record => panic!("expected independent write finish: {record:?}"),
    }
    let mut frame = frame_writer(0x0101, 3).expect("measured frame");
    frame.put(b"end").expect("body fits");
    two.input.extend(frame.finish().expect("frame").bytes().iter().copied());
    for _ in 0_u32..20_u32 {
        two.step(&mut peer);
    }
    match two.events.pop() {
        Some(Event::Body { kind: 0x0101, body }) => assert_eq!(&*body, b"end"),
        event => panic!("expected last body after finish: {event:?}"),
    }
}

#[test]
fn one_stream_finish_follows_the_in_flight_read_answer() {
    let (mut channel, _) = ready_pair_mode(StreamMode::One);
    channel.down(Request::Read);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Read(stream::Down::Demand { read: stream::Read::Fill(1), room: 0 })) => {}
        record => panic!("expected first header byte: {record:?}"),
    }
    channel.down(Request::Finish);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.below.is_empty());
    channel.machine.up(LowerEvent::Read(stream::Up::Bytes(Box::from([0_u8]))), &mut channel.events, &mut channel.below);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Read(stream::Down::Demand { read: stream::Read::Fill(7), room: 0 })) => {}
        record => panic!("expected rest of header: {record:?}"),
    }
    channel.machine.up(
        LowerEvent::Read(stream::Up::Bytes(Box::from([5_u8, 0, 0, 0, 0, 0, 0]))),
        &mut channel.events,
        &mut channel.below,
    );
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.events.pop() {
        Some(Event::Ping) => {}
        event => panic!("expected in-flight read answer: {event:?}"),
    }
    match channel.below.pop() {
        Some(Lower::Read(stream::Down::Finish)) => {}
        record => panic!("expected Finish after answer: {record:?}"),
    }
}

#[test]
fn two_pipes_keep_reading_after_write_failure_and_drain_after_read_failure() {
    let (mut initiator, mut responder) = ready_pair_mode(StreamMode::Two);
    let mut writer = frame_writer(0x0101, 1).expect("measured frame");
    writer.put(b"x").expect("body fits");
    responder.down(Request::Send { token: Token::new(3), frame: writer.finish().expect("frame") });
    drop(responder.events.pop());
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let right = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected write room: {record:?}"),
    };
    responder.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled {
            right,
            outcome: stream::OutputOutcome::Failed(stream::Fault::Other),
        }),
        &mut responder.events,
        &mut responder.below,
    );
    match responder.events.pop() {
        Some(Event::OutputFailed) => {}
        event => panic!("expected output failure: {event:?}"),
    }
    responder.down(Request::Read);
    let ping = control_frame(&Control::Ping, &limits()).expect("Ping frame");
    responder.input.extend(ping.bytes().iter().copied());
    for _ in 0_u32..10_u32 {
        responder.step(&mut initiator);
    }
    match responder.events.pop() {
        Some(Event::Ping) => {}
        event => panic!("reading should continue after output failure: {event:?}"),
    }

    let (_, mut responder) = ready_pair_mode(StreamMode::Two);
    let mut writer = frame_writer(0x0101, 1).expect("measured frame");
    writer.put(b"z").expect("body fits");
    responder.down(Request::Send { token: Token::new(4), frame: writer.finish().expect("frame") });
    drop(responder.events.pop());
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let right = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected write room: {record:?}"),
    };
    responder.machine.up(
        LowerEvent::Read(stream::Up::Failed(stream::Fault::Other)),
        &mut responder.events,
        &mut responder.below,
    );
    assert!(responder.events.is_empty(), "queued output still needs its grant");
    responder.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
        &mut responder.events,
        &mut responder.below,
    );
    responder.machine.poll(&mut responder.events, &mut responder.below);
    match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) => assert_eq!(bytes.get(8..), Some(&b"z"[..])),
        record => panic!("queued last word should drain: {record:?}"),
    }
    match responder.events.pop() {
        Some(Event::Drained) => {}
        event => panic!("expected drain: {event:?}"),
    }
    match responder.events.pop() {
        Some(Event::Closed { why: super::Closed::Stream, .. }) => {}
        event => panic!("expected stream terminal: {event:?}"),
    }
}

#[test]
fn one_stream_failure_cancels_output_and_closes_once() {
    let (_, mut responder) = ready_pair_mode(StreamMode::One);
    let mut writer = frame_writer(0x0101, 1).expect("measured frame");
    writer.put(b"x").expect("body fits");
    responder.down(Request::Send { token: Token::new(5), frame: writer.finish().expect("frame") });
    drop(responder.events.pop());
    responder.machine.poll(&mut responder.events, &mut responder.below);
    let right = match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected write room: {record:?}"),
    };
    responder.machine.up(
        LowerEvent::Read(stream::Up::Failed(stream::Fault::Reset)),
        &mut responder.events,
        &mut responder.below,
    );
    responder.machine.poll(&mut responder.events, &mut responder.below);
    match responder.below.pop() {
        Some(Lower::Write(stream::OutputDown::Cancel { right: cancelled })) => assert_eq!(cancelled, right),
        record => panic!("expected cancellation: {record:?}"),
    }
    responder.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Cancelled }),
        &mut responder.events,
        &mut responder.below,
    );
    match responder.events.pop() {
        Some(Event::Closed { why: super::Closed::Stream, .. }) => {}
        event => panic!("expected one stream terminal: {event:?}"),
    }
    responder.machine.poll(&mut responder.events, &mut responder.below);
    assert!(responder.events.is_empty());
}

#[test]
fn write_failure_during_refusal_reports_stream_terminal() {
    let (_, mut responder) = ready_pair_mode(StreamMode::Two);
    responder.down(Request::Refuse { reason: 256, text: Box::from(*b"bye") });
    responder.machine.up(LowerEvent::WriteFailed(stream::Fault::Reset), &mut responder.events, &mut responder.below);
    match responder.events.pop() {
        Some(Event::OutputFailed) => {}
        event => panic!("expected output failure: {event:?}"),
    }
    match responder.events.pop() {
        Some(Event::Closed { why: super::Closed::Stream, .. }) => {}
        event => panic!("expected stream terminal: {event:?}"),
    }
    responder.machine.poll(&mut responder.events, &mut responder.below);
    assert!(responder.below.is_empty());
}

#[test]
fn role_aware_schema_accepts_large_received_kinds_and_refuses_large_sent_kinds() {
    for role in [Role::Initiator, Role::Responder] {
        let mut source = schema();
        let bounds = Limits { output_bytes: 128, ..limits() };
        for version_index in 0..source.versions.len() {
            let version = source.versions.get_mut(version_index).unwrap();
            for kind_index in 0..version.kinds.len() {
                let kind = version.kinds.get_mut(kind_index).unwrap();
                let sent = match role {
                    Role::Initiator => kind.direction == Direction::FromInitiator,
                    Role::Responder => kind.direction == Direction::FromResponder,
                };
                kind.largest = if sent { 120 } else { 4096 };
            }
        }
        assert_eq!(source.check(role, &bounds), Ok(()));
        Machine::new(source.clone(), role, bounds).expect("large received body is independent of output");
        for mode in [StreamMode::One, StreamMode::Two] {
            Machine::with_mode(source.clone(), role, bounds, mode).expect("both transports admit the same local role");
        }
        for version_index in 0..source.versions.len() {
            let version = source.versions.get_mut(version_index).unwrap();
            for kind_index in 0..version.kinds.len() {
                let kind = version.kinds.get_mut(kind_index).unwrap();
                let sent = match role {
                    Role::Initiator => kind.direction == Direction::FromInitiator,
                    Role::Responder => kind.direction == Direction::FromResponder,
                };
                if sent {
                    kind.largest = 121;
                }
            }
        }
        assert_eq!(source.check(role, &bounds), Err(SchemaError::Kind));
        assert_eq!(Machine::new(source.clone(), role, bounds).err(), Some(SchemaError::Kind));
        assert_eq!(Machine::with_mode(source, role, bounds, StreamMode::Two).err(), Some(SchemaError::Kind));
    }
}

#[test]
fn valid_accepts_test_limits() {
    for role in [Role::Initiator, Role::Responder] {
        assert!(super::valid(&schema(), role, &limits()));
    }
}

#[test]
fn valid_refuses_an_invalid_schema() {
    let mut source = schema();
    source.versions.get_mut(1).unwrap().version = 3;
    assert!(!super::valid(&source, Role::Initiator, &limits()));
    assert_eq!(Machine::new(source, Role::Initiator, limits()).err(), Some(SchemaError::Versions));
}

#[test]
fn valid_refuses_an_output_queue_past_u32() {
    let bounds = Limits { output_frames: u32::MAX, ..limits() };
    assert!(!super::valid(&schema(), Role::Initiator, &bounds));
    assert_eq!(Machine::new(schema(), Role::Initiator, bounds).err(), Some(SchemaError::InvalidLimit));
}

#[test]
fn unrepresentable_control_frame_limits_fail_validation() {
    skein_rows::check(
        [
            ("valid_refuses_a_terms_body_past_u32", Limits { kinds: u32::MAX, ..limits() }),
            ("valid_refuses_an_open_body_past_u32", Limits { credential: u32::MAX, ..limits() }),
            ("valid_refuses_a_control_frame_past_u32: credential", Limits { credential: u32::MAX - 14, ..limits() }),
            ("valid_refuses_a_control_frame_past_u32: kinds", Limits { kinds: (u32::MAX - 4) / 6, ..limits() }),
        ],
        |bounds| assert!(!super::valid(&schema(), Role::Initiator, &bounds)),
    );
}

#[test]
#[expect(clippy::disallowed_macros, reason = "Debug regression tests format made-up credential canaries")]
fn debug_of_an_opening_shows_the_credentials_length_never_its_bytes() {
    const CANARY: &[u8; 13] = b"canary-c42e8b";
    let canary_bytes = alloc::format!("{CANARY:?}");
    let request = Request::Open { credential: Box::from(*CANARY) };
    let event = Event::Opening { credential: Box::from(*CANARY), lowest: 1, highest: 2 };
    let control = Control::Open { magic: *b"test", lowest: 1, highest: 2, features: 0, credential: Box::from(*CANARY) };
    for (debug, pretty, expected) in [
        (alloc::format!("{request:?}"), alloc::format!("{request:#?}"), "Open { credential_len: 13 }"),
        (
            alloc::format!("{event:?}"),
            alloc::format!("{event:#?}"),
            "Opening { credential_len: 13, lowest: 1, highest: 2 }",
        ),
        (
            alloc::format!("{control:?}"),
            alloc::format!("{control:#?}"),
            "Open { magic: [116, 101, 115, 116], lowest: 1, highest: 2, features: 0, credential_len: 13 }",
        ),
    ] {
        assert!(!debug.contains("canary-c42e8b"));
        assert!(!debug.contains(&canary_bytes));
        assert_eq!(debug, expected);
        assert!(!pretty.contains("canary-c42e8b"));
        assert!(!pretty.replace([' ', '\n'], "").contains(&canary_bytes.replace(' ', "")));
        assert!(pretty.contains("credential_len: 13"));
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "Debug regression tests format made-up body canaries")]
fn debug_of_a_frame_and_a_body_shows_lengths_never_bytes() {
    const CANARY: &[u8; 16] = b"canary-body-7c42";
    let canary_bytes = alloc::format!("{CANARY:?}");
    let canary_numbers = canary_bytes.trim_matches(['[', ']']);
    let mut writer = frame_writer(256, 16).expect("measured frame");
    writer.put(CANARY).expect("body fits");
    let frame = writer.finish().expect("frame");
    let frame_debug = alloc::format!("{frame:?}");
    let frame_pretty = alloc::format!("{frame:#?}");
    let send = Request::Send { token: Token::new(7), frame };
    let body = Event::Body { kind: 256, body: Box::from(*CANARY) };
    for (debug, pretty, expected) in [
        (frame_debug, frame_pretty, "Frame { kind: 256, body_len: 16 }"),
        (
            alloc::format!("{send:?}"),
            alloc::format!("{send:#?}"),
            "Send { token: Token(7), frame: Frame { kind: 256, body_len: 16 } }",
        ),
        (alloc::format!("{body:?}"), alloc::format!("{body:#?}"), "Body { kind: 256, body_len: 16 }"),
    ] {
        assert!(!debug.contains("canary-body-7c42"));
        assert!(!debug.contains(canary_numbers), "body bytes reached Debug: {debug}");
        assert_eq!(debug, expected);
        assert!(!pretty.contains("canary-body-7c42"));
        assert!(!pretty.replace([' ', '\n'], "").contains(&canary_numbers.replace(' ', "")));
        assert!(pretty.contains("body_len: 16"));
    }

    let mut writer = frame_writer(256, 1).expect("measured frame");
    writer.put(b"x").expect("body fits");
    let frame = writer.finish().expect("frame");
    assert_eq!(alloc::format!("{frame:?}"), "Frame { kind: 256, body_len: 1 }");
    let send = Request::Send { token: Token::new(7), frame };
    assert_eq!(alloc::format!("{send:?}"), "Send { token: Token(7), frame: Frame { kind: 256, body_len: 1 } }");
    let body = Event::Body { kind: 256, body: Box::from(*b"x") };
    assert_eq!(alloc::format!("{body:?}"), "Body { kind: 256, body_len: 1 }");

    let (mut initiator, mut responder) = ready_pair();
    let mut writer = frame_writer(257, 17).expect("measured frame");
    writer.put(CANARY).expect("first chunk fits");
    writer.put(b"x").expect("last byte fits");
    let frame = writer.finish().expect("frame");
    initiator.input.extend(frame.bytes().get(..24).expect("header and first body chunk").iter().copied());
    initiator.down(Request::Read);
    for _ in 0_u32..3_u32 {
        initiator.step(&mut responder);
    }
    let debug = alloc::format!("{:?}", initiator.machine);
    let pretty = alloc::format!("{:#?}", initiator.machine);
    assert!(!debug.contains("canary-body-7c42"));
    assert!(!debug.contains(canary_numbers), "partial body bytes reached Debug: {debug}");
    assert!(debug.contains("read: Body { header: Header { kind: 257, body_len: 17 }, body_len: 17, filled: 16 }"));
    assert!(!pretty.contains("canary-body-7c42"));
    let compact = pretty.replace([' ', '\n'], "");
    assert!(!compact.contains(&canary_numbers.replace(' ', "")));
    assert!(compact.contains("read:Body{header:Header{kind:257,body_len:17,},body_len:17,filled:16,}"));
    assert!(initiator.events.is_empty(), "partial body has not been delivered");
}

#[test]
#[expect(clippy::disallowed_macros, reason = "Debug regression tests preserve derived record formatting")]
fn debug_of_records_without_a_credential_is_the_derived_form() {
    let request = Request::Refuse { reason: 1, text: Box::from(*b"busy") };
    let mut terms = List::with_capacity(1);
    terms.push(Term { kind: 256, largest: 32 }).expect("one term fits");
    let event = Event::Ready { version: 2, terms };
    let control = Control::Accept { version: 2, features: 0 };
    for (debug, expected) in [
        (alloc::format!("{request:?}"), "Refuse { reason: 1, text: [98, 117, 115, 121] }"),
        (
            alloc::format!("{event:?}"),
            "Ready { version: 2, terms: List { items: [Term { kind: 256, largest: 32 }], capacity: 1 } }",
        ),
        (alloc::format!("{control:?}"), "Accept { version: 2, features: 0 }"),
    ] {
        assert_eq!(debug, expected);
    }
}

#[test]
fn oversized_open_credential_silently_closes_and_owner_close_is_ignored() {
    let mut channel = Peer::new(Role::Initiator);
    channel.down(Request::Open { credential: skein_lib::bytes::zeroed(65) });
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty(), "oversized Open has no owner answer");
    assert!(channel.below.is_empty(), "oversized Open sends nothing");
    channel.down(Request::Close);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty(), "Close after oversized Open is ignored");
    assert!(channel.below.is_empty());
}

#[test]
fn two_pipes_framing_error_after_finish_waits_for_owner_close() {
    let (mut channel, _) = ready_pair_mode(StreamMode::Two);
    channel.down(Request::Finish);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::FinishWrite) => {}
        record => panic!("expected write Finish: {record:?}"),
    }
    channel.down(Request::Read);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    drop(channel.below.pop());
    channel.machine.up(LowerEvent::Read(stream::Up::Bytes(Box::from([0_u8]))), &mut channel.events, &mut channel.below);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    drop(channel.below.pop());
    channel.machine.up(
        LowerEvent::Read(stream::Up::Bytes(Box::from([0_u8; 7]))),
        &mut channel.events,
        &mut channel.below,
    );
    channel.machine.poll(&mut channel.events, &mut channel.below);
    let right = match channel.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected refusal room: {record:?}"),
    };
    channel.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
        &mut channel.events,
        &mut channel.below,
    );
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) => {
            assert_eq!(parse_header(bytes.get(..8).expect("header")).expect("Refuse").kind, 3);
        }
        record => panic!("expected refusal: {record:?}"),
    }
    match channel.events.pop() {
        Some(Event::Drained) => {}
        event => panic!("expected refusal drain: {event:?}"),
    }
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty(), "framing refusal after Finish has no terminal");
    assert!(channel.below.is_empty(), "Finish is not repeated");
    channel.down(Request::Close);
    match channel.events.pop() {
        Some(Event::Closed { why: super::Closed::Owner, .. }) => {}
        event => panic!("expected owner terminal: {event:?}"),
    }
    assert!(channel.events.is_empty());
}

fn queued_output_right(channel: &mut Peer) -> Token {
    let mut writer = frame_writer(0x0101, 1).expect("measured frame");
    writer.put(b"x").expect("body fits");
    channel.down(Request::Send { token: Token::new(9), frame: writer.finish().expect("frame") });
    match channel.events.pop() {
        Some(Event::Sent { token }) => assert_eq!(token, Token::new(9)),
        event => panic!("expected admission: {event:?}"),
    }
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
        record => panic!("expected room: {record:?}"),
    }
}

#[test]
fn one_stream_output_failure_then_grant_has_no_terminal() {
    let (_, mut channel) = ready_pair();
    let right = queued_output_right(&mut channel);
    channel.machine.up(LowerEvent::WriteFailed(stream::Fault::Reset), &mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty(), "failure still holds the output right");
    channel.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
        &mut channel.events,
        &mut channel.below,
    );
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty(), "grant after failure leaves no terminal");
    assert!(channel.below.is_empty(), "failed output neither sends nor releases the grant");
}

#[test]
fn end_with_demand_out_still_withdraws_the_demand() {
    let (mut channel, _) = ready_pair();
    channel.down(Request::Read);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Read(stream::Down::Demand { read: stream::Read::Fill(1), room: 0 })) => {}
        record => panic!("expected header demand: {record:?}"),
    }
    channel.machine.up(LowerEvent::Read(stream::Up::End), &mut channel.events, &mut channel.below);
    match channel.events.pop() {
        Some(Event::Ended) => {}
        event => panic!("expected read answer: {event:?}"),
    }
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Read(stream::Down::Demand { read: stream::Read::Nothing, room: 0 })) => {}
        record => panic!("expected withdrawal after End: {record:?}"),
    }
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.below.is_empty(), "withdrawal goes down once");
    assert!(channel.events.is_empty());
}

#[test]
fn two_pipes_truncation_drains_ignores_close_and_refusal_failure_reports_in_order() {
    let (_, mut channel) = ready_pair_mode(StreamMode::Two);
    let right = queued_output_right(&mut channel);
    channel.down(Request::Read);
    channel.machine.poll(&mut channel.events, &mut channel.below);
    drop(channel.below.pop());
    channel.machine.up(LowerEvent::Read(stream::Up::Bytes(Box::from([1_u8]))), &mut channel.events, &mut channel.below);
    channel.machine.up(LowerEvent::Read(stream::Up::End), &mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty(), "truncation waits for queued output");
    channel.down(Request::Close);
    assert!(channel.events.is_empty(), "Close while draining is ignored");
    channel.machine.up(
        LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
        &mut channel.events,
        &mut channel.below,
    );
    channel.machine.poll(&mut channel.events, &mut channel.below);
    match channel.below.pop() {
        Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) => assert_eq!(bytes.get(8..), Some(&b"x"[..])),
        record => panic!("expected queued output to drain: {record:?}"),
    }
    match channel.events.pop() {
        Some(Event::Drained) => {}
        event => panic!("expected drain: {event:?}"),
    }
    match channel.events.pop() {
        Some(Event::Closed { why: super::Closed::Truncated, .. }) => {}
        event => panic!("expected truncation terminal: {event:?}"),
    }
    assert!(channel.events.is_empty());
    assert!(channel.below.is_empty());

    let (_, mut channel) = ready_pair_mode(StreamMode::Two);
    channel.down(Request::Refuse { reason: 256, text: Box::from([]) });
    channel.machine.up(LowerEvent::WriteFailed(stream::Fault::Reset), &mut channel.events, &mut channel.below);
    match channel.events.pop() {
        Some(Event::OutputFailed) => {}
        event => panic!("expected output failure before terminal: {event:?}"),
    }
    match channel.events.pop() {
        Some(Event::Closed { why: super::Closed::Stream, .. }) => {}
        event => panic!("expected stream terminal: {event:?}"),
    }
    channel.machine.poll(&mut channel.events, &mut channel.below);
    assert!(channel.events.is_empty());
    assert!(channel.below.is_empty());
}

#[test]
fn two_pipes_read_failure_during_opening_drains_terms_before_terminal() {
    let mut initiator = Peer::new_mode(Role::Initiator, StreamMode::Two);
    let mut responder = Peer::new_mode(Role::Responder, StreamMode::Two);
    initiator.down(Request::Open { credential: Box::from([]) });
    for _ in 0_u32..20_u32 {
        initiator.step(&mut responder);
        responder.step(&mut initiator);
    }
    match responder.events.pop() {
        Some(Event::Opening { .. }) => {}
        event => panic!("expected opening: {event:?}"),
    }
    responder.down(Request::Accept { version: 2 });
    responder.machine.up(
        LowerEvent::Read(stream::Up::Failed(stream::Fault::Reset)),
        &mut responder.events,
        &mut responder.below,
    );
    assert!(responder.events.is_empty(), "opening output still needs to drain");
    for kind in [2, 4] {
        responder.machine.poll(&mut responder.events, &mut responder.below);
        let right = match responder.below.pop() {
            Some(Lower::Write(stream::OutputDown::Room { right, .. })) => right,
            record => panic!("expected opening output room: {record:?}"),
        };
        responder.machine.up(
            LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
            &mut responder.events,
            &mut responder.below,
        );
        responder.machine.poll(&mut responder.events, &mut responder.below);
        match responder.below.pop() {
            Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) => {
                assert_eq!(parse_header(bytes.get(..8).expect("header")).expect("opening frame").kind, kind);
            }
            record => panic!("expected opening output: {record:?}"),
        }
        match responder.events.pop() {
            Some(Event::Drained) => {}
            event => panic!("expected opening drain: {event:?}"),
        }
    }
    match responder.events.pop() {
        Some(Event::Closed { why: super::Closed::Stream, .. }) => {}
        event => panic!("expected terminal after Terms: {event:?}"),
    }
    responder.machine.poll(&mut responder.events, &mut responder.below);
    assert!(responder.events.is_empty());
    assert!(responder.below.is_empty());
}
