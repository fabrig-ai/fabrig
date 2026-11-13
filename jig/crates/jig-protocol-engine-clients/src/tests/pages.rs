use super::fixtures;
use crate::translate::pages;
use jig_client_protocol_wire::v21 as w;
use jig_domain_core_vocabulary::reads;

fn activity(read: &reads::Page, limits: &w::Limits) -> Result<w::ReadValue, w::Problem> {
    pages::page(
        &fixtures::models(),
        &reads::Read::Activity { task: 7, before: None },
        read,
        &crate::Private::none(),
        limits,
    )
}
#[test]
fn a_heard_job_result_does_not_refuse_the_activity_read() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{List, Reader, Wall};
    for bytes in [b"".as_slice(), b"job bytes", &[0, 255, 7]] {
        let mut exchange = fixtures::read_exchange();
        let limits = exchange.env.limits.wire;
        let read =
            w::Read::Activity(w::ReadActivity::new(&limits, w::ReadActivityParts { task: 7, before: None }).unwrap());
        let (owner, reply, _) = exchange.read(&read);
        let word = t::Word {
            number: 19,
            from: t::Party::Task(11),
            kind: t::MessageKind::Result(t::ResultKind::Job),
            words: bytes.into(),
            at: Wall::from_nanos(23),
            hits: 2,
            eligible: true,
        };
        let mut rows = List::with_capacity(3);
        for (position, line) in [
            (3, t::Line::RunBegan { attempt: 13, why: t::Why::Made, resumed: false }),
            (2, t::Line::Heard(word)),
            (
                1,
                t::Line::Words {
                    message: 17,
                    by: t::Party::Person(5),
                    kind: t::MessageKind::Words,
                    words: Box::from(b"hello".as_slice()),
                },
            ),
        ] {
            rows.push(reads::PageRow::Activity {
                line: t::ActivityLine { task: 7, position, at: Wall::from_nanos(29), line },
                turn: None,
            })
            .unwrap();
        }
        exchange.release_read(reply, reads::Answer::Page(Box::new(fixtures::page(rows))));
        let w::ReadAnswer::Value(answer) = exchange.read_answer(owner) else {
            panic!("served activity, not malformed")
        };
        let w::ReadValue::Activity(page) = w::ReadValue::decode(&limits, &mut Reader::new(answer.record())).unwrap()
        else {
            panic!("activity")
        };
        let mut pieces = Reader::new(page.rows());
        let mut wire_rows = List::with_capacity(3);
        while let Some(piece) = skein_codec::piece::split(&mut pieces).unwrap() {
            wire_rows.push(w::DecodedActivityLine::decode(&limits, &mut Reader::new(&piece)).unwrap()).unwrap();
        }
        assert_eq!(wire_rows.len(), 3);
        let w::Activity::Heard(heard) = wire_rows.get(1).unwrap().source().line() else { panic!("heard") };
        let w::MessageKind::Result(result) = heard.value().kind() else { panic!("result") };
        assert_eq!(result.value(), &w::WordResult::Job);
        assert_eq!(heard.value().words(), b"");
        let client = jig_client_domain::Limits { rows: 3, ..*fixtures::client_limits().client() };
        let client = jig_client_protocol_link::Limits::new(client, 4).unwrap();
        let page = jig_client_protocol_link::translate::pages::activity_page(page, &client).unwrap();
        assert_eq!(page.rows.len(), 3);
        assert!(page.cut);
        for (row, (position, line)) in page.rows.iter().zip([
            (3, c::Activity::RunBegan { attempt: 13, why: c::RunWhy::Made, resumed: false }),
            (
                2,
                c::Activity::Heard(c::Word {
                    number: 19,
                    from: c::Requester::Task(11),
                    kind: c::MessageKind::Result(c::WordResult::Job),
                    words: c::Text::new(Box::new([]), 0).unwrap(),
                    at: Wall::from_nanos(23),
                    hits: 2,
                    eligible: true,
                }),
            ),
            (
                1,
                c::Activity::Words {
                    message: 17,
                    by: c::Requester::Person(5),
                    kind: c::MessageKind::Words,
                    words: c::Text::new(Box::from(b"hello".as_slice()), 5).unwrap(),
                },
            ),
        ]) {
            assert_eq!(*row.source, c::ActivityLine { task: 7, position, at: Wall::from_nanos(29), line });
            assert!(row.decoded.is_none());
        }
    }
}

