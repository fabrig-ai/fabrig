use super::fixtures;
use crate::{Unencoded, translate};
use jig_domain_link::{CountKind, Hosting, Phase, SourceName};
use jig_protocol_worker_wire::Record;
use skein_lib::{Duration, Token};
fn translated(record: translate::Up) -> (jig_domain_host::LinkUp, translate::Parts) {
    let mut parts = translate::Parts::new(4);
    let link = translate::up(record, &mut parts);
    (link, parts)
}
fn up(record: translate::Up) -> jig_domain_host::LinkUp {
    let (link, parts) = translated(record);
    assert!(parts.is_empty(), "this record has no application parts");
    link
}
fn answer(record: jig_protocol_worker_wire::Answer) -> (jig_domain_host::RunAnswer, translate::Parts) {
    let (link, parts) = translated(translate::Up::Answer(record));
    let jig_domain_host::LinkUp::Answer { run, attempt, answer } = link else {
        panic!("an answer");
    };
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    (answer, parts)
}
#[test]
fn a_hello_record_becomes_the_hubs_hello_with_its_bounds() {
    let hello = translate::hello(fixtures::hello_record()).expect("valid hello");
    assert_eq!(hello.slots, 1);
    assert_eq!(hello.stop_bound, Duration::from_secs(20));
    assert_eq!(hello.workstreams.as_ref(), &[9]);
    assert_eq!(
        hello.hosting.as_ref(),
        &[Hosting { run: Token::new(7), attempt: Token::new(2), phase: Phase::Waiting }]
    );
    let model = hello.bounds.endpoints.first().expect("one model");
    assert_eq!(model.endpoint, 1);
    assert_eq!(model.window, 8192);
    assert_eq!(model.output, 1024);
    assert_eq!(hello.bounds.turns, 8);
    assert_eq!(hello.bounds.spend, 100);
    assert_eq!(hello.bounds.time, Duration::from_secs(60));
    assert_eq!(hello.bounds.waiting, Duration::from_secs(10));
    assert_eq!(hello.bounds.checks, Duration::from_secs(2));
    assert_eq!(hello.bounds.message_bytes, 16);
    assert_eq!(hello.bounds.inbox, 1);
    assert_eq!(hello.bounds.unacknowledged, 1);
}

#[test]
fn a_turn_record_becomes_the_hubs_turn_with_its_refusals() {
    let link = up(translate::Up::Turn(fixtures::turn_record()));
    let jig_domain_host::LinkUp::Turn { run, attempt, turn } = link else {
        panic!("a turn");
    };
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert!(turn.opens_window, "window marker kept");
    assert_eq!(
        turn.refused.as_ref(),
        &[jig_domain_host::Refused { name: Token::new(8), reason: jig_domain_host::MessageRefusal::Full }]
    );
    assert_eq!(turn.read, Some(Token::new(9)));
    assert_eq!(turn.body.as_ref(), b"opaque-turn");
}
#[test]
fn a_fact_record_becomes_the_hubs_activity() {
    assert_eq!(
        up(translate::Up::Fact(fixtures::fact_record())),
        jig_domain_host::LinkUp::Activity {
            run: Token::new(7),
            attempt: Token::new(2),
            activity: jig_domain_host::AgentActivity::Count {
                kind: CountKind::LlmStarted,
                elapsed: Duration::from_secs(2),
                count: 3
            }
        }
    );
}
#[test]
fn each_account_notice_record_becomes_its_link_notice() {
    let (run, attempt) = (Token::new(7), Token::new(2));
    assert_eq!(
        up(translate::Up::Unauthorized(fixtures::unauthorized_record())),
        jig_domain_host::LinkUp::Unauthorized { run, attempt, account: 1, generation: 4 }
    );
    for reset in [Some(Duration::from_secs(5)), None] {
        assert_eq!(
            up(translate::Up::Limited(fixtures::limited_record(reset))),
            jig_domain_host::LinkUp::Limited { run, attempt, account: 1, reset }
        );
    }
    assert_eq!(
        up(translate::Up::AccountRefused(fixtures::account_refused_record())),
        jig_domain_host::LinkUp::AccountRefused {
            run,
            attempt,
            account: 1,
            code: Box::from(b"forbidden".as_slice()),
            reason: Box::from(b"suspended".as_slice()),
        }
    );
}

#[test]
fn a_failed_answer_keeps_the_limited_or_refused_account_it_turned_on() {
    use jig_domain_host::RunFailure;
    for (record, failure) in [
        (
            fixtures::limited_account(Some(Duration::from_secs(5))),
            RunFailure::Limited { account: 1, reset: Some(Duration::from_secs(5)) },
        ),
        (fixtures::limited_account(None), RunFailure::Limited { account: 1, reset: None }),
        (fixtures::refused_account(), RunFailure::Refused { account: 1 }),
    ] {
        let (answer, _parts) = answer(fixtures::answer_failed_on(record));
        let jig_domain_host::Ending::Failed { failure: found, .. } = answer.ending else {
            panic!("failed");
        };
        assert_eq!(found, jig_domain_host::Failure::Run(failure));
    }
}

