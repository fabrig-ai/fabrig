use super::*;
use skein_lib::length::len_u64;

fn a_history_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::history_page(fixtures::read_wire_history_page(), &fixtures::limits()),
        Ok(fixtures::read_value_history_page())
    );
}
fn a_note_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::note_page(fixtures::read_wire_note_page(), &fixtures::limits()),
        Ok(fixtures::read_value_note_page())
    );
}
fn a_owner_ended_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::ended_page(fixtures::read_wire_owner_ended_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_ended_page())
    );
}
fn a_owner_inbox_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::inbox_page(fixtures::read_wire_owner_inbox_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_inbox_page())
    );
}
#[test]
fn an_inbox_entry_33_goals_deep_is_received() {
    use skein_lib::{List, Writer};
    let base = fixtures::limits();
    let limits = Limits::new(jig_client_domain::Limits { rows: 8, ..*base.client() }, base.exchanges()).unwrap();
    let mut page = jig_client_test_records::selected_inbox_pages()[0].clone();
    let mut rows = Reader::new(&page.rows);
    let bytes = skein_codec::piece::split(&mut rows).unwrap().unwrap();
    let row = w::InboxEntry::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let mut goals = List::with_capacity(33);
    for goal in 1..=33 {
        goals.push(goal).unwrap();
    }
    let selection = w::InboxSelection { cursor: row.cursor.clone(), goals: goals.clone() };
    let mut encoded = Writer::new(
        usize::try_from(selection.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    selection.write(&jig_client_protocol_wire::CEILINGS, &mut encoded).unwrap();
    let bytes = encoded.finish();
    let mut piece = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &bytes).unwrap();
    page.selection = Some(piece.finish());
    let page = page;
    let mut encoded = Writer::new(
        usize::try_from(page.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    page.write(&jig_client_protocol_wire::CEILINGS, &mut encoded).unwrap();
    let bytes = encoded.finish();
    let page = w::InboxPage::decode(limits.receiving(), &mut Reader::new(&bytes)).unwrap();
    let page = translate::pages::inbox_page(page, &limits).unwrap();
    assert_eq!(page.rows.len(), 1);
    let selection = page.selection.unwrap();
    assert_eq!(selection.len(), 1);
    let selection = selection.get(0).unwrap();
    assert_eq!(selection.cursor, page.rows.get(0).unwrap().cursor);
    assert_eq!(selection.goals, goals);
}
fn a_owner_spend_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::spend_page(fixtures::read_wire_owner_spend_page(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_spend_page())
    );
}
fn a_owner_proposal_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::proposal(fixtures::read_wire_owner_proposal(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_proposal())
    );
}
#[test]
fn escalation_acts_fit_the_contract_bound() {
    let base = fixtures::limits();
    let limits = Limits::new(jig_client_domain::Limits { rows: 32, ..*base.client() }, base.exchanges()).unwrap();
    let offered = w::Offered { effect: None, act: w::ActKind::Accept, admission: w::Admission::Allowed };
    let mut encoded = skein_lib::Writer::new(
        usize::try_from(offered.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    offered.write(&jig_client_protocol_wire::CEILINGS, &mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut acts = skein_lib::Writer::new(5 * (encoded.len() + 4));
    for _ in 0_u32..5 {
        skein_codec::piece::write(&mut acts, &encoded).unwrap();
    }
    let mut parts = fixtures::read_wire_owner_escalation();
    parts.acts = acts.finish();
    let record = parts;
    let document = translate::pages::escalation_fields(&record, &limits).unwrap();
    assert_eq!(document.acts.len(), 5);
    assert!(document.acts.capacity() <= c::MAX_OFFERED_ACTS);
}

fn a_owner_escalation_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::escalation(fixtures::read_wire_owner_escalation(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_escalation())
    );
}
fn a_question_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::question(fixtures::read_wire_question(), &fixtures::limits()),
        Ok(fixtures::read_value_question())
    );
}
fn a_owner_person_task_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::person_task(fixtures::read_wire_owner_person_task(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_person_task())
    );
}
fn a_result_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::result(fixtures::read_wire_result(), &fixtures::limits()),
        Ok(fixtures::read_value_result())
    );
}
fn a_owner_ended_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::objects::ended(fixtures::read_wire_owner_ended(), &fixtures::limits()),
        Ok(fixtures::read_value_owner_ended())
    );
}

