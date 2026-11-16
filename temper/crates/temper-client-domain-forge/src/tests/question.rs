use crate::question::{Choice, Parsed, parse};
use skein_lib::{Writer, bytes};

const EXAMPLE: &str = "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local.\n\nSaw: The box test reproduces it every time.\nChoice: In jig’s store (recommended)\nChoice: In temper serve";
const QUESTION: &str = "Fix the dropped watch in jig’s store, or keep it alive in temper serve?";
const CONTEXT: &str =
    "MakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local.";

fn choice(words: &[u8], recommended: bool) -> Choice {
    Choice { words: words.into(), recommended }
}

fn fallback(words: &[u8], question: &[u8], context: &[u8]) {
    let parsed = parse(words, false);
    assert_eq!(
        parsed,
        Parsed {
            question: question.into(),
            context: context.into(),
            saw: None,
            choices: Box::from([]),
            findings: Box::from([]),
            group: None
        }
    );
    assert_eq!(parsed.primary(), None);
}

#[test]
fn the_design_example_has_question_context_saw_and_choices() {
    let parsed = parse(EXAMPLE.as_bytes(), false);
    assert_eq!(
        parsed,
        Parsed {
            question: QUESTION.as_bytes().into(),
            context: CONTEXT.as_bytes().into(),
            saw: Some(Box::from(b"The box test reproduces it every time.".as_slice())),
            choices: Box::from([choice("In jig’s store".as_bytes(), true), choice(b"In temper serve", false)]),
            findings: Box::from([]),
            group: None,
        }
    );
    assert_eq!(parsed.primary(), Some(0));
}

#[test]
fn ordinary_final_words_leave_choice_lines_in_the_context() {
    fallback(
        b"Question?\n\nContext.\nChoice: A\nOrdinary words.",
        b"Question?",
        b"Context.\nChoice: A\nOrdinary words.",
    );
}

#[test]
fn choice_lines_above_the_trailing_block_stay_context() {
    let parsed = parse(b"Question?\n\nChoice: Context\nOrdinary words.\n\nChoice: Answer", false);
    assert_eq!(&*parsed.context, b"Choice: Context\nOrdinary words.");
    assert_eq!(&*parsed.choices, &[choice(b"Answer", false)]);
}

#[test]
fn blank_lines_among_and_after_trailing_lines_are_skipped() {
    let parsed =
        parse(b"Question? \r\n\t\r\nContext. \r\n\nSaw: Evidence \r\n\nChoice:  A \t\r\n \t\r\nChoice: B\n\n", false);
    assert_eq!(&*parsed.question, b"Question?");
    assert_eq!(&*parsed.context, b"Context.");
    assert_eq!(parsed.saw.as_deref(), Some(b"Evidence".as_slice()));
    assert_eq!(&*parsed.choices, &[choice(b"A", false), choice(b"B", false)]);
}

#[test]
fn unmarked_choices_draw_the_first_one_primary() {
    let parsed = parse(b"Question?\nChoice: A\nChoice: B", false);
    assert_eq!(&*parsed.choices, &[choice(b"A", false), choice(b"B", false)]);
    assert_eq!(parsed.primary(), Some(0));
}

#[test]
fn a_second_recommended_choice_is_primary() {
    let parsed = parse(b"Question?\nChoice: A\nChoice: B (recommended)", false);
    assert_eq!(&*parsed.choices, &[choice(b"A", false), choice(b"B", true)]);
    assert_eq!(parsed.primary(), Some(1));
}

#[test]
fn only_the_first_mark_recommends_but_both_endings_disappear() {
    let parsed = parse(b"Question?\nChoice: A (recommended)\nChoice: B (recommended)", false);
    assert_eq!(&*parsed.choices, &[choice(b"A", true), choice(b"B", false)]);
    assert_eq!(parsed.primary(), Some(0));
}

#[test]
fn saw_without_choices_falls_back_and_stays_context() {
    fallback(b"Question?\n\nContext.\nSaw: Evidence", b"Question?", b"Context.\nSaw: Evidence");
}

#[test]
fn several_saw_lines_join_in_order_among_choices() {
    for (words, expected) in [
        (b"Question?\nSaw: First\nChoice: A\nSaw: Second\nChoice: B".as_slice(), b"First\nSecond".as_slice()),
        (b"Question?\nSaw: \nChoice: A\nSaw: Second\nSaw: ", b"\nSecond\n"),
    ] {
        let parsed = parse(words, false);
        assert_eq!(parsed.saw.as_deref(), Some(expected));
    }
}

#[test]
fn a_cut_read_uses_the_whole_example_as_fallback() {
    let parsed = parse(EXAMPLE.as_bytes(), true);
    let expected = "MakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local.\n\nSaw: The box test reproduces it every time.\nChoice: In jig’s store (recommended)\nChoice: In temper serve";
    assert_eq!(
        parsed,
        Parsed {
            question: QUESTION.as_bytes().into(),
            context: expected.as_bytes().into(),
            saw: None,
            choices: Box::from([]),
            findings: Box::from([]),
            group: None,
        }
    );
    assert_eq!(parsed.primary(), None);
}

