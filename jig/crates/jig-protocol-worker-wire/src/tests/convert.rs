//! The frame and its carried layouts cross together (jig/docs/design/protocol/channel/layouts.md, 8).
use crate::layouts::{Cause, Gap, Peer, Plans, Site, Unconverted};
use crate::{KINDS, Kind, convert, v22 as wire};
use skein_channel::{Frame, frame_writer};
use skein_codec::layout::{Mode, Refused};
use skein_lib::{Buffer, List, Reader, Rng};
use wire::Record;

use super::layouts::samples::{DOWN, ROOM, Shim, UP, frame as draw_frame, parse, replace, stated};
mod fixtures;
use fixtures::{LIMITS, Layouts, Payloads, parts, turn, window};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

fn frame(kind: u16, body: &[u8]) -> Frame {
    let mut writer = frame_writer(kind, u32::try_from(body.len()).expect("sample length")).expect("sample frame");
    writer.put(body).expect("room for sample");
    writer.finish().expect("complete frame")
}

fn body(frame: &Frame) -> &[u8] {
    frame.bytes().get(8..).expect("body follows header")
}

fn tag(kind: u16, base: u16) -> u8 {
    u8::try_from(kind.checked_sub(base).expect("direction's kind")).expect("kind has one tag byte")
}

fn number(base: u16, tag: u8) -> u16 {
    base.checked_add(u16::from(tag)).expect("base plus one-byte tag")
}

fn from_tagged(base: u16, tagged: &[u8]) -> Frame {
    let (&tag, body) = tagged.split_first().expect("tagged test value");
    frame(number(base, tag), body)
}

struct Shims {
    send: Shim,
    sent: Shim,
    receive: Shim,
    out: Buffer,
}

impl Shims {
    fn new(engine: &skein_codec::schema::Schema, worker: &skein_codec::schema::Schema) -> Self {
        Self {
            send: Shim::new(Mode::Keep, engine, worker, "FromEngine"),
            sent: Shim::new(Mode::Read, worker, engine, "FromEngine"),
            receive: Shim::new(Mode::Keep, engine, worker, "FromWorker"),
            out: Buffer::with_capacity(ROOM),
        }
    }

    fn check_down(&mut self, peer: &mut Peer, engine: &[u8], worker_parts: &[u8]) {
        let current = from_tagged(DOWN, engine);
        let sent = convert::send(peer, current, &LIMITS).expect("engine sends a kind the worker knows");
        self.send.convert(worker_parts, &mut self.out).expect("form the expected worker frame");
        assert_eq!(sent.bytes(), from_tagged(DOWN, self.out.as_slice()).bytes(), "kind {:x}", sent.kind());
        let mut derived = Buffer::with_capacity(ROOM);
        derived.put(&[tag(sent.kind(), DOWN)]).expect("frame tag");
        derived.put(body(&sent)).expect("small frame body");
        self.sent.convert(derived.as_slice(), &mut self.out).expect("read back the derived record");
        let decoded = wire::FromEngine::decode(&LIMITS, &mut Reader::new(self.out.as_slice())).expect("current codec");
        assert_eq!(&*decoded.encode(&LIMITS).expect("current record"), worker_parts);
    }

    fn check_up(&mut self, peer: &mut Peer, engine: &[u8], worker_parts: &[u8]) {
        self.receive.convert(worker_parts, &mut self.out).expect("form the worker frame");
        let worker = from_tagged(UP, self.out.as_slice());
        let received =
            convert::receive(peer, worker.kind(), body(&worker), &LIMITS).expect("worker sends a known kind");
        assert_eq!(received.bytes(), from_tagged(UP, engine).bytes(), "kind {:x}", received.kind());
    }
}

