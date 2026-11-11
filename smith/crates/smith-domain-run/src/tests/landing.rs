use super::*;

#[test]
fn an_outcome_that_does_not_fit_the_spec_is_rejected_and_the_run_goes_on() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    let requests = harness.step(finish(conversation, 7, Declared::Change(change())));
    let problems = Problems { listed: Box::new([Problem::ChangeNotAllowed]), more: 0 };
    assert_eq!(&*requests, &[returned(7, Returned::Rejected { problems })]);
    let huge = verdict(b"approve", Box::new([]));
    let Declared::Verdict(mut huge) = huge else { unreachable!("a verdict") };
    huge.text = Box::from([b'x'; 2000].as_slice());
    let requests = harness.step(finish(conversation, 8, Declared::Verdict(huge)));
    let problems = Problems { listed: Box::new([Problem::TooLarge { max: LIMITS.outcome_bytes }]), more: 0 };
    assert_eq!(&*requests, &[returned(8, Returned::Rejected { problems })]);
    // Nudged out, the run fails as unfinished, counting what it rejected.
    for _ in 0..LIMITS.nudges {
        drop(harness.step(end_turn(conversation)));
    }
    assert_eq!(&*harness.step(end_turn(conversation)), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    let unfinished = RunFailure::Policy(Policy::Unfinished { nudges: LIMITS.nudges, rejected: 2 });
    assert_eq!(answered(requests), (1, failed(unfinished, Spend::ZERO)));
}

#[test]
fn a_verdict_that_fits_is_accepted_and_the_run_finishes_with_it() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    let outcome = verdict(b"request", Box::new([comment()]));
    let requests = harness.step(finish(conversation, 7, outcome));
    assert_eq!(&*requests, &[returned(7, Returned::Accepted), Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, spend(5)));
    let accepted = Answer::Accepted {
        outcome: verdict(b"request", Box::new([comment()])),
        spent: Spend { units: 0, ..spend(5) },
        turns: 0,
    };
    assert_eq!(answered(requests), (1, accepted));
}

