use super::*;
use jig_client_protocol_wire as w;
use jig_client_shell_web::adapter::Heard;
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
};
use skein_ui::Act;
use std::{fs, path::Path};

fn samples() -> Vec<(Vec<u8>, What)> {
    use Shape::{Bearer, Code, Jwt, Key, RefreshToken};
    [
        (["s", "k", "-", "Ab0123456789_-cdefGHI"].concat(), What::Shape(Key)),
        (["ey", "JabcDEF.Abcdef12.Zyxwvu98"].concat(), What::Shape(Jwt)),
        (["Bear", "er ", "Ab0123456789_+/-="].concat(), What::Shape(Bearer)),
        (["co", "de=", "Ab0123%+"].concat(), What::Shape(Code)),
        (["refresh", "_token", "\":\"", "Ab0123%+"].concat(), What::Shape(RefreshToken)),
        (["refresh", "_token", "=", "Ab0123%+"].concat(), What::Shape(RefreshToken)),
        (["private", "<&\"value>"].concat(), What::Original),
    ]
    .into_iter()
    .map(|(value, what)| (value.into_bytes(), what))
    .collect()
}
fn originals() -> Originals {
    Originals::of(&[], &[["private", "<&\"value>"].concat().into_bytes().into()])
}
fn piece(bytes: &[u8]) -> Line {
    Line::Heard(Heard::Piece { bytes: bytes.into() })
}
fn data(record: &[u8]) -> Vec<u8> {
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(record.len()).unwrap()).unwrap();
    let mut writer = Writer::new(usize::try_from(size).unwrap());
    base64::encode(Alphabet::Standard, record, &mut writer).unwrap();
    [b"data: ".as_slice(), &writer.finish(), b"\n\n"].concat()
}
fn escaped(value: &[u8]) -> String {
    String::from_utf8(value.to_vec())
        .unwrap()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn field_places(value: &[u8]) -> Vec<(&'static str, Line)> {
    vec![
        ("act name", Line::Act { act: Act::Press, event: Box::from([]), name: value.into() }),
        (
            "header value",
            Line::Open {
                piece: 1,
                path: b"/".as_slice().into(),
                headers: vec![(b"header".as_slice().into(), value.into())],
            },
        ),
        (
            "framed body",
            Line::Heard(Heard::Answered {
                exchange: 1,
                status: 200,
                body: [u32::try_from(value.len()).unwrap().to_be_bytes().as_slice(), value].concat().into(),
                headers: Vec::new(),
            }),
        ),
        ("path query", Line::Path([b"/?".as_slice(), value].concat().into())),
        ("place query", Line::Place([b"/?".as_slice(), value].concat().into())),
    ]
}
#[test]
fn each_shape_is_found_in_each_place() {
    let originals = originals();
    let mut failures = 0;
    let mut rows = 0;
    for (value, what) in samples() {
        for (place, line) in field_places(&value) {
            rows += 1;
            if lines(&[line], &originals).unwrap().map(|found| (found.what, found.place)) != Some((what, Place::Field))
            {
                eprintln!("missing {what:?}: {place}");
                failures += 1;
            }
        }
        let event = data(&value);
        for cut in 0..=event.len() {
            rows += 1;
            if lines(&[piece(&event[..cut]), piece(&event[cut..])], &originals)
                .unwrap()
                .map(|found| (found.what, found.place))
                != Some((what, Place::Record))
            {
                eprintln!("missing {what:?}: record pieces at {cut}");
                failures += 1;
            }
        }
        rows += 1;
        if page(&escaped(&value), &originals) != Some(what) {
            eprintln!("missing {what:?}: escaped page");
            failures += 1;
        }
    }
    eprintln!("shape/place rows: {rows}; missing: {failures}");
    assert_eq!(failures, 0, "every shape/place row must detect its kind");
}
fn delivery(value: &[u8], from: u32, writing: bool) -> w::StreamEvent {
    let through = from.checked_add(u32::try_from(value.len()).unwrap()).unwrap();
    let writing_call = writing.then(|| {
        w::WritingCall::new(
            &w::CEILINGS,
            w::WritingCallParts {
                id: b"call".as_slice().into(),
                tool: b"tool".as_slice().into(),
                from,
                through,
                input: value.into(),
                cut: false,
            },
        )
        .unwrap()
    });
    w::StreamEvent::Text(
        w::StreamText::new(
            &w::CEILINGS,
            w::StreamTextParts {
                watch: 1,
                value: w::RunText::new(
                    &w::CEILINGS,
                    w::RunTextParts {
                        attempt: 1,
                        began: None,
                        turn: w::ProvisionalTurnName::new(
                            &w::CEILINGS,
                            w::ProvisionalTurnNameParts { attempt: 1, number: 1, retry: 0 },
                        )
                        .unwrap(),
                        from: if writing { 0 } else { from },
                        through: if writing { 0 } else { through },
                        text: if writing { Box::from([]) } else { value.into() },
                        cut: false,
                        call: None,
                        writing: writing_call,
                    },
                )
                .unwrap(),
            },
        )
        .unwrap(),
    )
}
fn streamed(event: &w::StreamEvent) -> Line {
    let mut writer = Writer::new(usize::try_from(event.measure()).unwrap());
    event.encode(&mut writer).unwrap();
    piece(&data(&writer.finish()))
}
fn split_deliveries(writing: bool) {
    let originals = originals();
    let mut failures = 0;
    let mut rows = 0;
    for (value, what) in samples() {
        for cut in 0..=value.len() {
            let pair = [
                streamed(&delivery(&value[..cut], 0, writing)),
                streamed(&delivery(&value[cut..], u32::try_from(cut).unwrap(), writing)),
            ];
            rows += 1;
            if lines(&pair, &originals).unwrap().map(|found| found.what) != Some(what) {
                eprintln!("missing {what:?}: writing={writing}, cut={cut}");
                failures += 1;
            }
        }
    }
    eprintln!("delivery rows: {rows}; missing: {failures}");
    assert_eq!(failures, 0, "every delivery cut must detect its kind");
}
#[test]
fn a_run_texts_split_token_is_found() {
    split_deliveries(false);
}
#[test]
fn a_writing_calls_split_input_is_found() {
    split_deliveries(true);
}
#[test]
fn middle_deliveries_alone_hold_no_hit() {
    let originals = originals();
    for writing in [false, true] {
        for (value, what) in samples() {
            let cut = value.len() / 2;
            let pair = [
                streamed(&delivery(&value[..cut], 0, writing)),
                streamed(&delivery(&value[cut..], u32::try_from(cut).unwrap(), writing)),
            ];
            for line in &pair {
                assert!(lines(std::slice::from_ref(line), &originals).unwrap().is_none());
            }
            assert_eq!(
                lines(&pair, &originals).unwrap().map(|found| (found.what, found.place)),
                Some((what, Place::RunText))
            );
        }
    }
}
#[test]
fn a_gap_joins_nothing() {
    let originals = originals();
    for writing in [false, true] {
        for (value, _) in samples() {
            let cut = value.len() / 2;
            assert!(
                lines(
                    &[
                        streamed(&delivery(&value[..cut], 0, writing)),
                        streamed(&delivery(&value[cut..], u32::try_from(cut + 1).unwrap(), writing))
                    ],
                    &originals
                )
                .unwrap()
                .is_none()
            );
        }
    }
}
#[test]
fn every_length_prefix_before_a_key() {
    let originals = originals();
    let (key, what) = samples().remove(0);
    let mut failures = 0;
    for byte in 0..=u8::MAX {
        if bytes(&[&[0, 0, 0, byte], key.as_slice()].concat(), &originals) != Some(what) {
            eprintln!("missing key after prefix {byte}");
            failures += 1;
        }
    }
    assert_eq!(failures, 0);
}
#[test]
fn a_51_byte_keys_own_prefix() {
    let key = [["s", "k", "-"].concat().as_bytes(), "A".repeat(48).as_bytes()].concat();
    assert_eq!(bytes(&[b"\0\0\0\x33".as_slice(), &key].concat(), &originals()), Some(What::Shape(Shape::Key)));
}
#[test]
fn these_pass() {
    let originals = originals();
    let mut values = vec![
        "task-pages-and-more-than-twenty-bytes".to_owned(),
        "temper-task-pages-a-long-name".into(),
        "risk-a-long-hyphenated-word-here".into(),
        "code_challenge=abcdefghijkl".into(),
        "status_code=200".into(),
        ["Bear", "er tokens"].concat(),
        ["s", "k", "- short"].concat(),
        ["ey", "J"].concat(),
        ["refresh", "_token is"].concat(),
    ];
    values.extend([
        ["s", "k", "-", &"x".repeat(20)].concat(),
        ["Bear", "er ", &"x".repeat(16)].concat(),
        ["co", "de=", &"x".repeat(8)].concat(),
        ["refresh", "_token=", &"x".repeat(8)].concat(),
        ["refresh", "_token", "\":\"", &"x".repeat(8)].concat(),
        ["xxxxxxxx", ".", "xxxxxxxx", ".", "xxxxxxxx"].concat(),
        "x".repeat(40),
    ]);
    for (row, value) in values.iter().enumerate() {
        assert!(bytes(value.as_bytes(), &originals).is_none(), "safe row {row}");
    }
}
#[test]
fn these_are_found_at_their_edges() {
    let originals = originals();
    let cases = [
        (["s", "k", "-"].concat(), 20, Shape::Key),
        (["Bear", "er "].concat(), 16, Shape::Bearer),
        (["co", "de="].concat(), 8, Shape::Code),
        (["error_co", "de="].concat(), 8, Shape::Code),
        (["device_co", "de="].concat(), 8, Shape::Code),
        (["refresh", "_token="].concat(), 8, Shape::RefreshToken),
    ];
    for (prefix, bound, shape) in cases {
        assert_eq!(bytes(format!("{prefix}{}", "A".repeat(bound)).as_bytes(), &originals), Some(What::Shape(shape)));
        assert!(bytes(format!("{prefix}{}", "A".repeat(bound - 1)).as_bytes(), &originals).is_none());
    }
    for mask in 0..64 {
        let prefix: String = "bearer"
            .chars()
            .enumerate()
            .map(|(i, c)| if mask & (1 << i) == 0 { c } else { c.to_ascii_uppercase() })
            .collect();
        assert_eq!(
            bytes(format!("{prefix} {}", "A".repeat(16)).as_bytes(), &originals),
            Some(What::Shape(Shape::Bearer))
        );
    }
    for lengths in [[8, 8, 8], [7, 8, 8], [8, 7, 8], [8, 8, 7]] {
        let text = ["ey", "J", &"A".repeat(lengths[0] - 3), ".", &"A".repeat(lengths[1]), ".", &"A".repeat(lengths[2])]
            .concat();
        assert_eq!(bytes(text.as_bytes(), &originals).is_some(), lengths == [8, 8, 8]);
    }
}
#[test]
fn found_names_the_line_and_the_record() {
    let originals = originals();
    let (value, what) = samples().remove(0);
    for (line, place, record) in [
        (Line::Path(value.clone().into()), Place::Field, "path"),
        (piece(&data(&value)), Place::Record, "heard"),
        (piece(&[b": ".as_slice(), &value, b"\n"].concat()), Place::Plain, "heard"),
    ] {
        assert_eq!(lines(&[Line::Start, line], &originals).unwrap(), Some(Found { line: 2, record, place, what }));
    }
}
#[test]
fn found_holds_no_bytes() {
    for (value, _) in samples() {
        let found = lines(&[Line::Path(value.clone().into())], &originals()).unwrap().expect("a refusal");
        let debug = format!("{found:?}");
        assert!(!debug.as_bytes().windows(value.len()).any(|part| part == value));
    }
}
#[test]
fn original_forms_and_digest_halves_are_found() {
    let originals = Originals::of(&[], &[[" \tprivate", "<&\"\nvalue>\r "].concat().into_bytes().into()]);
    for original in originals.sought() {
        for form in original.forms() {
            assert_eq!(bytes(form, &originals), Some(What::Original));
        }
        for (digest, placeholder) in original.digests() {
            assert_eq!(bytes(digest, &originals), Some(What::Original));
            assert!(bytes(placeholder, &originals).is_none());
        }
    }
}
#[test]
fn the_committed_journeys_pass() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/client/web/journeys");
    let originals = Originals::of(&[], &[]);
    let mut recordings = 0;
    let mut pages = 0;
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap() == "recording" {
                let recording = fs::read(&path).unwrap();
                let recording = crate::journey::lines(&recording).unwrap();
                assert!(lines(&recording, &originals).unwrap().is_none(), "committed recording");
                recordings += 1;
            } else if path.extension().is_some_and(|ext| ext == "html") {
                assert!(page(&fs::read_to_string(path).unwrap(), &originals).is_none(), "committed page");
                pages += 1;
            }
        }
    }
    assert!(recordings > 0 && pages > 0);
    eprintln!("scanned {recordings} recordings and {pages} printed pages");
}