#[test]
fn job_results_cross_word_lines_and_question_answers_without_bytes() {
    use jig_domain_tasks as t;
    use skein_lib::{Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    for bytes in [b"".as_slice(), b"job bytes", &[0, 255, 7]] {
        let word = t::Word {
            number: 19,
            from: t::Party::Task(11),
            kind: t::MessageKind::Result(t::ResultKind::Job),
            words: bytes.into(),
            at: Wall::from_nanos(23),
            hits: 2,
            eligible: true,
        };
        for line in [
            t::Line::Words { message: 19, by: word.from, kind: word.kind.clone(), words: word.words.clone() },
            t::Line::Heard(word.clone()),
            t::Line::Notified { recipient: t::Party::Task(7), word },
        ] {
            let source = t::ActivityLine { task: 7, position: 2, at: Wall::from_nanos(29), line };
            let answer = crate::translate::objects::question_answer::answer(&source, &limits).unwrap();
            assert_eq!(crate::translate::activity::line(&source, &limits).unwrap(), answer);
            let mut out = Writer::new(usize::try_from(answer.measure()).unwrap());
            answer.encode(&mut out).unwrap();
            let decoded = w::ActivityLine::decode(&limits, &mut Reader::new(&out.finish())).unwrap();
            assert_eq!(decoded, answer);
            let (kind, words) = match decoded.line() {
                w::Activity::Words(row) => (row.kind(), row.words()),
                w::Activity::Heard(row) => (row.value().kind(), row.value().words()),
                w::Activity::Notified(row) => (row.word().kind(), row.word().words()),
                other @ (w::Activity::RunBegan(_)
                | w::Activity::RunEnded(_)
                | w::Activity::Turn(_)
                | w::Activity::Decided(_)
                | w::Activity::Milestone(_)
                | w::Activity::Kept(_)
                | w::Activity::Stood(_)
                | w::Activity::SessionEnded(_)
                | w::Activity::TalkOpened(_)
                | w::Activity::TalkEnded(_)
                | w::Activity::NowWritten(_)) => panic!("word carrier: {other:?}"),
            };
            let w::MessageKind::Result(result) = kind else { panic!("result") };
            assert_eq!(result.value(), &w::WordResult::Job);
            assert_eq!(words, b"");
            assert_eq!((decoded.task(), decoded.position(), decoded.at()), (7, 2, 29));
        }
    }
}

#[test]
fn an_activity_page_carries_each_turn_s_bytes() {
    let limits = fixtures::wire_limits();
    let source = fixtures::activity_page();
    assert_eq!(
        activity(&source, &limits),
        Ok(w::ReadValue::Activity(jig_client_test_records::carried_activity_page()))
    );
    for withheld in [false, true] {
        let source = fixtures::private_activity(withheld);
        let w::ReadValue::Activity(page) = pages::page(
            &fixtures::models(),
            &reads::Read::Activity { task: 7, before: None },
            &source,
            &fixtures::private(),
            &limits,
        )
        .unwrap() else {
            panic!("activity page")
        };
        let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(page.rows())).unwrap().unwrap();
        let line = w::DecodedActivityLine::decode(&limits, &mut skein_lib::Reader::new(&bytes)).unwrap();
        let archive = line.decoded().as_ref().unwrap();
        let w::TurnBytes::Carried(turn) = archive.turn() else { panic!("carried projection") };
        assert_eq!(turn.bytes(), fixtures::private_transcript(withheld).as_ref(), "withheld {withheld}");
        assert_eq!(
            (
                archive.task(),
                archive.name().attempt(),
                archive.name().number(),
                archive.cumulative(),
                *archive.read(),
                archive.at()
            ),
            (7, 31, 37, 19, Some(0), 23)
        );
    }
    for (label, limits, path) in [
        ("turn bytes", w::Limits { turn_carried_bytes: 2, ..limits }, w::Path::TurnCarriedBytes),
        ("page pieces", w::Limits { activity_page_rows: 0, ..limits }, w::Path::ActivityPageRows),
        ("opaque cursor", w::Limits { cursor_value: 1, ..limits }, w::Path::CursorValue),
    ] {
        assert_eq!(activity(&source, &limits), Err(w::Problem { path, reason: skein_codec::Reason::Bound }), "{label}");
    }
}
#[test]
fn a_turn_past_the_page_crosses_as_its_size() {
    let limits = fixtures::wire_limits();
    let source = fixtures::oversized_turn_page();
    assert_eq!(
        activity(&source, &limits),
        Ok(w::ReadValue::Activity(jig_client_test_records::oversized_activity_page()))
    );
}
#[test]
fn a_history_page_keeps_its_cut_flag() {
    let limits = fixtures::wire_limits();
    let mut source = fixtures::history_page();
    let literal = jig_client_test_records::mixed_history_page();
    for (label, cut) in [("cut", true), ("whole", false)] {
        source.cut = cut;
        let mut expected = literal.clone().into_parts();
        expected.cut = cut;
        assert_eq!(
            pages::page(
                &fixtures::models(),
                &reads::Read::History { task: 7, subtree: false, from: None },
                &source,
                &crate::Private::none(),
                &limits
            ),
            Ok(w::ReadValue::History(w::HistoryPage::new(&limits, expected).expect(label))),
            "{label}"
        );
    }
}
#[test]
fn an_inbox_page_keeps_its_two_groups_apart() {
    let limits = fixtures::wire_limits();
    let needs = fixtures::inbox_page(false);
    let updates = fixtures::inbox_page(true);
    let records = jig_client_test_records::selected_inbox_pages();
    for (label, group, source, record) in [
        ("needs", reads::Group::Needs, &needs, records[0].clone()),
        ("updates", reads::Group::Updates, &updates, records[1].clone()),
    ] {
        let read = reads::Read::Inbox { group, filter: reads::Filter { project: None, goal: None }, from: None };
        assert_eq!(
            pages::page(&fixtures::models(), &read, source, &crate::Private::none(), &limits),
            Ok(w::ReadValue::Inbox(record)),
            "{label}"
        );
    }
}
#[test]
fn a_notes_page_becomes_its_record() {
    let limits = fixtures::wire_limits();
    let source = fixtures::notes_page();
    let read = reads::Read::Notes { project: 13, from: None };
    assert_eq!(
        pages::page(&fixtures::models(), &read, &source, &crate::Private::none(), &limits),
        Ok(w::ReadValue::Notes(jig_client_test_records::scoped_notes_page()))
    );
}

