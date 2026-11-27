//! Opening and converting worker layouts (jig/docs/design/protocol/channel/layouts.md, 8).
use crate as wire;
use crate::layouts::{Cause, MAX_TEXTS, Nested, Peer, Plans, Site, Theirs};
use crate::{Carried, KINDS, Kind, Record, Root, v22};
#[expect(clippy::disallowed_types, reason = "the test spells a fingerprint independently of the protocol")]
use core::fmt::Write;
use skein_codec::{
    layout::{Mode, Refused},
    schema::Schema,
};
use skein_lib::{Buffer, List, Reader, Rng};

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "sample fixtures edit text and read ASCII schema identifiers"
)]
#[path = "layout_samples.rs"]
pub(super) mod samples;
use super::fixtures::LIMITS;
use samples::{DOWN, ROOM, Shim, UP, draw, frame, parse, put, replace, stated};

const APP: &str = "family jig-test-app 1\nrecord Down { number: u16 engine_note: u32 = 7 }\nrecord Up { number: u16 engine_note: u8 = 7 }";
const TEXTS: [&str; 5] =
    [crate::SCHEMA, smith_channel::PIECES_SCHEMA, smith_charter::SCHEMA, smith_transcript::SCHEMA, APP];
const CARRIED: [(Carried, &str, Mode); 9] = [
    (Carried::Items, "Down", Mode::Keep),
    (Carried::Left, "Up", Mode::Read),
    (Carried::Saved, "Up", Mode::Read),
    (Carried::Preparation, "Up", Mode::Read),
    (Carried::JobSpec, "Down", Mode::Keep),
    (Carried::JobResult, "Up", Mode::Read),
    (Carried::Grant, "Down", Mode::Keep),
    (Carried::Ask, "Up", Mode::Read),
    (Carried::Answer, "Down", Mode::Keep),
];

fn roots() -> List<Root> {
    let mut roots = List::with_capacity(9);
    for (carried, declaration, _) in CARRIED {
        roots.push(Root { carried, wire: "jig-test-app", declaration }).expect("one root per part");
    }
    roots
}

fn judge_worker(worker: &Schema) -> Peer {
    Plans::new(&[crate::SCHEMA], &List::uncapped())
        .expect("current text")
        .judge(&[stated(worker)])
        .expect("worker layout fits")
}

fn current_table() -> List<skein_channel::Kind> {
    crate::schema(&LIMITS).expect("current wire table").kinds
}

fn kind_in(table: &List<skein_channel::Kind>, number: u16) -> Option<skein_channel::Kind> {
    for rule in table {
        if rule.kind == number {
            return Some(*rule);
        }
    }
    None
}

#[test]
fn equal_layouts_do_not_parse_worker_texts_or_convert_values() {
    let mut plans = Plans::new(&TEXTS, &roots()).expect("engine texts");
    let engine = TEXTS.map(parse);
    let mut theirs = engine.each_ref().map(stated);
    for wire in &mut theirs {
        wire.text = b"not a schema";
    }
    let mut peer = plans.judge(&theirs).expect("equal digests bypass parsing");
    assert!(peer.same());
    assert!(peer.gaps().is_empty());
    assert_eq!(plans.derivations(crate::wire::FAMILY), 0);
    assert_eq!(peer.table(&LIMITS).expect("peer table"), current_table());
    let mut out = Buffer::with_capacity(8);
    let bytes = b"opaque";
    for rule in KINDS {
        if rule.kind >= DOWN {
            assert_eq!(peer.sends(rule.body), Some(rule.kind));
            assert_eq!(peer.send(rule.body, bytes, &mut out), Ok(rule.kind));
        } else {
            assert_eq!(peer.receive(rule.kind, bytes, &mut out), Ok(rule.body));
        }
        assert_eq!(out.as_slice(), bytes);
    }
    for nested in [Nested::Charter, Nested::Turn] {
        peer.down(nested, bytes, &mut out).expect("same part down");
        assert_eq!(out.as_slice(), bytes);
    }
    for nested in [Nested::Result, Nested::Turn] {
        peer.up(nested, bytes, &mut out).expect("same part up");
        assert_eq!(out.as_slice(), bytes);
    }
    for (part, _, mode) in CARRIED {
        match mode {
            Mode::Keep => peer.down(Nested::Carried(part), bytes, &mut out),
            Mode::Read => peer.up(Nested::Carried(part), bytes, &mut out),
        }
        .expect("same carried part");
        assert_eq!(out.as_slice(), bytes);
    }
    assert_eq!(peer.up(Nested::Charter, bytes, &mut out).expect_err("wrong direction").cause, Cause::Malformed);
}

