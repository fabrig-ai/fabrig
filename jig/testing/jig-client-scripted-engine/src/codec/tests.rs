//! Scripted admission at the link's whole-document boundary (client/domain/testing.md, 4).
use super::*;
use crate::{Bounds, Engine, Invalid, Script, Truth, Up};
use jig_client_protocol_link::{Undecodable, translate};
use skein_lib::{Duration, Time, Token};

fn frame() -> c::Me {
    let text = c::Text::new(Box::from(b"party".as_slice()), 32).expect("fixture text fits");
    let text = c::DocumentText::complete(text);
    c::Me {
        party: c::MeParty { number: 1, kind: c::PartyKind::Person, login: text.clone(), name: text },
        projects: List::with_capacity(0),
        needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1),
        deployment_role: None,
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
    }
}

#[test]
fn scripted_front_refuses_the_document_the_link_refuses_over_its_bound() {
    let value = c::ReadValue::Me(Box::new(frame()));
    let record = values::wire_read_value(&value, &w::CEILINGS).expect("wire frame fits");
    let bytes = encoded(&record, w::ReadValue::measure, w::ReadValue::encode).expect("measured document");
    let exact = Bounds { document_bytes: u64::from(record.measure()), ..Bounds::default() };
    assert_eq!(translate::reads::value(&bytes, &front::limits(&exact).expect("valid limits")), Ok(value.clone()));
    let mut truth = Truth::default();
    truth.frames.insert(1, frame());
    truth.sign_ins.insert(11, 1);
    let mut engine = Engine::new(Script { bounds: exact, truth: truth.clone(), ..Script::default() }, 1)
        .expect("the complete document fits exactly");
    let token = Token::new(1);
    engine.read(Time::ZERO, 11, token, c::Read::Me).expect("valid read");
    assert!(engine.poll(Time::ZERO).iter().any(|up| matches!(
        up, Up::Read { token: found, terminal: c::ReadResult::Value(found_value) }
            if *found == token && found_value == &value
    )));
    let below = Bounds { document_bytes: exact.document_bytes - 1, ..exact };
    assert_eq!(
        translate::reads::value(&bytes, &front::limits(&below).expect("valid limits")),
        Err(Undecodable::TooLarge)
    );
    assert!(matches!(Engine::new(Script { bounds: below, truth, ..Script::default() }, 1), Err(Invalid::Payload)));
}

#[test]
fn scripted_answers_and_watch_documents_are_the_links_decoded_values() {
    let bounds = Bounds::default();
    let value = c::Delivery::Me(Box::new(frame()));
    assert_eq!(delivery(&value, &bounds), Ok(value));
    let value = c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Lifted { hold: 7, released: 1 }));
    assert_eq!(answer(&value, &bounds), Ok(value));
    let value = c::Answer::Refused { refusal: c::Refusal::KeyConflict, at: Some(3) };
    assert_eq!(answer(&value, &bounds), Ok(value));
}

#[test]
fn recursive_fact_pieces_preserve_parent_indexes_and_child_order() {
    let value = c::HoldReason::Dependency { task: 7, how: c::Ending::Done };
    let record = facts::hold_reason(&value, &w::CEILINGS).expect("recursive facts fit");
    let mut reader = skein_lib::Reader::new(record.rows());
    let root = skein_codec::piece::split(&mut reader).expect("piece length").expect("root piece");
    let root = w::Fact::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&root)).expect("generated root codec");
    let w::Fact::HoldReason(root) = root else { panic!("hold-reason root") };
    assert_eq!(*root.parent(), None);
    let w::HoldReasonFields::Dependency(dependency) = root.value() else { panic!("dependency fields") };
    assert_eq!(dependency.how(), 1);
    let child = skein_codec::piece::split(&mut reader).expect("piece length").expect("child piece");
    let child = w::Fact::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&child)).expect("generated child codec");
    let w::Fact::Ending(child) = child else { panic!("ending child") };
    assert_eq!(*child.parent(), Some(0));
    assert_eq!(child.value(), &w::EndingFields::Done);
    assert!(reader.is_empty());
}

