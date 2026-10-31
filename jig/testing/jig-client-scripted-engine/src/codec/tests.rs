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
    let value = c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Lifted { hold: 7, released: 1 }));
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
