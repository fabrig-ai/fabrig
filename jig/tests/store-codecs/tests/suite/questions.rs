//! Person answer additions keep their question numbers across storage.
use jig_domain_tasks as tasks;
use jig_protocol_store::{codec, tasks_v9 as wire};
use skein_lib::{Reader, Writer};

#[test]
fn person_answer_notice_and_refusal_round_trip_with_their_question_numbers() {
    for question in [0, 1, u64::MAX] {
        let value = tasks::MessageKind::QuestionAnswered { question };
        let encoded = codec::tasks::activity::message_kind(&value, &wire::CEILINGS).expect("notice");
        let mut writer = Writer::new(encoded.measure() as usize);
        encoded.encode(&mut writer).expect("measured notice");
        let bytes = writer.finish();
        assert_eq!(bytes.len(), 9);
        let decoded = wire::MessageKind::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("notice");
        assert_eq!(codec::tasks::activity::read_message_kind(decoded, &wire::CEILINGS), Ok(value));
        let problem = tasks::Problem::new(None, tasks::Refusal::PersonAnswers { question });
        let bytes = codec::tasks::carried::problem(&problem, &wire::CEILINGS).expect("refusal");
        assert_eq!(codec::tasks::carried::read_problem(&bytes, &wire::CEILINGS), Ok(problem));
    }
}
