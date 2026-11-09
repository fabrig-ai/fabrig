use crate::tests::fixtures;
use jig_client::{
    Readout,
    contract::{ReadNone, ReadResult, ReadValue},
};
#[test]
fn a_turn_past_the_read_bound_exits_0() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(fixtures::TURN_CALL);
    root.serve(call, fixtures::turn_too_large(fixtures::TOO_LARGE));
    let printed = root.answer(call).expect("sized answer");
    assert_eq!((printed.status, printed.stdout), (0, fixtures::golden("turn_limit")));
}
#[test]
fn a_large_page_turn_prints_its_read_call() {
    let printed = fixtures::read_answer(fixtures::ACTIVITY_CALL, fixtures::activity_too_large());
    assert_eq!(printed.stdout, fixtures::appended(
        b"brief: task text 41 --field specification --from 0\ninstructions: task text 41 --field instructions --from 0\n",
        &fixtures::golden("activity_turn")));
}
#[test]
fn a_page_turn_that_fits_prints_whole() {
    let ReadResult::Value(ReadValue::Turn(turn)) = fixtures::turn_call(None) else { panic!("decoded turn") };
    let mut rows = skein_lib::List::with_capacity(fixtures::PAGE_ROWS);
    rows.push(crate::tests::documents::turn(1, turn.at, turn.name, turn.turn)).expect("one decoded turn row");
    let printed = fixtures::read_answer(
        fixtures::ACTIVITY_CALL,
        ReadResult::Value(ReadValue::Activity(Box::new(jig_client::contract::ActivityPage {
            cut: false,
            rows,
            before: None,
            after: None,
        }))),
    );
    assert_eq!(printed.status, 0);
    assert_eq!(
        printed.stdout.as_ref(),
        b"brief: task text 41 --field specification --from 0\ninstructions: task text 41 --field instructions --from 0\nturn 3.7\ncall 2 \xc2\xb7 read\nrequest\nreply\nresult 2 \xc2\xb7 failed\nsummary\nno more\n"
    );
    assert!(skein_lib::bytes::find(&printed.stdout, b"read with:").is_none());
}
#[test]
fn a_turn_part_is_cut_with_its_continuation() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_text(fixtures::LONG_PART));
    fixtures::check_golden("turn_part", &printed.stdout);
}
#[test]
fn a_turn_part_at_the_bound_is_complete() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_text(fixtures::PART_SIZE));
    fixtures::check_golden("turn_part_complete", &printed.stdout);
}
#[test]
fn a_turn_continuation_starts_at_its_byte_offset() {
    let printed = fixtures::read_answer(fixtures::TURN_NEXT_CALL, fixtures::turn_text(fixtures::LONG_PART));
    fixtures::check_golden("turn_part_next", &printed.stdout);
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the step table inspects rendered UTF-8 lines")]
fn a_part_read_says_its_cut_first() {
    let mut failures = skein_lib::List::with_capacity(5); // Five part/control rows (tools.md, 11.1).
    for (argv, turn, first, last, uncut) in [
        (
            fixtures::TURN_PART_CALL,
            fixtures::turn_text(fixtures::LONG_PART),
            "part 1 page 1 of 2 holds 65536 bytes; 1 bytes left; read on with: task turn 41 3.7 --part 1 --from 65536",
            Some("1 bytes left; read on with: task turn 41 3.7 --part 1 --from 65536"),
            false,
        ),
        (
            fixtures::TURN_NEXT_CALL,
            fixtures::turn_text(fixtures::LONG_PART),
            "part 1 page 2 of 2 holds 1 bytes; no more",
            Some("no more"),
            false,
        ),
        (fixtures::TURN_PART_CALL, fixtures::turn_text(fixtures::PART_SIZE), "turn 3.7", None, true),
        (
            fixtures::TURN_OUTCOME_CALL,
            fixtures::turn_call(None),
            "part 2 page 2 of 2 holds 3 bytes; no more",
            Some("no more"),
            false,
        ),
        (fixtures::TURN_CALL, fixtures::turn_text(fixtures::LONG_PART), "turn 3.7", None, true),
    ] {
        let printed = fixtures::read_answer(argv, turn);
        let stdout = core::str::from_utf8(&printed.stdout).unwrap();
        let last_matches = match last {
            Some(line) => stdout.lines().last() == Some(line),
            None => true,
        };
        let correct = printed.status == 0
            && stdout.lines().next() == Some(first)
            && last_matches
            && (!uncut || !stdout.contains(" page "));
        if !correct {
            failures.push((argv, Box::<str>::from(stdout.lines().next().unwrap_or("")))).unwrap();
        }
    }
    assert!(failures.is_empty(), "cut part positions: {failures:?}");
}

#[test]
fn client_document_limits_exit_1() {
    for readout in [Readout::Limit, Readout::Jig(ReadResult::None(ReadNone::TooLarge { bytes: fixtures::TOO_LARGE }))] {
        let (mut root, call) = fixtures::read_call(fixtures::TASK_CALL);
        root.serve_readout(call, readout);
        assert_eq!(root.answer(call).expect("limit answered").status, 1);
    }
}
#[test]
fn chat_prints_only_the_hosts_provisional_text() {
    let (mut root, call) = fixtures::read_call(fixtures::CHAT_CALL);
    root.serve(call, fixtures::activity_too_large());
    root.deliver_run(fixtures::no_provisional());
    assert_eq!(root.answer(call).expect("chat answered").stdout, fixtures::golden("activity_turn"));
}

#[test]
fn a_run_closes_its_watch_at_its_first_delivery() {
    let (mut root, call) = fixtures::read_call(fixtures::RUN_CALL);
    root.deliver_run(fixtures::no_provisional());
    assert_eq!(root.closed.len(), fixtures::PAGE_ROWS);
    let _printed = root.answer(call).expect("run answered");
}

#[test]
fn a_turn_prints_its_supplied_blocks() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_call(None));
    assert_eq!(printed.stdout, fixtures::golden("turn_call"));
}
#[test]
fn a_path_is_never_cut() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_call(Some(fixtures::long_path())));
    fixtures::check_golden("turn_path", &printed.stdout);
}
#[test]
fn a_json_turn_part_carries_its_continuation() {
    let printed = fixtures::read_answer(fixtures::TURN_JSON_CALL, fixtures::turn_text(fixtures::LONG_PART));
    fixtures::check_golden("turn_part_json", &printed.stdout);
}