#[test]
fn an_untooled_answer_keeps_its_failure_detail_and_work_from_bytes() {
    use jig_protocol_worker_wire as wire;
    let record = fixtures::answer_failed_on(wire::RunFailure::Untooled);
    let mut writer = skein_lib::Writer::new(skein_lib::usize_of(
        record.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"),
    ));
    record.write(&jig_protocol_worker_wire::CEILINGS, &mut writer).expect("answer bytes");
    let bytes = writer.finish();
    let record = wire::Answer::decode(&codec(), &mut skein_lib::Reader::new(&bytes)).expect("answer");
    let (answer, parts) = answer(record);
    assert_eq!(
        answer.ending,
        jig_domain_host::Ending::Failed {
            failure: jig_domain_host::Failure::Run(jig_domain_host::RunFailure::Untooled),
            detail: b"detail".as_slice().into(),
            work: jig_domain_host::Work { left: Some(Token::new(4)), saved: Some(Token::new(5)) },
            model: None,
        }
    );
    let parts = parts.finish();
    assert_eq!(parts[0].bytes.as_ref(), b"left");
    assert_eq!(parts[1].bytes.as_ref(), b"saved");
}

#[test]
fn an_answer_record_keeps_its_unread_fence() {
    let (answer, parts) = answer(fixtures::answer_record());
    drop(parts);
    assert_eq!(answer.read, Some(Token::new(9)));
    assert_eq!(
        answer.refused.as_ref(),
        &[jig_domain_host::Refused { name: Token::new(8), reason: jig_domain_host::MessageRefusal::Full }]
    );
}
#[test]
fn an_answer_hands_left_and_saved_work_up_as_parts() {
    let (answer, parts) = answer(fixtures::answer_record());
    let jig_domain_host::Ending::Ended { work, .. } = answer.ending else {
        panic!("ended");
    };
    assert_eq!(work.left, Some(Token::new(4)));
    assert_eq!(work.saved, Some(Token::new(5)));
    assert!(!parts.is_empty(), "application bytes extracted");
    assert_eq!(parts.next(), 6);
    let parts = parts.finish();
    assert_eq!(parts.first().expect("left part").bytes.as_ref(), b"left");
    assert_eq!(parts.get(1).expect("saved part").bytes.as_ref(), b"saved");
}
#[test]
fn a_call_record_keeps_its_name_bytes() {
    let mut parts = fixtures::call_record();
    parts.contract = 3;
    let record = parts;
    let link = up(translate::Up::Call(record));
    let jig_domain_host::LinkUp::Call { run, attempt, contract, call, tool, writes, input, deadline } = link else {
        panic!("call");
    };
    assert_eq!(contract, 3);
    assert_eq!(call.as_ref(), &[0, 2, 7]);
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert_eq!(tool.as_ref(), b"read");
    assert!(!writes, "write bit kept");
    assert_eq!(input.as_ref(), b"input");
    assert_eq!(deadline, Duration::from_secs(4));
}

#[test]
fn a_failed_answers_resource_precedes_its_left_and_saved_parts() {
    let (answer, parts) = answer(fixtures::answer_with_resource());
    let jig_domain_host::Ending::Failed { failure, detail, work, .. } = answer.ending else {
        panic!("failed");
    };
    assert_eq!(
        failure,
        jig_domain_host::Failure::Unprepared(jig_domain_host::Preparation::Permanent { resource: Some(Token::new(4)) })
    );
    assert_eq!(detail.as_ref(), b"detail");
    assert_eq!(work.left, Some(Token::new(5)));
    assert_eq!(work.saved, Some(Token::new(6)));
    assert_eq!(parts.next(), 7);
    let parts = parts.finish();
    assert_eq!(parts[0].bytes.as_ref(), b"resource");
    assert_eq!(parts[1].bytes.as_ref(), b"left");
    assert_eq!(parts[2].bytes.as_ref(), b"saved");
}

