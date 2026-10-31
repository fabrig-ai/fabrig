use super::*;

#[test]
fn opaque_fifo_wakes_waiting_and_read_advances_only_on_main_turn() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.running(1, 9);
    let call = Token::new(20);
    let wait = Event::Delegated {
        conversation,
        call,
        name: crate::CallName { activation: 1, completion: 1, position: 0 },
        ask: Ask::Wait,
        deadline: Time::from_nanos(u64::MAX),
    };
    assert_eq!(harness.step(wait).as_ref(), &[Request::Waiting { host_run: Token::new(1), read: None }]);
    for (name, text) in [(0_u64, b"first".as_slice()), (99_u64, b"second".as_slice()), (7_u64, b"third".as_slice())] {
        let got = harness.step(Event::Message {
            finish_requested: false,
            run,
            name: Token::new(name),
            label: bytes(b"person"),
            text: bytes(text),
        });
        if name == 0 {
            assert_eq!(got.as_ref(), &[returned(20, Returned::Waited { text: crate::prompt::waited() })]);
        } else {
            assert!(got.is_empty());
        }
    }
    assert_eq!(
        harness.step(Event::Turn { conversation, record: Token::new(50), sequence: 1, opens_window: true }).as_ref(),
        &[Request::Turn {
            host_run: Token::new(1),
            record: Token::new(50),
            number: 1,
            position: 1,
            opens_window: true,
            read: None,
            spent: Spend::ZERO,
        }]
    );
    assert_eq!(
        harness.step(end_turn(conversation)).as_ref(),
        &[Request::Say { peer: Token::new(9), text: bytes(b"person: first\n\nperson: second\n\nperson: third") }]
    );
    assert_eq!(
        harness.step(Event::Turn { conversation, record: Token::new(51), sequence: 2, opens_window: false }).as_ref(),
        &[Request::Turn {
            host_run: Token::new(1),
            record: Token::new(51),
            number: 2,
            position: 2,
            opens_window: false,
            read: Some(Token::new(7)),
            spent: Spend::ZERO,
        }]
    );
    assert_eq!(
        harness
            .step(Event::Delegated {
                conversation,
                call,
                name: crate::CallName { activation: 1, completion: 3, position: 0 },
                ask: Ask::Wait,
                deadline: Time::from_nanos(u64::MAX),
            })
            .as_ref(),
        &[Request::Waiting { host_run: Token::new(1), read: Some(Token::new(7)) }]
    );
    harness.after(Duration::from_secs(30));
    assert_eq!(harness.fire().as_ref(), &[returned(20, Returned::Withheld), Request::Close { peer: Token::new(9) }]);
    harness.step(Event::Turn { conversation, record: Token::new(52), sequence: 3, opens_window: false });
    assert_eq!(
        answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))),
        (1, Answer::Parked { spent: Spend::ZERO, turns: 3 })
    );
    harness.domain.reclaim();
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
}

#[test]
fn zero_waiting_time_parks_with_its_wait_result_withheld() {
    let mut harness = Harness::new(LIMITS);
    let policy = Charter { waiting: Duration::ZERO, ..charter() };
    let requests = harness.start_workspace(70, policy, None);
    let [Request::Admitted { .. }, Request::Open { conversation, .. }] = requests.as_ref() else {
        panic!("opens");
    };
    let conversation = *conversation;
    harness.step(Event::Started { conversation, peer: Token::new(71) });
    assert_eq!(
        harness
            .step(Event::Delegated {
                conversation,
                call: Token::new(72),
                name: crate::CallName { activation: 1, completion: 1, position: 0 },
                ask: Ask::Wait,
                deadline: EXPIRY,
            })
            .as_ref(),
        &[
            Request::Waiting { host_run: Token::new(70), read: None },
            returned(72, Returned::Withheld),
            Request::Close { peer: Token::new(71) }
        ]
    );
    harness.step(Event::Turn { conversation, record: Token::new(73), sequence: 1, opens_window: true });
    assert_eq!(
        answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))),
        (70, Answer::Parked { spent: Spend::ZERO, turns: 1 })
    );
}

