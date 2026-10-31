use super::*;

#[test]
fn a_run_whose_main_conversation_fails_answers_with_its_fault_and_what_it_spent() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    assert!(harness.step(Event::Used { conversation, spend: spend(100) }).is_empty(), "within the budget");
    let end = End::Fault(Fault::Provider);
    let requests = harness.step(ended(conversation, end, spend(100)));
    assert_eq!(answered(requests), (1, failed(Failure::Model(Fault::Provider), spend(100))));
    assert_eq!(harness.domain.next_deadline(), None);
    harness.domain.reclaim();
    assert_eq!((harness.domain.runs(), harness.domain.conversations()), (0, 0));
}

#[test]
fn a_run_whose_main_conversation_runs_out_of_budget_fails_for_budget() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    let end = End::Budget(Exhausted::Turns);
    let requests = harness.step(ended(conversation, end, Spend::ZERO));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget { exhausted: Exhausted::Turns, used: Some(0), bound: Some(u64::from(BUDGET.turns)) },
                Spend::ZERO
            )
        )
    );
}

#[test]
fn a_main_conversation_refused_at_its_entrance_refuses_the_run() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.admit(1);
    let requests = harness.step(ended(conversation, End::Busy, Spend::ZERO));
    assert_eq!(answered(requests), (1, Answer::Refused(Refusal::Busy)));
    let (_, conversation) = harness.admit(2);
    let requests = harness.step(ended(conversation, End::Invalid, Spend::ZERO));
    assert_eq!(answered(requests), (2, Answer::Refused(Refusal::Invalid(Invalid::Conversation))));
}

#[test]
fn an_llm_that_stops_without_finishing_is_nudged_until_its_nudges_run_out() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    let peer = PEER;
    for nudge in 1..=LIMITS.nudges {
        assert!(harness.step(Event::Used { conversation, spend: spend(5) }).is_empty(), "within the budget");
        let text = crate::prompt::nudge(crate::Mode::Finite, Stop::EndTurn, nudge, LIMITS.nudges);
        assert_eq!(&*harness.step(end_turn(conversation)), &[Request::Say { peer, text }]);
    }
    assert!(harness.step(Event::Used { conversation, spend: spend(5) }).is_empty(), "within the budget");
    assert_eq!(&*harness.step(end_turn(conversation)), &[Request::Close { peer }]);
    assert_eq!(harness.domain.next_deadline(), None);
    // A turn that won the race with the close is spent all the same.
    assert!(harness.step(Event::Used { conversation, spend: spend(5) }).is_empty(), "winding down");
    let total = Spend {
        units: 20,
        turns: 4,
        input: Some(20),
        output: Some(40),
        cache_read: Some(0),
        cache_write: Some(0),
        reasoning: None,
    };
    let requests = harness.step(ended(conversation, End::Closed, total));
    let unfinished = Failure::Policy(Policy::Unfinished { nudges: LIMITS.nudges, rejected: 0 });
    assert_eq!(answered(requests), (1, failed(unfinished, total)));
}

#[test]
fn an_llm_whose_last_stop_shows_a_fault_fails_the_run_with_it() {
    let faults =
        [(Stop::MaxTokens, Fault::Truncated), (Stop::Refusal, Fault::Declined), (Stop::NoCalls, Fault::Malformed)];
    for (stop, fault) in faults {
        let mut harness = Harness::new(Limits { nudges: 0, ..LIMITS });
        let (_, conversation) = harness.running(1, 100);
        drop(harness.step(Event::Yielded { conversation, stop, text: bytes(b"") }));
        let requests = harness.step(ended(conversation, End::Closed, Spend::ZERO));
        assert_eq!(answered(requests), (1, failed(Failure::Model(fault), Spend::ZERO)));
    }
}

#[test]
fn an_llm_with_no_input_or_output_left_is_not_nudged() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend) }));
    assert_eq!(&*harness.step(end_turn(conversation)), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, spend(BUDGET.spend)));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget { exhausted: Exhausted::Spend, used: Some(BUDGET.spend), bound: Some(BUDGET.spend) },
                spend(BUDGET.spend)
            )
        )
    );
}

#[test]
fn an_llm_with_no_turn_left_is_not_nudged() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    let all = Spend { turns: BUDGET.turns, ..spend(5) };
    assert!(harness.step(Event::Used { conversation, spend: all }).is_empty(), "at the budget");
    let yielded = Event::Yielded { conversation, stop: Stop::EndTurn, text: bytes(b"") };
    assert_eq!(&*harness.step(yielded), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Closed, all));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget {
                    exhausted: Exhausted::Turns,
                    used: Some(u64::from(BUDGET.turns)),
                    bound: Some(u64::from(BUDGET.turns))
                },
                all
            )
        )
    );
}

#[test]
fn spending_past_the_budget_keeps_admitted_turns_stable_until_yield() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    assert!(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend) }).is_empty(), "at the budget");
    assert!(harness.step(Event::Used { conversation, spend: spend(1) }).is_empty(), "past it: main keeps its turn");
    assert!(harness.step(Event::Used { conversation, spend: spend(1) }).is_empty());
    assert_eq!(&*harness.step(end_turn(conversation)), &[Request::Close { peer: PEER }]);
    let total = Spend {
        units: BUDGET.spend + 2,
        turns: 3,
        input: Some(BUDGET.spend + 2),
        output: Some(30),
        cache_read: Some(0),
        cache_write: Some(0),
        reasoning: None,
    };
    let requests = harness.step(ended(conversation, End::Closed, total));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget {
                    exhausted: Exhausted::Spend,
                    used: Some(BUDGET.spend + 2),
                    bound: Some(BUDGET.spend)
                },
                total
            )
        )
    );

    // Or it yields, and is closed then.
    let (_, conversation) = harness.running(2, 101);
    drop(harness.step(Event::Used { conversation, spend: spend(BUDGET.spend + 1) }));
    assert_eq!(&*harness.step(end_turn(conversation)), &[Request::Close { peer: Token::new(101) }], "no nudge past it");
}

#[test]
fn what_an_end_counts_beyond_the_turns_used_is_charged() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    drop(harness.step(Event::Used { conversation, spend: spend(10) }));
    let end = End::Fault(Fault::Completion {
        failure: crate::CompletionFailure::Limit { which: crate::Limit::History, bound: 65_536 },
        evidence: crate::CompletionEvidence::Unsent,
        model: 0,
        attempts: 0,
        report: crate::Report::none(),
    });
    let requests = harness.step(ended(conversation, end, spend(30)));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Model(Fault::Completion {
                    failure: crate::CompletionFailure::Limit { which: crate::Limit::History, bound: 65_536 },
                    evidence: crate::CompletionEvidence::Unsent,
                    model: 0,
                    attempts: 0,
                    report: crate::Report::none(),
                }),
                Spend { units: 10, ..spend(30) }
            )
        )
    );
}

#[test]
fn the_deadline_closes_main_and_fails_the_run_for_time() {
    let mut harness = Harness::new(LIMITS);
    let (_, conversation) = harness.running(1, 100);
    harness.after(BUDGET.time);
    assert_eq!(&*harness.fire(), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(conversation, End::Budget(Exhausted::Time), Spend::ZERO));
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