#[test]
fn fallback_splits_the_first_paragraph_from_the_rest() {
    fallback(
        b"First line.\nSecond line.\n\nContext.\n\nMore context.\n\n",
        b"First line.\nSecond line.",
        b"Context.\n\nMore context.",
    );
}

#[test]
fn one_paragraph_is_all_question_in_the_fallback() {
    fallback(b"First line.\nSecond line.", b"First line.\nSecond line.", b"");
}

#[test]
fn the_first_line_is_never_a_trailing_choice() {
    fallback(b"Choice: A", b"Choice: A", b"");
}

#[test]
fn empty_and_oversized_choices_leave_no_choices_and_fall_back() {
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).unwrap();
    let oversized = bytes::zeroed(bound + 1);
    for invalid in [b"".as_slice(), b" \t", b" (recommended)", &oversized] {
        let words = bytes::concat(&[b"Question?\n\nChoice: ", invalid]);
        let context = words.strip_prefix(b"Question?\n\n").unwrap().trim_ascii_end();
        fallback(&words, b"Question?", context);
    }
}

#[test]
fn choices_at_the_answer_bound_are_accepted_after_trimming() {
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).unwrap();
    let mut answer = bytes::zeroed(bound);
    answer.fill(b'A');
    let words = bytes::concat(&[b"Question?\nChoice:  \t", &answer, b" (recommended) \r\nChoice: \nChoice: B"]);
    let parsed = parse(&words, false);
    assert_eq!(&*parsed.choices, &[choice(&answer, true), choice(b"B", false)]);
}

#[test]
fn choices_stop_at_sixteen() {
    for count in [16, 17] {
        let header = b"Question?\n\nContext.\n";
        let line = b"Choice: A\n";
        let mut words = Writer::new(header.len() + line.len() * count);
        words.put(header).unwrap();
        for _ in 0..count {
            words.put(line).unwrap();
        }
        let parsed = parse(&words.finish(), false);
        assert_eq!(parsed.choices.len(), if count == 16 { 16 } else { 0 });
        assert_eq!(parsed.saw, None);
        if count == 17 {
            assert!(parsed.context.ends_with(b"Choice: A"));
        }
    }
}

#[test]
fn empty_words_have_empty_fields() {
    fallback(b"", b"", b"");
}

#[test]
fn every_byte_prefix_parses_even_inside_utf8_characters() {
    let words = EXAMPLE.as_bytes();
    for end in 0..=words.len() {
        let prefix = &words[..end];
        for cut in [false, true] {
            let parsed = parse(prefix, cut);
            assert!(parsed.question.len() <= prefix.len());
            assert!(parsed.context.len() <= prefix.len());
        }
    }
    let parsed = parse(b"Q\xff?\nChoice: A\xfe", false);
    assert_eq!(&*parsed.question, b"Q\xff?");
    assert_eq!(&*parsed.choices, &[choice(b"A\xfe", false)]);
}

#[test]
fn in_words() {
    use super::fixtures::acts as f;
    use jig_client::contract as c;
    let part = c::Part { connector: 1, kind: c::PartKind::Result, key: c::PartKey::Number(1) };
    for address in [c::PersonAddress::Party(73), c::PersonAddress::Role(0)] {
        for (contract, report) in [
            (c::TaskContractDocument::Report { words: 4096 }, true),
            (c::TaskContractDocument::Verdict { choices: skein_lib::List::uncapped(), cut: false }, false),
            (c::TaskContractDocument::Approval { state: Box::from(*b"Ready"), part: part.clone() }, false),
            (c::TaskContractDocument::Change { connector: 1, kind: 1, words: 32 }, false),
            (c::TaskContractDocument::Job { bytes: 32 }, false),
        ] {
            let c::ReadValue::PersonTask(mut task) = f::person(c::ActKind::Answer) else { panic!("person fixture") };
            task.address = address;
            task.contract = contract;
            assert_eq!(
                crate::question::in_words(&task),
                report && address == c::PersonAddress::Party(73),
                "{address:?}: {:?}",
                task.contract
            );
        }
    }
}

#[test]
fn findings_join_in_first_order_and_leave_context_and_choices() {
    use crate::forge::findings::Id;
    let parsed = parse("Three bugs in temper. Start a goal for each?\n\nNone blocks a landing.\n\nFindings: 2/2 2/8\nFindings: 2/10 2/2\nGroup: bugs · in temper\nChoice: A goal for each (recommended)\nChoice: Backlog them".as_bytes(), false);
    assert_eq!(&*parsed.question, b"Three bugs in temper. Start a goal for each?");
    assert_eq!(&*parsed.context, b"None blocks a landing.");
    assert_eq!(
        &*parsed.findings,
        &[Id { project: 2, number: 2 }, Id { project: 2, number: 8 }, Id { project: 2, number: 10 }]
    );
    assert_eq!(
        parsed.group,
        Some(crate::question::Group { noun: b"bugs".as_slice().into(), phrase: b"in temper".as_slice().into() })
    );
    assert_eq!(&*parsed.choices, &[choice(b"A goal for each", true), choice(b"Backlog them", false)]);
}