#[test]
fn worker_only_additions_do_not_make_keep_gaps() {
    let text = samples::richer(crate::SCHEMA);
    let text = replace(&text, ", part_busy: PartBusy", ", part_busy: PartBusy, worker_only: PartBusy");
    let peer = judge_worker(&parse(&text));
    assert!(!peer.same());
    assert!(peer.gaps().is_empty());
}

#[test]
fn missing_optional_kinds_are_not_gaps_but_their_present_payloads_can_refuse() {
    let text = replace(
        crate::SCHEMA,
        ", restarting: Restarting, figures: Figures, read_usage: ReadUsage, pause: Pause, part_kept: PartKept, part_busy: PartBusy",
        "",
    );
    assert!(judge_worker(&parse(&text)).gaps().is_empty());

    let text = replace(&text, "job: Job, ", "");
    let text = replace(&text, "asked: Asked", "asked: Asked, figures: Figures");
    let text = replace(&text, "places: u32 limits: list 1024 Limit", "places: u32 limits: list 512 Limit");
    let gaps = judge_worker(&parse(&text)).gaps();
    assert_eq!(gaps.len(), 2);
    let job = gaps.get(0).expect("required kind");
    assert_eq!((job.site, job.why), (Site::Frame, Refused::Variant));
    assert_eq!(&*job.words(), b"the worker's layout cannot keep jig-worker-wire FromEngine.job: variant");
    let figures = gaps.get(1).expect("present optional kind's payload");
    assert_eq!((figures.site, figures.why), (Site::Frame, Refused::Narrowed));
    assert_eq!(&*figures.words(), b"the worker's layout cannot keep jig-worker-wire Figures.limits: narrowed");
}

#[test]
fn keep_gaps_follow_value_type_order_not_wire_or_root_order() {
    let frame = replace(crate::SCHEMA, "record Check { name: text 32 }", "record Check { name: text 3 }");
    let charter = replace(smith_charter::SCHEMA, "    check_timeout: option duration\n", "");
    let turn = replace(smith_transcript::SCHEMA, "    model: text 128\n", "    model: text 32\n");
    let app = replace(APP, " engine_note: u32 = 7", "");
    let worker = [&app, &turn, &charter, &frame].map(|text| parse(text));
    let mut roots = List::with_capacity(9);
    for &(carried, declaration, _) in CARRIED.iter().rev() {
        roots.push(Root { carried, wire: "jig-test-app", declaration }).expect("one root per part");
    }
    let gaps = Plans::new(&[APP, crate::SCHEMA, smith_charter::SCHEMA, smith_transcript::SCHEMA], &roots)
        .expect("engine texts")
        .judge(&worker.each_ref().map(stated))
        .expect("worker layout fits")
        .gaps();
    let expected = [
        (Site::Frame, "jig-worker-wire", Refused::Narrowed, "Check", "name"),
        (Site::Charter, "smith-charter", Refused::Field, "Charter", "check_timeout"),
        (Site::Turn, "smith-transcript", Refused::Narrowed, "Turn", "model"),
        (Site::Carried(Carried::Items), "jig-test-app", Refused::Field, "Down", "engine_note"),
        (Site::Carried(Carried::JobSpec), "jig-test-app", Refused::Field, "Down", "engine_note"),
        (Site::Carried(Carried::Grant), "jig-test-app", Refused::Field, "Down", "engine_note"),
        (Site::Carried(Carried::Answer), "jig-test-app", Refused::Field, "Down", "engine_note"),
    ];
    assert_eq!(gaps.len(), 7);
    for (gap, expected) in gaps.iter().zip(expected) {
        assert_eq!((gap.site, gap.wire.as_ref(), gap.why, gap.declaration.as_ref(), gap.part.as_ref()), expected);
    }
}

#[test]
fn cached_kind_bounds_follow_the_supplied_limits() {
    const ASK_BYTES: u32 = 5;
    // Ask has three u64 fields and a u32 length before its bytes.
    const ASK_BOUND: u32 = 3 * 8 + 4 + ASK_BYTES;
    let mut plans = Plans::new(&[crate::SCHEMA], &List::uncapped()).expect("engine text");
    let worker = parse(crate::SCHEMA);
    let peer = plans.judge(&[stated(&worker)]).expect("same worker layout");
    plans.fit_table(&peer, &LIMITS).expect("prime the cache");

    let limits = v22::Limits { ask_bytes: ASK_BYTES, ..LIMITS };
    let table = plans.fit_table(&peer, &limits).expect("fit updated bounds");
    assert_eq!(kind_in(&table, Kind::Ask.number()).expect("ask kind").largest, ASK_BOUND);
}