#[test]
fn bounded_message_and_input_at_idle_deadline_preserve_existing_fifo() {
    let limits = Limits { messages: 1, message_bytes: 10, ..LIMITS };
    let mut harness = Harness::new(limits);
    let (run, conversation) = harness.running(1, 9);
    assert!(
        harness
            .step(Event::Message {
                finish_requested: false,
                label: Box::from(&b"host"[..]),
                run,
                name: Token::new(5),
                text: bytes(b"full")
            })
            .is_empty()
    );
    assert_eq!(
        harness.step(end_turn(conversation)).as_ref(),
        &[Request::Say { peer: Token::new(9), text: bytes(b"host: full") }]
    );
    harness.step(Event::Turn { conversation, record: Token::new(50), sequence: 1, opens_window: true });
    assert_eq!(
        harness
            .step(Event::Delegated {
                conversation,
                call: Token::new(20),
                name: crate::CallName { activation: 1, completion: 2, position: 0 },
                ask: Ask::Wait,
                deadline: EXPIRY,
            })
            .as_ref(),
        &[Request::Waiting { host_run: Token::new(1), read: Some(Token::new(5)) }]
    );
    harness.after(Duration::from_secs(30));
    assert_eq!(
        &*harness.step(Event::Message {
            finish_requested: false,
            label: Box::from(&b"host"[..]),
            run,
            name: Token::new(0),
            text: bytes(b"wake")
        }),
        &[returned(20, Returned::Waited { text: crate::prompt::waited() })]
    );
    assert_eq!(
        harness.step(end_turn(conversation)).as_ref(),
        &[Request::Say { peer: Token::new(9), text: bytes(b"host: wake") }]
    );
    assert!(!harness.domain.is_due(harness.env.now), "same-iteration input cancels idle expiry before fire");
    assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: Token::new(9) }]);
    assert_eq!(
        &*harness.step(ended(conversation, End::Closed, Spend::ZERO)),
        &[Request::Answer {
            to: ReplyTo::new(Token::new(1)),
            answer: Answer::Failed { failure: Failure::Cancelled, spent: Spend::ZERO, turns: 1 },
            read: Some(Token::new(5)),
            detail: Box::default(),
        }]
    );
    harness.domain.reclaim();
    assert_eq!(harness.domain.runs(), 0);
}

#[test]
fn labelled_message_ingress_checks_rendered_bytes_names_and_inbox_before_retaining() {
    let mut harness = Harness::new(Limits { message_bytes: 12, messages: 2, ..LIMITS });
    let (run, _) = harness.running(1, 9);
    let message = |name, text: &[u8]| Event::Message {
        finish_requested: false,
        run,
        name: Token::new(name),
        label: bytes(b"peer"),
        text: bytes(text),
    };
    assert!(harness.step(message(10, b"123456")).is_empty(), "exact rendered bound is admitted");
    for (name, text, reason) in [
        (11, b"1234567".as_slice(), crate::MessageRefusal::TooLarge),
        (10, b"x".as_slice(), crate::MessageRefusal::NameInUse),
    ] {
        assert_eq!(
            &*harness.step(message(name, text)),
            &[Request::MessageRefused { host_run: Token::new(1), name: Token::new(name), reason }]
        );
    }
    assert!(harness.step(message(12, b"x")).is_empty(), "refused inputs consume no inbox room");
    assert_eq!(
        &*harness.step(message(13, b"x")),
        &[Request::MessageRefused {
            host_run: Token::new(1),
            name: Token::new(13),
            reason: crate::MessageRefusal::Full
        }]
    );
    harness.step(Event::Cancel { run });
    assert_eq!(
        &*harness.step(message(14, b"x")),
        &[Request::MessageRefused {
            host_run: Token::new(1),
            name: Token::new(14),
            reason: crate::MessageRefusal::Ending
        }]
    );
}

#[test]
fn preparation_refuses_bad_messages_without_disrupting_the_admitted_opening() {
    let mut harness = Harness::new(Limits { message_bytes: 12, messages: 1, ..LIMITS });
    let run = harness.prepare(1);
    assert_eq!(
        &*harness.step(Event::Message {
            finish_requested: false,
            run,
            name: Token::new(1),
            label: bytes(b"peer"),
            text: bytes(b"1234567")
        }),
        &[Request::MessageRefused {
            host_run: Token::new(1),
            name: Token::new(1),
            reason: crate::MessageRefusal::TooLarge
        }]
    );
    assert!(
        harness
            .step(Event::Message {
                finish_requested: false,
                run,
                name: Token::new(2),
                label: bytes(b"peer"),
                text: bytes(b"123456")
            })
            .is_empty()
    );
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Open { conversation, opening }] = &*requests else { panic!("prepared run opens main") };
    assert_eq!(opening.prompt.as_ref(), b"Begin the work your brief describes.\n\npeer: 123456");
    let main = *conversation;
    harness.step(Event::Started { conversation: main, peer: Token::new(9) });
    let requests =
        harness.step(Event::Turn { conversation: main, record: Token::new(40), sequence: 1, opens_window: true });
    let [Request::Turn { read, .. }] = requests.as_ref() else { panic!("one told turn") };
    assert_eq!(*read, Some(Token::new(2)));
    harness.step(end_turn(main));
}