#[test]
fn committed_turns_use_the_transcript_and_client_codecs() {
    let text = c::Text::new(Box::from(b"reply".as_slice()), 32).expect("fixture text fits");
    let value = c::ReadValue::Turn(Box::new(c::ArchivedTurn {
        task: 7,
        name: c::TurnName { attempt: 1, number: 1 },
        cumulative: 5,
        read: None,
        at: skein_lib::Wall::from_nanos(3),
        turn: c::TurnRead::Decoded(c::Turn {
            messages: List::from_boxed(Box::new([c::Message {
                speaker: c::Speaker::Assistant,
                blocks: List::from_boxed(Box::new([c::Block::Text(text)])),
            }])),
            used: c::Usage { input: 1, output: 2, cached: 3, cache_written: 4 },
            spent: 5,
            read_through: None,
            cut: false,
        }),
    }));
    assert_eq!(read(&value, &Bounds::default()), Ok(value));
}

#[test]
fn committed_call_outcomes_survive_with_or_without_an_explicit_result_block() {
    let text = |bytes: &[u8]| c::Text::new(bytes.into(), 32).expect("fixture text fits");
    for explicit in [false, true] {
        let mut blocks = vec![c::Block::Call(c::Call {
            number: 1,
            tool: text(b"tool"),
            input: text(b"input"),
            outcome: Some(text(b"refused")),
            failed: true,
            objects: List::with_capacity(0),
            parts: List::with_capacity(0),
        })];
        if explicit {
            blocks.push(c::Block::Result { call: 1, summary: text(b"refused"), failed: true });
        }
        let archive = c::ArchivedTurn {
            task: 7,
            name: c::TurnName { attempt: 1, number: 1 },
            cumulative: 5,
            read: None,
            at: skein_lib::Wall::EPOCH,
            turn: c::TurnRead::Decoded(c::Turn {
                messages: List::from_boxed(Box::new([c::Message {
                    speaker: c::Speaker::Assistant,
                    blocks: List::from_boxed(blocks.into_boxed_slice()),
                }])),
                used: c::Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                spent: 5,
                read_through: None,
                cut: false,
            }),
        };
        let decoded = read(&c::ReadValue::Turn(Box::new(archive)), &Bounds::default()).expect("archive admitted");
        let c::ReadValue::Turn(archive) = decoded else { panic!("archive") };
        let c::TurnRead::Decoded(turn) = &archive.turn else { panic!("decoded turn") };
        let blocks = &turn.messages.get(0).expect("message").blocks;
        assert_eq!(blocks.len(), 2);
        let c::Block::Call(call) = blocks.get(0).expect("call") else { panic!("call block") };
        assert_eq!(call.outcome, Some(text(b"refused")));
        assert!(call.failed);
        assert!(matches!(blocks.get(1), Some(c::Block::Result { call: 1, failed: true, .. })));
    }
}

/// Both job executor forms round-trip through the link (jig/docs/design/client/domain/contract.md, 5.7).
#[test]
fn a_job_chip_round_trips_wire_to_domain_to_wire() {
    let limits = front::limits(&Bounds::default()).expect("valid limits");
    let mut details = crate::tests::chip_details(c::ChipPhase::Active(c::RunStage::Running { attempt: 1 }));
    details.details.executor = c::Executor::Job { kind: 7 };
    let chip = c::Chip {
        task: 8,
        title: c::DocumentText::complete(c::Text::new(Box::from(b"job task".as_slice()), 32).expect("fixture title")),
        executor: c::ChipExecutor::Job { kind: 7 },
        phase: c::Phase::Running,
        details: Box::new(details),
    };
    let wire = facts::chip(&chip, limits.wire()).expect("job chip fits");
    let restored = translate::documents::chip(wire.clone(), &limits).expect("job chip decodes");
    assert_eq!(restored, chip);
    assert_eq!(facts::chip(&restored, limits.wire()), Ok(wire));
}

/// jig/docs/design/protocol/channel.md, 4.3: scripted engines carry the fleet's reason.
#[test]
fn job_kinds_hello_refusal_round_trips() {
    let record = values::wire_hello_refusal(&c::HelloRefusal::JobKinds, &w::CEILINGS).expect("unit refusal");
    let bytes = encoded(&record, w::HelloRefusal::measure, w::HelloRefusal::encode).expect("encoded refusal");
    let decoded = w::HelloRefusal::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("decoded refusal");
    assert_eq!(decoded, record);
}