#[test]
fn frames_round_trip_at_both_worker_layouts() {
    let engine = parse(crate::SCHEMA);
    let older = parse(&samples::older(crate::SCHEMA));
    let richer = parse(&samples::richer(crate::SCHEMA));
    let mut rng = Rng::new(0x2280_0100);
    let mut bridged = Buffer::with_capacity(ROOM);
    let mut returned = Buffer::with_capacity(ROOM);
    let mut tagged = Buffer::with_capacity(ROOM);
    for (worker, missing) in [(&older, Some(Kind::PartBusy)), (&richer, None)] {
        let mut peer = judge_worker(worker);
        assert!(!peer.same());
        let mut sent = Shim::new(Mode::Read, worker, &engine, "FromEngine");
        let mut received = Shim::new(Mode::Keep, &engine, worker, "FromWorker");
        for rule in KINDS {
            let original = frame(&engine, *rule, &mut rng, true);
            let (&tag, body) = original.split_first().expect("tagged value");
            if rule.kind >= DOWN {
                if Some(rule.body) == missing {
                    assert_eq!(
                        peer.send(rule.body, body, &mut bridged).expect_err("absent kind").cause,
                        Cause::Variant
                    );
                    continue;
                }
                let kind = peer.send(rule.body, body, &mut bridged).expect("engine sends supported kind");
                assert_eq!(peer.sends(rule.body), Some(kind));
                tagged.clear();
                put(&mut tagged, &[offset(kind, DOWN)]);
                put(&mut tagged, bridged.as_slice());
                sent.convert(tagged.as_slice(), &mut returned).expect("shim reads worker layout");
                let decoded = v22::FromEngine::decode(&crate::CEILINGS, &mut Reader::new(returned.as_slice()))
                    .expect("shim produced engine codec bytes");
                assert_eq!(decoded.encode(&crate::CEILINGS).expect("frame encodes"), original, "{:?}", rule.body);
            } else {
                received.convert(&original, &mut tagged).expect("shim keeps worker message");
                let (&worker, body) = tagged.as_slice().split_first().expect("worker tag");
                assert_eq!(peer.receive(number(worker, UP), body, &mut returned), Ok(rule.body));
                assert_eq!(number(tag, UP), rule.kind);
                assert_eq!(returned.as_slice(), original.get(1..).expect("original body"), "{:?}", rule.body);
            }
        }
    }
}

fn offset(kind: u16, base: u16) -> u8 {
    u8::try_from(kind.checked_sub(base).expect("in the frame range")).expect("one byte tag")
}

fn number(tag: u8, base: u16) -> u16 {
    base.checked_add(u16::from(tag)).expect("in the frame range")
}