#[test]
fn every_kind_crosses_with_its_carried_parts() {
    let layouts = Layouts::new();
    let mut peer = layouts.peer();
    let (ours, theirs) = Payloads::both(&layouts);
    let ([engine, ..], [worker, ..]) = (&layouts.engine, &layouts.worker);
    let mut shims = Shims::new(engine, worker);
    let mut covered = List::uncapped();
    for (engine, worker) in ours.engine().into_iter().zip(theirs.engine()) {
        let current = engine.encode(&LIMITS).expect("current engine frame");
        let carried = worker.encode(&LIMITS).expect("worker payloads in engine record");
        shims.check_down(&mut peer, &current, &carried);
        covered.push(from_tagged(DOWN, &current).kind()).expect("covered engine kind");
    }
    for (engine, worker) in ours.worker().into_iter().zip(theirs.worker()) {
        let current = engine.encode(&LIMITS).expect("current worker frame");
        let carried = worker.encode(&LIMITS).expect("worker payloads in engine record");
        shims.check_up(&mut peer, &current, &carried);
        covered.push(from_tagged(UP, &current).kind()).expect("covered worker kind");
    }
    let mut rng = Rng::new(0x2201_8001);
    for rule in KINDS {
        if covered.as_slice().contains(&rule.kind) {
            continue;
        }
        let record = draw_frame(engine, *rule, &mut rng, true);
        if rule.kind >= DOWN {
            shims.check_down(&mut peer, &record, &record);
        } else {
            shims.check_up(&mut peer, &record, &record);
        }
    }
}