#[test]
fn a_change_runs_each_repositorys_checks_then_is_delivered_and_accepted() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    let held = crate::lease::Lease::Workspace { owner };
    assert_eq!(harness.domain.runs.get(skein_lib::Id::from_token(run)).unwrap().lease, held);
    let requests = harness.step(Event::Checked { owner, ran: ran(0, b"ok") });
    let [Request::Check { owner: second, program, .. }, Request::Checking { .. }] = &*requests else {
        panic!("expected the second check, got {requests:?}");
    };
    assert_eq!((second, program.root), (&owner, Token::new(901)));
    assert_eq!(harness.domain.runs.get(skein_lib::Id::from_token(run)).unwrap().lease, held);
    let requests = harness.step(Event::Checked { owner, ran: ran(0, b"ok") });
    assert_eq!(
        &*requests,
        &[Request::Deliver {
            host_run: Token::new(1),
            owner,
            change: change(),
            name: crate::CallName { activation: 1, completion: 1, position: 0 },
            deadline: harness.env.now.saturating_add(LIMITS.delivery_timeout).min(EXPIRY)
        }]
    );
    assert_eq!(harness.domain.calls(), 1);
    assert_eq!(harness.domain.runs.get(skein_lib::Id::from_token(run)).unwrap().lease, held);
    let requests = harness.step(Event::Delivered { owner, delivery: delivered() });
    assert_eq!(harness.domain.runs.get(skein_lib::Id::from_token(run)).unwrap().lease, crate::lease::Lease::Free);
    assert_eq!(&*requests, &[returned(7, Returned::Delivered(receipts())), Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    let accepted = Answer::Accepted { outcome: Declared::Change(change()), spent: Spend::ZERO, turns: 0 };
    assert_eq!(answered(requests), (1, accepted));
    harness.domain.reclaim();
    assert_eq!((harness.domain.runs(), harness.domain.conversations(), harness.domain.calls()), (0, 0, 0));
}

#[test]
fn each_directory_retries_a_timed_out_check_under_the_same_lease() {
    let limits = Limits { check_timeout: Duration::from_secs(60), ..LIMITS };
    let mut harness = Harness::new(limits);
    let mut policy = coding();
    policy.reserve.time = Duration::from_secs(60);
    let (run, conversation) = harness.coding_policy(1, 100, policy);
    let requests = harness.step(finish(conversation, 7, Declared::Change(change())));
    let [Request::Check { owner, deadline, .. }, Request::Checking { deadline: told, .. }] = &*requests else {
        panic!("first check: {requests:?}");
    };
    assert_eq!((*deadline, *told), (Time::from_nanos(Duration::from_secs(60).as_nanos()), *deadline));
    let owner = *owner;
    for root in [Token::new(900), Token::new(901)] {
        harness.after(Duration::from_secs(60));
        let requests = harness.step(Event::Checked { owner, ran: timed_out_check() });
        let [Request::Check { owner: retry, program, deadline, .. }, Request::Checking { deadline: told, .. }] =
            &*requests
        else {
            panic!("timed out check retries immediately: {requests:?}");
        };
        assert_eq!((*retry, program.root), (owner, root));
        assert_eq!((*deadline, *told), (harness.env.now.saturating_add(Duration::from_secs(120)), *deadline));
        assert_eq!(harness.domain.calls(), 1);
        assert_eq!(
            harness.domain.runs.get(skein_lib::Id::from_token(run)).unwrap().lease,
            crate::lease::Lease::Workspace { owner }
        );
        harness.after(Duration::from_secs(120));
        let requests = harness.step(pass_checks(owner));
        if root == Token::new(900) {
            let [Request::Check { program, .. }, Request::Checking { .. }] = &*requests else {
                panic!("next directory: {requests:?}");
            };
            assert_eq!(program.root, Token::new(901));
        } else {
            let [Request::Deliver { owner: submitted, .. }] = &*requests else { panic!("delivery: {requests:?}") };
            assert_eq!(*submitted, owner);
        }
    }
}

#[test]
fn a_second_timeout_refuses_with_the_retrys_given_time() {
    let mut harness = Harness::new(Limits { check_timeout: Duration::from_secs(60), ..LIMITS });
    let mut policy = coding();
    policy.reserve.time = Duration::from_secs(60);
    let (_, conversation) = harness.coding_policy(1, 100, policy);
    let requests = harness.step(finish(conversation, 7, Declared::Change(change())));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = &*requests else { panic!("first check") };
    let owner = *owner;
    harness.after(Duration::from_secs(60));
    let retry = harness.step(Event::Checked { owner, ran: timed_out_check() });
    let [Request::Check { .. }, Request::Checking { .. }] = &*retry else { panic!("retry: {retry:?}") };
    harness.after(Duration::from_secs(120));
    assert_eq!(
        harness.step(Event::Checked { owner, ran: timed_out_check() }).as_ref(),
        [returned(
            7,
            Returned::ChecksFailed {
                repository: bytes(b"temper"),
                check: bytes(b".temper/pre-pr"),
                given: Duration::from_secs(120),
                ran: timed_out_check()
            }
        )]
    );
    harness.domain.reclaim();
    assert_eq!(harness.domain.calls(), 0);
}

#[test]
fn the_charters_check_bound_replaces_the_limit_and_is_cut_by_the_call() {
    for (bound, call_seconds, expected_seconds) in [
        (None, 500, 300),
        (Some(Duration::from_secs(400)), 500, 400),
        (Some(Duration::from_secs(400)), 200, 200),
        (Some(Duration::from_secs(86_400)), 1000, 600),
    ] {
        let mut harness = Harness::new(LIMITS);
        let mut policy = coding();
        policy.check_timeout = bound;
        let (_, conversation) = harness.coding_policy(1, 100, policy);
        let deadline = harness.env.now.saturating_add(Duration::from_secs(call_seconds));
        let requests = harness.step(finish_by(conversation, 7, Declared::Change(change()), deadline));
        let [Request::Check { owner, deadline, .. }, Request::Checking { deadline: told, .. }] = &*requests else {
            panic!("first check: {requests:?}");
        };
        let given = Duration::from_secs(expected_seconds);
        assert_eq!((*deadline, *told), (harness.env.now.saturating_add(given), *deadline));
        let owner = *owner;
        assert_eq!(
            harness.step(Event::Checked { owner, ran: ran(1, b"failed") }).as_ref(),
            [returned(
                7,
                Returned::ChecksFailed {
                    repository: bytes(b"temper"),
                    check: bytes(b".temper/pre-pr"),
                    given,
                    ran: ran(1, b"failed")
                }
            )]
        );
    }
}

#[test]
fn the_check_bound_is_admitted_from_one_second_to_one_day_independently_of_reserve() {
    for (bound, admitted) in [
        (Duration::from_millis(999), false),
        (Duration::from_secs(1), true),
        (Duration::from_secs(86_400), true),
        (Duration::from_nanos(86_400_000_000_001), false),
    ] {
        let mut harness = Harness::new(LIMITS);
        let mut policy = coding();
        policy.check_timeout = Some(bound);
        let requests = harness.start(1, policy);
        if admitted {
            let [Request::Admitted { .. }, Request::Read { .. }] = &*requests else { panic!("admitted: {requests:?}") };
        } else {
            assert_eq!(answered(requests), (1, Answer::Refused(Refusal::Invalid(Invalid::MalformedCharter))));
        }
    }
}

#[test]
fn retries_are_cut_at_the_call_and_ordinary_deadlines() {
    for (start, call_until, expected) in [(0, 150, 150), (150, 600, 300)] {
        let mut harness = Harness::new(Limits { check_timeout: Duration::from_secs(60), ..LIMITS });
        let (_, conversation) = harness.coding(1, 100);
        harness.after(Duration::from_secs(start));
        let requests = harness.step(finish_by(
            conversation,
            7,
            Declared::Change(change()),
            Time::ZERO.saturating_add(Duration::from_secs(call_until)),
        ));
        let [Request::Check { owner, .. }, Request::Checking { .. }] = &*requests else { panic!("first check") };
        let owner = *owner;
        harness.after(Duration::from_secs(60));
        let requests = harness.step(Event::Checked { owner, ran: timed_out_check() });
        let [Request::Check { deadline, .. }, Request::Checking { deadline: told, .. }] = &*requests else {
            panic!("retry with room: {requests:?}");
        };
        assert_eq!((*deadline, *told), (Time::ZERO.saturating_add(Duration::from_secs(expected)), *deadline));
        harness.after(Duration::from_secs(90));
        assert_eq!(
            harness.step(Event::Checked { owner, ran: timed_out_check() }).as_ref(),
            [returned(
                7,
                Returned::ChecksFailed {
                    repository: bytes(b"temper"),
                    check: bytes(b".temper/pre-pr"),
                    given: Duration::from_secs(90),
                    ran: timed_out_check()
                }
            )]
        );
    }
}

#[test]
fn checks_without_retry_room_or_a_timeout_return_feedback_at_once() {
    for (exit, start, call_until) in [
        (Exit::Code { code: 1 }, 0, 600),
        (Exit::Cancelled, 0, 600),
        (Exit::Unstarted, 0, 600),
        (Exit::Signalled { signal: 15 }, 0, 600),
        (Exit::TimedOut, 0, 120),
        (Exit::TimedOut, 0, 119),
        (Exit::TimedOut, 180, 600),
        (Exit::TimedOut, 181, 600),
        (Exit::TimedOut, 301, 600),
    ] {
        let mut harness = Harness::new(Limits { check_timeout: Duration::from_secs(60), ..LIMITS });
        let (_, conversation) = harness.coding(1, 100);
        harness.after(Duration::from_secs(start));
        let requests = harness.step(finish_by(
            conversation,
            7,
            Declared::Change(change()),
            Time::ZERO.saturating_add(Duration::from_secs(call_until)),
        ));
        let [Request::Check { owner, .. }, Request::Checking { .. }] = &*requests else { panic!("first check") };
        let owner = *owner;
        harness.after(Duration::from_secs(60));
        assert_eq!(
            harness.step(Event::Checked { owner, ran: Ran { exit, ..timed_out_check() } }).as_ref(),
            [returned(
                7,
                Returned::ChecksFailed {
                    repository: bytes(b"temper"),
                    check: bytes(b".temper/pre-pr"),
                    given: Duration::from_secs(60),
                    ran: Ran { exit, ..timed_out_check() }
                }
            )],
            "exit={exit:?}, start={start}, call_until={call_until}"
        );
    }
}

#[test]
fn winding_down_for_turns_does_not_retry_even_with_ordinary_time_left() {
    let mut harness = Harness::new(Limits { check_timeout: Duration::from_secs(60), ..LIMITS });
    let mut policy = coding();
    policy.budget.turns = 4;
    policy.reserve.turns = 2;
    let (_, main) = harness.coding_policy(1, 100, policy);
    drop(harness.step(Event::Used { conversation: main, spend: Spend { turns: 2, ..Spend::ZERO } }));
    let permit = crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0));
    let crate::CompletionPermit::Final { .. } = permit else { panic!("finishing reserve: {permit:?}") };
    let requests = harness.step(finish(main, 7, Declared::Change(change())));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = &*requests else {
        panic!("first check: {requests:?}")
    };
    let owner = *owner;
    harness.after(Duration::from_secs(60));
    let requests = harness.step(Event::Checked { owner, ran: timed_out_check() });
    let [Request::Return { result: Returned::ChecksFailed { given, .. }, .. }] = &*requests else {
        panic!("no retry in wind-down: {requests:?}");
    };
    assert_eq!(*given, Duration::from_secs(60));
}