#[test]
fn a_page_keeps_its_two_cursors_as_bytes() {
    assert_eq!(
        translate::pages::note_page(fixtures::note_page_with_cursors(), &fixtures::limits()),
        Ok(fixtures::expected_note_page_with_cursors())
    );
}
#[test]
fn an_ended_task_keeps_its_ending() {
    assert_eq!(
        translate::objects::ended(fixtures::ended_task_record(), &fixtures::limits()),
        Ok(fixtures::expected_ended_task())
    );
}
#[test]
fn a_page_admits_each_generated_row_at_its_limit() {
    assert_eq!(
        translate::pages::ended_page(fixtures::ended_page_piece(fixtures::ended_rows(2)), &fixtures::limits()),
        Ok(fixtures::expected_owner_ended_page(2))
    );
}
#[test]
fn a_page_one_row_past_its_limit_is_refused() {
    assert_eq!(
        translate::pages::ended_page(fixtures::ended_page_piece(fixtures::ended_rows(3)), &fixtures::limits()),
        Err(Undecodable::TooLarge)
    );
}
#[test]
fn a_page_with_an_incomplete_piece_header_is_refused() {
    assert_eq!(
        translate::pages::ended_page(fixtures::ended_page_piece(Box::from([1, 0, 0])), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
#[test]
fn a_page_with_trailing_row_bytes_is_refused() {
    assert_eq!(
        translate::pages::ended_page(fixtures::ended_page_piece(fixtures::trailing_ended_row()), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
#[test]
fn a_page_one_byte_past_the_document_limit_is_refused() {
    let record = fixtures::ended_page_piece(fixtures::ended_rows(1));
    let mut client = *fixtures::limits().client();
    client.document_bytes = u64::from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
        .checked_sub(1)
        .unwrap();
    let limits = Limits::new(client, 4).unwrap();
    assert_eq!(translate::pages::ended_page(record, &limits), Err(Undecodable::TooLarge));
}

fn a_activity_page_record_becomes_its_literal_contract_value() {
    assert_eq!(
        translate::pages::activity_page(fixtures::read_wire_activity_page(), &fixtures::limits()),
        Ok(fixtures::read_value_activity_page())
    );
}

#[test]
fn an_activity_page_decodes_each_turn() {
    assert_eq!(
        translate::pages::activity_page(fixtures::decoded_activity_record(), &fixtures::limits()),
        Ok(fixtures::decoded_activity_page())
    );
}
#[test]
fn an_undecodable_turn_crosses_up_as_such() {
    let limits = fixtures::limits();
    for (record, expected) in fixtures::unread_pages() {
        assert_eq!(translate::pages::activity_page(record, &limits), Ok(expected));
    }
}
#[test]
fn an_archived_turn_keeps_its_envelope_when_decoding_fails() {
    assert_eq!(
        translate::pages::activity_page(fixtures::archived_activity_record(), &fixtures::limits()),
        Ok(fixtures::archived_activity_page())
    );
}
#[test]
fn a_turns_receiving_cut_stays_on_its_decoded_value() {
    let limits = fixtures::turn_limits(2, 65536);
    assert_eq!(
        translate::pages::activity_page(fixtures::cut_activity_record(), &limits),
        Ok(fixtures::cut_activity_page())
    );
}
#[test]
fn a_turn_at_its_admission_bound_is_decoded() {
    assert_eq!(len_u64(fixtures::full_turn().len()), fixtures::TURN_PAYLOAD_BYTES);
    let limits = fixtures::turn_limits(32, fixtures::TURN_PAYLOAD_BYTES);
    assert_eq!(
        translate::pages::carried_turn_fields(&fixtures::full_turn(), &limits),
        fixtures::decoded_turn(b"abc", false)
    );
}
#[test]
fn a_turn_past_its_admission_bound_is_carried_as_too_large() {
    let limits = fixtures::turn_limits(32, fixtures::TURN_PAYLOAD_BYTES.checked_sub(1).unwrap());
    assert_eq!(
        translate::pages::carried_turn_fields(&fixtures::full_turn(), &limits),
        c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes: fixtures::TURN_PAYLOAD_BYTES } }
    );
}

#[test]
fn an_owner_turn_without_carried_bytes_crosses_up_with_its_problem() {
    let limits = fixtures::limits();
    assert_eq!(fixtures::MARKER_BYTES, u64::from(w::CEILINGS.turn_carried_bytes).checked_add(1).unwrap());
    for (name, record, expected) in fixtures::turn_pages() {
        assert_eq!(translate::pages::activity_page(record, &limits), Ok(expected), "{name}");
    }
}

#[test]
fn page_and_object_records_become_their_literal_contract_values() {
    table([
        (
            "a_history_page_record_becomes_its_literal_contract_value",
            a_history_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_note_page_record_becomes_its_literal_contract_value",
            a_note_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_ended_page_record_becomes_its_literal_contract_value",
            a_owner_ended_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_inbox_page_record_becomes_its_literal_contract_value",
            a_owner_inbox_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_spend_page_record_becomes_its_literal_contract_value",
            a_owner_spend_page_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_proposal_record_becomes_its_literal_contract_value",
            a_owner_proposal_record_becomes_its_literal_contract_value,
        ),
        (
            "a_owner_escalation_record_becomes_its_literal_contract_value",
            a_owner_escalation_record_becomes_its_literal_contract_value,
        ),
        ("a_question_record_becomes_its_literal_contract_value", a_question_record_becomes_its_literal_contract_value),
        (
            "a_owner_person_task_record_becomes_its_literal_contract_value",
            a_owner_person_task_record_becomes_its_literal_contract_value,
        ),
        ("a_result_record_becomes_its_literal_contract_value", a_result_record_becomes_its_literal_contract_value),
        (
            "a_owner_ended_record_becomes_its_literal_contract_value",
            a_owner_ended_record_becomes_its_literal_contract_value,
        ),
        (
            "a_activity_page_record_becomes_its_literal_contract_value",
            a_activity_page_record_becomes_its_literal_contract_value,
        ),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    skein_rows::check(cases, |case| case());
}

#[test]
fn a_task_page_decodes_its_two_groups_with_their_cursors() {
    let limits = fixtures::limits();
    let mut page = jig_client_test_records::task_page_record();
    page.live = jig_client_test_records::tasks_rows_record().rows;
    page.live_after = Some(w::Cursor { value: b"live".as_slice().into() });
    let after = Some(w::Cursor { value: b"ended".as_slice().into() });
    page.ended = w::EndedGroup::Rows(w::EndedRows { rows: Box::new([]), after, cut: true });
    let page = translate::pages::task_page(page, &limits).unwrap();
    assert_eq!(page.live.len(), 1);
    assert_eq!(page.live_after.as_ref().unwrap().value.bytes(), b"live");
    let c::EndedGroup::Rows { rows, after, cut } = page.ended else { panic!("ended rows") };
    assert_eq!(rows.len(), 0);
    assert_eq!(after.as_ref().unwrap().value.bytes(), b"ended");
    assert!(cut);
}

#[test]
fn an_offer_decodes_the_effect_with_its_counts_or_reservation() {
    let limits = fixtures::limits();
    let mut impact = fixtures::read_wire_impact();
    impact.live_runs = 11;
    impact.proposals = 13;
    let effect = w::Effect::Cancel(impact);
    let value = w::Offered { act: w::ActKind::Cancel, admission: w::Admission::Allowed, effect: Some(effect) };
    let value = translate::documents::offered_fields(&value, &limits).unwrap();
    let Some(c::Effect::Cancel(impact)) = value.effect else { panic!("cancel effect") };
    assert_eq!((impact.live_runs, impact.proposals), (11, 13));
    assert_eq!(impact.tasks, fixtures::read_value_impact().tasks);
    let effect = w::Effect::MoveTask(w::EffectMove { reservation: 17 });
    assert_eq!(translate::documents::effect_fields(&effect), c::Effect::Move { reservation: 17 });
}

#[test]
fn proposal_history_and_activity_keep_their_card_facts() {
    let limits = fixtures::limits();
    for made in [false, true] {
        let record = w::Activity::Decided(w::ActivityDecided {
            revision: 5,
            proposal: Some(w::ActivityDecidedProposal { number: 19, made }),
        });
        assert_eq!(
            translate::pages::activity_fields(&record, &limits).unwrap(),
            c::Activity::Decided { revision: 5, proposal: Some(c::DecidedProposal { number: 19, made }) }
        );
    }
    let mut projects = skein_lib::List::with_capacity(1);
    projects.push(23).unwrap();
    let record = w::ProposalHistory {
        reason: w::DocumentText { bytes: b"why".as_slice().into(), cut: true },
        action: w::RetainedAction::Settings(w::RetainedActionSettings {
            projects: projects.clone(),
            against: 3,
            items: skein_lib::List::with_capacity(0),
            made: Some(5),
        }),
        number: 19,
        proposer: 7,
        as_holder: false,
        state: w::ProposalHistoryState::Withdrawn,
    };
    let proposal = translate::pages::proposal_history_fields(&record, &limits).unwrap();
    assert_eq!(proposal.reason.bytes(), b"why");
    assert!(proposal.reason.cut);
    let c::RetainedAction::Settings(action) = proposal.action else { panic!("settings") };
    assert_eq!(action.projects, projects);
    assert_eq!(action.made, Some(5));
}

fn line_limits() -> Limits {
    let base = fixtures::limits();
    Limits::new(jig_client_domain::Limits { text: 32, long_text: 128, ..*base.client() }, base.exchanges()).unwrap()
}

fn word_lines(words: &[u8]) -> [w::Activity; 3] {
    let requester = w::Requester::Deployment(w::RequesterDeployment { project: 1 });
    let word = w::Word {
        number: 1,
        from: requester.clone(),
        kind: w::MessageKind::Words,
        words: Box::from(words),
        at: 2,
        hits: 1,
        eligible: true,
    };
    [
        w::Activity::Words(w::ActivityWords {
            message: 1,
            by: requester.clone(),
            kind: w::MessageKind::Words,
            words: Box::from(words),
        }),
        w::Activity::Heard(w::ActivityHeard { value: word.clone() }),
        w::Activity::Notified(w::ActivityNotified { recipient: requester, word }),
    ]
}

fn word_line(line: w::Activity) -> w::ActivityLine {
    w::ActivityLine { task: 7, position: 1, at: 2, line }
}

fn word_page(line: w::Activity) -> w::ActivityPage {
    let row = w::DecodedActivityLine { source: word_line(line), decoded: None };
    let mut encoded = skein_lib::Writer::new(
        usize::try_from(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    row.write(&jig_client_protocol_wire::CEILINGS, &mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut piece = skein_lib::Writer::new(encoded.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &encoded).unwrap();
    w::ActivityPage { rows: piece.finish(), before: None, after: None, cut: false }
}

#[test]
fn a_line_at_long_text_decodes_whole() {
    let limits = line_limits();
    for size in [33, 128] {
        let mut words = skein_lib::bytes::zeroed(size);
        words.fill(b'w');
        for line in word_lines(&words) {
            let page = translate::pages::activity_page(word_page(line), &limits).unwrap();
            assert_eq!(page.rows.len(), 1);
            let words_read = match &page.rows.get(0).unwrap().source.line {
                c::Activity::Words { words, .. } => words,
                c::Activity::Heard(word) | c::Activity::Notified { word, .. } => &word.words,
                other @ (c::Activity::RunBegan { .. }
                | c::Activity::RunEnded { .. }
                | c::Activity::Turn { .. }
                | c::Activity::Decided { .. }
                | c::Activity::Milestone { .. }
                | c::Activity::NowWritten { .. }
                | c::Activity::Stood { .. }
                | c::Activity::TalkOpened { .. }
                | c::Activity::TalkEnded { .. }
                | c::Activity::SessionEnded { .. }
                | c::Activity::Kept { .. }) => panic!("word line: {other:?}"),
            };
            assert_eq!(words_read.bytes(), words.as_ref());
        }
    }
}

#[test]
fn a_line_past_long_text_fails_its_page() {
    for line in word_lines(&[b'w'; 129]) {
        assert_eq!(translate::pages::activity_page(word_page(line), &line_limits()), Err(Undecodable::TooLarge));
    }
}

#[test]
fn an_ended_question_answer_past_text_decodes() {
    let limits = line_limits();
    let mut ended = fixtures::read_wire_owner_ended();
    let question = fixtures::read_wire_question();
    ended.key = w::ObjectKey::Question(w::ObjectKeyQuestion { task: 7, message: question.message });
    let answer = word_line(w::Activity::Words(w::ActivityWords {
        message: 19,
        by: w::Requester::Deployment(w::RequesterDeployment { project: 1 }),
        kind: w::MessageKind::Answer(w::MessageKindAnswer { question: question.message }),
        words: Box::new([b'a'; 128]),
    }));
    ended.value = w::EndedValue::Question(w::EndedValueQuestion { question, answer: Some(answer), ending: None });
    let ended = translate::objects::ended(ended, &limits).unwrap();
    let c::EndedValue::Question { answer: Some(answer), .. } = ended.value else { panic!("answered question") };
    let c::Activity::Words { words, .. } = answer.line else { panic!("answer words") };
    assert_eq!(words.bytes(), &[b'a'; 128]);
}

#[test]
fn a_message_and_an_answer_encode_to_long_text() {
    let limits = line_limits();
    for size in [128, 129] {
        let mut words = skein_lib::bytes::zeroed(size);
        words.fill(b'w');
        let words = c::Text::new(words, 129).unwrap();
        for ask in [
            c::Ask::Message { project: 3, task: 7, words: words.clone() },
            c::Ask::Answer { project: 3, on: c::AnswerOn::Question { task: 7, message: 1, words } },
        ] {
            let record = translate::asks::ask(&ask, limits.wire());
            if size == 129 {
                assert_eq!(record.and_then(|record| record.encode(limits.wire())).unwrap_err().reason, skein_codec::Reason::Bound);
                continue;
            }
            let record = record.unwrap();
            let encoded_words = if let w::Ask::Message(record) = &record {
                record.words.as_ref()
            } else {
                let w::Ask::Answer(record) = &record else { panic!("words ask: {record:?}") };
                let w::AnswerOn::Question(answer) = &record.on else { panic!("question answer") };
                answer.words.as_ref()
            };
            assert_eq!(encoded_words, &[b'w'; 128]);
        }
    }
}

#[test]
fn talk_words_take_the_message_bound_and_both_talk_acts_restore() {
    let limits = line_limits();
    for size in [128, 129] {
        let source =
            c::Ask::Talk { project: 3, task: 7, words: c::Text::new(skein_lib::bytes::zeroed(size), 129).unwrap() };
        let record = translate::asks::ask(&source, limits.wire());
        if size == 129 {
            assert_eq!(record.and_then(|record| record.encode(limits.wire())).unwrap_err().reason, skein_codec::Reason::Bound);
        } else {
            assert_eq!(translate::store::restore_ask(record.unwrap(), &limits).unwrap(), source);
        }
    }
    for source in [
        c::Ask::Talk { project: 3, task: 7, words: c::Text::new(b"hello".as_slice().into(), 32).unwrap() },
        c::Ask::Leave { project: 3, task: 7 },
    ] {
        let record = translate::asks::ask(&source, limits.wire()).unwrap();
        assert_eq!(translate::store::restore_ask(record, &limits).unwrap(), source);
    }
}

#[test]
fn history_rows_keep_attempts_holds_and_their_absences() {
    use skein_lib::Wall;
    let base = fixtures::limits();
    let limits = Limits::new(jig_client_domain::Limits { rows: 3, ..*base.client() }, base.exchanges()).unwrap();
    let page = translate::pages::history_page(jig_client_test_records::mixed_history_page(), &limits).unwrap();
    assert!(page.cut);
    let [c::HistoryRow::Change(change), c::HistoryRow::Attempt(attempt), c::HistoryRow::Hold(hold)] =
        page.rows.as_slice()
    else {
        panic!("change, attempt, hold in page order")
    };
    assert_eq!((change.task, change.revision), (7, 19));
    assert_eq!(
        **attempt,
        c::AttemptLine {
            task: 7,
            attempt: 2,
            claimed: Some(Wall::from_nanos(14)),
            started: Some(Wall::from_nanos(15)),
            ended: Some(Wall::from_nanos(19)),
            end: Some(c::RunEnd::Finished),
            phases: Some(3)
        }
    );
    assert_eq!(
        **hold,
        c::HoldLine {
            task: 7,
            position: 31,
            at: Wall::from_nanos(37),
            before: c::PriorPhase::Waiting,
            why: c::HoldCategory::Deadline,
            source: None,
            ending: None
        }
    );
    let absent = w::AttemptLine {
        task: 0,
        attempt: u64::MAX,
        claimed: None,
        started: None,
        ended: None,
        end: None,
        phases: None,
    };
    assert_eq!(
        translate::pages::attempt_line_fields(&absent),
        c::AttemptLine {
            task: 0,
            attempt: u64::MAX,
            claimed: None,
            started: None,
            ended: None,
            end: None,
            phases: None
        }
    );
    let mut source = jig_client_test_records::mixed_history_page();
    source.before = Some(w::Cursor { value: Box::from([0, 255]) });
    source.after = Some(w::Cursor { value: Box::from([254, 1]) });
    let page = translate::pages::history_page(source, &limits).unwrap();
    assert_eq!(page.before.unwrap().value.bytes(), &[0, 255]);
    assert_eq!(page.after.unwrap().value.bytes(), &[254, 1]);
}

#[test]
fn task_queries_carry_parts_and_pages_return_opaque_values_and_the_ended_mark() {
    let limits = fixtures::limits();
    let query = fixtures::part_query();
    let mut h = fixtures::Harness::new(&limits);
    let exchange = h.read(Readable::Jig(c::Read::Tasks { query: query.clone(), live: None, ended: None }));
    let Request::Exchange { message: Message::Read(bytes), .. } = h.out.iter().next().unwrap() else { panic!("read") };
    let owned = w::Owned::decode(limits.wire(), &mut Reader::new(bytes)).unwrap();
    let w::Read::Tasks(read) = w::Read::decode(limits.wire(), &mut Reader::new(owned.record.as_ref())).unwrap() else {
        panic!("task query")
    };
    assert_eq!(read.query.parts.len(), query.parts.len());
    for (actual, expected) in read.query.parts.iter().zip(query.parts.iter()) {
        assert_eq!(
            (actual.owner.number, actual.kind, actual.version),
            (expected.owner, expected.kind, expected.version)
        );
    }
    h.answer(
        exchange,
        &fixtures::jig_read_value_answer(&w::ReadValue::Tasks(fixtures::part_page(fixtures::row_parts()))),
    );
    let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page)))) = h.read_answered() else {
        panic!("task page")
    };
    assert_eq!(page.ended_mark, 61);
    let c::EndedGroup::Rows { rows, .. } = &page.ended else { panic!("ended rows") };
    for rows in [&page.live, rows] {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.get(0).unwrap().parts, fixtures::expected_row_parts());
    }
}

#[test]
fn mismatched_row_part_counts_and_value_envelopes_make_reads_malformed() {
    for (parts, asks) in fixtures::malformed_row_parts() {
        let mut query = fixtures::part_query();
        query.parts.truncate(asks);
        let mut h = fixtures::Harness::new(&fixtures::limits());
        let exchange = h.read(Readable::Jig(c::Read::Tasks { query, live: None, ended: None }));
        h.answer(exchange, &fixtures::jig_read_value_answer(&w::ReadValue::Tasks(fixtures::part_page(parts))));
        let Some(Readout::Jig(c::ReadResult::None(why))) = h.read_answered() else {
            panic!("malformed row was accepted")
        };
        assert_eq!(*why, c::ReadNone::Malformed);
        assert!(h.has(&Request::Fact { fact: Fact::UndecodableDocument }));
    }
}

#[test]
fn a_v28_task_page_has_empty_parts_and_zero_ended_mark() {
    use jig_client_protocol_wire as w;
    let limits = fixtures::limits();
    let rows = jig_client_test_records::tasks_rows_record().rows;
    let record = w::TaskPage {
        ended_mark: 0,
        live: rows.clone(),
        live_after: None,
        ended: w::EndedGroup::Rows(w::EndedRows { rows, after: None, cut: false }),
    };
    let mut writer = skein_lib::Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    let bytes = writer.finish();
    let record = w::TaskPage::decode(limits.receiving(), &mut Reader::new(&bytes)).unwrap();
    let page = translate::pages::task_page(record, &limits).unwrap();
    assert_eq!(page.ended_mark, 0);
    assert_eq!(page.live.len(), 1);
    assert!(page.live.get(0).unwrap().parts.is_empty());
    let c::EndedGroup::Rows { rows, .. } = page.ended else { panic!("ended rows") };
    assert_eq!(rows.len(), 1);
    assert!(rows.get(0).unwrap().parts.is_empty());
}
