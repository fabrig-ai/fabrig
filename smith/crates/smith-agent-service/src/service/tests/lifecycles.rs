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
        (run::Answer::Refused(refusal.clone()), None, Some(smith_protocol_events::refusal_record(&refusal))),
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
    channel_event(&mut service.stages, &mut service.frame, channel::Heard::WriteFailed);
    teardown::follow(&mut service.stages, Some(&service.frame));
    assert_eq!(service.stages.teardown, Teardown::Live { failure: Some(Failure::WriteFailed) });
    assert!(service.stages.io_requests.is_empty());
    assert!(!service.stages.llm.closed());
    assert_eq!(done(&service), None);
    let Some(domain::Event::Disconnected { run }) = service.stages.domain_events.pop() else {
        panic!("failed output releases durable credit before cancellation settles")
    };
    assert_eq!(run, Token::new(19));
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
                results: None,
                hooks: None,
                name: Box::from(b"first".as_slice()),
                path: Box::from(b"/first".as_slice()),
                writable: true,
                git: true,
                conflicts: Box::default(),
            },
            smith_protocol_translate::Mount {
                results: None,
                hooks: None,
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
        channel_event(&mut service.stages, &mut service.frame, channel::Heard::Drain { grace: None });
        assert_eq!(service.stages.admission, Admission::Draining { grace: None });
        assert!(service.stages.domain_events.is_empty());
        if cancel {
            channel_event(&mut service.stages, &mut service.frame, channel::Heard::Cancel);
        }
        service.stages.admission.admit(run);
        domain_down(&mut service.stages, Some(&mut service.frame));
        let event = service.stages.domain_events.pop().expect("held control delivered");
        match event {
            domain::Event::Cancel { run: owner } => {
                assert!(cancel);
                assert_eq!(owner, run);
            }
            domain::Event::Drain { run: owner, grace: None } => {
                assert!(!cancel);
                assert_eq!(owner, run);
            }
            other => panic!("held control, got {other:?}"),
        }
        domain_down(&mut service.stages, Some(&mut service.frame));
        assert!(service.stages.domain_events.is_empty());
    }
}

fn start_stubborn_command(stages: &mut Stages) -> (Token, Token, [Token; 2]) {
    use smith_protocol_machine::{Below, BelowEvent, FromDomain};

    let root = Token::new(44);
    let owner = Token::new(51);
    let child = Token::new(52);
    let pipes = [Token::new(53), Token::new(54)];
    stages.machine.workspace(&run::Workspace {
        directories: Box::new([run::Directory {
            root,
            host: b"/workspace".as_slice().into(),
            name: b"workspace".as_slice().into(),
            writable: true,
            git: false,
            conflicts: Box::new([]),
            results: None,
            hooks: None,
        }]),
    });
    stages.machine.from_domain(
        &stages.machine_env,
        FromDomain::Op {
            owner,
            op: domain::tools::Op::Spawn {
                cwd: domain::tools::Place { root, path: Box::new([]) },
                command: b"hold".as_slice().into(),
                head: 1,
                tail: 1,
                after: None,
            },
            deadline: Time::ZERO.saturating_add(stages.machine_env.limits.group_stop),
        },
        &mut stages.machine_events,
        &mut stages.machine_below,
    );
    let Some(Below::Spawn { owner: started, .. }) = stages.machine_below.pop() else { panic!("spawn requested") };
    assert_eq!(started, owner);
    stages.machine.from_below(
        &stages.machine_env,
        BelowEvent::Process(io::Event::Spawned { owner, child, pipes: Box::new(pipes) }),
        &mut stages.machine_events,
        &mut stages.machine_below,
    );
    stages.machine_below.clear(); // The fake command stays open without answering its pipe demands.
    (owner, child, pipes)
}