fn down(record: jig_domain_host::LinkDown) -> translate::Down {
    use jig_domain_host::LinkDown;
    let translated = match record {
        LinkDown::ReadList { .. } | LinkDown::Assign { .. } | LinkDown::Grant { .. } => {
            translate::down(record, &fixtures::parts(), fixtures::values(), &codec())
        }
        LinkDown::Sources { .. }
        | LinkDown::Check { .. }
        | LinkDown::SignOut { .. }
        | LinkDown::Message { .. }
        | LinkDown::Asked { .. }
        | LinkDown::Relayed { .. }
        | LinkDown::TurnKept { .. }
        | LinkDown::Acknowledge { .. }
        | LinkDown::AssignJob { .. }
        | LinkDown::Cancel { .. } => translate::down(record, &[], Box::new([]), &codec()),
    };
    translated.expect("complete output")
}
fn codec() -> jig_protocol_worker_wire::v18::Limits {
    jig_protocol_worker_wire::sizes::limits(&fixtures::SIZES).expect("small codec")
}
fn assignment_output() -> jig_domain_host::LinkDown {
    jig_domain_host::LinkDown::Assign { assignment: fixtures::assignment() }
}
fn assigned() -> jig_protocol_worker_wire::Assignment {
    assignment_record(down(assignment_output()))
}
fn assignment_record(record: translate::Down) -> jig_protocol_worker_wire::Assignment {
    let translate::Down::Assign(record) = record else {
        panic!("assignment");
    };
    record
}
#[test]
fn an_assignment_keeps_its_start_messages_and_identity() {
    let fields = assigned();
    assert_eq!((fields.task, fields.attempt), (7, 2));
    assert_eq!(fields.identity, fixtures::IDENTITY);
    assert_eq!(fields.messages.len(), 1);
    let message = &fields.messages.get(0).expect("message");
    assert_eq!(message.name, 8);
    assert_eq!(message.sender.as_ref(), b"sender");
    assert_eq!(message.words.as_ref(), b"message");
    assert!(message.finish_requested);
}
#[test]
fn an_assignments_items_come_from_their_part() {
    let fields = assigned();
    let workspace = fields.workspace.expect("workspace");
    assert_eq!(workspace.workstream, 9);
    assert_eq!(workspace.items.as_ref(), b"items");
    assert!(fields.save);
}
#[test]
fn a_missing_workspace_part_drops_the_assignment() {
    assert_eq!(
        translate::down(assignment_output(), &[], fixtures::values(), &codec()).expect_err("missing part"),
        Unencoded::field("workspace.items")
    );
}
#[test]
fn an_assignments_window_is_one_piece_with_each_turns_length() {
    assert_eq!(assigned().transcript.as_ref(), &[0, 0, 0, 2, 0, 127, 0, 0, 0, 3, 255, 1, 2]);
}
#[test]
fn a_transcript_past_its_bound_including_lengths_drops_the_assignment() {
    let mut limits = codec();
    limits.assignment_transcript = 12;
    assert_eq!(
        translate::down(assignment_output(), &fixtures::parts(), fixtures::values(), &limits)
            .expect_err("transcript bound"),
        Unencoded {
            field: "Assignment.transcript",
            reason: None,
            figure: Some(13),
            bound: Some(12),
            ceiling: Some(u64::from(jig_protocol_worker_wire::v18::CEILINGS.assignment_transcript)),
        }
    );
}
#[test]
fn a_charter_past_its_bound_names_its_bytes_limit_and_ceiling() {
    let mut limits = codec();
    limits.assignment_charter = 6;
    assert_eq!(
        translate::down(assignment_output(), &fixtures::parts(), fixtures::values(), &limits)
            .expect_err("charter bound"),
        Unencoded {
            field: "Assignment.charter",
            reason: None,
            figure: Some(7),
            bound: Some(6),
            ceiling: Some(u64::from(jig_protocol_worker_wire::v18::CEILINGS.assignment_charter))
        }
    );
    limits.assignment_charter = 7;
    translate::down(assignment_output(), &fixtures::parts(), fixtures::values(), &limits).expect("charter at bound");
}
#[test]
fn an_encoding_refusal_keeps_its_codec_path_and_reason() {
    let mut assignment = fixtures::assignment();
    assignment.messages[0].words = Box::from([0xff]);
    assert_eq!(
        translate::down(
            jig_domain_host::LinkDown::Assign { assignment },
            &fixtures::parts(),
            fixtures::values(),
            &codec()
        )
        .expect("translated fields")
        .frame(&codec())
        .expect_err("not UTF-8"),
        Unencoded {
            field: jig_protocol_worker_wire::v18::name(jig_protocol_worker_wire::v18::paths::MESSAGE_WORDS)
                .expect("generated path"),
            reason: Some(skein_codec::Reason::Utf8),
            figure: None,
            bound: None,
            ceiling: None
        }
    );
}
#[test]
fn a_full_list_names_its_count_limit_and_ceiling() {
    let mut limits = codec();
    limits.assignment_messages = 0;
    assert_eq!(
        translate::down(assignment_output(), &fixtures::parts(), fixtures::values(), &limits)
            .expect_err("message count"),
        Unencoded {
            field: "Assignment.messages",
            reason: None,
            figure: Some(1),
            bound: Some(0),
            ceiling: Some(u64::from(jig_protocol_worker_wire::v18::CEILINGS.assignment_messages))
        }
    );
}
#[test]
fn repeated_source_names_name_the_encoders_field() {
    let name = program_name();
    assert_eq!(
        translate::down(
            jig_domain_host::LinkDown::Sources { sources: Box::new([name, name]) },
            &[],
            Box::new([]),
            &codec()
        )
        .expect_err("repeated name"),
        Unencoded::field("sources.names")
    );
}
#[test]
fn a_fresh_assignment_has_an_empty_transcript_piece() {
    let mut assignment = fixtures::assignment();
    assignment.turns = Box::new([]);
    let record = assignment_record(down(jig_domain_host::LinkDown::Assign { assignment }));
    assert_eq!((record.transcript.as_ref()).len(), 0);
}
#[test]
fn a_grant_without_a_value_is_left_out() {
    let record = assignment_record(
        translate::down(
            assignment_output(),
            &fixtures::parts(),
            Box::new([None, fixtures::values()[1].take()]),
            &codec(),
        )
        .expect("assignment"),
    );
    assert_eq!(record.grants.len(), 1);
    let jig_protocol_worker_wire::GrantValue::Application(value) = &assignment_grant(&record).value else {
        panic!("application value")
    };
    assert_eq!(value.secret.expose(), fixtures::TOKEN_TWO);
}
fn granted(form: jig_domain_link::Form, value: crate::GrantValue) -> Result<translate::Down, Unencoded> {
    translate::down(
        jig_domain_host::LinkDown::Grant { run: Token::new(7), attempt: Token::new(2), grant: fixtures::grant(form) },
        &[],
        Box::new([Some(value)]),
        &codec(),
    )
}
fn granted_value(down: Result<translate::Down, Unencoded>) -> jig_protocol_worker_wire::Grant {
    let Ok(translate::Down::Grant(record)) = down else { panic!("granted") };
    assert_eq!((record.task, record.attempt), (7, 2));
    assert_eq!((record.grant.number, record.grant.generation), (1, 4));
    record.grant.clone()
}
#[test]
fn each_credential_scheme_reaches_the_wire_with_its_token() {
    use jig_domain_link::{Form, Scheme};
    use jig_protocol_worker_wire as wire;
    for (form, scheme, expected) in [
        (Scheme::Bearer, crate::Scheme::Bearer, wire::Scheme::Bearer),
        (Scheme::Key, crate::Scheme::Key, wire::Scheme::Key),
    ] {
        for (read, kind) in [(false, wire::Kind::Grant), (true, wire::Kind::ReadList)] {
            let metadata = fixtures::grant(Form::Credential(form));
            let down = if read {
                jig_domain_host::LinkDown::ReadList { read: 7, endpoint: 3, grant: metadata }
            } else {
                jig_domain_host::LinkDown::Grant { run: Token::new(7), attempt: Token::new(2), grant: metadata }
            };
            assert_eq!(crate::host::grants(&down), &[metadata]);
            let frame = translate::down(down, &[], Box::new([Some(fixtures::credential_value(scheme))]), &codec())
                .expect("credential record")
                .frame(&codec())
                .unwrap();
            assert_eq!(frame.kind(), kind.number());
            let mut reader = skein_lib::Reader::new(frame.bytes().get(8..).expect("frame body"));
            let grant = if read {
                let record = wire::ReadList::decode(&codec(), &mut reader).expect("read");
                assert_eq!((record.read, record.endpoint), (7, 3));
                record.grant
            } else {
                let record = wire::Granted::decode(&codec(), &mut reader).expect("grant");
                assert_eq!((record.task, record.attempt), (7, 2));
                record.grant
            };
            assert_eq!(
                (grant.number, grant.generation, grant.valid),
                (metadata.account, metadata.generation, metadata.valid)
            );
            let wire::GrantValue::Credential(value) = &grant.value else { panic!("credential value") };
            assert_eq!((&value.scheme), &expected);
            assert_eq!(value.bytes.expose(), fixtures::TOKEN_ONE);
        }
    }
}
#[test]
fn an_application_grant_reaches_the_wire_with_its_secret() {
    let value = fixtures::values()[1].take().expect("application value");
    let grant = granted_value(granted(jig_domain_link::Form::Application, value));
    let jig_protocol_worker_wire::GrantValue::Application(value) = &grant.value else { panic!("application value") };
    assert_eq!(value.secret.expose(), fixtures::TOKEN_TWO);
}
#[test]
fn a_value_unlike_its_grants_form_is_not_sent() {
    use jig_domain_link::{Form, Scheme};
    let bearer = Form::Credential(Scheme::Bearer);
    let key = Form::Credential(Scheme::Key);
    for (form, value) in [
        (key, fixtures::credential_value(crate::Scheme::Bearer)),
        (bearer, fixtures::credential_value(crate::Scheme::Key)),
        (bearer, fixtures::values()[1].take().expect("application")),
        (key, fixtures::values()[1].take().expect("application")),
        (Form::Application, fixtures::credential_value(crate::Scheme::Bearer)),
        (Form::Application, fixtures::credential_value(crate::Scheme::Key)),
    ] {
        assert_eq!(
            granted(form, value).expect_err("value unlike form"),
            Unencoded::field("grant.value"),
            "value unlike {form:?}"
        );
    }
}
#[test]
fn a_standalone_grant_without_a_value_is_not_sent() {
    assert_eq!(
        translate::down(
            jig_domain_host::LinkDown::Grant {
                run: Token::new(7),
                attempt: Token::new(2),
                grant: fixtures::grant(jig_domain_link::Form::Credential(jig_domain_link::Scheme::Bearer)),
            },
            &[],
            Box::new([None]),
            &codec(),
        )
        .expect_err("missing standalone value"),
        Unencoded::field("grant.value")
    );
}
#[test]
fn a_down_event_debug_shows_value_lengths_only() {
    let text = alloc::fmt::format(format_args!("{:?}", fixtures::assignment_down(Token::new(1))));
    for secret in [fixtures::TOKEN_ONE, fixtures::TOKEN_TWO] {
        for window in text.as_bytes().windows(secret.len()) {
            assert_ne!(window, secret);
        }
        assert!(!text.contains(&alloc::fmt::format(format_args!("{secret:?}"))));
    }
    assert!(text.contains("Secret"));
}
#[test]
fn a_relayed_busy_answer_carries_no_text() {
    let translate::Down::Relayed(record) = down(jig_domain_host::LinkDown::Relayed {
        run: Token::new(7),
        attempt: Token::new(2),
        call: Box::from([0, 2, 7]),
        answer: jig_domain_host::Relayed::Busy,
    }) else {
        panic!("relayed");
    };
    assert_eq!((record.task, record.attempt), (7, 2));
    assert_eq!(record.name.as_ref(), &[0, 2, 7]);
    assert_eq!((&record.answer), &jig_protocol_worker_wire::RelayAnswer::Busy);
}
#[test]
fn a_turn_kept_record_names_the_turn() {
    let translate::Down::TurnKept(record) =
        down(jig_domain_host::LinkDown::TurnKept { run: Token::new(7), attempt: Token::new(2), turn: 3 })
    else {
        panic!("kept");
    };
    assert_eq!((record.task, record.attempt, record.number), (7, 2, 3));
}
#[test]
fn an_assignments_committed_call_keeps_its_answer() {
    let fields = assigned();
    assert_eq!(fields.answered.len(), 1);
    let call = answered_call(&fields.answered);
    assert_eq!(call.name.as_ref(), &[0, 2, 7]);
    assert_eq!(call.tool.as_ref(), b"read");
    let jig_protocol_worker_wire::SettledAnswer::Host(answer) = &call.answer else {
        panic!("host answer");
    };
    assert!(answer.error);
    assert_eq!(answer.body.as_ref(), b"answer");
}