#[test]
fn messages_during_stopping_over_winding_and_closed_each_have_an_ending_terminal() {
    let mut preparing = Harness::new(LIMITS);
    let stopping = preparing.prepare(1);
    preparing.step(Event::Cancel { run: stopping });
    assert_eq!(
        &*preparing.step(Event::Message {
            finish_requested: false,
            run: stopping,
            name: Token::new(20),
            label: bytes(b"peer"),
            text: bytes(b"stopping"),
        }),
        &[Request::MessageRefused {
            host_run: Token::new(1),
            name: Token::new(20),
            reason: crate::MessageRefusal::Ending,
        }]
    );

    let mut running = Harness::new(LIMITS);
    let (run, main) = running.running(2, 9);
    running.step(Event::Used { conversation: main, spend: spend(BUDGET.spend) });
    for (name, phase) in [(21, "over"), (22, "winding"), (23, "closed")] {
        assert_eq!(
            &*running.step(Event::Message {
                finish_requested: false,
                run,
                name: Token::new(name),
                label: bytes(b"peer"),
                text: bytes(phase.as_bytes()),
            }),
            &[Request::MessageRefused {
                host_run: Token::new(2),
                name: Token::new(name),
                reason: crate::MessageRefusal::Ending,
            }],
            "{phase} refuses each message once"
        );
        match name {
            21 => {
                running.step(end_turn(main));
            }
            22 => {
                running.step(ended(main, End::Closed, spend(BUDGET.spend)));
            }
            23 => {}
            _ => unreachable!("three known phases"),
        }
    }
}

#[test]
fn ordered_offers_end_each_message_read_or_unread_and_the_answer_keeps_the_last_told_fence() {
    for (offer_messages, offer_bytes) in [(2, 128), (5, 20)] {
        let mut harness = Harness::new(Limits { messages: 5, message_bytes: 9, offer_messages, offer_bytes, ..LIMITS });
        let (run, main) = harness.running(1, 9);
        for name in [99, 0, 8, 7, 6] {
            assert!(
                harness
                    .step(Event::Message {
                        finish_requested: false,
                        run,
                        name: Token::new(name),
                        label: bytes(b"peer"),
                        text: bytes(b"aaa")
                    })
                    .is_empty()
            );
        }
        assert_eq!(
            &*harness.step(end_turn(main)),
            &[Request::Say { peer: Token::new(9), text: bytes(b"peer: aaa\n\npeer: aaa") }]
        );
        let told =
            harness.step(Event::Turn { conversation: main, record: Token::new(40), sequence: 1, opens_window: true });
        let [Request::Turn { read, .. }] = &*told else { panic!("one told turn") };
        assert_eq!(*read, Some(Token::new(0)), "names are opaque, including a fence of zero");
        assert_eq!(
            &*harness.step(end_turn(main)),
            &[Request::Say { peer: Token::new(9), text: bytes(b"peer: aaa\n\npeer: aaa") }]
        );
        harness.step(Event::Cancel { run });
        let terminal = harness.step(ended(main, End::Closed, Spend::ZERO));
        let [Request::Answer { read, .. }] = &*terminal else { panic!("one run terminal") };
        assert_eq!(*read, Some(Token::new(0)), "an untold offer cannot move the final fence");
        let mut terminals = alloc::collections::BTreeMap::new();
        while let Some(fact) = harness.pop_fact() {
            match fact.kind {
                Fact::MessageRead { name, turn, .. } => {
                    assert_eq!(turn, 1);
                    assert!(terminals.insert(name, "read").is_none(), "each message ends once");
                }
                Fact::MessageUnread { name, .. } => {
                    assert!(terminals.insert(name, "unread").is_none(), "each message ends once");
                }
                Fact::MessageFence { .. }
                | Fact::MessageReceived { .. }
                | Fact::MessageRefused { .. }
                | Fact::Admitted { .. }
                | Fact::Budget { .. }
                | Fact::Prepared { .. }
                | Fact::Opened { .. }
                | Fact::Ended { .. }
                | Fact::Called { .. }
                | Fact::Returned { .. }
                | Fact::CheckStarted { .. }
                | Fact::CheckFinished { .. }
                | Fact::Delivered { .. }
                | Fact::Answered { .. } => {}
            }
        }
        assert_eq!(terminals.len(), 5, "each accepted message ends");
        for name in [99, 0] {
            assert_eq!(terminals[&Token::new(name)], "read");
        }
        for name in [8, 7, 6] {
            assert_eq!(terminals[&Token::new(name)], "unread", "offered and queued both end unread");
        }
    }
}

