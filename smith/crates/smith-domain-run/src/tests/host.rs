use super::*;

#[test]
fn host_relay_retries_keep_the_call_name_and_input() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(71, 99);
    let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
    let first = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
    assert_eq!(first.attempt, 1);
    assert!(harness.step(Event::HostReturned { relay: first, reply: crate::HostReply::Busy }).is_empty());
    harness.after(LIMITS.host_backoff);
    let second = host_submission(&harness.fire());
    assert_eq!((second.owner, second.attempt), (first.owner, 2));
    assert!(
        harness
            .step(Event::HostReturned {
                relay: first,
                reply: crate::HostReply::Answered(
                    crate::HostAnswer::new(bytes(b"stale"), false).expect("small answer")
                )
            })
            .is_empty(),
        "stale attempt cannot resolve current attempt"
    );
    assert!(
        harness
            .step(Event::HostReturned { relay: second, reply: crate::HostReply::Unanswered(crate::Unanswered::Lost) })
            .is_empty()
    );
    harness.after(LIMITS.host_backoff);
    let third = host_submission(&harness.fire());
    assert_eq!(third.attempt, 3);
    let seen = crate::HostAnswer::new(bytes(b"recorded first decision"), true).expect("bounded exact error");
    assert_eq!(
        &*harness.step(Event::HostReturned { relay: third, reply: crate::HostReply::Answered(seen.clone()) }),
        &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostAnswered(seen) }]
    );
    assert!(
        harness.step(Event::HostReturned { relay: third, reply: crate::HostReply::Busy }).is_empty(),
        "one logical feedback"
    );
    assert_eq!(harness.domain.next_deadline(), Some(Time::ZERO.saturating_add(BUDGET.time)));
}

#[test]
fn a_host_call_busy_many_times_keeps_its_name_until_it_succeeds() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(71, 99);
    let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
    let mut relay = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
    for attempt in 1..=8 {
        assert_eq!(relay.attempt, attempt);
        assert!(harness.step(Event::HostReturned { relay, reply: crate::HostReply::Busy }).is_empty());
        harness.after(LIMITS.host_backoff);
        relay = host_submission(&harness.fire());
    }
    let answer = crate::HostAnswer::new(bytes(b"decided after eight busy attempts"), false).expect("bounded text");
    assert_eq!(
        &*harness.step(Event::HostReturned { relay, reply: crate::HostReply::Answered(answer.clone()) }),
        &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostAnswered(answer) }]
    );
}

#[test]
fn a_decided_host_answer_over_the_receiving_cap_is_not_unknown() {
    let limits = Limits { host_reply_bytes: 8, ..LIMITS };
    let mut harness = Harness::new(limits);
    let (_, conversation) = harness.running(71, 99);
    let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
    let relay = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
    let answer = crate::HostAnswer::new(bytes(b"nine bytes"), false).expect("constructor-bounded host text");
    assert_eq!(
        &*harness.step(Event::HostReturned { relay, reply: crate::HostReply::Answered(answer) }),
        &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostTooLarge { bytes: 10, max: 8 } }]
    );
}

#[test]
fn a_host_reported_too_large_answer_is_an_error_and_is_not_retried() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(71, 99);
    let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
    let relay = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
    assert_eq!(
        &*harness.step(Event::HostReturned { relay, reply: crate::HostReply::TooLarge }),
        &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostReportedTooLarge }]
    );
    assert!(
        harness.step(Event::HostReturned { relay, reply: crate::HostReply::TooLarge }).is_empty(),
        "the settled reply cannot ask the host again"
    );
}

