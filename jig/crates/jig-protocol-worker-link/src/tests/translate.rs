use crate::{Request, tests::fixtures, translate};
use jig_domain_host as host;
use jig_domain_link::{CountKind, Form, Grant, Hello, PROGRAM_ENDPOINTS, Scheme, SourceName};
use jig_protocol_worker_wire::{self as wire, Part};
use skein_lib::{List, Queue, Reader, Time, Token};
fn translated(kind: u16, bytes: &[u8], limits: &wire::v14::Limits) -> (host::LinkDown, Box<[Part]>, Queue<Request>) {
    let mut parts = Queue::uncapped();
    let mut values = Queue::with_capacity(4);
    let down =
        translate::down(kind, bytes, limits, Time::ZERO, &mut parts, &mut 1, &mut values).expect("translated record");
    let mut payloads = List::with_capacity(parts.len());
    for _ in 0..parts.len() {
        payloads.push(parts.pop().expect("queued part")).expect("one slot per part");
    }
    (down, payloads.into_boxed(), values)
}
#[test]
fn calls_carry_the_charters_contract() {
    let limits = fixtures::codec();
    let up = host::LinkUp::Call {
        contract: 3,
        run: Token::new(7),
        attempt: Token::new(2),
        call: b"call".as_slice().into(),
        tool: b"tool".as_slice().into(),
        writes: false,
        input: b"input".as_slice().into(),
        deadline: skein_lib::Duration::from_secs(1),
    };
    let frame = translate::up(up, &[], &limits).expect("call frame");
    let record = wire::Call::decode(&limits, &mut body(&frame)).expect("call");
    assert_eq!(record.contract(), 3);
}