fn timed_out_check() -> Ran {
    Ran { exit: Exit::TimedOut, output: bytes(b"tail"), cut: 3 }
}

#[test]
fn a_change_that_fails_its_checks_or_its_delivery_goes_back_to_the_llm() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    let failing = Ran { exit: Exit::Code { code: 1 }, output: bytes(b"test parse ... FAILED"), cut: 12 };
    let requests = harness.step(Event::Checked { owner, ran: failing });
    let failing = Ran { exit: Exit::Code { code: 1 }, output: bytes(b"test parse ... FAILED"), cut: 12 };
    assert_eq!(
        &*requests,
        &[returned(
            7,
            Returned::ChecksFailed {
                repository: bytes(b"temper"),
                check: bytes(b".temper/pre-pr"),
                given: LIMITS.check_timeout,
                ran: failing,
            }
        )]
    );
    let owner = harness.land(conversation, 8);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    assert_eq!(
        &*harness.step(Event::Delivered {
            owner,
            delivery: Delivery::Failed(crate::DeliveryFailure::new(0, crate::DeliveryReason::Unknown))
        }),
        &[returned(
            8,
            Returned::DeliveryFailed { failure: crate::DeliveryFailure::new(0, crate::DeliveryReason::Unknown) }
        )]
    );
    assert!(harness.step(end_turn(conversation)).len() == 1, "the run goes on: a nudge");
}

#[test]
fn a_change_can_be_delivered_after_failing_checks_when_its_contract_allows_it() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = coding();
    policy.outcome.change.as_mut().expect("change contract").checks_must_pass = false;
    let (_, conversation) = harness.coding_policy(1, 100, policy);
    let owner = harness.land(conversation, 7);
    let requests = harness.step(Event::Checked { owner, ran: ran(1, b"first failed") });
    let [Request::Check { program, .. }, Request::Checking { .. }] = requests.as_ref() else {
        panic!("the second check still runs: {requests:?}");
    };
    assert_eq!(program.root, Token::new(901));
    let requests = harness.step(Event::Checked {
        owner,
        ran: Ran { exit: Exit::Signalled { signal: 15 }, output: bytes(b"second failed"), cut: 0 },
    });
    let [Request::Deliver { .. }] = requests.as_ref() else { panic!("delivery follows both checks: {requests:?}") };
    let mut check_exits = List::with_capacity(2);
    for fact in facts(&mut harness).as_ref() {
        match fact {
            Fact::CheckFinished { exit, .. } => check_exits.push(*exit).expect("two requested checks"),
            Fact::MessageRead { .. }
            | Fact::MessageFence { .. }
            | Fact::MessageUnread { .. }
            | Fact::MessageReceived { .. }
            | Fact::MessageRefused { .. }
            | Fact::Admitted { .. }
            | Fact::Prepared { .. }
            | Fact::Opened { .. }
            | Fact::Budget { .. }
            | Fact::Ended { .. }
            | Fact::Called { .. }
            | Fact::Returned { .. }
            | Fact::CheckStarted { .. }
            | Fact::Delivered { .. }
            | Fact::Answered { .. } => {}
        }
    }
    assert_eq!(check_exits.as_slice(), [Exit::Code { code: 1 }, Exit::Signalled { signal: 15 }]);
    assert_eq!(
        harness.step(Event::Delivered { owner, delivery: delivered() }).as_ref(),
        [returned(7, Returned::Delivered(receipts())), Request::Close { peer: PEER }]
    );
    assert_eq!(
        answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))),
        (1, Answer::Accepted { outcome: Declared::Change(change()), spent: Spend::ZERO, turns: 0 })
    );
}

#[test]
fn a_change_whose_branch_moved_ends_the_run_as_stale() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    let requests = harness.step(Event::Delivered { owner, delivery: Delivery::Stale });
    assert_eq!(&*requests, &[returned(7, Returned::Stale), Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(answered(requests), (1, failed(RunFailure::Stale, Spend::ZERO)));
}

#[test]
fn past_the_budget_the_deadline_fails_the_run_for_the_part_it_went_past() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
    harness.after(BUDGET.time);
    assert_eq!(&*harness.fire(), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, spend(BUDGET.spend + 1)));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                RunFailure::Budget {
                    exhausted: Exhausted::Spend,
                    used: Some(BUDGET.spend + 1),
                    bound: Some(BUDGET.spend)
                },
                spend(BUDGET.spend + 1)
            )
        )
    );
}