#[test]
fn host_uncertainty_survives_busy_until_withdrawal_or_caller_expiry() {
    enum Stop {
        Withdraw,
        CallerExpiry,
    }
    for stop in [Stop::Withdraw, Stop::CallerExpiry] {
        let mut harness = Harness::new(LIMITS);
        let (_, conversation) = harness.running(71, 99);
        let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
        let first = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
        assert!(
            harness
                .step(Event::HostReturned {
                    relay: first,
                    reply: crate::HostReply::Unanswered(crate::Unanswered::Lost)
                })
                .is_empty()
        );
        harness.after(LIMITS.host_backoff);
        let second = host_submission(&harness.fire());
        assert!(harness.step(Event::HostReturned { relay: second, reply: crate::HostReply::Busy }).is_empty());
        let requests = match stop {
            Stop::Withdraw => harness.step(Event::Withdraw { conversation, call: Token::new(41) }),
            Stop::CallerExpiry => {
                harness.env.now = deadline;
                harness.fire()
            }
        };
        assert_eq!(&*requests, &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostUnknown }]);
        assert_eq!(
            (harness.domain.runs(), harness.domain.conversations()),
            (1, 1),
            "call-only stop retains ordinary run"
        );
    }
}

#[test]
fn host_withdrawal_and_timeout_retain_relay_until_terminal_answer_wins() {
    enum Stop {
        Withdraw,
        RelayTimeout,
        CancelRun,
    }
    for stop in [Stop::Withdraw, Stop::RelayTimeout, Stop::CancelRun] {
        let mut harness = Harness::new(LIMITS);
        let (run, conversation) = harness.running(71, 99);
        let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
        let relay = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
        match stop {
            Stop::Withdraw => assert_eq!(
                &*harness.step(Event::Withdraw { conversation, call: Token::new(41) }),
                &[Request::WithdrawHost { relay }]
            ),
            Stop::RelayTimeout => {
                harness.after(Duration::from_secs(5));
                assert_eq!(&*harness.fire(), &[Request::WithdrawHost { relay }]);
            }
            Stop::CancelRun => {
                assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: Token::new(99) }]);
                assert_eq!(
                    &*harness.step(Event::Withdraw { conversation, call: Token::new(41) }),
                    &[Request::WithdrawHost { relay }]
                );
            }
        }
        assert!(harness.step(Event::Withdraw { conversation, call: Token::new(41) }).is_empty());
        assert_eq!(harness.domain.calls(), 1, "withdrawal did not fabricate terminal");
        let seen = crate::HostAnswer::new(bytes(b"already recorded"), false).expect("small answer");
        assert_eq!(
            &*harness.step(Event::HostReturned { relay, reply: crate::HostReply::Answered(seen.clone()) }),
            &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostAnswered(seen) }]
        );
        match stop {
            Stop::CancelRun => {
                let (_, answer) = answered(harness.step(ended(conversation, End::Closed, Spend::ZERO)));
                assert_eq!(answer, failed(Failure::Cancelled, Spend::ZERO));
            }
            Stop::Withdraw | Stop::RelayTimeout => {}
        }
    }
}

#[test]
fn host_unknown_is_conveyed_after_withdrawn_terminal_when_shutdown_disallows_recovery() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(71, 99);
    let deadline = harness.env.now.saturating_add(Duration::from_secs(30));
    let relay = host_submission(&harness.step(host_ask(conversation, 41, deadline)));
    assert_eq!(
        &*harness.step(Event::Withdraw { conversation, call: Token::new(41) }),
        &[Request::WithdrawHost { relay }]
    );
    assert_eq!(
        &*harness
            .step(Event::HostReturned { relay, reply: crate::HostReply::Unanswered(crate::Unanswered::Withdrawn) }),
        &[Request::Return { spent: 0, call: Token::new(41), result: Returned::HostUnknown }]
    );
}