fn assigned() -> (host::Assignment, Box<[Part]>, Queue<Request>) {
    let record = fixtures::assignment_record(fixtures::window());
    let bytes = fixtures::assignment_bytes(&record);
    let (down, parts, values) = translated(0x0180, &bytes, &fixtures::codec());
    let host::LinkDown::Assign { assignment } = down else { panic!("assignment: {down:?}") };
    (assignment, parts, values)
}
fn body(frame: &skein_channel::Frame) -> Reader<'_> {
    Reader::new(frame.bytes().get(8..).expect("body after header"))
}
fn granted(value: wire::GrantValue) -> (Grant, crate::GrantValue) {
    granted_at(value, false)
}
fn granted_at(value: wire::GrantValue, read: bool) -> (Grant, crate::GrantValue) {
    let grant = fixtures::grant_record(value);
    let (kind, bytes) = if read {
        let record =
            wire::ReadList::new(&fixtures::codec(), wire::ReadListParts { read: 7, endpoint: 3, grant }).expect("read");
        let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).expect("read size"));
        record.encode(&mut bytes).expect("encode read");
        (wire::Kind::ReadList, bytes.finish())
    } else {
        (
            wire::Kind::Grant,
            fixtures::granted_bytes(
                &wire::Granted::new(&fixtures::codec(), wire::GrantedParts { task: 7, attempt: 2, grant })
                    .expect("grant"),
            ),
        )
    };
    let (link, parts, mut values) = translated(kind.number(), &bytes, &fixtures::codec());
    let hub = alloc::fmt::format(format_args!("{link:?}{parts:?}"));
    assert!(
        !hub.contains(&alloc::fmt::format(format_args!("{:?}", fixtures::TOKEN_ONE))),
        "the hub never hears secret bytes"
    );
    assert!(parts.is_empty());
    let grant = if read {
        let host::LinkDown::ReadList { read, endpoint, grant } = link else { panic!("read list") };
        assert_eq!((read, endpoint), (7, 3));
        grant
    } else {
        let host::LinkDown::Grant { run, attempt, grant } = link else { panic!("grant") };
        assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
        grant
    };
    assert_eq!((grant.account, grant.generation), (1, 3));
    let Request::Value { account, generation, value, expires } = values.pop().expect("one value") else {
        panic!("value")
    };
    assert!(values.is_empty());
    assert_eq!((account, generation), (1, 3));
    assert_eq!(expires, Time::ZERO.saturating_add(skein_lib::Duration::from_secs(60)));
    (grant, value)
}
#[test]
fn a_granted_credential_hands_its_scheme_and_token_out_and_the_hub_hears_the_account() {
    for (scheme, expected, form) in [
        (wire::Scheme::Bearer, crate::Scheme::Bearer, Scheme::Bearer),
        (wire::Scheme::Key, crate::Scheme::Key, Scheme::Key),
    ] {
        for read in [false, true] {
            let (grant, value) = granted_at(fixtures::credential_value(scheme.clone()), read);
            assert_eq!(grant.form, Form::Credential(form));
            let crate::GrantValue::Credential { scheme, token } = value else { panic!("credential: {value:?}") };
            assert_eq!((scheme, token.expose()), (expected, fixtures::TOKEN_ONE));
        }
    }
}
#[test]
fn a_granted_application_secret_names_the_application_table() {
    let (grant, value) = granted(fixtures::application_value());
    assert_eq!(grant.form, Form::Application);
    let crate::GrantValue::Application { secret } = value else { panic!("application: {value:?}") };
    assert_eq!(secret.expose(), fixtures::TOKEN_ONE);
}
#[test]
fn a_hub_turn_becomes_a_turn_record_with_its_refusals() {
    let frame = translate::up(fixtures::turn(), &[], &fixtures::codec()).expect("turn");
    let record = wire::Turn::decode(&fixtures::codec(), &mut body(&frame)).expect("turn");
    assert_eq!(record.body(), b"turn");
    assert!(record.opens_window());
    assert_eq!(record.refused().get(0).expect("refusal").reason(), &wire::MessageRefusal::Full);
}
#[test]
fn a_hub_answer_writes_left_and_saved_work_from_their_parts() {
    let frame = translate::up(fixtures::answer(), &fixtures::work_parts(), &fixtures::codec()).expect("answer");
    let record = wire::Answer::decode(&fixtures::codec(), &mut body(&frame)).expect("answer");
    let wire::Ending::Ended(ended) = record.ending() else { panic!("ended") };
    assert_eq!(ended.work().left().as_deref(), Some(b"left".as_slice()));
    assert_eq!(ended.work().saved().as_deref(), Some(b"saved".as_slice()));
}
#[test]
fn activity_becomes_a_fact_record() {
    let up = host::LinkUp::Activity {
        run: Token::new(7),
        attempt: Token::new(2),
        activity: host::AgentActivity::Count {
            kind: CountKind::LlmFinished,
            elapsed: skein_lib::Duration::from_secs(1),
            count: 2,
        },
    };
    let frame = translate::up(up, &[], &fixtures::codec()).expect("fact");
    let record = wire::Fact::decode(&fixtures::codec(), &mut body(&frame)).expect("fact");
    let wire::FactValue::Count(count) = record.value() else { panic!("count") };
    assert_eq!((count.kind(), count.value()), (&wire::CountKind::LlmFinished, 2));
}
#[test]
fn an_assignment_record_hands_its_items_up_as_a_part() {
    let (assignment, parts, _) = assigned();
    let workspace = assignment.assignment.workspace.expect("workspace");
    assert_eq!(parts.as_ref(), [Part { token: workspace.items, bytes: b"items".as_slice().into() }]);
    assert_eq!(workspace.workstream, 3);
}
#[test]
fn an_assignment_record_keeps_its_start_messages_in_order() {
    let limits = fixtures::codec();
    let mut record = fixtures::assignment_record(fixtures::window()).into_parts();
    let mut messages = List::with_capacity(2);
    for name in [5, 6] {
        messages
            .push(
                wire::Message::new(
                    &limits,
                    wire::MessageParts {
                        name,
                        sender: b"sender".as_slice().into(),
                        words: b"words".as_slice().into(),
                        finish_requested: name == 6,
                    },
                )
                .expect("message"),
            )
            .expect("room");
    }
    record.messages = messages;
    let mut limits = limits;
    limits.assignment_messages = 2;
    let record = wire::Assignment::new(&limits, record).expect("two messages");
    let bytes = fixtures::assignment_bytes(&record);
    let (link, _, _) = translated(0x0180, &bytes, &limits);
    let host::LinkDown::Assign { assignment } = link else { panic!("assignment") };
    let mut names = List::with_capacity(2);
    for message in assignment.messages {
        names.push((message.name, message.finish_requested)).expect("two names");
    }
    assert_eq!(names.into_boxed().as_ref(), [(Token::new(5), false), (Token::new(6), true)]);
}
#[test]
fn a_message_record_becomes_the_hubs_inbound_message() {
    for finish_requested in [false, true] {
        let record = wire::Inbound::new(
            &fixtures::codec(),
            wire::InboundParts {
                task: 7,
                attempt: 2,
                name: 5,
                sender: b"sender".as_slice().into(),
                words: b"words".as_slice().into(),
                finish_requested,
            },
        )
        .expect("message");
        let bytes = fixtures::inbound_bytes(&record);
        let (link, _, _) = translated(0x0181, &bytes, &fixtures::codec());
        assert_eq!(
            link,
            host::LinkDown::Message {
                run: Token::new(7),
                attempt: Token::new(2),
                name: Token::new(5),
                sender: b"sender".as_slice().into(),
                words: b"words".as_slice().into(),
                finish_requested
            }
        );
    }
}
#[test]
fn a_named_part_missing_from_the_root_is_dropped() {
    assert!(translate::up(fixtures::answer(), &[], &fixtures::codec()).is_none());
}