#[test]
fn run_prints_the_hosts_provisional_text() {
    let (mut root, call) = fixtures::read_call(fixtures::RUN_CALL);
    root.deliver_run(fixtures::provisional());
    assert_eq!(root.answer(call).expect("run answered").stdout, fixtures::golden("run_provisional"));
}

#[test]
fn a_document_sized_turn_prints_escaped_tool_names_with_the_shipped_limits() {
    use crate::limits;
    use jig_client::contract as c;
    use skein_lib::List;

    // Leave 256 bytes per call for its source framing. Each retained tool
    // name fits CLIENT_TEXT; together their six-byte NUL escapes pass ask_bytes.
    let tool_bytes = limits::CLIENT_TEXT.checked_sub(256).expect("room for call framing");
    let mut blocks = List::with_capacity(limits::ROWS);
    for number in 0..limits::ROWS {
        blocks
            .push(c::Block::Call(c::Call {
                number: u64::from(number),
                tool: c::Text::new(fixtures::maximally_escaped_json_text(tool_bytes), limits::CLIENT_TEXT)
                    .expect("tool within shipped text bound"),
                input: c::Text::new(Box::default(), limits::CLIENT_TEXT).expect("empty input"),
                outcome: None,
                failed: false,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("calls within shipped block bound");
    }
    let printed = fixtures::read_answer(
        &["task", "turn", fixtures::TASK_CALL[2], "3.7", "--json"],
        fixtures::turn_blocks(blocks),
    );

    assert_eq!(printed.status, 0, "{:?}", printed.stderr);
    let escaped = tool_bytes.checked_mul(limits::ROWS).expect("bounded document");
    assert_eq!(fixtures::occurrences(&printed.stdout, br"\u0000"), usize::try_from(escaped).expect("count fits"));
}

#[test]
fn supplied_system_words_print_in_text_and_json() {
    use jig_client::contract as c;
    let supplied = b"Selected instructions and supplied brief";
    for (call, json) in [(fixtures::TURN_CALL, false), (fixtures::TURN_JSON_CALL, true)] {
        let mut blocks = skein_lib::List::with_capacity(1);
        blocks
            .push(c::Block::Text(c::Text::new(supplied.as_slice().into(), 128).expect("supplied words")))
            .expect("block");
        let mut value = fixtures::turn_blocks(blocks);
        let ReadResult::Value(ReadValue::Turn(turn)) = &mut value else { panic!("archived turn") };
        let c::TurnRead::Decoded(turn) = &mut turn.turn else { panic!("decoded turn") };
        turn.messages.get_mut(0).expect("supplied message").speaker = c::Speaker::System;
        let printed = fixtures::read_answer(call, value);
        assert_eq!(printed.status, 0);
        assert!(skein_lib::bytes::find(&printed.stdout, supplied).is_some());
        if json {
            assert!(skein_lib::bytes::find(&printed.stdout, br#""speaker":"system""#).is_some());
        }
    }
}

#[test]
fn invalid_run_ends_print_their_reason() {
    use jig_client::contract as c;
    for (reason, expected) in [
        (c::InvalidReason::Form, b"run 3 ended: its result broke its contract (form)".as_slice()),
        (c::InvalidReason::Verdict, b"run 3 ended: its result broke its contract (verdict)"),
        (c::InvalidReason::Words, b"run 3 ended: its result broke its contract (words)"),
        (c::InvalidReason::Change, b"run 3 ended: its result broke its contract (change)"),
        (c::InvalidReason::Followups, b"run 3 ended: its result broke its contract (followups)"),
        (c::InvalidReason::UnreadableResult, b"run 3 ended: its result couldn't be read"),
    ] {
        let mut rows = skein_lib::List::with_capacity(1);
        rows.push(c::DecodedActivityLine {
            source: Box::new(c::ActivityLine {
                task: 41,
                position: 1,
                at: skein_lib::Wall::from_nanos(0),
                line: c::Activity::RunEnded {
                    attempt: 3,
                    end: c::RunEnd::Failed { class: c::HoldFailure::Invalid, invalid_reason: Some(reason) },
                    spent: 0,
                },
            }),
            decoded: None,
        })
        .expect("one run end");
        let page = ReadResult::Value(ReadValue::Activity(Box::new(c::ActivityPage {
            cut: false,
            rows,
            before: None,
            after: None,
        })));
        let printed = fixtures::read_answer(fixtures::ACTIVITY_CALL, page.clone());
        assert_eq!(printed.status, 0);
        assert!(skein_lib::bytes::find(&printed.stdout, expected).is_some(), "{reason:?}: {:?}", printed.stdout);
        let json = fixtures::read_answer(&["task", "activity", "41", "--json"], page);
        assert!(skein_lib::bytes::find(&json.stdout, b"invalid_reason").is_some(), "{reason:?}: {:?}", json.stdout);
    }
}
