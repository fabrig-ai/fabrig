use super::*;

#[test]
fn each_settled_answer_gives_its_last_line() {
    let failed = run::Failure::Model(run::Fault::Limited { account: 7, reset_at: Some(Time::from_nanos(75_000_000)) });
    let refusal = run::Refusal::Invalid(run::Invalid::Workspace);
    let now = Time::from_nanos(2_000_000);
    let rows = [
        (run::Answer::Parked { spent: run::Spend::ZERO, turns: 0 }, None, None),
        (run::Answer::Failed { failure: failed.clone(), spent: run::Spend::ZERO, turns: 0 }, None,
            Some(smith_protocol_events::failure_record(failed, now))),
        (run::Answer::Refused(refusal), None, Some(smith_protocol_events::refusal_record(refusal))),
        (run::Answer::Parked { spent: run::Spend::ZERO, turns: 0 }, Some(Failure::Send), None),
    ];
    for (answer, unsent, expected) in rows {
        let mut service = without_channel();
        crate::terminal::remember(&mut service.stages, &answer, Box::from(b"detail".as_slice()), now);
        if let Some(reason) = unsent {
            service.stages.mark_failed(reason);
        }
        teardown::answered(&mut service.stages);
        for _ in 0..3 {
            teardown::advance(&mut service.stages, Some(&service.frame));
        }
        assert_eq!(done(&service), Some(unsent.is_none()));
        match crate::last(&service).expect("settled last line") {
            crate::Last::Quiet => assert!(expected.is_none() && unsent.is_none()),
            crate::Last::Failed { failure, detail } => {
                assert_eq!(Some(*failure), expected);
                assert_eq!(detail.as_ref(), b"detail");
            }
            crate::Last::Unanswered(reason) => assert_eq!(Some(reason), unsent),
        }
    }
}