#[test]
fn a_withdrawn_landing_stops_what_is_in_flight_and_returns_once_it_has() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    assert!(harness.step(Event::Withdraw { conversation, call: Token::new(6) }).is_empty(), "not the landing call");
    assert_eq!(&*harness.step(Event::Withdraw { conversation, call: Token::new(7) }), &[Request::Abort { owner }]);
    assert_eq!(&*harness.step(Event::Aborted { owner }), &[returned(7, Returned::Cancelled)]);
    assert_eq!(harness.checks_ended, 1, "abort terminal closes the check span");
    assert!(harness.step(Event::Withdraw { conversation, call: Token::new(7) }).is_empty(), "returned already");
    harness.domain.reclaim();

    // Withdrawn while pushing, and the push wins the race: it landed.
    let owner = harness.land(conversation, 8);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    assert!(
        harness.step(Event::Withdraw { conversation, call: Token::new(8) }).is_empty(),
        "submitted delivery is never abandoned"
    );
    let requests = harness.step(Event::Delivered { owner, delivery: delivered() });
    assert_eq!(&*requests, &[returned(8, Returned::Delivered(receipts())), Request::Close { peer: PEER }]);
}

#[test]
fn a_landing_past_its_deadline_is_stopped_and_returns_timed_out_once_it_has() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let minute = Duration::from_secs(60);
    let requests =
        harness.step(finish_by(conversation, 7, Declared::Change(change()), Time::ZERO.saturating_add(minute)));
    let [Request::Check { owner, deadline, .. }, Request::Checking { .. }] = &*requests else {
        panic!("expected the first check, got {requests:?}");
    };
    // A check may not outlive its caller's earlier deadline.
    assert_eq!(*deadline, Time::ZERO.saturating_add(minute).min(Time::ZERO.saturating_add(LIMITS.check_timeout)));
    let owner = *owner;
    harness.after(minute);
    assert_eq!(&*harness.fire(), &[Request::Abort { owner }]);
    assert!(!harness.domain.is_due(harness.env.now), "a call's deadline fires once");
    // Checks that pass before the abort lands push nothing.
    assert_eq!(&*harness.step(pass_checks(owner)), &[returned(7, Returned::TimedOut)]);
    assert_eq!(harness.step(end_turn(conversation)).len(), 1, "the run goes on: a nudge");
    harness.domain.reclaim();
}

#[test]
fn a_submitted_delivery_keeps_its_terminal_after_caller_expiry() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let minute = Duration::from_secs(60);
    let deadline = harness.env.now.saturating_add(minute);
    let requests = harness.step(finish_by(conversation, 8, Declared::Change(change()), deadline));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = &*requests else {
        panic!("expected the first check, got {requests:?}");
    };
    let owner = *owner;
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    harness.after(minute);
    assert!(harness.fire().is_empty(), "call expiry retains the submitted host operation");
    assert_eq!(
        &*harness.step(Event::Delivered {
            owner,
            delivery: Delivery::Failed(crate::DeliveryFailure::new(0, crate::DeliveryReason::TimedOut))
        }),
        &[returned(
            8,
            Returned::DeliveryFailed { failure: crate::DeliveryFailure::new(0, crate::DeliveryReason::TimedOut) }
        )]
    );
    harness.domain.reclaim();
    assert_eq!(harness.domain.calls(), 0);
}

#[test]
fn past_the_budget_a_finish_in_the_turn_in_flight_still_counts() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
    let requests = harness.step(finish(conversation, 7, verdict(b"approve", Box::new([]))));
    assert_eq!(&*requests, &[returned(7, Returned::Accepted), Request::Close { peer: PEER }]);
    let total = spend(BUDGET.spend + 1);
    let requests = harness.step(ended(conversation, End::Closed, total));
    assert_eq!(
        answered(requests),
        (1, Answer::Accepted { outcome: verdict(b"approve", Box::new([])), spent: total, turns: 0 })
    );
}

#[test]
fn a_refused_finish_after_spend_exhaustion_closes_main_for_budget() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(2, 101);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
    let requests = harness.step(finish(conversation, 8, verdict(b"reject", Box::new([]))));
    let problems = Problems { listed: Box::new([Problem::UnknownVerdict]), more: 0 };
    assert_eq!(&*requests, &[returned(8, Returned::Rejected { problems }), Request::Close { peer: Token::new(101) }]);
}

#[test]
fn a_run_out_of_time_while_it_prepares_answers_once_its_look_has_ended() {
    let mut harness = Harness::new(LIMITS);
    let run = harness.prepare(1);
    harness.after(BUDGET.time);
    assert!(harness.fire().is_empty(), "the look is in flight");
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    assert_eq!(
        answered(requests),
        (1, failed(RunFailure::Budget { exhausted: Exhausted::Time, used: None, bound: None }, Spend::ZERO))
    );
}

#[test]
fn nothing_to_delivery_is_specific_feedback_and_the_run_can_retry() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    assert_eq!(
        &*harness.step(Event::Delivered { owner, delivery: Delivery::Nothing }),
        &[returned(7, Returned::Nothing)]
    );
    assert_eq!(harness.step(end_turn(conversation)).len(), 1, "a retry is nudged");
}