#[test]
fn a_message_record_keeps_its_name_sender_and_words() {
    for finish_requested in [false, true] {
        let translate::Down::Message(record) = down(jig_domain_host::LinkDown::Message {
            run: Token::new(7),
            attempt: Token::new(2),
            name: Token::new(8),
            sender: Box::from(b"sender".as_slice()),
            words: Box::from(b"message".as_slice()),
            finish_requested,
        }) else {
            panic!("message");
        };
        assert_eq!((record.task, record.attempt, record.name), (7, 2, 8));
        assert_eq!(record.sender.as_ref(), b"sender");
        assert_eq!(record.words.as_ref(), b"message");
        assert_eq!(record.finish_requested, finish_requested);
    }
}
#[test]
fn a_relayed_answer_keeps_its_text_and_error() {
    let translate::Down::Relayed(record) = down(jig_domain_host::LinkDown::Relayed {
        run: Token::new(7),
        attempt: Token::new(2),
        call: Box::from([0, 2, 7]),
        answer: jig_domain_host::Relayed::Answer { error: true, text: Box::from(b"error".as_slice()) },
    }) else {
        panic!("relayed");
    };
    let jig_protocol_worker_wire::RelayAnswer::Text(answer) = &record.answer else {
        panic!("text");
    };
    assert!(answer.error);
    assert_eq!(answer.text.as_ref(), b"error");
}
#[test]
fn an_acknowledgement_names_its_attempt() {
    let translate::Down::Acknowledge(record) =
        down(jig_domain_host::LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(2) })
    else {
        panic!("acknowledge");
    };
    assert_eq!((record.task, record.attempt), (7, 2));
}
#[test]
fn a_cancel_names_its_attempt() {
    let translate::Down::Cancel(record) =
        down(jig_domain_host::LinkDown::Cancel { run: Token::new(7), attempt: Token::new(2) })
    else {
        panic!("cancel");
    };
    assert_eq!((record.task, record.attempt), (7, 2));
}
#[test]
fn each_committed_delivery_outcome_keeps_its_evidence() {
    use jig_domain_host as host;
    use jig_protocol_worker_wire as wire;
    for (input, expected) in [
        (host::DeliveryOutcome::Delivered, wire::DeliveryOutcome::Delivered),
        (host::DeliveryOutcome::Nothing, wire::DeliveryOutcome::Nothing),
        (host::DeliveryOutcome::Stale, wire::DeliveryOutcome::Stale),
        (host::DeliveryOutcome::Refused, wire::DeliveryOutcome::Refused),
        (host::DeliveryOutcome::Failed, wire::DeliveryOutcome::Failed),
    ] {
        let mut assignment = fixtures::assignment();
        assignment.answered[0].answer =
            host::SettledAnswer::Delivery { outcome: input, evidence: Box::from([255, 0, 127]) };
        let record = assignment_record(down(host::LinkDown::Assign { assignment }));
        let call = answered_call(&record.answered);
        let wire::SettledAnswer::Delivery(answer) = &call.answer else {
            panic!("delivery");
        };
        assert_eq!((&answer.outcome), &expected);
        assert_eq!(answer.evidence.as_ref(), &[255, 0, 127]);
    }
}

