use super::*;

#[test]
fn each_settled_answer_gives_its_last_line() {
    let failed =
        run::RunFailure::Model(run::Fault::Limited { account: 7, reset_at: Some(Time::from_nanos(75_000_000)) });
    let refusal = run::Refusal::Invalid(run::Invalid::Workspace);
    let now = Time::from_nanos(2_000_000);
    let rows = [
        (run::Answer::Parked { spent: run::Spend::ZERO, turns: 0 }, None, None),
        (
            run::Answer::Failed { failure: failed.clone(), spent: run::Spend::ZERO, turns: 0 },
            None,
            Some(smith_protocol_events::failure_record(failed, now)),
        ),
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
        for _ in 0_u32..3 {
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
        Teardown::Live { failure: None },
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
    assert_eq!(service.stages.teardown, Teardown::Live { failure: Some(Failure::Send) });
    assert!(service.stages.io_requests.is_empty());
    assert!(!service.stages.llm.closed());
    assert_eq!(done(&service), None);
    route_io_event(
        &mut service.stages,
        Some(&mut service.frame),
        io::Event::Shutdown { signal: kernel::ServiceSignal::Interrupt, from: None },
    );
    assert_cancel(&mut service.stages.domain_events, Token::new(19));
}

#[test]
fn a_cancel_before_the_handle_is_delivered_once_after_it() {
    use super::super::admission::Admission;
    let mut service = service();
    let run = Token::new(19);
    assert_eq!(service.stages.admission, Admission::Waiting);
    channel_event(&mut service.stages, &mut service.frame, channel::Heard::Cancel);
    assert_eq!(service.stages.admission, Admission::Cancelled);
    assert!(service.stages.domain_events.is_empty());
    service.stages.admission.admit(run);
    assert_eq!(service.stages.admission, Admission::Owed { run });
    assert!(service.stages.domain_events.is_empty());
    domain_down(&mut service.stages, Some(&mut service.frame));
    assert_eq!(service.stages.admission, Admission::Admitted { run });
    assert_cancel(&mut service.stages.domain_events, run);
    domain_down(&mut service.stages, Some(&mut service.frame));
    assert!(service.stages.domain_events.is_empty());
}

#[test]
fn a_waiting_cancel_goes_before_the_messages_that_waited_for_the_run() {
    let mut service = service();
    channel_event(&mut service.stages, &mut service.frame, channel::Heard::Cancel);
    channel_event(&mut service.stages, &mut service.frame, host_message(31));
    service.stages.domain_requests.push(domain::Request::Admitted { host_run: START_REPLY, run: Token::new(19) });
    domain_down(&mut service.stages, Some(&mut service.frame));
    assert_eq!(service.stages.pending_messages.len(), 1);
    assert_cancel(&mut service.stages.domain_events, Token::new(19));
    assert!(service.stages.domain_events.is_empty());
    domain_down(&mut service.stages, Some(&mut service.frame));
    assert!(service.stages.pending_messages.is_empty());
    match service.stages.domain_events.pop().expect("next pass flushes message") {
        domain::Event::Message { run, name, .. } => {
            assert_eq!(run, Token::new(19));
            assert_eq!(name, Token::new(31));
        }
        other => panic!("message: {other:?}"),
    }
    assert!(service.stages.domain_events.is_empty());
}

#[test]
fn a_start_attaches_its_mounts_in_order_then_enters_the_domain() {
    fn start() -> Box<smith_protocol_translate::DecodedStart> {
        decoded_start(Box::new([
            smith_protocol_translate::Mount {
                name: Box::from(b"first".as_slice()),
                path: Box::from(b"/first".as_slice()),
                writable: true,
                git: true,
                conflicts: Box::default(),
            },
            smith_protocol_translate::Mount {
                name: Box::from(b"second".as_slice()),
                path: Box::from(b"/second".as_slice()),
                writable: false,
                git: false,
                conflicts: Box::default(),
            },
        ]))
    }
    let mut agent = service();
    let decoded = start();
    let expected = decoded.charter.outcome.clone();
    channel_event(&mut agent.stages, &mut agent.frame, channel::Heard::Start { start: decoded });
    assert_eq!(agent.root_to_open(), Some(b"/first".as_slice()));
    assert!(agent.stages.domain_events.is_empty());
    agent.root_opened(Ok(kernel::Fd::new(90)), Box::from(b"/host/first".as_slice()));
    assert_eq!(agent.root_to_open(), Some(b"/second".as_slice()));
    assert!(agent.stages.domain_events.is_empty());
    agent.root_opened(Ok(kernel::Fd::new(91)), Box::from(b"/host/second".as_slice()));
    assert!(agent.root_to_open().is_none());
    assert_eq!(agent.stages.opening.outcome(), &expected);
    match agent.stages.domain_events.pop().expect("all mounts enter with Start") {
        domain::Event::Start(start) => {
            let workspace = start.workspace.expect("attached workspace");
            assert_eq!(workspace.directories.len(), 2);
            for (directory, (name, host, writable)) in workspace.directories.iter().zip([
                (b"first".as_slice(), b"/host/first".as_slice(), true),
                (b"second".as_slice(), b"/host/second".as_slice(), false),
            ]) {
                assert_eq!(directory.name.as_ref(), name);
                assert_eq!(directory.host.as_ref(), host);
                assert_eq!(directory.writable, writable);
                assert_eq!(directory.git, writable);
            }
        }
        other => panic!("Start: {other:?}"),
    }
    assert!(agent.stages.domain_events.is_empty());
    let mut refused = service();
    channel_event(&mut refused.stages, &mut refused.frame, channel::Heard::Start { start: start() });
    refused.root_opened(Err(()), Box::default());
    assert!(refused.root_to_open().is_none());
    match refused.stages.teardown {
        Teardown::Answered { .. } | Teardown::Closing { .. } | Teardown::Closed { .. } => {}
        other @ Teardown::Live { .. } => panic!("refused Start passed Live: {other:?}"),
    }
}


#[test]
fn a_drain_before_admission_is_delivered_once_and_cancel_can_override_it() {
    use super::super::admission::Admission;
    for cancel in [false, true] {
        let mut service = service();
        let run = Token::new(19);
        channel_event(&mut service.stages, &mut service.frame, channel::Heard::Drain);
        assert_eq!(service.stages.admission, Admission::Draining);
        assert!(service.stages.domain_events.is_empty());
        if cancel { channel_event(&mut service.stages, &mut service.frame, channel::Heard::Cancel); }
        service.stages.admission.admit(run);
        domain_down(&mut service.stages, Some(&mut service.frame));
        let event = service.stages.domain_events.pop().expect("held control delivered");
        assert!(if cancel { matches!(event, domain::Event::Cancel { run: owner } if owner == run) }
            else { matches!(event, domain::Event::Drain { run: owner } if owner == run) });
        domain_down(&mut service.stages, Some(&mut service.frame));
        assert!(service.stages.domain_events.is_empty());
    }
}
