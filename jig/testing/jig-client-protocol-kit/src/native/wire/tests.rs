use super::*;
use crate::fixtures::{PROJECT, WIRE_TASK};
use jig_client::contract as c;
use jig_client_kit::harness::Receipt;
use skein_lib::List;
const ATTEMPT: Token = Token::new(7);
const FIRST_MESSAGE: u64 = 19;
const KEY: [u8; 16] = [3; 16];
fn encoded<T: w::Record<Limits = w::Limits>>(value: &T) -> Box<[u8]> {
    value.encode(&w::CEILINGS).expect("bounded wire fixture")
}
fn request() -> Request {
    let mut operations = List::with_capacity(2);
    operations
        .push(c::Operation::Message {
            project: PROJECT,
            task: WIRE_TASK,
            words: c::Text::new(b"continue".as_slice().into(), 64).expect("bounded wire fixture"),
        })
        .expect("bounded wire fixture");
    operations.push(c::Operation::Stop { project: PROJECT, task: WIRE_TASK }).expect("bounded wire fixture");
    let ask = translate::asks::ask(&c::Ask::Compound { project: PROJECT, operations }, &w::CEILINGS)
        .expect("bounded wire fixture");
    let keyed = w::Keyed {
        key: KEY,
        owner: w::Owner::Jig,
        version: jig_client_protocol_wire::VERSION,
        request: encoded(&ask),
        value: None,
    };
    Request::Exchange { exchange: ATTEMPT, credential: None, message: Message::Request(encoded(&keyed)) }
}
fn reply() -> Event {
    let value = w::DocumentOutcome::Compound(w::OutcomeCompound {
        project: PROJECT,
        operations: 2,
        first_message: FIRST_MESSAGE,
    });
    let outcome = w::Outcome::Document(w::OutcomeDocument { value });
    let answer = w::Final::Done(w::Done { outcome });
    let value = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: encoded(&answer) };
    let answer = w::KeyedAnswer::FinalAnswer(value);
    Event::Reply { exchange: ATTEMPT, message: encoded(&answer) }
}
#[test]
fn a_compound_word_receipt_joins_its_observed_target_and_answered_message() {
    let mut wire = Wire::default();
    wire.sent(&request());
    let receipts = wire.additional_receipts(&reply());
    let [Receipt::Answer { attempt, final_answer, words, .. }] = receipts.as_slice() else {
        panic!("compound words receipt: {receipts:?}");
    };
    assert_eq!((*attempt, *final_answer, *words), (ATTEMPT, true, Some((WIRE_TASK, FIRST_MESSAGE))));
}