#[test]
fn failed_delivery_reason_and_diagnostics_return_to_the_finish_caller() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    let failure = crate::DeliveryFailure {
        directory: 1,
        reason: crate::DeliveryReason::RefusedByTarget,
        diagnostic: crate::Diagnostic::new(b"remote: protected branch", 17),
    };
    assert_eq!(
        &*harness.step(Event::Delivered { owner, delivery: Delivery::Failed(failure) }),
        &[returned(7, Returned::DeliveryFailed { failure })]
    );
    let mut pushed = None;
    for fact in facts(&mut harness) {
        match fact {
            Fact::Delivered { status: push, .. } => pushed = Some(push),
            Fact::MessageReceived { .. }
            | Fact::MessageRefused { .. }
            | Fact::MessageRead { .. }
            | Fact::MessageFence { .. }
            | Fact::MessageUnread { .. }
            | Fact::Admitted { .. }
            | Fact::Budget { .. }
            | Fact::Prepared { .. }
            | Fact::Opened { .. }
            | Fact::Ended { .. }
            | Fact::Called { .. }
            | Fact::Returned { .. }
            | Fact::CheckStarted { .. }
            | Fact::CheckFinished { .. }
            | Fact::Answered { .. } => {}
        }
    }
    let pushed = pushed.expect("delivery ended fact");
    assert_eq!(
        pushed,
        crate::DeliveryStatus::Failed(crate::DeliveryReason::RefusedByTarget),
        "facts retain classification, no diagnostic content"
    );
}

#[test]
fn a_failed_delivery_with_an_unmounted_directory_is_broken_evidence() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    let failure = crate::DeliveryFailure::new(99, crate::DeliveryReason::RefusedByTarget);
    assert_eq!(
        &*harness.step(Event::Delivered { owner, delivery: Delivery::Failed(failure) }),
        &[returned(
            7,
            Returned::DeliveryFailed { failure: crate::DeliveryFailure::new(0, crate::DeliveryReason::Broken) }
        )]
    );
}

#[test]
fn reports_and_declared_failures_settle_once_without_checks_or_delivery() {
    for (failure, text) in [(false, b"".as_slice()), (true, b"no".as_slice())] {
        let mut harness = Harness::new(LIMITS);
        let (run, conversation) = harness.running_on(1, 100, text_charter(failure));
        let requests = harness.step(finish(conversation, 7, text_result(failure, text)));
        assert_eq!(&*requests, &[returned(7, Returned::Accepted), Request::Close { peer: PEER }]);
        assert!(harness.step(Event::Cancel { run: Token::new(999) }).is_empty());
        assert!(
            harness.step(Event::Priced { conversation, own_spent: 5, subtree_spent: 5 }).is_empty(),
            "an own price notice charges the terminal's usage independently"
        );
        let requests = harness.step(ended(conversation, End::Closed, spend(5)));
        assert_eq!(
            answered(requests),
            (1, Answer::Accepted { outcome: text_result(failure, text), spent: spend(5), turns: 0 })
        );
        assert!(harness.step(Event::Cancel { run }).is_empty(), "a late cancel emits no second answer");
    }
}

#[test]
fn empty_failure_reason_is_accepted_without_starting_delivery() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running_on(1, 100, text_charter(true));
    let requests = harness.step(finish(conversation, 7, text_result(true, b"")));
    assert_eq!(&*requests, &[returned(7, Returned::Accepted), Request::Close { peer: PEER }]);
}

#[test]
fn impossible_result_contracts_refuse_before_any_preparation_or_session() {
    let mut c = text_charter(false);
    c.outcome.report.as_mut().unwrap().fields[0].max = 0;
    let mut harness = Harness::new(LIMITS);
    assert_eq!(answered(harness.start(1, c)), (1, Answer::Refused(Refusal::Invalid(Invalid::Outcome))));
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (0, 0));
    let min = u64::try_from(size_of::<Field>()).unwrap() + 6 + 1;
    let mut harness = Harness::new(Limits { outcome_bytes: min - 1, ..LIMITS });
    assert_eq!(
        answered(harness.start(2, text_charter(false))),
        (2, Answer::Refused(Refusal::Invalid(Invalid::Outcome)))
    );
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (0, 0));
}

#[test]
fn unknown_extra_field_ownership_is_checked_before_shape_judgement() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running_on(1, 100, text_charter(false));
    let mut oversized = List::with_capacity(u32::try_from(LIMITS.outcome_bytes).expect("the fixture cap is small"));
    for _ in 0..LIMITS.outcome_bytes {
        oversized.push(b'x').expect("room for exactly the declared cap");
    }
    let value = Declared::Report(crate::outcome::Report {
        text: Box::new([]),
        fields: Box::new([
            Field { name: bytes(b"source"), value: bytes(b"ref") },
            Field { name: bytes(b"opaque"), value: oversized.into_boxed() },
        ]),
    });
    let requests = harness.step(finish(conversation, 7, value));
    assert_eq!(
        &*requests,
        &[returned(7, Returned::Rejected { problems: crate::outcome::too_large(LIMITS.outcome_bytes) })]
    );
    assert_eq!(
        &*harness.step(finish(conversation, 8, text_result(false, b""))),
        &[returned(8, Returned::Accepted), Request::Close { peer: PEER }]
    );
}

#[test]
fn a_mid_run_delivery_without_a_change_contract_needs_passing_checks() {
    for (exit, output, cut, custom, call_time) in [
        (Exit::Code { code: 1 }, b"draft failed".as_slice(), 0, true, LIMITS.budget.time),
        (Exit::TimedOut, b"tail".as_slice(), 3, false, LIMITS.budget.time),
        (Exit::TimedOut, b"tail".as_slice(), 3, true, LIMITS.budget.time),
        (Exit::TimedOut, b"tail".as_slice(), 3, true, Duration::from_secs(1)),
    ] {
        let mut harness = Harness::new(LIMITS);
        let mut policy = mid_report();
        if !custom {
            policy.conventions = None;
        }
        let (_, conversation) = mid_running_policy(&mut harness, policy);
        harness.after(Duration::from_secs(1));
        let Event::Delegated { conversation, call, name, ask, .. } = mid_ask(conversation, 3) else {
            panic!("delivery call");
        };
        let deadline = harness.env.now.saturating_add(call_time).min(EXPIRY);
        let given = LIMITS.check_timeout.min(deadline.saturating_since(harness.env.now));
        let check = if custom { b".temper/pre-pr".as_slice() } else { b".smith/check".as_slice() };
        let requests = harness.step(Event::Delegated { conversation, call, name, ask, deadline });
        let [Request::Check { owner, program, deadline, .. }, Request::Checking { .. }] = requests.as_ref() else {
            panic!("discovered check runs: {requests:?}");
        };
        assert_eq!(program.path.as_ref(), check);
        assert_eq!(*deadline, harness.env.now.saturating_add(given));
        let owner = *owner;
        harness.after(Duration::from_millis(1));
        assert_eq!(
            harness.step(Event::Checked { owner, ran: Ran { exit, output: bytes(output), cut } }).as_ref(),
            [returned(
                50,
                Returned::ChecksFailed {
                    repository: bytes(b"temper"),
                    check: bytes(check),
                    given,
                    ran: Ran { exit, output: bytes(output), cut },
                }
            )],
            "exit={exit:?}, custom={custom}, call_time={call_time:?}"
        );
    }
}