#[test]
fn each_shape_is_found_in_plain_stream_text() {
    let originals = originals();
    let mut missing = 0;
    for (value, what) in samples() {
        let comment = [b": ".as_slice(), &value, b"\n"].concat();
        for cut in 0..=comment.len() {
            if lines(&[piece(&comment[..cut]), piece(&comment[cut..])], &originals)
                .unwrap()
                .map(|found| (found.what, found.place))
                != Some((what, Place::Plain))
            {
                eprintln!("missing {what:?}: plain pieces at {cut}");
                missing += 1;
            }
        }
    }
    assert_eq!(missing, 0, "plain stream text is scanned after joining pieces");
}
fn parts(event: w::StreamEvent) -> w::StreamTextParts {
    match event {
        w::StreamEvent::Text(text) => text.into_parts(),
        w::StreamEvent::Hello(_) | w::StreamEvent::Delivery(_) | w::StreamEvent::Ended(_) => {
            unreachable!("text fixture")
        }
    }
}
fn from_parts(parts: w::StreamTextParts) -> Line {
    streamed(&w::StreamEvent::Text(w::StreamText::new(&w::CEILINGS, parts).unwrap()))
}
fn with_run(parts: &w::StreamTextParts, run: w::RunTextParts) -> w::StreamTextParts {
    w::StreamTextParts { watch: parts.watch, value: w::RunText::new(&w::CEILINGS, run).unwrap() }
}
#[test]
fn distinct_runs_and_calls_never_join() {
    let originals = originals();
    for writing in [false, true] {
        for (value, _) in samples() {
            let cut = value.len() / 2;
            let first = streamed(&delivery(&value[..cut], 0, writing));
            let second = parts(delivery(&value[cut..], u32::try_from(cut).unwrap(), writing));
            let mut changed = Vec::new();
            let mut watch = second.clone();
            watch.watch += 1;
            changed.push(("watch", watch));
            let run = second.value.clone().into_parts();
            let mut attempt = run.clone();
            attempt.attempt += 1;
            changed.push(("attempt", with_run(&second, attempt)));
            let turn = run.turn.clone().into_parts();
            let mut turn_attempt = turn.clone();
            turn_attempt.attempt += 1;
            let mut turn_number = turn.clone();
            turn_number.number += 1;
            let mut turn_retry = turn;
            turn_retry.retry += 1;
            for (name, turn) in
                [("turn attempt", turn_attempt), ("turn number", turn_number), ("turn retry", turn_retry)]
            {
                let mut turn_run = run.clone();
                turn_run.turn = w::ProvisionalTurnName::new(&w::CEILINGS, turn).unwrap();
                changed.push((name, with_run(&second, turn_run)));
            }
            if let Some(call) = &run.writing {
                let mut call = call.clone().into_parts();
                call.id = b"other".as_slice().into();
                let mut call_run = run.clone();
                call_run.writing = Some(w::WritingCall::new(&w::CEILINGS, call).unwrap());
                changed.push(("call id", with_run(&second, call_run)));
            }
            for (name, second) in changed {
                assert!(
                    lines(&[first.clone(), from_parts(second)], &originals).unwrap().is_none(),
                    "different {name}, writing={writing}"
                );
            }
            assert!(
                lines(&[first, streamed(&delivery(&value[cut..], u32::try_from(cut).unwrap(), !writing))], &originals)
                    .unwrap()
                    .is_none(),
                "run text and call input are separate"
            );
        }
    }
}
#[test]
fn touching_overlapping_and_reordered_deliveries_join() {
    let originals = originals();
    let mut missing = 0;
    for writing in [false, true] {
        for (value, what) in samples() {
            let cut = value.len() / 2;
            for overlap in [0, 1, cut] {
                let first = streamed(&delivery(&value[..cut], 7, writing));
                let second =
                    streamed(&delivery(&value[cut - overlap..], u32::try_from(7 + cut - overlap).unwrap(), writing));
                for pair in [[first.clone(), second.clone()], [second.clone(), first.clone()]] {
                    if lines(&pair, &originals).unwrap().map(|found| found.what) != Some(what) {
                        eprintln!("missing {what:?}: overlapping/reordered writing={writing}, overlap={overlap}");
                        missing += 1;
                    }
                }
            }
        }
    }
    assert_eq!(missing, 0);
}
#[test]
fn malformed_offsets_join_nothing() {
    let originals = originals();
    for writing in [false, true] {
        for (value, _) in samples() {
            let cut = value.len() / 2;
            let first = streamed(&delivery(&value[..cut], 0, writing));
            let second = parts(delivery(&value[cut..], u32::try_from(cut).unwrap(), writing));
            for (from, through) in [(u32::try_from(cut).unwrap(), u32::MAX), (u32::MAX, 0)] {
                let mut run = second.value.clone().into_parts();
                if writing {
                    let mut call = run.writing.take().unwrap().into_parts();
                    call.from = from;
                    call.through = through;
                    run.writing = Some(w::WritingCall::new(&w::CEILINGS, call).unwrap());
                } else {
                    run.from = from;
                    run.through = through;
                }
                assert!(lines(&[first.clone(), from_parts(with_run(&second, run))], &originals).unwrap().is_none());
            }
        }
    }
}
#[test]
fn invalid_records_join_nothing_and_are_still_scanned() {
    let originals = originals();
    for writing in [false, true] {
        for (value, what) in samples() {
            let cut = value.len() / 2;
            let first = streamed(&delivery(&value[..cut], 0, writing));
            let event = delivery(&value[cut..], u32::try_from(cut).unwrap(), writing);
            let mut writer = Writer::new(usize::try_from(event.measure()).unwrap());
            event.encode(&mut writer).unwrap();
            let mut record = writer.finish().into_vec();
            record.push(0);
            assert!(
                lines(&[first, piece(&data(&record))], &originals).unwrap().is_none(),
                "trailing record joins nothing"
            );
            assert_eq!(lines(&[piece(&data(&value))], &originals).unwrap().map(|found| found.what), Some(what));
        }
    }
}
#[test]
fn undecodable_base64_is_refused() {
    assert_eq!(lines(&[Line::Start, piece(b"data: !\n")], &originals()), Err(Undecoded { line: 2 }));
}
#[test]
fn field_order_precedes_streams_and_joins() {
    let originals = originals();
    let (value, what) = samples().remove(0);
    assert_eq!(
        lines(&[piece(&data(&value)), Line::Path(value.into())], &originals).unwrap(),
        Some(Found { line: 2, record: "path", place: Place::Field, what })
    );
}
#[test]
fn a_pages_escapes_are_decoded_once() {
    let value = ["co", "de=", "Abcdefg", "&"].concat();
    let originals = Originals::of(&[], &[value.into_bytes().into()]);
    assert_eq!(page("co&#100;e=Abcdefg&amp;", &originals), None);
    let value = ["private", "<&\">"].concat();
    let triple = escaped(escaped(escaped(value.as_bytes()).as_bytes()).as_bytes());
    let originals = Originals::of(&[], &[value.into_bytes().into()]);
    assert_eq!(page(&triple, &originals), None);
    assert_eq!(page("private&lt;&amp;&quot;&gt;", &originals), Some(What::Original));
    assert_eq!(page("private&amp;lt;&amp;amp;&amp;quot;&amp;gt;", &originals), Some(What::Original));
}
#[test]
fn refresh_separators_and_code_boundaries_hold() {
    let originals = originals();
    for separators in ["", "=", "\":\"", "\\\": "] {
        let text = ["refresh", "_token", separators, "Abcdef12"].concat();
        assert_eq!(bytes(text.as_bytes(), &originals), Some(What::Shape(Shape::RefreshToken)));
    }
    let text = ["refresh", "_token", "     ", "Abcdef12"].concat();
    assert!(bytes(text.as_bytes(), &originals).is_none());
    for prefix in ["a", "Z", "0", "9"] {
        assert!(bytes([prefix, "co", "de=Abcdef12"].concat().as_bytes(), &originals).is_none());
    }
}