#[test]
fn the_hello_carries_the_hubs_agent_bounds_and_hosting() {
    let frame = translate::hello(fixtures::hello(), &fixtures::codec()).expect("hello");
    let record = wire::Hello::decode(&fixtures::codec(), &mut body(&frame)).expect("hello");
    assert_eq!((record.slots(), record.stop_bound()), (1, skein_lib::Duration::from_secs(1)));
    assert_eq!(record.workstreams().get(0), Some(&3));
    let run = record.hosting().get(0).expect("hosting");
    assert_eq!((run.task(), run.attempt(), run.phase()), (7, 2, &wire::Phase::Active));
    let bounds = record.bounds();
    let model = bounds.endpoints().get(0).expect("model");
    assert_eq!((model.endpoint(), model.window(), model.output()), (1, 16, 8));
    assert_eq!(
        (bounds.turns(), bounds.spend(), bounds.inbox(), bounds.message_bytes(), bounds.unacknowledged()),
        (1, 10, 1, 16, 1)
    );
}

#[test]
fn a_hub_invalid_start_keeps_its_words_in_the_answer_record() {
    let host::LinkUp::Answer { run, attempt, mut answer } = fixtures::answer() else { panic!("answer") };
    answer.ending = host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
        cause: jig_domain_host::StartCause::Other,
        why: b"budget too small".as_slice().into(),
    }));
    let frame = translate::up(host::LinkUp::Answer { run, attempt, answer }, &[], &fixtures::codec())
        .expect("invalid start answer");
    let record = wire::Answer::decode(&fixtures::codec(), &mut body(&frame)).expect("answer");
    let wire::Ending::Refused(refusal) = record.ending() else { panic!("refusal") };
    let wire::Refusal::Invalid(invalid) = refusal.value() else { panic!("invalid") };
    let wire::Invalid::Start(start) = invalid.value() else { panic!("start") };
    assert_eq!(start.why(), b"budget too small");
}

#[test]
fn each_account_notice_becomes_its_record() {
    let codec = fixtures::codec();
    let (run, attempt) = (fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO);
    let frame = translate::up(host::LinkUp::Unauthorized { run, attempt, account: 1, generation: 3 }, &[], &codec)
        .expect("unauthorized");
    let record = wire::Unauthorized::decode(&codec, &mut body(&frame)).expect("unauthorized");
    assert_eq!(
        (frame.kind(), record.task(), record.attempt(), record.account(), record.generation()),
        (0x0105, 7, 2, 1, 3)
    );
    for reset in [Some(skein_lib::Duration::from_secs(60)), None] {
        let frame =
            translate::up(host::LinkUp::Limited { run, attempt, account: 1, reset }, &[], &codec).expect("limited");
        let record = wire::Limited::decode(&codec, &mut body(&frame)).expect("limited");
        assert_eq!(
            (frame.kind(), record.task(), record.attempt(), record.account(), *record.reset()),
            (0x0106, 7, 2, 1, reset)
        );
    }
    let refused = host::LinkUp::AccountRefused {
        run,
        attempt,
        account: 1,
        code: b"forbidden".as_slice().into(),
        reason: b"suspended".as_slice().into(),
    };
    let frame = translate::up(refused, &[], &codec).expect("account refused");
    let record = wire::AccountRefused::decode(&codec, &mut body(&frame)).expect("account refused");
    assert_eq!(
        (frame.kind(), record.task(), record.account(), record.code(), record.reason()),
        (0x0107, 7, 1, b"forbidden".as_slice(), b"suspended".as_slice())
    );
}