#[test]
fn nested_values_cross_at_each_owners_layout_and_in_their_direction() {
    let engine = TEXTS.map(parse);
    let charter = replace(smith_charter::SCHEMA, "    check_timeout: option duration", "    worker_note: u8 = 6");
    let charter = replace(&charter, "    label: option text 64", "    worker_note: u8 = 8");
    let transcript = replace(smith_transcript::SCHEMA, "    system: option text 1048576", "    worker_note: u8 = 9");
    let app = replace(APP, "engine_note: u32 = 7", "worker_note: u8 = 8");
    let app = replace(&app, "engine_note: u8 = 7", "worker_note: u16 = 8");
    let theirs =
        [parse(&samples::older(crate::SCHEMA)), parse(TEXTS[1]), parse(&charter), parse(&transcript), parse(&app)];
    let mut peer = Plans::new(&TEXTS, &roots())
        .expect("engine texts")
        .judge(&theirs.each_ref().map(stated))
        .expect("all plans derive");
    assert!(!peer.same());
    let mut rng = Rng::new(0x0508_2280);
    let mut original = Buffer::with_capacity(ROOM);
    let mut changed = Buffer::with_capacity(ROOM);
    let mut returned = Buffer::with_capacity(ROOM);
    for (nested, declaration, mode, ours, worker) in [
        (Nested::Charter, "Charter", Mode::Keep, &engine[2], &theirs[2]),
        (Nested::Result, "RunResult", Mode::Read, &engine[2], &theirs[2]),
        (Nested::Turn, "Turn", Mode::Keep, &engine[3], &theirs[3]),
        (Nested::Turn, "Turn", Mode::Read, &engine[3], &theirs[3]),
    ]
    .into_iter()
    .chain(CARRIED.map(|(part, declaration, mode)| (Nested::Carried(part), declaration, mode, &engine[4], &theirs[4])))
    {
        original.clear();
        draw(ours, ours.find(declaration.as_bytes()).expect("sample root"), &mut rng, &mut original, true);
        match mode {
            Mode::Keep => {
                peer.down(nested, original.as_slice(), &mut changed).expect("peer keeps part");
                Shim::new(Mode::Read, worker, ours, declaration)
                    .convert(changed.as_slice(), &mut returned)
                    .expect("worker reads part");
            }
            Mode::Read => {
                Shim::new(Mode::Keep, ours, worker, declaration)
                    .convert(original.as_slice(), &mut changed)
                    .expect("worker keeps part");
                peer.up(nested, changed.as_slice(), &mut returned).expect("peer reads part");
            }
        }
        assert_ne!(changed.as_slice(), original.as_slice(), "{nested:?} has another layout");
        assert_eq!(returned.as_slice(), original.as_slice(), "{nested:?}");
    }
    peer.up(Nested::Carried(Carried::Left), &[0, 42, 0, 99], &mut returned)
        .expect("up skips a nondefault worker field");
    assert_eq!(returned.as_slice(), &[0, 42, 7]);
}

#[test]
fn refusing_a_value_names_the_source_part_and_clears_output() {
    let worker =
        replace(&samples::older(crate::SCHEMA), "record Check { name: text 32 }", "record Check { name: text 3 }");
    let mut peer = judge_worker(&parse(&worker));
    let mut out = Buffer::with_capacity(ROOM);
    assert_eq!(peer.sends(Kind::PartBusy), None);
    let refused = peer.send(Kind::PartBusy, &[], &mut out).expect_err("worker lacks the kind");
    assert_eq!(refused.cause, Cause::Variant);
    assert_eq!(&*refused.read(), b"jig-worker-wire FromEngine.part_busy: variant");

    let mut assignment = super::fixtures::assignment_resumed();
    assignment.parts = Box::from(b"kept-part".as_slice());
    let body = assignment.encode(&crate::CEILINGS).expect("valid assignment bytes");
    let refused = peer.send(Kind::Assignment, &body, &mut out).expect_err("only the default parts may drop");
    assert_eq!(refused.cause, Cause::Field);
    assert_eq!(&*refused.kept(), b"the worker's layout cannot keep jig-worker-wire Assignment.parts: field");
    assert!(out.is_empty());

    let body = v22::Check { name: Box::from(b"abcd".as_slice()) }.encode(&LIMITS).expect("engine bound holds");
    let refused = peer.send(Kind::Check, &body, &mut out).expect_err("worker bound is three");
    assert_eq!(refused.cause, Cause::Narrowed);
    assert_eq!(&*refused.read(), b"jig-worker-wire Check.name: narrowed");
    let body = v22::Check { name: Box::from(b"abc".as_slice()) }.encode(&LIMITS).expect("at the worker bound");
    assert_eq!(peer.send(Kind::Check, &body, &mut out), Ok(Kind::Check.number()));
    assert_eq!(out.as_slice(), &*body);
    let refused = peer.send(Kind::Check, &body, &mut Buffer::with_capacity(6)).expect_err("output one byte short");
    assert_eq!(refused.cause, Cause::Large);
    assert_eq!(&*refused.read(), b"jig-worker-wire FromEngine.check: too large");

    let mut peer = judge_worker(&parse(&samples::richer(crate::SCHEMA)));
    let shifted_fact = Kind::Fact.number().checked_add(1).expect("worker tag inserted");
    let refused = peer.receive(shifted_fact, &[1], &mut out).expect_err("fact body is cut");
    assert_eq!(refused.cause, Cause::Malformed);
    assert_eq!(&*refused.read(), b"jig-worker-wire FromWorker.fact: malformed");
    assert!(out.is_empty());
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "a test assembles refusal texts independently of their formatter"
)]
fn opening_refuses_the_first_bad_wire_in_engine_order_with_its_words() {
    let mut plans = Plans::new(&TEXTS, &roots()).expect("engine texts");
    let engine = TEXTS.map(parse);
    let [jig, pieces, charter, transcript, app] = engine.each_ref().map(stated);
    let malformed = Theirs { layout: [0; 32], text: b"family smith-charter 5\n@", ..charter };
    let foreign = Theirs { layout: [0; 32], text: APP.as_bytes(), ..charter };
    let wrong = Theirs { layout: [1; 32], ..charter };
    let pieces_wrong = Theirs { layout: [2; 32], ..pieces };
    let mut pieces_engine = String::new();
    for byte in pieces.layout.iter().take(8) {
        write!(pieces_engine, "{byte:02x}").expect("two hex digits");
    }
    let pieces_text = format!(
        "wire smith-pieces: the worker's layout 0202020202020202 is not the engine's {pieces_engine}, and pieces cross as written"
    );
    for (wires, why) in [
        ([app, transcript, malformed, pieces].as_slice(), b"wire jig-worker-wire: the worker states none".as_slice()),
        ([app, transcript, malformed, pieces_wrong, jig].as_slice(), pieces_text.as_bytes()),
        (
            [app, transcript, malformed, pieces, jig].as_slice(),
            b"wire smith-charter: the worker's text does not parse at line 2",
        ),
        (
            [app, transcript, foreign, pieces, jig].as_slice(),
            b"wire smith-charter: the worker's text is of jig-test-app",
        ),
        (
            [app, transcript, wrong, pieces, jig].as_slice(),
            b"wire smith-charter: the worker's text is not its layout 0101010101010101",
        ),
        ([app, transcript, charter, pieces, jig, jig].as_slice(), b"wire jig-worker-wire: stated twice"),
    ] {
        assert_eq!(&*plans.judge(wires).expect_err("opening refusal"), why);
    }
    let name = "n".repeat(300);
    let extra = Theirs { name: &name, layout: [0; 32], text: b"not a schema" };
    assert!(plans.judge(&[jig, pieces, charter, transcript, app, extra]).expect("unknown wire is ignored").same());
    let refused = plans
        .judge(&[jig, pieces, charter, transcript, app, extra, extra])
        .expect_err("even unknown names may not repeat");
    assert_eq!(&*refused, format!("wire {name}: stated twice").as_bytes().get(..256).expect("long diagnostic"));
}