#[test]
fn kept_lines_and_met_notices_cross_both_client_translations() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{List, Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    for (removed, question) in [(false, None), (true, Some(29))] {
        let mut rows = List::with_capacity(2);
        for (position, line) in [
            (13, t::Line::Kept { attempt: 3, pending: Box::new([19, 23]), question }),
            (
                11,
                t::Line::Heard(t::Word {
                    number: 17,
                    from: t::Party::Task(29),
                    kind: t::MessageKind::Met { dependency: 19, removed },
                    words: Box::new([]),
                    at: Wall::from_nanos(31),
                    hits: 1,
                    eligible: true,
                }),
            ),
        ] {
            rows.push(reads::PageRow::Activity {
                line: t::ActivityLine { task: 7, position, at: Wall::from_nanos(37), line },
                turn: None,
            })
            .unwrap();
        }
        let w::ReadValue::Activity(record) = activity(&fixtures::page(rows), &limits).unwrap() else {
            panic!("activity page");
        };
        let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut out).unwrap();
        let bytes = out.finish();
        let record = w::ActivityPage::decode(&limits, &mut Reader::new(&bytes)).unwrap();
        let actual =
            jig_client_protocol_link::translate::pages::activity_page(record, &fixtures::client_limits()).unwrap();
        let c::DecodedActivityLine { source, decoded } = actual.rows.get(0).unwrap();
        assert!(decoded.is_none());
        let c::Activity::Kept { attempt, pending, question: kept_question } = &source.line else { panic!("kept") };
        assert_eq!((*attempt, pending.as_slice()), (3, [19, 23].as_slice()));
        assert_eq!(*kept_question, question);
        let c::DecodedActivityLine { source, .. } = actual.rows.get(1).unwrap();
        let c::Activity::Heard(word) = &source.line else { panic!("heard") };
        assert_eq!(word.kind, c::MessageKind::Met { dependency: 19, removed });
        assert_eq!(word.from, c::Requester::Task(29));
    }
}