#[test]
fn an_orphan_past_cancel_grace_drops_the_run_and_aborts_the_group() {
    use super::super::{orphan, stages};
    use smith_protocol_machine::{Below, BelowEvent};

    let mut service = admit_pending(event_service(), 19);
    let stages = &mut service.stages;
    let grace = stages.limits.link.cancel_grace();
    stages.machine_env.limits.group_stop = grace.checked_add(stages.limits.io.close_timeout).expect("long stop");
    let (owner, child, pipes) = start_stubborn_command(stages);

    orphan::host_lost(stages, Failure::ChannelEnded);
    orphan::abandon(stages, &mut service.frame);
    let Some(domain::Event::Disconnected { run }) = stages.domain_events.pop() else { panic!("orphan cancel") };
    assert_eq!(run, Token::new(19));
    domain_request(stages, Some(&mut service.frame), domain::Request::Lower(domain::Lower::CancelIo { owner }));
    let Some(Below::Process(io::Request::Signal {
        child: id,
        to: kernel::Target::Group,
        signal: kernel::Signal::Terminate,
        ..
    })) = stages.machine_below.pop()
    else {
        panic!("cancel terminates the group first")
    };
    assert_eq!(id, child);
    let drop_at = Time::ZERO.saturating_add(grace);
    let before = Time::from_nanos(grace.as_nanos().checked_sub(1).expect("positive grace"));
    stages::clocks(stages, before, Wall::EPOCH);
    orphan::expire(stages);
    assert_eq!(stages.teardown, Teardown::Live { failure: Some(Failure::ChannelEnded) });
    assert_eq!(orphan::deadline(stages), Some(drop_at));
    assert!(stages.machine_below.is_empty(), "the grace has not expired");

    // The command ignored Terminate, so the machine still owes its terminal.
    stages::clocks(stages, drop_at, Wall::EPOCH);
    orphan::expire(stages);
    assert!(stages.host.dropped());
    assert_eq!(orphan::deadline(stages), None);
    let Some(Below::Process(io::Request::Close { entity })) = stages.machine_below.pop() else {
        panic!("drop closes the surviving group")
    };
    assert_eq!(entity, child);
    teardown::follow(stages, Some(&service.frame));
    for event in [
        BelowEvent::Process(io::Event::Closed { owner }),
        BelowEvent::Pipe(io::Event::Closed { owner: pipes[0] }),
        BelowEvent::Pipe(io::Event::Closed { owner: pipes[1] }),
    ] {
        stages.machine.from_below(&stages.machine_env, event, &mut stages.machine_events, &mut stages.machine_below);
    }
    component_up(stages, Some(&mut service.frame));
    assert!(stages.domain_events.is_empty(), "a dropped run hears no more terminals");
    teardown::follow(stages, Some(&service.frame));
    let Some(io::Request::Usage { owner }) = stages.io_requests.pop() else { panic!("usage request") };
    assert_eq!(owner, USAGE_OWNER);
    record_usage(stages, Some(&service.frame), io::Measured::Unread);
    assert_eq!(
        stages.teardown,
        Teardown::Closing {
            at: drop_at,
            answer: Answer::Unsent(Failure::ChannelEnded),
            close: Close::Aborted,
            step: Step::Stream,
        }
    );
    teardown::follow(stages, Some(&service.frame));
    assert_eq!(done(&service), Some(false));
    assert_eq!(failure(&service), Some(Failure::ChannelEnded));
}

#[test]
fn loss_while_start_awaits_admission_cancels_at_the_handle() {
    let mut service = with_channel_streams(None);
    let start = decoded_start(Box::new([]));
    channel_event(&mut service.stages, &mut service.frame, channel::Heard::Start { start });
    assert!(service.stages.admission.run().is_none());
    assert!(!service.stages.domain_events.is_empty(), "Start has crossed the framed face");
    channel_event(
        &mut service.stages,
        &mut service.frame,
        channel::Heard::Ended { why: skein_channel::Closed::Stream },
    );
    assert_eq!(service.stages.teardown, Teardown::Live { failure: Some(Failure::ChannelEnded) });
    for pipe in [Token::new(11), Token::new(12)] {
        let Some(io::Request::Abort { entity }) = service.stages.io_requests.pop() else {
            panic!("both channel ends are aborted")
        };
        assert_eq!(entity, pipe);
    }
    let run = Token::new(91);
    link::request(&mut service.stages, &mut service.frame, domain::Request::Admitted { host_run: START_REPLY, run });
    admission::follow(&mut service.stages);
    let Some(domain::Event::Disconnected { run: named }) = service.stages.domain_events.pop() else {
        panic!("admission delivers the pending disconnect before other work")
    };
    assert_eq!(named, run);
    assert!(service.stages.channel_below.is_empty(), "no admission frame to the lost host");
}
