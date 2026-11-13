//! A resumed start fitted to its room (jig/docs/design/protocol/channel/README.md, 11).
use super::fixtures::{self, SIZES_LARGE, SIZES_ONE_SLOT, repeated};
use crate::fit::{Fitted, cut_words, fit};
use crate::{self as wire, sizes, v13};
use skein_lib::List;

// The resumed fixture's fixed fields57 + workspace27 + charter14 + transcript24
// + messages60 + grants74 + header8, without any committed calls.
const BASE_FRAME: u32 = 264;
// Each named call: length4 + name6 + length4 + tool6 + arm1 + error/outcome1
// + body length4. A 256-byte host answer cuts to 64 + 3 decimal digits.
const CALL_FIELDS: u32 = 26;
const ANSWER_BYTES: u32 = 256;
const WORD_BYTES: u32 = 67;
const FIVE_CUT: u32 = BASE_FRAME + 5 * (CALL_FIELDS + WORD_BYTES);
const TWO_WHOLE: u32 = FIVE_CUT + 2 * (ANSWER_BYTES - WORD_BYTES);

#[derive(Clone, Copy, Debug)]
enum Answer {
    Host { bytes: u32, error: bool },
    Delivery,
}

const HOST: Answer = Answer::Host { bytes: ANSWER_BYTES, error: false };

fn calls(limits: &v13::Limits, answers: &[Answer]) -> List<wire::AnsweredCall> {
    let mut calls = List::with_capacity(u32::try_from(answers.len()).unwrap());
    for (position, answer) in answers.iter().enumerate() {
        let position = u8::try_from(position).unwrap();
        let answer = match *answer {
            Answer::Host { bytes, error } => wire::SettledAnswer::Host(
                wire::HostAnswer::new(limits, wire::HostAnswerParts { error, body: repeated(position, bytes) })
                    .unwrap(),
            ),
            Answer::Delivery => wire::SettledAnswer::Delivery(
                wire::DeliveryAnswer::new(
                    limits,
                    wire::DeliveryAnswerParts {
                        outcome: wire::DeliveryOutcome::Delivered,
                        evidence: repeated(0xff, ANSWER_BYTES),
                    },
                )
                .unwrap(),
            ),
        };
        let name = repeated(b'a'.checked_add(position).unwrap(), 6);
        let tool = repeated(b'k'.checked_add(position).unwrap(), 6);
        let call = wire::AnsweredCall::new(limits, wire::AnsweredCallParts { name, tool, answer }).unwrap();
        calls.push(call).unwrap();
    }
    calls
}

