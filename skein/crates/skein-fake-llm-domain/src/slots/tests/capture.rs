//! Slot evidence and its classes (skein/docs/design/testing/fakes/drawn-agent.md, 4).

use core::mem::size_of;

use super::{CONFIG, called, history, job, menu, no_scripts, path, reply, sized_batch};
use crate::api::{Answer, Capture, Drawn, Error, Filled, Line, Menu, Rules, Seeds, Slot, Source};
use crate::{Config, Domain};
use skein_lib::bytes::copy_of;

const CONFIGURED: Config = Config { capture_answers: 4, capture_bytes: 4096, ..CONFIG };
const JOB_BODY: &[u8] = br#"{"job":"{{#job}}"}"#;
const JOB_ARGUMENTS: &[u8] = br#"{"job":7}"#;
const CLASS: &[u8] = b"consumed";

fn drawn(config: Config, seed: u64, menu: Menu, replay: Capture) -> Result<Domain, Error> {
    let rules = Rules { cued: Box::new([]), asked: Box::new([]) };
    Domain::drawn(&config, Seeds::One(seed), no_scripts(), menu, rules, replay)
}

fn job_menu() -> Menu {
    menu(JOB_BODY, Box::new([job()]))
}

fn mixed_menu() -> Menu {
    Menu { arguments: Box::new([copy_of(JOB_ARGUMENTS), copy_of(JOB_BODY)]), ..job_menu() }
}

fn job_reply(domain: &mut Domain, config: Config) -> Answer {
    reply(domain, config, history(b"job 7 running")).expect("job reply")
}

fn first_id(answer: &Answer) -> &[u8] {
    called(answer.parts.first().expect("first call")).0
}

fn first_draw(capture: &Capture) -> &Drawn {
    capture.answers.first().expect("captured answer")
}

fn only_slot(capture: &Capture) -> &Filled {
    let slots = &first_draw(capture).slots;
    assert_eq!(slots.len(), 1);
    slots.first().expect("captured slot")
}

#[test]
fn capture_records_only_the_call_that_uses_the_job_slot() {
    let config = Config { calls_per_answer: 2, ..CONFIGURED };
    // Seed 11 draws the literal first, then a slot whose filled arguments are identical.
    let mut domain = drawn(config, 11, mixed_menu(), Capture::none()).expect("provider");
    let answer = job_reply(&mut domain, config);
    assert_eq!(answer.parts.len(), 2);
    for part in &answer.parts {
        assert_eq!(called(part).2, JOB_ARGUMENTS);
    }
    let (id, _, _) = called(answer.parts.get(1).expect("second call"));
    assert_eq!(
        only_slot(&domain.capture()),
        &Filled {
            call: 1,
            id: copy_of(id),
            slot: 0,
            value: copy_of(b"7"),
            source: Source::Conversation,
            class: Box::new([])
        }
    );
}