#[test]
fn carried_messages_open_after_the_briefs_instruction_and_before_later_messages() {
    let mut harness = Harness::new(LIMITS);
    let requests = harness.step(carried_start(
        charter(),
        Box::new([carried(99, b"first"), carried(0, b"second")]),
        Some(workspace()),
    ));
    let [Request::Admitted { run, .. }, Request::Read { .. }] = &*requests else { panic!("admitted preparation") };
    let run = *run;
    assert!(
        harness
            .step(Event::Message {
                finish_requested: false,
                run,
                name: Token::new(8),
                label: bytes(b"peer"),
                text: bytes(b"third")
            })
            .is_empty()
    );
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Open { conversation, opening }] = &*requests else { panic!("one opening") };
    let main = *conversation;
    assert_eq!(
        &*opening.prompt,
        b"Begin the work your brief describes.\n\nperson: first\n\nperson: second\n\npeer: third"
    );
    harness.step(Event::Started { conversation: main, peer: Token::new(9) });
    let requests =
        harness.step(Event::Turn { conversation: main, record: Token::new(40), sequence: 1, opens_window: true });
    let [Request::Turn { read, .. }] = &*requests else { panic!("one told turn") };
    assert_eq!(*read, Some(Token::new(8)));
    let mut reads = List::with_capacity(LIMITS.messages);
    for _ in 0..harness.observed.len().checked_add(harness.domain.pending_facts()).expect("held facts fit u32") {
        if let Some(fact) = harness.pop_fact()
            && let Fact::MessageRead { name, .. } = fact.kind
        {
            reads.push(name).expect("bounded read count");
        }
    }
    assert_eq!(reads.as_slice(), [Token::new(99), Token::new(0), Token::new(8)]);
}

#[test]
fn an_empty_brief_opens_on_carried_messages_or_on_its_instructions_without_wait() {
    let batches: [Box<[crate::Message]>; 2] = [Box::new([]), Box::new([carried(0, b"work")])];
    for messages in batches {
        let mut charter = charter();
        charter.brief.sections = Box::new([]);
        charter.grants.wait = false;
        charter.instructions = bytes(b"Follow these instructions.");
        let expected: &[u8] =
            if messages.is_empty() { b"Begin the work your instructions describe." } else { b"person: work" };
        let mut harness = Harness::new(LIMITS);
        let requests = harness.step(carried_start(charter, messages, None));
        let [Request::Admitted { .. }, Request::Open { opening, .. }] = &*requests else {
            panic!("one immediate opening")
        };
        assert_eq!(&*opening.prompt, expected);
        assert!(skein_lib::bytes::find(&opening.system, b"Follow these instructions.").is_some());
    }
}

#[test]
fn a_start_refuses_all_carried_messages_atomically_at_its_count_byte_and_name_bounds() {
    let limits = Limits { messages: 2, message_bytes: 9, ..LIMITS };
    let batches: [Box<[crate::Message]>; 3] = [
        Box::new([carried(1, b"a"), carried(2, b"a"), carried(3, b"a")]),
        Box::new([carried(1, b"aa")]),
        Box::new([carried(0, b"a"), carried(0, b"a")]),
    ];
    for messages in batches {
        let mut harness = Harness::new(limits);
        let requests = harness.step(carried_start(charter(), messages, None));
        assert_eq!(answered(requests), (1, Answer::Refused(Refusal::Invalid(Invalid::Messages))));
        assert!(harness.pop_fact().is_none(), "no partial message admission");
        let requests = harness.step(carried_start(charter(), Box::new([carried(0, b"a")]), None));
        let [Request::Admitted { .. }, Request::Open { opening, .. }] = &*requests else {
            panic!("exact bound admitted after refusal")
        };
        assert!(opening.prompt.ends_with(b"person: a"));
    }
}

