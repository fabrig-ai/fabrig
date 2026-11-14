use super::*;
use jig_domain_link::AskAnswer;

#[test]
fn asks_are_fenced_as_calls_and_answers_recheck_liveness() {
    for channel in [C1, Token::new(0)] {
        let mut h = Harness::new(Limits { engine_slots: 1, ..LIMITS });
        let kinds = if channel == C1 {
            h.hello(C1, 2, &[], &[]);
            Kinds::Workers
        } else {
            Kinds::Engine
        };
        h.step(Event::Start {
            placement: None,
            version: crate::Version { charter: 1, revision: 1 },
            grants: Box::new([]),
            reply_to: to(A1),
            run: R1,
            attempt: A1,
            workstream: 1,
            kinds,
            assignment: references(),
        });
        let placed = h.settle();
        let [Request::Assign { channel: at, .. }, Request::Placed { .. }] = &*placed else {
            panic!("placed: {placed:?}")
        };
        assert_eq!(*at, channel);
        for (from, run, attempt) in [(C2, R1, A1), (channel, R2, A1), (channel, R1, A2)] {
            assert_eq!(
                &*h.step(Event::Ask { channel: from, run, attempt, number: 7, bytes: b"x".as_slice().into() }),
                &[Request::Asked { channel: from, run, attempt, number: 7, answer: AskAnswer::Refused }]
            );
        }
        assert_eq!(
            &*h.step(Event::Ask { channel, run: R1, attempt: A1, number: 1, bytes: b"x".as_slice().into() }),
            &[Request::Ask { channel, run: R1, attempt: A1, number: 1, bytes: b"x".as_slice().into() }]
        );
        for answer in [AskAnswer::Bytes(b"y".as_slice().into()), AskAnswer::Refused] {
            assert_eq!(
                &*h.step(Event::Asked { channel, run: R1, attempt: A1, number: 1, answer: answer.clone() }),
                &[Request::Asked { channel, run: R1, attempt: A1, number: 1, answer }]
            );
        }
        assert_eq!(h.domain.calls(), 0, "asks retain no call and commit nothing");
        assert!(
            h.step(Event::Asked { channel: C2, run: R1, attempt: A1, number: 1, answer: AskAnswer::Refused })
                .is_empty()
        );
        h.step(Event::Cancel { run: R1, attempt: A1 });
        assert!(
            h.step(Event::Asked { channel, run: R1, attempt: A1, number: 1, answer: AskAnswer::Refused }).is_empty()
        );
        assert_eq!(
            &*h.step(Event::Ask { channel, run: R1, attempt: A1, number: 2, bytes: b"x".as_slice().into() }),
            &[Request::Asked { channel, run: R1, attempt: A1, number: 2, answer: AskAnswer::Refused }]
        );
    }
}
