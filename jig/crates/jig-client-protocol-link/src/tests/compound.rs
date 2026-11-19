use super::*;
const PROJECT: u32 = 3;
const FIRST_MESSAGE: u64 = 19;
#[test]
fn a_compound_answer_names_its_first_message_and_operations() {
    let limits = fixtures::limits();
    let value = w::DocumentOutcome::Compound(w::OutcomeCompound {
        project: PROJECT,
        operations: 2,
        first_message: FIRST_MESSAGE,
    });
    let outcome = w::Outcome::Document(w::OutcomeDocument { value });
    let answer = w::Final::Done(w::Done { outcome });
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    let wire = w::KeyedAnswer::FinalAnswer(w::Owned {
        owner: w::Owner::Jig,
        version: jig_client_protocol_wire::VERSION,
        record: bytes.finish(),
    });
    assert_eq!(
        translate::answers::answer(wire, &limits),
        Ok(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Compound {
            project: PROJECT,
            operations: 2,
            first_message: FIRST_MESSAGE
        })))
    );
}