#[test]
fn prepared_empty_brief_awaits_first_message_without_opening_main() {
    let mut harness = Harness::new(LIMITS);
    let requests = harness.start_workspace(80, message_driven_charter(), None);
    let [Request::Admitted { run, .. }, Request::Waiting { host_run, read }] = requests.as_ref() else {
        panic!("prepared main awaits without a session: {requests:?}")
    };
    let run = *run;
    assert_eq!((*host_run, *read), (Token::new(80), None));
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (1, 1, 0));
    assert_eq!(harness.domain.next_deadline(), Some(Time::ZERO.saturating_add(message_driven_charter().waiting)));
    harness.after(Duration::from_secs(1));
    let requests = harness.step(Event::Message {
        finish_requested: false,
        run,
        name: Token::new(0),
        label: bytes(b"person"),
        text: bytes(b"real task"),
    });
    let [Request::Open { conversation, opening }] = requests.as_ref() else {
        panic!("first message opens main: {requests:?}")
    };
    assert_eq!(opening.prompt.as_ref(), b"person: real task");
    let conversation = *conversation;
    assert!(harness.step(Event::Started { conversation, peer: Token::new(81) }).is_empty());
    let requests = harness.step(Event::Turn { conversation, record: Token::new(82), sequence: 1, opens_window: true });
    let [Request::Turn { read, number, spent, .. }] = requests.as_ref() else { panic!("first turn") };
    assert_eq!((*read, *number, *spent), (Some(Token::new(0)), 1, Spend::ZERO));
    let facts = observed_facts(&mut harness);
    let mut prepared = 0_u32;
    for fact in &facts {
        if let Fact::Prepared { .. } = fact.kind {
            prepared += 1;
            assert_eq!(fact.at, Time::ZERO, "prepared time precedes later work");
        }
    }
    assert_eq!(prepared, 1, "one preparation even when its opening comes later");
}

#[test]
fn a_message_before_preparation_opens_directly_and_one_after_preparation_wakes_once() {
    for before in [true, false] {
        let mut harness = Harness::new(LIMITS);
        let requests = harness.start(83, message_driven_charter());
        let [Request::Admitted { run, .. }, Request::Read { .. }] = requests.as_ref() else { panic!("preparation") };
        let run = *run;
        if before {
            assert!(
                harness
                    .step(Event::Message {
                        finish_requested: false,
                        run,
                        name: Token::new(1),
                        label: bytes(b"person"),
                        text: bytes(b"task")
                    })
                    .is_empty()
            );
        }
        let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
        let requests = if before {
            requests
        } else {
            assert_eq!(requests.as_ref(), [Request::Waiting { host_run: Token::new(83), read: None }]);
            harness.step(Event::Message {
                finish_requested: false,
                run,
                name: Token::new(1),
                label: bytes(b"person"),
                text: bytes(b"task"),
            })
        };
        let [Request::Open { opening, .. }] = requests.as_ref() else { panic!("one task opening") };
        assert_eq!(opening.prompt.as_ref(), b"person: task");
    }
}

#[test]
fn awaiting_cancellation_retires_unopened_main_and_answers_once() {
    let mut harness = Harness::new(LIMITS);
    let requests = harness.start_workspace(84, message_driven_charter(), None);
    let [Request::Admitted { run, .. }, Request::Waiting { .. }] = requests.as_ref() else { panic!("awaiting") };
    let run = *run;
    assert_eq!(answered(harness.step(Event::Cancel { run })), (84, failed(Failure::Cancelled, Spend::ZERO)));
    assert!(harness.step(Event::Cancel { run }).is_empty());
    assert_eq!(harness.domain.next_deadline(), None);
    harness.domain.reclaim();
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
}