#[test]
fn a_mid_run_delivery_uses_the_change_contracts_optional_checks() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = mid_report();
    policy.outcome.change = Some(ChangeSpec { checks_must_pass: false, fields: Box::new([]) });
    let (_, conversation) = mid_running_policy(&mut harness, policy);
    let requests = harness.step(mid_ask(conversation, 3));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = requests.as_ref() else {
        panic!("discovered check runs: {requests:?}");
    };
    let owner = *owner;
    let requests = harness.step(Event::Checked { owner, ran: ran(1, b"draft failed") });
    let [Request::Deliver { .. }] = requests.as_ref() else { panic!("delivery follows the check: {requests:?}") };
    assert_eq!(
        harness.step(Event::Delivered { owner, delivery: delivered() }).as_ref(),
        [returned(50, Returned::Delivered(receipts()))]
    );
}

struct MidDelivery {
    harness: Harness,
    run: Token,
    conversation: Token,
    owner: Token,
}

impl MidDelivery {
    fn pending() -> Self {
        let mut harness = Harness::new(LIMITS);
        let (run, conversation) = mid_running(&mut harness);
        let owner = submit_mid(&mut harness, conversation);
        Self { harness, run, conversation, owner }
    }

    fn short_caller() -> Self {
        let mut harness = Harness::new(LIMITS);
        let (run, conversation) = mid_running(&mut harness);
        let caller_deadline = harness.env.now.saturating_add(Duration::from_secs(1));
        let mut ask = mid_ask(conversation, 3);
        let Event::Delegated { deadline, .. } = &mut ask else { unreachable!("delegated fixture") };
        *deadline = caller_deadline;
        let requests = harness.step(ask);
        let [Request::Check { owner, .. }, Request::Checking { .. }] = requests.as_ref() else {
            panic!("delivery checks first")
        };
        let owner = *owner;
        let requests = harness.step(pass_checks(owner));
        let [Request::Deliver { deadline, .. }] = requests.as_ref() else {
            panic!("checks submit exactly one delivery")
        };
        assert_eq!(*deadline, caller_deadline);
        Self { harness, run, conversation, owner }
    }

    fn delivered(&mut self) -> Box<[Request]> {
        let requests = self.harness.step(Event::Delivered { owner: self.owner, delivery: delivered() });
        assert!(
            self.harness.step(Event::Delivered { owner: self.owner, delivery: delivered() }).is_empty(),
            "stale duplicate is inert before reclaim"
        );
        requests
    }

    fn accepted_delivery(&mut self) {
        assert_eq!(self.delivered().as_ref(), &[returned(50, Returned::Delivered(receipts()))]);
    }

    fn expire_caller(&mut self) {
        self.harness.after(Duration::from_secs(1));
        assert!(self.harness.domain.is_due(self.harness.env.now));
        assert!(self.harness.fire().is_empty(), "expiry retains the host terminal right");
    }

    fn withdraw_caller(&mut self) {
        assert!(
            self.harness.step(Event::Withdraw { conversation: self.conversation, call: Token::new(50) }).is_empty()
        );
    }

    fn ended(&mut self, expected: Answer, spent: Spend) {
        assert_eq!(answered(self.harness.step(ended(self.conversation, End::Closed, spent))).1, expected);
        self.harness.domain.reclaim();
        assert!(
            self.harness.step(Event::Delivered { owner: self.owner, delivery: delivered() }).is_empty(),
            "stale generation is inert after reclaim"
        );
    }

    fn cancelled(&mut self) {
        assert_eq!(self.harness.step(Event::Cancel { run: self.run }).as_ref(), &[Request::Close { peer: PEER }]);
        self.ended(Answer::Failed { failure: RunFailure::Cancelled, spent: Spend::ZERO, turns: 0 }, Spend::ZERO);
    }

    fn reported(&mut self) {
        drop(self.harness.step(finish(self.conversation, 51, report())));
        self.ended(Answer::Accepted { outcome: report(), spent: Spend::ZERO, turns: 0 }, Spend::ZERO);
    }
}

#[test]
fn submitted_mid_delivery_settles_before_a_cancelled_run_answers() {
    let mut delivery = MidDelivery::pending();
    drop(delivery.harness.step(Event::Cancel { run: delivery.run }));
    delivery.withdraw_caller();
    delivery.accepted_delivery();
    delivery.ended(Answer::Failed { failure: RunFailure::Cancelled, spent: Spend::ZERO, turns: 0 }, Spend::ZERO);
}

#[test]
fn submitted_mid_delivery_settles_before_main_reports() {
    let mut delivery = MidDelivery::pending();
    delivery.accepted_delivery();
    delivery.reported();
}