fn assignment_grant(record: &jig_protocol_worker_wire::Assignment) -> &jig_protocol_worker_wire::Grant {
    record.grants.get(0).expect("grant")
}
#[test]
fn a_turn_busy_record_names_the_turn() {
    let translate::Down::TurnBusy(record) = translate::turn_busy(Token::new(7), Token::new(2), 3) else {
        panic!("busy");
    };
    assert_eq!((record.task, record.attempt, record.number), (7, 2, 3));
}

fn answered_call(
    calls: &skein_lib::List<jig_protocol_worker_wire::AnsweredCall>,
) -> &jig_protocol_worker_wire::AnsweredCall {
    calls.get(0).expect("call")
}

#[test]
fn an_invalid_start_answer_keeps_its_words_in_the_hub() {
    use jig_protocol_worker_wire as wire;
    let start = wire::InvalidStart { cause: wire::StartCause::Grants, why: b"budget too small".as_slice().into() };
    let invalid = wire::InvalidValue { value: wire::Invalid::Start(start) };
    let refusal = wire::RefusalValue { value: wire::Refusal::Invalid(invalid) };
    let mut fields = fixtures::answer_record();
    fields.ending = wire::Ending::Refused(refusal);
    let record = fields;
    let (answer, _) = answer(record);
    assert_eq!(
        answer.ending,
        jig_domain_host::Ending::Refused(jig_domain_host::Refusal::Invalid(jig_domain_host::Invalid::Start {
            cause: jig_domain_host::StartCause::Grants,
            why: b"budget too small".as_slice().into()
        },))
    );
}