#[test]
fn proposal_lines_keep_their_identity_and_making_through_the_client() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{List, Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    for proposal in [
        None,
        Some(t::DecidedProposal { number: 19, made: true }),
        Some(t::DecidedProposal { number: 19, made: false }),
    ] {
        let mut rows = List::with_capacity(1);
        rows.push(reads::PageRow::Activity {
            line: t::ActivityLine {
                task: 7,
                position: 3,
                at: Wall::EPOCH,
                line: t::Line::Decided { revision: 5, proposal },
            },
            turn: None,
        })
        .unwrap();
        let w::ReadValue::Activity(record) = activity(&fixtures::page(rows), &limits).unwrap() else {
            panic!("activity")
        };
        let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut out).unwrap();
        let bytes = out.finish();
        let record = w::ActivityPage::decode(&limits, &mut Reader::new(&bytes)).unwrap();
        let actual =
            jig_client_protocol_link::translate::pages::activity_page(record, &fixtures::client_limits()).unwrap();
        let expected = match proposal {
            Some(value) => Some(c::DecidedProposal { number: value.number, made: value.made }),
            None => None,
        };
        assert_eq!(actual.rows.get(0).unwrap().source.line, c::Activity::Decided { revision: 5, proposal: expected });
    }
}

#[test]
fn a_line_at_the_largest_profile_bound_encodes() {
    use jig_domain_tasks as t;
    use skein_lib::{Reader, Wall, Writer};
    let mut words = skein_lib::bytes::zeroed(262_144);
    words.fill(b'w');
    let word = t::Word {
        number: 1,
        from: t::Party::Person(3),
        kind: t::MessageKind::Words,
        words: words.clone(),
        at: Wall::from_nanos(5),
        hits: 1,
        eligible: true,
    };
    for line in [
        t::Line::Words { message: 1, by: t::Party::Person(3), kind: t::MessageKind::Words, words: words.clone() },
        t::Line::Heard(word.clone()),
        t::Line::Notified { recipient: t::Party::Person(3), word },
    ] {
        let line = t::ActivityLine { task: 7, position: 1, at: Wall::from_nanos(5), line };
        let record = crate::translate::activity::line(&line, &w::CEILINGS).unwrap();
        let mut encoded = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut encoded).unwrap();
        let encoded = encoded.finish();
        let decoded = w::ActivityLine::decode(&w::CEILINGS, &mut Reader::new(&encoded)).unwrap();
        let decoded_words = match decoded.line() {
            w::Activity::Words(record) => record.words(),
            w::Activity::Heard(record) => record.value().words(),
            w::Activity::Notified(record) => record.word().words(),
            other @ (w::Activity::RunBegan(_)
            | w::Activity::RunEnded(_)
            | w::Activity::Turn(_)
            | w::Activity::Decided(_)
            | w::Activity::Milestone(_)
            | w::Activity::NowWritten(_)
            | w::Activity::Stood(_)
            | w::Activity::TalkOpened(_)
            | w::Activity::TalkEnded(_)
            | w::Activity::SessionEnded(_)
            | w::Activity::Kept(_)) => panic!("word line: {other:?}"),
        };
        assert_eq!(decoded_words, words.as_ref());
    }
}

