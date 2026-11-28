//! Drawn frames cross the worker's layout, or are refused without alteration.
use core::convert::Infallible;

use jig_protocol_worker_wire as wire;
use skein_codec::{
    layout::Mode,
    schema::{Shape, VariantIx},
};
use skein_lib::{Buffer, List, Rng};
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};
use wire::layouts::{Cause, Plans};

#[path = "../../../../../crates/jig-protocol-worker-wire/src/tests/layout_samples.rs"]
mod samples;
use samples::{DOWN, ROOM, Shim, UP, draw, frame, older, parse, put, richer, stated};

#[test]
fn fuzzy_every_frame_round_trips_through_the_worker_layout_or_is_refused() -> Result<(), Case<Infallible>> {
    cases(this_test!("fuzzy_every_frame_round_trips_through_the_worker_layout_or_is_refused"), [0x2280_5eed], |seed| {
        check_round_trips(seed);
        Ok(())
    })
}

fn check_round_trips(seed: u64) {
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
    let mut rng = Rng::new(seed);

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
        } else if rule.body != wire::Kind::Assignment || gate() {
            // Only the gate's batch is pinned to draw both empty and nonempty assignments.
            assert!(accepted > 0, "round trips for {:?}", rule.body);
            if rule.body == wire::Kind::Assignment {
                assert!(refused > 0, "a nonempty assignment's parts cannot be dropped");
            }
        }
    }
}

#[test]
fn drawn_worker_frames_arrive_whole_or_are_refused() -> Result<(), Case<Infallible>> {
    cases(this_test!("drawn_worker_frames_arrive_whole_or_are_refused"), [0x2216_0100], |seed| {
        check_receive(seed);
        Ok(())
    })
}

fn check_receive(seed: u64) {
    let engine = parse(wire::SCHEMA);
    let worker = parse(&richer(wire::SCHEMA));
    let mut peer = Plans::new(&[wire::SCHEMA], &List::uncapped())
        .expect("engine text")
        .judge(&[stated(&worker)])
        .expect("different worker layout");
    let root = worker.find(b"FromWorker").expect("worker declaration");
    let Shape::Enum { first, count } = worker.declaration(root).shape else { panic!("worker kinds") };
    let mut expected = Shim::new(Mode::Read, &worker, &engine, "FromWorker");
    let mut bytes = Buffer::with_capacity(ROOM);
    let mut current = Buffer::with_capacity(ROOM);
    let mut rng = Rng::new(seed);
    let mut sampled = 0;
    for tag in 0..count {
        let variant = worker.variant(VariantIx::new(first.index().checked_add(tag).expect("worker variant index")));
        let name = worker.spelled(variant.name);
        let refused = name == b"worker_only";
        if !refused && ![b"announced".as_slice(), b"fact", b"ask"].contains(&name) {
            continue;
        }
        sampled += 1;
        let tag = u8::try_from(tag).expect("worker kind tag");
        let kind = UP.checked_add(u16::from(tag)).expect("worker kind");
        for _ in 0..12 {
            bytes.clear();
            put(&mut bytes, &[tag]);
            draw(&worker, variant.record.expect("worker kind payload"), &mut rng, &mut bytes, false);
            let (_, body) = bytes.as_slice().split_first().expect("drawn tag and body");
            let received = wire::convert::receive(&mut peer, kind, body, &wire::CEILINGS);
            if refused {
                let failure = received.expect_err("the engine lacks this kind");
                assert_eq!(failure.cause, Cause::Variant);
                assert_eq!(&*failure.read(), b"jig-worker-wire FromWorker.worker_only: variant");
            } else {
                let received = received.expect("worker kind has a read plan");
                expected.convert(bytes.as_slice(), &mut current).expect("read drawn worker value");
                let (&tag, body) = current.as_slice().split_first().expect("engine tag and body");
                assert_eq!(received.kind(), UP.checked_add(u16::from(tag)).expect("engine kind"));
                assert_eq!(received.bytes().get(8..).expect("body follows header"), body);
            }
        }
    }
    assert_eq!(sampled, 4);
}
