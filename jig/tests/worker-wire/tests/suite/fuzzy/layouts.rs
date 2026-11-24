//! Drawn frames cross the worker's layout, or are refused without alteration.
use jig_protocol_worker_wire as wire;
use skein_codec::layout::Mode;
use skein_lib::{Buffer, List, Rng};
use wire::layouts::{Cause, Plans};

#[path = "../../../../../crates/jig-protocol-worker-wire/src/tests/layout_samples.rs"]
mod samples;
use samples::{DOWN, ROOM, Shim, UP, frame, older, parse, put, richer, stated};

#[test]
fn fuzzy_every_frame_round_trips_through_the_worker_layout_or_is_refused() {
    let engine = parse(wire::SCHEMA);
    let worker = parse(&richer(&older(wire::SCHEMA)));
    let mut plans = Plans::new(&[wire::SCHEMA], &List::uncapped()).expect("engine text");
    let mut peer = plans.judge(&[stated(&worker)]).expect("different worker layout");
    assert!(!peer.same());
    let mut sent = Shim::new(Mode::Read, &worker, &engine, "FromEngine");
    let mut received = Shim::new(Mode::Keep, &engine, &worker, "FromWorker");
    let mut body = Buffer::with_capacity(ROOM);
    let mut tagged = Buffer::with_capacity(ROOM);
    let mut returned = Buffer::with_capacity(ROOM);
    let mut rng = Rng::new(0x2280_5eed);

    for rule in wire::KINDS {
        let mut accepted = 0;
        let mut refused = 0;
        for _ in 0..48 {
            let original = frame(&engine, *rule, &mut rng, false);
            let (_, old) = original.split_first().expect("tagged sample");
            if rule.kind < DOWN {
                received.convert(&original, &mut tagged).expect("shim keeps old worker fields");
                let (&tag, bytes) = tagged.as_slice().split_first().expect("tagged worker reply");
                let kind = UP.checked_add(u16::from(tag)).expect("worker kind");
                let heap = skein_heap::Span::start();
                assert_eq!(peer.receive(kind, bytes, &mut body), Ok(rule.body));
                assert_eq!(heap.made(), 0, "receiving with a fitted workspace allocates nothing");
                assert_eq!(body.as_slice(), old, "{:?}", rule.body);
                accepted += 1;
            } else {
                let heap = skein_heap::Span::start();
                let result = peer.send(rule.body, old, &mut body);
                let made = heap.made();
                match result {
                    Ok(kind) => {
                        assert_eq!(made, 0, "sending with a fitted workspace allocates nothing");
                        let tag = u8::try_from(kind.checked_sub(DOWN).expect("engine kind")).expect("engine tag");
                        tagged.clear();
                        put(&mut tagged, &[tag]);
                        put(&mut tagged, body.as_slice());
                        sent.convert(tagged.as_slice(), &mut returned).expect("shim reads a supported kind");
                        assert_eq!(returned.as_slice(), &*original, "{:?}", rule.body);
                        accepted += 1;
                    }
                    Err(failure) => {
                        let expect = if rule.body == wire::Kind::PartBusy {
                            Cause::Variant
                        } else {
                            assert_eq!(rule.body, wire::Kind::Assignment, "only assignments can lose a field");
                            assert_eq!(failure.part.as_ref(), "parts");
                            Cause::Field
                        };
                        assert_eq!(failure.cause, expect);
                        assert!(body.is_empty());
                        refused += 1;
                    }
                }
            }
        }
        if rule.body == wire::Kind::PartBusy {
            assert_eq!((accepted, refused), (0, 48));
        } else {
            assert!(accepted > 0, "round trips for {:?}", rule.body);
            if rule.body == wire::Kind::Assignment {
                assert!(refused > 0, "a nonempty assignment's parts cannot be dropped");
            }
        }
    }
}