#[test]
fn malformed_host_mount_or_zero_origin_never_becomes_successful_delivery() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = mid_running(&mut harness);
    assert_eq!(
        harness.step(mid_ask(conversation, 0)).as_ref(),
        &[returned(50, Returned::Refused { refusal: AskRefusal::Name })]
    );
    let owner = submit_mid(&mut harness, conversation);
    let wrong = crate::Delivered::new(Box::new([crate::Receipt::new(1, bytes(b"unmounted")).expect("sealed ordinal")]))
        .expect("sealed terminal");
    assert_eq!(
        harness.step(Event::Delivered { owner, delivery: Delivery::Delivered(wrong) }).as_ref(),
        &[returned(
            50,
            Returned::DeliveryFailed { failure: crate::DeliveryFailure::new(0, crate::DeliveryReason::Broken) }
        )]
    );
    drop(harness.step(finish(conversation, 51, report())));
    let answer = answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))).1;
    assert_eq!(answer, Answer::Accepted { outcome: report(), spent: Spend::ZERO, turns: 0 });
}

#[test]
fn time_shutdown_keeps_an_already_submitted_mid_landing() {
    let mut delivery = MidDelivery::pending();
    delivery.harness.env.now = EXPIRY;
    for _ in 0_u32..2 {
        if delivery.harness.domain.is_due(delivery.harness.env.now) {
            drop(delivery.harness.fire());
        }
    }
    delivery.accepted_delivery();
    delivery.ended(
        Answer::Failed {
            failure: RunFailure::Budget { exhausted: Exhausted::Time, used: Some(600_000), bound: Some(600_000) },
            spent: Spend::ZERO,
            turns: 0,
        },
        Spend::ZERO,
    );
}

#[test]
fn spend_shutdown_keeps_an_already_submitted_mid_landing() {
    let mut delivery = MidDelivery::pending();
    let spent = spend(BUDGET.spend + 1);
    drop(delivery.harness.step(Event::Used { conversation: delivery.conversation, spend: spent }));
    assert_eq!(
        delivery.delivered().as_ref(),
        &[returned(50, Returned::Delivered(receipts())), Request::Close { peer: PEER }]
    );
    delivery.ended(
        Answer::Failed {
            failure: RunFailure::Budget {
                exhausted: Exhausted::Spend,
                used: Some(BUDGET.spend + 1),
                bound: Some(BUDGET.spend),
            },
            spent,
            turns: 0,
        },
        spent,
    );
}

#[test]
fn generic_non_marker_host_refusal_is_feedback_and_report_can_finish() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = mid_running(&mut harness);
    let owner = submit_mid(&mut harness, conversation);
    let refusal = crate::DeliveryRefusal::new(None, bytes(b"host asks for corrected metadata"))
        .expect("bounded generic correctable feedback");
    assert_eq!(
        harness.step(Event::Delivered { owner, delivery: Delivery::Refused(refusal.clone()) }).as_ref(),
        &[returned(50, Returned::DeliveryRefused(refusal))]
    );
    drop(harness.step(finish(conversation, 51, report())));
    let answer = answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))).1;
    assert_eq!(answer, Answer::Accepted { outcome: report(), spent: Spend::ZERO, turns: 0 });
}

#[test]
fn impossible_delivery_grant_and_zero_host_deadline_refuse_before_preparation() {
    let mut charter = mid_report();
    charter.grants.deliver =
        Some(ChangeSpec { checks_must_pass: true, fields: Box::new([FieldRule { name: bytes(b"ticket"), max: 0 }]) });
    let mut harness = Harness::new(LIMITS);
    assert_eq!(answered(harness.start(1, charter)), (1, Answer::Refused(Refusal::Invalid(Invalid::Grants))));
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (0, 0));
    let mut harness = Harness::new(Limits { delivery_timeout: Duration::ZERO, ..LIMITS });
    assert_eq!(answered(harness.start(2, mid_report())), (2, Answer::Refused(Refusal::Invalid(Invalid::Workspace))));
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (0, 0));
}

#[test]
fn caller_expiry_keeps_submitted_delivery_until_its_terminal_then_cancellation() {
    let mut delivery = MidDelivery::short_caller();
    delivery.expire_caller();
    delivery.accepted_delivery();
    delivery.cancelled();
}

#[test]
fn caller_expiry_keeps_submitted_delivery_until_its_terminal_then_report() {
    let mut delivery = MidDelivery::short_caller();
    delivery.expire_caller();
    delivery.accepted_delivery();
    delivery.reported();
}

#[test]
fn caller_withdrawal_keeps_submitted_delivery_until_its_terminal_then_cancellation() {
    let mut delivery = MidDelivery::short_caller();
    delivery.withdraw_caller();
    delivery.accepted_delivery();
    delivery.cancelled();
}

#[test]
fn caller_withdrawal_keeps_submitted_delivery_until_its_terminal_then_report() {
    let mut delivery = MidDelivery::short_caller();
    delivery.withdraw_caller();
    delivery.accepted_delivery();
    delivery.reported();
}

#[test]
fn a_save_settles_jobs_skips_checks_and_a_later_delivery_checks_again() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = mid_report();
    policy.grants.save = true;
    let (run, conversation) = mid_running_policy(&mut harness, policy);
    let mut ask = mid_ask(conversation, 3);
    let Event::Delegated { ask: tool, .. } = &mut ask else { unreachable!("delegated") };
    *tool = Ask::Save;
    let requests = harness.step(ask);
    let [Request::Save { owner, .. }] = requests.as_ref() else { panic!("unchecked save after jobs: {requests:?}") };
    let owner = *owner;
    assert_eq!(
        harness.domain.runs.get(skein_lib::Id::from_token(run)).expect("the run lives").lease,
        crate::lease::Lease::Workspace { owner }
    );
    assert_eq!(
        harness.step(Event::Delivered { owner, delivery: delivered() }).as_ref(),
        [returned(50, Returned::Delivered(receipts()))]
    );
    assert_eq!(
        harness.domain.runs.get(skein_lib::Id::from_token(run)).expect("the run lives").lease,
        crate::lease::Lease::Free
    );
    // A save is no result and does not exempt a later delivery from checking.
    let requests = harness.step(mid_ask(conversation, 4));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = requests.as_ref() else {
        panic!("delivery checks after saving: {requests:?}")
    };
    let owner = *owner;
    let requests = harness.step(Event::Checked { owner, ran: ran(1, b"unfinished") });
    let [Request::Return { result: Returned::ChecksFailed { .. }, .. }] = requests.as_ref() else {
        panic!("the checked delivery is refused: {requests:?}")
    };
}