#[test]
fn standing_activity_crosses_both_client_translations_at_its_words_bound() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{List, Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    let client = jig_client_domain::Limits { long_text: t::STOOD_WORDS, ..*fixtures::client_limits().client() };
    let client = jig_client_protocol_link::Limits::new(client, 4).unwrap();
    let words = Box::<[u8]>::from([b'w'; 512]);
    let mut cases = List::with_capacity(7);
    cases
        .push((
            t::Line::Stood { attempt: 3, words: words.clone() },
            c::Activity::Stood { attempt: 3, words: c::Text::new(words, t::STOOD_WORDS).unwrap() },
        ))
        .unwrap();
    for (end, expected) in [
        (t::SessionEnd::Quiet, c::SessionEnd::Quiet),
        (t::SessionEnd::Bound, c::SessionEnd::Bound),
        (t::SessionEnd::Unoffered, c::SessionEnd::Unoffered),
        (t::SessionEnd::Fresh, c::SessionEnd::Fresh),
        (t::SessionEnd::Closed, c::SessionEnd::Closed),
        (t::SessionEnd::Rules, c::SessionEnd::Rules),
    ] {
        cases
            .push((t::Line::SessionEnded { attempt: 3, end }, c::Activity::SessionEnded { attempt: 3, end: expected }))
            .unwrap();
    }
    for (line, expected) in cases.into_boxed() {
        let mut rows = List::with_capacity(1);
        rows.push(reads::PageRow::Activity {
            line: t::ActivityLine { task: 7, position: 13, at: Wall::from_nanos(37), line },
            turn: None,
        })
        .unwrap();
        let w::ReadValue::Activity(record) = activity(&fixtures::page(rows), &limits).unwrap() else {
            panic!("activity page")
        };
        let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut out).unwrap();
        let record = w::ActivityPage::decode(&limits, &mut Reader::new(&out.finish())).unwrap();
        let actual = jig_client_protocol_link::translate::pages::activity_page(record, &client).unwrap();
        let c::DecodedActivityLine { source, decoded } = actual.rows.get(0).unwrap();
        assert_eq!(
            source.as_ref(),
            &c::ActivityLine { task: 7, position: 13, at: Wall::from_nanos(37), line: expected }
        );
        assert!(decoded.is_none());
    }
    let line = t::ActivityLine {
        task: 7,
        position: 13,
        at: Wall::EPOCH,
        line: t::Line::Stood { attempt: 3, words: Box::from([b'w'; 513]) },
    };
    assert_eq!(crate::translate::activity::line(&line, &limits).unwrap_err().path, w::Path::ActivityStoodWords);
    let line = t::ActivityLine {
        task: 7,
        position: 13,
        at: Wall::EPOCH,
        line: t::Line::Stood { attempt: 3, words: Box::from([b'w'; 512]) },
    };
    let record = crate::translate::activity::line(&line, &limits).unwrap();
    let narrower = w::Limits { activity_stood_words: t::STOOD_WORDS - 1, ..limits };
    let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut out).unwrap();
    assert_eq!(
        w::ActivityLine::decode(&narrower, &mut Reader::new(&out.finish())).unwrap_err().path,
        w::Path::ActivityStoodWords
    );
}