fn program_name() -> SourceName {
    SourceName::new(b"claude").expect("source name")
}
#[test]
fn a_program_grant_crosses_as_its_name() {
    let name = program_name();
    let grant = granted_value(granted(jig_domain_link::Form::Program(name), crate::GrantValue::Program { name }));
    let jig_protocol_worker_wire::GrantValue::Program(value) = &grant.value else { panic!("program value") };
    assert_eq!(value.name.as_ref(), b"claude");
}
#[test]
fn another_value_under_a_program_form_is_not_sent() {
    for value in [
        fixtures::credential_value(crate::Scheme::Bearer),
        fixtures::credential_value(crate::Scheme::Key),
        fixtures::values()[1].take().expect("application"),
    ] {
        assert_eq!(
            granted(jig_domain_link::Form::Program(program_name()), value).expect_err("program value"),
            Unencoded::field("grant.value")
        );
    }
}
#[test]
fn a_program_name_unlike_its_form_s_is_not_sent() {
    let name = SourceName::new(b"other").expect("source name");
    assert_eq!(
        granted(jig_domain_link::Form::Program(program_name()), crate::GrantValue::Program { name })
            .expect_err("program name"),
        Unencoded::field("grant.value")
    );
}
#[test]
fn a_program_value_under_another_form_is_not_sent() {
    use jig_domain_link::{Form, Scheme};
    for form in [Form::Credential(Scheme::Bearer), Form::Credential(Scheme::Key), Form::Application] {
        assert_eq!(
            granted(form, crate::GrantValue::Program { name: program_name() }).expect_err("program under another form"),
            Unencoded::field("grant.value")
        );
    }
}

