use super::*;

#[test]
fn a_cancel_while_preparing_stops_the_run_once_its_look_has_ended() {
    let mut harness = Harness::new(LIMITS);
    let run = harness.prepare(1);
    assert!(harness.step(Event::Cancel { run }).is_empty(), "the look is in flight");
    assert_eq!(harness.domain.next_deadline(), None);
    assert!(harness.step(Event::Cancel { run }).is_empty(), "stopping: the ending is decided");
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    assert_eq!(answered(requests), (1, cancelled()));
    harness.domain.reclaim();
    assert_eq!(
        (harness.domain.runs(), harness.domain.conversations()),
        (0, 0),
        "main was never opened, and is gone too"
    );
}

#[test]
fn a_cancel_while_main_opens_closes_main_once_it_starts() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.admit(1);
    assert!(harness.step(Event::Cancel { run }).is_empty(), "main has no peer to close yet");
    assert!(harness.step(Event::Cancel { run }).is_empty(), "the ending is decided");
    let peer = PEER;
    assert_eq!(&*harness.step(Event::Started { conversation, peer }), &[Request::Close { peer }]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(answered(requests), (1, cancelled()));

    // Or main is refused, and the cancel is still how the run ends.
    let (run, conversation) = harness.admit(2);
    drop(harness.step(Event::Cancel { run }));
    let requests = harness.step(ended(conversation, End::Busy, Spend::ZERO));
    assert_eq!(answered(requests), (2, cancelled()));
}

#[test]
fn a_cancel_while_main_works_closes_it() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.running(1, 100);
    assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: PEER }]);
    // The conversation failed before it saw the close: the cancel still wins.
    let requests = harness.step(ended(conversation, End::Fault(Fault::Provider), Spend::ZERO));
    assert_eq!(answered(requests), (1, cancelled()));
}

#[test]
fn a_cancel_while_main_is_over_the_budget_closes_it() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.running(1, 100);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
    assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: PEER }]);
    let spent = spend(BUDGET.spend + 1);
    let requests = harness.step(ended(conversation, End::Budget(Exhausted::Spend), spent));
    assert_eq!(answered(requests), (1, failed(Failure::Cancelled, spent)));
}

#[test]
fn a_cancel_while_a_change_is_checked_stops_the_checks_once_main_withdraws_its_call() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: PEER }]);
    assert_eq!(&*harness.step(Event::Withdraw { conversation, call: Token::new(7) }), &[Request::Abort { owner }]);
    // The checks pass before the abort lands: nothing is pushed.
    assert_eq!(&*harness.step(pass_checks(owner)), &[returned(7, Returned::Cancelled)]);
    assert_eq!(harness.checks_ended, 1, "the losing abort still closes the check span");
    // A finish that crossed the close is cancelled too.
    let crossed = finish(conversation, 8, verdict(b"approve", Box::new([])));
    assert_eq!(&*harness.step(crossed), &[returned(8, Returned::Cancelled)]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(answered(requests), (1, cancelled()));
}

#[test]
fn a_push_that_lands_while_a_cancel_closes_main_wins_over_it() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    assert_eq!(&*harness.step(Event::Cancel { run }), &[Request::Close { peer: PEER }]);
    assert!(
        harness.step(Event::Withdraw { conversation, call: Token::new(7) }).is_empty(),
        "submitted delivery is never abandoned"
    );
    // The host terminal reports a landing while run cancellation settles.
    assert_eq!(
        &*harness.step(Event::Delivered { owner, delivery: delivered() }),
        &[returned(7, Returned::Delivered(receipts()))]
    );
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    let accepted = Answer::Accepted { outcome: Declared::Change(change()), spent: Spend::ZERO, turns: 0 };
    assert_eq!(answered(requests), (1, accepted));

    // Or the host operation times out while run cancellation settles.
    // Cancellation does not abandon that submitted operation or invent its result.
    let (run, conversation) = harness.coding(2, 101);
    let owner = harness.land(conversation, 8);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(Event::Cancel { run }));
    drop(harness.step(Event::Withdraw { conversation, call: Token::new(8) }));
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
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(answered(requests), (2, cancelled()));
}