#[test]
fn saving_without_a_delivery_grant_is_refused_at_admission() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = charter();
    policy.grants.save = true;
    let requests = harness.start_workspace(1, policy, Some(workspace()));
    assert_eq!(answered(requests), (1, Answer::Refused(Refusal::Invalid(Invalid::MalformedCharter))));
}

#[test]
fn a_save_waits_for_jobs_and_a_finishing_change_still_checks_every_repository() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = coding();
    policy.grants.deliver = policy.outcome.change.clone();
    policy.grants.save = true;
    let (_, conversation) = harness.coding_policy(1, 100, policy);
    let mut ask = mid_ask(conversation, 3);
    let Event::Delegated { ask: tool, .. } = &mut ask else { unreachable!("delegated") };
    *tool = Ask::Save;
    step(&mut harness.domain, &harness.env, ask, &mut harness.out);
    assert_eq!(harness.out.pop(), Some(Request::CancelJobs { peer: PEER }));
    assert!(harness.out.is_empty(), "saving waits until every job settles");
    let requests = harness.step(Event::JobsCancelled { conversation });
    let [Request::Save { owner, .. }] = requests.as_ref() else { panic!("save after settlement: {requests:?}") };
    let owner = *owner;
    drop(harness.step(Event::Delivered { owner, delivery: delivered() }));
    let owner = harness.land(conversation, 8);
    let requests = harness.step(pass_checks(owner));
    let [Request::Check { .. }, Request::Checking { .. }] = requests.as_ref() else {
        panic!("second repository checks: {requests:?}")
    };
    let requests = harness.step(pass_checks(owner));
    let [Request::Deliver { .. }] = requests.as_ref() else { panic!("checked finish delivers: {requests:?}") };
    let requests = harness.step(Event::Delivered { owner, delivery: delivered() });
    let [Request::Return { result: Returned::Delivered(_), .. }, Request::Close { .. }] = requests.as_ref() else {
        panic!("checked finish closes: {requests:?}")
    };
}

#[test]
fn a_save_does_not_become_a_change_result_and_late_saves_are_cancelled() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = mid_report();
    policy.grants.save = true;
    let (run, conversation) = mid_running_policy(&mut harness, policy);
    let mut ask = mid_ask(conversation, 3);
    let Event::Delegated { ask: tool, .. } = &mut ask else { unreachable!("delegated") };
    *tool = Ask::Save;
    let requests = harness.step(ask);
    let [Request::Save { owner, .. }] = requests.as_ref() else { panic!("save") };
    let owner = *owner;
    drop(harness.step(Event::Cancel { run }));
    assert_eq!(
        harness.step(Event::Delivered { owner, delivery: delivered() }).as_ref(),
        [returned(50, Returned::Delivered(receipts()))]
    );
    let mut late = mid_ask(conversation, 4);
    let Event::Delegated { ask: tool, .. } = &mut late else { unreachable!("delegated") };
    *tool = Ask::Save;
    assert_eq!(harness.step(late).as_ref(), [returned(50, Returned::Cancelled)]);
    assert_eq!(
        answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))),
        (1, Answer::Failed { failure: RunFailure::Cancelled, spent: Spend::ZERO, turns: 0 })
    );
}

#[test]
fn save_is_cancelled_in_the_finishing_reserve() {
    let mut harness = Harness::new(LIMITS);
    let mut policy = mid_report();
    policy.grants.save = true;
    policy.budget.turns = 4;
    policy.reserve.turns = 2;
    let (_, main) = mid_running_policy(&mut harness, policy);
    drop(harness.step(Event::Used { conversation: main, spend: Spend { turns: 2, ..Spend::ZERO } }));
    let permit = crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0));
    let crate::CompletionPermit::Final { .. } = permit else { panic!("finishing reserve: {permit:?}") };
    let mut ask = mid_ask(main, 3);
    let Event::Delegated { ask: tool, .. } = &mut ask else { unreachable!("delegated") };
    *tool = Ask::Save;
    assert_eq!(harness.step(ask).as_ref(), [returned(50, Returned::Cancelled)]);
}

#[test]
fn saving_preserves_host_feedback_and_a_stale_branch_ends_the_run() {
    let refusal = crate::DeliveryRefusal::new(None, bytes(b"unfinished work refused")).expect("bounded refusal");
    let failure = crate::DeliveryFailure::new(0, crate::DeliveryReason::TimedOut);
    for (delivery, expected, stale) in [
        (Delivery::Nothing, Returned::Nothing, false),
        (Delivery::Refused(refusal.clone()), Returned::DeliveryRefused(refusal), false),
        (Delivery::Failed(failure), Returned::DeliveryFailed { failure }, false),
        (Delivery::Stale, Returned::Stale, true),
    ] {
        let mut harness = Harness::new(LIMITS);
        let mut policy = mid_report();
        policy.grants.save = true;
        let (_, conversation) = mid_running_policy(&mut harness, policy);
        let mut ask = mid_ask(conversation, 3);
        let Event::Delegated { ask: tool, .. } = &mut ask else { unreachable!("delegated") };
        *tool = Ask::Save;
        let requests = harness.step(ask);
        let [Request::Save { owner, .. }] = requests.as_ref() else { panic!("save submitted: {requests:?}") };
        let owner = *owner;
        let requests = harness.step(Event::Delivered { owner, delivery });
        if stale {
            assert_eq!(requests.as_ref(), [returned(50, expected), Request::Close { peer: PEER }]);
            assert_eq!(
                answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))).1,
                failed(RunFailure::Stale, Spend::ZERO)
            );
        } else {
            assert_eq!(requests.as_ref(), [returned(50, expected)]);
            drop(harness.step(finish(conversation, 51, report())));
            assert_eq!(
                answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))).1,
                Answer::Accepted { outcome: report(), spent: Spend::ZERO, turns: 0 }
            );
        }
    }
}