#[test]
fn a_missing_default_refuses_read_but_not_keep_at_opening() {
    let mut plans = Plans::new(&[smith_charter::SCHEMA], &List::uncapped()).expect("engine charter text");
    let retyped = parse(&replace(smith_charter::SCHEMA, "instructions: text 262144", "instructions: u32"));
    let refused = plans.judge(&[stated(&retyped)]).expect_err("retype is not derived");
    assert_eq!(&*refused, b"wire smith-charter Charter: no plan to the worker: retyped at Charter.instructions");
    let no_result = parse(&replace(smith_charter::SCHEMA, "    items: list 256 Item\n", ""));
    let refused = plans.judge(&[stated(&no_result)]).expect_err("Read needs an engine field's default");
    assert_eq!(&*refused, b"wire smith-charter RunResult: no plan from the worker: no default at RunResult.items");
    let no_charter = parse(&replace(smith_charter::SCHEMA, "    instructions: text 262144\n", ""));
    let mut peer = plans.judge(&[stated(&no_charter)]).expect("Keep can refuse per value");
    let schema = parse(smith_charter::SCHEMA);
    let mut original = Buffer::with_capacity(ROOM);
    draw(&schema, schema.find(b"Charter").expect("Charter root"), &mut Rng::new(5), &mut original, true);
    let mut out = Buffer::with_capacity(ROOM);
    let refused =
        peer.down(Nested::Charter, original.as_slice(), &mut out).expect_err("Charter.instructions has no default");
    assert_eq!(&*refused.kept(), b"the worker's layout cannot keep smith-charter Charter.instructions: field");
    peer.down(Nested::Carried(Carried::Items), b"opaque", &mut out).expect("an unregistered part crosses unchanged");
    assert_eq!(out.as_slice(), b"opaque");
}