fn refusal(got: Unconverted, cause: Cause, wire: &str, declaration: &str, part: &str, words: &[u8]) {
    assert_eq!(got.cause, cause);
    assert_eq!(got.wire.as_ref(), wire);
    assert_eq!(got.declaration.as_ref(), declaration);
    assert_eq!(got.part.as_ref(), part);
    assert_eq!(&*got.read(), words);
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the fixture edits an already parsed schema text")]
fn refusals_keep_the_plan_names_and_causes() {
    let mut layouts = Layouts::new();
    let [frames, ..] = &mut layouts.worker;
    let text = core::str::from_utf8(frames.text()).expect("worker schema text");
    let text = replace(text, ", part_busy: PartBusy", "");
    let text = replace(&text, "    parts: bytes 134217728 = 0x\n", "");
    *frames = parse(&replace(&text, "record Check { name: text 32 }", "record Check { name: text 3 }"));
    let mut peer = layouts.peer();
    let absent = wire::PartBusy { task: 1, attempt: 2, number: 3 }.encode(&LIMITS).expect("part busy");
    refusal(
        convert::send(&mut peer, frame(Kind::PartBusy.number(), &absent), &LIMITS).expect_err("worker lacks this kind"),
        Cause::Variant,
        "jig-worker-wire",
        "FromEngine",
        "part_busy",
        b"jig-worker-wire FromEngine.part_busy: variant",
    );
    refusal(
        convert::receive(&mut peer, UP, &[8], &LIMITS).expect_err("the engine lacks worker_only"),
        Cause::Variant,
        "jig-worker-wire",
        "FromWorker",
        "worker_only",
        b"jig-worker-wire FromWorker.worker_only: variant",
    );

    let (ours, _) = Payloads::both(&layouts);
    let assignment = ours.assignment().encode(&LIMITS).expect("nonempty kept parts");
    let error = convert::send(&mut peer, frame(Kind::Assignment.number(), &assignment), &LIMITS)
        .expect_err("the worker cannot keep a non-default field it lacks");
    assert_eq!(&*error.kept(), b"the worker's layout cannot keep jig-worker-wire Assignment.parts: field");
    refusal(error, Cause::Field, "jig-worker-wire", "Assignment", "parts", b"jig-worker-wire Assignment.parts: field");

    let check = wire::Check { name: Box::from(b"abcd".as_slice()) }.encode(&LIMITS).expect("engine name bound");
    refusal(
        convert::send(&mut peer, frame(Kind::Check.number(), &check), &LIMITS).expect_err("worker bound is three"),
        Cause::Narrowed,
        "jig-worker-wire",
        "Check",
        "name",
        b"jig-worker-wire Check.name: narrowed",
    );
}

#[test]
fn assignment_parts_gap_uses_the_untranslated_refusal_words() {
    let worker = parse(&replace(crate::SCHEMA, "    parts: bytes 134217728 = 0x\n", ""));
    let mut peer = Plans::new(&[crate::SCHEMA], &List::uncapped())
        .expect("current text")
        .judge(&[stated(&worker)])
        .expect("the worker's layout can connect");
    let gaps = peer.gaps();
    assert_eq!(
        gaps.as_slice(),
        &[Gap {
            wire: "jig-worker-wire".into(),
            site: Site::Frame,
            why: Refused::Field,
            declaration: "Assignment".into(),
            part: "parts".into(),
        }]
    );
    let gap = gaps.get(0).expect("one gap");
    assert_eq!(&*gap.words(), b"the worker's layout cannot keep jig-worker-wire Assignment.parts: field");

    let (ours, _) = Payloads::both(&Layouts::new());
    let assignment = ours.assignment().encode(&LIMITS).expect("nonempty kept parts");
    let refused = convert::send(&mut peer, frame(Kind::Assignment.number(), &assignment), &LIMITS)
        .expect_err("the assignment has parts the worker cannot keep");
    assert_eq!(gap.words(), refused.kept());
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the fixture edits an already parsed schema text")]
fn conversion_bounds_the_frame_and_each_part() {
    let mut layouts = Layouts::new();
    let [frame_schema, ..] = &mut layouts.worker;
    let text = core::str::from_utf8(frame_schema.text()).expect("worker frame schema");
    let text = replace(text, "record Check { name: text 32 }", "record Check { name: text 32 worker_note: u32 = 9 }");
    let text = replace(&text, "record AskBytes { bytes: bytes 1048576 }", "record AskBytes { bytes: bytes 4 }");
    *frame_schema = parse(&text);
    let mut peer = layouts.peer();
    let ([engine, ..], [worker, ..]) = (&layouts.engine, &layouts.worker);
    let (ours, theirs) = Payloads::both(&layouts);
    let mut shims = Shims::new(engine, worker);

    // The full-size Check at either limit grows by the worker's default.
    // Reusing the smaller bound after a limits change would refuse the second.
    for name in [b"a".as_slice(), b"abcd"] {
        let limits = wire::Limits { check_name: u32::try_from(name.len()).expect("short name"), ..LIMITS };
        let check = wire::Check { name: name.into() }.encode(&limits).expect("at the engine limit");
        let sent =
            convert::send(&mut peer, frame(Kind::Check.number(), &check), &limits).expect("fits the worker bound");
        assert_eq!(body(&sent).strip_suffix(&9_u32.to_be_bytes()), Some(check.as_ref()));
    }

    // The application answer starts above the worker's narrowed field bound;
    // only converting the part before the record makes it fit.
    let answer = |bytes| {
        wire::FromEngine::Asked(wire::Asked {
            run: 1,
            attempt: 2,
            number: 3,
            answer: wire::AskAnswer::Bytes(wire::AskBytes { bytes }),
        })
        .encode(&LIMITS)
        .expect("answer fixture")
    };
    shims.check_down(&mut peer, &answer(ours.answer), &answer(theirs.answer));
    let ask = wire::FromWorker::Ask(wire::Ask { run: 1, attempt: 2, number: 3, bytes: theirs.ask });
    let mut tagged = Buffer::with_capacity(ROOM);
    Shim::new(Mode::Keep, engine, worker, "FromWorker")
        .convert(&ask.encode(&LIMITS).expect("worker ask payload"), &mut tagged)
        .expect("worker frame bytes");
    let worker = from_tagged(UP, tagged.as_slice());
    let fits_source = wire::Limits { ask_bytes: 5, ..LIMITS };
    refusal(
        convert::receive(&mut peer, worker.kind(), body(&worker), &fits_source)
            .expect_err("the converted part is larger"),
        Cause::Large,
        "convert-test-app",
        "Up",
        "",
        b"convert-test-app Up: too large",
    );
    let smaller_frame = wire::Limits { ask_bytes: 4, ..LIMITS };
    refusal(
        convert::receive(&mut peer, worker.kind(), body(&worker), &smaller_frame)
            .expect_err("the raw part cannot fit in the converted frame"),
        Cause::Large,
        "jig-worker-wire",
        "FromWorker",
        "ask",
        b"jig-worker-wire FromWorker.ask: too large",
    );
}

#[test]
fn resuming_drops_refused_kept_parts_but_requires_every_turn() {
    let layouts = Layouts::new();
    let mut peer = layouts.peer();
    let (ours, theirs) = Payloads::both(&layouts);
    let [engine, _, _, transcript, _] = &layouts.engine;
    let [worker, ..] = &layouts.worker;
    let missing = turn(transcript, &mut Rng::new(0x2231_8201), 1);
    let mut current = ours.assignment();
    current.parts = parts(&[ours.turns.first().expect("first turn").clone(), missing.clone()]);
    let mut expected = theirs.assignment();
    expected.parts = parts(&theirs.turns[..1]);
    Shims::new(engine, worker).check_down(
        &mut peer,
        &wire::FromEngine::Assignment(current).encode(&LIMITS).expect("resumed engine frame"),
        &wire::FromEngine::Assignment(expected).encode(&LIMITS).expect("worker frame holds the kept part"),
    );
    let mut current = ours.assignment();
    current.transcript = window(&[ours.turns.first().expect("first turn").clone(), missing]);
    let body = current.encode(&LIMITS).expect("resumed assignment with the bad turn");
    refusal(
        convert::send(&mut peer, frame(Kind::Assignment.number(), &body), &LIMITS)
            .expect_err("a window turn is required"),
        Cause::Variant,
        "smith-transcript",
        "TurnKind",
        "window",
        b"smith-transcript TurnKind.window: variant",
    );
}

#[test]
fn same_layout_moves_send_and_only_allocates_receive_ownership() {
    let layouts = Layouts::new();
    let mut peer = Layouts::plans().judge(&layouts.engine.each_ref().map(stated)).expect("same layouts");
    assert!(peer.same());
    // These bytes are not an Assignment or an Answer: neither path may decode them.
    let bytes = b"same bytes";
    let source = frame(Kind::Assignment.number(), bytes);
    let pointer = source.bytes().as_ptr();
    let span = skein_heap::Span::start();
    let sent = convert::send(&mut peer, source, &LIMITS).expect("same layout");
    assert_eq!(span.made(), 0);
    assert_eq!(sent.bytes().as_ptr(), pointer);
    assert_eq!(sent.bytes(), frame(Kind::Assignment.number(), bytes).bytes());
    let span = skein_heap::Span::start();
    let received = convert::receive(&mut peer, Kind::Answer.number(), bytes, &LIMITS).expect("same layout");
    assert_eq!(span.made(), 1, "only the owned frame");
    assert_eq!(received.bytes(), frame(Kind::Answer.number(), bytes).bytes());
}

#[test]
fn a_derived_pieces_layout_does_not_cross() {
    let mut layouts = Layouts::new();
    let [_, pieces, ..] = &mut layouts.worker;
    *pieces = Layouts::pieces_change();
    let refusal = Layouts::plans().judge(&layouts.worker.each_ref().map(stated)).expect_err("pieces stay opaque");
    assert!(refusal.starts_with(b"wire smith-pieces: the worker's layout "));
    assert!(refusal.ends_with(b", and pieces cross as written"));
}

#[test]
fn a_part_without_a_root_crosses_as_written() {
    let layouts = Layouts::new();
    let ([engine, ..], [worker, ..]) = (&layouts.engine, &layouts.worker);
    let mut peer = Plans::new(&[crate::SCHEMA], &List::uncapped())
        .expect("no application roots")
        .judge(&[stated(worker)])
        .expect("frame plans");
    let mut shim = Shims::new(engine, worker);
    let opaque = b"opaque".as_slice();
    let up = wire::FromWorker::Ask(wire::Ask { run: 7, attempt: 8, number: 9, bytes: opaque.into() });
    let down = wire::FromEngine::Asked(wire::Asked {
        run: 7,
        attempt: 8,
        number: 9,
        answer: wire::AskAnswer::Bytes(wire::AskBytes { bytes: opaque.into() }),
    });
    let up = up.encode(&LIMITS).expect("opaque ask");
    let down = down.encode(&LIMITS).expect("opaque answer");
    shim.check_up(&mut peer, &up, &up);
    shim.check_down(&mut peer, &down, &down);
}
