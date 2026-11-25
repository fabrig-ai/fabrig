//! Person answer additions keep their question numbers across storage.
use jig_domain_tasks as tasks;
use jig_protocol_store::{codec, tasks_v13 as wire};
use skein_lib::Reader;
use wire::Record;

#[test]
fn person_answer_notice_and_refusal_round_trip_with_their_question_numbers() {
    for question in [0, 1, u64::MAX] {
        let value = tasks::MessageKind::QuestionAnswered { question };
        let encoded = codec::tasks::activity::message_kind(&value, &wire::CEILINGS).expect("notice");
        let bytes = encoded.encode(&wire::CEILINGS).expect("bounded value");
        assert_eq!(bytes.len(), 9);
        let decoded = wire::MessageKind::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("notice");
        assert_eq!(codec::tasks::activity::read_message_kind(decoded, &wire::CEILINGS), Ok(value));
        let problem = tasks::Problem::new(None, tasks::Refusal::PersonAnswers { question });
        let bytes = codec::tasks::carried::problem(&problem, &wire::CEILINGS).expect("refusal");
        assert_eq!(codec::tasks::carried::read_problem(&bytes, &wire::CEILINGS), Ok(problem));
    }
}

#[test]
fn a_credit_without_an_offer_reads_at_its_original_holder() {
    const ASKED: u64 = 21;
    const QUESTION: u64 = 41;
    const AT_LINE: u64 = 8;
    let text = super::values::TASKS;
    let older_text = text.replace(" offer: option QuestionOffer = none }", " }");
    let older = skein_codec::schema::parse(older_text.as_bytes()).expect("previous tasks schema");
    let current = skein_codec::schema::parse(text.as_bytes()).expect("current tasks schema");
    let plan = skein_codec::layout::Plan::derive(
        &older,
        older.root(b"QuestionCredit", None).expect("previous credit"),
        &[],
        &current,
        current.root(b"QuestionCredit", None).expect("current credit"),
        &[],
    )
    .expect("placement defaults to the original holder");
    let holder = wire::Party::Task(wire::Number { value: ASKED });
    let holder_bytes = holder.encode(&wire::CEILINGS).expect("unchanged party encoding");
    let previous = [&QUESTION.to_be_bytes()[..], &holder_bytes, &holder_bytes, &AT_LINE.to_be_bytes()].concat();
    let mut output = skein_lib::Buffer::with_capacity(u32::try_from(previous.len() + 1).expect("one more option byte"));
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    plan.convert(&previous, &mut output, &mut work).expect("convert old credit");
    let credit = wire::QuestionCredit::decode(&wire::CEILINGS, &mut Reader::new(output.as_slice())).expect("credit");
    assert_eq!(
        credit,
        wire::QuestionCredit { number: QUESTION, answerer: holder.clone(), asked: holder, line: AT_LINE, offer: None }
    );
}