#[test]
fn talk_lines_and_the_two_session_ends_cross_the_client_wire() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{Reader, Wall, Writer};
    for (line, expected) in [
        (t::Line::TalkOpened { party: 41 }, c::Activity::TalkOpened { party: 41 }),
        (t::Line::TalkEnded { party: 41, lapsed: false }, c::Activity::TalkEnded { party: 41, lapsed: false }),
        (t::Line::TalkEnded { party: 41, lapsed: true }, c::Activity::TalkEnded { party: 41, lapsed: true }),
        (
            t::Line::SessionEnded { attempt: 3, end: t::SessionEnd::Talk },
            c::Activity::SessionEnded { attempt: 3, end: c::SessionEnd::Talk },
        ),
        (
            t::Line::SessionEnded { attempt: 3, end: t::SessionEnd::Left },
            c::Activity::SessionEnded { attempt: 3, end: c::SessionEnd::Left },
        ),
    ] {
        let mut rows = skein_lib::List::with_capacity(1);
        rows.push(reads::PageRow::Activity {
            line: t::ActivityLine { task: 7, position: 13, at: Wall::from_nanos(37), line },
            turn: None,
        })
        .unwrap();
        let w::ReadValue::Activity(record) = activity(&fixtures::page(rows), &w::CEILINGS).unwrap() else {
            panic!("activity page")
        };
        let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut bytes).unwrap();
        let record = w::ActivityPage::decode(&w::CEILINGS, &mut Reader::new(&bytes.finish())).unwrap();
        let decoded =
            jig_client_protocol_link::translate::pages::activity_page(record, &fixtures::client_limits()).unwrap();
        assert_eq!(decoded.rows.get(0).unwrap().source.line, expected);
    }
}

#[test]
fn now_written_crosses_the_client_wire_at_its_line_bound() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    use skein_lib::{Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    let client = jig_client_domain::Limits { long_text: t::NOW_LINE, ..*fixtures::client_limits().client() };
    let client = jig_client_protocol_link::Limits::new(client, 4).unwrap();
    let text = "界".repeat(170) + "ab";
    let line = t::ActivityLine {
        task: 7,
        position: 13,
        at: Wall::from_nanos(37),
        line: t::Line::NowWritten { attempt: 3, line: text.as_bytes().into() },
    };
    let mut rows = skein_lib::List::with_capacity(1);
    rows.push(reads::PageRow::Activity { line: line.clone(), turn: None }).unwrap();
    let w::ReadValue::Activity(record) = activity(&fixtures::page(rows), &limits).unwrap() else {
        panic!("activity page")
    };
    let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut bytes).unwrap();
    let record = w::ActivityPage::decode(&limits, &mut Reader::new(&bytes.finish())).unwrap();
    let decoded = jig_client_protocol_link::translate::pages::activity_page(record, &client).unwrap();
    let actual = decoded.rows.get(0).unwrap();
    assert_eq!(
        actual.source.as_ref(),
        &c::ActivityLine {
            task: 7,
            position: 13,
            at: Wall::from_nanos(37),
            line: c::Activity::NowWritten {
                attempt: 3,
                line: c::Text::new(text.as_bytes().into(), t::NOW_LINE).unwrap()
            },
        }
    );
    assert!(actual.decoded.is_none());
    let record = crate::translate::activity::line(&line, &limits).unwrap();
    let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut bytes).unwrap();
    let narrower = w::Limits { activity_now_written_line: t::NOW_LINE - 1, ..limits };
    assert_eq!(
        w::ActivityLine::decode(&narrower, &mut Reader::new(&bytes.finish())).unwrap_err().path,
        w::Path::ActivityNowWrittenLine
    );
    let line = t::ActivityLine { line: t::Line::NowWritten { attempt: 3, line: Box::from([b'n'; 513]) }, ..line };
    assert_eq!(crate::translate::activity::line(&line, &limits).unwrap_err().path, w::Path::ActivityNowWrittenLine);
}