#[test]
fn an_account_refused_reason_past_its_bound_is_not_sent() {
    let refused = host::LinkUp::AccountRefused {
        run: fixtures::RUN_SEVEN,
        attempt: fixtures::ATTEMPT_TWO,
        account: 1,
        code: b"forbidden".as_slice().into(),
        reason: [b'r'; 17].as_slice().into(),
    };
    assert!(translate::up(refused, &[], &fixtures::codec()).is_none());
}

#[test]
fn a_failed_answer_keeps_the_limited_or_refused_account_in_its_record() {
    let reset = Some(skein_lib::Duration::from_secs(60));
    for failure in [
        host::RunFailure::Limited { account: 1, reset },
        host::RunFailure::Limited { account: 1, reset: None },
        host::RunFailure::Refused { account: 1 },
    ] {
        let host::LinkUp::Answer { run, attempt, mut answer } = fixtures::answer() else { panic!("answer") };
        answer.ending = host::Ending::Failed {
            failure: host::Failure::Run(failure),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
            model: None,
        };
        let frame =
            translate::up(host::LinkUp::Answer { run, attempt, answer }, &[], &fixtures::codec()).expect("answer");
        let record = wire::Answer::decode(&fixtures::codec(), &mut body(&frame)).expect("answer");
        let wire::Ending::Failed(failed) = record.ending() else { panic!("failed") };
        let wire::Failure::Run(run_failure) = failed.failure() else { panic!("run failure") };
        let kept = if let wire::RunFailure::Limited(limited) = run_failure.value() {
            host::RunFailure::Limited { account: limited.account(), reset: *limited.reset() }
        } else if let wire::RunFailure::Refused(refused) = run_failure.value() {
            host::RunFailure::Refused { account: refused.account() }
        } else {
            panic!("not an account: {:?}", run_failure.value())
        };
        assert_eq!(kept, failure);
    }
}

fn program_granted(name: &[u8]) -> Box<[u8]> {
    let record = wire::Granted::new(
        &fixtures::codec(),
        wire::GrantedParts { task: 7, attempt: 2, grant: fixtures::grant_record(program_value(name)) },
    )
    .expect("grant");
    fixtures::granted_bytes(&record)
}
#[test]
fn the_worker_link_derives_the_program_form() {
    let bytes = program_granted(b"claude");
    let (down, _, values) = translated(0x0187, &bytes, &fixtures::codec());
    let host::LinkDown::Grant { grant, .. } = down else { panic!("grant") };
    assert_eq!(grant.form, Form::Program(SourceName::new(b"claude").expect("source name")));
    assert!(values.is_empty(), "a program name needs no table entry");
}
#[test]
fn a_program_name_the_rules_refuse_is_malformed() {
    for name in [b"".as_slice(), b"Upper", b"bad_name", b"../claude"] {
        let mut parts = Queue::uncapped();
        let mut values = Queue::with_capacity(4);
        assert!(
            translate::down(
                0x0187,
                &program_granted(name),
                &fixtures::codec(),
                Time::ZERO,
                &mut parts,
                &mut 1,
                &mut values
            )
            .is_none(),
            "{name:?}"
        );
        assert!(values.is_empty());
    }
}

fn program_value(name: &[u8]) -> wire::GrantValue {
    wire::GrantValue::Program(
        wire::ProgramValue::new(&fixtures::codec(), wire::ProgramValueParts { name: name.into() }).expect("wire name"),
    )
}
fn program_assignment(name: &[u8]) -> Box<[u8]> {
    let mut grants = List::with_capacity(1);
    grants.push(fixtures::grant_record(program_value(name))).expect("one grant");
    assignment_grants(grants)
}
#[test]
fn an_assignment_keeps_a_program_name_without_a_table_value() {
    let (down, _, values) = translated(0x0180, &program_assignment(b"claude"), &fixtures::codec());
    let host::LinkDown::Assign { assignment } = down else { panic!("assignment") };
    assert_eq!(assignment.assignment.grants[0].form, Form::Program(SourceName::new(b"claude").expect("source name")));
    assert!(values.is_empty());
}
#[test]
fn an_assignment_with_a_refused_program_name_is_malformed() {
    let mut parts = Queue::uncapped();
    let mut values = Queue::with_capacity(4);
    assert!(
        translate::down(
            0x0180,
            &program_assignment(b"../claude"),
            &fixtures::codec(),
            Time::ZERO,
            &mut parts,
            &mut 1,
            &mut values
        )
        .is_none()
    );
}