#[test]
fn stream_records_and_plain_text_keep_line_order() {
    let originals = originals();
    let (value, what) = samples().remove(0);
    let plain = piece(&[b": ".as_slice(), &value, b"\n"].concat());
    let record = piece(&data(&value));
    for pair in [[plain.clone(), record.clone()], [record, plain]] {
        assert_eq!(lines(&pair, &originals).unwrap().map(|found| (found.line, found.what)), Some((1, what)));
    }
}

#[test]
fn candidate_originals_and_shapes_are_scanned_separately() {
    use crate::journey::Kind;
    use skein_lib::sha256::digest;

    let value = [b"a/ & ".as_slice(), b"\"<>\\\n"].concat();
    for kind in [Kind::Approval, Kind::Credential, Kind::Bearer] {
        let originals = Originals::new(vec![(kind, value.clone().into())], &[]);
        for form in [
            value.as_slice(),
            b"a%2F%20%26%20%22%3C%3E%5C%0A",
            b"a%2f%20%26%20%22%3c%3e%5c%0a",
            b"a/ &amp; &quot;&lt;&gt;\\\n",
            b"a/ & \\\"<>\\\\\\n",
            &digest(&value),
        ] {
            let field = [b"before ".as_slice(), form, b" after"].concat();
            assert!(crate::journey::original(&field, &originals), "{kind:?}");
            assert_eq!(bytes(&field, &originals), Some(What::Original), "{kind:?}");
            assert_eq!(crate::journey::shape(&field), None, "{kind:?} has no shape");
        }
        assert!(!original(&vec![b'x'; value.len()], &originals), "{kind:?} placeholder");
        assert!(!original(&digest(&vec![b'x'; value.len()]), &originals), "{kind:?} placeholder digest");
    }
    for (value, what) in samples() {
        if let What::Shape(expected) = what {
            let originals = Originals::new(vec![(Kind::Approval, value.clone().into())], &[]);
            assert!(original(&value, &originals));
            assert_eq!(shape(&value), Some(expected));
            assert_eq!(bytes(&value, &originals), Some(What::Original), "original precedes shape {expected:?}");
            assert_eq!(bytes(&value, &Originals::new(Vec::new(), &[])), Some(what));
        }
    }
}