#[test]
fn capture_records_the_never_issued_draw() {
    let job = Slot { weight: 1000, ..job() };
    let mut domain = drawn(CONFIGURED, 1, menu(JOB_BODY, Box::new([job])), Capture::none()).expect("provider");
    let answer = job_reply(&mut domain, CONFIGURED);
    assert_eq!(super::arguments(&answer), br#"{"job":0}"#);
    assert_eq!(
        only_slot(&domain.capture()),
        &Filled {
            call: 0,
            id: copy_of(first_id(&answer)),
            slot: 0,
            value: copy_of(b"0"),
            source: Source::Unissued,
            class: Box::new([])
        }
    );
}

#[test]
fn capture_records_all_slots_as_never_issued_after_answer_fallback() {
    let (config, menu, request, _, cap) = sized_batch();
    let answer_bytes = cap.checked_sub(1).expect("nonempty batch");
    let config = Config { answer_bytes, calls_per_answer: config.calls_per_answer, ..CONFIGURED };
    let mut domain = drawn(config, 2, menu, Capture::none()).expect("provider");
    let answer = reply(&mut domain, config, request).expect("batch fits at its never-issued values");
    assert_eq!(answer.parts.len(), 2);
    let captured = domain.capture();
    let slots = &first_draw(&captured).slots;
    assert_eq!(slots.len(), 4);
    let expected: [(u32, u32, &[u8]); 4] = [(0, 0, b"-"), (0, 1, b"none"), (1, 0, b"-"), (1, 1, b"none")];
    for (draw, (call, slot, value)) in slots.iter().zip(expected) {
        let part = answer.parts.get(usize::try_from(call).expect("small fixture")).expect("recorded call");
        assert_eq!((draw.call, &*draw.id, draw.slot), (call, called(part).0, slot));
        assert_eq!((&*draw.value, draw.source), (value, Source::Unissued));
    }
}

#[test]
fn classify_keeps_the_worlds_class() {
    let mut domain = drawn(CONFIGURED, 1, job_menu(), Capture::none()).expect("provider");
    let answer = job_reply(&mut domain, CONFIGURED);
    assert!(domain.classify(first_id(&answer), b"job", CLASS));
    assert_eq!(&*only_slot(&domain.capture()).class, CLASS);
}

#[test]
fn classify_refuses_a_missing_id_or_slot_draw() {
    let menu = Menu { slots: Box::new([job(), path()]), ..job_menu() };
    let mut domain = drawn(CONFIGURED, 1, menu, Capture::none()).expect("provider");
    let answer = job_reply(&mut domain, CONFIGURED);
    let before = domain.capture();
    assert!(!domain.classify(b"not a call", b"job", CLASS));
    assert!(!domain.classify(first_id(&answer), b"no such slot", CLASS));
    assert!(!domain.classify(first_id(&answer), b"path", CLASS));
    assert_eq!(domain.capture(), before);
}

#[test]
fn classify_refuses_invalid_labels() {
    let mut domain = drawn(CONFIGURED, 1, job_menu(), Capture::none()).expect("provider");
    let answer = job_reply(&mut domain, CONFIGURED);
    for class in [b"".as_slice(), b"with-dash", b"\xff", &[b'x'; 33]] {
        assert!(!domain.classify(first_id(&answer), b"job", class), "invalid label {class:?}");
    }
    let captured = domain.capture();
    assert!(only_slot(&captured).class.is_empty());
    assert!(!captured.full);
}

fn job_draw_bytes() -> u32 {
    let lengths = [
        size_of::<Drawn>(),
        size_of::<Line>(),
        size_of::<Filled>(),
        b"ls".len(),
        JOB_ARGUMENTS.len(),
        b"call_0000000000000001".len(),
        b"7".len(),
    ];
    let mut bytes = 0;
    for len in lengths {
        let len = u32::try_from(len).expect("small fixture");
        bytes = u32::checked_add(bytes, len).expect("small fixture");
    }
    bytes
}

fn classed_bytes() -> u32 {
    job_draw_bytes().checked_add(u32::try_from(CLASS.len()).expect("small class")).expect("small fixture")
}

#[test]
fn classify_marks_capture_full_when_the_label_does_not_fit() {
    let config = Config { capture_bytes: classed_bytes().checked_sub(1).expect("one byte short"), ..CONFIGURED };
    let mut domain = drawn(config, 1, job_menu(), Capture::none()).expect("provider");
    let answer = job_reply(&mut domain, config);
    assert!(!domain.classify(first_id(&answer), b"job", CLASS));
    let captured = domain.capture();
    assert!(captured.full);
    assert!(only_slot(&captured).class.is_empty());
    assert!(!domain.classify(first_id(&answer), b"job", b"x"), "a full capture keeps no later class");
}

#[test]
fn capture_byte_bound_counts_the_slot_draw_and_its_owned_bytes() {
    let exact = job_draw_bytes();
    for (capture_bytes, full) in [(exact, false), (exact.checked_sub(1).expect("one byte short"), true)] {
        let config = Config { capture_bytes, ..CONFIGURED };
        let mut domain = drawn(config, 1, job_menu(), Capture::none()).expect("provider");
        let answer = job_reply(&mut domain, config);
        assert_eq!(super::arguments(&answer), JOB_ARGUMENTS);
        let captured = domain.capture();
        assert_eq!(captured.full, full);
        if full {
            assert!(captured.answers.is_empty(), "no partial record");
        } else {
            assert_eq!(&*only_slot(&captured).value, b"7");
        }
    }
}

#[test]
fn replay_keeps_slot_draws_under_fresh_call_ids_without_redrawing() {
    let config = Config { calls_per_answer: 2, ..CONFIGURED };
    let mut original = drawn(config, 11, mixed_menu(), Capture::none()).expect("provider");
    let a = history(b"job 7 running");
    let b = history(b"job 9 running");
    let original_a = reply(&mut original, config, a.clone()).expect("answer A");
    let original_b = reply(&mut original, config, b.clone()).expect("answer B");
    let captured = original.capture();
    // Replaying in the other order shifts the ids. With no bodies, nothing can be drawn.
    let menu = Menu { arguments: Box::new([]), ..job_menu() };
    let mut replay = drawn(config, 1, menu, captured.clone()).expect("replay, including unclassified draws");
    let played_b = reply(&mut replay, config, b).expect("replayed B");
    let played_a = reply(&mut replay, config, a).expect("replayed A");
    let kept = replay.capture();
    assert_eq!(kept.answers.len(), 2);
    let replies = [(&original_b, &played_b), (&original_a, &played_a)];
    for ((record, played), (expected, answer)) in captured.answers.iter().rev().zip(&kept.answers).zip(replies) {
        assert_eq!((played.key, &played.answer), (record.key, &record.answer));
        assert_eq!((answer.finish, answer.usage.output), (expected.finish, expected.usage.output));
        assert_eq!(answer.parts.len(), expected.parts.len());
        for (new, old) in answer.parts.iter().zip(&expected.parts) {
            let (id, name, args) = called(new);
            let (old_id, old_name, old_args) = called(old);
            assert_ne!(id, old_id);
            assert_eq!((name, args), (old_name, old_args));
        }
        assert_eq!(played.slots.len(), record.slots.len());
        for (old, new) in record.slots.iter().zip(&played.slots) {
            let call = answer.parts.get(usize::try_from(old.call).expect("small fixture")).expect("slot's call");
            assert_eq!(new, &Filled { id: copy_of(called(call).0), ..old.clone() });
        }
    }
    assert!(!replay.classify(first_id(&played_a), b"job", CLASS), "the identical literal call drew no slot");
    let (id, _, _) = called(played_a.parts.get(1).expect("slotted call"));
    assert!(replay.classify(id, b"job", CLASS));
}

#[test]
fn capture_byte_bound_counts_a_replayed_class() {
    let exact = classed_bytes();
    let config = Config { capture_bytes: exact, ..CONFIGURED };
    let mut original = drawn(config, 1, job_menu(), Capture::none()).expect("provider");
    let answer = job_reply(&mut original, config);
    assert!(original.classify(first_id(&answer), b"job", CLASS));
    let captured = original.capture();
    for (capture_bytes, full) in [(exact, false), (exact.checked_sub(1).expect("one byte short"), true)] {
        let config = Config { capture_bytes, ..CONFIGURED };
        let mut replay = drawn(config, 2, job_menu(), captured.clone()).expect("replay");
        assert_eq!(super::arguments(&job_reply(&mut replay, config)), JOB_ARGUMENTS);
        let kept = replay.capture();
        assert_eq!(kept.full, full);
        if full {
            assert!(kept.answers.is_empty(), "the draw and its class cannot be separated");
        } else {
            assert_eq!(&*only_slot(&kept).class, CLASS);
        }
    }
}

fn one_captured_job() -> Drawn {
    let mut domain = drawn(CONFIGURED, 1, job_menu(), Capture::none()).expect("provider");
    let _answer = job_reply(&mut domain, CONFIGURED);
    first_draw(&domain.capture()).clone()
}

#[test]
fn replay_refuses_an_undeclared_slot_index() {
    let mut record = one_captured_job();
    record.slots.first_mut().expect("job draw").slot = 1;
    let capture = Capture { answers: Box::new([record]), full: false };
    assert_eq!(drawn(CONFIGURED, 1, job_menu(), capture).err(), Some(Error::InvalidRequest));
}

#[test]
fn replay_refuses_a_slot_draw_without_a_call_at_its_position() {
    let record = one_captured_job();
    let mut after_end = record.clone();
    after_end.slots.first_mut().expect("job draw").call = 1;
    let at_text = Drawn { answer: crate::tests::says(b"not a call"), ..record };
    for record in [after_end, at_text] {
        let capture = Capture { answers: Box::new([record]), full: false };
        assert_eq!(drawn(CONFIGURED, 1, job_menu(), capture).err(), Some(Error::InvalidRequest));
    }
}