#[test]
fn the_workers_table_uses_worker_tags_and_known_limit_names() {
    let worker = samples::richer(&samples::older(crate::SCHEMA));
    let mut peer = judge_worker(&parse(&worker));
    let table = peer.table(&LIMITS).expect("peer table");
    let current = current_table();
    assert_eq!(table.len(), current.len().checked_sub(1).expect("one fewer kind"));
    assert!(kind_in(&table, UP).is_none(), "worker-only receipt stays out of terms");
    assert!(kind_in(&table, Kind::PartBusy.number()).is_none(), "dropped send stays out of terms");
    for rule in KINDS {
        if rule.body == Kind::PartBusy {
            continue;
        }
        let mapped = if rule.body == Kind::Message {
            Kind::Assignment.number()
        } else if rule.body == Kind::Assignment {
            Kind::Message.number()
        } else if rule.kind < DOWN {
            rule.kind.checked_add(1).expect("one inserted tag")
        } else {
            rule.kind
        };
        let got = kind_in(&table, mapped).expect("mapped kind occurs");
        let original = kind_in(&current, rule.kind).expect("current kind");
        let delta = if rule.body == Kind::Assignment {
            LIMITS.assignment_parts.checked_add(4).expect("length plus bytes")
        } else {
            0
        };
        let expected = original.largest.checked_sub(delta).expect("removed field size");
        let expected =
            if rule.body == Kind::Fact { expected.checked_add(4).expect("one u32 appended") } else { expected };
        assert_eq!(got.largest, expected, "{:?}", rule.body);
        assert_eq!(got.direction, original.direction);
    }
    let refusal =
        peer.receive(UP, &[7], &mut Buffer::with_capacity(8)).expect_err("worker-only tag has no engine kind");
    assert_eq!(refusal.cause, Cause::Variant);
    assert_eq!(&*refusal.read(), b"jig-worker-wire FromWorker.worker_only: variant");

    let lacks_limit = replace(&worker, "worker_note: u32 = 19", "worker_note: option list 3 bytes 4");
    let mut plans = Plans::new(&TEXTS, &roots()).expect("engine texts");
    let schemas = [parse(&lacks_limit), parse(TEXTS[1]), parse(TEXTS[2]), parse(TEXTS[3]), parse(TEXTS[4])];
    let refused = plans.judge(&schemas.each_ref().map(stated)).expect_err("unknown limit at one option's value");
    assert_eq!(
        &*refused,
        b"wire jig-worker-wire: the worker's text bounds Fact by fact_worker_note_some, which the engine lacks"
    );
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test fixtures differ byte-for-byte in their comments"
)]
fn per_wire_cache_is_keyed_by_text_and_evicts_the_least_recently_seen() {
    let mut roots = List::with_capacity(1);
    roots.push(Root { carried: Carried::Items, wire: "jig-test-app", declaration: "Down" }).expect("one root");
    let changed = APP.replace("engine_note", "worker_note");
    // Renamed fields alone leave the family fingerprint unchanged. Add a
    // worker default to enter the conversion path before varying comments.
    let changed = replace(&changed, "worker_note: u32 = 7", "worker_note: u8 = 8");
    let mut workers = List::with_capacity(MAX_TEXTS.checked_add(1).expect("one over capacity"));
    for index in 0..=MAX_TEXTS {
        let text: String = format!("# worker {index}\n{changed}");
        workers.push(parse(&text)).expect("one fixture per text");
    }
    let first = stated(workers.get(0).expect("first worker"));
    let second = stated(workers.get(1).expect("second worker"));
    let mut plans = Plans::new(&[APP], &roots).expect("one engine text");
    assert_ne!(first.layout, stated(&parse(APP)).layout);
    assert_eq!(first.layout, second.layout, "comments do not change the layout");
    for index in 0..MAX_TEXTS {
        plans.judge(&[stated(workers.get(index).expect("worker within capacity"))]).expect("worker fits");
        assert_eq!(plans.derivations("jig-test-app"), index.checked_add(1).expect("one derivation per new text"));
    }
    plans.judge(&[first]).expect("cache hit refreshes the oldest entry");
    assert_eq!(plans.derivations("jig-test-app"), MAX_TEXTS);
    let wrong = Theirs { layout: [0; 32], ..first };
    assert_eq!(
        &*plans.judge(&[wrong]).expect_err("cached text still needs its layout"),
        b"wire jig-test-app: the worker's text is not its layout 0000000000000000"
    );
    plans.judge(&[stated(workers.get(MAX_TEXTS).expect("one over capacity"))]).expect("new entry evicts second");
    plans.judge(&[first]).expect("refreshed entry survives");
    assert_eq!(plans.derivations("jig-test-app"), MAX_TEXTS.checked_add(1).expect("one new plan"));
    plans.judge(&[second]).expect("evicted entry is derived again");
    assert_eq!(plans.derivations("jig-test-app"), MAX_TEXTS.checked_add(2).expect("two new plans"));
}
