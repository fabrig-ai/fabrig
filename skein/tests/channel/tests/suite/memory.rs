//! Heap ceilings while a largest input body and a full output byte queue coexist.
use skein_channel::{
    Control, Direction, Event, Kind, Limits, Lower, LowerEvent, MAX_DOWN, MAX_UP, Machine, Request, Role, Schema, Term,
    Version, control_frame, frame_writer,
};
use skein_heap::Meter;
use skein_lib::{List, Queue, Token, stream};

fn schema() -> Schema {
    let mut kinds = List::with_capacity(2);
    kinds.push(Kind { kind: 0x0100, direction: Direction::FromInitiator, largest: 120 }).expect("kind");
    kinds.push(Kind { kind: 0x0101, direction: Direction::FromResponder, largest: 120 }).expect("kind");
    let mut versions = List::with_capacity(1);
    versions.push(Version { version: 1, kinds }).expect("version");
    Schema { magic: *b"heap", versions }
}

fn limits() -> Limits {
    Limits { chunk: 8, credential: 16, skip: 32, output_bytes: 128, output_frames: 2, kinds: 2 }
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the memory world rejects unexpected boundary records")]
#[expect(clippy::too_many_arguments, reason = "the small memory world keeps the machine's three queues explicit")]
fn feed(
    machine: &mut Machine,
    above: &mut Queue<Event>,
    below: &mut Queue<Lower>,
    pending: &mut Option<usize>,
    bytes: &[u8],
    drain_output: bool,
    ready: &mut bool,
    body_seen: &mut bool,
    expected: (u16, usize),
) {
    let mut offset = 0;
    for _ in 0..200 {
        machine.poll(above, below);
        while let Some(lower) = below.pop() {
            match lower {
                Lower::Read(stream::Down::Demand { read: stream::Read::Fill(count), room: 0 }) => {
                    *pending = Some(usize::try_from(count).expect("bounded demand"));
                }
                Lower::Write(stream::OutputDown::Room { right, .. }) => {
                    if drain_output {
                        machine.up(
                            LowerEvent::Write(stream::OutputUp::Settled {
                                right,
                                outcome: stream::OutputOutcome::Granted,
                            }),
                            above,
                            below,
                        );
                    }
                }
                Lower::Write(stream::OutputDown::Send { .. }) => {}
                lower => panic!("unexpected lower record: {lower:?}"),
            }
        }
        if let Some(count) = *pending
            && bytes.len().saturating_sub(offset) >= count
        {
            let end = offset.checked_add(count).expect("input length");
            machine.up(LowerEvent::Read(stream::Up::Bytes(Box::from(&bytes[offset..end]))), above, below);
            *pending = None;
            offset = end;
        }
        while let Some(event) = above.pop() {
            match event {
                Event::Opening { .. } => machine.down(Request::Accept { version: 1 }, above, below),
                Event::Ready { .. } => *ready = true,
                Event::Body { kind, body } => {
                    assert_eq!(kind, expected.0);
                    assert_eq!(body.len(), expected.1);
                    *body_seen = true;
                }
                Event::Drained | Event::Sent { .. } => {}
                event => panic!("unexpected owner event: {event:?}"),
            }
        }
        if offset == bytes.len()
            && (!drain_output || machine.waiting().write == skein_channel::WriteWait::Nothing)
            && (pending.is_some() || *ready || *body_seen)
        {
            return;
        }
    }
    panic!("frame did not settle");
}

// Bounds the channel and boundary queues with a 120-byte input body and full output bytes.
// Measured peak: 664 bytes; a quarter of headroom, rounded up to two significant digits.
const FULL_FRAME_HEAP_CEILING: u64 = 830;

#[test]
fn largest_input_with_full_output_stays_under_heap_ceiling() {
    let source = schema();
    let limits = limits();
    let open = control_frame(
        &Control::Open { magic: *b"heap", lowest: 1, highest: 1, features: 0, credential: Box::from([]) },
        &limits,
    )
    .expect("open");
    let mut entries = List::with_capacity(1);
    entries.push(Term { kind: 0x0101, largest: 120 }).expect("term");
    let terms = control_frame(&Control::Terms { entries }, &limits).expect("terms");
    let mut input = frame_writer(0x0100, 120).expect("input");
    input.put(&[7; 120]).expect("body");
    let input = input.finish().expect("frame");

    let meter = Meter::new();
    meter.start();
    let mut machine = Machine::new(source.clone(), Role::Responder, limits).expect("machine");
    let mut above = Queue::<Event>::with_capacity(MAX_UP);
    let mut below = Queue::<Lower>::with_capacity(MAX_DOWN);
    let mut pending = None;
    let mut ready = false;
    let mut body_seen = false;
    feed(
        &mut machine,
        &mut above,
        &mut below,
        &mut pending,
        open.bytes(),
        true,
        &mut ready,
        &mut body_seen,
        (0x0100, 120),
    );
    feed(
        &mut machine,
        &mut above,
        &mut below,
        &mut pending,
        terms.bytes(),
        true,
        &mut ready,
        &mut body_seen,
        (0x0100, 120),
    );
    assert!(ready);
    let mut output = frame_writer(0x0101, 120).expect("output");
    output.put(&[9; 120]).expect("body");
    machine.down(
        Request::Send { token: Token::new(1), frame: output.finish().expect("frame") },
        &mut above,
        &mut below,
    );
    machine.down(Request::Read, &mut above, &mut below);
    feed(
        &mut machine,
        &mut above,
        &mut below,
        &mut pending,
        input.bytes(),
        false,
        &mut ready,
        &mut body_seen,
        (0x0100, 120),
    );
    assert!(body_seen);
    assert_eq!(machine.room().bytes, 0, "the queued frame fills the output byte capacity");
    let measured = meter.end();
    assert!(measured.peak() <= FULL_FRAME_HEAP_CEILING, "peak {} exceeds ceiling", measured.peak());
    drop(machine);
    drop(above);
    drop(below);
    assert_eq!(meter.held(), 0);
}

// Bounds either role and its boundary queues with a 4096-byte input and full 128-byte output.
// Each role peaked at 4696 bytes; a quarter of headroom, rounded up to two significant digits.
const LARGE_RECEIVE_HEAP_CEILING: u64 = 5900;

#[test]
fn large_receive_with_full_output_stays_under_heap_ceiling() {
    for role in [Role::Initiator, Role::Responder] {
        let mut source = schema();
        let limits = Limits { chunk: 64, ..limits() };
        let (sent, received): (u16, u16) = match role {
            Role::Initiator => (0x0100, 0x0101),
            Role::Responder => (0x0101, 0x0100),
        };
        source.versions.get_mut(0).unwrap().kinds.get_mut(u32::from(received - 0x0100)).unwrap().largest = 4096;
        let opening = match role {
            Role::Initiator => Control::Accept { version: 1, features: 0 },
            Role::Responder => {
                Control::Open { magic: *b"heap", lowest: 1, highest: 1, features: 0, credential: Box::new([]) }
            }
        };
        let opening = control_frame(&opening, &limits).unwrap();
        let mut entries = List::with_capacity(1);
        entries.push(Term { kind: sent, largest: 120 }).unwrap();
        let terms = control_frame(&Control::Terms { entries }, &limits).unwrap();
        let mut input = frame_writer(received, 4096).unwrap();
        input.put(&[7; 4096]).unwrap();
        let input = input.finish().unwrap();
        let meter = Meter::new();
        meter.start();
        let mut machine = Machine::new(source.clone(), role, limits).unwrap();
        let mut above = Queue::<Event>::with_capacity(MAX_UP);
        let mut below = Queue::<Lower>::with_capacity(MAX_DOWN);
        let mut pending = None;
        let mut ready = false;
        let mut body_seen = false;
        if role == Role::Initiator {
            machine.down(Request::Open { credential: Box::new([]) }, &mut above, &mut below);
        }
        feed(
            &mut machine,
            &mut above,
            &mut below,
            &mut pending,
            opening.bytes(),
            true,
            &mut ready,
            &mut body_seen,
            (received, 4096),
        );
        feed(
            &mut machine,
            &mut above,
            &mut below,
            &mut pending,
            terms.bytes(),
            true,
            &mut ready,
            &mut body_seen,
            (received, 4096),
        );
        assert!(ready);
        let mut output = frame_writer(sent, 120).unwrap();
        output.put(&[9; 120]).unwrap();
        machine.down(Request::Send { token: Token::new(1), frame: output.finish().unwrap() }, &mut above, &mut below);
        machine.down(Request::Read, &mut above, &mut below);
        feed(
            &mut machine,
            &mut above,
            &mut below,
            &mut pending,
            input.bytes(),
            false,
            &mut ready,
            &mut body_seen,
            (received, 4096),
        );
        assert!(body_seen);
        assert_eq!(machine.room().bytes, 0, "the queued frame fills the output byte capacity");
        let measured = meter.end();
        assert!(measured.peak() <= LARGE_RECEIVE_HEAP_CEILING, "{role:?} peak {} exceeds ceiling", measured.peak());
        drop(machine);
        drop(above);
        drop(below);
        assert_eq!(meter.held(), 0);
    }
}