#[test]
fn host_declarations_are_admitted_as_bounded_unique_contracts_before_io() {
    let baseline = charter();
    assert_eq!(crate::charter::check(&baseline, Some(&workspace()), &LIMITS), Ok(LIMITS.guide_bytes));
    for invalid in 0_u32..5 {
        let mut declared = charter();
        match invalid {
            0 => declared.grants.host_tools[0].name = bytes(b"finish"),
            1 => {
                declared.grants.host_tools =
                    Box::new([declared.grants.host_tools[0].clone(), declared.grants.host_tools[0].clone()]);
            }
            2 => declared.grants.host_tools[0].timeout = Duration::ZERO,
            3 => declared.grants.host_tools[0].description = Box::new([]),
            4 => declared.grants.host_tools[0].schema = Box::new([]),
            _ => unreachable!("five concrete refusals"),
        }
        let mut harness = Harness::new(LIMITS);
        let (_, answer) = answered(harness.start(71, declared));
        assert_eq!(answer, Answer::Refused(Refusal::Invalid(Invalid::Grants)));
        assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
    }
    let mut harness = Harness::new(Limits { host_backoff: Duration::ZERO, ..LIMITS });
    let (_, answer) = answered(harness.start(71, baseline));
    assert_eq!(answer, Answer::Refused(Refusal::Invalid(Invalid::Grants)));
}

#[test]
fn the_prompt_tool_s_name_is_reserved() {
    let mut declared = charter();
    declared.grants.host_tools[0].name = b"permission_prompt".as_slice().into();
    let mut harness = Harness::new(LIMITS);
    assert_eq!(answered(harness.start(71, declared)), (71, Answer::Refused(Refusal::Invalid(Invalid::Grants))));
}

#[test]
fn host_declarations_reserve_only_smith_tool_names() {
    for name in crate::charter::RESERVED_NAMES {
        let mut declared = charter();
        declared.grants.host_tools[0].name = name.into();
        let mut harness = Harness::new(LIMITS);
        assert_eq!(answered(harness.start(71, declared)), (71, Answer::Refused(Refusal::Invalid(Invalid::Grants))));
    }
    for name in [b"read_file".as_slice(), b"list_dir", b"write_file", b"edit_file", b"run_shell", b"subagent"] {
        let mut declared = charter();
        declared.grants.host_tools[0].name = name.into();
        let mut harness = Harness::new(LIMITS);
        let requests = harness.start(71, declared);
        match requests.first() {
            Some(Request::Admitted { .. }) => {}
            _ => panic!("{name:?}: {requests:?}"),
        }
    }
}

#[test]
fn undeclared_host_and_effect_mismatch_are_refused_before_relay() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(71, 99);
    for (tool, effect, problem) in [
        (b"absent".as_slice(), crate::HostEffect::Read, crate::HostProblem::Undeclared),
        (b"comment".as_slice(), crate::HostEffect::Write, crate::HostProblem::Effect),
    ] {
        let event = Event::Delegated {
            conversation,
            call: Token::new(101),
            name: crate::CallName { activation: 1, completion: 1, position: 0 },
            ask: Ask::Host {
                tool: bytes(tool),
                effect,
                input: crate::HostInput::attested(bytes(b"{}")).expect("attested empty object"),
            },
            deadline: Time::ZERO.saturating_add(Duration::from_secs(30)),
        };
        assert_eq!(
            &*harness.step(event),
            &[Request::Return { spent: 0, call: Token::new(101), result: Returned::HostRejected(problem) }]
        );
    }
    assert_eq!(harness.domain.calls(), 0);
}

#[test]
fn a_charters_host_deadline_uses_the_tool_ceiling_and_preserves_the_default() {
    let limits = Limits { host_timeout: Duration::from_secs(1), host_timeout_max: Duration::from_secs(7), ..LIMITS };
    for seconds in [5_u64, 9] {
        let mut policy = charter();
        policy.grants.host_tools[0].timeout = Duration::from_secs(seconds);
        let mut harness = Harness::new(limits);
        let (_, conversation) = harness.running_on(71, 99, policy);
        let caller = harness.env.now.saturating_add(Duration::from_secs(30));
        let requests = harness.step(host_ask(conversation, 41, caller));
        let [Request::HostCall { deadline, .. }] = requests.as_ref() else { panic!("one admitted relay") };
        assert_eq!(*deadline, harness.env.now.saturating_add(Duration::from_secs(seconds.min(7))));
        assert_eq!(harness.env.limits.host_timeout, Duration::from_secs(1));
    }
}