fn assignment_grants(grants: List<wire::Grant>) -> Box<[u8]> {
    let mut record = fixtures::assignment_record(fixtures::window()).into_parts();
    record.grants = grants;
    fixtures::assignment_bytes(&wire::Assignment::new(&fixtures::codec(), record).expect("assignment"))
}
#[test]
fn a_refused_later_program_grant_leaves_no_value_queued() {
    let mut grants = List::with_capacity(2);
    grants.push(fixtures::grant_record(fixtures::credential_value(wire::Scheme::Bearer))).expect("first grant");
    grants.push(fixtures::grant_record(program_value(b"../claude"))).expect("later grant");
    let mut parts = Queue::uncapped();
    let mut values = Queue::with_capacity(4);
    let refused = translate::down(
        0x0180,
        &assignment_grants(grants),
        &fixtures::codec(),
        Time::ZERO,
        &mut parts,
        &mut 1,
        &mut values,
    );
    assert!(refused.is_none());
    assert!(values.is_empty(), "a refused assignment queues no earlier credential");
}

#[test]
fn the_hosts_endpoint_cap_is_the_worker_wires_ceiling() {
    assert_eq!(PROGRAM_ENDPOINTS, wire::v14::CEILINGS.bounds_program_endpoints);
}

fn hello_with_program_endpoints(count: u32) -> Hello {
    let mut hello = fixtures::hello();
    let mut endpoints = List::with_capacity(count);
    for endpoint in 0..count {
        endpoints.push(endpoint).expect("one slot per endpoint");
    }
    hello.bounds.program_endpoints = endpoints.into_boxed();
    hello
}

#[test]
fn the_hello_admits_the_endpoint_cap_and_refuses_past_it_or_a_duplicate() {
    // domain/hosts/charters.md, 1: 64 distinct endpoint identities per hello.
    const ENDPOINT_CAP: u32 = 64;
    let mut duplicate = hello_with_program_endpoints(ENDPOINT_CAP);
    *duplicate.bounds.program_endpoints.last_mut().expect("nonempty endpoint list") = 0;
    let limits = fixtures::codec();
    for (hello, admitted) in [
        (hello_with_program_endpoints(ENDPOINT_CAP), true),
        (hello_with_program_endpoints(ENDPOINT_CAP + 1), false),
        (duplicate, false),
    ] {
        assert_eq!(translate::hello(hello, &limits).is_some(), admitted);
    }
}

#[test]
fn an_assignment_decodes_each_live_ask_without_changing_it() {
    for ask in [
        host::LiveAsk::NONE,
        host::LiveAsk { text: true, reasoning: false, input: false },
        host::LiveAsk { text: false, reasoning: true, input: false },
        host::LiveAsk { text: false, reasoning: false, input: true },
        host::LiveAsk { text: true, reasoning: true, input: true },
    ] {
        let limits = fixtures::codec();
        let mut fields = fixtures::assignment_record(fixtures::window()).into_parts();
        fields.live = wire::LiveAsk::new(
            &limits,
            wire::LiveAskParts { text: ask.text, reasoning: ask.reasoning, input: ask.input },
        )
        .unwrap();
        let record = wire::Assignment::new(&limits, fields).unwrap();
        let (down, _, _) = translated(0x0180, &fixtures::assignment_bytes(&record), &limits);
        let host::LinkDown::Assign { assignment } = down else { panic!("assignment") };
        assert_eq!(assignment.assignment.live, ask);
    }
}
#[test]
fn every_piece_kind_encodes_whole_including_offsets_past_u32() {
    for kind in [
        host::LiveKind::Text,
        host::LiveKind::Reasoning,
        host::LiveKind::Input { call: Box::from(&b"call-id"[..]), tool: Box::from(&b"tool"[..]) },
    ] {
        let piece = host::Piece {
            turn: 17,
            retry: 23,
            kind: kind.clone(),
            offset: u64::from(u32::MAX) + 19,
            bytes: Box::from([0xff, 0, 0x80]),
        };
        let frame = translate::up(
            host::LinkUp::Piece { run: Token::new(83), attempt: Token::new(97), piece },
            &[],
            &fixtures::codec(),
        )
        .unwrap();
        assert_eq!(frame.kind(), wire::Kind::Fact.number());
        let fact = wire::Fact::decode(&fixtures::codec(), &mut body(&frame)).unwrap();
        assert_eq!((fact.task(), fact.attempt()), (83, 97));
        let wire::FactValue::Piece(piece) = fact.value() else { panic!("piece") };
        assert_eq!(
            (piece.turn(), piece.retry(), piece.offset(), piece.bytes()),
            (17, 23, u64::from(u32::MAX) + 19, [0xff, 0, 0x80].as_slice())
        );
        match (kind, piece.kind()) {
            (host::LiveKind::Text, wire::LiveKind::Text) | (host::LiveKind::Reasoning, wire::LiveKind::Reasoning) => {}
            (host::LiveKind::Input { call, tool }, wire::LiveKind::Input(input)) => {
                assert_eq!(input.id(), call.as_ref());
                assert_eq!(input.tool(), tool.as_ref());
            }
            other => panic!("kind changed: {other:?}"),
        }
    }
}