#[test]
fn malformed_findings_invalidate_the_list_not_the_question() {
    for token in [
        b"2/x".as_slice(),
        b"2/",
        b"/3",
        b"+2/3",
        b"2/+3",
        b"2/3/4",
        b"4294967296/3",
        b"2/4294967296",
        b"2/3\t2/4",
        b"\xff/3",
    ] {
        let words = bytes::concat(&[
            b"Q?\n\nContext\nFindings: 2/2 ",
            token,
            b"\nFindings: 2/8\nGroup: bugs\nSaw: Evidence\nChoice: A",
        ]);
        let parsed = parse(&words, false);
        assert!(parsed.findings.is_empty(), "{token:?}");
        assert_eq!(&*parsed.context, b"Context");
        assert_eq!(&*parsed.group.unwrap().noun, b"bugs");
        assert_eq!(parsed.saw.as_deref(), Some(b"Evidence".as_slice()));
        assert_eq!(&*parsed.choices, &[choice(b"A", false)]);
    }
    assert!(parse(b"Q?\nFindings:  \nChoice: A", false).findings.is_empty());
}

#[test]
fn distinct_findings_stop_at_sixteen() {
    use crate::forge::findings::Id;
    for count in [16_u32, 17] {
        let header = b"Q?\nFindings: ";
        let tail = b"\nFindings: 02/002\nChoice: A";
        let mut size = header.len() + tail.len();
        for number in 1..=count {
            size += 3 + skein_lib::Decimal::of(u64::from(number)).as_bytes().len();
        }
        let mut words = Writer::new(size);
        words.put(header).unwrap();
        for number in 1..=count {
            words.put(b"2/").unwrap();
            words.put(skein_lib::Decimal::of(u64::from(number)).as_bytes()).unwrap();
            words.put(b" ").unwrap();
        }
        words.put(tail).unwrap();
        let parsed = parse(&words.finish(), false);
        assert_eq!(parsed.findings.len(), if count == 16 { 16 } else { 0 });
        if count == 16 {
            assert_eq!(parsed.findings.first(), Some(&Id { project: 2, number: 1 }));
            assert_eq!(parsed.findings.last(), Some(&Id { project: 2, number: 16 }));
        }
        assert_eq!(&*parsed.choices, &[choice(b"A", false)]);
    }
    let parsed = parse(b"Q?\nFindings: 0/0 4294967295/4294967295\nChoice: A", false);
    assert_eq!(&*parsed.findings, &[Id { project: 0, number: 0 }, Id { project: u32::MAX, number: u32::MAX }]);
}

#[test]
fn group_uses_first_line_and_bounds_the_trimmed_label() {
    let bound = [b'a'; 256];
    let beyond = [b'a'; 257];
    for (label, expected) in [
        (" bugs · in temper ".as_bytes(), Some((b"bugs".as_slice(), b"in temper".as_slice()))),
        (b" bugs ", Some((b"bugs".as_slice(), b"".as_slice()))),
        (bound.as_slice(), Some((bound.as_slice(), b"".as_slice()))),
        (beyond.as_slice(), None),
        (b" ", None),
        (" · in temper".as_bytes(), None),
    ] {
        let words = bytes::concat(&[b"Q?\nGroup: ", label, "\nGroup: later · ignored\nChoice: A".as_bytes()]);
        let parsed = parse(&words, false);
        match expected {
            Some((noun, phrase)) => {
                let group = parsed.group.expect("valid label");
                assert_eq!(&*group.noun, noun);
                assert_eq!(&*group.phrase, phrase);
            }
            None => assert_eq!(parsed.group, None, "{label:?}"),
        }
    }
}

#[test]
fn findings_above_trailing_block_are_context() {
    let parsed = parse(b"Q?\n\nFindings: 2/2\nGroup: context\nOrdinary words.\n\nChoice: A", false);
    assert!(parsed.findings.is_empty());
    assert_eq!(parsed.group, None);
    assert_eq!(&*parsed.context, b"Findings: 2/2\nGroup: context\nOrdinary words.");
    assert_eq!(&*parsed.choices, &[choice(b"A", false)]);
}

#[test]
fn fallback_has_no_findings_or_group() {
    for (choice, cut) in [(b"".as_slice(), false), (b"\nChoice: A".as_slice(), true)] {
        let context = bytes::concat(&[b"Context\nFindings: 2/2\nGroup: bugs", choice]);
        let words = bytes::concat(&[b"Q?\n\n", &context]);
        let parsed = parse(&words, cut);
        assert!(parsed.findings.is_empty());
        assert_eq!(parsed.group, None);
        assert_eq!(&*parsed.question, b"Q?");
        assert_eq!(parsed.context, context);
    }
}

#[test]
fn first_line_never_names_findings_or_group() {
    for first in [b"Findings: 2/2".as_slice(), b"Group: bugs"] {
        let words = bytes::concat(&[first, b"\nChoice: A"]);
        let parsed = parse(&words, false);
        assert_eq!(parsed.question.as_ref(), first);
        assert!(parsed.findings.is_empty());
        assert_eq!(parsed.group, None);
    }
}