#[test]
fn a_duplicate_program_endpoint_hello_is_refused_before_delivery() {
    let mut hello = fixtures::hello_record();
    let mut bounds = hello.bounds;
    let mut endpoints = skein_lib::List::with_capacity(2);
    endpoints.push(7).unwrap();
    endpoints.push(7).unwrap();
    bounds.program_endpoints = endpoints;
    hello.bounds = bounds;

    assert!(translate::hello(hello).is_none());
}

#[test]
fn an_assignment_encodes_each_live_ask_without_changing_it() {
    use jig_domain_host as host;
    for ask in [
        host::LiveAsk::NONE,
        host::LiveAsk { text: true, reasoning: false, input: false },
        host::LiveAsk { text: false, reasoning: true, input: false },
        host::LiveAsk { text: false, reasoning: false, input: true },
        host::LiveAsk { text: true, reasoning: true, input: true },
    ] {
        let mut assignment = fixtures::assignment();
        assignment.assignment.live = ask;
        let record = assignment_record(down(host::LinkDown::Assign { assignment }));
        let mut bytes = skein_lib::Writer::new(
            usize::try_from(record.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"))
                .unwrap(),
        );
        record.write(&jig_protocol_worker_wire::CEILINGS, &mut bytes).unwrap();
        let bytes = bytes.finish();
        let record =
            jig_protocol_worker_wire::Assignment::decode(&codec(), &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!((record.live.text, record.live.reasoning, record.live.input), (ask.text, ask.reasoning, ask.input));
    }
}
#[test]
fn every_piece_kind_decodes_whole_including_offsets_past_u32() {
    use jig_domain_host as host;
    use jig_protocol_worker_wire as wire;
    let limits = codec();
    let call = wire::InputCall { id: Box::from(&b"call-id"[..]), tool: Box::from(&b"tool"[..]) };
    for (kind, expected) in [
        (wire::LiveKind::Text, host::LiveKind::Text),
        (wire::LiveKind::Reasoning, host::LiveKind::Reasoning),
        (
            wire::LiveKind::Input(call),
            host::LiveKind::Input { call: Box::from(&b"call-id"[..]), tool: Box::from(&b"tool"[..]) },
        ),
    ] {
        let piece = wire::Piece {
            turn: 17,
            retry: 23,
            kind,
            offset: u64::from(u32::MAX) + 19,
            bytes: Box::from([0xff, 0, 0x80]),
        };
        let fact = wire::Fact { task: 83, attempt: 97, value: wire::FactValue::Piece(piece) };
        let mut bytes = skein_lib::Writer::new(
            usize::try_from(fact.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits")).unwrap(),
        );
        fact.write(&jig_protocol_worker_wire::CEILINGS, &mut bytes).unwrap();
        let decoded = translate::decode(wire::Kind::Fact.number(), &bytes.finish(), &limits).unwrap();
        assert_eq!(
            up(decoded),
            host::LinkUp::Piece {
                run: Token::new(83),
                attempt: Token::new(97),
                piece: host::Piece {
                    turn: 17,
                    retry: 23,
                    kind: expected,
                    offset: u64::from(u32::MAX) + 19,
                    bytes: Box::from([0xff, 0, 0x80])
                }
            }
        );
    }
}

#[cfg(test)]
mod list_tests {
    use crate::tests::fixtures;
    use crate::{
        Event, Request,
        connection::{Connection, Phase},
    };
    use alloc::boxed::Box;
    use jig_domain_host as host;
    use jig_domain_link::{Form, ListFailure, ListOutcome, Scheme, SourceName};
    use skein_lib::Secret;
    use skein_lib::{Queue, Time, Token};

    #[test]
    fn a_read_without_a_matching_credential_answers_agent_without_sending() {
        let name = SourceName::new(b"source").expect("source name");
        for (form, value) in [
            (Form::Credential(Scheme::Bearer), None), // The requested generation is no longer held.
            (
                Form::Application,
                Some(crate::GrantValue::Application { secret: Secret::new(Box::from(b"canary".as_slice())) }),
            ),
            (Form::Program(name), Some(crate::GrantValue::Program { name })),
            (
                Form::Credential(Scheme::Bearer),
                Some(crate::GrantValue::Application { secret: Secret::new(Box::from(b"canary".as_slice())) }),
            ),
            (
                Form::Credential(Scheme::Bearer),
                Some(crate::GrantValue::Credential {
                    scheme: crate::Scheme::Key,
                    token: Secret::new(Box::from(b"canary".as_slice())),
                }),
            ),
        ] {
            let mut component = fixtures::component();
            let channel = Token::new(1);
            let mut state = Connection::new(
                crate::limits::schema(&fixtures::LIMITS).expect("schema"),
                &fixtures::LIMITS,
                Time::ZERO,
                false,
            );
            state.channel = channel;
            state.phase = Phase::Open;
            state.heard = true;
            component.connections.insert(Token::new(2), state).expect("connection slot");
            let mut out = Queue::uncapped();
            component.down(
                Time::ZERO,
                Event::Down {
                    channel,
                    down: host::LinkDown::ReadList { read: 7, endpoint: 3, grant: fixtures::grant(form) },
                    parts: Box::new([]),
                    values: Box::new([value]),
                },
                &mut out,
            );
            let Request::Up { channel: heard, up, parts } = out.pop().expect("immediate refusal") else {
                panic!("a read refusal goes upward");
            };
            assert_eq!(heard, channel);
            assert_eq!(up, host::LinkUp::ListRead { read: 7, outcome: ListOutcome::Failed(ListFailure::Agent) });
            assert!(parts.is_empty());
            assert!(out.is_empty(), "a refused read has no lower request");
        }
    }
}

#[test]
fn list_answers_decode_rows_and_every_failure() {
    use jig_domain_link::{ListFailure, ListOutcome, ListedModel};
    use jig_protocol_worker_wire as wire;
    for (body, outcome) in [
        ([0, 0, 0, 0, 0].as_slice(), ListOutcome::Listed(Box::new([]))),
        (
            [
                0, 0, 0, 0, 1, 0, 0, 0, 1, b'm', 1, 0, 0, 0, 1, b'M', 1, 0, 0, 0, 10, 1, 0, 0, 0, 5, 0, 0, 0, 1, 0, 0,
                0, 3, b'l', b'o', b'w',
            ]
            .as_slice(),
            ListOutcome::Listed(Box::new([ListedModel {
                name: b"m".as_slice().into(),
                shown: Some(b"M".as_slice().into()),
                window: Some(10),
                output: Some(5),
                efforts: Box::new([b"low".as_slice().into()]),
            }])),
        ),
        ([1, 0, 1, 173].as_slice(), ListOutcome::Failed(ListFailure::Refused { status: 429 })),
        ([1, 1].as_slice(), ListOutcome::Failed(ListFailure::Unreachable)),
        ([1, 2].as_slice(), ListOutcome::Failed(ListFailure::Unreadable)),
        ([1, 3].as_slice(), ListOutcome::Failed(ListFailure::Deadline)),
        ([1, 4].as_slice(), ListOutcome::Failed(ListFailure::Agent)),
    ] {
        let mut bytes = skein_lib::Writer::new(8 + body.len());
        bytes.put(&7_u64.to_be_bytes()).expect("read number");
        bytes.put(body).expect("list outcome");
        let bytes = bytes.finish();
        let decoded = translate::decode(wire::Kind::ListRead.number(), &bytes, &codec()).expect("list answer");
        assert_eq!(up(decoded), jig_domain_host::LinkUp::ListRead { read: 7, outcome });
    }
}

#[test]
fn an_empty_draining_record_becomes_the_hubs_draining_notice() {
    use jig_protocol_worker_wire as wire;
    let limits = codec();
    let record = wire::Draining {};
    let mut bytes = skein_lib::Writer::new(0);
    record.write(&jig_protocol_worker_wire::CEILINGS, &mut bytes).expect("empty encoding");
    let bytes = bytes.finish();
    let decoded = translate::decode(wire::Kind::Draining.number(), &bytes, &limits).expect("draining record");
    assert_eq!(up(decoded), jig_domain_host::LinkUp::Draining);
    assert!(translate::decode(wire::Kind::Draining.number(), &[0], &limits).is_none());
}

#[test]
fn a_version_refusal_decodes_its_wire_and_range_without_parts() {
    use jig_protocol_worker_wire as wire;
    let mismatch = wire::Mismatch { wire: b"smith-charter".as_slice().into(), version: 5, lowest: 4, highest: 4 };
    let refusal = wire::RefusalValue { value: wire::Refusal::Version(mismatch) };
    let mut fields = fixtures::answer_record();
    fields.ending = wire::Ending::Refused(refusal);
    let (answer, parts) = answer(fields);
    assert!(parts.is_empty());
    assert_eq!(
        answer.ending,
        jig_domain_host::Ending::Refused(jig_domain_host::Refusal::Version(jig_domain_host::Mismatch {
            wire: b"smith-charter".as_slice().into(),
            version: 5,
            lowest: 4,
            highest: 4,
        }))
    );
}