#[test]
fn list_answers_encode_rows_and_every_failure() {
    use jig_domain_link::{ListFailure, ListOutcome, ListedModel};
    for (outcome, body) in [
        (ListOutcome::Listed(Box::new([])), [0, 0, 0, 0, 0].as_slice()),
        (
            ListOutcome::Listed(Box::new([ListedModel {
                name: b"m".as_slice().into(),
                shown: Some(b"M".as_slice().into()),
                window: Some(10),
                output: Some(5),
                efforts: Box::new([b"low".as_slice().into()]),
            }])),
            [
                0, 0, 0, 0, 1, 0, 0, 0, 1, b'm', 1, 0, 0, 0, 1, b'M', 1, 0, 0, 0, 10, 1, 0, 0, 0, 5, 0, 0, 0, 1, 0, 0,
                0, 3, b'l', b'o', b'w',
            ]
            .as_slice(),
        ),
        (ListOutcome::Failed(ListFailure::Refused { status: 429 }), [1, 0, 1, 173].as_slice()),
        (ListOutcome::Failed(ListFailure::Unreachable), [1, 1].as_slice()),
        (ListOutcome::Failed(ListFailure::Unreadable), [1, 2].as_slice()),
        (ListOutcome::Failed(ListFailure::Deadline), [1, 3].as_slice()),
        (ListOutcome::Failed(ListFailure::Agent), [1, 4].as_slice()),
    ] {
        let frame =
            translate::up(host::LinkUp::ListRead { read: 7, outcome }, &[], &fixtures::codec()).expect("list answer");
        let mut bytes = skein_lib::Writer::new(8 + body.len());
        bytes.put(&7_u64.to_be_bytes()).expect("read number");
        bytes.put(body).expect("list outcome");
        let bytes = bytes.finish();
        assert_eq!(frame.kind(), 0x010a);
        assert_eq!(frame.bytes().get(8..).expect("body"), bytes.as_ref());
    }
}

#[test]
fn a_hubs_draining_notice_becomes_the_empty_draining_record() {
    let limits = fixtures::codec();
    let frame = translate::up(host::LinkUp::Draining, &[], &limits).expect("draining frame");
    assert_eq!(frame.kind(), wire::Kind::Draining.number());
    let mut body = body(&frame);
    assert!(body.is_empty());
    wire::Draining::decode(&limits, &mut body).expect("draining record");
}

#[test]
fn a_version_refusal_encodes_its_wire_and_range_without_parts() {
    let mismatch = host::Mismatch { wire: b"smith-charter".as_slice().into(), version: 5, lowest: 4, highest: 4 };
    let value = host::LinkUp::Answer {
        run: Token::new(7),
        attempt: Token::new(2),
        answer: host::RunAnswer {
            turns: 0,
            spent: 0,
            read: None,
            refused: Box::new([]),
            ending: host::Ending::Refused(host::Refusal::Version(mismatch)),
        },
    };
    let limits = fixtures::codec();
    let frame = translate::up(value, &[], &limits).expect("version refusal");
    let record = wire::Answer::decode(&limits, &mut body(&frame)).expect("answer");
    let wire::Ending::Refused(value) = record.ending() else { panic!("refused") };
    let wire::Refusal::Version(value) = value.value() else { panic!("version") };
    assert_eq!(value.wire(), b"smith-charter");
    assert_eq!((value.version(), value.lowest(), value.highest()), (5, 4, 4));
}