#[test]
fn a_push_that_lands_after_the_deadline_wins_over_it() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    harness.after(BUDGET.time);
    // The run's deadline is main's expiry, and its call's deadline.
    assert_eq!(&*harness.fire(), &[Request::Close { peer: PEER }]);
    assert!(harness.fire().is_empty(), "call expiry retains the submitted host operation");
    // The push wins the race with its cancel.
    assert_eq!(
        &*harness.step(Event::Delivered { owner, delivery: delivered() }),
        &[returned(7, Returned::Delivered(receipts()))]
    );
    assert!(harness.step(Event::Withdraw { conversation, call: Token::new(7) }).is_empty(), "returned already");
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    let accepted = Answer::Accepted { outcome: Declared::Change(change()), spent: Spend::ZERO, turns: 0 };
    assert_eq!(answered(requests), (1, accepted));

    // Over the budget, too.
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
    let owner = harness.land(conversation, 8);
    drop(harness.step(pass_checks(owner)));
    drop(harness.step(pass_checks(owner)));
    harness.after(BUDGET.time);
    assert_eq!(&*harness.fire(), &[Request::Close { peer: PEER }]);
    assert!(harness.fire().is_empty(), "call expiry retains the submitted host operation");
    assert!(harness.step(Event::Withdraw { conversation, call: Token::new(8) }).is_empty(), "stopped already");
    assert_eq!(
        &*harness.step(Event::Delivered { owner, delivery: delivered() }),
        &[returned(8, Returned::Delivered(receipts()))]
    );
    let total = spend(BUDGET.spend + 1);
    let requests = harness.step(ended(conversation, End::Closed, total));
    assert_eq!(
        answered(requests),
        (1, Answer::Accepted { outcome: Declared::Change(change()), spent: total, turns: 0 })
    );
}

#[test]
fn checks_that_pass_once_the_run_winds_down_push_nothing() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.coding(1, 100);
    let owner = harness.land(conversation, 7);
    harness.after(BUDGET.time);
    assert_eq!(&*harness.fire(), &[Request::Close { peer: PEER }]);
    // Main's caller deadline equals the run's time budget and has passed;
    // this pre-submission check returns timeout without any host delivery.
    assert_eq!(&*harness.step(pass_checks(owner)), &[returned(7, Returned::TimedOut)]);
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget { exhausted: Exhausted::Time, used: Some(600_000), bound: Some(600_000) },
                Spend::ZERO
            )
        )
    );
}

#[test]
fn a_cancel_while_the_run_winds_down_changes_nothing() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.running(1, 100);
    harness.after(BUDGET.time);
    drop(harness.fire());
    assert!(harness.step(Event::Cancel { run }).is_empty(), "the ending is decided");
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget { exhausted: Exhausted::Time, used: Some(600_000), bound: Some(600_000) },
                Spend::ZERO
            )
        )
    );

    // Winding down to an accepted outcome, too.
    let (run, conversation) = harness.running(2, 101);
    drop(harness.step(finish(conversation, 7, verdict(b"approve", Box::new([])))));
    assert!(harness.step(Event::Cancel { run }).is_empty(), "the ending is decided");
    let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
    let accepted = Answer::Accepted { outcome: verdict(b"approve", Box::new([])), spent: Spend::ZERO, turns: 0 };
    assert_eq!(answered(requests), (2, accepted));
}

#[test]
fn a_cancel_of_a_run_that_has_answered_changes_nothing() {
    let mut harness = Harness::new(LIMITS);
    let (run, conversation) = harness.running(1, 100);
    drop(harness.step(ended(conversation, End::Fault(Fault::Provider), Spend::ZERO)));
    assert!(harness.step(Event::Cancel { run }).is_empty(), "answered, not reclaimed yet");
    harness.domain.reclaim();
    assert!(harness.step(Event::Cancel { run }).is_empty(), "answered and gone");
    // Its slot taken by another run, the old name still finds nothing.
    let (_, _) = harness.running(2, 101);
    assert!(harness.step(Event::Cancel { run }).is_empty(), "a stale name");
    assert_eq!(harness.domain.runs(), 1);
}

#[test]
fn a_finish_with_no_room_for_its_call_is_busy_and_the_run_goes_on() {
    let mut harness = Harness::new(Limits { calls: 1, ..LIMITS });
    let (_, first) = harness.coding(1, 100);
    let _: Token = harness.land(first, 7);
    let (_, second) = harness.coding(2, 101);
    let requests = harness.step(finish(second, 8, Declared::Change(change())));
    assert_eq!(&*requests, &[returned(8, Returned::Busy)]);
    assert_eq!(harness.domain.calls(), 1);
    assert_eq!(harness.step(end_turn(second)).len(), 1, "the run goes on: a nudge");
}
