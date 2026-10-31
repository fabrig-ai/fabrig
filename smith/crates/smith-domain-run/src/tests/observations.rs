use super::*;

#[test]
fn mains_end_names_no_call_in_its_budget() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    drop(harness.step(finish(main, 8, verdict(b"approve", Box::new([])))));
    drop(facts(&mut harness));
    drop(harness.step(ended(main, End::Closed, spend(50))));
    let mut snapshots = List::with_capacity(2);
    for fact in &facts(&mut harness) {
        if let Fact::Budget { call, .. } = fact {
            snapshots.push(*call).expect("two slots admit any duplicate");
        }
    }
    assert_eq!(snapshots.as_slice(), &[None]);
}

#[test]
fn a_run_tells_what_it_did_as_content_free_facts() {
    let mut harness = Harness::new(LIMITS);
    let reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(60) };
    let requests = harness.start(1, Charter { reserve, ..agents() });
    let [Request::Admitted { run, .. }, Request::Read { .. }] = &*requests else { panic!("expected a read") };
    let run = *run;
    drop(harness.step(Event::Read { owner: run, read: Read::Text { text: bytes(b"Be kind."), whole: true } }));
    let main = Token::new(0);
    drop(harness.step(Event::Started { conversation: main, peer: PEER }));
    let asked = crate::Share { turns: Some(u32::MAX), time: Some(BUDGET.time) };
    let opened = harness.step(ask(main, 7, families(true, false, false), None, Some(asked)));
    let [Request::Open { conversation: child, .. }] = opened.as_ref() else { panic!("child opens") };
    let child = *child;
    drop(harness.step(Event::Started { conversation: child, peer: Token::new(101) }));
    harness.after(Duration::from_secs(3));
    drop(harness.step(Event::Used { conversation: child, spend: spend(100) }));
    harness.after(Duration::from_secs(2));
    drop(harness.step(Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"done") }));
    drop(harness.step(ended(child, End::Closed, spend(100))));
    drop(harness.step(finish(main, 8, verdict(b"approve", Box::new([])))));
    drop(harness.step(ended(main, End::Closed, Spend::ZERO)));
    let told = facts(&mut harness);
    let expected = [
        Fact::Admitted { run, resumed: false },
        Fact::Prepared { run, guides: 1, checks: 0 },
        Fact::Opened { run, conversation: main, kind: crate::facts::Kind::Main },
        Fact::Called { run, conversation: main, call: Token::new(7), ask: Asked::SubAgent },
        Fact::Opened {
            run,
            conversation: child,
            kind: crate::facts::Kind::Child {
                parent: main,
                call: Token::new(7),
                asked,
                granted: crate::Granted {
                    turns: 8,
                    time: Duration::from_secs(540),
                    spend: BUDGET.spend - 2,
                    turns_set: crate::Set::Reserve,
                    time_set: crate::Set::Reserve,
                },
            },
        },
        Fact::Budget {
            run,
            call: None,
            standing: crate::facts::Standing {
                stage: crate::Stage::Ordinary,
                turns: 1,
                turns_limit: BUDGET.turns,
                spend: 100,
                spend_limit: BUDGET.spend,
                time: Duration::from_secs(3),
                time_limit: BUDGET.time,
                reserve,
                reserve_spend: 2,
            },
        },
        Fact::Ended { run, conversation: child, end: crate::facts::EndKind::Closed },
        Fact::Budget {
            run,
            call: Some(Token::new(7)),
            standing: crate::facts::Standing {
                stage: crate::Stage::Ordinary,
                turns: 1,
                turns_limit: BUDGET.turns,
                spend: 100,
                spend_limit: BUDGET.spend,
                time: Duration::from_secs(5),
                time_limit: BUDGET.time,
                reserve,
                reserve_spend: 2,
            },
        },
        Fact::Returned {
            run,
            call: Token::new(7),
            result: Return::Answered {
                cut: 0,
                used: crate::facts::Used { turns: 1, time: Duration::from_secs(5), spend: 100 },
            },
        },
        Fact::Called { run, conversation: main, call: Token::new(8), ask: Asked::Finish },
        Fact::Returned { run, call: Token::new(8), result: Return::Accepted },
        Fact::Ended { run, conversation: main, end: crate::facts::EndKind::Closed },
        Fact::Answered { run, answer: Answered::Accepted },
    ];
    assert_eq!(&*told, &expected);
}

#[test]
fn uncapped_facts_keep_the_admitted_terminal_and_every_prior_fact() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.running(1, 100);
    assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(answered(requests), (1, cancelled()));
    let observed = facts(&mut harness);
    assert_eq!(observed.len(), 5);
    assert_eq!(
        observed.last(),
        Some(&Fact::Answered { run, answer: Answered::Failed(crate::facts::FailureKind::Cancelled) })
    );
}

#[test]
fn delayed_facts_keep_activation_history_and_child_parentage() {
    for transcript in [None, Some(Token::new(77))] {
        let mut harness = Harness::new(LIMITS);
        harness.env.now = Time::from_nanos(7);
        let started = harness.step(Event::Start {
            messages: Box::default(),
            reply_to: ReplyTo::new(Token::new(1)),
            host_run: Token::new(1),
            identity: None,
            activation: 1,
            window: crate::Window { turns: u32::MAX, bytes: u64::MAX, largest_turn: 1 },
            charter: agents(),
            workspace: Some(workspace()),
            transcript,
            resumed: transcript.is_some(),
        });
        let [Request::Admitted { run, .. }, Request::Read { .. }] = &*started else { panic!("run admitted") };
        let run = *run;
        harness.env.now = Time::from_nanos(11);
        let opening =
            harness.step(Event::Read { owner: run, read: Read::Text { text: bytes(b"Be kind."), whole: true } });
        let [Request::Open { conversation: main, opening }] = &*opening else { panic!("main opens") };
        let main = *main;
        assert_eq!(opening.transcript, transcript);
        drop(harness.step(Event::Started { conversation: main, peer: PEER }));
        harness.env.now = Time::from_nanos(13);
        let (child, _) = harness.child(main, 7, families(true, false, false), 101);
        harness.env.now = Time::from_nanos(99);
        let facts = observed_facts(&mut harness);
        assert_eq!(facts[0].at, Time::from_nanos(7));
        assert_eq!(facts[0].kind, Fact::Admitted { run, resumed: transcript.is_some() });
        assert_eq!(facts[2].at, Time::from_nanos(11));
        assert_eq!(facts[2].kind, Fact::Opened { run, conversation: main, kind: crate::facts::Kind::Main });
        assert_eq!(facts[4].at, Time::from_nanos(13));
        assert_eq!(
            facts[4].kind,
            Fact::Opened {
                run,
                conversation: child,
                kind: crate::facts::Kind::Child {
                    parent: main,
                    call: Token::new(7),
                    asked: crate::Share::UNASKED,
                    granted: crate::Granted {
                        turns: BUDGET.turns,
                        time: EXPIRY.saturating_since(Time::from_nanos(13)),
                        spend: BUDGET.spend,
                        turns_set: crate::Set::Pool,
                        time_set: crate::Set::Deadline
                    },
                }
            }
        );
    }
}
