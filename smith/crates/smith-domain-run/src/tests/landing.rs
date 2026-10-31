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
    let unfinished = Failure::Policy(Policy::Unfinished { nudges: LIMITS.nudges, rejected: 2 });
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
fn a_change_runs_each_repositorys_checks_then_is_pushed_and_accepted() {
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
fn a_change_that_fails_its_checks_or_its_push_goes_back_to_the_llm() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    let failing = Ran { exit: Exit::Code { code: 1 }, output: bytes(b"test parse ... FAILED"), cut: 12 };
    let requests = harness.step(Event::Checked { owner, ran: failing });
    let failing = Ran { exit: Exit::Code { code: 1 }, output: bytes(b"test parse ... FAILED"), cut: 12 };
    assert_eq!(&*requests, &[returned(7, Returned::ChecksFailed { repository: bytes(b"temper"), ran: failing })]);
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
    assert_eq!(answered(requests), (1, failed(Failure::Stale, Spend::ZERO)));
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
                Failure::Budget {
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

    // Past its deadline while it is pushed.
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

    // A refused one closes main, and the run fails for budget.
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
        (1, failed(Failure::Budget { exhausted: Exhausted::Time, used: None, bound: None }, Spend::ZERO))
    );
}

#[test]
fn nothing_to_push_is_specific_feedback_and_the_run_can_retry() {
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
fn failed_push_reason_and_diagnostics_return_to_the_finish_caller() {
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
fn reports_and_declared_failures_settle_once_without_checks_or_push() {
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
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = mid_running(&mut harness);
    let requests = harness.step(mid_ask(conversation, 3));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = requests.as_ref() else {
        panic!("discovered check runs: {requests:?}");
    };
    let owner = *owner;
    let failing = ran(1, b"draft failed");
    assert_eq!(
        harness.step(Event::Checked { owner, ran: ran(1, b"draft failed") }).as_ref(),
        [returned(50, Returned::ChecksFailed { repository: bytes(b"temper"), ran: failing })]
    );
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

#[test]
fn mid_report_landing_settles_before_an_interrupted_run_answers() {
    for interrupted in [false, true] {
        let mut harness = Harness::new(LIMITS);
        let (run, conversation) = mid_running(&mut harness);
        let owner = submit_mid(&mut harness, conversation);
        if interrupted {
            drop(harness.step(Event::Cancel { run }));
            assert!(harness.step(Event::Withdraw { conversation, call: Token::new(50) }).is_empty());
        }
        let requests = harness.step(Event::Delivered { owner, delivery: delivered() });
        assert_eq!(requests.as_ref(), &[returned(50, Returned::Delivered(receipts()))]);
        assert!(
            harness.step(Event::Delivered { owner, delivery: delivered() }).is_empty(),
            "stale duplicate is inert before reclaim"
        );
        if !interrupted {
            drop(harness.step(finish(conversation, 51, report())));
        }
        let answer = answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))).1;
        if interrupted {
            assert_eq!(answer, Answer::Failed { failure: Failure::Cancelled, spent: Spend::ZERO, turns: 0 });
        } else {
            assert_eq!(answer, Answer::Accepted { outcome: report(), spent: Spend::ZERO, turns: 0 });
        }
        harness.domain.reclaim();
        assert!(
            harness.step(Event::Delivered { owner, delivery: delivered() }).is_empty(),
            "stale generation is inert after reclaim"
        );
    }
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
fn time_and_spend_shutdown_keep_an_already_submitted_mid_landing() {
    for timed in [false, true] {
        let mut harness = Harness::new(LIMITS);
        let (_, conversation) = mid_running(&mut harness);
        let owner = submit_mid(&mut harness, conversation);
        let expected = if timed {
            harness.env.now = EXPIRY;
            for _ in 0_u32..2 {
                if harness.domain.is_due(harness.env.now) {
                    drop(harness.fire());
                }
            }
            Failure::Budget { exhausted: Exhausted::Time, used: Some(600_000), bound: Some(600_000) }
        } else {
            drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
            Failure::Budget { exhausted: Exhausted::Spend, used: Some(BUDGET.spend + 1), bound: Some(BUDGET.spend) }
        };
        let expected_spend = if timed { Spend::ZERO } else { spend(BUDGET.spend + 1) };
        let requests = harness.step(Event::Delivered { owner, delivery: delivered() });
        if timed {
            assert_eq!(requests.as_ref(), &[returned(50, Returned::Delivered(receipts()))]);
        } else {
            assert_eq!(
                requests.as_ref(),
                &[returned(50, Returned::Delivered(receipts())), Request::Close { peer: PEER },]
            );
        }
        // Closing the session waits for the host terminal; its one End
        // then reports the spend it already used, without another delivery.
        let answer = answered(harness.step(ended(conversation, End::Closed, expected_spend))).1;
        assert_eq!(answer, Answer::Failed { failure: expected, spent: expected_spend, turns: 0 });
    }
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
fn caller_only_stop_keeps_submitted_mid_delivery_and_ordinary_continuation() {
    for expired in [false, true] {
        for later_cancel in [false, true] {
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
            let requests = harness.step(Event::Checked { owner, ran: ran(0, b"passed") });
            let [Request::Deliver { deadline, .. }] = requests.as_ref() else {
                panic!("checks submit exactly one delivery")
            };
            assert_eq!(*deadline, caller_deadline);
            if expired {
                harness.env.now = caller_deadline;
                assert!(harness.domain.is_due(harness.env.now));
                assert!(harness.fire().is_empty(), "expiry retains the host terminal right");
            } else {
                assert!(harness.step(Event::Withdraw { conversation, call: Token::new(50) }).is_empty());
            }
            assert_eq!(
                harness.step(Event::Delivered { owner, delivery: delivered() }).as_ref(),
                &[returned(50, Returned::Delivered(receipts()))],
                "caller-only stop neither abandons the host nor closes main"
            );
            if later_cancel {
                assert_eq!(harness.step(Event::Cancel { run }).as_ref(), &[Request::Close { peer: PEER }]);
            } else {
                drop(harness.step(finish(conversation, 51, report())));
            }
            let answer = answered(harness.step(ended(conversation, End::Closed, Spend::ZERO))).1;
            assert_eq!(
                answer,
                if later_cancel {
                    Answer::Failed { failure: Failure::Cancelled, spent: Spend::ZERO, turns: 0 }
                } else {
                    Answer::Accepted { outcome: report(), spent: Spend::ZERO, turns: 0 }
                }
            );
        }
    }
}
