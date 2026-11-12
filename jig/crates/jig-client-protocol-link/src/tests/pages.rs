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
    let mut page = jig_client_test_records::selected_inbox_pages()[0].clone().into_parts();
    let mut rows = Reader::new(&page.rows);
    let bytes = skein_codec::piece::split(&mut rows).unwrap().unwrap();
    let row = w::InboxEntry::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let mut goals = List::with_capacity(33);
    for goal in 1..=33 {
        goals.push(goal).unwrap();
    }
    let selection = w::InboxSelection::new(
        &w::CEILINGS,
        w::InboxSelectionParts { cursor: row.cursor().clone(), goals: goals.clone() },
    )
    .unwrap();
    let mut encoded = Writer::new(usize::try_from(selection.measure()).unwrap());
    selection.encode(&mut encoded).unwrap();
    let bytes = encoded.finish();
    let mut piece = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &bytes).unwrap();
    page.selection = Some(piece.finish());
    let page = w::InboxPage::new(&w::CEILINGS, page).unwrap();
    let mut encoded = Writer::new(usize::try_from(page.measure()).unwrap());
    page.encode(&mut encoded).unwrap();
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
    let offered = w::Offered::new(
        limits.wire(),
        w::OfferedParts { effect: None, act: w::ActKind::Accept, admission: w::Admission::Allowed },
    )
    .unwrap();
    let mut encoded = skein_lib::Writer::new(usize::try_from(offered.measure()).unwrap());
    offered.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut acts = skein_lib::Writer::new(5 * (encoded.len() + 4));
    for _ in 0_u32..5 {
        skein_codec::piece::write(&mut acts, &encoded).unwrap();
    }
    let mut parts = fixtures::read_wire_owner_escalation().into_parts();
    parts.acts = acts.finish();
    let record = w::Escalation::new(limits.wire(), parts).unwrap();
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
    client.document_bytes = u64::from(record.measure()).checked_sub(1).unwrap();
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
    assert_eq!(len_u64(fixtures::FULL_TURN.len()), fixtures::TURN_PAYLOAD_BYTES);
    let limits = fixtures::turn_limits(32, fixtures::TURN_PAYLOAD_BYTES);
    assert_eq!(
        translate::pages::carried_turn_fields(fixtures::FULL_TURN, &limits),
        fixtures::decoded_turn(b"abc", false)
    );
}
#[test]
fn a_turn_past_its_admission_bound_is_carried_as_too_large() {
    let limits = fixtures::turn_limits(32, fixtures::TURN_PAYLOAD_BYTES.checked_sub(1).unwrap());
    assert_eq!(
        translate::pages::carried_turn_fields(fixtures::FULL_TURN, &limits),
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
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}

#[test]
fn a_task_page_decodes_its_two_groups_with_their_cursors() {
    let limits = fixtures::limits();
    let mut page = jig_client_test_records::task_page_record().into_parts();
    page.live = jig_client_test_records::tasks_rows_record().into_parts().rows;
    page.live_after = Some(w::Cursor::new(limits.wire(), w::CursorParts { value: b"live".as_slice().into() }).unwrap());
    let after = Some(w::Cursor::new(limits.wire(), w::CursorParts { value: b"ended".as_slice().into() }).unwrap());
    page.ended = w::EndedGroup::Rows(
        w::EndedRows::new(limits.wire(), w::EndedRowsParts { rows: Box::new([]), after, cut: true }).unwrap(),
    );
    let page = translate::pages::task_page(w::TaskPage::new(limits.wire(), page).unwrap(), &limits).unwrap();
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
    let mut impact = fixtures::read_wire_impact().into_parts();
    impact.live_runs = 11;
    impact.proposals = 13;
    let effect = w::Effect::Cancel(w::Impact::new(limits.wire(), impact).unwrap());
    let value = w::Offered::new(
        limits.wire(),
        w::OfferedParts { act: w::ActKind::Cancel, admission: w::Admission::Allowed, effect: Some(effect) },
    )
    .unwrap();
    let value = translate::documents::offered_fields(&value, &limits).unwrap();
    let Some(c::Effect::Cancel(impact)) = value.effect else { panic!("cancel effect") };
    assert_eq!((impact.live_runs, impact.proposals), (11, 13));
    assert_eq!(impact.tasks, fixtures::read_value_impact().tasks);
    let effect =
        w::Effect::MoveTask(w::EffectMove::new(limits.wire(), w::EffectMoveParts { reservation: 17 }).unwrap());
    assert_eq!(translate::documents::effect_fields(&effect), c::Effect::Move { reservation: 17 });
}

#[test]
fn proposal_history_and_activity_keep_their_card_facts() {
    let limits = fixtures::limits();
    let wire = limits.wire();
    for made in [false, true] {
        let record = w::Activity::Decided(
            w::ActivityDecided::new(
                wire,
                w::ActivityDecidedParts {
                    revision: 5,
                    proposal: Some(
                        w::ActivityDecidedProposal::new(wire, w::ActivityDecidedProposalParts { number: 19, made })
                            .unwrap(),
                    ),
                },
            )
            .unwrap(),
        );
        assert_eq!(
            translate::pages::activity_fields(&record, &limits).unwrap(),
            c::Activity::Decided { revision: 5, proposal: Some(c::DecidedProposal { number: 19, made }) }
        );
    }
    let mut projects = skein_lib::List::with_capacity(1);
    projects.push(23).unwrap();
    let record = w::ProposalHistory::new(
        wire,
        w::ProposalHistoryParts {
            reason: w::DocumentText::new(wire, w::DocumentTextParts { bytes: b"why".as_slice().into(), cut: true })
                .unwrap(),
            action: w::RetainedAction::Settings(
                w::RetainedActionSettings::new(
                    wire,
                    w::RetainedActionSettingsParts {
                        projects: projects.clone(),
                        against: 3,
                        items: skein_lib::List::with_capacity(0),
                        made: Some(5),
                    },
                )
                .unwrap(),
            ),
            number: 19,
            proposer: 7,
            as_holder: false,
            state: w::ProposalHistoryState::Withdrawn,
        },
    )
    .unwrap();
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
    let requester = w::Requester::Deployment(
        w::RequesterDeployment::new(&w::CEILINGS, w::RequesterDeploymentParts { project: 1 }).unwrap(),
    );
    let word = w::Word::new(
        &w::CEILINGS,
        w::WordParts {
            number: 1,
            from: requester.clone(),
            kind: w::MessageKind::Words,
            words: Box::from(words),
            at: 2,
            hits: 1,
            eligible: true,
        },
    )
    .unwrap();
    [
        w::Activity::Words(
            w::ActivityWords::new(
                &w::CEILINGS,
                w::ActivityWordsParts {
                    message: 1,
                    by: requester.clone(),
                    kind: w::MessageKind::Words,
                    words: Box::from(words),
                },
            )
            .unwrap(),
        ),
        w::Activity::Heard(w::ActivityHeard::new(&w::CEILINGS, w::ActivityHeardParts { value: word.clone() }).unwrap()),
        w::Activity::Notified(
            w::ActivityNotified::new(&w::CEILINGS, w::ActivityNotifiedParts { recipient: requester, word }).unwrap(),
        ),
    ]
}

fn word_line(line: w::Activity) -> w::ActivityLine {
    w::ActivityLine::new(&w::CEILINGS, w::ActivityLineParts { task: 7, position: 1, at: 2, line }).unwrap()
}

fn word_page(line: w::Activity) -> w::ActivityPage {
    let row = w::DecodedActivityLine::new(
        &w::CEILINGS,
        w::DecodedActivityLineParts { source: word_line(line), decoded: None },
    )
    .unwrap();
    let mut encoded = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut piece = skein_lib::Writer::new(encoded.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &encoded).unwrap();
    w::ActivityPage::new(
        &w::CEILINGS,
        w::ActivityPageParts { rows: piece.finish(), before: None, after: None, cut: false },
    )
    .unwrap()
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
    let mut ended = fixtures::read_wire_owner_ended().into_parts();
    let question = fixtures::read_wire_question();
    ended.key = w::ObjectKey::Question(
        w::ObjectKeyQuestion::new(&w::CEILINGS, w::ObjectKeyQuestionParts { task: 7, message: question.message() })
            .unwrap(),
    );
    let answer = word_line(w::Activity::Words(
        w::ActivityWords::new(
            &w::CEILINGS,
            w::ActivityWordsParts {
                message: 19,
                by: w::Requester::Deployment(
                    w::RequesterDeployment::new(&w::CEILINGS, w::RequesterDeploymentParts { project: 1 }).unwrap(),
                ),
                kind: w::MessageKind::Answer(
                    w::MessageKindAnswer::new(&w::CEILINGS, w::MessageKindAnswerParts { question: question.message() })
                        .unwrap(),
                ),
                words: Box::new([b'a'; 128]),
            },
        )
        .unwrap(),
    ));
    ended.value = w::EndedValue::Question(
        w::EndedValueQuestion::new(
            &w::CEILINGS,
            w::EndedValueQuestionParts { question, answer: Some(answer), ending: None },
        )
        .unwrap(),
    );
    let ended = translate::objects::ended(w::Ended::new(&w::CEILINGS, ended).unwrap(), &limits).unwrap();
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
                assert_eq!(record.unwrap_err().reason, skein_codec::Reason::Bound);
                continue;
            }
            let record = record.unwrap();
            let encoded_words = if let w::Ask::Message(record) = &record {
                record.words()
            } else {
                let w::Ask::Answer(record) = &record else { panic!("words ask: {record:?}") };
                let w::AnswerOn::Question(answer) = record.on() else { panic!("question answer") };
                answer.words()
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
            assert_eq!(record.unwrap_err().reason, skein_codec::Reason::Bound);
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
