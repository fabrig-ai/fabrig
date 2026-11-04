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
            teardown::follow(&mut service.stages, Some(&service.frame));
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

#[test]
fn the_first_failure_stands_in_every_phase() {
    for phase in [
        Teardown::Live,
        Teardown::Answered { at: Time::ZERO, answer: Answer::Sent, close: Close::Graceful },
        Teardown::Closing { at: Time::ZERO, answer: Answer::Sent, close: Close::Graceful, step: Step::Components },
    ] {
        let mut service = service();
        service.stages.teardown = phase;
        service.stages.mark_failed(Failure::Send);
        service.stages.mark_failed(Failure::WriteFailed);
        assert_eq!(failure(&service), Some(Failure::Send));
        teardown::answered(&mut service.stages);
        match service.stages.teardown {
            Teardown::Answered { answer, .. } | Teardown::Closing { answer, .. } => {
                assert_eq!(answer, Answer::Unsent(Failure::Send));
            }
            other => panic!("answered phase: {other:?}"),
        }
    }
}

#[test]
fn a_failure_before_the_answer_leaves_the_run_live() {
    let mut service = admitted(19);
    service.stages.mark_failed(Failure::Send);
    teardown::follow(&mut service.stages, Some(&service.frame));
    assert_eq!(service.stages.teardown, Teardown::LiveUnsent { failure: Failure::Send });
    assert!(service.stages.io_requests.is_empty());
    assert!(!service.stages.llm.closed());
    assert_eq!(done(&service), None);
    route_io_event(&mut service.stages, Some(&mut service.frame),
        io::Event::Shutdown { signal: kernel::ServiceSignal::Interrupt, from: None });
    match service.stages.domain_events.pop().expect("live shutdown cancels") {
        domain::Event::Cancel { run } => assert_eq!(run, Token::new(19)),
        other => panic!("cancel: {other:?}"),
    }
}