fn resumed(limits: &v13::Limits, answered: List<wire::AnsweredCall>) -> wire::Assignment {
    let mut parts = fixtures::assignment_resumed().into_parts();
    parts.answered = answered;
    wire::Assignment::new(limits, parts).unwrap()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the fit's rules and compares every returned part")]
fn a_resumed_start_is_fitted_to_its_room() {
    struct Row<'a> {
        name: &'a str,
        answers: &'a [Answer],
        charter: Option<u32>,
        room: u32,
        fitted: Fitted,
        cut: &'a [bool],
        bytes: u32,
    }
    let limits = sizes::limits(&sizes::Sizes { answer: ANSWER_BYTES, answered: 5, ..SIZES_LARGE }).unwrap();
    let rows = [
        Row {
            name: "fits",
            answers: &[HOST; 3],
            charter: None,
            room: BASE_FRAME + 3 * (CALL_FIELDS + ANSWER_BYTES) + 1,
            fitted: Fitted::Whole,
            cut: &[false; 3],
            bytes: BASE_FRAME + 3 * (CALL_FIELDS + ANSWER_BYTES),
        },
        Row {
            name: "past by its calls",
            answers: &[HOST; 5],
            charter: None,
            room: TWO_WHOLE + 1,
            fitted: Fitted::Cut { cut: 3 },
            cut: &[true, true, true, false, false],
            bytes: TWO_WHOLE,
        },
        Row {
            name: "met exactly",
            answers: &[HOST; 5],
            charter: None,
            room: TWO_WHOLE,
            fitted: Fitted::Cut { cut: 3 },
            cut: &[true, true, true, false, false],
            bytes: TWO_WHOLE,
        },
        Row {
            name: "one byte short",
            answers: &[HOST; 5],
            charter: None,
            room: TWO_WHOLE - 1,
            fitted: Fitted::Cut { cut: 4 },
            cut: &[true, true, true, true, false],
            bytes: TWO_WHOLE - (ANSWER_BYTES - WORD_BYTES),
        },
        Row {
            name: "a delivery",
            answers: &[Answer::Delivery, HOST, HOST, HOST, HOST],
            charter: None,
            room: TWO_WHOLE + 1,
            fitted: Fitted::Cut { cut: 3 },
            cut: &[false, true, true, true, false],
            bytes: TWO_WHOLE,
        },
        Row {
            name: "short answers",
            answers: &[Answer::Host { bytes: 66, error: false }, HOST, HOST, HOST, HOST],
            charter: None,
            room: TWO_WHOLE + 1,
            fitted: Fitted::Cut { cut: 2 },
            cut: &[false, true, true, false, false],
            bytes: TWO_WHOLE - 1,
        },
        Row {
            name: "errors",
            answers: &[Answer::Host { bytes: ANSWER_BYTES, error: true }, HOST, HOST, HOST, HOST],
            charter: None,
            room: TWO_WHOLE + 1,
            fitted: Fitted::Cut { cut: 3 },
            cut: &[true, true, true, false, false],
            bytes: TWO_WHOLE,
        },
        Row {
            name: "still past",
            answers: &[HOST; 5],
            charter: Some(2048),
            room: TWO_WHOLE + 1,
            fitted: Fitted::Past { bytes: FIVE_CUT + 2048 - 14 },
            cut: &[true; 5],
            bytes: FIVE_CUT + 2048 - 14,
        },
        Row {
            name: "an older smaller answer stays cut after the first that cannot fit",
            answers: &[Answer::Host { bytes: 128, error: false }, HOST, HOST, HOST, HOST],
            charter: None,
            // After the newest two, 93 bytes remain: an older 128-byte answer
            // could fit, but the intervening 256-byte answer cannot.
            room: TWO_WHOLE + CALL_FIELDS + WORD_BYTES,
            fitted: Fitted::Cut { cut: 3 },
            cut: &[true, true, true, false, false],
            bytes: TWO_WHOLE,
        },
    ];
    let mut failures = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        let mut parts = resumed(&limits, calls(&limits, row.answers)).into_parts();
        if let Some(bytes) = row.charter {
            parts.charter = repeated(b'c', bytes);
        }
        let assignment = wire::Assignment::new(&limits, parts).unwrap();
        let mut expected = assignment.clone().into_parts();
        let mut answered = List::with_capacity(expected.answered.capacity());
        assert_eq!(expected.answered.len(), u32::try_from(row.cut.len()).unwrap(), "{}", row.name);
        for (call, cut) in expected.answered.into_boxed().into_iter().zip(row.cut) {
            let mut parts = call.into_parts();
            if *cut {
                let host = match parts.answer {
                    wire::SettledAnswer::Host(host) => host,
                    wire::SettledAnswer::Delivery(_) => panic!("a delivery is never cut"),
                };
                let mut host = host.into_parts();
                host.body = cut_words(u32::try_from(host.body.len()).unwrap());
                parts.answer = wire::SettledAnswer::Host(wire::HostAnswer::new(&limits, host).unwrap());
            }
            answered.push(wire::AnsweredCall::new(&limits, parts).unwrap()).unwrap();
        }
        expected.answered = answered;
        let (returned, fitted) = fit(assignment, &limits, row.room);
        let bytes = returned.measure() + 8;
        let within_room = match fitted {
            Fitted::Whole | Fitted::Cut { .. } => bytes <= row.room,
            Fitted::Past { bytes: past } => past == bytes && bytes > row.room,
        };
        // Comparing all parts holds names, tools, positions, error flags and
        // every field outside the cut bodies, including workspace and grants.
        let same_parts = returned.into_parts() == expected;
        if fitted != row.fitted || bytes != row.bytes || !within_room || !same_parts {
            failures.push((row.name, fitted, bytes, row.room, same_parts)).unwrap();
        }
    }
    assert!(failures.is_empty(), "rows (name, fitted, bytes, room, same parts): {:?}", failures.as_slice());
}

#[test]
fn the_smallest_resumed_start_is_measured() {
    // Stock core call records, call name, wire tool ceiling and answer bytes
    // (jig/docs/design/protocol/channel/README.md, 10).
    for (count, name, tool, answer, expected) in [(32, 64, 64, 69_632, 6817), (128, 64, 64, 135_168, 27_201)] {
        let limits = sizes::limits(&sizes::Sizes {
            answered: count,
            call_name: name,
            tool_name: tool,
            answer,
            ..SIZES_ONE_SLOT
        })
        .unwrap();
        let mut answered = List::with_capacity(count);
        for _ in 0..count {
            let host = wire::HostAnswer::new(&limits, wire::HostAnswerParts { error: false, body: cut_words(answer) })
                .unwrap();
            let call = wire::AnsweredCall::new(
                &limits,
                wire::AnsweredCallParts {
                    name: repeated(b'n', name),
                    tool: repeated(b't', tool),
                    answer: wire::SettledAnswer::Host(host),
                },
            )
            .unwrap();
            answered.push(call).unwrap();
        }
        let mut parts = fixtures::assignment_resumed().into_parts();
        parts.workspace = None;
        parts.save = false;
        parts.charter = [].into();
        parts.transcript = [].into();
        parts.answered = answered;
        parts.messages = List::with_capacity(0);
        parts.grants = List::with_capacity(0);
        let assignment = wire::Assignment::new(&limits, parts).unwrap();
        assert_eq!(assignment.measure() + 8, expected);
        assert_eq!(sizes::smallest_resumed(count, name, tool, answer), Some(expected));
    }
    assert_eq!(sizes::smallest_resumed(u32::MAX, 64, 64, 1), None);
    assert_eq!(cut_words(135_168).as_ref(), b"[this call ran; its answer of 135168 bytes was left out of this start]");
}