#[test]
fn awaiting_idle_wall_and_zero_waiting_deadlines_park_without_usage_or_session() {
    for (waiting, time) in [(7, 30), (30, 7), (0, 30)] {
        let mut harness = Harness::new(LIMITS);
        let charter = Charter {
            waiting: Duration::from_secs(waiting),
            budget: Budget { time: Duration::from_secs(time), ..BUDGET },
            ..message_driven_charter()
        };
        let requests = harness.start_workspace(85, charter, None);
        if waiting == 0 {
            let [
                Request::Admitted { .. },
                Request::Waiting { read: None, .. },
                Request::Answer { answer, read: None, .. },
            ] = requests.as_ref()
            else {
                panic!("immediate awaiting park: {requests:?}")
            };
            assert_eq!(*answer, Answer::Parked { spent: Spend::ZERO, turns: 0 });
        } else {
            let [Request::Admitted { .. }, Request::Waiting { read: None, .. }] = requests.as_ref() else {
                panic!("awaiting")
            };
            harness.after(Duration::from_secs(waiting.min(time)));
            assert_eq!(answered(harness.fire()), (85, Answer::Parked { spent: Spend::ZERO, turns: 0 }));
        }
        assert_eq!(harness.domain.next_deadline(), None);
        harness.domain.reclaim();
        assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
    }
}

#[test]
fn awaiting_keeps_resume_binding_until_first_message_opens_main() {
    let mut harness = Harness::new(LIMITS);
    let mut event = carried_start(Charter { resume: true, ..message_driven_charter() }, Box::default(), None);
    if let Event::Start { transcript, activation, .. } = &mut event {
        *transcript = Some(Token::new(900));
        *activation = 2;
    }
    let requests = harness.step(event);
    let [Request::Admitted { run, .. }, Request::Waiting { .. }] = requests.as_ref() else { panic!("awaiting") };
    let requests = harness.step(Event::Message {
        finish_requested: false,
        run: *run,
        name: Token::new(1),
        label: bytes(b"person"),
        text: bytes(b"new task"),
    });
    let [Request::Open { opening, .. }] = requests.as_ref() else { panic!("restored main opens") };
    assert_eq!(opening.transcript, Some(Token::new(900)));
    assert_eq!(opening.activation, 2);
    assert_eq!(opening.prompt.as_ref(), b"person: new task");
}

#[test]
fn a_finish_crossing_a_message_continues_only_with_wait_authority() {
    for wait in [true, false] {
        let mut harness = Harness::new(LIMITS);
        let mut charter = text_charter(false);
        charter.grants.wait = wait;
        let (run, main) = harness.running_on(95, 100, charter);
        assert!(
            harness
                .step(Event::Message {
                    finish_requested: false,
                    run,
                    name: Token::new(0),
                    label: bytes(b"person"),
                    text: bytes(b"late work")
                })
                .is_empty()
        );
        let requests = harness.step(finish(main, 96, text_result(false, b"done")));
        if wait {
            assert_eq!(requests.as_ref(), [returned(96, Returned::Crossed { text: crate::prompt::crossed() })]);
            let requests = harness.step(end_turn(main));
            let [Request::Say { peer, text }] = requests.as_ref() else { panic!("queued work is the continuation") };
            assert_eq!((*peer, text.as_ref()), (PEER, b"person: late work".as_slice()));
            let requests = harness.step(Event::Turn {
                conversation: main,
                record: Token::new(97),
                sequence: 1,
                opens_window: true,
            });
            let [Request::Turn { read, .. }] = requests.as_ref() else { panic!("told turn fence") };
            assert_eq!(*read, Some(Token::new(0)));
            assert_eq!(
                harness.step(finish(main, 98, text_result(false, b"done"))).as_ref(),
                [returned(98, Returned::Accepted), Request::Close { peer: PEER }]
            );
        } else {
            assert_eq!(requests.as_ref(), [returned(96, Returned::Accepted), Request::Close { peer: PEER }]);
        }
        harness.step(ended(main, End::Closed, Spend::ZERO));
        let facts = observed_facts(&mut harness);
        let mut read = 0_u32;
        let mut unread = 0_u32;
        for fact in facts {
            if let Fact::MessageRead { name, .. } = fact.kind
                && name == Token::new(0)
            {
                read += 1;
            }
            if let Fact::MessageUnread { name, .. } = fact.kind
                && name == Token::new(0)
            {
                unread += 1;
            }
        }
        assert_eq!((read, unread), if wait { (1, 0) } else { (0, 1) });
    }
}
